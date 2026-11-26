use std::{
    borrow::Cow,
    cell::{Cell, OnceCell},
    collections::{HashMap, HashSet},
};

use swc_next_formatter_core::Writer;
use swc_next_markdown_parser::{
    Block, BlockKind, Definition, Inline, InlineKind, Link, LinkTarget, ReferenceKind, SourceLine,
    Span, can_continue_link_definition, is_table_delimiter_row, liquid_block_delimiter, parse,
    starts_display_math, starts_footnote_definition,
};
use swc_next_simd::{cmp::SimdPartialEq, u8x16};

use crate::{Formatter, ProseWrap, blocks::write_source_lines, link_text};

#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
enum Separation {
    #[default]
    None,
    Space,
    SourceLine,
    HardLine,
    ExplicitLine,
    LiteralLine,
}

#[derive(Clone, Copy)]
struct Word {
    before: Separation,
    start: usize,
    end: usize,
    html_only: bool,
    prefix: LinePrefix,
}

#[derive(Clone, Copy, Default, PartialEq)]
enum LinePrefix {
    #[default]
    None,
    Escape,
    Indent,
}

impl Word {
    fn text(self, buffer: &str) -> &str {
        &buffer[self.start..self.end]
    }

    fn value(self, buffer: &str) -> Cow<'_, str> {
        let value = self.text(buffer);
        match self.prefix {
            LinePrefix::None => Cow::Borrowed(value),
            LinePrefix::Escape => Cow::Owned(format!("\\{value}")),
            LinePrefix::Indent => Cow::Owned(format!("    {value}")),
        }
    }

    fn liquid_needs_following_word(self, buffer: &str) -> bool {
        let text = self.text(buffer).trim_end();
        if !text.ends_with('}') {
            return false;
        }
        let standalone =
            |tag: &str| liquid_block_delimiter(tag).is_some_and(|end| tag.ends_with(end));
        if (self.before == Separation::None || self.before >= Separation::HardLine)
            && self.prefix == LinePrefix::None
            && standalone(text)
        {
            return true;
        }
        // A verbatim inline can itself put a Liquid opener at the start of its last line.
        text.rsplit_once(['\r', '\n']).is_some_and(|(_, line)| {
            let content = line.trim_start_matches([' ', '\t']);
            // A tab in a container can consume fewer than four columns.
            line.len() - content.len() < 4 && standalone(content)
        })
    }
}

struct Prose<'a> {
    wrap: ProseWrap,
    words: Vec<Word>,
    buffer: String,
    start: usize,
    before: Separation,
    prefix: LinePrefix,
    html_only: bool,
    source: &'a InlineSource<'a>,
}

struct InlineSource<'a> {
    source: &'a str,
    nodes: &'a [Inline],
    lines: &'a [SourceLine],
    line_index: Cell<usize>,
    first_ticks: OnceCell<HashMap<usize, usize>>,
}

impl<'a> InlineSource<'a> {
    fn new(source: &'a str, block: &'a Block, nodes: &'a [Inline]) -> Self {
        Self {
            source,
            nodes,
            lines: &block.lines,
            line_index: Cell::new(0),
            first_ticks: OnceCell::new(),
        }
    }

    // Inline nodes, including children of links and emphasis, are visited in source order.
    fn indent_protects_break(&self, end: usize) -> bool {
        let mut index = self.line_index.get();
        while let [previous, next, ..] = &self.lines[index..] {
            if previous.ending.end >= end {
                self.line_index.set(index);
                return previous.ending.end == end
                    && next.indent_protects_paragraph(self.source, *previous);
            }
            index += 1;
        }
        self.line_index.set(index);
        false
    }

    // A shorter code delimiter can pair with an earlier literal run, even across a link label.
    // Index first occurrences only when a code span would actually be shortened.
    fn literal_tick_precedes(&self, width: usize, offset: usize) -> bool {
        let first = self.first_ticks.get_or_init(|| {
            let mut first = HashMap::new();
            Self::collect_ticks(self.source, self.nodes, &mut first);
            first
        });
        first.get(&width).is_some_and(|&start| start < offset)
    }

