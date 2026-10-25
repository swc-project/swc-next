//! Typed comma-expression traversal and borrowed separator boundaries.

use swc_next_ecma_ast::{Comment, CommentKind, Expr, ExprData, GetSpan, SequenceExpression, Span};

use super::parentheses::{ExpressionContent, ResolvedExpressionHead, SourceParentheses};
use super::{ExpressionContext, ExpressionPosition, Formatter, HeadComments, TailOwner};
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{ContentMode, DelimitedFollowing, DelimitedSuppression};
use crate::{FormatError, Semicolons, document::Writer};

struct SequenceBoundary<'ast> {
    next: Expr,
    next_content: Option<ExpressionContent>,
    next_position: ExpressionPosition,
    next_start: u32,
    source_end: u32,
    comma: Option<u32>,
    trailing: &'ast [Comment],
    leading: &'ast [Comment],
    suppression: Option<DelimitedSuppression>,
    packet: Option<SequenceTrailingPacket<'ast>>,
    before_comma: LineConstraint,
    separator: LineConstraint,
}

/// A complete source-inline EOL run and its resolved source ownership.
struct SequenceTrailingPacket<'ast> {
    comments: &'ast [Comment],
    binding: SequencePacketBinding,
}

#[derive(Clone, Copy)]
enum SequencePacketBinding {
    /// The outside Line suffix stays on its original comma side.
    InlineLine {
        source: Span,
    },
    InactiveOptimizer,
    IgnoredPrevious(Span),
}

enum SequenceComments {
    Formatted,
    SourceGap,
}

struct SequenceCommentPartition<'ast> {
    comments: &'ast [Comment],
    trailing: &'ast [Comment],
    leading: &'ast [Comment],
    before_count: usize,
    packet_end: Option<usize>,
    sensitive: bool,
}

