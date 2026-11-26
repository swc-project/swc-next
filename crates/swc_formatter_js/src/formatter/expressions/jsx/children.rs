//! One forward JSX sibling stream prepares borrowed words and their boundaries.

use swc_next_ecma_ast::{
    Expr, ExprData, GetSpan, JsxChild, JsxChildData, JsxExpressionContainer, JsxExpressionData,
    Span, SubRange,
};

use super::super::parentheses::ExpressionContent;
use super::{Formatter, JsxBraceRole};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::ContentMode;
use crate::{
    FormatError, QuoteStyle,
    document::{ConditionTarget, FillEntry, Fragment, GroupId, Writer, WriterExt},
};

#[derive(Default)]
pub(super) struct JsxChildrenLayout {
    pub(super) has_content: bool,
    pub(super) contains_tag: bool,
    pub(super) multiple_expressions: bool,
    pub(super) template_only: bool,
    pub(super) leading_line: JsxBoundaryLine,
    pub(super) trailing_line: JsxBoundaryLine,
    pub(super) source: Option<Span>,
    pub(super) group: Option<GroupId>,
    grouped_text: bool,
    leading_space: Option<JsxLeadingSpace>,
    trailing_space: bool,
    only_space: Option<usize>,
    single_quote: bool,
    fixed_line: bool,
}

#[derive(Clone, Copy)]
enum JsxLeadingSpace {
    Separator {
        spaces: usize,
        following: LineConstraint,
    },
    Literal,
}

impl JsxChildrenLayout {
    fn space_layout(&self) -> JsxSpaceLayout {
        if self.fixed_line {
            JsxSpaceLayout::Fixed
        } else {
            JsxSpaceLayout::Conditional(ConditionTarget::Current)
        }
    }

    pub(super) fn write<'ast>(&self, content: Fragment<'ast>, writer: &mut Writer<'ast>) {
        if let Some(spaces) = self.only_space {
            write_jsx_space(self.single_quote, spaces, None, self.space_layout(), writer);
            return;
        }
        let write = |writer: &mut Writer<'ast>| {
            if let Some(JsxLeadingSpace::Separator { spaces, following }) = self.leading_space {
                write_jsx_space(
                    self.single_quote,
                    spaces,
                    Some(following),
                    self.space_layout(),
                    writer,
                );
            }
            writer.write_fragment(content);
        };
        if self.grouped_text {
            writer.group(false, write);
        } else {
            write(writer);
        }
    }
}

#[derive(Clone, Copy)]
enum JsxSeparator {
    Words { spaces: usize },
    Space { spaces: usize },
    TextLine,
    Newline { empty_line: bool },
}

#[derive(Clone, Copy, Default)]
pub(super) enum JsxBoundaryLine {
    #[default]
    Soft,
    Inline,
    Hard,
}

impl JsxBoundaryLine {
    fn for_text_edge(touches_space: bool, multiline: bool) -> Self {
        match (touches_space, multiline) {
            (false, _) => Self::Soft,
            (true, false) => Self::Inline,
            (true, true) => Self::Hard,
        }
    }

    pub(super) fn write(self, writer: &mut Writer<'_>) {
        match self {
            Self::Soft => writer.soft_line(),
            Self::Inline => {}
            Self::Hard => writer.hard_line(),
        }
    }
}

#[derive(Clone, Copy)]
enum JsxContentEdge {
    Word {
        single_character: bool,
        starts_with_space: bool,
        ends_with_space: bool,
    },
    NonText {
        self_closing: bool,
        nested_jsx: bool,
    },
    WhitespaceLiteral {
        leading: bool,
    },
}

impl JsxContentEdge {
    fn starts_with_space(self) -> bool {
        matches!(
            self,
            Self::Word {
                starts_with_space: true,
                ..
            }
        )
    }

    fn ends_with_space(self) -> bool {
        matches!(
            self,
            Self::Word {
                ends_with_space: true,
                ..
            }
        )
    }
}

