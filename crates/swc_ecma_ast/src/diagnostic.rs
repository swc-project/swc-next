use std::borrow::Cow;

use crate::Span;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Severity {
    Error = 0,
    Warning = 1,
    Hint = 2,
    Info = 3,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Hint => "hint",
            Self::Info => "info",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Label<'a> {
    pub span: Span,
    pub message: Cow<'a, str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic<'a> {
    pub severity: Severity,
    pub message: Cow<'a, str>,
    pub span: Span,
    pub help: Option<Cow<'a, str>>,
    pub labels: Vec<Label<'a>>,
}

impl<'a> Diagnostic<'a> {
    pub fn error(message: &'a str, span: Span) -> Self {
        Self {
            severity: Severity::Error,
            message: Cow::Borrowed(message),
            span,
            help: None,
            labels: Vec::new(),
        }
    }
}
