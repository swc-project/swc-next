//! Conditional type children retain their grammar roles and prepared token gaps.

use swc_next_ecma_ast::{GetSpan, Span, TsConditionalType, TsTypeData, TsTypeNameData};

use super::compound::{PendingTypeHead, PreparedTypeContent};
use super::context::TypeContext;
use super::{
    Formatter, PreparedPrecedingTypePacket, PreparedType, RhsLayout, SuppressedTypeHead,
    SuppressedTypeHeadRole, TypeLayout, TypeOperatorSlot,
};
use crate::formatter::comments::{CommentBinding, CommentBoundary, ExpressionTail};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::expressions::TailOwner;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, IndentStyle, document::Writer};

#[derive(Clone, Copy)]
enum ConditionalTypeSeparator {
    Extends,
    Question,
    Colon,
}

impl ConditionalTypeSeparator {
    fn token(self) -> &'static str {
        match self {
            Self::Extends => "extends",
            Self::Question => "?",
            Self::Colon => ":",
        }
    }
}

struct ConditionalTypeBoundary<'ast> {
    separator: ConditionalTypeSeparator,
    layout: ConditionalTypeLayout<'ast>,
}

enum ConditionalTypeLayout<'ast> {
    Source {
        span: Span,
        opening: TypeLayout,
    },
    Preceding {
        packet: PreparedPrecedingTypePacket<'ast>,
        after: CommentBoundary<'ast>,
    },
    Suppressed(SuppressedTypeHead<'ast>),
    Formatted {
        before: CommentBoundary<'ast>,
        after: CommentBoundary<'ast>,
    },
}

pub(super) struct PreparedConditionalType<'ast> {
    check: ConditionalCheck<'ast>,
    body: ConditionalBody<'ast>,
}

struct ConditionalBody<'ast> {
    node: TsConditionalType,
    extends: PreparedType,
    consequent: PreparedType,
    alternate: PreparedType,
    head: ConditionalTypeBoundary<'ast>,
    question: ConditionalTypeBoundary<'ast>,
    colon: ConditionalTypeBoundary<'ast>,
}

