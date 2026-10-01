//! Constructor keyword ownership and member-expression callee grammar.
use super::AccessKind;

use swc_next_ecma_ast::{ExprData, GetSpan, NewExpression, Span};

use super::{ExpressionContext, ExpressionPosition, Formatter, MemberLayout, TailOwner};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_new(
        &mut self,
        constructor: NewExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let callee = constructor.callee(self.ast);
        let callee_span = callee.span(self.ast);
        let span = constructor.span(self.ast);
        if constructor.type_arguments(self.ast).is_some() {
            return Err(self.unsupported("typed constructor", constructor));
        }
        let arguments = if span.end == callee_span.end {
            None
        } else {
            Some(
                self.source_arguments(
                    constructor.arguments(self.ast),
                    callee,
                    span.end,
                    AccessKind::Ordinary,
                )
                .ok_or_else(|| self.unsupported("constructor opening boundary", constructor))?,
            )
        };
        let keyword_end = span.start + 3;
        let leading = self
            .comments
            .between(Span::new(keyword_end, callee_span.start))?;
        let leading_mode = self.content_mode(leading);
        let mode = if leading_mode == ContentMode::Verbatim {
            leading_mode
        } else if let Some(arguments) = &arguments {
            arguments.callee_mode
        } else {
            ContentMode::Formatted
        };
        writer.text("new");
        let next = leading
            .first()
            .map_or(callee_span.start, |comment| comment.span.start);
        self.line_constraint(keyword_end, next)
            .write(writer, FlatSeparator::Space, Writer::space);
        self.comments.claim(leading)?;
        let mut callee_context = ExpressionContext::at(ExpressionPosition::NewCallee)
            .with_tail_owner(TailOwner::ParentOperation)
            .with_content_mode(mode);
        callee_context.member_layout = MemberLayout::Tight;
        callee_context.conditional_envelope = context.conditional_envelope.terminal_head();
        let tail = self.format_expression_with_leading_comments(
            callee,
            NodeHead {
                comments: leading,
                content_start: callee_span.start,
                layout: CommentRunLayout::FlatLeading(callee_span.start),
                gap_end: None,
            },
            callee_context,
            writer,
        )?;
        if let Some(arguments) = arguments {
            self.format_source_arguments(arguments, tail, writer)
        } else {
            self.format_expression_tail(tail, writer)?;
            writer.text("()");
            Ok(None)
        }
    }

    /// The constructor callee must stay a MemberExpression, including its receiver spine.
    pub(super) fn new_callee_contains_call(&self, mut data: ExprData, mode: ContentMode) -> bool {
        loop {
            match data {
                ExprData::CallExpression(_) => return true,
                ExprData::MemberExpression(member) => {
                    let object = member.object(self.ast);
                    data = if mode == ContentMode::Formatted {
                        self.expression_content(object).data
                    } else {
                        self.ast.expr_data(object)
                    };
                }
                _ => return false,
            }
        }
    }
}
