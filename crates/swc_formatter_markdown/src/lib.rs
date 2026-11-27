//! Source-backed Markdown formatting and embedded code fence composition.
//!
//! Experimental and unstable. Use the Markdown parser's source spans to retain
//! constructs that require verbatim output.

#![deny(clippy::all)]
#![deny(unused)]

mod blocks;
mod extensions;
mod fences;
mod front_matter;
mod inline;
mod link_text;
mod tables;

use swc_next_formatter_core::{Doc, Writer};
use swc_next_formatter_host::{DocumentProducer, EmbeddedError, FormatContext};
use swc_next_markdown_parser::{Block, BlockKind, Document, RecoveryReason, Span, parse};

/// Layout of soft breaks within Markdown prose.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProseWrap {
    /// Keep the existing line breaks within paragraphs.
    #[default]
    Preserve,
    /// Wrap paragraphs to the document printer's line width.
    Always,
    /// Avoid wrapping prose while retaining meaningful source breaks.
    Never,
}

/// Markdown options independent of shared document layout.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SyntaxOptions {
    /// How to wrap paragraphs and other prose.
    pub prose_wrap: ProseWrap,
    /// Prefer single quotes for link titles when both quote styles fit.
    pub single_quote: bool,
}

/// Metadata for a fenced code block whose fallback has a caller-owned label.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FenceSlot {
    /// Start of the source block as a UTF-8 byte offset.
    pub start: usize,
    /// Exclusive end of the source block as a UTF-8 byte offset.
    pub end: usize,
    /// Language name from the info string.
    pub language: String,
    /// Remaining info string, if present.
    pub meta: Option<String>,
    /// Source spelling to print when the decoded info would reparse with different fields.
    pub source_info: Option<String>,
    /// Logical code with LF line endings and no structural terminal newline.
    pub body: String,
}

/// A Markdown block whose surrounding syntax belongs to the parent formatter.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EmbeddedBlock {
    Fence(FenceSlot),
    FrontMatter {
        start: usize,
        end: usize,
        language: String,
        opening: String,
        closing: String,
        /// Complete guest document between the delimiter lines, with LF line endings.
        body: String,
    },
}

/// Produces Markdown documents within a formatting session.
#[derive(Debug, Default)]
pub struct MarkdownDocumentProducer;

impl DocumentProducer for MarkdownDocumentProducer {
    type Options = SyntaxOptions;
    type Error = EmbeddedError;

    fn produce<'a>(
        &self,
        context: FormatContext<'a, '_>,
        source: &'a str,
        options: &SyntaxOptions,
    ) -> Result<Doc<'a>, EmbeddedError> {
        ast_to_doc(&parse(source), context, *options)
    }
}

/// Formats a parsed Markdown document with the context's embedded dispatcher.
pub fn ast_to_doc<'a>(
    document: &Document<'a>,
    context: FormatContext<'a, '_>,
    options: SyntaxOptions,
) -> Result<Doc<'a>, EmbeddedError> {
    format_document(document, context, options, None)
}

/// Produces a fallback Doc and its replaceable blocks in source and label order.
pub fn plugin_document<'a>(
    document: &Document<'a>,
    context: FormatContext<'a, '_>,
    options: SyntaxOptions,
) -> Result<(Doc<'a>, Vec<EmbeddedBlock>), EmbeddedError> {
    let mut slots = Vec::new();
    let doc = format_document(document, context, options, Some(&mut slots))?;
    Ok((doc, slots))
}

fn format_document<'a, 'doc>(
    document: &'doc Document<'a>,
    context: FormatContext<'a, '_>,
    options: SyntaxOptions,
    slots: Option<&'doc mut Vec<EmbeddedBlock>>,
) -> Result<Doc<'a>, EmbeddedError> {
    let mut formatter = Formatter {
        document,
        context,
        options,
        slots,
        prefix_columns: 0,
        lists_aligned: true,
        original_item_widths: true,
    };
    formatter.document()
}

struct Formatter<'a, 'services, 'slots> {
    document: &'slots Document<'a>,
    context: FormatContext<'a, 'services>,
    options: SyntaxOptions,
    slots: Option<&'slots mut Vec<EmbeddedBlock>>,
    prefix_columns: usize,
    lists_aligned: bool,
    original_item_widths: bool,
}

impl<'a> Formatter<'a, '_, '_> {
    fn document(&mut self) -> Result<Doc<'a>, EmbeddedError> {
        let mut writer = Writer::new(self.context.allocator());
        if matches!(
            self.document.blocks.as_slice(),
            [Block {
                kind: BlockKind::Recovery(RecoveryReason::InputSize),
                ..
            }]
        ) {
            let source = self
                .document
                .source
                .strip_prefix('\u{feff}')
                .unwrap_or(self.document.source);
            blocks::write_source_lines(&mut writer, source);
            return Ok(writer.finish());
        }
        if self.blocks(
            &self.document.blocks,
            blocks::Container::Document,
            &mut writer,
        )? {
            writer.literal_line();
        }
        Ok(writer.finish())
    }

    fn text(&self, span: Span) -> &'a str {
        &self.document.source[span.start..span.end]
    }

    fn aligned<T>(
        &mut self,
        prefix: &'a str,
        writer: &mut Writer<'a>,
        render: impl FnOnce(&mut Self, &mut Writer<'a>) -> T,
    ) -> T {
        if prefix.is_empty() {
            return render(self, writer);
        }
        self.prefix_columns += prefix.len();
        let result = writer.align_text(prefix, |writer| render(self, writer));
        self.prefix_columns -= prefix.len();
        result
    }
}

#[cfg(test)]
mod tests;