#[derive(Default)]
struct JsxChildrenStream {
    layout: JsxChildrenLayout,
    previous: Option<(JsxContentEdge, Span, FillEntry)>,
    pending: Option<(JsxSeparator, Span)>,
    contains_text: bool,
    seen_expression: bool,
    has_empty_line: bool,
    reflow_text_edges: bool,
}

impl JsxChildrenStream {
    fn text<'ast>(
        &mut self,
        chunks: impl Iterator<Item = (JsxTextChunk<'ast>, Span)>,
        text_span: Span,
        formatter: &Formatter<'ast, '_>,
        writer: &mut Writer<'ast>,
    ) -> bool {
        let mut meaningful = text_span.start == text_span.end;
        for (chunk, source) in chunks {
            match chunk {
                JsxTextChunk::Word {
                    text,
                    starts_with_space,
                    ends_with_space,
                } => {
                    let edge = JsxContentEdge::Word {
                        single_character: text.chars().nth(1).is_none(),
                        starts_with_space,
                        ends_with_space,
                    };
                    self.before(edge, source, formatter, writer);
                    let (entry, ()) = writer.fill_entry_with_handle(|writer| writer.text(text));
                    self.after(edge, source, entry);
                    self.contains_text = true;
                    meaningful = true;
                }
                JsxTextChunk::Whitespace {
                    line_breaks,
                    spaces,
                } => {
                    meaningful |= line_breaks == 0;
                    let first = source.start == text_span.start;
                    let last = source.end == text_span.end;
                    let separator = if !first && !last {
                        JsxSeparator::Words { spaces }
                    } else if line_breaks == 0 {
                        JsxSeparator::Space { spaces }
                    } else if first && last {
                        JsxSeparator::Newline {
                            empty_line: line_breaks == 2,
                        }
                    } else {
                        JsxSeparator::TextLine
                    };
                    self.separator(separator, source, formatter, writer);
                }
            }
        }
        meaningful
    }

    fn separator(
        &mut self,
        separator: JsxSeparator,
        source: Span,
        formatter: &Formatter<'_, '_>,
        writer: &mut Writer<'_>,
    ) {
        // A lone physical line break between children has no JSX value. Keep
        // the boundary selected by those children, including after a space literal.
        if matches!(separator, JsxSeparator::Newline { empty_line: false })
            || matches!(separator, JsxSeparator::Newline { .. })
                && self.previous.as_ref().is_some_and(|(edge, ..)| {
                    matches!(edge, JsxContentEdge::WhitespaceLiteral { .. })
                })
        {
            return;
        }
        if let Some((JsxSeparator::Space { spaces }, previous)) = self.pending
            && matches!(separator, JsxSeparator::Space { .. })
            && matches!(
                formatter.line_constraint(previous.end, source.start),
                LineConstraint::NewLine(_)
            )
        {
            self.pending = None;
            self.space_literal(spaces, previous, formatter, writer);
        }
        match (&mut self.pending, separator) {
            (Some((JsxSeparator::Space { spaces }, span)), next) => {
                if let JsxSeparator::Space { spaces: additional } = next {
                    *spaces += additional;
                    span.end = source.end;
                }
            }
            (pending, separator) => *pending = Some((separator, source)),
        }
        self.contains_text |= matches!(separator, JsxSeparator::Space { .. });
    }

    fn before(
        &mut self,
        next: JsxContentEdge,
        source: Span,
        formatter: &Formatter<'_, '_>,
        writer: &mut Writer<'_>,
    ) {
        self.flush_space_literal(formatter, writer);
        self.boundary(next, source, formatter, writer);
    }

    fn flush_space_literal(&mut self, formatter: &Formatter<'_, '_>, writer: &mut Writer<'_>) {
        if let Some((
            JsxSeparator::Space {
                spaces: spaces @ 2..,
            }
            | JsxSeparator::Words {
                spaces: spaces @ 2..,
                ..
            },
            source,
        )) = self.pending
        {
            self.pending = None;
            self.space_literal(spaces, source, formatter, writer);
        }
    }

