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
    document::{FillEntry, Fragment, Writer},
};

#[derive(Default)]
pub(super) struct JsxChildrenLayout {
    pub(super) has_content: bool,
    pub(super) contains_tag: bool,
    pub(super) multiple_expressions: bool,
    pub(super) template_only: bool,
    pub(super) leading_inline: bool,
    pub(super) trailing_inline: bool,
    pub(super) source: Option<Span>,
    grouped_text: bool,
    leading_space: Option<LineConstraint>,
    trailing_space: bool,
    only_space: bool,
    single_quote: bool,
}

impl JsxChildrenLayout {
    pub(super) fn write<'ast>(&self, content: Fragment<'ast>, writer: &mut Writer<'ast>) {
        if self.only_space {
            write_jsx_space(
                self.single_quote,
                JsxSpacePlacement::Lone,
                LineConstraint::Unconstrained,
                writer,
            );
            return;
        }
        let write = |writer: &mut Writer<'ast>| {
            if let Some(following) = self.leading_space {
                write_jsx_space(
                    self.single_quote,
                    JsxSpacePlacement::Leading,
                    following,
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
    Words,
    Space,
    TextLine,
    Newline { empty_line: bool },
}

#[derive(Clone, Copy)]
enum JsxContentEdge {
    Word { single_character: bool },
    NonText { self_closing: bool },
    Literal,
}

#[derive(Default)]
struct JsxChildrenStream {
    layout: JsxChildrenLayout,
    previous: Option<(JsxContentEdge, Span, FillEntry)>,
    pending: Option<(JsxSeparator, Span)>,
    contains_text: bool,
    seen_expression: bool,
    has_empty_line: bool,
}

impl JsxChildrenStream {
    fn separator(&mut self, separator: JsxSeparator, source: Span) {
        // Canonical space supersedes adjacent physical formatting newlines.
        if !matches!(self.pending, Some((JsxSeparator::Space, _))) {
            self.pending = Some((separator, source));
        }
        self.contains_text |= matches!(separator, JsxSeparator::Space);
    }

    fn before(
        &mut self,
        next: JsxContentEdge,
        source: Span,
        formatter: &Formatter<'_, '_>,
        writer: &mut Writer<'_>,
    ) {
        let pending = self.pending.take();
        let Some((previous, previous_source, _)) = self.previous.as_ref() else {
            let start = if let Some((JsxSeparator::Space, space)) = pending {
                self.layout.leading_space =
                    Some(formatter.line_constraint(space.end, source.start));
                space.start
            } else {
                source.start
            };
            self.layout.source = Some(Span::new(start, source.end));
            self.layout.leading_inline =
                matches!(next, JsxContentEdge::Literal) && self.layout.leading_space.is_none();
            return;
        };
        let previous = *previous;
        let previous_end = previous_source.end;
        writer.fill_entry(|writer| match pending {
            Some((JsxSeparator::Space, space)) => {
                let before = formatter.line_constraint(previous_end, space.start);
                let following = formatter.line_constraint(space.end, source.start);
                before.write(writer, FlatSeparator::Empty, |_| {});
                write_jsx_space(
                    self.layout.single_quote,
                    JsxSpacePlacement::Between,
                    following,
                    writer,
                );
            }
            Some((JsxSeparator::Words, _)) => {
                let constraint = formatter.line_constraint(previous_end, source.start);
                constraint.write(writer, FlatSeparator::Space, Writer::line);
            }
            Some((JsxSeparator::TextLine, _)) => {
                let single_character_next_to_non_self_closing = matches!(
                    (previous, next),
                    (
                        JsxContentEdge::Word {
                            single_character: true
                        },
                        JsxContentEdge::NonText {
                            self_closing: false
                        },
                    ) | (
                        JsxContentEdge::NonText {
                            self_closing: false
                        },
                        JsxContentEdge::Word {
                            single_character: true
                        },
                    )
                );
                let constraint = formatter.line_constraint(previous_end, source.start);
                constraint.write(writer, FlatSeparator::Empty, |writer| {
                    if single_character_next_to_non_self_closing {
                        writer.soft_line();
                    } else {
                        writer.hard_line();
                    }
                });
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
            None => {
                let constraint = formatter.line_constraint(previous_end, source.start);
                constraint.write(writer, FlatSeparator::Empty, |writer| {
                    match (previous, next) {
                        (JsxContentEdge::Literal, _) | (_, JsxContentEdge::Literal) => {}
                        (JsxContentEdge::NonText { .. }, JsxContentEdge::NonText { .. }) => {
                            writer.hard_line()
                        }
                        (
                            JsxContentEdge::Word { single_character },
                            JsxContentEdge::NonText { self_closing },
                        )
                        | (
                            JsxContentEdge::NonText { self_closing },
                            JsxContentEdge::Word { single_character },
                        ) => {
                            if self_closing && !single_character {
                                writer.hard_line();
                            } else {
                                writer.soft_line();
                            }
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
        self.layout.trailing_inline = matches!(edge, JsxContentEdge::Literal);
    }

    fn finish(
        mut self,
        formatter: &Formatter<'_, '_>,
        writer: &mut Writer<'_>,
    ) -> JsxChildrenLayout {
        if let Some((JsxSeparator::Space, source)) = self.pending {
            self.layout.source.get_or_insert(source).end = source.end;
            if let Some((_, previous, entry)) = self.previous {
                self.layout.trailing_space = true;
                self.layout.trailing_inline = false;
                let before = formatter.line_constraint(previous.end, source.start);
                writer.extend_fill_entry(entry, |writer| {
                    before.write(writer, FlatSeparator::Empty, |_| {});
                    write_jsx_space(
                        self.layout.single_quote,
                        JsxSpacePlacement::Trailing,
                        LineConstraint::Unconstrained,
                        writer,
                    );
                });
            } else {
                self.layout.only_space = true;
                self.layout.has_content = true;
            }
        }
        self.layout.grouped_text = self.contains_text
            && self.layout.leading_space.is_none()
            && !self.layout.trailing_space
            && !self.layout.only_space;
        self.layout
    }
}

#[derive(Clone, Copy)]
enum JsxSpacePlacement {
    Between,
    Leading,
    Trailing,
    Lone,
}

fn write_jsx_space<'ast>(
    single_quote: bool,
    placement: JsxSpacePlacement,
    following: LineConstraint,
    writer: &mut Writer<'ast>,
) {
    let space = if single_quote { "{' '}" } else { "{\" \"}" };
    if matches!(following, LineConstraint::NewLine(_)) {
        // Text space before a required line break has no JSX value.
        writer.text(space);
    } else if matches!(placement, JsxSpacePlacement::Between) {
        writer.if_current_break(space);
        writer.if_current_flat(" ");
    } else {
        writer.if_break(space);
        writer.if_flat(" ");
    }
    if matches!(
        placement,
        JsxSpacePlacement::Between | JsxSpacePlacement::Leading
    ) {
        following.write(writer, FlatSeparator::Empty, Writer::soft_line);
    }
}

/// Every slice still borrows the original JSXText. Only the four JSX formatting
/// characters separate words; Unicode whitespace is never Rust trim whitespace.
struct JsxTextChunks<'ast> {
    remaining: &'ast str,
    meaningful: bool,
}

enum JsxTextChunk<'ast> {
    Word(&'ast str),
    Whitespace { line_breaks: u8 },
}

impl<'ast> Iterator for JsxTextChunks<'ast> {
    type Item = JsxTextChunk<'ast>;

    fn next(&mut self) -> Option<Self::Item> {
        let first = *self.remaining.as_bytes().first()?;
        let whitespace = is_jsx_whitespace(first);
        let mut end = 0;
        let mut line_breaks = 0_u8;
        let mut previous_cr = false;
        for byte in self.remaining.bytes() {
            if is_jsx_whitespace(byte) != whitespace {
                break;
            }
            end += 1;
            // Prettier normalizes physical line endings before reading JSXText.
            if byte == b'\r' || byte == b'\n' && !previous_cr {
                line_breaks = (line_breaks + 1).min(2);
            }
            previous_cr = byte == b'\r';
        }
        // A boundary is an ASCII formatting character, so it cannot split UTF-8.
        let (chunk, remaining) = self.remaining.split_at(end);
        self.remaining = remaining;
        self.meaningful |= !whitespace || line_breaks == 0;
        Some(if whitespace {
            JsxTextChunk::Whitespace { line_breaks }
        } else {
            JsxTextChunk::Word(chunk)
        })
    }
}

fn is_jsx_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

/// Child eligibility and its eventual printer share the actual prepared core.
struct JsxChildExpression {
    original: Expr,
    content: Option<ExpressionContent>,
    space: bool,
    template: bool,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_jsx_children(
        &mut self,
        children: SubRange<JsxChild>,
        writer: &mut Writer<'ast>,
    ) -> Result<JsxChildrenLayout, FormatError> {
        let ast = self.ast;
        let mut stream = JsxChildrenStream::default();
        stream.layout.single_quote = self.options.quote_style == QuoteStyle::Single;
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
                    && expression
                        .as_ref()
                        .is_some_and(|expression| expression.template);
                if expression
                    .as_ref()
                    .is_some_and(|expression| expression.space)
                {
                    previous_ignore = false;
                    stream.separator(JsxSeparator::Space, child.span(ast));
                    continue;
                }
                if let JsxChildData::JsxText(text) = data {
                    let raw = ast.get_utf8(text.value(ast));
                    if jsx_text_needs_literal(raw) {
                        // Opaque text contains a non-formatting character.
                        previous_ignore = false;
                        let source = text.span(ast);
                        stream.before(JsxContentEdge::Literal, source, self, writer);
                        let (entry, ()) =
                            writer.fill_entry_with_handle(|writer| writer.literal_text(raw));
                        stream.after(JsxContentEdge::Literal, source, entry);
                        stream.contains_text = true;
                        continue;
                    }
                    let mut chunks = JsxTextChunks {
                        remaining: raw,
                        meaningful: raw.is_empty(),
                    };
                    let mut first = true;
                    let text_span = text.span(ast);
                    let mut start = text_span.start;
                    while let Some(chunk) = chunks.next() {
                        let end = text_span.end - chunks.remaining.len() as u32;
                        let source = Span::new(start, end);
                        start = end;
                        match chunk {
                            JsxTextChunk::Word(word) => {
                                let mut characters = word.chars();
                                let single_character =
                                    characters.next().is_some() && characters.next().is_none();
                                let edge = JsxContentEdge::Word { single_character };
                                stream.before(edge, source, self, writer);
                                let (entry, ()) =
                                    writer.fill_entry_with_handle(|writer| writer.text(word));
                                stream.after(edge, source, entry);
                                stream.contains_text = true;
                            }
                            JsxTextChunk::Whitespace { line_breaks } => {
                                stream.contains_text |= line_breaks == 0;
                                let last = chunks.remaining.is_empty();
                                let separator = if !first && !last {
                                    JsxSeparator::Words
                                } else if line_breaks == 0 {
                                    JsxSeparator::Space
                                } else if first && last {
                                    JsxSeparator::Newline {
                                        empty_line: line_breaks == 2,
                                    }
                                } else {
                                    JsxSeparator::TextLine
                                };
                                stream.separator(separator, source);
                            }
                        }
                        first = false;
                    }
                    // Ignore follows the original sibling, not its canonical spelling.
                    if chunks.meaningful {
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
                let edge = JsxContentEdge::NonText { self_closing };
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
                        JsxChildData::JsxElement(element) => self
                            .format_jsx_element(element, super::TailOwner::Expression, writer)
                            .map(|_| ()),
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
        let space = matches!(
            printed,
            ExprData::StringLiteral(literal)
                if self.ast.get_wtf8(literal.value(self.ast)).as_str() == Some(" ")
        ) && self.comments.in_range(container.span(self.ast)).is_empty()
            && !matches!(
                self.line_constraint(container.span(self.ast).start, container.span(self.ast).end),
                LineConstraint::NewLine(_)
            );
        Some(JsxChildExpression {
            original,
            content,
            space,
            template: matches!(
                printed,
                ExprData::TemplateLiteral(_) | ExprData::TaggedTemplateExpression(_)
            ),
        })
    }
}

fn jsx_text_needs_literal(raw: &str) -> bool {
    raw.chars().any(|character| {
        matches!(character, '\u{b}' | '\u{c}' | '\u{200b}' | '\u{feff}')
            || !character.is_ascii() && character.is_whitespace()
    })
}
