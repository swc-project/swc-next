//! Source and AST formatting for Markdown.
//!
//! The native host supplies [`BuiltinFormatters`] by default. Pass an
//! [`EmbeddedDispatcher`] to choose how to format fenced content.

use swc_next_formatter_core::PrintOptions;
use swc_next_formatter_host::{EmbeddedDispatcher, EmbeddedError, EmbeddedOptions, FormatSession};
pub use swc_next_formatter_markdown::{MarkdownDocumentProducer, ProseWrap, SyntaxOptions};
use swc_next_markdown_parser::Document;

use crate::{BuiltinFormatters, restore_bom};

/// Options for whole-document Markdown formatting.
#[derive(Clone, Copy, Debug, Default)]
pub struct FormatOptions {
    /// Layout shared by the Markdown document and its fences.
    pub layout: PrintOptions,
    /// Markdown prose and punctuation.
    pub syntax: SyntaxOptions,
    /// Whether to format code fences and how deeply they may nest.
    pub embedded: EmbeddedOptions,
}

impl FormatOptions {
    fn builtin_formatters(self) -> BuiltinFormatters {
        BuiltinFormatters {
            markdown: self.syntax,
            ..Default::default()
        }
    }
}

/// Parse and format a Markdown document.
pub fn format(source: &str, options: FormatOptions) -> Result<String, EmbeddedError> {
    format_with_dispatcher(source, options, &options.builtin_formatters())
}

/// Format a previously parsed document, preserving its file-leading BOM.
pub fn format_ast(ast: &Document<'_>, options: FormatOptions) -> Result<String, EmbeddedError> {
    format_ast_with_dispatcher(ast, options, &options.builtin_formatters())
}

/// Format source using the supplied host for all embedded documents.
pub fn format_with_dispatcher(
    source: &str,
    options: FormatOptions,
    dispatcher: &dyn EmbeddedDispatcher,
) -> Result<String, EmbeddedError> {
    let session = FormatSession::new(options.layout);
    let context = session.context_with_embedded(dispatcher, options.embedded);
    let doc = context.produce(&MarkdownDocumentProducer, source, &options.syntax)?;
    Ok(restore_bom(source, doc.print(options.layout)))
}

/// Format a parsed document using the supplied host, preserving its file-leading BOM.
pub fn format_ast_with_dispatcher(
    ast: &Document<'_>,
    options: FormatOptions,
    dispatcher: &dyn EmbeddedDispatcher,
) -> Result<String, EmbeddedError> {
    let session = FormatSession::new(options.layout);
    let context = session.context_with_embedded(dispatcher, options.embedded);
    let doc = swc_next_formatter_markdown::ast_to_doc(ast, context, options.syntax)?;
    Ok(restore_bom(ast.source(), doc.print(options.layout)))
}
