//! Removal of computations whose values are unused.
//!
//! An expression statement, a `void` operand, a `for` initializer or update,
//! and every item of a sequence except the last are evaluated only for their
//! side effects. Their children are final when the traversal exits them, so
//! one bottom-up rewrite keeps only the parts that may be observable.

use swc_next_ecma_ast::{
    ArgumentData, Ast, BinaryOperator, Expr, ExprData, ForStatementInit, ForStatementInitData,
    GetSpan, LogicalOperator, ObjectPropertyKindData, PropertyKeyData, Span, Stmt, StmtData,
    UnaryExpression, UnaryOperator,
};

use super::{
    Compressor, dead_code, number, statements,
    value::{self, Value},
};
use crate::CompressOptions;

impl Compressor {
    /// Returns an expression with the side effects of `expr`, for a position
    /// whose value is unused, or `None` when nothing observable remains.
    /// Returns `expr` itself when nothing can be removed.
    pub(super) fn drop_value(&self, ast: &mut Ast<'_>, expr: Expr) -> Option<Expr> {
        let mut effects = Vec::new();
        self.collect_effects(ast, expr, &mut effects);
        if let ExprData::SequenceExpression(sequence) = ast.expr_data(expr)
            && effects.iter().copied().eq(sequence
                .expressions(ast)
                .iter()
                .map(|id| ast.get_node_in_sub_range(id)))
        {
            return Some(expr);
        }
        (!effects.is_empty()).then(|| statements::sequence(ast, &effects))
    }

    /// Removes the unused values of an expression statement or of a `for`
    /// initializer and update, and rewrites called function expressions.
    /// Returns the statement that replaces an expression statement with
    /// nothing to evaluate.
    pub(super) fn drop_statement_values(&self, ast: &mut Ast<'_>, node: Stmt) -> Option<Stmt> {
        match ast.stmt_data(node) {
            StmtData::ExpressionStatement(statement) => {
                let expression = statement.expression(ast);
                let kept = if self.options.side_effects {
                    self.drop_value(ast, expression)
                } else {
                    Some(expression)
                };
                let Some(kept) = kept else {
                    return Some(ast.stmt_empty_statement(node.span(ast)));
                };
                let kept = statement_iifes(ast, &self.options, kept);
                if kept != expression {
                    statement.set_expression(ast, kept);
                }
            }
            StmtData::ForStatement(statement) if self.options.side_effects => {
                if let Some(init) = statement.init(ast)
                    && let ForStatementInitData::Expr(init) = ast.for_statement_init_data(init)
                {
                    let kept = self.drop_value(ast, init);
                    if kept != Some(init) {
                        statement.set_init(ast, kept.map(ForStatementInit::Expr));
                    }
                }
                if let Some(update) = statement.update(ast) {
                    let kept = self.drop_value(ast, update);
                    if kept != Some(update) {
                        statement.set_update(ast, kept);
                    }
                }
            }
            _ => {}
        }
        None
    }

    /// Removes the side-effect-free parts of a `void` operand, and replaces
    /// an operand with nothing to evaluate with `0`, so the result prints as
    /// the canonical `void 0`.
    pub(super) fn drop_void_operand(
        &self,
        ast: &mut Ast<'_>,
        node: Expr,
        unary: UnaryExpression,
    ) -> Option<Expr> {
        let argument = unary.argument(ast);
        let kept = match self.drop_value(ast, argument) {
            Some(kept) if kept == argument => return None,
            Some(kept) => kept,
            None if matches!(ast.expr_data(argument), ExprData::NumericLiteral(literal)
                if literal.value(ast) == 0.0) =>
            {
                return None;
            }
            None => number(ast, argument.span(ast), 0.0),
        };
        let span = node.span(ast);
        Some(Expr::UnaryExpression(ast.unary_expression(
            span,
            kept,
            UnaryOperator::Void,
        )))
    }