    fn space_literal(
        &mut self,
        spaces: usize,
        source: Span,
        formatter: &Formatter<'_, '_>,
        writer: &mut Writer<'_>,
    ) {
        let edge = JsxContentEdge::WhitespaceLiteral {
            leading: self.previous.is_none(),
        };
        self.boundary(edge, source, formatter, writer);
        let (entry, ()) = writer.fill_entry_with_handle(|writer| {
            write_jsx_space(
                self.layout.single_quote,
                spaces,
                None,
                JsxSpaceLayout::Explicit,
                writer,
            );
        });
        self.after(edge, source, entry);
        self.contains_text = true;
    }

    fn boundary(
        &mut self,
        next: JsxContentEdge,
        source: Span,
        formatter: &Formatter<'_, '_>,
        writer: &mut Writer<'_>,
    ) {
        let pending = self.pending.take();
        let Some((previous, previous_source, _)) = self.previous.as_ref() else {
            let start = if let Some((JsxSeparator::Space { spaces }, space)) = pending {
                self.layout.leading_space = Some(JsxLeadingSpace::Separator {
                    spaces,
                    following: jsx_line_constraint(
                        formatter,
                        space.end,
                        source.start,
                        next.starts_with_space(),
                    ),
                });
                space.start
            } else {
                if matches!(next, JsxContentEdge::WhitespaceLiteral { .. }) {
                    self.layout.leading_space = Some(JsxLeadingSpace::Literal);
                }
                source.start
            };
            self.layout.source = Some(Span::new(start, source.end));
            self.layout.leading_line = if self.layout.leading_space.is_none() {
                JsxBoundaryLine::for_text_edge(
                    next.starts_with_space(),
                    matches!(pending, Some((JsxSeparator::TextLine, _))),
                )
            } else {
                JsxBoundaryLine::Soft
            };
            return;
        };
        let previous = *previous;
        let previous_end = previous_source.end;
        let touches_space = previous.ends_with_space() || next.starts_with_space();
        writer.fill_entry(|writer| match pending {
            Some((JsxSeparator::Space { spaces }, space))
            | Some((
                JsxSeparator::Words {
                    spaces: spaces @ 2..,
                    ..
                },
                space,
            )) => {
                let before = jsx_line_constraint(
                    formatter,
                    previous_end,
                    space.start,
                    previous.ends_with_space(),
                );
                let following = jsx_line_constraint(
                    formatter,
                    space.end,
                    source.start,
                    next.starts_with_space(),
                );
                before.write(writer, FlatSeparator::Empty, |_| {});
                let layout = if matches!(before, LineConstraint::NewLine(_)) {
                    JsxSpaceLayout::Explicit
                } else {
                    self.layout.space_layout()
                };
                write_jsx_space(
                    self.layout.single_quote,
                    spaces,
                    Some(following),
                    layout,
                    writer,
                );
            }
            Some((JsxSeparator::Words { .. }, _)) => {
                let constraint =
                    jsx_line_constraint(formatter, previous_end, source.start, touches_space);
                constraint.write(writer, FlatSeparator::Space, |writer| writer.line());
            }
            Some((JsxSeparator::Newline { empty_line }, _)) => {
                let constraint = formatter.line_constraint(previous_end, source.start);
                constraint.write(writer, FlatSeparator::Empty, |writer| {
                    writer.hard_line();
                    if empty_line {
                        writer.hard_line();
                        self.has_empty_line = true;
                    }
                });
            }
            Some((JsxSeparator::TextLine, _)) | None => {
                let text_line = pending.is_some();
                let constraint =
                    jsx_line_constraint(formatter, previous_end, source.start, touches_space);
                constraint.write(writer, FlatSeparator::Empty, |writer| {
                    match (previous, next) {
                        (
                            JsxContentEdge::Word {
                                single_character, ..
                            },
                            JsxContentEdge::NonText { self_closing, .. },
                        )
                        | (
                            JsxContentEdge::NonText { self_closing, .. },
                            JsxContentEdge::Word {
                                single_character, ..
                            },
                        ) => {
                            // Reflow must expose the same soft break on the next pass.
                            let optional = !text_line || self.reflow_text_edges;
                            if single_character && (!self_closing || optional)
                                || !self_closing && optional
                            {
                                writer.soft_line();
                            } else {
                                writer.hard_line();
                            }
                        }
                        _ if text_line => writer.hard_line(),
                        // A nested JSX child can choose its own layout when
                        // this boundary breaks.
                        (
                            JsxContentEdge::WhitespaceLiteral { leading },
                            JsxContentEdge::NonText { nested_jsx, .. },
                        ) if leading || !nested_jsx => writer.hard_line(),
                        (JsxContentEdge::WhitespaceLiteral { .. }, _) => writer.soft_line(),
                        (_, JsxContentEdge::WhitespaceLiteral { .. }) => {}
                        (JsxContentEdge::NonText { .. }, JsxContentEdge::NonText { .. }) => {
                            writer.hard_line()
                        }
                        (JsxContentEdge::Word { .. }, JsxContentEdge::Word { .. }) => {}
                    }
                });
            }
        });
    }