    fn collect_ticks(source: &str, nodes: &[Inline], first: &mut HashMap<usize, usize>) {
        for node in nodes {
            match &node.kind {
                InlineKind::Text => {
                    let mut text = node.span.text(source);
                    while let Some(start) = text.find('`') {
                        let run = &text[start..];
                        text = run.trim_start_matches('`');
                        first
                            .entry(run.len() - text.len())
                            .or_insert(node.span.start);
                    }
                }
                InlineKind::Emphasis(children)
                | InlineKind::Strong(children)
                | InlineKind::Delete(children) => {
                    Self::collect_ticks(source, children, first);
                }
                InlineKind::Link(link) | InlineKind::Image(link) => {
                    Self::collect_ticks(source, &link.children, first)
                }
                _ => {}
            }
        }
    }
}

mod emphasis;
use emphasis::{Emphasis, EmphasisScope};

impl<'a> Prose<'a> {
    fn new(source: &'a InlineSource<'a>, wrap: ProseWrap, capacity: usize) -> Self {
        Self {
            wrap,
            words: Vec::new(),
            buffer: String::with_capacity(capacity),
            start: 0,
            before: Separation::None,
            prefix: LinePrefix::None,
            html_only: false,
            source,
        }
    }

    fn atom(&mut self, text: &str) {
        if !text.is_empty() {
            self.html_only = false;
        }
        self.buffer.push_str(text);
    }

    fn html(&mut self, text: &str) {
        self.html_only |= self.buffer.len() == self.start;
        self.buffer.push_str(text);
    }

    fn text(&mut self, text: &str) {
        let bytes = text.as_bytes();
        if self.wrap == ProseWrap::Preserve && normalized_spaces(text) {
            if text.starts_with(' ') {
                self.separate(Separation::Space);
            }
            self.atom(text.trim_matches(' '));
            if text.ends_with(' ') {
                self.separate(Separation::Space);
            }
            return;
        }
        let mut start = 0;
        for (index, &byte) in bytes.iter().enumerate() {
            let separator = match byte {
                // A single interior space on a preserved line belongs to the same text run.
                b' ' if self.wrap == ProseWrap::Preserve
                    && start < index
                    && bytes
                        .get(index + 1)
                        .is_some_and(|b| !b.is_ascii_whitespace()) =>
                {
                    continue;
                }
                b' ' | b'\t' => Separation::Space,
                b'\r' | b'\n' => Separation::SourceLine,
                _ => continue,
            };
            self.atom(&text[start..index]);
            self.separate(separator);
            start = index + 1;
        }
        self.atom(&text[start..]);
    }

    fn separate(&mut self, separator: Separation) {
        self.finish_word();
        if !self.words.is_empty() {
            self.before = self.before.max(separator);
        }
    }

    fn finish_word(&mut self) {
        if self.start != self.buffer.len() {
            self.words.push(Word {
                before: std::mem::take(&mut self.before),
                start: self.start,
                end: self.buffer.len(),
                html_only: std::mem::take(&mut self.html_only),
                prefix: std::mem::take(&mut self.prefix),
            });
            // Adjacent words can then be joined by extending a range.
            self.buffer.push(' ');
            self.start = self.buffer.len();
        }
    }

    fn starts_table_delimiter(&self, index: usize) -> bool {
        let text = self.words[index].text(&self.buffer);
        if text == "|" {
            // Padding after the opening pipe puts the first cell in the next word.
            return self.words.get(index + 1).is_some_and(|next| {
                let cell = next.text(&self.buffer);
                next.before < Separation::HardLine
                    && cell.starts_with([':', '-'])
                    && is_table_delimiter_row(cell)
            });
        }
        text.starts_with(['|', ':', '-']) && is_table_delimiter_row(text)
    }

    // After a required break, attach the delimiter to the first word that rules out
    // a table. If the whole source line is a delimiter, its first marker needs escaping.
    fn anchor_table_delimiter(&self, index: usize) -> (usize, LinePrefix) {
        let mut end = index;
        for word in &self.words[index + 1..] {
            if word.before != Separation::Space {
                break;
            }
            end += 1;
            let text = word.text(&self.buffer);
            if text != "|" && !is_table_delimiter_row(text) {
                return (end, LinePrefix::None);
            }
        }
        let row = &self.buffer[self.words[index].start..self.words[end].end];
        let prefix = if is_table_delimiter_row(row) {
            LinePrefix::Escape
        } else {
            LinePrefix::None
        };
        (end, prefix)
    }

