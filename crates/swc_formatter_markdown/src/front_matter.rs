use swc_next_formatter_core::{LabelId, LineEnding, PrintOptions, Writer};
use swc_next_formatter_host::{EmbeddedError, EmbeddedOutcome, InputKind};
use swc_next_markdown_parser::{Block, FrontMatter};

use crate::{EmbeddedBlock, Formatter, blocks::write_source_lines};

impl<'a> Formatter<'a, '_, '_> {
    pub(super) fn front_matter(
        &mut self,
        block: &Block,
        matter: &FrontMatter,
        writer: &mut Writer<'a>,
    ) -> Result<(), EmbeddedError> {
        let delimiter = self.text(matter.opening);
        let explicit = self.document.source[matter.opening.end..block.lines[0].content.end].trim();
        let language = match (explicit, delimiter) {
            ("", "---") => "yaml",
            ("", "+++") => "toml",
            (name, _) => name,
        };
        let raw = &self.document.source[block.span.start..matter.closing.end];
        if !matches!(language, "yaml" | "toml") {
            write_source_lines(writer, raw);
            return Ok(());
        }
        let body = || block.logical_text(self.document.source, matter.body);
        let closing = self.text(matter.closing);
        if let Some(slots) = &mut self.slots {
            let label = LabelId(u32::try_from(slots.len()).map_err(EmbeddedError::new)?);
            slots.push(EmbeddedBlock::FrontMatter {
                start: block.span.start,
                end: matter.closing.end,
                language: language.to_owned(),
                opening: format!("{delimiter}{explicit}"),
                closing: closing.to_owned(),
                body: body(),
            });
            writer.label(label, |writer| write_source_lines(writer, raw));
            return Ok(());
        }
        match self
            .context
            .embed_with(language, InputKind::Document, |arena| {
                arena.alloc_str(&body())
            })? {
            EmbeddedOutcome::Formatted(doc) => {
                // Front matter starts at the document root. Let the guest own
                // all its line endings, including those inside a final scalar.
                let formatted = doc.print(PrintOptions {
                    line_ending: LineEnding::Lf,
                    ..self.context.print_options()
                });
                writer.mark_as_root(|writer| {
                    writer.text(delimiter);
                    writer.text(explicit);
                    writer.hard_line();
                    write_source_lines(writer, &formatted);
                    if !formatted.is_empty() && !formatted.ends_with('\n') {
                        writer.literal_line();
                    }
                    writer.text(closing);
                });
            }
            EmbeddedOutcome::PreserveOriginal(_) => write_source_lines(writer, raw),
        }
        Ok(())
    }
}