    fn after(&mut self, edge: JsxContentEdge, source: Span, entry: FillEntry) {
        self.previous = Some((edge, source, entry));
        self.layout.has_content = true;
        self.layout.source.get_or_insert(source).end = source.end;
        self.layout.trailing_space = matches!(edge, JsxContentEdge::WhitespaceLiteral { .. });
    }

    fn finish(
        mut self,
        formatter: &Formatter<'_, '_>,
        writer: &mut Writer<'_>,
    ) -> JsxChildrenLayout {
        self.flush_space_literal(formatter, writer);
        if let Some((edge, ..)) = self.previous {
            self.layout.trailing_line = JsxBoundaryLine::for_text_edge(
                edge.ends_with_space(),
                matches!(self.pending, Some((JsxSeparator::TextLine, _))),
            );
        }
        if let Some((JsxSeparator::Space { spaces }, source)) = self.pending {
            self.layout.source.get_or_insert(source).end = source.end;
            if let Some((edge, previous, entry)) = self.previous {
                self.layout.trailing_space = true;
                self.layout.trailing_line = JsxBoundaryLine::Soft;
                let before = jsx_line_constraint(
                    formatter,
                    previous.end,
                    source.start,
                    edge.ends_with_space(),
                );
                // The trailing space belongs to the enclosing JSX layout, while
                // its retained fill entry may independently fit on one line.
                let layout = if matches!(before, LineConstraint::NewLine(_)) {
                    JsxSpaceLayout::Explicit
                } else {
                    self.layout.group =
                        (spaces == 1 && !self.layout.fixed_line).then(|| writer.next_group_id());
                    self.layout
                        .group
                        .map_or(self.layout.space_layout(), |group| {
                            JsxSpaceLayout::Conditional(ConditionTarget::Group(group))
                        })
                };
                writer.extend_fill_entry(entry, |writer| {
                    before.write(writer, FlatSeparator::Empty, |_| {});
                    write_jsx_space(self.layout.single_quote, spaces, None, layout, writer);
                });
            } else {
                self.layout.only_space = Some(spaces);
                self.layout.has_content = true;
            }
        }
        self.layout.grouped_text = self.contains_text
            && self.layout.leading_space.is_none()
            && !self.layout.trailing_space
            && self.layout.only_space.is_none();
        self.layout
    }
}

fn jsx_line_constraint(
    formatter: &Formatter<'_, '_>,
    start: u32,
    end: u32,
    touches_space: bool,
) -> LineConstraint {
    let constraint = formatter.line_constraint(start, end);
    if constraint != LineConstraint::Unconstrained || !touches_space {
        return constraint;
    }
    // Moving a physical line can activate or trim significant text whitespace.
    if formatter.line_breaks(start, end) == 0 {
        LineConstraint::SameLine
    } else {
        LineConstraint::NewLine(1)
    }
}

