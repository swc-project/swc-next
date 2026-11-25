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

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{AstCtx, Diagnostic, NodeKind, Severity, Span};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_formatter_host::FormatSession;

pub use options::{
    ArrowParentheses, FormatOptions, IndentStyle, IndentWidth, InvalidIndentWidth, LineEnding,
    QuoteProperties, QuoteStyle, Semicolons, SyntaxOptions, TrailingCommas,
};
pub use producer::{EcmaDocumentProducer, ast_to_doc};

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

fn parse_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    mut parser_options: ParserOptions,
) -> Result<AstCtx<'a>, FormatError> {
    parser_options.comments = CommentMode::Flat;
    parser_options.preserve_parens = true;
    let parsed = Parser::init(allocator, source, parser_options, NoTokenParserConfig).parse();
    if parsed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(FormatError::Parse(parsed.diagnostics));
    }
    Ok(parsed.ast)
}

/// Parse source and encode an unprinted document for the SWCFMT Prettier adapter.
///
/// The binary protocol is internal and must match the JavaScript decoder version.
/// All borrowed source and document storage is released before returning.
pub fn format_to_doc_buffer(
    source: &str,
    parser_options: ParserOptions,
    options: FormatOptions,
) -> Result<Vec<u8>, FormatError> {
    let session = FormatSession::new(options.into());
    let producer = EcmaDocumentProducer::new(parser_options);
    Ok(session
        .produce(&producer, source, &options.into())?
        .encode())
}
