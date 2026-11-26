use std::borrow::Cow;

use swc_next_formatter_core::{Writer, measure_text_width};
use swc_next_markdown_parser::{Alignment, Block, Table};

use crate::{Formatter, ProseWrap};

struct Cell<'a> {
    text: Cow<'a, str>,
    width: usize,
}

impl<'a> Formatter<'a, '_, '_> {
    pub(super) fn table(&self, block: &Block, table: &Table, writer: &mut Writer<'a>) {
        let mut widths = vec![3; table.header.len()];
        let mut rows = std::iter::once(&table.header)
            .chain(&table.rows)
            .map(|row| {
                if widths.len() < row.len() {
                    widths.resize(row.len(), 3);
                }
                row.iter()
                    .zip(&mut widths)
                    .map(|(cell, widest)| {
                        let text = self.inline_text(block, &cell.children);
                        let width = measure_text_width(&text);
                        *widest = (*widest).max(width);
                        Cell { text, width }
                    })
                    .collect::<Vec<_>>()
            });
        let Some(header) = rows.next() else {
            return;
        };
        let rows = rows.collect::<Vec<_>>();
        let aligned_width = widths.iter().map(|width| width + 3).sum::<usize>() + 1;
        let print_width = usize::from(self.context.print_options().line_width);
        let compact = self.options.prose_wrap == ProseWrap::Never
            && self.prefix_columns + aligned_width > print_width;
        let max_padding = if compact {
            0
        } else {
            widths.iter().copied().max().unwrap_or(0)
        };
        let padding = " ".repeat(1 + max_padding);
        let mut buffer = String::new();
        let mut write_row = |writer: &mut Writer<'a>, row: &[Cell<'_>]| {
            buffer.clear();
            buffer.push('|');
            for (column, cell) in row.iter().enumerate() {
                let extra = if compact {
                    0
                } else {
                    widths[column] - cell.width
                };
                let before = match table.alignments.get(column) {
                    Some(Alignment::Right) => extra,
                    Some(Alignment::Center) => extra / 2,
                    _ => 0,
                };
                buffer.push_str(&padding[..1 + before]);
                buffer.push_str(&cell.text);
                buffer.push_str(&padding[..1 + extra - before]);
                buffer.push('|');
            }
            writer.text_copied(&buffer);
        };
        write_row(writer, &header);
        writer.hard_line();
        writer.text("|");
        for (alignment, &width) in table.alignments.iter().zip(&widths) {
            let (left, right) = match alignment {
                Alignment::None => ('-', '-'),
                Alignment::Left => (':', '-'),
                Alignment::Right => ('-', ':'),
                Alignment::Center => (':', ':'),
            };
            let middle = if compact { 1 } else { width - 2 };
            writer.text(format!(" {left}{}{right} |", "-".repeat(middle)));
        }
        for row in rows {
            writer.hard_line();
            write_row(writer, &row);
        }
    }
}
