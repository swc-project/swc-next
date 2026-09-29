//! Callees and source-owned argument delimiters.
use super::arguments::SourceArguments;
use super::{ExpressionContext, ExpressionPosition, Formatter, MemberLayout, TailOwner};
use crate::formatter::comments::ExpressionTail;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};
use swc_next_ecma_ast::{CallExpression, Expr, ExprData, GetSpan, Span};
pub(super) struct CallCallee {
    pub(super) expression: Expr,
    pub(super) mode: ContentMode,
}
impl CallCallee {
    pub(super) fn context(&self, context: ExpressionContext) -> ExpressionContext {
        let mut callee = ExpressionContext::at(ExpressionPosition::CallCallee)
            .with_parent_head(context)
            .with_tail_owner(TailOwner::ParentOperation)
            .with_content_mode(self.mode);
        callee.member_chain_layout = context.member_chain_layout;
        callee.conditional_envelope = context.conditional_envelope.chain_head();
        callee
    }
}
impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn call_span_is_source_clear(&self, span: Span) -> bool {
        if self.diagnostic_range_is_sensitive(span) {
            return false;
        }
        let comments = self.ast.comments();
        let first = comments.partition_point(|comment| comment.span.end <= span.start);
        comments
            .get(first)
            .is_none_or(|comment| comment.span.start >= span.end)
    }

    fn call_member_callee_is_compact(&self, mut expression: Expr) -> bool {
        if !self.call_span_is_source_clear(expression.span(self.ast)) {
            return false;
        }
        loop {
            expression = match self.expression_content(expression).data {
                ExprData::MemberExpression(member) => {
                    let Ok(head) = self.prepare_member_head(member) else {
                        return false;
                    };
                    if !matches!(self.source_clear_member_has_chain_barrier(&head), Ok(false)) {
                        return false;
                    }
                    head.object
                }
                data => {
                    return matches!(
                        data,
                        ExprData::IdentifierReference(_)
                            | ExprData::ThisExpression(_)
                            | ExprData::StringLiteral(_)
                            | ExprData::NumericLiteral(_)
                            | ExprData::BigIntLiteral(_)
                            | ExprData::BooleanLiteral(_)
                            | ExprData::NullLiteral(_)
                            | ExprData::RegExpLiteral(_)
                    );
                }
            };
        }
    }

    pub(super) fn source_call_callee(
        &self,
        call: CallExpression,
    ) -> Result<CallCallee, FormatError> {
        if self.has_suppression {
            return Ok(self.source_call(call)?.callee_head());
        }
        if call.optional(self.ast) || call.type_arguments(self.ast).is_some() {
            return Err(self.unsupported("typed or optional call", call));
        }
        Ok(CallCallee {
            expression: call.callee(self.ast),
            mode: ContentMode::Formatted,
        })
    }
    pub(super) fn format_call(
        &mut self,
        call: CallExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let arguments = self.source_call(call)?;
        let mut callee_context = arguments.callee_context(context);
        let member_callee = arguments.callee_mode == ContentMode::Formatted
            && matches!(
                self.expression_content(arguments.callee).data,
                ExprData::MemberExpression(_)
            )
            && self.expression_needs_parentheses(
                self.ast.expr_data(arguments.callee),
                callee_context.position,
                arguments.callee_mode,
            ) != Some(true)
            && !arguments.has_outside_prefix();
        if member_callee && self.call_member_callee_is_compact(arguments.callee) {
            callee_context.member_layout = MemberLayout::Tight;
            callee_context.member_chain_layout = super::MemberChainLayout::Local;
        }
        let write = |writer: &mut Writer<'ast>| {
            let tail = self.format_expression(arguments.callee, callee_context, writer)?;
            self.format_source_arguments(arguments, tail, writer)
        };
        if member_callee {
            // The complete member call owns receiver frames even when its
            // receiver is not eligible for the tight, atomic-callee path.
            writer.group(false, write)
        } else {
            write(writer)
        }
    }

    pub(super) fn source_call(
        &self,
        call: CallExpression,
    ) -> Result<SourceArguments<'ast>, FormatError> {
        if call.optional(self.ast) || call.type_arguments(self.ast).is_some() {
            return Err(self.unsupported("typed or optional call", call));
        }
        self.source_arguments(
            call.arguments(self.ast),
            call.callee(self.ast),
            call.span(self.ast).end,
        )
        .ok_or_else(|| self.unsupported("call opening boundary", call))
    }
}