enum ConditionalCheck<'ast> {
    Type(PreparedType),
    Prepared(Box<PreparedTypeContent<'ast>>),
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn conditional_type_rhs_layout(&self, node: TsConditionalType) -> Option<RhsLayout> {
        let ast = self.ast;
        let is_generic = |mut node| loop {
            let types = match ast.ts_type_data(node) {
                TsTypeData::TsParenthesizedType(parentheses) => {
                    node = parentheses.type_annotation(ast);
                    continue;
                }
                TsTypeData::TsUnionType(union) => union.types(ast),
                TsTypeData::TsIntersectionType(intersection) => intersection.types(ast),
                TsTypeData::TsTypeReference(reference) => {
                    return reference.type_arguments(ast).is_some();
                }
                TsTypeData::TsFunctionType(function) => {
                    return function.type_parameters(ast).is_some();
                }
                _ => return false,
            };
            if types.len() != 1 {
                return false;
            }
            let Some(child) = types.first() else {
                return false;
            };
            node = ast.get_node_in_sub_range(child);
        };
        (is_generic(node.check_type(ast)) || is_generic(node.extends_type(ast)))
            .then_some(RhsLayout::BreakAfterOperator)
    }

    fn prepare_conditional_type(
        &self,
        node: TsConditionalType,
    ) -> Result<(PreparedType, ConditionalBody<'ast>), FormatError> {
        let ast = self.ast;
        let mut check = self.prepare_type(node.check_type(ast), ContentMode::Formatted);
        let mut extends = self.prepare_type(node.extends_type(ast), ContentMode::Formatted);
        let mut consequent = self.prepare_type(node.true_type(ast), ContentMode::Formatted);
        let mut alternate = self.prepare_type(node.false_type(ast), ContentMode::Formatted);
        let head = self.prepare_conditional_type_boundary(
            &mut check,
            &mut extends,
            ConditionalTypeSeparator::Extends,
        )?;
        let question = self.prepare_conditional_type_boundary(
            &mut extends,
            &mut consequent,
            ConditionalTypeSeparator::Question,
        )?;
        let colon = self.prepare_conditional_type_boundary(
            &mut consequent,
            &mut alternate,
            ConditionalTypeSeparator::Colon,
        )?;
        Ok((
            check,
            ConditionalBody {
                node,
                extends,
                consequent,
                alternate,
                head,
                question,
                colon,
            },
        ))
    }

    pub(super) fn format_conditional_head(
        &mut self,
        node: TsConditionalType,
        pending: &mut PendingTypeHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedConditionalType<'ast>, FormatError> {
        let (check, body) = self.prepare_conditional_type(node)?;
        let content =
            self.format_movable_type_head(check, pending, TypeContext::ConditionalCheck, writer)?;
        let check = match content {
            PreparedTypeContent::Type(node) => ConditionalCheck::Type(node),
            content => ConditionalCheck::Prepared(Box::new(content)),
        };
        Ok(PreparedConditionalType { check, body })
    }

    pub(super) fn format_conditional_type(
        &mut self,
        node: TsConditionalType,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let (check, body) = self.prepare_conditional_type(node)?;
        self.format_conditional_members(ConditionalCheck::Type(check), body, layout, writer)
    }

    pub(super) fn format_prepared_conditional_type(
        &mut self,
        conditional: PreparedConditionalType<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_type_in_context(
            TsTypeData::TsConditionalType(conditional.body.node),
            context,
            layout,
            writer,
            |formatter, layout, writer| {
                formatter.format_conditional_members(
                    conditional.check,
                    conditional.body,
                    layout,
                    writer,
                )
            },
        )
    }

    fn format_conditional_members(
        &mut self,
        check: ConditionalCheck<'ast>,
        body: ConditionalBody<'ast>,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let options = self.options;
        let ConditionalBody {
            node,
            extends,
            consequent,
            alternate,
            head,
            question,
            colon,
            ..
        } = body;
        let content = |writer: &mut Writer<'ast>| {
            if layout == TypeLayout::ConditionalConsequent {
                writer.if_flat("(");
            }
            let print_head = |writer: &mut Writer<'ast>| {
                let span = node.check_type(self.ast).span(self.ast);
                let output =
                    self.format_type_before_continuation(span, writer, |formatter, writer| {
                        let tail = match check {
                            ConditionalCheck::Type(check) => {
                                let layout =
                                    formatter.continuation_type_layout(span, TypeLayout::Root);
                                formatter.format_prepared_type_with_layout(
                                    check,
                                    TypeContext::ConditionalCheck,
                                    layout,
                                    writer,
                                )?
                            }
                            ConditionalCheck::Prepared(check) => {
                                let layout = formatter
                                    .continuation_type_layout(span, check.opening_layout());
                                formatter.format_prepared_type_content(
                                    *check,
                                    TypeContext::ConditionalCheck,
                                    layout,
                                    writer,
                                )?
                            }
                        };
                        Ok(crate::formatter::expressions::ExpressionOutput::unreported(
                            tail,
                        ))
                    })?;
                self.format_expression_tail(output.pending, writer)?;
                let tail = self.format_conditional_type_extends(head, extends, writer)?;
                self.format_expression_tail(tail, writer)?;
                Ok::<_, FormatError>(())
            };
            if layout == TypeLayout::ConditionalAlternate {
                writer.align(2, print_head)?;
            } else {
                print_head(writer)?;
            }
            let branches = |writer: &mut Writer<'ast>| {
                self.format_conditional_type_branch(
                    question,
                    consequent,
                    TypeLayout::ConditionalConsequent,
                    true,
                    writer,
                    |formatter, tail, writer| formatter.format_expression_tail(tail, writer),
                )?;
                self.format_conditional_type_branch(
                    colon,
                    alternate,
                    TypeLayout::ConditionalAlternate,
                    false,
                    writer,
                    |_, tail, _| Ok(tail),
                )
            };
            let tail = match layout {
                TypeLayout::ConditionalConsequent => match options.indent_style {
                    IndentStyle::Space => writer.align(
                        usize::from(options.indent_width.value()).saturating_sub(2),
                        branches,
                    ),
                    IndentStyle::Tab => branches(writer),
                },
                TypeLayout::ConditionalAlternate => branches(writer),
                _ => writer.indent(branches),
            }?;
            if layout == TypeLayout::ConditionalConsequent {
                self.format_expression_tail(tail, writer)?;
                writer.if_flat(")");
                Ok(None)
            } else {
                Ok(tail)
            }
        };
        if matches!(
            layout,
            TypeLayout::ConditionalConsequent | TypeLayout::ConditionalAlternate
        ) {
            content(writer)
        } else {
            writer.group(false, content)
        }
    }

    fn prepare_conditional_type_boundary(
        &self,
        left: &mut PreparedType,
        right: &mut PreparedType,
        separator: ConditionalTypeSeparator,
    ) -> Result<ConditionalTypeBoundary<'ast>, FormatError> {
        let gap = Span::new(
            left.node.span(self.ast).end,
            right.node.span(self.ast).start,
        );
        let comments = self.comments.in_range(gap);
        let token = separator.token();
        let Some(operator) = self.find_token(gap, comments, token) else {
            return Err(FormatError::unsupported(
                "conditional type separator",
                None,
                gap,
            ));
        };
        let operator_span = Span::new(operator, operator + token.len() as u32);
        let split = comments.partition_point(|comment| comment.span.end <= operator);
        let (before, after) = comments.split_at(split);
        let mut source = false;
        let mut suppressed_head = None;
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            let packet_owner = match (separator, left.data) {
                (ConditionalTypeSeparator::Extends, TsTypeData::TsTypeReference(reference))
                    if reference.type_arguments(self.ast).is_none()
                        && matches!(
                            self.ast.ts_type_name_data(reference.type_name(self.ast)),
                            TsTypeNameData::IdentifierReference(_)
                        ) =>
                {
                    Some(left.node.span(self.ast))
                }
                _ => None,
            };
            let attachment = self.type_attachment_head(right.node);
            let before_head = self.prepare_type_suppression_head(&attachment, before);
            for (run, position) in self.suppressed_comment_runs(before, operator) {
                let selection = self.prepare_type_suppression_run(
                    &attachment,
                    before_head,
                    run,
                    position,
                    super::TypeSuppressionContext::TypeBoundary,
                );
                selection.retain_source_head(right, self.ast);
                if !matches!(position, SuppressionPosition::OwnLine)
                    && let Some(owner) = packet_owner
                    && let Some((packet, after)) = self.prepare_type_operator_boundary(
                        comments,
                        run,
                        owner,
                        right,
                        &selection,
                        TypeOperatorSlot::new(operator_span, LineConstraint::SameLine),
                    )
                {
                    left.source_owner = Some(left.node);
                    return Ok(ConditionalTypeBoundary {
                        separator,
                        layout: ConditionalTypeLayout::Preceding { packet, after },
                    });
                }
                source = true;
                if matches!(position, SuppressionPosition::OwnLine) {
                    right.source_owner = Some(right.node);
                } else {
                    left.source_owner = Some(left.node);
                }
            }
            let head = self.prepare_type_suppression_head(&attachment, after);
            for (run, position) in self.suppressed_comment_runs(after, gap.end) {
                let selection = self.prepare_type_suppression_run(
                    &attachment,
                    head,
                    run,
                    position,
                    super::TypeSuppressionContext::TypeBoundary,
                );
                let selected = selection.target();
                selection.retain_source_head(right, self.ast);
                if let Some(owner) = packet_owner
                    && let Some((packet, after)) = self.prepare_type_operator_boundary(
                        comments,
                        run,
                        owner,
                        right,
                        &selection,
                        TypeOperatorSlot::new(operator_span, LineConstraint::SameLine),
                    )
                {
                    left.source_owner = Some(left.node);
                    return Ok(ConditionalTypeBoundary {
                        separator,
                        layout: ConditionalTypeLayout::Preceding { packet, after },
                    });
                }
                source = true;
                if selected.preceding() {
                    left.source_owner = Some(left.node);
                }
                if !selected.following() {
                    continue;
                }
                // Conditional operands keep their whole attachment target,
                // including own-line unions, independently of source frames.
                self.select_type_source_owner(right, attachment.node, &attachment);
            }
            self.finalize_type_head(right, &attachment, comments);
            if matches!(separator, ConditionalTypeSeparator::Extends) && left.source_owner.is_none()
            {
                suppressed_head = self.prepare_suppressed_type_head(
                    *right,
                    &attachment,
                    gap,
                    before,
                    after,
                    SuppressedTypeHeadRole::ConditionalExtends,
                );
            }
        }
        let binding = self.comment_binding(comments, gap, Some(right.node.span(self.ast)));
        let layout = if let Some(head) = suppressed_head {
            ConditionalTypeLayout::Suppressed(head)
        } else if source || !matches!(binding, CommentBinding::Unbound) {
            let last_end = after
                .last()
                .map_or(operator + token.len() as u32, |comment| comment.span.end);
            // The source boundary owns any line before the operand. Carry that
            // opening forward so a formatted union does not emit a second one.
            let opening = if self.line_breaks(last_end, gap.end) == 0 {
                TypeLayout::Root
            } else {
                TypeLayout::OpeningProvided
            };
            ConditionalTypeLayout::Source { span: gap, opening }
        } else {
            let (before, after) = match separator {
                ConditionalTypeSeparator::Extends => (before, after),
                ConditionalTypeSeparator::Question | ConditionalTypeSeparator::Colon => {
                    self.split_separator_comments(comments, split, gap.start, Some(gap.end))
                }
            };
            ConditionalTypeLayout::Formatted {
                before: self.prepare_comment_boundary(before, gap.start, operator),
                after: self.prepare_comment_boundary(after, operator + token.len() as u32, gap.end),
            }
        };
        Ok(ConditionalTypeBoundary { separator, layout })
    }

    /// The separator owns the complete following packet, including escaped type heads.
    fn format_conditional_type_branch<T>(
        &mut self,
        boundary: ConditionalTypeBoundary<'ast>,
        node: PreparedType,
        layout: TypeLayout,
        continuation: bool,
        writer: &mut Writer<'ast>,
        finish: impl FnOnce(
            &mut Self,
            Option<ExpressionTail<'ast>>,
            &mut Writer<'ast>,
        ) -> Result<T, FormatError>,
    ) -> Result<T, FormatError> {
        let context = TypeContext::Root;
        let head = self.type_movable_head(node, context);
        let emitted_prefix = !matches!(&boundary.layout, ConditionalTypeLayout::Source { .. });
        let leading = match boundary.layout {
            ConditionalTypeLayout::Suppressed(head) => {
                writer.space();
                writer.text(boundary.separator.token());
                writer.space();
                self.format_suppressed_type_head(&head, writer)?;
                &[][..]
            }
            ConditionalTypeLayout::Preceding { packet, after } => {
                self.format_preceding_type_packet(&packet, writer)?;
                writer.space();
                writer.text(boundary.separator.token());
                writer.space();
                after.comments()
            }
            ConditionalTypeLayout::Source { span: gap, .. } => {
                self.format_verbatim(gap, writer)?;
                &[][..]
            }
            ConditionalTypeLayout::Formatted { before, after } => {
                self.format_comment_boundary(&before, writer)?;
                before
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::line);
                writer.text(boundary.separator.token());
                writer.space();
                after.comments()
            }
        };
        let indent_style = self.options.indent_style;
        let print = |writer: &mut Writer<'ast>| {
            let mut pending = PendingTypeHead::new(leading, node.node.span(self.ast).start, head);
            if emitted_prefix {
                pending = pending.with_emitted_prefix();
            }
            let content = self.format_movable_type_head(node, &mut pending, context, writer)?;
            let span = node.node.span(self.ast);
            let tail = if continuation {
                let layout = self.continuation_type_layout(span, layout);
                self.format_type_before_continuation(span, writer, |formatter, writer| {
                    formatter
                        .format_prepared_type_content(content, context, layout, writer)
                        .map(crate::formatter::expressions::ExpressionOutput::unreported)
                })?
                .pending
            } else {
                self.format_prepared_type_content(content, context, layout, writer)?
            };
            finish(self, tail, writer)
        };
        match indent_style {
            IndentStyle::Space => writer.align(2, print),
            IndentStyle::Tab => writer.indent(print),
        }
    }

    fn format_conditional_type_extends(
        &mut self,
        boundary: ConditionalTypeBoundary<'ast>,
        node: PreparedType,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let layout = match boundary.layout {
            ConditionalTypeLayout::Suppressed(head) => {
                writer.space();
                writer.text(boundary.separator.token());
                writer.space();
                self.format_suppressed_type_head(&head, writer)?;
                TypeLayout::Root
            }
            ConditionalTypeLayout::Preceding { packet, after } => {
                // Later conditional boundaries can select this operand as a raw
                // owner. Resolve its complete head from that final selection.
                let head = self.prepare_binary_type_head(
                    node,
                    TypeContext::ConditionalExtends,
                    &after,
                    after.source_end(),
                    TailOwner::ParentOperation,
                );
                self.format_preceding_type_packet(&packet, writer)?;
                writer.space();
                writer.text(boundary.separator.token());
                writer.space();
                let span = node.node.span(self.ast);
                return self
                    .format_type_before_continuation(span, writer, |formatter, writer| {
                        formatter
                            .format_binary_type_rhs_head(
                                node,
                                TypeContext::ConditionalExtends,
                                &after,
                                head,
                                writer,
                            )
                            .map(crate::formatter::expressions::ExpressionOutput::unreported)
                    })
                    .map(|output| output.pending);
            }
            ConditionalTypeLayout::Source { span, opening } => {
                self.format_source_gap(span, writer)?;
                opening
            }
            ConditionalTypeLayout::Formatted { before, after } => {
                self.format_comment_boundary(&before, writer)?;
                before.gap().write(writer, FlatSeparator::Space, |writer| {
                    match boundary.separator {
                        // A conditional check and `extends` must remain on the same line.
                        ConditionalTypeSeparator::Extends => writer.space(),
                        ConditionalTypeSeparator::Question | ConditionalTypeSeparator::Colon => {
                            writer.line();
                        }
                    }
                });
                writer.text(boundary.separator.token());
                self.format_comment_boundary(&after, writer)?;
                after
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
                TypeLayout::Root
            }
        };
        let span = node.node.span(self.ast);
        let layout = self.continuation_type_layout(span, layout);
        self.format_type_before_continuation(span, writer, |formatter, writer| {
            formatter.format_prepared_type_output(
                node,
                TypeContext::ConditionalExtends,
                layout,
                writer,
            )
        })
        .map(|output| output.pending)
    }
}