    // At a required line start, keep the later dollar sign that made this line prose.
    fn anchor_math_opener(&self, index: usize) -> usize {
        if !starts_display_math(self.words[index].text(&self.buffer)) {
            return index;
        }
        self.words[index + 1..]
            .iter()
            .take_while(|word| word.before == Separation::Space)
            .position(|word| word.text(&self.buffer).contains('$'))
            .map_or(index, |offset| index + offset + 1)
    }

    fn finish(&mut self) {
        let wrap = self.wrap;
        self.finish_word();
        if self.words.is_empty() {
            return;
        }
        let mut count = 1;
        // A group can grow as words join. Inspect only its last original word for Liquid.
        let mut previous_word = self.words[0];
        let mut join_through = if wrap == ProseWrap::Always {
            self.anchor_math_opener(0)
        } else {
            0
        };
        for index in 1..self.words.len() {
            let mut word = self.words[index];
            let value = word.text(&self.buffer);
            let ends_line = self
                .words
                .get(index + 1)
                .is_none_or(|next| next.before >= Separation::SourceLine);
            let setext = ends_line && setext_marker(value);
            let join = word.before == Separation::SourceLine
                && (value.starts_with('>') || word.prefix == LinePrefix::Indent && !setext);
            if join {
                word.before = Separation::Space;
                word.prefix = LinePrefix::None;
            }
            let previous_group = self.words[count - 1];
            let left = previous_group.text(&self.buffer);
            let cjk = || has_ambiguous_cjk_boundary(left, value);
            let adjacent_html = previous_group.html_only && word.html_only;
            word.before = match word.before {
                Separation::SourceLine if wrap == ProseWrap::Preserve || adjacent_html || cjk() => {
                    Separation::HardLine
                }
                Separation::SourceLine => Separation::Space,
                other => other,
            };
            if word.before >= Separation::HardLine {
                if setext {
                    word.prefix = LinePrefix::Escape;
                } else if wrap == ProseWrap::Always && word.prefix == LinePrefix::None {
                    if self.starts_table_delimiter(index) {
                        (join_through, word.prefix) = self.anchor_table_delimiter(index);
                    } else {
                        join_through = self.anchor_math_opener(index);
                    }
                }
            }
            if word.before == Separation::Space
                && (join
                    || wrap != ProseWrap::Always
                    || index <= join_through
                    || cjk()
                    || ends_with_unescaped_backslash(left)
                    || previous_word.liquid_needs_following_word(&self.buffer)
                    || self.starts_table_delimiter(index)
                    || requires_prefix_word(value)
                        && !(adjacent_html && html_can_start_line(value)))
            {
                let previous = &mut self.words[count - 1];
                previous.end = word.end;
                previous.html_only &= word.html_only;
            } else {
                if word.before == Separation::HardLine && ends_with_unescaped_backslash(left) {
                    // A single trailing space keeps the source newline a soft break.
                    // Include the space stored after every word and preserve it in the Doc.
                    self.words[count - 1].end += 1;
                    word.before = Separation::LiteralLine;
                }
                self.words[count] = word;
                count += 1;
            }
            previous_word = word;
        }
        self.words.truncate(count);
    }
}

impl<'a> Formatter<'a, '_, '_> {
    pub(super) fn inlines(
        &self,
        block: &Block,
        nodes: &[Inline],
        wrap: ProseWrap,
        writer: &mut Writer<'a>,
    ) {
        if wrap != ProseWrap::Always
            && let Some(text) = self.plain_inline_text(nodes)
        {
            writer.text(text);
            return;
        }
        let wrap = if wrap != ProseWrap::Preserve
            && matches!(block.kind, BlockKind::Paragraph(_))
            && self.text(block.span).contains("]]")
            && self.has_literal_wiki_opening(nodes)
        {
            // Reflow can join a literal opening to a closing pair on a
            // different line and turn ordinary text into a wiki link.
            ProseWrap::Preserve
        } else {
            wrap
        };
        let source = InlineSource::new(self.document.source, block, nodes);
        let capacity = nodes.first().zip(nodes.last()).map_or(0, |(first, last)| {
            last.span.end.saturating_sub(first.span.start)
        });
        let mut prose = Prose::new(&source, wrap, capacity);
        self.collect_inlines(block, nodes, wrap, Emphasis::default(), &mut prose);
        prose.finish();
        if wrap == ProseWrap::Always && prose.words.len() > 1 {
            writer.fill(|writer| {
                let mut parts = prose.words.iter().peekable();
                while let Some(part) = parts.next() {
                    if part.before != Separation::None {
                        writer.fill_entry(|writer| write_separator(writer, part.before));
                    }
                    writer.fill_entry(|writer| {
                        write_source_lines(writer, &part.value(&prose.buffer));
                        // A Markdown hard break belongs to the content being
                        // laid out, like a newline inside an inline node.
                        while let Some(part) =
                            parts.next_if(|part| part.before >= Separation::ExplicitLine)
                        {
                            write_separator(writer, part.before);
                            write_source_lines(writer, &part.value(&prose.buffer));
                        }
                    });
                }
            });
        } else {
            for part in prose.words {
                write_separator(writer, part.before);
                write_source_lines(writer, &part.value(&prose.buffer));
            }
        }
    }

