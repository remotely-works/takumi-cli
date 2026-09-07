use std::fmt;

use takumi_core::resources::font::FontError;
use takumi_html::HtmlError;
use takumi_pdf::PdfError;

/// Everything that can stop a render, with the exit code it maps to.
#[derive(Debug)]
pub enum CliError {
    Usage(String),
    Io(std::io::Error),
    Html(HtmlError),
    Font(FontError),
    Lang(takumi_core::Error),
    Pdf(PdfError),
}

impl CliError {
    /// 2 for bad input to the cli, 3 when a registered font is missing glyphs
    /// (the caller can fix that with `--fonts-dir`), 1 for everything else.
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Usage(_) | Self::Lang(_) => 2,
            Self::Pdf(PdfError::MissingGlyphs(_)) => 3,
            _ => 1,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => f.write_str(message),
            Self::Io(error) => write!(f, "{error}"),
            Self::Html(error) => write!(f, "{error}"),
            Self::Font(error) => write!(f, "{error}"),
            Self::Lang(error) => write!(f, "{error}"),
            Self::Pdf(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<HtmlError> for CliError {
    fn from(error: HtmlError) -> Self {
        Self::Html(error)
    }
}

impl From<FontError> for CliError {
    fn from(error: FontError) -> Self {
        Self::Font(error)
    }
}

impl From<takumi_core::Error> for CliError {
    fn from(error: takumi_core::Error) -> Self {
        Self::Lang(error)
    }
}

impl From<PdfError> for CliError {
    fn from(error: PdfError) -> Self {
        Self::Pdf(error)
    }
}
