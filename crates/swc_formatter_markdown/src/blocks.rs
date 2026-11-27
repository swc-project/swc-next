use swc_next_formatter_core::Writer;
use swc_next_formatter_host::EmbeddedError;
use swc_next_markdown_parser::{
    Block, BlockExtension, BlockKind, Dialect, Heading, Ignore, List, ListItem, SourceLine, Span,
    Task,
};

use crate::{Formatter, ProseWrap};

#[derive(Clone, Copy)]
pub(super) enum Container {
    Document,
    Quote {
        per_block_indent: bool,
    },
    Footnote,
    Item {
        loose: bool,
        alternate_marker: bool,
        continuation_indent: &'static str,
        task: Option<Task>,
    },
}

#[derive(Clone, Copy)]
struct ListLayout {
    alternate_marker: bool,
    minimum_indent: usize,
}

impl<'a> Formatter<'a, '_, '_> {
    pub(super) fn blocks(
        &mut self,
        blocks: &[Block],
        container: Container,
        writer: &mut Writer<'a>,
    ) -> Result<bool, EmbeddedError> {
        let mut index = 0;
        let mut previous: Option<&Block> = None;
        let mut alternate_list = false;
        let mut quote_prefix = "";
        let mut ignore_next = false;
        // Range directives apply to document siblings. Single-block directives
        // also work inside lists and block quotes.
        let last_ignore_end = if matches!(container, Container::Document) {
            blocks
                .iter()
                .rposition(|block| matches!(block.kind, BlockKind::Ignore(Ignore::End)))
        } else {
            None
        };

        while let Some(block) = blocks.get(index) {
            if matches!(block.kind, BlockKind::Blank) {
                index += 1;
                continue;
            }

            let minimum_indent = if let BlockKind::List(list) = &block.kind {
                blocks[index + 1..]
                    .iter()
                    .find(|block| !matches!(block.kind, BlockKind::Blank))
                    .map_or(0, |next| {
                        minimum_list_indent_before(list, next, self.document.source)
                    })
            } else {
                0
            };
            if matches!(
                container,
                Container::Quote {
                    per_block_indent: true
                }
            ) {
                // Keep the sibling code on the same quote margin as its list.
                if !matches!(block.kind, BlockKind::IndentedCode(_)) || quote_prefix.is_empty() {
                    quote_prefix = self.quote_prefix(block, minimum_indent);
                }
            }
            alternate_list = match &block.kind {
                BlockKind::List(list) if ignore_next => list
                    .items
                    .first()
                    .is_some_and(|item| self.text(item.marker).ends_with(['*', ')'])),
                BlockKind::List(list) => {
                    !alternate_list && previous.is_some_and(|before| {
                        matches!(&before.kind, BlockKind::List(other) if other.ordered == list.ordered)
                    })
                }
                BlockKind::Html
                    if !ignore_next
                        && let Some(before) = previous
                        && let Some(opening) = lazy_html_opening(before, block) =>
                {
                    let blank = self.needs_blank_line(&blocks[..index], before, block, container);
                    self.lazy_html_block(block, opening, blank, container, quote_prefix, writer);
                    alternate_list = false;
                    previous = Some(block);
                    index += 1;
                    continue;
                }
                _ => false,
            };
            if let Some(before) = previous {
                writer.literal_line_with_indent();
                if !ignore_next && self.needs_blank_line(&blocks[..index], before, block, container)
                {
                    if !quote_prefix.is_empty() {
                        writer.text(&quote_prefix[..quote_prefix.len() - 1]);
                    }
                    writer.hard_line();
                }
            }

            if !ignore_next
                && matches!(block.kind, BlockKind::Ignore(Ignore::Start))
                && last_ignore_end.is_some_and(|end| end > index)
                && let Some(end) = blocks[index + 1..]
                    .iter()
                    .position(|block| matches!(block.kind, BlockKind::Ignore(Ignore::End)))
            {
                let last = index + 1 + end;
                write_source_lines(
                    writer,
                    &self.document.source[block.span.start..blocks[last].span.end],
                );
                previous = Some(&blocks[last]);
                index = last + 1;
                continue;
            }
            let list = ListLayout {
                alternate_marker: alternate_list,
                minimum_indent,
            };
            let render = |formatter: &mut Self, writer: &mut Writer<'a>| {
                if ignore_next {
                    formatter.raw_block(block, container, writer);
                    if matches!(container, Container::Quote { .. })
                        && formatter.options.prose_wrap == ProseWrap::Always
                        && matches!(block.kind, BlockKind::List(_))
                        && matches!(&blocks[index + 1..], [last] if matches!(last.kind, BlockKind::Blank))
                    {
                        writer.hard_line();
                        writer.trim();
                    }
                    Ok(())
                } else {
                    formatter.present_block(block, container, previous.is_none(), list, writer)
                }
            };
            if quote_prefix.is_empty() {
                render(self, writer)?;
            } else {
                writer.text(quote_prefix);
                self.aligned(quote_prefix, writer, render)?;
            }
            ignore_next = !ignore_next && matches!(block.kind, BlockKind::Ignore(Ignore::Next));
            previous = Some(block);
            index += 1;
        }
        Ok(previous.is_some())
    }

    fn present_block(
        &mut self,
        block: &Block,
        container: Container,
        first: bool,
        list: ListLayout,
        writer: &mut Writer<'a>,
    ) -> Result<(), EmbeddedError> {
        let alignment = match container {
            Container::Item {
                task: Some(task), ..
            } if first && matches!(block.kind, BlockKind::Paragraph(_)) => {
                writer.text(if task.checked { "[x] " } else { "[ ] " });
                "    "
            }
            Container::Item {
                continuation_indent,
                ..
            } if !first => {
                // Math handles this prefix together with its fence indentation.
                if matches!(
                    block.kind,
                    BlockKind::IndentedCode(_)
                        | BlockKind::Html
                        | BlockKind::Extension(BlockExtension::Math { .. })
                ) {
                    ""
                } else {
                    writer.text(continuation_indent);
                    continuation_indent
                }
            }
            _ => "",
        };
        self.aligned(alignment, writer, |formatter, writer| {
            if first && matches!(container, Container::Footnote) {
                // Indented code needs its own line to retain its code-block indentation.
                let must_break = matches!(block.kind, BlockKind::IndentedCode(_));
                writer.group(must_break, |writer| {
                    writer.soft_line();
                    formatter.block(block, container, first, list, writer)
                })
            } else {
                formatter.block(block, container, first, list, writer)
            }
        })
    }

    fn block(
        &mut self,
        block: &Block,
        container: Container,
        first: bool,
        list_layout: ListLayout,
        writer: &mut Writer<'a>,
    ) -> Result<(), EmbeddedError> {
        match &block.kind {
            BlockKind::Paragraph(inlines) => {
                self.inlines(block, inlines, self.options.prose_wrap, writer);
            }
            BlockKind::Heading(heading) => self.heading(block, heading, writer),
            BlockKind::ThematicBreak => writer.text(match container {
                Container::Document if first => "***",
                Container::Item {
                    alternate_marker: false,
                    ..
                } => "***",
                _ => "---",
            }),
            BlockKind::BlockQuote(children) => {
                if children
                    .iter()
                    .any(|block| !matches!(block.kind, BlockKind::Blank))
                {
                    let per_block_indent = self.document.dialect() == Dialect::Extended
                        && self.text(block.span).contains('\t')
                        && self.text(block.span).contains("$$")
                        && self.needs_variable_quote_indent(children);
                    let prefix = if per_block_indent { "" } else { "> " };
                    let quote = Container::Quote { per_block_indent };
                    writer.text(prefix);
                    self.aligned(prefix, writer, |formatter, writer| {
                        formatter.blocks(children, quote, writer)
                    })?;
                } else {
                    writer.text(">");
                }
            }
            BlockKind::List(list) => {
                let before_marker = match container {
                    Container::Item {
                        continuation_indent,
                        ..
                    } if !first => continuation_indent.len(),
                    _ => 0,
                };
                let parent_aligned = self.lists_aligned;
                self.lists_aligned =
                    parent_aligned && list_layout.minimum_indent == 0 && self.is_aligned_list(list);
                let result = self.list(list, list_layout, before_marker, writer);
                self.lists_aligned = parent_aligned;
                result?;
            }
            BlockKind::Fence(fence) => self.fence(block, fence, writer)?,
            BlockKind::IndentedCode(code) => {
                let spelling = self.text(block.span);
                if spelling.starts_with("    ") || spelling.starts_with('\t') {
                    writer.text("    ");
                    writer.align_text("    ", |writer| {
                        super::fences::write_code_lines(writer, &code.value, false);
                    });
                } else {
                    self.unlabelled_fence(&code.value, writer);
                }
            }
            BlockKind::Definition(definition) => self.definition(block, definition, writer),
            BlockKind::Table(table) => self.table(block, table, writer),
            BlockKind::Extension(BlockExtension::Footnote { label, blocks }) => {
                self.footnote(*label, blocks, writer)?;
            }
            BlockKind::Extension(BlockExtension::Math { meta, body, .. }) => {
                let continuation = match container {
                    Container::Item {
                        continuation_indent,
                        ..
                    } if !first => continuation_indent,
                    _ => "",
                };
                self.display_math(block, *meta, body, continuation, writer);
            }
            BlockKind::FrontMatter(matter) => self.front_matter(block, matter, writer)?,
            BlockKind::Html
            | BlockKind::Ignore(_)
            | BlockKind::Extension(_)
            | BlockKind::Recovery(_)
            | BlockKind::Raw => {
                self.raw_block(block, container, writer);
            }
            BlockKind::Blank => {}
        }
        Ok(())
    }

    fn heading(&self, block: &Block, heading: &Heading, writer: &mut Writer<'a>) {
        if heading.opening.is_none() {
            self.inlines(block, &heading.children, self.options.prose_wrap, writer);
            if let Some(underline) = heading.closing {
                writer.hard_line();
                writer.text(self.text(underline));
            }
            return;
        }

        writer.text("#".repeat(usize::from(heading.level)));
        let Some(last) = heading.children.last() else {
            return;
        };
        writer.space();
        self.inlines(block, &heading.children, ProseWrap::Never, writer);

        // A final, space-separated run of literal hashes would become the closer.
        if let Some(closing) = heading.closing
            && let Some(before_hash) = self.document.source[..last.span.end].strip_suffix('#')
            && before_hash.trim_end_matches('#').ends_with([' ', '\t'])
        {
            writer.space();
            writer.text(self.text(closing));
        }
    }

    fn needs_blank_line(
        &self,
        siblings: &[Block],
        previous: &Block,
        next: &Block,
        container: Container,
    ) -> bool {
        if matches!(
            (&previous.kind, &next.kind),
            (BlockKind::Definition(_), BlockKind::Definition(_))
        ) {
            return false;
        }
        let source_blank =
            || has_blank_line(&self.document.source[previous.span.end..next.span.start]);
        if matches!(previous.kind, BlockKind::Definition(_))
            && matches!(&next.kind, BlockKind::Heading(heading) if heading.opening.is_none())
        {
            // Adjacent definitions at the start of a Setext paragraph share
            // its original source range. Keep a single-line definition separate.
            let previous_definition = siblings
                .iter()
                .rev()
                .nth(1)
                .is_some_and(|block| matches!(block.kind, BlockKind::Definition(_)));
            return source_blank() || (previous.lines.len() == 1 && !previous_definition);
        }
        if matches!(
            previous.kind,
            BlockKind::Extension(BlockExtension::Liquid { .. })
        ) || matches!(
            next.kind,
            BlockKind::Extension(BlockExtension::Liquid { .. })
        ) {
            return source_blank();
        }
        if matches!(&next.kind, BlockKind::Html | BlockKind::Ignore(_))
            && matches!(
                &previous.kind,
                BlockKind::Html | BlockKind::Ignore(_) | BlockKind::Paragraph(_)
            )
        {
            return source_blank();
        }
        if let Container::Item { loose, .. } = container {
            if matches!(next.kind, BlockKind::List(_)) {
                return source_blank() && !matches!(previous.kind, BlockKind::List(_));
            }
            if !loose {
                return source_blank();
            }
        }
        true
    }

    fn raw_block(&self, block: &Block, container: Container, writer: &mut Writer<'a>) {
        if matches!(container, Container::Document) {
            write_source_lines(writer, self.text(block.span));
        } else {
            let content = block.logical_text(self.document.source, block.span);
            if matches!(block.kind, BlockKind::Html)
                && let Some(&line) = block.lines.first()
                && content.trim_start_matches(' ').starts_with('\t')
            {
                // A tab can move the opening tag to code indentation at its new column.
                let source = self.document.source;
                let indent = line.indent(source);
                let output = SourceLine {
                    column: self.prefix_columns,
                    ..line
                };
                if (line.lazy || output.indent(source) != indent)
                    && let Some(spaces) = "   ".get(..indent)
                {
                    writer.text(spaces);
                    write_source_lines(writer, content.trim_start_matches([' ', '\t']));
                    return;
                }
            }
            write_source_lines(writer, &content);
        }
    }

    #[cold]
    fn lazy_html_block(
        &mut self,
        block: &Block,
        opening: &SourceLine,
        blank: bool,
        container: Container,
        quote_prefix: &'static str,
        writer: &mut Writer<'a>,
    ) {
        let omit_prefix = opening.starts_lazy_html(self.document.source);
        let replay_prefix = !omit_prefix && self.original_item_widths;
        if replay_prefix {
            // An outer quote may be missing while the inner item is still indented.
            writer.literal_line();
        } else if omit_prefix {
            writer.dedent(Writer::literal_line_with_indent);
        } else {
            writer.literal_line_with_indent();
        }
        if blank {
            if let Some(marker) = quote_prefix.strip_suffix(' ') {
                writer.text(marker);
            }
            writer.hard_line();
        }
        if !replay_prefix && !omit_prefix {
            writer.text(quote_prefix);
        }
        self.aligned(quote_prefix, writer, |formatter, writer| {
            if replay_prefix {
                writer.text(formatter.text(opening.raw));
                let body = Span::new(opening.content.end, block.span.end);
                write_source_lines(writer, &block.logical_text(formatter.document.source, body));
            } else {
                formatter.raw_block(block, container, writer);
            }
        });
    }

    fn list(
        &mut self,
        list: &List,
        layout: ListLayout,
        before_marker: usize,
        writer: &mut Writer<'a>,
    ) -> Result<(), EmbeddedError> {
        let mut previous = None;
        let start = list.start.unwrap_or(1);
        let item_number = |item: &ListItem| {
            self.text(item.marker)
                .trim_end_matches(['.', ')'])
                .parse::<u64>()
                .ok()
        };
        let use_ones = list.ordered
            && list.items.get(1).and_then(item_number) == Some(1)
            && (start != 0 || list.items.get(2).and_then(item_number) == Some(1));
        let tab_width = usize::from(self.context.print_options().indent_width.value());
        for (index, item) in list.items.iter().enumerate() {
            if let Some((end, loose)) = previous {
                writer.hard_line();
                if loose || has_blank_line(&self.document.source[end..item.span.start]) {
                    writer.hard_line();
                }
            }
            let mut children = item
                .blocks
                .iter()
                .filter(|block| !matches!(block.kind, BlockKind::Blank));
            let first = children.next();
            let mut loose = false;
            let mut minimum_indent = layout.minimum_indent;
            let mut may_replay_html = first.is_some_and(|child| formats_children(&child.kind));
            if let Some(mut before) = first {
                for child in children {
                    loose = loose
                        || has_blank_line(&self.document.source[before.span.end..child.span.start]);
                    if matches!(child.kind, BlockKind::Html)
                        && let Some(line) = lazy_html_opening(before, child)
                    {
                        if line.starts_lazy_html(self.document.source) {
                            // The opening must fall short of the item's continuation indent.
                            minimum_indent =
                                minimum_indent.max(line.indent(self.document.source) + 1);
                        } else {
                            may_replay_html = true;
                        }
                    } else if formats_children(&child.kind) {
                        may_replay_html = true;
                    }
                    before = child;
                }
            }
            let number = || {
                if use_ones && index != 0 {
                    1
                } else {
                    start.saturating_add(index as u64).min(999_999_999)
                }
            };
            let (max_padding, math_starts_on_next_line) = match first.map(|block| &block.kind) {
                None | Some(BlockKind::IndentedCode(_)) => (1, false),
                Some(BlockKind::Extension(BlockExtension::Math { body, .. }))
                    if self.document.dialect() == Dialect::Extended =>
                {
                    let padding = self.math_padding(body);
                    let widest_marker = if list.ordered {
                        item.marker
                            .len()
                            .max(number().checked_ilog10().unwrap_or(0) as usize + 2)
                    } else {
                        1
                    };
                    if padding == [0; 4] {
                        (4, false)
                    } else if index + 1 == list.items.len()
                        && minimum_indent > 3 + widest_marker + 1
                        && let Some(indent) = inline_math_width(
                            padding,
                            self.prefix_columns,
                            minimum_indent,
                            widest_marker,
                        )
                    {
                        // An empty marker gets one separator column. Keep the opener
                        // on its marker line at a width that protects literal "$$" lines.
                        minimum_indent = indent;
                        (4, false)
                    } else {
                        (1, true)
                    }
                }
                _ => (4, false),
            };
            let marker = if list.ordered {
                let number = number();
                let delimiter = if layout.alternate_marker { ')' } else { '.' };
                // Only the final nonempty item can absorb a following code block.
                // Keep enough digits when three leading spaces, the delimiter,
                // and the permitted separator still fall short.
                let digits = if index + 1 == list.items.len() && first.is_some() {
                    minimum_indent
                        .saturating_sub(3 + 1 + max_padding)
                        .min(item.marker.len().saturating_sub(1))
                } else {
                    0
                };
                format!("{number:0digits$}{delimiter}")
            } else if layout.alternate_marker {
                "*".to_string()
            } else {
                "-".to_string()
            };
            let aligned_padding = if list.ordered && self.lists_aligned && tab_width != 0 {
                let extra = (tab_width - (marker.len() + 1) % tab_width) % tab_width;
                if extra <= 3 { 1 + extra } else { 1 }
            } else {
                1
            };
            let mut padding = minimum_indent
                .saturating_sub(marker.len())
                .max(aligned_padding)
                .min(max_padding);
            let leading = minimum_indent.saturating_sub(marker.len() + padding).min(3);
            let mut indent = leading + marker.len() + padding;
            if first.is_some_and(|child| matches!(child.kind, BlockKind::List(_))) {
                // Spaces before a nested marker on the opening line all contribute
                // to the outer item's width, including on its later HTML lines.
                let extra = tab_width
                    .saturating_sub(indent)
                    .min(3)
                    .min(max_padding - padding);
                indent += extra;
                padding += extra;
            }
            writer.text(&"   "[..leading]);
            writer.text(marker);
            if let Some(first) = first {
                if !math_starts_on_next_line {
                    writer.text(&"    "[..padding]);
                }
                // Parsed items fit three leading spaces, nine digits, a delimiter and four
                // spaces of padding. A caller-edited marker can require a wider prefix.
                const ITEM_INDENT: &str = "                 ";
                let prefix = ITEM_INDENT
                    .get(..indent)
                    .unwrap_or_else(|| self.context.allocator().alloc_str(&" ".repeat(indent)));
                let continuation = tab_width.saturating_sub(indent).min(3);
                let container = Container::Item {
                    loose,
                    alternate_marker: layout.alternate_marker,
                    continuation_indent: &"   "[..continuation],
                    task: item.task,
                };
                let changed_width = may_replay_html
                    && self.original_item_widths
                    && !self.item_width_matches(item, first, before_marker + indent);
                if changed_width {
                    self.original_item_widths = false;
                }
                let result = self.aligned(prefix, writer, |formatter, writer| {
                    if math_starts_on_next_line {
                        writer.hard_line();
                    }
                    formatter.blocks(&item.blocks, container, writer)
                });
                if changed_width {
                    self.original_item_widths = true;
                }
                result?;
            }
            previous = Some((item.span.end, loose));
        }
        Ok(())
    }

    fn needs_variable_quote_indent(&self, children: &[Block]) -> bool {
        let mut next_indent = 0;
        for child in children
            .iter()
            .rev()
            .filter(|b| !matches!(b.kind, BlockKind::Blank))
        {
            if self.quote_prefix(child, next_indent) != "> " {
                return true;
            }
            next_indent = indentation_before_code(child);
        }
        false
    }

    fn quote_prefix(&self, block: &Block, minimum_indent: usize) -> &'static str {
        if let BlockKind::List(list) = &block.kind
            && let Some(item) = list.items.last()
            && minimum_indent > 3 + item.marker.len() + 1
            && let Some(first) = item
                .blocks
                .iter()
                .find(|b| !matches!(b.kind, BlockKind::Blank))
            && let BlockKind::Extension(BlockExtension::Math { body, .. }) = &first.kind
        {
            // The quote margin changes the tab stops of the enclosed math body.
            let padding = self.math_padding(body);
            if let Some(leading) = (0..4).find(|leading| {
                let column = self.prefix_columns + leading + 2;
                inline_math_width(padding, column, minimum_indent, item.marker.len()).is_some()
            }) {
                return &"   > "[3 - leading..];
            }
        }
        "> "
    }

    fn item_width_matches(&self, item: &ListItem, first: &Block, width: usize) -> bool {
        let Some(body) = first.lines.first() else {
            return false;
        };
        let end = item
            .task
            .map_or(body.content.start, |task| task.marker.start);
        // A tab can straddle the original container boundary. Rebuild the
        // prefix when its column width cannot be read from the marker's bytes.
        body.raw.start <= item.marker.start
            && self
                .document
                .source
                .as_bytes()
                .get(item.span.start..end)
                .is_some_and(|prefix| prefix.len() == width && !prefix.contains(&b'\t'))
    }

    fn is_aligned_list(&self, list: &List) -> bool {
        if !list.ordered {
            return true;
        }
        // The first two items distinguish deliberate column alignment from a
        // single space after every marker. Use original positions, before renumbering.
        let position = |item: &ListItem| {
            let block = item
                .blocks
                .iter()
                .find(|b| !matches!(b.kind, BlockKind::Blank))?;
            let gap = &self.document.source[item.marker.end..block.span.start];
            let padded = gap
                .as_bytes()
                .get(..2)
                .is_some_and(|bytes| bytes.iter().all(u8::is_ascii_whitespace));
            Some((self.content_column(block)?, padded))
        };
        let Some((first, padded)) = list.items.first().and_then(position) else {
            return false;
        };
        if padded {
            return true;
        }
        let width = usize::from(self.context.print_options().indent_width.value());
        let on_tab_stop = width != 0 && first.is_multiple_of(width);
        match list.items.get(1) {
            None => on_tab_stop,
            Some(second) => position(second)
                .is_some_and(|(column, padded)| column == first && (on_tab_stop || padded)),
        }
    }

    fn content_column(&self, block: &Block) -> Option<usize> {
        let line = block.lines.first()?;
        let text = self.text(line.content);
        Some(
            line.content.start - line.raw.start
                + (text.len() - text.trim_start_matches([' ', '\t']).len()),
        )
    }
}

