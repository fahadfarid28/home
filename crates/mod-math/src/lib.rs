include!(".dylo/spec.rs");
include!(".dylo/support.rs");

#[cfg(feature = "impl")]
use noteyre::BsForResults;

#[cfg(feature = "impl")]
#[derive(Default)]
struct ModImpl {}

/// The mode for LaTeX/MathML rendering
pub enum MathMode {
    Inline,
    Block,
}

/// A simple plugin that renders math/LaTeX-ish markup to MathML
#[dylo::export]
impl Mod for ModImpl {
    /// Render math markup with pulldown-latex
    fn render_math(
        &self,
        input: &str,
        mode: MathMode,
        w: &mut dyn std::io::Write,
    ) -> noteyre::Result<()> {
        let storage = pulldown_latex::Storage::new();
        let parser = pulldown_latex::Parser::new(input, &storage);
        let mut config = pulldown_latex::RenderConfig::default();
        config.display_mode = match mode {
            MathMode::Inline => pulldown_latex::config::DisplayMode::Inline,
            MathMode::Block => pulldown_latex::config::DisplayMode::Block,
        };

        let mut rendered = String::new();
        pulldown_latex::push_mathml(&mut rendered, parser, config).bs()?;
        w.write_all(rendered.as_bytes())?;

        Ok(())
    }
}
