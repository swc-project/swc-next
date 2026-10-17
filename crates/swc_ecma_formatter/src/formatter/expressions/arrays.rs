//! Array slots, comment boundaries, and grouped delimiter layout.

use swc_next_ecma_ast::{
    Argument, ArgumentData, ArrayExpression, CommentKind, Expr, ExprData, GetSpan, Span,
    UnaryOperator,
};

use super::Formatter;
use super::arguments::{
    ArgumentLayout, ArrayArgumentSlot, ElisionLayout, FollowingArgument, Separator,
};
use super::parentheses::NumericContentView;
use crate::formatter::comments::{ArrayCommentOwner, CommentRunLayout};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::ContentMode;
use crate::{
    FormatError, TrailingCommas,
    document::{Fragment, Writer},
};

#[derive(Clone, Copy)]
pub(super) enum ArrayArgumentArity {
    Single,
    Multiple,
}

enum SourceArray<'ast> {
    Verbatim(Span),
    Empty,
    Formatted(PreparedArray<'ast>),
}

struct PreparedArray<'ast> {
    array: ArrayExpression,
    content: Span,
    expand: bool,
    opening: LineConstraint,
    layout: ArgumentLayout,
    numeric_comments: Option<ArrayCommentOwner<'ast>>,
}

struct NumericArrayFacts {
    fill_first: Option<Argument>,
    reference: Option<ReferenceNumericBounds>,
}

struct ReferenceNumericBounds {
    first: Expr,
    last: Expr,
}

#[derive(Clone, Copy)]
pub(super) struct RecordedArray<'ast> {
    pub(super) normal: Fragment<'ast>,
    pub(super) forced: Option<Fragment<'ast>>,
}

