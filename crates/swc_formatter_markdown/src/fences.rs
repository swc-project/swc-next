use std::borrow::Cow;

use swc_next_formatter_core::{Doc, LabelId, LineEnding, Writer};
use swc_next_formatter_host::{EmbeddedError, EmbeddedOutcome, InputKind};
use swc_next_markdown_parser::{Block, Fence};

use crate::{EmbeddedBlock, FenceSlot, Formatter, link_text};

#[derive(Clone, Copy)]
struct FenceInfo<'source> {
    head: &'source str,
    meta: Option<&'source str>,
    marker: u8,
}

impl<'a> FenceInfo<'a> {
    fn new(head: &'a str, meta: Option<&'a str>) -> Self {
        // A backtick fence cannot contain backticks in its printed info string.
        let marker = if head.contains('`') || meta.is_some_and(|text| text.contains('`')) {
            b'~'
        } else {
            b'`'
        };
        Self { head, meta, marker }
    }
}

impl<'a> Formatter<'a, '_, '_> {
    pub(super) fn fence(
        &mut self,
        block: &Block,
        fence: &Fence,
        writer: &mut Writer<'a>,
    ) -> Result<(), EmbeddedError> {
        let mut keep_spelling = false;
        let mut decode = |span, is_language| match link_text::decoded(self.text(span)).value {
            Cow::Borrowed(text) => text,
            Cow::Owned(text) => {
                // Fields are split and trimmed before character references and escapes are decoded.
                let padding = [' ', '\t'];
                let changes_field = if is_language {
                    text.contains(padding)
                } else {
                    text.starts_with(padding) || text.ends_with(padding)
                };
                keep_spelling |= changes_field
                    || text.contains(['\r', '\n'])
                    || matches!(link_text::decoded(&text).value, Cow::Owned(_));
                self.context.allocator().alloc_str(&text)
            }
        };
        let language = fence.lang.map_or("", |span| decode(span, true));
        let meta = fence.meta.map(|span| decode(span, false));
        let source_info = fence
            .info
            .filter(|_| keep_spelling)
            .map(|span| self.text(span));
        let info = match source_info {
            Some(raw) => FenceInfo::new(raw, None),
            None => FenceInfo::new(language, meta),
        };
        let code = &fence.code.value;
        let minimum = maximum_run(code, info.marker) + 1;

        if language.is_empty() {
            write_fence(writer, info, minimum, |writer| {
                write_code_lines(writer, code, true)
            });
            return Ok(());
        }

        if let Some(slots) = self.slots.as_mut() {
            let id = LabelId(u32::try_from(slots.len()).map_err(EmbeddedError::new)?);
            slots.push(EmbeddedBlock::Fence(FenceSlot {
                start: block.span.start,
                end: block.span.end,
                language: language.to_string(),
                meta: meta.map(str::to_string),
                source_info: source_info.map(str::to_owned),
                body: code.clone(),
            }));
            writer.label(id, |writer| {
                write_fence(writer, info, minimum, |writer| {
                    write_code_lines(writer, code, true)
                });
            });
            return Ok(());
        }

        match self
            .context
            .embed_with(language, InputKind::Document, |arena| arena.alloc_str(code))?
        {
            EmbeddedOutcome::Formatted(guest) => {
                let guest = guest.strip_trailing_hardline().replace_end_of_line();
                let minimum = self.guest_maximum_run(&guest, info.marker) + 1;
                writer.mark_as_root(|writer| {
                    write_fence(writer, info, minimum, |writer| {
                        writer.write_doc(&guest);
                        writer.hard_line();
                    });
                });
            }
            EmbeddedOutcome::PreserveOriginal(_) => {
                write_fence(writer, info, minimum, |writer| {
                    write_code_lines(writer, code, true)
                });
            }
        }
        Ok(())
    }

    fn guest_maximum_run(&self, guest: &Doc<'a>, marker: u8) -> usize {
        let mut options = self.context.print_options();
        options.line_ending = LineEnding::Lf;
        let prefix = self
            .context
            .allocator()
            .alloc_str(&" ".repeat(self.prefix_columns));
        let projected = |width| {
            let mut probe = Writer::new(self.context.allocator());
            probe.text(&*prefix);
            probe.align_text(prefix, |writer| {
                writer.mark_as_root(|writer| writer.write_doc(guest));
            });
            let printed = probe.finish().print(swc_next_formatter_core::PrintOptions {
                line_width: width,
                ..options
            });
            maximum_run(&printed, marker)
        };
        projected(options.line_width).max(projected(u16::MAX))
    }
}

fn write_fence<'a>(
    writer: &mut Writer<'a>,
    info: FenceInfo<'a>,
    minimum: usize,
    body: impl FnOnce(&mut Writer<'a>),
) {
    let delimiter = char::from(info.marker).to_string().repeat(minimum.max(3));
    writer.text_copied(&delimiter);
    writer.text(info.head);
    if let Some(meta) = info.meta {
        writer.space();
        writer.text(meta);
    }
    writer.hard_line();
    body(writer);
    writer.text(delimiter);
}

// Empty logical lines have no container padding of their own. Preserve
// trailing whitespace that actually belongs to an unformatted code line.
pub(super) fn write_code_lines(writer: &mut Writer<'_>, code: &str, trailing_line: bool) {
    let mut lines = code.split('\n').peekable();
    while let Some(line) = lines.next() {
        writer.text_copied(line);
        if !trailing_line && lines.peek().is_none() {
            break;
        }
        if line.is_empty() {
            writer.hard_line();
        } else {
            writer.literal_line_with_indent();
        }
    }
}

pub(super) fn unlabelled_fence(code: &str, writer: &mut Writer<'_>) {
    let info = FenceInfo::new("", None);
    write_fence(writer, info, maximum_run(code, info.marker) + 1, |writer| {
        write_code_lines(writer, code, true)
    });
}

fn maximum_run(source: &str, marker: u8) -> usize {
    let mut bytes = source.as_bytes();
    let mut longest = 0;
    while let Some(start) = memchr::memchr(marker, bytes) {
        let run = &bytes[start..];
        let width = run.iter().take_while(|&&byte| byte == marker).count();
        longest = longest.max(width);
        bytes = &run[width..];
    }
    longest
}
