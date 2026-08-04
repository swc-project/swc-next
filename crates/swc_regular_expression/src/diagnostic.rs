use crate::span::Span;

#[derive(Debug)]
pub struct Diagnostic;

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid regular expression")
    }
}

impl Diagnostic {
    pub fn error<T>(_: T) -> Self {
        Self
    }

    pub fn with_label(self, _: Span) -> Self {
        self
    }

    pub fn with_labels(self, _: impl IntoIterator<Item = Span>) -> Self {
        self
    }

    pub fn with_help<T>(self, _: T) -> Self {
        self
    }
}

pub type Result<T> = std::result::Result<T, Diagnostic>;