    fn has_literal_wiki_opening(&self, nodes: &[Inline]) -> bool {
        nodes.iter().any(|node| {
            let is_opening = || {
                let text = self.text(node.span);
                text.contains("[[")
                    || text.ends_with('[') && self.document.source[node.span.end..].starts_with('[')
            };
            match &node.kind {
                InlineKind::Text | InlineKind::Escape => is_opening(),
                InlineKind::Link(link) => {
                    matches!(&link.target, LinkTarget::Reference { identifier, .. }
                        if self.document.definition(identifier).is_none() && is_opening())
                        || self.has_literal_wiki_opening(&link.children)
                }
                InlineKind::Emphasis(children)
                | InlineKind::Strong(children)
                | InlineKind::Delete(children) => self.has_literal_wiki_opening(children),
                _ => false,
            }
        })
    }

    fn plain_inline_text(&self, nodes: &[Inline]) -> Option<&'a str> {
        let [
            Inline {
                span,
                kind: InlineKind::Text,
            },
        ] = nodes
        else {
            return None;
        };
        let text = self.text(*span).trim_matches(' ');
        normalized_spaces(text).then_some(text)
    }

    pub(super) fn inline_text(&self, block: &Block, nodes: &[Inline]) -> Cow<'a, str> {
        if let Some(text) = self.plain_inline_text(nodes) {
            return Cow::Borrowed(text);
        }
        Cow::Owned(self.collect_inline_text(
            block,
            nodes,
            ProseWrap::Never,
            Emphasis::default(),
            &InlineSource::new(self.document.source, block, nodes),
        ))
    }

    fn collect_inline_text(
        &self,
        block: &Block,
        nodes: &[Inline],
        wrap: ProseWrap,
        context: Emphasis,
        source: &InlineSource<'_>,
    ) -> String {
        let mut prose = Prose::new(source, wrap, 0);
        self.collect_inlines(block, nodes, wrap, context, &mut prose);
        prose.finish();
        let mut result = String::with_capacity(prose.buffer.len());
        for part in prose.words {
            match part.before {
                Separation::None => {}
                Separation::Space => result.push(' '),
                _ => result.push('\n'),
            }
            result.push_str(&part.value(&prose.buffer));
        }
        result
    }

    fn collect_inlines(
        &self,
        block: &Block,
        nodes: &[Inline],
        wrap: ProseWrap,
        context: Emphasis,
        prose: &mut Prose<'_>,
    ) {
        let emphasis = EmphasisScope::new(self.document.source, nodes);
        for (index, node) in nodes.iter().enumerate() {
            match &node.kind {
                InlineKind::Text => prose.text(self.text(node.span)),
                InlineKind::Escape | InlineKind::Autolink => prose.atom(self.text(node.span)),
                InlineKind::Html => prose.html(&self.inline_source(block, node.span)),
                InlineKind::Raw | InlineKind::Extension(_) | InlineKind::Recovery(_) => {
                    self.raw_inline(block, node.span, prose);
                }
                InlineKind::SoftBreak | InlineKind::HardBreak => {
                    if matches!(node.kind, InlineKind::SoftBreak) {
                        prose.separate(Separation::SourceLine);
                    } else if self.text(node.span).starts_with('\\') {
                        prose.atom("\\");
                        prose.separate(Separation::ExplicitLine);
                    } else {
                        prose.atom("  ");
                        prose.separate(Separation::LiteralLine);
                    }
                    if prose.source.indent_protects_break(node.span.end) {
                        prose.prefix = LinePrefix::Indent;
                    }
                }
                InlineKind::CodeSpan { opening, value, .. } => {
                    if wrap == ProseWrap::Preserve && self.text(node.span).contains(['\r', '\n']) {
                        self.raw_inline(block, node.span, prose);
                    } else {
                        write_code_span(value, *opening, self.document.source, prose);
                    }
                }
                InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                    let (marker, context) = emphasis.pair(index, children, context);
                    self.inline_container(block, children, wrap, marker, context, prose);
                }
                InlineKind::Delete(children) => {
                    let context = context.inside_inline();
                    self.inline_container(block, children, wrap, "~~", context, prose);
                }
                InlineKind::Link(link) | InlineKind::Image(link) => {
                    let context = context.inside_link();
                    self.link(block, node, link, wrap, context, prose);
                }
            }
        }
    }

    fn inline_source(&self, block: &Block, span: Span) -> Cow<'a, str> {
        let text = self.text(span);
        if text.contains(['\r', '\n']) {
            Cow::Owned(block.logical_text(self.document.source, span))
        } else {
            Cow::Borrowed(text)
        }
    }

    fn raw_inline(&self, block: &Block, span: Span, prose: &mut Prose<'_>) {
        prose.atom(&self.inline_source(block, span));
    }

    fn inline_container(
        &self,
        block: &Block,
        children: &[Inline],
        wrap: ProseWrap,
        marker: &str,
        context: Emphasis,
        prose: &mut Prose<'_>,
    ) {
        prose.atom(marker);
        self.collect_inlines(block, children, wrap, context, prose);
        prose.atom(marker);
    }

    fn link(
        &self,
        block: &Block,
        node: &Inline,
        link: &Link,
        wrap: ProseWrap,
        context: Emphasis,
        prose: &mut Prose<'_>,
    ) {
        let image = matches!(node.kind, InlineKind::Image(_));
        let brackets = if image { "![" } else { "[" };
        if let LinkTarget::Reference {
            kind: kind @ (ReferenceKind::Collapsed | ReferenceKind::Shortcut),
            ref identifier,
            ..
        } = link.target
        {
            // The visible label supplies the definition lookup key. Keep its
            // syntax and normalize only its whitespace when it is resolved.
            let resolved = self.document.definition(identifier).is_some();
            if resolved && self.reference_label_can_fold(&link.children) {
                prose.atom(brackets);
                let label = collapse_label(self.inline_source(block, link.label));
                if !image
                    && wrap == ProseWrap::Always
                    && link.children.iter().all(|child| {
                        matches!(
                            child.kind,
                            InlineKind::Text | InlineKind::SoftBreak | InlineKind::Escape
                        )
                    })
                {
                    prose.text(&label);
                } else {
                    prose.atom(&label);
                }
                prose.atom(if kind == ReferenceKind::Collapsed {
                    "][]"
                } else {
                    "]"
                });
            } else if !resolved
                && !image
                && self.document.definitions.is_empty()
                && self.text(link.label).contains(['\r', '\n'])
            {
                // Unresolved brackets are prose. With no definitions, reflow
                // and inline formatting cannot make them into a reference.
                prose.atom(brackets);
                self.collect_inlines(block, &link.children, wrap, context, prose);
                prose.atom(if kind == ReferenceKind::Collapsed {
                    "][]"
                } else {
                    "]"
                });
            } else {
                self.raw_inline(block, node.span, prose);
            }
            return;
        }

        prose.atom(brackets);
        if image {
            self.raw_inline(block, link.label, prose);
        } else {
            let label_wrap = if wrap == ProseWrap::Preserve {
                wrap
            } else {
                ProseWrap::Never
            };
            let label = match self.plain_inline_text(&link.children) {
                Some(text) => Cow::Borrowed(text),
                None => Cow::Owned(self.collect_inline_text(
                    block,
                    &link.children,
                    label_wrap,
                    context,
                    prose.source,
                )),
            };
            if matches!(link.target, LinkTarget::Reference { .. }) {
                let raw = self.text(link.label).as_bytes();
                if raw.first().is_some_and(u8::is_ascii_whitespace) {
                    prose.atom(" ");
                }
                prose.atom(&label);
                if raw.last().is_some_and(u8::is_ascii_whitespace) && !label.is_empty() {
                    prose.atom(" ");
                }
            } else {
                prose.atom(&label);
            }
        }
        prose.atom("]");
        match &link.target {
            LinkTarget::Reference {
                label: Some(label), ..
            } => {
                prose.atom("[");
                prose.atom(&collapse_label(self.inline_source(block, *label)));
                prose.atom("]");
            }
            LinkTarget::Reference { label: None, .. } => {}
            LinkTarget::Inline { destination, title } => {
                prose.atom("(");
                prose.atom(&link_text::destination(
                    self.text(*destination),
                    &block.kind,
                ));
                if let Some(title) = title.and_then(|span| self.title(block, span)) {
                    prose.atom(" ");
                    prose.atom(&title);
                }
                prose.atom(")");
            }
        }
    }

    fn reference_label_can_fold(&self, nodes: &[Inline]) -> bool {
        nodes.iter().all(|node| match &node.kind {
            // Folding part of an opaque value could change what the link shows.
            InlineKind::CodeSpan { .. }
            | InlineKind::Html
            | InlineKind::Raw
            | InlineKind::Extension(_)
            | InlineKind::Recovery(_) => {
                let raw = self.text(node.span);
                label_whitespace_is_normalized(raw)
            }
            InlineKind::Emphasis(children)
            | InlineKind::Strong(children)
            | InlineKind::Delete(children) => self.reference_label_can_fold(children),
            _ => true,
        })
    }

    fn title(&self, block: &Block, span: Span) -> Option<String> {
        link_text::title(
            &self.inline_source(block, span),
            self.options.single_quote,
            &block.kind,
        )
    }

    pub(super) fn definition(
        &self,
        block: &Block,
        definition: &Definition,
        writer: &mut Writer<'a>,
    ) {
        let label = self.inline_source(block, definition.label);
        writer.text("[");
        writer.text(collapse_label(label));
        writer.text("]:");
        let destination = link_text::destination(self.text(definition.destination), &block.kind);
        let title = definition.title.and_then(|span| self.title(block, span));
        // Indentation inside a multiline link title belongs to the title itself.
        if let Some(title) = title.as_deref().filter(|text| text.contains('\n')) {
            writer.space();
            writer.text(destination);
            writer.space();
            write_source_lines(writer, title);
            return;
        }
        let wrap = self.options.prose_wrap == ProseWrap::Always;
        let separator = if wrap { Writer::line } else { Writer::space };
        writer.group(false, |writer| {
            writer.indent(|writer| {
                if wrap && can_continue_link_definition(&destination) {
                    writer.line();
                } else {
                    writer.space();
                }
                writer.text(destination);
                if let Some(title) = &title {
                    separator(writer);
                    write_source_lines(writer, title);
                }
            });
        });
    }
}

