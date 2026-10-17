//! Restricted operand boundaries prepared before their keyword is emitted.

use swc_next_ecma_ast::{
    AssignmentTargetData, Comment, CommentKind, Expr, ExprData, GetSpan, Span,
};

use super::Formatter;
use super::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::chains::PreparedChain;
use super::expressions::parentheses::{
    ExpressionContent, ResolvedExpressionHead, SourceParentheses,
};
use super::expressions::{
    ExpressionContext, ExpressionPosition, OperandIndent, SourceTailOwner, TailOwner,
};
use super::suppression::ContentMode;
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
enum ArgumentEnvelope {
    Inline,
    Source,
    Breakable,
    Required,
}

#[derive(Clone, Copy)]
enum RestrictedGrammar {
    ReturnOrThrow,
    Yield,
}

impl RestrictedGrammar {
    fn argument_context(self, envelope: ArgumentEnvelope, mode: ContentMode) -> ExpressionContext {
        let position = match self {
            Self::ReturnOrThrow => ExpressionPosition::RestrictedArgument,
            Self::Yield => ExpressionPosition::YieldArgument,
        };
        let indent = match envelope {
            ArgumentEnvelope::Inline => OperandIndent::Expression,
            ArgumentEnvelope::Source | ArgumentEnvelope::Breakable | ArgumentEnvelope::Required => {
                OperandIndent::Parent
            }
        };
        let context = ExpressionContext::at(position)
            .with_source_tail_owner(SourceTailOwner::Enclosure)
            .with_operand_indent(indent)
            .with_content_mode(mode)
            .with_tail_owner(TailOwner::ParentOperation);
        if matches!(envelope, ArgumentEnvelope::Required) {
            context.inside_parentheses()
        } else {
            context
        }
    }
}

/// The first lexical owner and its removable source-prefix comment run.
struct RestrictedHead<'ast> {
    expression: Expr,
    comments: &'ast [Comment],
}

#[derive(Clone, Copy)]
struct RestrictedHeadFacts {
    expression: Expr,
    requires_envelope: bool,
}

/// Source comments shrink with the head walk; every packet is inspected once.
struct RestrictedHeadComments<'ast> {
    remaining: &'ast [Comment],
}

impl<'ast> RestrictedHeadComments<'ast> {
    fn new(comments: &'ast [Comment], span: Span) -> Self {
        let first = comments.partition_point(|comment| comment.span.start < span.start);
        let comments = &comments[first..];
        let count = comments.partition_point(|comment| comment.span.end <= span.end);
        Self {
            remaining: &comments[..count],
        }
    }

    fn before(&mut self, end: u32) {
        while let Some((last, rest)) = self.remaining.split_last()
            && last.span.end > end
        {
            self.remaining = rest;
        }
    }

    fn leading(&mut self, end: u32) -> &'ast [Comment] {
        let count = self
            .remaining
            .iter()
            .take_while(|comment| comment.span.end <= end)
            .count();
        let (leading, rest) = self.remaining.split_at(count);
        self.remaining = rest;
        leading
    }

    fn lookup(&mut self, span: Span) -> &'ast [Comment] {
        self.before(span.end);
        let count = self
            .remaining
            .iter()
            .rev()
            .take_while(|comment| comment.span.start >= span.start)
            .count();
        let (rest, lookup) = self.remaining.split_at(self.remaining.len() - count);
        self.remaining = rest;
        lookup
    }
}

