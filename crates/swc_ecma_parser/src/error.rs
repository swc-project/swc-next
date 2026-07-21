use std::{error, fmt};

use swc_experimental_ecma_ast::Span;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Hint,
    Info,
}

impl Default for Severity {
    fn default() -> Self {
        Self::Error
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Label {
    pub span: Span,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
    pub labels: Vec<Label>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    pub message: String,
    pub span: Span,
    pub(crate) reported: bool,
}

impl Error {
    pub(crate) fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span,
            reported: false,
        }
    }

    pub(crate) fn reported(span: Span, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span,
            reported: true,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}..{}",
            self.message, self.span.start, self.span.end
        )
    }
}

impl error::Error for Error {}