#[derive(Clone, Copy)]
enum JsxSpaceLayout {
    Explicit,
    Fixed,
    Conditional(ConditionTarget),
}

fn write_jsx_space<'ast>(
    single_quote: bool,
    spaces: usize,
    following: Option<LineConstraint>,
    layout: JsxSpaceLayout,
    writer: &mut Writer<'ast>,
) {
    if spaces > 1 {
        // Raw trailing spaces disappear at a physical JSX line boundary.
        writer.text(if single_quote { "{'" } else { "{\"" });
        writer.text(" ".repeat(spaces));
        writer.text(if single_quote { "'}" } else { "\"}" });
    } else {
        let space = if single_quote { "{' '}" } else { "{\" \"}" };
        if following.is_some_and(|boundary| matches!(boundary, LineConstraint::NewLine(_))) {
            // Text space before a required line break has no JSX value.
            writer.text(space);
        } else {
            match layout {
                JsxSpaceLayout::Explicit => writer.text(space),
                JsxSpaceLayout::Fixed => writer.text(" "),
                JsxSpaceLayout::Conditional(target) => {
                    writer.if_break_for(target, space);
                    writer.if_flat_for(target, " ");
                }
            }
        }
    }
    if let Some(boundary) = following {
        boundary.write(writer, FlatSeparator::Empty, |writer| writer.soft_line());
    }
}

/// Words borrow the original JSX text. Only ASCII spaces, tabs, CR and LF
/// participate in JSX whitespace folding.
struct JsxTextChunks<'ast> {
    remaining: &'ast str,
    span: Span,
    pending: Option<JsxTextGap>,
    preserves_whitespace: bool,
}

enum JsxTextChunk<'ast> {
    Word {
        text: &'ast str,
        starts_with_space: bool,
        ends_with_space: bool,
    },
    Whitespace {
        line_breaks: u8,
        spaces: usize,
    },
}

struct JsxTextGap {
    len: usize,
    line_breaks: u8,
    has_tab: bool,
}

impl JsxTextGap {
    fn scan(source: &[u8]) -> Self {
        let mut len = 0;
        let mut line_breaks = 0_u8;
        let mut previous_cr = false;
        let mut has_tab = false;
        for &byte in source.iter().take_while(|&&byte| is_jsx_whitespace(byte)) {
            len += 1;
            if byte == b'\r' || byte == b'\n' && !previous_cr {
                line_breaks = (line_breaks + 1).min(2);
            }
            previous_cr = byte == b'\r';
            has_tab |= byte == b'\t';
        }
        Self {
            len,
            line_breaks,
            has_tab,
        }
    }
}

impl<'ast> JsxTextChunks<'ast> {
    fn new(remaining: &'ast str, span: Span) -> Self {
        Self {
            remaining,
            span,
            pending: None,
            preserves_whitespace: false,
        }
    }

    fn advance(&mut self, len: usize) -> (&'ast str, Span) {
        let (value, remaining) = self.remaining.split_at(len);
        let end = self.span.end - remaining.len() as u32;
        let source = Span::new(self.span.start, end);
        self.span.start = end;
        self.remaining = remaining;
        (value, source)
    }

    fn whitespace(&mut self, gap: JsxTextGap) -> (JsxTextChunk<'ast>, Span) {
        let (_, source) = self.advance(gap.len);
        let chunk = JsxTextChunk::Whitespace {
            line_breaks: gap.line_breaks,
            spaces: if gap.line_breaks == 0 { gap.len } else { 1 },
        };
        (chunk, source)
    }
}

