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
        let mut types = constructor
            .type_arguments(self.ast)
            .map(|node| self.prepare_expression_type_arguments(callee, node, AccessKind::Ordinary))
            .transpose()?;
        let suffix_end = types
            .as_ref()
            .map_or(callee_span.end, |types| types.node.span(self.ast).end);
        let arguments = if span.end == suffix_end {
            None
        } else {
            let source = if let Some(types) = types.take() {
                self.source_arguments_after_types(
                    constructor.arguments(self.ast),
                    callee,
                    span.end,
                    types,
                )
            } else {
                self.source_arguments(
                    constructor.arguments(self.ast),
                    callee,
                    span.end,
                    AccessKind::Ordinary,
                )
            };
            Some(
                source
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
        } else if let Some(types) = &types {
            types.callee_mode
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
        let tail = self.format_expression_with_leading_comments(
            callee,
            NodeHead {
                comments: leading,
                content_start: callee_span.start,
                layout: CommentRunLayout::FlatLeading(callee_span.start),
                gap_end: None,
            },
            ExpressionContext::at(ExpressionPosition::NewCallee)
                .with_tail_owner(TailOwner::ParentOperation)
                .with_member_layout(MemberLayout::Tight)
                .with_content_mode(mode)
                .with_conditional_envelope(context.conditional_envelope.terminal_head()),
            None,
            writer,
        )?;
        if let Some(arguments) = arguments {
            self.format_source_arguments(arguments, tail, writer)
        } else {
            if let Some(types) = types {
                self.format_expression_type_arguments(types, tail, writer)?;
            } else {
                self.format_expression_tail(tail, writer)?;
            }
            writer.text("()");
            Ok(None)
        }
    }

    /// The constructor callee must stay a MemberExpression, including its receiver spine.
    pub(super) fn new_callee_contains_call(
        &self,
        mut data: ExprData,
        mut mode: ContentMode,
    ) -> bool {
        loop {
            match data {
                ExprData::CallExpression(_) | ExprData::ImportExpression(_) => return true,
                ExprData::TaggedTemplateExpression(template) => {
                    let Ok((tag, context)) = self.template_tag_head_operand(
                        template,
                        ExpressionContext::at(ExpressionPosition::Nested).with_content_mode(mode),
                    ) else {
                        return true;
                    };
                    mode = context.content_mode;
                    data = if mode == ContentMode::Formatted {
                        self.expression_content(tag).data
                    } else {
                        self.ast.expr_data(tag)
                    };
                }
                ExprData::TsNonNullExpression(assertion) => {
                    let expression = assertion.expression(self.ast);
                    data = if mode == ContentMode::Formatted {
                        self.expression_content(expression).data
                    } else {
                        self.ast.expr_data(expression)
                    };
                }
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