fn is_markdown_punctuation(ch: char) -> bool {
    ch.is_ascii_punctuation() || (!ch.is_alphanumeric() && !ch.is_whitespace() && ch >= '\u{2000}')
}

fn is_word_char(ch: char) -> bool {
    !(ch.is_whitespace() || is_markdown_punctuation(ch))
}

fn write_separator(writer: &mut Writer<'_>, separation: Separation) {
    match separation {
        Separation::None => {}
        Separation::Space => writer.line(),
        Separation::SourceLine | Separation::HardLine | Separation::ExplicitLine => {
            writer.hard_line()
        }
        Separation::LiteralLine => writer.literal_line_with_indent(),
    }
}

fn write_code_span(value: &str, opening: Span, source: &str, prose: &mut Prose<'_>) {
    let width = if let Some(first) = value.find('`') {
        let runs: HashSet<_> = value.as_bytes()[first..]
            .split(|&byte| byte != b'`')
            .filter(|run| !run.is_empty())
            .map(<[u8]>::len)
            .collect();
        let mut width = 1;
        while runs.contains(&width) {
            width += 1;
        }
        width
    } else {
        1
    };
    let original = opening.text(source);
    let marker =
        if width < original.len() && prose.source.literal_tick_precedes(width, opening.start) {
            Cow::Borrowed(original)
        } else if let Some(shorter) = original.get(..width) {
            Cow::Borrowed(shorter)
        } else {
            Cow::Owned("`".repeat(width))
        };
    let pad = value.starts_with('`')
        || value.ends_with('`')
        || (value.starts_with(' ')
            && value.ends_with(' ')
            && value.bytes().any(|byte| byte != b' '));
    prose.atom(&marker);
    if pad {
        prose.atom(" ");
    }
    prose.atom(value);
    if pad {
        prose.atom(" ");
    }
    prose.atom(&marker);
}

