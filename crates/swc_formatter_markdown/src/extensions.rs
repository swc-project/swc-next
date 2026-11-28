use swc_next_formatter_core::Writer;
use swc_next_formatter_host::EmbeddedError;
use swc_next_markdown_parser::{Block, BlockKind, Dialect, SourceLine, Span};

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
                formatter.inlines(first, formatter.document.inlines(*nodes), wrap, writer);
            } else {
                formatter.blocks(blocks, Container::Footnote, writer)?;
            }
            Ok(())
        })
    }

    // Padding needed to keep a body line containing only "$$" past the closing
    // fence's three-column limit, for each possible position in a tab stop.
    pub(super) fn math_padding(&self, body: &[SourceLine]) -> [usize; 4] {
        let mut padding = [0; 4];
        for (width, line, indent) in self.math_body_runs(body) {
            if width == 2 && indent < 4 {
                for (column, pad) in padding.iter_mut().enumerate() {
                    if (SourceLine { column, ..line }).indent(self.document.source) < 4 {
                        *pad = (*pad).max(4 - indent);
                    }
                }
            }
        }
        padding
    }

    pub(super) fn display_math(
        &self,
        block: &Block,
        meta: Option<Span>,
        body: &[SourceLine],
        continuation: &'a str,
        writer: &mut Writer<'a>,
    ) {
        let extended = self.document.dialect() == Dialect::Extended;
        let (width, padding) = if extended {
            // Keep a body line containing only "$$" indented past a closing fence.
            let required = self.math_padding(body)[self.prefix_columns % 4];
            let padding = if required == 0 {
                continuation
            } else {
                let indent = block.lines[0]
                    .indent(self.document.source)
                    .max(required)
                    .min(3);
                &"   "[..indent]
            };
            (2, padding)
        } else {
            // A standalone run inside the body must not become the closing marker.
            let width = self
                .math_body_runs(body)
                .filter(|&(_, _, indent)| indent <= 3)
                .map(|(width, _, _)| width)
                .max()
                .unwrap_or(1)
                + 1;
            (width, continuation)
        };
        let marker = if width == 2 {
            "$$"
        } else {
            self.context.allocator().alloc_str(&"$".repeat(width))
        };

        writer.text(padding);
        writer.align_text(padding, |writer| {
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
                    if extended {
                        writer.literal_line_with_indent();
                    } else {
                        writer.hard_line();
                    }
                }
            }
            writer.text(marker);
        });
    }

    fn math_body_runs<'b>(
        &'b self,
        body: &'b [SourceLine],
    ) -> impl Iterator<Item = (usize, SourceLine, usize)> + 'b {
        body.iter().filter_map(|line| {
            let text = self.text(line.content).trim_end_matches([' ', '\t']);
            let run = text.trim_start_matches([' ', '\t']);
            let indent = text.len() - run.len() + usize::from(line.virtual_spaces);
            (!run.is_empty() && run.bytes().all(|b| b == b'$')).then_some((
                run.len(),
                *line,
                indent,
            ))
        })
    }
}
