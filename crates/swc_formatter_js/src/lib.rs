//! Source-aware JavaScript and TypeScript document production for the SWC Next AST.
//!
//! Experimental and unstable. APIs, document formats, and formatting output may
//! change without notice.
//!
//! [`EcmaDocumentProducer`] parses source into documents for a host session.
//! [`ast_to_doc()`] builds a document from an existing program, and
//! [`format_to_doc_buffer()`] returns an encoded document. Printing and source
//! formatting convenience APIs belong to `swc_next_formatter`.
//! Formatting options live in `options`. Syntax support is added incrementally;
//! unsupported programs return [`FormatError::Unsupported`].
//!
//! Recursive document construction allows 128 active formatter scopes, or 32
//! with debug assertions enabled, to protect default thread stacks. Preparing
//! and emitting one node share a scope; retained frames can add scopes of their
//! own. Exceeding this budget returns [`FormatError::Unsupported`].

#![deny(clippy::all)]
#![deny(unused)]

mod document;
mod formatter;
mod options;
mod producer;

use std::{error::Error, fmt};

use swc_next_ecma_ast::{Diagnostic, NodeKind, Span};

pub use options::{
    ArrowParentheses, FormatOptions, IndentStyle, IndentWidth, InvalidIndentWidth, LineEnding,
    QuoteProperties, QuoteStyle, Semicolons, SyntaxOptions, TrailingCommas,
};
pub use producer::{EcmaDocumentProducer, ast_to_doc, format_to_doc_buffer};

/// Prettier version used as the output oracle for [`FormatOptions::prettier`].
pub const PRETTIER_COMPATIBILITY_VERSION: &str = "3.10.1";

pub use swc_next_formatter_core::DOCUMENT_FORMAT_VERSION;

#[derive(Debug)]
pub enum FormatError {
    Parse(Vec<Diagnostic>),
    Unsupported {
        construct: &'static str,
        kind: Option<NodeKind>,
        span: Span,
    },
}

impl FormatError {
    pub(crate) const fn unsupported(
        construct: &'static str,
        kind: Option<NodeKind>,
        span: Span,
    ) -> Self {
        Self::Unsupported {
            construct,
            kind,
            span,
        }
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(diagnostics) => write!(
                formatter,
                "cannot format source with {} parse error(s)",
                diagnostics.len()
            ),
            Self::Unsupported {
                construct, span, ..
            } => write!(
                formatter,
                "formatting {construct} at {}..{} is not implemented",
                span.start, span.end
            ),
        }
    }
}

impl Error for FormatError {}