impl<'ast> Iterator for JsxTextChunks<'ast> {
    type Item = (JsxTextChunk<'ast>, Span);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(gap) = self.pending.take() {
            return Some(self.whitespace(gap));
        }
        let bytes = self.remaining.as_bytes();
        let mut end = 0;
        let mut ends_with_space = false;
        while let Some(&byte) = bytes.get(end) {
            if !is_jsx_whitespace(byte) {
                if byte == b'&'
                    && let Some(length) = jsx_numeric_space_prefix(&bytes[end..])
                {
                    ends_with_space = true;
                    end += length;
                } else if byte.is_ascii() {
                    ends_with_space = matches!(byte, b'\x0b' | b'\x0c');
                    end += 1;
                } else {
                    let character = self.remaining[end..].chars().next().unwrap();
                    ends_with_space = is_jsx_line_edge_space(character);
                    end += character.len_utf8();
                }
                self.preserves_whitespace |= ends_with_space;
                continue;
            }
            let gap = JsxTextGap::scan(&bytes[end..]);
            // A break inside this horizontal run would trim its significant
            // spaces and tabs. Give the fill one entry for the whole word.
            if gap.line_breaks == 0
                && (gap.has_tab
                    || ends_with_space
                    || jsx_text_starts_with_edge_space(&self.remaining[end + gap.len..]))
            {
                self.preserves_whitespace = true;
                ends_with_space = true;
                end += gap.len;
                continue;
            }
            if end == 0 {
                return Some(self.whitespace(gap));
            }
            self.pending = Some(gap);
            break;
        }
        if end == 0 {
            return None;
        }
        // A boundary is an ASCII formatting character, so it cannot split UTF-8.
        let (word, source) = self.advance(end);
        Some((
            JsxTextChunk::Word {
                text: word,
                starts_with_space: jsx_text_starts_with_edge_space(word),
                ends_with_space,
            },
            source,
        ))
    }
}

fn is_jsx_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

fn is_jsx_line_edge_space(character: char) -> bool {
    character.is_whitespace() || matches!(character, '\u{200b}' | '\u{feff}')
}

fn jsx_text_starts_with_edge_space(text: &str) -> bool {
    text.chars().next().is_some_and(is_jsx_line_edge_space)
        || jsx_numeric_space_prefix(text.as_bytes()).is_some()
}

// JSX transforms can decode these references before trimming physical lines.
fn jsx_numeric_space_prefix(text: &[u8]) -> Option<usize> {
    let digits = text.strip_prefix(b"&#")?;
    let (digits, radix) = if let Some(digits) = digits
        .strip_prefix(b"x")
        .or_else(|| digits.strip_prefix(b"X"))
    {
        (digits, 16)
    } else {
        (digits, 10)
    };
    let mut value = 0;
    for (index, &byte) in digits.iter().enumerate() {
        if byte == b';' && matches!(value, 9 | 32) {
            return Some(text.len() - digits.len() + index + 1);
        }
        value = value * radix + char::from(byte).to_digit(radix)?;
        if value > 32 {
            break;
        }
    }
    None
}

fn jsx_text_may_need_edge_protection(text: &str) -> bool {
    !text.is_ascii()
        || text.contains("&#")
        || memchr::memchr3(b'\t', b'\x0b', b'\x0c', text.as_bytes()).is_some()
}

/// Child eligibility and its eventual printer share the actual prepared core.
struct JsxChildExpression {
    original: Expr,
    content: Option<ExpressionContent>,
    kind: JsxChildExpressionKind,
}