fn normalized_spaces(text: &str) -> bool {
    let (blocks, tail) = text.as_bytes().as_chunks::<16>();
    let mut previous_space = false;
    for block in blocks {
        let bytes = u8x16::from_array(*block);
        let line_or_tab = bytes.simd_eq(u8x16::splat(b'\t'))
            | bytes.simd_eq(u8x16::splat(b'\r'))
            | bytes.simd_eq(u8x16::splat(b'\n'));
        let spaces = bytes.simd_eq(u8x16::splat(b' ')).to_bitmask();
        if line_or_tab.any() || spaces & ((spaces << 1) | u64::from(previous_space)) != 0 {
            return false;
        }
        previous_space = spaces & (1 << 15) != 0;
    }
    for &byte in tail {
        if matches!(byte, b'\t' | b'\r' | b'\n') || previous_space && byte == b' ' {
            return false;
        }
        previous_space = byte == b' ';
    }
    true
}

fn label_whitespace_is_normalized(label: &str) -> bool {
    normalized_spaces(label) && !label.as_bytes().contains(&0x0c)
}

fn collapse_label(label: Cow<'_, str>) -> Cow<'_, str> {
    if label_whitespace_is_normalized(&label) {
        return label;
    }
    let mut normalized = String::with_capacity(label.len());
    let mut in_whitespace = false;
    for ch in label.chars() {
        if ch.is_ascii_whitespace() {
            if !in_whitespace {
                normalized.push(' ');
                in_whitespace = true;
            }
        } else {
            normalized.push(ch);
            in_whitespace = false;
        }
    }
    Cow::Owned(normalized)
}

fn setext_marker(word: &str) -> bool {
    let first = word.as_bytes().first().copied();
    matches!(first, Some(b'-' | b'=')) && word.bytes().all(|byte| Some(byte) == first)
}

fn ends_with_unescaped_backslash(text: &str) -> bool {
    text.ends_with('\\') && text.bytes().rev().take_while(|&b| b == b'\\').count() % 2 == 1
}

fn html_can_start_line(fragment: &str) -> bool {
    // Splitting two inline fragments is safe when the second one still
    // parses as a paragraph at the start of a line.
    matches!(
        parse(fragment).blocks.as_slice(),
        [Block {
            kind: BlockKind::Paragraph(_),
            ..
        }]
    )
}

fn requires_prefix_word(word: &str) -> bool {
    match word.as_bytes().first() {
        Some(b'<' | b'>') => true,
        Some(b'[') => starts_footnote_definition(word),
        Some(b'$') => starts_display_math(word),
        Some(b'{') => {
            let first_line = word.split(['\r', '\n']).next().unwrap_or(word);
            liquid_block_delimiter(first_line).is_some()
        }
        Some(b'#') => word.len() <= 6 && word.bytes().all(|byte| byte == b'#'),
        Some(b'+') => word.len() == 1,
        Some(b'`') => word.starts_with("```"),
        Some(b'~') => word.starts_with("~~~"),
        Some(b'=' | b'-' | b'*' | b'_') => word
            .bytes()
            .all(|byte| matches!(byte, b'=' | b'-' | b'*' | b'_')),
        Some(b'0'..=b'9') => word.strip_suffix(['.', ')']).is_some_and(|digits| {
            digits.len() <= 9 && digits.bytes().all(|byte| byte.is_ascii_digit())
        }),
        _ => false,
    }
}

fn has_ambiguous_cjk_boundary(previous: &str, next: &str) -> bool {
    let (Some(previous), Some(next)) = (previous.chars().next_back(), next.chars().next()) else {
        return false;
    };
    let cj = |character| {
        matches!(character,
        '\u{2e80}'..='\u{303f}' | '\u{3040}'..='\u{30ff}' | '\u{31c0}'..='\u{31ef}'
        | '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}' | '\u{f900}'..='\u{faff}'
        | '\u{ff00}'..='\u{ffef}' | '\u{20000}'..='\u{323af}')
    };
    let hangul = |character| {
        matches!(character,
        '\u{1100}'..='\u{11ff}' | '\u{3130}'..='\u{318f}' | '\u{ac00}'..='\u{d7ff}')
    };
    (cj(previous) && (!hangul(next) || is_markdown_punctuation(previous)))
        || (cj(next) && (!hangul(previous) || is_markdown_punctuation(next)))
}