pub(super) enum RecordedArrayArgument<'ast> {
    Ordinary(Fragment<'ast>),
    Selected(RecordedArray<'ast>),
}

struct ArgumentItem {
    holes: usize,
    argument: Option<Argument>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ArrayChildKind {
    Array,
    Object,
}

fn next_item(elements: &mut impl Iterator<Item = Option<Argument>>) -> Option<ArgumentItem> {
    let mut holes = 0;
    for element in elements.by_ref() {
        if let Some(argument) = element {
            return Some(ArgumentItem {
                holes,
                argument: Some(argument),
            });
        }
        holes += 1;
    }
    (holes != 0).then_some(ArgumentItem {
        holes,
        argument: None,
    })
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn format_array(
        &mut self,
        array: ArrayExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let source = self.source_array(array, false)?;
        self.format_source_array(source, writer)
    }

    fn source_array(
        &mut self,
        array: ArrayExpression,
        qualify: bool,
    ) -> Result<SourceArray<'ast>, FormatError> {
        let elements = array.elements(self.ast);
        let span = array.span(self.ast);
        let content = Span::new(span.start + 1, span.end - 1);
        if self.has_suppression
            && elements
                .iter()
                .all(|id| self.ast.get_node_in_sub_range(id).is_none())
            && self.content_mode(self.comments.in_range(content)) == ContentMode::Verbatim
        {
            return Ok(SourceArray::Verbatim(span));
        }
        if elements.is_empty() && self.comments.between(content)?.is_empty() {
            return Ok(SourceArray::Empty);
        }
        let expand = self.array_should_expand(array);
        let numeric = if qualify {
            self.numeric_array_facts(array)
        } else {
            NumericArrayFacts {
                fill_first: self.numeric_array_first(array),
                reference: None,
            }
        };
        let opening = self.opening_line_constraint(content);
        let (layout, numeric_comments) = match numeric.fill_first {
            Some(first) => {
                // Keep this old fallible query even if qualification demotes later.
                let comments = self.comments.between(content)?;
                let first_end = first.span(self.ast).end;
                let fill_line = comments.iter().any(|comment| {
                    comment.span.start >= first_end
                        && comment.kind == CommentKind::Line
                        && !self.source_position_starts_line(comment.span.start)
                });
                let layout = if fill_line {
                    ArgumentLayout::Lines
                } else {
                    ArgumentLayout::Fill
                };
                (
                    layout,
                    numeric.reference.map(|bounds| {
                        ArrayCommentOwner::new(
                            comments,
                            bounds.first.span(self.ast).end,
                            bounds.last.span(self.ast).start,
                        )
                    }),
                )
            }
            None => {
                // A reference-only number behind source protection must not add between.
                let comments = numeric.reference.map(|bounds| {
                    ArrayCommentOwner::new(
                        self.comments.in_range(content),
                        bounds.first.span(self.ast).end,
                        bounds.last.span(self.ast).start,
                    )
                });
                (ArgumentLayout::Lines, comments)
            }
        };
        Ok(SourceArray::Formatted(PreparedArray {
            array,
            content,
            expand,
            opening,
            layout,
            numeric_comments,
        }))
    }

    fn format_source_array(
        &mut self,
        source: SourceArray<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match source {
            SourceArray::Verbatim(span) => self.format_verbatim(span, writer),
            SourceArray::Empty => {
                writer.text("[]");
                Ok(())
            }
            SourceArray::Formatted(source) => writer.group(source.expand, |writer| {
                self.format_array_contents(source, writer).map(|_| ())
            }),
        }
    }

    pub(super) fn record_array(
        &mut self,
        array: ArrayExpression,
        arity: ArrayArgumentArity,
        writer: &mut Writer<'ast>,
    ) -> Result<RecordedArrayArgument<'ast>, FormatError> {
        let qualify = matches!(arity, ArrayArgumentArity::Multiple);
        let source = self.source_array(array, qualify)?;
        let source = match source {
            SourceArray::Formatted(source) => source,
            source => {
                let (value, ()) =
                    writer.record_with_value(|writer| self.format_source_array(source, writer))?;
                return Ok(RecordedArrayArgument::Ordinary(value));
            }
        };
        let expand = source.expand;
        let (content, concise) =
            writer.record_with_value(|writer| self.format_array_contents(source, writer))?;
        let (normal, ()) = writer.record_with_value(|writer| {
            writer.group(expand, |writer| writer.write_fragment(content));
            Ok::<_, FormatError>(())
        })?;
        if concise {
            return Ok(RecordedArrayArgument::Ordinary(normal));
        }
        let forced = if normal.preferred_breaks() {
            None
        } else {
            let (forced, ()) = writer.record_with_value(|writer| {
                writer.group(true, |writer| writer.write_fragment(content));
                Ok::<_, FormatError>(())
            })?;
            Some(forced)
        };
        Ok(RecordedArrayArgument::Selected(RecordedArray {
            normal,
            forced,
        }))
    }

    fn format_array_contents(
        &mut self,
        source: PreparedArray<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<bool, FormatError> {
        let PreparedArray {
            array,
            content,
            opening,
            layout,
            mut numeric_comments,
            ..
        } = source;
        writer.text("[");
        let end = writer.indent(|writer| {
            opening.write(writer, FlatSeparator::Empty, Writer::soft_line);
            match layout {
                ArgumentLayout::Lines => self.format_array_items(
                    array,
                    content,
                    layout,
                    numeric_comments.as_mut(),
                    writer,
                ),
                ArgumentLayout::Fill => writer.fill(|writer| {
                    self.format_array_items(
                        array,
                        content,
                        layout,
                        numeric_comments.as_mut(),
                        writer,
                    )
                }),
            }
        })?;
        self.line_constraint(end, content.end).write(
            writer,
            FlatSeparator::Empty,
            Writer::soft_line,
        );
        writer.text("]");
        if let Some(owner) = numeric_comments.as_mut() {
            owner.finish_source(self, content.end);
        }
        Ok(numeric_comments
            .as_ref()
            .is_some_and(ArrayCommentOwner::is_concise))
    }

    /// Selected Multiple qualification reduces two decisions in one element pass.
    fn numeric_array_facts(&self, array: ArrayExpression) -> NumericArrayFacts {
        let mut elements = array
            .elements(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id));
        let Some(Some(first)) = elements.next() else {
            return NumericArrayFacts {
                fill_first: None,
                reference: None,
            };
        };
        let mut fill = true;
        let mut reference = true;
        let mut reference_bounds = None;
        for argument in std::iter::once(Some(first)).chain(elements) {
            let Some(argument) = argument else {
                return NumericArrayFacts {
                    fill_first: None,
                    reference: None,
                };
            };
            let ArgumentData::Expr(expression) = self.ast.argument_data(argument) else {
                return NumericArrayFacts {
                    fill_first: None,
                    reference: None,
                };
            };
            let view = match (fill, reference) {
                (true, true) => NumericContentView::Both,
                (true, false) => NumericContentView::Source,
                (false, true) => NumericContentView::Reference,
                (false, false) => {
                    return NumericArrayFacts {
                        fill_first: None,
                        reference: None,
                    };
                }
            };
            let content = self.numeric_argument_content(expression, view);
            let source_numeric = matches!(content.source, Some(ExprData::NumericLiteral(_)));
            let reference_root = content.reference;
            let reference_numeric = reference_root
                .as_ref()
                .is_some_and(|root| matches!(root.data, ExprData::NumericLiteral(_)));
            let source_unary = match content.source {
                Some(ExprData::UnaryExpression(unary)) => Some(unary),
                _ => None,
            };
            let reference_unary = reference_root.as_ref().and_then(|root| match root.data {
                ExprData::UnaryExpression(unary) => Some(unary),
                _ => None,
            });
            let mut signed_source = false;
            let mut signed_reference = false;
            let signed = match (source_unary, reference_unary) {
                (Some(unary), Some(_)) => Some((unary, NumericContentView::Both)),
                (Some(unary), None) => Some((unary, NumericContentView::Source)),
                (None, Some(unary)) => Some((unary, NumericContentView::Reference)),
                (None, None) => None,
            };
            if let Some((unary, view)) = signed
                && matches!(
                    unary.operator(self.ast),
                    UnaryOperator::Positive | UnaryOperator::Negate
                )
            {
                let actual_operand = unary.argument(self.ast);
                let operand = self.numeric_argument_content(actual_operand, view);
                let source_operand = matches!(operand.source, Some(ExprData::NumericLiteral(_)));
                let reference_operand = operand
                    .reference
                    .as_ref()
                    .is_some_and(|root| matches!(root.data, ExprData::NumericLiteral(_)));
                if source_operand || reference_operand {
                    // Under the numeric reference proof, actual_operand's source
                    // envelope has exactly one normalized Numeric child. Both the
                    // operator-to-child gap and its trailing envelope attach to
                    // that child; comments outside this unary span do not.
                    let no_comments = self.comments.in_range(unary.span(self.ast)).is_empty();
                    signed_source = source_unary.is_some() && source_operand && no_comments;
                    signed_reference =
                        reference_unary.is_some() && reference_operand && no_comments;
                }
            }
            fill &= source_numeric || signed_source;
            reference &= reference_numeric || signed_reference;
            if reference && let Some(root) = reference_root {
                reference_bounds = Some(match reference_bounds {
                    Some(ReferenceNumericBounds { first, .. }) => ReferenceNumericBounds {
                        first,
                        last: root.expression,
                    },
                    None => ReferenceNumericBounds {
                        first: root.expression,
                        last: root.expression,
                    },
                });
            }
            if !fill && !reference {
                return NumericArrayFacts {
                    fill_first: None,
                    reference: None,
                };
            }
        }
        NumericArrayFacts {
            fill_first: fill.then_some(first),
            reference: if reference { reference_bounds } else { None },
        }
    }

    fn array_should_expand(&self, array: ArrayExpression) -> bool {
        let elements = array.elements(self.ast);
        if elements.len() < 2 {
            return false;
        }
        let mut kinds = elements.iter().map(|id| {
            let argument = self.ast.get_node_in_sub_range(id)?;
            let ArgumentData::Expr(expression) = self.ast.argument_data(argument) else {
                return None;
            };
            match self.expression_content(expression).data {
                ExprData::ArrayExpression(child) if child.elements(self.ast).len() > 1 => {
                    Some(ArrayChildKind::Array)
                }
                ExprData::ObjectExpression(child) if child.properties(self.ast).len() > 1 => {
                    Some(ArrayChildKind::Object)
                }
                _ => None,
            }
        });
        let Some(Some(kind)) = kinds.next() else {
            return false;
        };
        kinds.all(|next| next == Some(kind))
    }

    fn numeric_array_first(&self, array: ArrayExpression) -> Option<Argument> {
        let mut elements = array
            .elements(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id));
        let first = elements.next().flatten()?;
        for argument in std::iter::once(Some(first)).chain(elements) {
            let argument = argument?;
            let ArgumentData::Expr(expression) = self.ast.argument_data(argument) else {
                return None;
            };
            let data = self.expression_content(expression).data;
            let numeric = match data {
                ExprData::NumericLiteral(_) => true,
                ExprData::UnaryExpression(unary)
                    if matches!(
                        unary.operator(self.ast),
                        UnaryOperator::Positive | UnaryOperator::Negate
                    ) =>
                {
                    let argument = self.expression_content(unary.argument(self.ast)).data;
                    matches!(argument, ExprData::NumericLiteral(_))
                        && self.comments.in_range(unary.span(self.ast)).is_empty()
                }
                _ => false,
            };
            if !numeric {
                return None;
            }
        }
        Some(first)
    }

    fn format_array_items(
        &mut self,
        array: ArrayExpression,
        span: Span,
        layout: ArgumentLayout,
        mut numeric_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<u32, FormatError> {
        let ast = self.ast;
        let mut elements = array
            .elements(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id));
        let mut items = std::iter::from_fn(|| next_item(&mut elements)).peekable();
        let mut previous_end = span.start;
        let mut has_slot = false;
        let mut separator = Separator::Line;
        let mut following_mode = None;
        let mut elisions = ElisionLayout::Formatted;
        let constrained = self.diagnostic_range_is_sensitive(span);
        while let Some(item) = items.next() {
            // Resolve the initial source prefix before its elisions claim comments.
            // Later real items receive their binding from the preceding source gap.
            let mode = item.argument.map_or(ContentMode::Formatted, |argument| {
                following_mode.unwrap_or_else(|| {
                    if self.has_suppression {
                        self.content_mode(
                            self.comments
                                .in_range(Span::new(span.start, argument.span(ast).start)),
                        )
                    } else {
                        ContentMode::Formatted
                    }
                })
            });
            let source_elisions = match elisions {
                ElisionLayout::Formatted => 0,
                ElisionLayout::Source(holes) => {
                    debug_assert_eq!(
                        holes, item.holes,
                        "a source elision boundary covers the prepared AST slots"
                    );
                    holes
                }
            };
            for _ in source_elisions..item.holes {
                let end = item
                    .argument
                    .map_or(span.end, |argument| argument.span(ast).start);
                let gap = Span::new(previous_end, end);
                let comments = self.comments.between(gap)?;
                if constrained || !comments.is_empty() {
                    let Some(comma) = self.find_token(gap, comments, ",") else {
                        return Err(FormatError::unsupported(
                            "array elision boundary",
                            None,
                            gap,
                        ));
                    };
                    let count = comments.partition_point(|comment| comment.span.end <= comma);
                    let comments = self.comments.take(count);
                    match (comments.first(), has_slot) {
                        (Some(first), has_slot) => {
                            if has_slot {
                                self.comment_gap(previous_end, first.span.start, writer);
                            }
                            self.format_comment_run(
                                comments,
                                CommentRunLayout::Leading(comma),
                                writer,
                            )?;
                        }
                        (None, true) => layout.separator(
                            separator,
                            self.line_constraint(previous_end, comma),
                            writer,
                        ),
                        (None, false) => self.line_constraint(previous_end, comma).write(
                            writer,
                            FlatSeparator::Empty,
                            |_| {},
                        ),
                    }
                    writer.text(",");
                    previous_end = comma + 1;
                    separator = Separator::Line;
                    has_slot = true;
                    continue;
                }
                if has_slot {
                    layout.separator(separator, LineConstraint::Unconstrained, writer);
                    separator = Separator::Line;
                }
                writer.text(",");
                has_slot = true;
            }
            let Some(argument) = item.argument else {
                continue;
            };
            let argument_span = argument.span(ast);
            if has_slot {
                let next_start = self
                    .comments
                    .between(Span::new(previous_end, argument_span.start))?
                    .first()
                    .map_or(argument_span.start, |comment| comment.span.start);
                layout.separator(
                    separator,
                    self.line_constraint(previous_end, next_start),
                    writer,
                );
            }
            let boundary = layout.entry(writer, |writer| {
                self.format_argument_item_with_array_comments(
                    ArrayArgumentSlot {
                        argument,
                        mode,
                        layout,
                        following: match items.peek() {
                            Some(item) if item.holes != 0 => FollowingArgument::Elision {
                                argument: item.argument,
                                holes: item.holes,
                            },
                            Some(item) => item
                                .argument
                                .map_or(FollowingArgument::End, FollowingArgument::Argument),
                            None => FollowingArgument::End,
                        },
                    },
                    Span::new(previous_end, span.end),
                    self.options.trailing_commas != TrailingCommas::None,
                    numeric_comments.as_deref_mut(),
                    writer,
                )
            })?;
            previous_end = boundary.end;
            separator = boundary.separator;
            following_mode = Some(boundary.following_mode);
            elisions = boundary.elisions;
            has_slot = true;
        }
        let dangling = self.comments.between(Span::new(previous_end, span.end))?;
        let dangling = self.comments.take(dangling.len());
        if dangling
            .iter()
            .any(|comment| comment.kind == CommentKind::Line)
        {
            writer.break_parent();
        }
        if has_slot && let Some(first) = dangling.first() {
            self.comment_gap(previous_end, first.span.start, writer);
        }
        self.format_comment_run_with_array_comments(
            dangling,
            CommentRunLayout::Lines,
            numeric_comments,
            writer,
        )?;
        Ok(dangling
            .last()
            .map_or(previous_end, |comment| comment.span.end))
    }
}
