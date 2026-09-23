//! Source-aware JavaScript and TypeScript formatting for the SWC Next AST.
//!
//! [`format()`] parses source, while [`format_ast()`] accepts an existing program.
//! Formatting options live in `options`. Syntax support is added incrementally;
//! unsupported programs return [`FormatError::Unsupported`].

#![deny(clippy::all)]
#![deny(unused)]

mod options;

use std::{error::Error, fmt};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Diagnostic, GetSpan, NodeKind, Severity, Span};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParserOptions, Parser};

pub use options::{
    ArrowParentheses, FormatOptions, IndentStyle, IndentWidth, InvalidIndentWidth, LineEnding,
    QuoteProperties, QuoteStyle, Semicolons, TrailingCommas,
};

/// Prettier version used as the output oracle for [`FormatOptions::prettier`].
pub const PRETTIER_COMPATIBILITY_VERSION: &str = "3.9.9";

#[derive(Debug)]
pub enum FormatError {
    Parse(Vec<Diagnostic<'static>>),
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
    mut parser_options: ParserOptions,
    options: FormatOptions,
) -> Result<String, FormatError> {
    let allocator = Allocator::new();
    parser_options.comments = CommentMode::Flat;
    parser_options.preserve_parens = true;
    let parsed = Parser::init(&allocator, source, parser_options, NoTokenParserConfig).parse();
    if parsed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(FormatError::Parse(parsed.diagnostics));
    }
    format_ast(&parsed.ast, options)
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
pub fn format_ast(ast: &Ast<'_>, _options: FormatOptions) -> Result<String, FormatError> {
    let program = ast.root_program();
    if program.body(ast).is_empty()
        && program.directives(ast).is_empty()
        && program.hashbang(ast).is_none()
        && ast.comments().is_empty()
    {
        return Ok(if ast.source().starts_with('\u{feff}') {
            "\u{feff}".to_string()
        } else {
            String::new()
        });
    }
    Err(FormatError::unsupported(
        "program",
        Some(NodeKind::Program),
        program.span(ast),
    ))
}
