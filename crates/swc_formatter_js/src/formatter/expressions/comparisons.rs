//! TypeScript relational syntax across runtime comma lists.

use swc_next_ecma_ast::{
    Argument, ArgumentData, ArrowFunctionBodyData, Ast, BinaryExpression, BinaryOperator,
    CommentKind, Expr, ExprData, GetSpan, Span, UnaryOperator,
};

use super::binary::{BinaryLike, BinaryOperatorKind, OperandSide};
use super::{ExpressionContext, ExpressionPosition, Formatter};
use crate::formatter::comments::CommentBinding;
use crate::formatter::suppression::{ContentMode, DelimitedFollowing};

#[derive(Default)]
pub(in crate::formatter) struct ComparisonList {
    operators: Vec<(u32, ComparisonOperatorGap)>,
    parentheses: Vec<u32>,
}

#[derive(Clone, Copy)]
pub(super) enum ComparisonOperatorGap {
    BreakBefore,
    RetainCommentBoundary,
}

pub(super) fn comparison_argument(ast: &Ast<'_>, argument: Argument) -> Option<Expr> {
    match ast.argument_data(argument) {
        ArgumentData::Expr(expression) => Some(expression),
        ArgumentData::SpreadElement(_) => None,
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn with_comparison_list<T>(
        &mut self,
        items: impl Iterator<Item = Option<Expr>>,
        end: u32,
        format: impl FnOnce(&mut Self) -> T,
    ) -> T {
        if !self.ast.is_ts() {
            return format(self);
        }
        let list = self.prepare_comparison_list(items, end);
        if list.operators.is_empty() {
            return format(self);
        }
        self.comparison_lists.push(list);
        let result = format(self);
        self.comparison_lists.pop();
        result
    }

    // A closing angle can complete speculative type arguments across commas.
    // Keep the RHS on the operator's line whenever this is only a layout choice.
    fn prepare_comparison_list(
        &self,
        items: impl Iterator<Item = Option<Expr>>,
        end: u32,
    ) -> ComparisonList {
        let ast = self.ast;
        let mut list = ComparisonList::default();
        let mut opening: Option<BinaryExpression> = None;
        let mut angles = 0;
        let mut previous = None;
        let mut items = items.peekable();
        while let Some(item) = items.next() {
            let Some(item) = item else {
                opening = None;
                angles = 0;
                continue;
            };
            if let (Some(less), Some(previous)) = (opening, previous)
                && let Some(greater) = self.list_comparison(item, OperandSide::Left)
                && greater.operator(ast).as_str().len() <= angles
            {
                let following = items.peek().copied().flatten().map(|node| node.span(ast));
                let needs_fence = self.comparison_requires_fence(greater)
                    || self.comparison_gap_is_ignored(
                        greater,
                        previous,
                        item.span(ast),
                        following,
                        end,
                    );
                if needs_fence {
                    list.parentheses.push(less.right(ast).span(ast).start);
                    list.operators.push((
                        greater.right(ast).span(ast).start,
                        ComparisonOperatorGap::RetainCommentBoundary,
                    ));
                } else if greater.operator(ast) == BinaryOperator::GreaterThan {
                    list.operators.push((
                        greater.right(ast).span(ast).start,
                        ComparisonOperatorGap::BreakBefore,
                    ));
                }
                opening = None;
                angles = 0;
            }
            if items.peek().is_none() {
                break;
            }
            previous = Some(item.span(ast));
            if let Some(less) = self.list_comparison(item, OperandSide::Right) {
                angles += self.comparison_open_angles(less);
                opening = Some(less);
            } else if opening.is_some() && !self.comparison_type_spelling(item) {
                opening = None;
                angles = 0;
            }
        }
        list.operators.sort_unstable_by_key(|entry| entry.0);
        list.parentheses.sort_unstable();
        list
    }

    fn list_comparison(&self, mut expression: Expr, side: OperandSide) -> Option<BinaryExpression> {
        let ast = self.ast;
        let closing = side == OperandSide::Left;
        let mut position = ExpressionPosition::Nested;
        loop {
            let data = self.expression_data_for_position(expression, position);
            if position != ExpressionPosition::Nested
                && self.expression_needs_parentheses(data, position, ContentMode::Formatted)
                    == Some(true)
            {
                return None;
            }
            if let ExprData::BinaryExpression(node) = data
                && match side {
                    OperandSide::Left => comparison_closing_operator(node.operator(ast)),
                    OperandSide::Right => node.operator(ast) == BinaryOperator::LessThan,
                }
                && self.comparison_type_spelling(if closing {
                    node.left(ast)
                } else {
                    node.right(ast)
                })
            {
                return Some(node);
            }
            (expression, position) = match data {
                ExprData::ArrowFunctionExpression(node) => {
                    let ArrowFunctionBodyData::Expr(body) =
                        ast.arrow_function_body_data(node.body(ast))
                    else {
                        return None;
                    };
                    (body, ExpressionPosition::ArrowBody)
                }
                ExprData::AssignmentExpression(node) if !closing => {
                    (node.right(ast), ExpressionPosition::Nested)
                }
                ExprData::ConditionalExpression(node) => {
                    if closing {
                        (node.test(ast), ExpressionPosition::ConditionalTest)
                    } else {
                        (
                            node.alternate(ast),
                            ExpressionPosition::ConditionalAlternate,
                        )
                    }
                }
                _ => {
                    let binary = BinaryLike::from_data(data, ast)?;
                    (
                        if closing { binary.left } else { binary.right },
                        ExpressionPosition::BinaryOperand {
                            operator: binary.operator,
                            side,
                        },
                    )
                }
            };
        }
    }

    fn comparison_open_angles(&self, mut expression: BinaryExpression) -> usize {
        let mut angles = 1;
        while let ExprData::BinaryExpression(left) = self.expression_data(expression.left(self.ast))
            && left.operator(self.ast) == BinaryOperator::LessThan
            && self.comparison_type_spelling(left.right(self.ast))
        {
            angles += 1;
            expression = left;
        }
        angles
    }

    // Recognize expression spellings that can also occupy a type argument.
    // Ordinary calls and arithmetic already rule out a speculative type list.
    fn comparison_type_spelling(&self, mut expression: Expr) -> bool {
        let ast = self.ast;
        loop {
            match self.expression_data(expression) {
                ExprData::IdentifierReference(_)
                | ExprData::ThisExpression(_)
                | ExprData::NumericLiteral(_)
                | ExprData::BigIntLiteral(_)
                | ExprData::BooleanLiteral(_)
                | ExprData::StringLiteral(_)
                | ExprData::NullLiteral(_)
                | ExprData::TemplateLiteral(_)
                | ExprData::ArrayExpression(_)
                | ExprData::ObjectExpression(_)
                | ExprData::TsInstantiationExpression(_) => return true,
                ExprData::BinaryExpression(binary) => {
                    return matches!(
                        binary.operator(ast),
                        BinaryOperator::BitwiseAnd | BinaryOperator::BitwiseOr
                    );
                }
                ExprData::MemberExpression(member) if !member.optional(ast) => {
                    expression = member.object(ast);
                }
                ExprData::ArrowFunctionExpression(arrow) => {
                    let ArrowFunctionBodyData::Expr(body) =
                        ast.arrow_function_body_data(arrow.body(ast))
                    else {
                        return false;
                    };
                    expression = body;
                }
                ExprData::UnaryExpression(unary)
                    if unary.operator(ast) == UnaryOperator::Negate =>
                {
                    return matches!(
                        self.expression_data(unary.argument(ast)),
                        ExprData::NumericLiteral(_) | ExprData::BigIntLiteral(_)
                    );
                }
                _ => return false,
            }
        }
    }

    fn comparison_requires_fence(&self, greater: BinaryExpression) -> bool {
        let ast = self.ast;
        let left = greater.left(ast).span(ast);
        let right = greater.right(ast);
        if self.comparison_rhs_can_follow_type_arguments(greater.operator(ast), right) {
            return true;
        }
        let right_span = right.span(ast);
        let comments = self
            .comments
            .in_range(Span::new(left.start, right_span.start));
        if comments.is_empty() {
            return false;
        }
        // Only inline, unbound block comments can move with the optional break.
        // The existing binary printer owns protected and line-dependent gaps.
        let gap = Span::new(left.end, right_span.start);
        let token = greater.operator(ast).as_str();
        let Some(operator) = self.find_token(gap, comments, token) else {
            return true;
        };
        comments[0].span.start <= operator
            || comments
                .iter()
                .any(|comment| comment.kind != CommentKind::Block)
            || self.source_line_breaks(operator + token.len() as u32, gap.end) != 0
            || !matches!(
                self.comment_binding(comments, gap, Some(right_span)),
                CommentBinding::Unbound
            )
    }

    fn comparison_gap_is_ignored(
        &self,
        greater: BinaryExpression,
        previous: Span,
        item: Span,
        following: Option<Span>,
        list_end: u32,
    ) -> bool {
        let ast = self.ast;
        if !self.has_suppression {
            return false;
        }
        let gap = Span::new(
            greater.left(ast).span(ast).end,
            greater.right(ast).span(ast).start,
        );
        let comments = self.comments.in_range(gap);
        let token = greater.operator(ast).as_str();
        let Some(operator) = self.find_token(gap, comments, token) else {
            return false;
        };
        if self.source_line_breaks(operator + token.len() as u32, gap.end) == 0 {
            return false;
        }
        let (next, following) = following.map_or((list_end, DelimitedFollowing::End), |span| {
            (span.start, DelimitedFollowing::Node)
        });
        self.delimited_suppression(previous, item.start, DelimitedFollowing::Node)
            .is_some_and(|suppression| suppression.following == ContentMode::Verbatim)
            || self
                .delimited_suppression(item, next, following)
                .is_some_and(|suppression| suppression.preceding == Some(item))
    }

    // Even inline, these heads can follow a completed expression type list.
    fn comparison_rhs_can_follow_type_arguments(
        &self,
        operator: BinaryOperator,
        mut expression: Expr,
    ) -> bool {
        let mut context = ExpressionContext::at(ExpressionPosition::BinaryOperand {
            operator: BinaryOperatorKind::Binary(operator),
            side: OperandSide::Right,
        });
        loop {
            let data = self.expression_data_for_position(expression, context.position);
            if self.expression_is_parenthesized(data, context) {
                return true;
            }
            match data {
                ExprData::TemplateLiteral(_)
                | ExprData::RegExpLiteral(_)
                | ExprData::ParenthesizedExpression(_) => return true,
                ExprData::IdentifierReference(node) => {
                    return matches!(self.ast.get_utf8(node.name(self.ast)), "as" | "satisfies");
                }
                _ => match self.expression_head_operand(data, context) {
                    Ok(Some((child, next))) => {
                        expression = child;
                        context = next;
                    }
                    Ok(None) => return false,
                    Err(_) => return true,
                },
            }
        }
    }

    pub(super) fn comparison_list_parentheses(&self, data: ExprData) -> bool {
        if self.comparison_lists.is_empty() {
            return false;
        }
        let ExprData::BinaryExpression(node) = data else {
            return false;
        };
        if node.operator(self.ast) != BinaryOperator::LessThan {
            return false;
        }
        let start = node.right(self.ast).span(self.ast).start;
        self.comparison_lists
            .iter()
            .rev()
            .any(|list| list.parentheses.binary_search(&start).is_ok())
    }

    pub(super) fn comparison_list_operator_gap(
        &self,
        binary: BinaryLike,
    ) -> Option<ComparisonOperatorGap> {
        if self.comparison_lists.is_empty()
            || !matches!(binary.operator, BinaryOperatorKind::Binary(operator)
                if comparison_closing_operator(operator))
        {
            return None;
        }
        let start = binary.right.span(self.ast).start;
        self.comparison_lists.iter().rev().find_map(|list| {
            let index = list
                .operators
                .binary_search_by_key(&start, |entry| entry.0)
                .ok()?;
            Some(list.operators[index].1)
        })
    }
}

fn comparison_closing_operator(operator: BinaryOperator) -> bool {
    matches!(
        operator,
        BinaryOperator::GreaterThan
            | BinaryOperator::RightShift
            | BinaryOperator::UnsignedRightShift
    )
}