    /// Removes the unused items of a sequence whose value is used. Returns
    /// `None` when every item is kept.
    ///
    /// A sequence in a reference position keeps a leading `0` before a kept
    /// reference, so `(0, a.b)()` still calls `a.b` without a receiver.
    pub(super) fn trim_sequence(&mut self, ast: &mut Ast<'_>, node: Expr) -> Option<Expr> {
        let ExprData::SequenceExpression(sequence) = ast.expr_data(node) else {
            return None;
        };
        let reference_position = self.reference_positions.remove(&node.node_id());
        let items: Vec<_> = sequence
            .expressions(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .collect();
        let (&last, unused) = items.split_last()?;
        let mut kept = Vec::new();
        for &item in unused {
            self.collect_effects(ast, item, &mut kept);
        }
        if kept == unused {
            return None;
        }
        if reference_position && kept.is_empty() && dead_code::is_reference(ast, last) {
            if let [first] = unused
                && matches!(ast.expr_data(*first), ExprData::NumericLiteral(_))
            {
                return None;
            }
            kept.push(number(ast, node.span(ast), 0.0));
        }
        kept.push(last);
        Some(statements::sequence(ast, &kept))
    }

    /// Appends the parts of `expr` that may be observable to `effects`, in
    /// evaluation order.
    fn collect_effects(&self, ast: &mut Ast<'_>, expr: Expr, effects: &mut Vec<Expr>) {
        match ast.expr_data(expr) {
            ExprData::StringLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::RegExpLiteral(_)
            | ExprData::MetaProperty(_)
            | ExprData::Function(_)
            | ExprData::ArrowFunctionExpression(_) => {}
            ExprData::IdentifierReference(_) => {
                if !self.candidates.pure_reads.contains(&expr.node_id()) {
                    effects.push(expr);
                }
            }
            ExprData::ParenthesizedExpression(parenthesized) => {
                self.collect_effects(ast, parenthesized.expression(ast), effects);
            }
            ExprData::SequenceExpression(sequence) => {
                for id in sequence.expressions(ast).iter() {
                    let item = ast.get_node_in_sub_range(id);
                    self.collect_effects(ast, item, effects);
                }
            }
            ExprData::UnaryExpression(unary) => {
                let argument = unary.argument(ast);
                match unary.operator(ast) {
                    // Statements choose this form; see `iife_forms`.
                    UnaryOperator::LogicalNot if is_iife(ast, argument) => effects.push(expr),
                    UnaryOperator::Void | UnaryOperator::LogicalNot => {
                        self.collect_effects(ast, argument, effects);
                    }
                    // A read that is not pure keeps its `typeof`: an
                    // undeclared name can be a global accessor, and the read
                    // alone would throw.
                    UnaryOperator::Typeof
                        if matches!(
                            ast.expr_data(value::skip_parentheses(ast, argument)),
                            ExprData::IdentifierReference(_)
                        ) =>
                    {
                        let id = value::skip_parentheses(ast, argument).node_id();
                        if !self.candidates.pure_reads.contains(&id) {
                            effects.push(expr);
                        }
                    }
                    UnaryOperator::Typeof => self.collect_effects(ast, argument, effects),
                    // Numeric conversion of a primitive other than a BigInt
                    // or a symbol cannot call user code or throw.
                    UnaryOperator::Negate | UnaryOperator::Positive | UnaryOperator::BitwiseNot
                        if is_primitive(ast, argument) =>
                    {
                        self.collect_effects(ast, argument, effects);
                    }
                    _ => effects.push(expr),
                }
            }
            ExprData::BinaryExpression(binary)
                if !matches!(
                    binary.operator(ast),
                    BinaryOperator::In | BinaryOperator::Instanceof
                ) && is_primitive(ast, binary.left(ast))
                    && is_primitive(ast, binary.right(ast)) =>
            {
                self.collect_effects(ast, binary.left(ast), effects);
                self.collect_effects(ast, binary.right(ast), effects);
            }
            ExprData::LogicalExpression(logical) => {
                let left = logical.left(ast);
                let right = logical.right(ast);
                match self.drop_value(ast, right) {
                    None => self.collect_effects(ast, left, effects),
                    Some(kept) if kept == right => effects.push(expr),
                    Some(kept) => {
                        let span = expr.span(ast);
                        let operator = logical.operator(ast);
                        effects.push(logical_expression(ast, span, left, kept, operator));
                    }
                }
            }
            ExprData::ConditionalExpression(conditional) => {
                let test = conditional.test(ast);
                let consequent = conditional.consequent(ast);
                let alternate = conditional.alternate(ast);
                let span = expr.span(ast);
                match (
                    self.drop_value(ast, consequent),
                    self.drop_value(ast, alternate),
                ) {
                    (None, None) => self.collect_effects(ast, test, effects),
                    (Some(kept), None) => {
                        effects.push(logical_expression(
                            ast,
                            span,
                            test,
                            kept,
                            LogicalOperator::And,
                        ));
                    }
                    (None, Some(kept)) => {
                        effects.push(logical_expression(
                            ast,
                            span,
                            test,
                            kept,
                            LogicalOperator::Or,
                        ));
                    }
                    (Some(left), Some(right)) if left == consequent && right == alternate => {
                        effects.push(expr);
                    }
                    (Some(left), Some(right)) => effects.push(Expr::ConditionalExpression(
                        ast.conditional_expression(span, test, left, right),
                    )),
                }
            }
            ExprData::ArrayExpression(array) => {
                let mut elements = Vec::new();
                for id in array.elements(ast).iter() {
                    let Some(element) = ast.get_node_in_sub_range(id) else {
                        continue;
                    };
                    match ast.argument_data(element) {
                        ArgumentData::Expr(element) => elements.push(element),
                        // Spread runs the iterator protocol.
                        ArgumentData::SpreadElement(_) => {
                            effects.push(expr);
                            return;
                        }
                    }
                }
                for element in elements {
                    self.collect_effects(ast, element, effects);
                }
            }
            ExprData::ObjectExpression(object) => {
                let mut values = Vec::new();
                for id in object.properties(ast).iter() {
                    let property =
                        match ast.object_property_kind_data(ast.get_node_in_sub_range(id)) {
                            ObjectPropertyKindData::ObjectProperty(property) => property,
                            // Spread runs getters.
                            ObjectPropertyKindData::SpreadElement(_) => {
                                effects.push(expr);
                                return;
                            }
                        };
                    // A computed key that is not constant may call `toString`.
                    if let PropertyKeyData::Expr(key) = ast.property_key_data(property.key(ast))
                        && Value::of(ast, key).is_none()
                    {
                        effects.push(expr);
                        return;
                    }
                    values.push(property.value(ast));
                }
                for value in values {
                    self.collect_effects(ast, value, effects);
                }
            }
            // `ToString` of a constant cannot call user code or throw.
            ExprData::TemplateLiteral(template)
                if template
                    .expressions(ast)
                    .iter()
                    .all(|id| Value::of(ast, ast.get_node_in_sub_range(id)).is_some()) => {}
            _ => effects.push(expr),
        }
    }
}

/// Rewrites the calls of function expressions in an expression statement;
/// see `iife_forms`.
fn statement_iifes(ast: &mut Ast<'_>, options: &CompressOptions, expr: Expr) -> Expr {
    let mut items = Vec::new();
    statements::push_expressions(ast, expr, &mut items);
    if !iife_forms(ast, options, &mut items, true) {
        return expr;
    }
    statements::sequence(ast, &items)
}

/// Gives each call of a function expression in `items`, a sequence whose
/// value is unused when `statement` is set, its shortest form. Returns
/// whether an item changed.
///
/// With `negate_iife`, a call that starts a statement is negated:
/// `!function(){}()` is shorter than `(function(){})()`. With
/// `side_effects`, a negated call whose value is unused elsewhere loses the
/// negation, which needs no parentheses there. `collect_effects` keeps the
/// negation, so a repeated pass does not change either form.
pub(super) fn iife_forms(
    ast: &mut Ast<'_>,
    options: &CompressOptions,
    items: &mut [Expr],
    statement: bool,
) -> bool {
    let mut changed = false;
    let count = items.len();
    for (index, item) in items.iter_mut().enumerate() {
        if statement && index == 0 {
            if options.negate_iife && is_iife(ast, *item) {
                let span = item.span(ast);
                *item = Expr::UnaryExpression(ast.unary_expression(
                    span,
                    *item,
                    UnaryOperator::LogicalNot,
                ));
                changed = true;
            }
        } else if options.side_effects
            && (statement || index + 1 < count)
            && let ExprData::UnaryExpression(unary) = ast.expr_data(*item)
            && unary.operator(ast) == UnaryOperator::LogicalNot
            && is_iife(ast, unary.argument(ast))
        {
            *item = unary.argument(ast);
            changed = true;
        }
    }
    changed
}

/// Whether `expr` calls a function expression.
fn is_iife(ast: &Ast<'_>, expr: Expr) -> bool {
    matches!(ast.expr_data(expr), ExprData::CallExpression(call)
    if matches!(
        ast.expr_data(value::skip_parentheses(ast, call.callee(ast))),
        ExprData::Function(_)
    ))
}

fn logical_expression(
    ast: &mut Ast<'_>,
    span: Span,
    left: Expr,
    right: Expr,
    operator: LogicalOperator,
) -> Expr {
    Expr::LogicalExpression(ast.logical_expression(span, left, right, operator))
}

/// Whether `expr` evaluates, if it completes, to a primitive other than a
/// BigInt or a symbol. Operators that convert such values cannot call user
/// code or throw.
fn is_primitive(ast: &Ast<'_>, expr: Expr) -> bool {
    match ast.expr_data(expr) {
        ExprData::StringLiteral(_)
        | ExprData::NumericLiteral(_)
        | ExprData::BooleanLiteral(_)
        | ExprData::NullLiteral(_)
        | ExprData::TemplateLiteral(_) => true,
        ExprData::ParenthesizedExpression(parenthesized) => {
            is_primitive(ast, parenthesized.expression(ast))
        }
        ExprData::SequenceExpression(sequence) => sequence
            .expressions(ast)
            .iter()
            .next_back()
            .is_some_and(|id| is_primitive(ast, ast.get_node_in_sub_range(id))),
        ExprData::UnaryExpression(unary) => match unary.operator(ast) {
            UnaryOperator::Void
            | UnaryOperator::LogicalNot
            | UnaryOperator::Typeof
            | UnaryOperator::Delete => true,
            UnaryOperator::Negate | UnaryOperator::Positive | UnaryOperator::BitwiseNot => {
                is_primitive(ast, unary.argument(ast))
            }
        },
        ExprData::BinaryExpression(binary) => match binary.operator(ast) {
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::StrictEqual
            | BinaryOperator::StrictNotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual
            | BinaryOperator::In
            | BinaryOperator::Instanceof => true,
            _ => is_primitive(ast, binary.left(ast)) && is_primitive(ast, binary.right(ast)),
        },
        ExprData::LogicalExpression(logical) => {
            is_primitive(ast, logical.left(ast)) && is_primitive(ast, logical.right(ast))
        }
        ExprData::ConditionalExpression(conditional) => {
            is_primitive(ast, conditional.consequent(ast))
                && is_primitive(ast, conditional.alternate(ast))
        }
        _ => false,
    }
}
