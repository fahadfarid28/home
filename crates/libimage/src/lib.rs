use autotrait::autotrait;
// use eyre::Context;
use image_types::{ICodec, IntrinsicPixels};
use std::{io::Write, time::Instant};

use image::{DynamicImage, ImageDecoder, Rgb, Rgba};
use jpegxl_rs::encode::EncoderFrame;
use jxl_oxide::JxlImage;
use rgb::FromSlice;

struct ModImpl;

pub fn load() -> &'static dyn Mod {
    static MOD: ModImpl = ModImpl;
    &MOD
}

pub type Result<T, E = Report> = std::result::Result<T, E>;

/////////// mini-eyre starts
#[derive(Debug)]
pub struct Report {
    error: String,
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

// Allow converting any error type that implements Display into our Report
impl<E: std::error::Error> From<E> for Report {
    fn from(error: E) -> Self {
        Report {
            error: error.to_string(),
        }
    }
}

#[macro_export]
macro_rules! eyre {
        ($($tt:tt)*) => {
            Report {
                error: format!($($tt)*),
            }
        };
    }

use std::fmt::Display;

pub trait Context<T, E> {
    fn wrap_err<D>(self, context: D) -> Result<T>
    where
        D: Display;
}

impl<T, E> Context<T, E> for std::result::Result<T, E>
where
    E: Display,
{
    fn wrap_err<D>(self, context: D) -> Result<T>
    where
        D: Display,
    {
        self.map_err(|e| Report {
            error: format!("{context}: {e}"),
        })
    }
}

/////////// mini-eyre ends

#[autotrait]
impl Mod for ModImpl {
    fn transcode(
        &self,
        input: &[u8],
        ifmt: ICodec,
        ofmt: ICodec,
        target_width: Option<IntrinsicPixels>,
    ) -> Result<Vec<u8>> {
        let start_load = Instant::now();

        // Load the image from the input bytes
        let mut img = match ifmt {
            ICodec::PNG => image::load_from_memory_with_format(input, image::ImageFormat::Png)
                .map_err(|e| eyre!("{}", e))?,
            ICodec::JPG => image::load_from_memory_with_format(input, image::ImageFormat::Jpeg)
                .map_err(|e| eyre!("{}", e))?,
            ICodec::WEBP => image::load_from_memory_with_format(input, image::ImageFormat::WebP)
                .map_err(|e| eyre!("{}", e))?,
            ICodec::AVIF => image::load_from_memory_with_format(input, image::ImageFormat::Avif)
                .map_err(|e| eyre!("{}", e))?,
            ICodec::JXL => {
                let image = JxlImage::builder()
                    .read(input)
                    .map_err(|e| eyre!("jxl decoding error: {e}"))?;
                let fb = image
                    .render_frame(0)
                    .map_err(|e| eyre!("jxl rendering error: {e}"))?
                    .image();
                match fb.channels() {
                    3 => DynamicImage::from(
                        image::ImageBuffer::<Rgb<f32>, Vec<f32>>::from_raw(
                            fb.width() as u32,
                            fb.height() as u32,
                            fb.buf().to_vec(),
                        )
                        .ok_or_else(|| {
                            eyre!("failed to create ImageBuffer from jxl frame (RGB)")
                        })?,
                    ),
                    4 => DynamicImage::from(
                        image::ImageBuffer::<Rgba<f32>, Vec<f32>>::from_raw(
                            fb.width() as u32,
                            fb.height() as u32,
                            fb.buf().to_vec(),
                        )
                        .ok_or_else(|| {
                            eyre!("failed to create ImageBuffer from jxl frame (RGBA)")
                        })?,
                    ),
                    _ => {
                        unimplemented!(
                            "unsupported number of channels in jxl image: {}",
                            fb.channels()
                        )
                    }
                }
            }
            ICodec::HEIC => {
                let temp_heic = tempfile::NamedTempFile::new()
                    .map_err(|e| eyre!("{}", e))
                    .and_then(|mut f| f.write_all(input).map_err(|e| eyre!("{}", e)).map(|_| f))
                    .map_err(|e| {
                        eyre!("failed to create or write temporary file for HEIC input: {e}")
                    })?;
                let temp_png = tempfile::NamedTempFile::new()
                    .map_err(|e| eyre!("failed to create temporary file for HEIC output: {e}"))?;
                let temp_png_path = temp_png
                    .path()
                    .to_str()
                    .ok_or_else(|| eyre!("failed to get temporary png path as string"))?;

                let status = std::process::Command::new("magick")
                    .arg(temp_heic.path())
                    .arg(format!("png:{temp_png_path}"))
                    .status()
                    .map_err(|e| eyre!("failed to run imagemagick convert command: {e}"))?;

                if !status.success() {
                    return Err(eyre!("imagemagick convert failed with status: {}", status));
                }

                let png_data = fs_err::read(temp_png_path)
                    .map_err(|e| eyre!("failed to read temporary PNG output file: {e}"))?;

                image::load_from_memory_with_format(png_data.as_slice(), image::ImageFormat::Png)
                    .map_err(|e| eyre!("failed to load temporary PNG output into image: {e}"))?
            }
        };

        let duration_load = start_load.elapsed();

        let start_resize = Instant::now();
        if let Some(target_width) = target_width {
            // The image::imageops::resize() function preserves aspect ratio while scaling
            // to the maximum size that fits within the given width/height bounds. So we
            // can just pass the original image height - the output will maintain aspect
            // ratio while ensuring width == target_width.
            img = img.resize(
                target_width.into_inner(),
                img.height(),
                image::imageops::FilterType::Lanczos3,
            );
        }
        let duration_resize = start_resize.elapsed();

        let start_transcode = Instant::now();

        // Encode the image into the output format
        let vec = match ofmt {
            ICodec::AVIF => {
                let encoder = ravif::Encoder::new()
                    .with_quality(85.0)
                    .with_alpha_quality(85.0)
                    .with_num_threads(Some(num_cpus::get()))
                    .with_speed(4); // 3 is _really slow_ (15 seconds on brat!)

                let res = if img.color().has_alpha() {
                    // not as, because it might be a non-u8 subpixel format
                    let rgba = img.to_rgba8();
                    let img = ravif::Img::new(
                        rgba.as_raw().as_rgba(),
                        img.width() as _,
                        img.height() as _,
                    );
                    encoder
                        .encode_rgba(img)
                        .map_err(|e| eyre!("ravif_error: {e}"))?
                } else {
                    let rgb = img.to_rgb8();
                    let img =
                        ravif::Img::new(rgb.as_raw().as_rgb(), img.width() as _, img.height() as _);
                    encoder
                        .encode_rgb(img)
                        .map_err(|e| eyre!("ravif error: {e}"))?
                };

                res.avif_file
            }
            ICodec::WEBP => {
                // the WebP encoder only supports RGBA
                let img = img.to_rgba8();
                let img = DynamicImage::from(img);
                webp::Encoder::from_image(&img)
                    .map_err(|e| eyre!("webp encoder error: {e}"))?
                    .encode(82.0)
                    .to_vec()
            }
            ICodec::PNG => {
                use image::ImageEncoder as _;
                let mut bytes: Vec<u8> = Vec::new();
                let img = img.to_rgba8(); // sometimes we get Rgb32F (out of a JPEG-XL) and PNG does _not_ like that.
                let img = DynamicImage::from(img);
                image::codecs::png::PngEncoder::new(&mut bytes)
                    .write_image(
                        img.as_bytes(),
                        img.width(),
                        img.height(),
                        img.color().into(),
                    )
                    .map_err(|e| eyre!("png encoding error: {e}"))?;
                bytes
            }
            ICodec::JXL => {
                let runner = jpegxl_rs::ThreadsRunner::default();

                let mut encoder = jpegxl_rs::encoder_builder()
                    .parallel_runner(&runner)
                    .quality(2.8) // that's distance, actually (lower is better)
                    .speed(jpegxl_rs::encode::EncoderSpeed::Squirrel) // effort, 7
                    .build()
                    .map_err(|e| eyre!("jpegxl encoder build error: {e}"))?;

                // Handle RGB and RGBA cases separately
                if img.color().has_alpha() {
                    let rgba = img.to_rgba8();
                    encoder.has_alpha = true;
                    let frame = EncoderFrame::new(rgba.as_raw()).num_channels(4);
                    encoder
                        .encode_frame::<_, u8>(&frame, img.width(), img.height())
                        .map_err(|e| eyre!("jpegxl rgba frame encoding error: {e}"))?
                        .data
                } else {
                    let rgb = img.to_rgb8();
                    encoder.has_alpha = false;
                    let frame = EncoderFrame::new(rgb.as_raw()).num_channels(3);
                    encoder
                        .encode_frame::<_, u8>(&frame, img.width(), img.height())
                        .map_err(|e| eyre!("jpegxl rgb frame encoding error: {e}"))?
                        .data
                }
            }
            _ => {
                return Err(eyre!("unsupported image format: {:?}", ofmt));
            }
        };

        let duration_transcode = start_transcode.elapsed();
        tracing::info!(
            "\x1b[36m{:?}\x1b[0m => \x1b[36m{:?}\x1b[0m, load: \x1b[33m{:?}\x1b[0m, resize: \x1b[33m{:?}\x1b[0m, transcode: \x1b[33m{:?}\x1b[0m, total: \x1b[33m{:?}\x1b[0m",
            ifmt,
            ofmt,
            duration_load,
            duration_resize,
            duration_transcode,
            duration_load + duration_resize + duration_transcode
        );

        Ok(vec)
    }

