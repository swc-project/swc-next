//! Actual await/yield operands and lexical meta-property expressions.

use swc_next_ecma_ast::{
    AwaitExpression, Expr, ExprData, GetSpan, MetaProperty, Span, YieldExpression,
};

use super::{ExpressionContext, ExpressionPosition, Formatter, ParenthesesOwner, TailOwner};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_await(
        &mut self,
        expression: AwaitExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let receiver = context.is_call_or_member_receiver()
            && context.parentheses_owner == ParenthesesOwner::Expression;
        let shares_group = receiver
            && self.enclosing_await.is_some_and(|outer| {
                self.await_argument_starts_with(outer.argument(self.ast), expression)
            });
        let outer = self.enclosing_await.replace(expression);
        let mut write = |writer: &mut Writer<'ast>| {
            writer.text("await");
            self.format_unrestricted_keyword_argument(
                expression.span(self.ast).start + 5,
                expression.argument(self.ast),
                ExpressionPosition::AwaitArgument,
                writer,
            )
        };
        let result = if receiver {
            let mut write = |writer: &mut Writer<'ast>| {
                let tail = writer.indent(|writer| {
                    writer.soft_line();
                    write(writer)
                })?;
                writer.soft_line();
                Ok(tail)
            };
            if shares_group {
                write(writer)
            } else {
                writer.group(false, write)
            }
        } else {
            write(writer)
        };
        self.enclosing_await = outer;
        result
    }

    /// Only the nearest await's actual leftmost expression can share its group.
    /// Prefix tokens and function bodies end the walk before unrelated awaits.
    fn await_argument_starts_with(&self, mut argument: Expr, awaited: AwaitExpression) -> bool {
        loop {
            if argument == Expr::AwaitExpression(awaited) {
                return true;
            }
            let data = self.ast.expr_data(argument);
            argument = match data {
                ExprData::ParenthesizedExpression(node) => node.expression(self.ast),
                ExprData::ChainExpression(node) => node.expression(self.ast),
                ExprData::MemberExpression(node) => node.object(self.ast),
                ExprData::CallExpression(node) => node.callee(self.ast),
                ExprData::ConditionalExpression(node) => node.test(self.ast),
                ExprData::AssignmentExpression(node) => {
                    match self.ast.assignment_target_data(node.left(self.ast)) {
                        swc_next_ecma_ast::AssignmentTargetData::SimpleAssignmentTarget(target) => {
                            match self.simple_target_expression(target) {
                                Ok(expression) => expression,
                                Err(_) => return false,
                            }
                        }
                        _ => return false,
                    }
                }
                ExprData::SequenceExpression(node) => {
                    let Some(first) = node.expressions(self.ast).iter().next() else {
                        return false;
                    };
                    self.ast.get_node_in_sub_range(first)
                }
                ExprData::UpdateExpression(node) if !node.prefix(self.ast) => {
                    match self.simple_target_expression(node.argument(self.ast)) {
                        Ok(expression) => expression,
                        Err(_) => return false,
                    }
                }
                _ => {
                    let Some(binary) = super::binary::BinaryLike::from_data(data, self.ast) else {
                        return false;
                    };
                    binary.left
                }
            };
        }
    }

    pub(super) fn format_yield(
        &mut self,
        expression: YieldExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let keyword_end = expression.span(self.ast).start + 5;
        let Some(argument) = expression.argument(self.ast) else {
            writer.text("yield");
            return Ok(None);
        };
        if expression.delegate(self.ast) {
            let gap = Span::new(keyword_end, argument.span(self.ast).start);
            let comments = self.comments.in_range(gap);
            let Some(star) = self.find_token(gap, comments, "*") else {
                return Err(self.unsupported("delegated yield star boundary", expression));
            };
            writer.text("yield");
            self.format_source_token_gap(keyword_end, star, false, writer)?;
            writer.text("*");
            self.format_unrestricted_keyword_argument(
                star + 1,
                argument,
                ExpressionPosition::YieldArgument,
                writer,
            )
        } else {
            self.format_yield_argument(keyword_end, argument, |writer| writer.text("yield"), writer)
        }
    }

    fn format_unrestricted_keyword_argument(
        &mut self,
        keyword_end: u32,
        argument: Expr,
        position: ExpressionPosition,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = argument.span(self.ast);
        let leading = self.comments.between(Span::new(keyword_end, span.start))?;
        let mode = self.content_mode(leading);
        let first = leading
            .first()
            .map_or(span.start, |comment| comment.span.start);
        self.line_constraint(keyword_end, first)
            .write(writer, FlatSeparator::Space, Writer::space);
        self.comments.claim(leading)?;
        self.format_expression_with_leading_comments(
            argument,
            NodeHead {
                comments: leading,
                content_start: span.start,
                layout: CommentRunLayout::FlatLeading(span.start),
                gap_end: None,
            },
            ExpressionContext::at(position)
                .with_content_mode(mode)
                .with_tail_owner(TailOwner::ParentOperation),
            None,
            writer,
        )
    }

    pub(super) fn format_meta_property(
        &mut self,
        expression: MetaProperty,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let meta = expression.meta(ast);
        let property = expression.property(ast);
        let gap = Span::new(meta.span(ast).end, property.span(ast).start);
        let comments = self.comments.in_range(gap);
        let Some(dot) = self.find_token(gap, comments, ".") else {
            return Err(self.unsupported("meta property dot boundary", expression));
        };
        let meta_name = ast.get_utf8(meta.name(ast));
        let keyword_packet = if meta_name == "import" && self.has_suppression {
            let before_dot = self.comments.in_range(Span::new(gap.start, dot));
            (self.content_mode(before_dot) == ContentMode::Verbatim)
                .then_some(Span::new(meta.span(ast).start, dot))
        } else {
            None
        };
        if let Some(packet) = keyword_packet {
            // An ignored import keyword includes its trivia before the dot.
            self.format_verbatim(packet, writer)?;
        } else {
            writer.text(meta_name);
            self.format_source_token_gap(gap.start, dot, false, writer)?;
        }
        writer.text(".");
        self.format_source_token_gap(dot + 1, gap.end, false, writer)?;
        writer.text(ast.get_utf8(property.name(ast)));
        Ok(())
    }
}