struct SequenceItem<'ast> {
    expression: Expr,
    prepared: Option<ExpressionContent>,
    head: NodeHead<'ast>,
    mode: ContentMode,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn sequence_grammar_pair_is_enclosed(position: ExpressionPosition) -> bool {
        matches!(
            position,
            ExpressionPosition::ArrowBody | ExpressionPosition::RestrictedArgument
        )
    }
    fn sequence_member_pair_is_enclosed(
        &self,
        expression: Expr,
        content: &ExpressionContent,
        position: ExpressionPosition,
    ) -> bool {
        if content.parentheses.is_none()
            && !matches!(content.data, ExprData::ParenthesizedExpression(_))
        {
            return false;
        }
        let context = ExpressionContext::at(position);
        self.jsx_source_pair_is_enclosed(expression, content, context)
            || content.parentheses.is_some()
                && self.expression_is_parenthesized(content.data, context)
    }

    pub(super) fn sequence_member_parentheses(
        &self,
        content: &ExpressionContent,
        context: ExpressionContext<'ast>,
    ) -> Option<SourceParentheses> {
        if context.content_mode != ContentMode::Formatted
            || !matches!(
                context.position,
                ExpressionPosition::SequenceMember | ExpressionPosition::ForSequenceMember
            )
        {
            return None;
        }
        let removed = content.parentheses?;
        (removed.head.has_comments && self.expression_is_parenthesized(content.data, context))
            .then_some(removed.source)
    }

    /// A real enclosing pair can own the ordinary head exposed by removing the
    /// first member's source envelope. Bound packets stay with that member.
    pub(super) fn enclosed_sequence_head_start(&self, sequence: SequenceExpression) -> Option<u32> {
        let first = self
            .ast
            .get_node_in_sub_range(sequence.expressions(self.ast).first()?);
        let content = self.expression_content(first);
        let removed = content.parentheses?;
        if !removed.head.has_comments
            || !matches!(removed.head.binding, CommentBinding::Unbound)
            || self.sequence_member_pair_is_enclosed(
                first,
                &content,
                ExpressionPosition::SequenceMember,
            )
        {
            return None;
        }
        let start = content.expression.span(self.ast).start;
        let comments = self.ast.comments();
        let first =
            comments.partition_point(|comment| comment.span.start < sequence.span(self.ast).start);
        let comments = &comments[first..];
        let count = comments.partition_point(|comment| comment.span.end <= start);
        matches!(
            self.expression_head_binding(&comments[..count], content.expression),
            CommentBinding::Unbound
        )
        .then_some(start)
    }

    pub(super) fn sequence_head_has_source_cast(&self, sequence: SequenceExpression) -> bool {
        sequence.expressions(self.ast).first().is_some_and(|id| {
            self.source_cast_head(self.ast.get_node_in_sub_range(id))
                .is_some()
        })
    }

    pub(super) fn format_sequence(
        &mut self,
        sequence: SequenceExpression,
        context: ExpressionContext<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_sequence_with_leading_comments(sequence, &[], context, writer)
    }

    /// An enclosing pair can lease its claimed head to the first real member.
    /// That member owns any additional grammar pair before printing the packet.
    pub(super) fn format_sequence_with_leading_comments(
        &mut self,
        sequence: SequenceExpression,
        inherited: &'ast [Comment],
        mut context: ExpressionContext<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let mut expressions = sequence
            .expressions(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id));
        let Some(first) = expressions.next() else {
            return Err(self.unsupported("empty sequence expression", sequence));
        };
        let item_position = if matches!(
            context.position,
            ExpressionPosition::ForInitializer {
                grammar: super::ForInitializerGrammar::Expression,
                ..
            } | ExpressionPosition::ForHead
        ) {
            ExpressionPosition::ForSequenceMember
        } else {
            ExpressionPosition::SequenceMember
        };
        let first_content = self.expression_content(first);
        let first_start =
            if self.sequence_member_pair_is_enclosed(first, &first_content, item_position)
                || self.diagnostic_head_has_frame(
                    &first_content,
                    ExpressionContext::at(item_position).with_parent_head(context),
                )
            {
                first.span(ast).start
            } else {
                first_content.expression.span(ast).start
            };
        let enclosed_head = matches!(context.head_comments,
            HeadComments::ClaimedSequence(owner) if owner == sequence);
        if enclosed_head {
            context.head_comments = HeadComments::Pending;
        }
        let leading = if enclosed_head || !inherited.is_empty() {
            &[]
        } else {
            self.comments
                .between(Span::new(sequence.span(ast).start, first_start))?
        };
        self.comments.claim(leading)?;
        let head = NodeHead {
            comments: leading,
            content_start: first_start,
            layout: CommentRunLayout::Leading(first_start),
            gap_end: None,
        };
        let parenthesize =
            self.expression_is_parenthesized(ExprData::SequenceExpression(sequence), context);
        if parenthesize && !leading.is_empty() {
            let binding = self.expression_head_binding(leading, first);
            let binding = first_content
                .parentheses
                .map_or(binding, |removed| binding.combine(removed.head.binding));
            if !matches!(binding, CommentBinding::Unbound) {
                return self.format_prepared_expression_with_resolved_head(
                    Expr::SequenceExpression(sequence),
                    ExpressionContent {
                        expression: Expr::SequenceExpression(sequence),
                        data: ExprData::SequenceExpression(sequence),
                        parentheses: None,
                    },
                    ResolvedExpressionHead { head, binding },
                    context,
                    writer,
                );
            }
        }
        let generated = if parenthesize && self.has_type_casts && !leading.is_empty() {
            let content = ExpressionContent {
                expression: Expr::SequenceExpression(sequence),
                data: ExprData::SequenceExpression(sequence),
                parentheses: None,
            };
            self.generated_enclosure_owner(
                Expr::SequenceExpression(sequence),
                &content,
                &head,
                CommentBinding::Unbound,
                context,
            )
            .map(|owner| super::parentheses::PreparedGeneratedEnclosure { owner, content })
        } else {
            None
        };
        if let Some(enclosure) = generated {
            context.expression_start =
                self.format_node_head(head, context.expression_start, None, writer)?;
            return self.format_prepared_generated_enclosure(enclosure, context, writer);
        }
        let start = if enclosed_head {
            context.expression_start
        } else {
            self.format_node_head(head, context.expression_start, None, writer)?
        };
        if self.options.semicolons == Semicolons::AsNeeded && start.needs_asi_guard() {
            writer.text(";");
        }
        if parenthesize {
            writer.text("(");
            context.head_scope = None;
            context = self.with_head_scope(
                context,
                &ExpressionContent {
                    expression: Expr::SequenceExpression(sequence),
                    data: ExprData::SequenceExpression(sequence),
                    parentheses: None,
                },
            );
        }
        let arrow_body = parenthesize && Self::sequence_grammar_pair_is_enclosed(context.position);
        let tail = writer.group(false, |writer| {
            let write_items = |writer: &mut Writer<'ast>| {
                if arrow_body {
                    writer.soft_line();
                }
                let boundary = expressions
                    .next()
                    .map(|next| self.sequence_boundary(first, next, item_position))
                    .transpose()?;
                let tail = self.format_sequence_item(
                    SequenceItem {
                        expression: first,
                        prepared: Some(first_content),
                        head: NodeHead {
                            comments: inherited,
                            content_start: first_start,
                            layout: CommentRunLayout::FlatLeading(first_start),
                            gap_end: None,
                        },
                        mode: ContentMode::Formatted,
                    },
                    boundary.as_ref(),
                    ExpressionContext::at(item_position)
                        .with_parent_head(context)
                        .with_expression_start(if parenthesize {
                            super::ExpressionStart::None
                        } else {
                            context.expression_start
                        })
                        .with_tail_owner(context.tail_owner),
                    writer,
                )?;
                let Some(mut boundary) = boundary else {
                    return Ok(tail);
                };
                self.format_sequence_trailing(&mut boundary, tail, writer)?;
                let mut write_rest = |writer: &mut Writer<'ast>| loop {
                    boundary
                        .separator
                        .write(writer, FlatSeparator::Space, Writer::line);
                    let following = expressions
                        .next()
                        .map(|next| self.sequence_boundary(boundary.next, next, item_position))
                        .transpose()?;
                    self.comments.claim(boundary.leading)?;
                    let tail = self.format_sequence_item(
                        SequenceItem {
                            expression: boundary.next,
                            prepared: boundary.next_content.take(),
                            head: NodeHead {
                                comments: boundary.leading,
                                content_start: boundary.next_start,
                                layout: CommentRunLayout::Leading(boundary.next_start),
                                gap_end: None,
                            },
                            mode: boundary.suppression.as_ref().map_or_else(
                                || self.content_mode(boundary.leading),
                                |source| source.following,
                            ),
                        },
                        following.as_ref(),
                        ExpressionContext::at(item_position).with_tail_owner(context.tail_owner),
                        writer,
                    )?;
                    let Some(mut following) = following else {
                        break Ok(tail);
                    };
                    self.format_sequence_trailing(&mut following, tail, writer)?;
                    boundary = following;
                };
                if matches!(
                    context.position,
                    ExpressionPosition::Statement
                        | ExpressionPosition::ForHead
                        | ExpressionPosition::ForInitializer {
                            grammar: super::ForInitializerGrammar::Expression,
                            ..
                        }
                ) {
                    writer.indent(write_rest)
                } else {
                    write_rest(writer)
                }
            };
            let tail = if arrow_body {
                writer.indent(write_items)?
            } else {
                write_items(writer)?
            };
            let tail = if parenthesize {
                self.format_parentheses_tail(context.source_parentheses_tail, tail, None, writer)?
            } else {
                tail
            };
            if arrow_body {
                writer.soft_line();
            }
            Ok(tail)
        })?;
        if parenthesize {
            writer.text(")");
        }
        Ok(tail)
    }

    fn format_sequence_item(
        &mut self,
        item: SequenceItem<'ast>,
        following: Option<&SequenceBoundary<'ast>>,
        context: ExpressionContext<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let SequenceItem {
            expression,
            prepared,
            head,
            mode,
        } = item;
        let mode = if mode == ContentMode::Verbatim {
            mode
        } else {
            following.map_or(ContentMode::Formatted, |boundary| {
                if boundary
                    .suppression
                    .as_ref()
                    .and_then(|source| source.preceding)
                    .is_some_and(|target| target == expression.span(self.ast))
                {
                    ContentMode::Verbatim
                } else {
                    ContentMode::Formatted
                }
            })
        };
        let tail_owner = if following.is_some() {
            TailOwner::ParentOperation
        } else {
            context.tail_owner
        };
        let binding = self.expression_head_binding(head.comments, expression);
        let binding = prepared
            .as_ref()
            .and_then(|content| content.parentheses)
            .map_or(binding, |removed| binding.combine(removed.head.binding));
        self.format_expression_leading_prepared(
            expression,
            prepared,
            ResolvedExpressionHead { head, binding },
            context.with_content_mode(mode).with_tail_owner(tail_owner),
            None,
            writer,
        )
    }

    fn sequence_boundary(
        &self,
        previous: Expr,
        next: Expr,
        position: ExpressionPosition,
    ) -> Result<SequenceBoundary<'ast>, FormatError> {
        let start = previous.span(self.ast).end;
        let suppression = self.delimited_suppression(
            previous.span(self.ast),
            next.span(self.ast).start,
            DelimitedFollowing::Node,
        );
        let end = next.span(self.ast).start;
        let span = Span::new(start, end);
        // Retain the borrowed run while the previous operand consumes its own
        // comments. Its returned tail determines the remaining suffix.
        let comments = self
            .comments
            .in_range(Span::new(previous.span(self.ast).start, end));
        let outside = comments.partition_point(|comment| comment.span.end <= start);
        let outside_comments = &comments[outside..];
        let comma = if comments.is_empty() && !self.diagnostic_range_is_sensitive(span) {
            None
        } else {
            let Some(comma) = self.find_token(span, outside_comments, ",") else {
                return Err(FormatError::unsupported(
                    "sequence separator boundary",
                    None,
                    span,
                ));
            };
            Some(comma)
        };
        let packet = comma.and_then(|comma| {
            self.sequence_trailing_packet(previous, next, comma, outside_comments, suppression)
        });
        Ok(SequenceBoundary {
            next,
            next_content: None,
            next_position: position,
            next_start: end,
            source_end: start,
            comma,
            trailing: &[],
            leading: &[],
            suppression,
            packet,
            before_comma: LineConstraint::Unconstrained,
            separator: LineConstraint::Unconstrained,
        })
    }

    fn sequence_trailing_packet(
        &self,
        previous: Expr,
        next: Expr,
        comma: u32,
        comments: &'ast [Comment],
        suppression: Option<DelimitedSuppression>,
    ) -> Option<SequenceTrailingPacket<'ast>> {
        let source = Span::new(previous.span(self.ast).end, next.span(self.ast).start);
        if self.line_breaks(source.start, comma) != 0 {
            return None;
        }
        // The source boundary excludes the next pair's interior. Within that
        // gap, a Line comment also ends the comma-line packet before later heads.
        let first = comments.partition_point(|comment| comment.span.end <= comma);
        let end = comments.partition_point(|comment| comment.span.end <= source.end);
        let packet = &comments[first..end];
        let first = packet.first()?;
        let last = packet.last()?;
        let opening = Span::new(comma + 1, first.span.start);
        if self.line_breaks(opening.start, opening.end) != 0
            || !self.source(opening).chars().all(is_source_whitespace)
        {
            return None;
        }
        // An outside EOL Line run already belongs to the preceding operand.
        // Its source pair boundary, rather than a normalized content start,
        // owns the following gap. Interior head comments are certified later.
        if suppression.is_none()
            && let Some(line) = packet
                .iter()
                .position(|comment| comment.kind == CommentKind::Line)
        {
            let inline = &packet[..=line];
            let last = inline[line];
            if self.line_breaks(first.span.start, last.span.end) == 0
                && inline
                    .windows(2)
                    .all(|pair| self.source_comment_gap_is_run(pair[0], pair[1]))
                && matches!(
                    self.prepare_expression_source_head(inline, next)
                        .lexical_binding,
                    CommentBinding::Unbound
                )
            {
                return Some(SequenceTrailingPacket {
                    comments: inline,
                    binding: SequencePacketBinding::InlineLine { source },
                });
            }
        }
        if !self.comment_run_ends_line(*last, source.end)
            || packet
                .windows(2)
                .any(|pair| !self.source_comment_gap_is_run(pair[0], pair[1]))
        {
            return None;
        }
        if self.diagnostic_range_is_sensitive(source) {
            return None;
        }
        let mut optimizer = false;
        let mut ignored = false;
        for &comment in packet {
            let semantics = self.comment_semantics(comment);
            if comment.kind != CommentKind::Block
                || semantics.diagnostic
                || semantics.file_directive
                || semantics.type_cast
            {
                return None;
            }
            optimizer |= semantics.optimizer;
            ignored |= semantics.suppression;
        }
        let binding = match (optimizer, ignored) {
            (true, false)
                if matches!(
                    self.expression_head_binding(packet, next),
                    CommentBinding::Unbound
                ) =>
            {
                SequencePacketBinding::InactiveOptimizer
            }
            (false, true) => {
                let source = suppression?;
                let target = source.preceding?;
                if target != previous.span(self.ast) || source.following != ContentMode::Formatted {
                    return None;
                }
                SequencePacketBinding::IgnoredPrevious(target)
            }
            _ => return None,
        };
        Some(SequenceTrailingPacket {
            comments: packet,
            binding,
        })
    }

    fn sequence_comment_partition(
        &self,
        boundary: &SequenceBoundary<'ast>,
        span: Span,
    ) -> Result<SequenceCommentPartition<'ast>, FormatError> {
        let comments = self.comments.in_range(span);
        let start = span.start;
        let end = span.end;
        let sensitive = self.diagnostic_range_is_sensitive(span);
        if comments.is_empty() && !sensitive {
            return Ok(SequenceCommentPartition {
                comments,
                trailing: &[],
                leading: &[],
                before_count: 0,
                packet_end: None,
                sensitive,
            });
        }
        let Some(comma) = boundary.comma else {
            return Err(FormatError::unsupported(
                "sequence separator boundary",
                None,
                span,
            ));
        };
        let before_count = comments.partition_point(|comment| comment.span.end <= comma);
        let (trailing, leading) = if sensitive {
            comments.split_at(before_count)
        } else {
            self.split_separator_comments(comments, before_count, start, Some(end))
        };
        let packet_end = boundary.packet.as_ref().and_then(|packet| {
            let first = packet.comments[0];
            let last = packet.comments[packet.comments.len() - 1];
            let first_index = before_count;
            let end_index = first_index + packet.comments.len();
            // A pending operand tail cannot be moved merely to reach
            // this later packet. Its existing source partition stays intact.
            let owner_retained = match packet.binding {
                SequencePacketBinding::InlineLine { source } => {
                    source.start == start
                        && last.span.end <= source.end
                        && end_index
                            == comments.partition_point(|comment| comment.span.end <= last.span.end)
                }
                _ => !sensitive,
            };
            (owner_retained
                && trailing.len() >= first_index
                && comments
                    .get(first_index)
                    .is_some_and(|comment| comment.span == first.span)
                && comments
                    .get(end_index - 1)
                    .is_some_and(|comment| comment.span == last.span))
            .then_some(end_index)
        });
        Ok(SequenceCommentPartition {
            comments,
            trailing,
            leading,
            before_count,
            packet_end,
            sensitive,
        })
    }

    fn prepare_sequence_comments(
        &self,
        boundary: &mut SequenceBoundary<'ast>,
    ) -> Result<SequenceComments, FormatError> {
        let mut span = Span::new(boundary.source_end, boundary.next_start);
        let mut partition = self.sequence_comment_partition(boundary, span)?;
        let head = match (boundary.packet.as_ref(), partition.packet_end) {
            (Some(packet), Some(end))
                if matches!(packet.binding, SequencePacketBinding::InlineLine { .. }) =>
            {
                let leading = &partition.comments[end..];
                // A distinct following head owns its opening scope. Otherwise
                // the deferred Line suffix supplies the exterior coverage.
                if leading.is_empty() {
                    &packet.comments[packet.comments.len() - 1..]
                } else {
                    leading
                }
            }
            _ => partition.leading,
        };
        // Resolve the complete separator packet before preparing this head.
        // The renderer reuses it instead of restoring the old source prefix.
        let content = self.expression_content_with_leading(boundary.next, head);
        let end = if self.sequence_member_pair_is_enclosed(
            boundary.next,
            &content,
            boundary.next_position,
        ) || boundary.suppression.as_ref().is_some_and(|source| {
            source.preceding.is_some() || source.following == ContentMode::Verbatim
        }) {
            boundary.next.span(self.ast).start
        } else {
            content.expression.span(self.ast).start
        };
        boundary.next_content = Some(content);
        boundary.next_start = end;
        if span.end != end {
            span.end = end;
            if boundary.comma.is_none() {
                let comments = self.comments.in_range(span);
                if !comments.is_empty() || self.diagnostic_range_is_sensitive(span) {
                    boundary.comma = self.find_token(span, comments, ",");
                }
            }
            partition = self.sequence_comment_partition(boundary, span)?;
        }
        let SequenceCommentPartition {
            comments,
            trailing,
            leading,
            before_count,
            packet_end,
            sensitive,
        } = partition;
        let start = span.start;
        let end = span.end;
        let preserves_ignored_target = packet_end.is_some()
            && boundary.packet.as_ref().is_some_and(|packet| {
                matches!(packet.binding, SequencePacketBinding::IgnoredPrevious(target)
                    if boundary.suppression.is_some_and(|source| source.preceding == Some(target)))
            });
        if boundary
            .suppression
            .is_some_and(|source| source.preceding.is_some())
            && !preserves_ignored_target
        {
            return Ok(SequenceComments::SourceGap);
        }
        (boundary.trailing, boundary.leading) =
            packet_end.map_or((trailing, leading), |end| comments.split_at(end));
        if sensitive {
            let comma = boundary.comma.ok_or_else(|| {
                FormatError::unsupported("sequence separator boundary", None, span)
            })?;
            let next = boundary
                .leading
                .first()
                .map_or(end, |comment| comment.span.start);
            if let Some((packet, source)) = boundary.packet.as_ref().and_then(|packet| match packet
                .binding
            {
                SequencePacketBinding::InlineLine { source } if packet_end.is_some() => {
                    Some((packet, source))
                }
                _ => None,
            }) {
                // The deferred Line suffix is written after the comma. Only
                // the actual pre-comma run can constrain punctuation here.
                boundary.before_comma =
                    self.line_constraint_after_comments(&comments[..before_count], start, comma);
                let last = packet.comments[packet.comments.len() - 1];
                boundary.separator = self.source_line_constraint(last.span.end, source.end);
            } else {
                boundary.before_comma =
                    self.line_constraint_after_comments(boundary.trailing, start, comma);
                boundary.separator = self.line_constraint(comma + 1, next);
            }
        }
        Ok(SequenceComments::Formatted)
    }

    fn format_sequence_trailing(
        &mut self,
        boundary: &mut SequenceBoundary<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if let Some(tail) = tail {
            boundary.source_end = tail.source_end;
        }
        if matches!(
            self.prepare_sequence_comments(boundary)?,
            SequenceComments::SourceGap
        ) {
            let gap = Span::new(boundary.source_end, boundary.next_start);
            boundary.separator = self
                .format_delimited_source_gap(gap, DelimitedFollowing::Node, writer)?
                .following;
            return Ok(());
        }
        self.comments.claim(boundary.trailing)?;
        self.format_comment_run(
            boundary.trailing,
            CommentRunLayout::SourceAfter(boundary.source_end),
            writer,
        )?;
        boundary
            .before_comma
            .write(writer, FlatSeparator::Empty, |_| {});
        writer.text(",");
        Ok(())
    }
}
