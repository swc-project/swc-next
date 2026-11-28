use swc_next_formatter_core::Writer;
use swc_next_formatter_host::EmbeddedError;
use swc_next_markdown_parser::{Block, BlockKind, SourceLine, Span};

use crate::{Formatter, ProseWrap, blocks::Container};

impl<'a> Formatter<'a, '_, '_> {
    pub(super) fn footnote(
        &mut self,
        label: Span,
        blocks: &[Block],
        writer: &mut Writer<'a>,
    ) -> Result<(), EmbeddedError> {
        writer.text("[^");
        writer.text(self.text(label));
        writer.text("]:");
        let mut children = blocks
            .iter()
            .filter(|b| !matches!(b.kind, BlockKind::Blank));
        let Some(first) = children.next() else {
            return Ok(());
        };
        writer.space();
        let wrap = self.options.prose_wrap;
        self.aligned("    ", writer, |formatter, writer| {
            if let BlockKind::Paragraph(nodes) = &first.kind
                && children.next().is_none()
                && (wrap == ProseWrap::Never
                    || wrap == ProseWrap::Preserve && first.lines.len() == 1)
            {
                formatter.inlines(first, nodes, wrap, writer);
            } else {
                formatter.blocks(blocks, Container::Footnote, writer)?;
            }
            Ok(())
        })
    }

    pub(super) fn display_math(
        &self,
        meta: Option<Span>,
        body: &[SourceLine],
        writer: &mut Writer<'a>,
    ) {
        // A standalone run inside the body must not become the closing marker.
        let width = body.iter().fold(2, |width, line| {
            let text = self.text(line.content).trim_end_matches([' ', '\t']);
            let run = text.trim_start_matches([' ', '\t']);
            let indent = text.len() - run.len() + usize::from(line.virtual_spaces);
            if indent <= 3 && run.bytes().all(|b| b == b'$') {
                width.max(run.len() + 1)
            } else {
                width
            }
        });
        let marker = if width == 2 {
            "$$"
        } else {
            self.context.allocator().alloc_str(&"$".repeat(width))
        };
        writer.text(marker);
        if let Some(meta) = meta {
            writer.space();
            writer.text(self.text(meta));
        }
        writer.hard_line();
        if body.len() > 1
            || body.first().is_some_and(|l| {
                !self
                    .text(l.content)
                    .trim_end_matches([' ', '\t'])
                    .is_empty()
            })
        {
            for line in body {
                writer.text(&"    "[..usize::from(line.virtual_spaces)]);
                writer.text(self.text(line.content));
                writer.hard_line();
            }
        }
        writer.text(marker);
    }
}
