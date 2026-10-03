//! Callees and source-owned argument delimiters.
use super::AccessKind;
use super::arguments::{ArgumentGroup, SourceArguments};
use super::{ExpressionContext, ExpressionPosition, Formatter, MemberLayout, TailOwner};
use crate::formatter::comments::ExpressionTail;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};
use swc_next_ecma_ast::{Argument, CallExpression, Expr, ExprData, GetSpan, Span, SubRange};
pub(super) struct CallCallee {
    pub(super) expression: Expr,
    pub(super) mode: ContentMode,
    pub(super) access: AccessKind,
    pub(super) curried_call: Option<CallExpression>,
}
impl CallCallee {
    pub(super) fn context(&self, context: ExpressionContext) -> ExpressionContext {
        let mut callee = ExpressionContext::at(ExpressionPosition::CallCallee {
            access: self.access,
            curried_call: self.curried_call,
        })
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
                ExprData::ChainExpression(chain) => chain.expression(self.ast),
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
                    return super::chains::atomic_chain_head(data)
                        || matches!(data, ExprData::Super(_));
                }
            };
        }
    }

    pub(super) fn source_call_callee(
        &self,
        call: CallExpression,
        context: ExpressionContext,
    ) -> Result<CallCallee, FormatError> {
        if self.has_suppression {
            return Ok(self.source_call(call, context)?.callee_head());
        }
        if call.type_arguments(self.ast).is_some() {
            return Err(self.unsupported("typed call", call));
        }
        Ok(CallCallee {
            expression: call.callee(self.ast),
            curried_call: self.curried_call_callee(
                call.callee(self.ast),
                call.arguments(self.ast),
                ContentMode::Formatted,
            ),
            mode: ContentMode::Formatted,
            access: if call.optional(self.ast) {
                AccessKind::Optional
            } else {
                AccessKind::Ordinary
            },
        })
    }
    /// The RHS spine owner has verified this call's suffix window.
    /// Suppression still uses the complete source argument owner.
    pub(super) fn rhs_call_callee(
        &self,
        call: CallExpression,
        context: ExpressionContext,
    ) -> Option<CallCallee> {
        let ast = self.ast;
        if self.has_suppression || call.type_arguments(ast).is_some() {
            let source = self.source_call(call, context).ok()?;
            if source.callee_mode != ContentMode::Formatted || source.has_outside_prefix() {
                return None;
            }
            return Some(source.callee_head());
        }
        let callee = self.source_call_callee(call, context).ok()?;
        self.source_clear_argument_opening(
            call.arguments(ast),
            callee.expression,
            call.span(ast).end,
            callee.access,
        )
        .then_some(callee)
    }

    pub(super) fn format_call(
        &mut self,
        call: CallExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let arguments = self.source_call(call, context)?;
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
        let group = arguments.curried_callee.map(|_| arguments.group);
        let write = |writer: &mut Writer<'ast>| {
            let tail = self.format_expression(arguments.callee, callee_context, writer)?;
            self.format_source_arguments(arguments, tail, writer)
        };
        if member_callee {
            // The complete member call owns receiver frames even when its
            // receiver is not eligible for the tight, atomic-callee path.
            writer.group(false, write)
        } else {
            match group {
                Some(group) => group.write(false, writer, write),
                None => write(writer),
            }
        }
    }

    pub(super) fn source_call(
        &self,
        call: CallExpression,
        context: ExpressionContext,
    ) -> Result<SourceArguments<'ast>, FormatError> {
        if call.type_arguments(self.ast).is_some() {
            return Err(self.unsupported("typed call", call));
        }
        let mut arguments = self
            .source_arguments(
                call.arguments(self.ast),
                call.callee(self.ast),
                call.span(self.ast).end,
                if call.optional(self.ast) {
                    AccessKind::Optional
                } else {
                    AccessKind::Ordinary
                },
            )
            .ok_or_else(|| self.unsupported("call opening boundary", call))?;
        arguments.group = if matches!(context.position,
            ExpressionPosition::CallCallee { curried_call: Some(curried), .. } if curried == call)
        {
            ArgumentGroup::ParentCall
        } else {
            ArgumentGroup::Local
        };
        arguments.curried_callee = self.curried_call_callee(
            arguments.callee,
            call.arguments(self.ast),
            arguments.callee_mode,
        );
        Ok(arguments)
    }
    fn curried_call_callee(
        &self,
        callee: Expr,
        arguments: SubRange<Argument>,
        mode: ContentMode,
    ) -> Option<CallExpression> {
        if arguments.is_empty() || mode != ContentMode::Formatted {
            return None;
        }
        match self.expression_content(callee).data {
            ExprData::CallExpression(child)
                if child.arguments(self.ast).len() > arguments.len() =>
            {
                Some(child)
            }
            _ => None,
        }
    }
}