struct RestrictedArgument<'ast> {
    argument: Expr,
    head: RestrictedHead<'ast>,
    keyword_end: u32,
    mode: ContentMode,
    envelope: ArgumentEnvelope,
    closing: LineConstraint,
    grammar: RestrictedGrammar,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_restricted_argument(
        &mut self,
        keyword_end: u32,
        argument: Expr,
        write_keyword: impl FnOnce(&mut Writer<'ast>),
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_restricted_operand(
            keyword_end,
            argument,
            RestrictedGrammar::ReturnOrThrow,
            write_keyword,
            writer,
        )
    }
    pub(super) fn format_yield_argument(
        &mut self,
        keyword_end: u32,
        argument: Expr,
        write_keyword: impl FnOnce(&mut Writer<'ast>),
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_restricted_operand(
            keyword_end,
            argument,
            RestrictedGrammar::Yield,
            write_keyword,
            writer,
        )
    }
    fn format_restricted_operand(
        &mut self,
        keyword_end: u32,
        argument: Expr,
        grammar: RestrictedGrammar,
        write_keyword: impl FnOnce(&mut Writer<'ast>),
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = argument.span(self.ast);
        let leading = self.comments.between(Span::new(keyword_end, span.start))?;
        let mode = self.content_mode(leading);
        let protected_opening = self.diagnostic_line_is_sensitive(span.start);
        let content = self.expression_content(argument);
        let mut head = RestrictedHead {
            expression: argument,
            comments: leading,
        };
        let envelope = if protected_opening || mode == ContentMode::Verbatim {
            if matches!(content.data, ExprData::ParenthesizedExpression(_)) {
                ArgumentEnvelope::Source
            } else {
                ArgumentEnvelope::Inline
            }
        } else {
            let lexical_head = self.restricted_lexical_head(argument, grammar)?;
            let leading_requires_envelope =
                self.restricted_head_requires_envelope(leading, span.start);
            if lexical_head.requires_envelope {
                head = RestrictedHead {
                    expression: lexical_head.expression,
                    comments: self.comments.between(Span::new(
                        keyword_end,
                        lexical_head.expression.span(self.ast).start,
                    ))?,
                };
            }
            if leading_requires_envelope || lexical_head.requires_envelope {
                ArgumentEnvelope::Required
            } else {
                match content.data {
                    ExprData::BinaryExpression(_) | ExprData::LogicalExpression(_) => {
                        ArgumentEnvelope::Breakable
                    }
                    ExprData::ParenthesizedExpression(_) => ArgumentEnvelope::Source,
                    _ => ArgumentEnvelope::Inline,
                }
            }
        };
        let prepared = RestrictedArgument {
            argument,
            head,
            keyword_end,
            mode,
            envelope,
            closing: self.line_constraint(span.end, span.end),
            grammar,
        };
        if !protected_opening
            && mode == ContentMode::Formatted
            && matches!(
                content.data,
                ExprData::CallExpression(_) | ExprData::MemberExpression(_)
            )
        {
            self.format_restricted_chain(prepared, &content, write_keyword, writer)
        } else {
            write_keyword(writer);
            self.format_restricted_expression(prepared, writer)
        }
    }

    /// A chain plan belongs only to this branch, including plans whose head
    /// keeps its existing owner. Ordinary operands never carry that payload.
    fn format_restricted_chain(
        &mut self,
        mut prepared: RestrictedArgument<'ast>,
        content: &ExpressionContent,
        write_keyword: impl FnOnce(&mut Writer<'ast>),
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let start = prepared.head.expression.span(self.ast).start;
        let chain: Option<PreparedChain<'ast>> = self.prepare_restricted_fluent_chain(
            prepared.argument,
            content,
            prepared
                .grammar
                .argument_context(prepared.envelope, prepared.mode),
            &ResolvedExpressionHead {
                head: NodeHead {
                    comments: prepared.head.comments,
                    content_start: start,
                    layout: CommentRunLayout::FlatLeading(start),
                    gap_end: None,
                },
                binding: self.expression_head_binding(prepared.head.comments, prepared.argument),
            },
        )?;
        let Some(mut chain) = chain else {
            write_keyword(writer);
            return self.format_restricted_expression(prepared, writer);
        };
        if chain.has_call_operation_head() {
            // This is an optional readability frame for the actual call
            // operation head, not a restricted-grammar requirement.
            prepared.envelope = ArgumentEnvelope::Breakable;
            chain.set_enclosing_context(
                prepared
                    .grammar
                    .argument_context(prepared.envelope, prepared.mode),
            );
        }
        chain.set_leading_comment_layout(
            if matches!(prepared.envelope, ArgumentEnvelope::Required) {
                CommentRunLayout::GroupedLeading(start)
            } else {
                CommentRunLayout::FlatLeading(start)
            },
        );
        write_keyword(writer);
        self.format_restricted_envelope(
            &prepared,
            |formatter, _, writer| formatter.format_prepared_chain(chain, writer),
            writer,
        )
    }

    fn format_restricted_expression(
        &mut self,
        prepared: RestrictedArgument<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_restricted_envelope(
            &prepared,
            |formatter, content_start, writer| {
                formatter.comments.claim(prepared.head.comments)?;
                let context = prepared
                    .grammar
                    .argument_context(prepared.envelope, prepared.mode);
                formatter.format_expression_with_leading_comments(
                    prepared.argument,
                    NodeHead {
                        comments: prepared.head.comments,
                        content_start,
                        layout: if matches!(prepared.envelope, ArgumentEnvelope::Required) {
                            CommentRunLayout::GroupedLeading(content_start)
                        } else {
                            CommentRunLayout::FlatLeading(content_start)
                        },
                        gap_end: None,
                    },
                    context,
                    None,
                    writer,
                )
            },
            writer,
        )
    }

    fn format_restricted_envelope(
        &mut self,
        prepared: &RestrictedArgument<'ast>,
        write_argument: impl FnOnce(
            &mut Self,
            u32,
            &mut Writer<'ast>,
        ) -> Result<Option<ExpressionTail<'ast>>, FormatError>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let RestrictedArgument {
            argument,
            head,
            keyword_end,
            envelope,
            closing,
            ..
        } = prepared;
        let content_start = head.expression.span(self.ast).start;
        let first = match envelope {
            ArgumentEnvelope::Inline | ArgumentEnvelope::Source => head
                .comments
                .first()
                .map_or(content_start, |comment| comment.span.start),
            ArgumentEnvelope::Breakable | ArgumentEnvelope::Required => {
                argument.span(self.ast).start
            }
        };
        self.line_constraint(*keyword_end, first).write(
            writer,
            FlatSeparator::Space,
            Writer::space,
        );
        match envelope {
            ArgumentEnvelope::Inline | ArgumentEnvelope::Source => {
                write_argument(self, content_start, writer)
            }
            ArgumentEnvelope::Breakable | ArgumentEnvelope::Required => {
                let write = |writer: &mut Writer<'ast>| {
                    if matches!(envelope, ArgumentEnvelope::Required) {
                        writer.text("(");
                    } else {
                        writer.if_break("(");
                    }
                    let tail = writer.indent(|writer| {
                        if matches!(envelope, ArgumentEnvelope::Required) {
                            writer.hard_line();
                        } else {
                            writer.soft_line();
                        }
                        let tail = write_argument(self, content_start, writer)?;
                        if matches!(envelope, ArgumentEnvelope::Required) {
                            self.format_expression_tail(tail, writer)?;
                            Ok(None)
                        } else {
                            Ok(tail)
                        }
                    })?;
                    closing.write(writer, FlatSeparator::Empty, |writer| {
                        if matches!(envelope, ArgumentEnvelope::Required) {
                            writer.hard_line();
                        } else {
                            writer.soft_line();
                        }
                    });
                    if matches!(envelope, ArgumentEnvelope::Required) {
                        writer.text(")");
                    } else {
                        writer.if_break(")");
                    }
                    Ok(tail)
                };
                if matches!(envelope, ArgumentEnvelope::Breakable) {
                    writer.group_with_flat_width(write)
                } else {
                    writer.group(true, write)
                }
            }
        }
    }

    fn restricted_lexical_head(
        &self,
        mut expression: Expr,
        grammar: RestrictedGrammar,
    ) -> Result<RestrictedHeadFacts, FormatError> {
        let mut comments =
            RestrictedHeadComments::new(self.ast.comments(), expression.span(self.ast));
        if comments.remaining.is_empty() {
            return Ok(RestrictedHeadFacts {
                expression,
                requires_envelope: false,
            });
        }
        let mut context = ExpressionContext::at(match grammar {
            RestrictedGrammar::ReturnOrThrow => ExpressionPosition::RestrictedArgument,
            RestrictedGrammar::Yield => ExpressionPosition::YieldArgument,
        });
        let mut requires_envelope = false;
        loop {
            comments.before(expression.span(self.ast).end);
            let data = self.ast.expr_data(expression);
            if context.is_verbatim() || self.expression_is_parenthesized(data, context) {
                break;
            }
            expression = match data {
                ExprData::ParenthesizedExpression(parenthesized) => {
                    let child = parenthesized.expression(self.ast);
                    let boundary = SourceParentheses {
                        outer: parenthesized.span(self.ast),
                        inner: child.span(self.ast),
                        inner_target: super::expressions::parentheses::ParenthesisInner::Expression(
                            child,
                        ),
                    };
                    if !self.parentheses_are_removable(boundary) {
                        break;
                    }
                    let leading = comments.leading(boundary.inner.start);
                    requires_envelope |=
                        self.restricted_head_requires_envelope(leading, boundary.inner.start);
                    child
                }
                ExprData::MemberExpression(member) => {
                    if matches!(grammar, RestrictedGrammar::ReturnOrThrow)
                        && !member.computed(self.ast)
                    {
                        let gap = Span::new(
                            member.object(self.ast).span(self.ast).end,
                            member.property(self.ast).span(self.ast).start,
                        );
                        let comments = comments.lookup(gap);
                        let token = if member.optional(self.ast) { "?." } else { "." };
                        let Some(operator) = self.find_token(gap, comments, token) else {
                            return Err(self.unsupported("member operator boundary", member));
                        };
                        let lookup = self.member_lookup_comments(comments, gap, operator, false);
                        requires_envelope |= lookup.iter().any(|comment| {
                            self.line_breaks(comment.span.start, comment.span.end) != 0
                                || self.source_position_starts_line(comment.span.start)
                        });
                    }
                    let Some((child, child_context)) =
                        self.expression_head_operand(data, context)?
                    else {
                        break;
                    };
                    context = child_context;
                    child
                }
                ExprData::SequenceExpression(sequence) => {
                    let Some(first) = sequence.expressions(self.ast).iter().next() else {
                        break;
                    };
                    self.ast.get_node_in_sub_range(first)
                }
                ExprData::AssignmentExpression(assignment) => {
                    let AssignmentTargetData::SimpleAssignmentTarget(target) =
                        self.ast.assignment_target_data(assignment.left(self.ast))
                    else {
                        break;
                    };
                    context = ExpressionContext::at(ExpressionPosition::AssignmentTarget);
                    super::expressions::targets::SimpleTarget::new(self.ast, target).expression()
                }
                ExprData::UpdateExpression(update) if !update.prefix(self.ast) => {
                    context = ExpressionContext::at(ExpressionPosition::UpdateArgument);
                    super::expressions::targets::SimpleTarget::new(
                        self.ast,
                        update.argument(self.ast),
                    )
                    .expression()
                }
                _ => {
                    let Some((child, child_context)) =
                        self.expression_head_operand(data, context)?
                    else {
                        break;
                    };
                    context = child_context;
                    child
                }
            };
        }
        Ok(RestrictedHeadFacts {
            expression,
            requires_envelope,
        })
    }

    /// A removable pair's opening whitespace does not constrain the exposed head.
    fn restricted_head_requires_envelope(&self, comments: &[Comment], end: u32) -> bool {
        comments.iter().enumerate().any(|(index, comment)| {
            let next = comments.get(index + 1).map_or(end, |next| next.span.start);
            comment.kind == CommentKind::Line
                || self.line_breaks(comment.span.start, comment.span.end) != 0
                || self.line_breaks(comment.span.end, next) != 0
        })
    }
}
