//! Source-aware JavaScript and TypeScript formatting for the SWC Next AST.
//!
//! [`format()`] parses source, while [`format_ast()`] accepts an existing program.
//! [`EcmaDocumentProducer`] builds documents for a language-independent host session.
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
pub use producer::EcmaDocumentProducer;

/// Prettier version used as the output oracle for [`FormatOptions::prettier`].
pub const PRETTIER_COMPATIBILITY_VERSION: &str = "3.9.9";

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

/// Parse and format source text.
///
/// Comment collection and parenthesis preservation are always enabled because
/// the formatter needs both to preserve annotation boundaries.
pub fn format(
    source: &str,
    parser_options: ParserOptions,
    options: FormatOptions,
) -> Result<String, FormatError> {
    let session = FormatSession::new(options.into());
    let producer = EcmaDocumentProducer::new(parser_options);
    let document = session.produce(&producer, source, &options.into())?;
    Ok(restore_bom(source, document.print(session.print_options())))
}

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

/// Format an already parsed program AST.
///
/// The AST must have a program root, as produced by [`Parser::parse`]. Expression
/// and statement fragment roots are not supported.
///
/// The caller must provide an AST parsed with [`CommentMode::Flat`] and
/// `preserve_parens: true`, retaining its original source and complete comment
/// list. These prerequisites are not checked; missing comments or parentheses
/// can lose annotations or change which expression they apply to.
/// Use [`format()`] to enable both parsing requirements automatically.
///
/// # Panics
///
/// Panics if the AST has no root or its root is not a program.
pub fn format_ast(ast: &AstCtx<'_>, options: FormatOptions) -> Result<String, FormatError> {
    let session = FormatSession::new(options.into());
    let document = producer::ast_to_doc(ast, session.context(), options.into())?;
    Ok(restore_bom(
        ast.source(),
        document.print(session.print_options()),
    ))
}

fn restore_bom(source: &str, mut output: String) -> String {
    if source.starts_with('\u{feff}') {
        output.insert(0, '\u{feff}');
    }
    output
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