    fn dimensions(&self, input: &[u8], ifmt: ICodec) -> Result<(IntrinsicPixels, IntrinsicPixels)> {
        let input = std::io::Cursor::new(input);

        let (width, height) = match ifmt {
            ICodec::PNG => {
                let decoder = image::codecs::png::PngDecoder::new(input)
                    .map_err(|e| eyre!("failed to create PNG decoder: {e}"))?;
                decoder.dimensions()
            }
            ICodec::JPG => {
                let decoder = image::codecs::jpeg::JpegDecoder::new(input)
                    .map_err(|e| eyre!("failed to create JPG decoder: {e}"))?;
                decoder.dimensions()
            }
            ICodec::WEBP => {
                let decoder = image::codecs::webp::WebPDecoder::new(input)
                    .map_err(|e| eyre!("failed to create WEBP decoder: {e}"))?;
                decoder.dimensions()
            }
            ICodec::AVIF => {
                let decoder = image::codecs::avif::AvifDecoder::new(input)
                    .map_err(|e| eyre!("failed to create AVIF decoder: {e}"))?;
                decoder.dimensions()
            }
            ICodec::JXL => {
                let image = JxlImage::builder()
                    .read(input)
                    .map_err(|e| eyre!("jxl decoding error: {e}"))?;
                (image.width(), image.height())
            }
            ICodec::HEIC => {
                // Using ImageMagick is probably too slow/heavy just for dimensions.
                // A dedicated HEIC dimensions reader would be better, but for now,
                // mark as unsupported.
                return Err(eyre!("heic dimensions: unsupported :("));
            }
        };
        Ok((IntrinsicPixels::from(width), IntrinsicPixels::from(height)))
    }
}