fn inline_math_width(
    padding: [usize; 4],
    column: usize,
    minimum: usize,
    marker_width: usize,
) -> Option<usize> {
    (minimum..=3 + marker_width + 4).find(|&width| padding[(column + width) % 4] == 0)
}

fn lazy_html_opening<'b>(previous: &Block, html: &'b Block) -> Option<&'b SourceLine> {
    match &previous.kind {
        BlockKind::Paragraph(_) => html.lines.first().filter(|line| line.lazy),
        _ => None,
    }
}

fn formats_children(kind: &BlockKind) -> bool {
    matches!(
        kind,
        BlockKind::List(_)
            | BlockKind::BlockQuote(_)
            | BlockKind::Extension(BlockExtension::Footnote { .. })
    )
}

fn minimum_list_indent_before(list: &List, block: &Block, source: &str) -> usize {
    if matches!(block.kind, BlockKind::Html) {
        // A verbatim sibling must fall short of the preceding item's body.
        let indent = block.lines.first().map_or(0, |line| line.indent(source));
        let shortest_item = if list.ordered { 3 } else { 2 };
        return if indent < shortest_item {
            0
        } else {
            indent + 1
        };
    }
    indentation_before_code(block)
}

fn indentation_before_code(block: &Block) -> usize {
    let BlockKind::IndentedCode(code) = &block.kind else {
        return 0;
    };
    // A following code block must stay at a smaller column than the item body;
    // otherwise a second parse would consume it as part of the list.
    let extra = code
        .value
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .map(|byte| if byte == b'\t' { 4 } else { 1 })
        .sum::<usize>();
    5 + extra
}

fn has_blank_line(gap: &str) -> bool {
    let mut carriage_return = false;
    let mut lines = 0;
    for byte in gap.bytes() {
        if byte == b'\r' || (byte == b'\n' && !carriage_return) {
            lines += 1;
            if lines == 2 {
                return true;
            }
        }
        carriage_return = byte == b'\r';
    }
    false
}

pub(super) fn write_source_lines(writer: &mut Writer<'_>, mut source: &str) {
    while let Some(index) = memchr::memchr2(b'\r', b'\n', source.as_bytes()) {
        writer.text_copied(&source[..index]);
        writer.literal_line_with_indent();
        let rest = &source[index..];
        source = &rest[1 + usize::from(rest.starts_with("\r\n"))..];
    }
    writer.text_copied(source);
}
