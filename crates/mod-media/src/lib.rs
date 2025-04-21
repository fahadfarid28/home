include!(".dylo/spec.rs");
include!(".dylo/support.rs");

use config::WebConfig;
use conflux::{InputPathRef, Media, RevisionView};
use image::LogicalPixels;

#[cfg(feature = "impl")]
mod impls;

#[cfg(feature = "impl")]
#[derive(Default)]
struct ModImpl {}

pub struct MediaMarkupOpts<'a> {
    pub path: &'a InputPathRef,
    pub media: &'a Media,
    pub rv: &'a dyn RevisionView,

    pub id: Option<&'a str>,
    pub title: Option<&'a str>,
    pub alt: Option<&'a str>,

    // these override the media's original width/height and specify CSS pixel dimensions
    pub width: Option<LogicalPixels>,
    pub height: Option<LogicalPixels>,

    pub class: Option<&'a str>,
    pub web: WebConfig,
}

#[dylo::export]
impl Mod for ModImpl {
    /// Generate HTML markup for a `Media`
    fn media_html_markup(&self, opts: MediaMarkupOpts<'_>) -> noteyre::Result<String> {
        impls::media_markup(opts)
    }
}