#[derive(Clone, Copy)]
enum JsxChildExpressionKind {
    Expression,
    Whitespace(usize),
    Template,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_jsx_children(
        &mut self,
        children: SubRange<JsxChild>,
        fixed_line: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<JsxChildrenLayout, FormatError> {
        self.with_recursion_scope(|formatter| {
            formatter.format_jsx_children_inner(children, fixed_line, writer)
        })
    }

    fn format_jsx_children_inner(
        &mut self,
        children: SubRange<JsxChild>,
        fixed_line: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<JsxChildrenLayout, FormatError> {
        let ast = self.ast;
        let mut stream = JsxChildrenStream::default();
        stream.layout.single_quote = self.options.quote_style == QuoteStyle::Single;
        stream.layout.fixed_line = fixed_line;
        let prepare_text = children.len() > 1
            && children.iter().any(|id| {
                let child = ast.get_node_in_sub_range(id);
                match ast.jsx_child_data(child) {
                    JsxChildData::JsxText(text) => {
                        jsx_text_may_need_edge_protection(ast.get_utf8(text.value(ast)))
                    }
                    _ => false,
                }
            });
        let mut prepared_text = Vec::new();
        // Plain ASCII text has no protected edges to coordinate.
        if prepare_text {
            for id in children.iter() {
                let child = ast.get_node_in_sub_range(id);
                if let JsxChildData::JsxText(text) = ast.jsx_child_data(child) {
                    let mut chunks =
                        JsxTextChunks::new(ast.get_utf8(text.value(ast)), text.span(ast));
                    prepared_text.extend(chunks.by_ref());
                    stream.reflow_text_edges |= chunks.preserves_whitespace;
                }
            }
        }
        let mut prepared_text = prepared_text.into_iter().peekable();
        let mut previous_ignore = false;
        writer.fill_with_empty_line_policy(|writer| {
            for id in children.iter() {
                let child = ast.get_node_in_sub_range(id);
                let data = ast.jsx_child_data(child);
                let expression = match data {
                    JsxChildData::JsxExpressionContainer(container) => {
                        self.prepare_jsx_child_expression(container)
                    }
                    _ => None,
                };
                // A sole template owns wrapping through its interpolations.
                stream.layout.template_only = children.len() == 1
                    && expression.as_ref().is_some_and(|expression| {
                        matches!(expression.kind, JsxChildExpressionKind::Template)
                    });
                if let Some(JsxChildExpression {
                    kind: JsxChildExpressionKind::Whitespace(spaces),
                    ..
                }) = expression.as_ref()
                {
                    previous_ignore = false;
                    if matches!(stream.pending, Some((JsxSeparator::Newline { .. }, _))) {
                        stream.pending = None;
                    }
                    let touches_text_space = stream
                        .previous
                        .as_ref()
                        .is_some_and(|(edge, ..)| edge.ends_with_space())
                        || prepared_text.peek().is_some_and(|(chunk, source)| {
                            source.start == child.span(ast).end
                                && matches!(
                                    chunk,
                                    JsxTextChunk::Word {
                                        starts_with_space: true,
                                        ..
                                    }
                                )
                        });
                    if *spaces == 1 && !touches_text_space {
                        stream.separator(
                            JsxSeparator::Space { spaces: 1 },
                            child.span(ast),
                            self,
                            writer,
                        );
                    } else {
                        stream.flush_space_literal(self, writer);
                        stream.space_literal(*spaces, child.span(ast), self, writer);
                    }
                    continue;
                }
                if let JsxChildData::JsxText(text) = data {
                    let span = text.span(ast);
                    let meaningful = if !prepare_text {
                        let chunks = JsxTextChunks::new(ast.get_utf8(text.value(ast)), span);
                        stream.text(chunks, span, self, writer)
                    } else {
                        let chunks = std::iter::from_fn(|| {
                            prepared_text.next_if(|(_, source)| source.end <= span.end)
                        });
                        stream.text(chunks, span, self, writer)
                    };
                    // Ignore follows the original sibling, not its canonical spelling.
                    if meaningful {
                        previous_ignore = false;
                    }
                    continue;
                }
                let self_closing = match data {
                    JsxChildData::JsxElement(element) => {
                        stream.layout.contains_tag = true;
                        element.opening_element(ast).self_closing(ast)
                    }
                    JsxChildData::JsxFragment(_) => {
                        stream.layout.contains_tag = true;
                        false
                    }
                    JsxChildData::JsxExpressionContainer(_) => {
                        stream.layout.multiple_expressions |= stream.seen_expression;
                        stream.seen_expression = true;
                        false
                    }
                    _ => false,
                };
                let nested_jsx = matches!(
                    data,
                    JsxChildData::JsxElement(_) | JsxChildData::JsxFragment(_)
                ) && !self_closing;
                let edge = JsxContentEdge::NonText {
                    self_closing,
                    nested_jsx,
                };
                let source = child.span(ast);
                stream.before(edge, source, self, writer);
                let ignored = previous_ignore
                    && matches!(
                        data,
                        JsxChildData::JsxElement(_) | JsxChildData::JsxFragment(_)
                    );
                let next_ignore = if self.has_suppression {
                    match data {
                        JsxChildData::JsxExpressionContainer(container) => {
                            matches!(
                                ast.jsx_expression_data(container.expression(ast)),
                                JsxExpressionData::JsxEmptyExpression(_)
                            ) && self.content_mode(self.comments.in_range(container.span(ast)))
                                == ContentMode::Verbatim
                        }
                        _ => false,
                    }
                } else {
                    false
                };
                let (entry, result) = writer.fill_entry_with_handle(|writer| {
                    if ignored {
                        return self.format_verbatim(child.span(ast), writer);
                    }
                    match data {
                        JsxChildData::JsxText(_) => {
                            unreachable!("text siblings are consumed before child dispatch")
                        }
                        JsxChildData::JsxExpressionContainer(container) => {
                            if let Some(expression) = expression {
                                self.format_jsx_braced_expression_prepared(
                                    expression.original,
                                    container.span(ast),
                                    false,
                                    JsxBraceRole::Child,
                                    expression.content,
                                    writer,
                                )
                            } else {
                                self.format_jsx_container(container, JsxBraceRole::Child, writer)
                            }
                        }
                        JsxChildData::JsxSpreadChild(spread) => self.format_jsx_braced_expression(
                            spread.expression(ast),
                            spread.span(ast),
                            true,
                            JsxBraceRole::Child,
                            writer,
                        ),
                        JsxChildData::JsxElement(element) => {
                            self.format_jsx_element(element, writer)
                        }
                        JsxChildData::JsxFragment(fragment) => {
                            self.format_jsx_fragment(fragment, writer)
                        }
                    }
                });
                result?;
                previous_ignore = next_ignore;
                stream.after(edge, source, entry);
            }
            let collapse_empty_lines = stream.has_empty_line && stream.contains_text;
            Ok((stream.finish(self, writer), collapse_empty_lines))
        })
    }

    fn prepare_jsx_child_expression(
        &self,
        container: JsxExpressionContainer,
    ) -> Option<JsxChildExpression> {
        let JsxExpressionData::Expr(original) =
            self.ast.jsx_expression_data(container.expression(self.ast))
        else {
            return None;
        };
        let data = self.ast.expr_data(original);
        let content = matches!(data, ExprData::ParenthesizedExpression(_))
            .then(|| self.expression_content(original));
        let printed = content.as_ref().map_or(data, |content| content.data);
        let kind = match printed {
            ExprData::TemplateLiteral(_) | ExprData::TaggedTemplateExpression(_) => {
                JsxChildExpressionKind::Template
            }
            ExprData::StringLiteral(literal)
                if self.comments.in_range(container.span(self.ast)).is_empty()
                    && !matches!(
                        self.line_constraint(
                            container.span(self.ast).start,
                            container.span(self.ast).end,
                        ),
                        LineConstraint::NewLine(_)
                    ) =>
            {
                self.ast
                    .get_wtf8(literal.value(self.ast))
                    .as_str()
                    .filter(|text| !text.is_empty() && text.bytes().all(|byte| byte == b' '))
                    .map_or(JsxChildExpressionKind::Expression, |text| {
                        JsxChildExpressionKind::Whitespace(text.len())
                    })
            }
            _ => JsxChildExpressionKind::Expression,
        };
        Some(JsxChildExpression {
            original,
            content,
            kind,
        })
    }
}
