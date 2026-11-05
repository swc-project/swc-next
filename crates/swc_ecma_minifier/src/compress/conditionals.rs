//! Rewrites of `if` statements, conditional expressions, and comparisons.
//!
//! The traversal exits the branches of an `if` statement before the
//! statement itself, so a nested `if` has already become an expression when
//! its parent is rewritten. Each rewrite inspects only nodes that are final.

use swc_next_ecma_ast::{
    Ast, BinaryExpression, BinaryOperator, ConditionalExpression, Expr, ExprData, GetSpan,
    LogicalOperator, Span, Stmt, StmtData, SubRange, UnaryOperator,
};

use super::{
    Compressor, statements,
    value::{Value, skip_parentheses},
};

impl Compressor {
    /// Rewrites an `if` statement whose branches are expressions or
    /// returns as an expression statement or a `return` statement, and
    /// shortens the others in place.
    ///
    /// `if (a) b();` becomes `a && b();`, `if (a) b(); else c();` becomes
    /// `a ? b() : c();`, and `if (a) return b; else return c;` becomes
    /// `return a ? b : c;`. Nested `if` statements without `else` join
    /// their tests, and an empty branch is removed.
    pub(super) fn compress_if(&self, ast: &mut Ast<'_>, node: Stmt) -> Option<Stmt> {
        let StmtData::IfStatement(statement) = ast.stmt_data(node) else {
            return None;
        };
        let span = node.span(ast);
        let mut test = statement.test(ast);
        let mut consequent = statement.consequent(ast);
        let mut alternate = statement
            .alternate(ast)
            .filter(|alternate| !is_empty(ast, *alternate));
        if is_empty(ast, consequent) {
            let Some(kept) = alternate.take() else {
                return Some(ast.stmt_expression_statement(span, test));
            };
            if let StmtData::ExpressionStatement(kept) = ast.stmt_data(kept) {
                let expression = guard(ast, span, test, kept.expression(ast), false);
                return Some(ast.stmt_expression_statement(span, expression));
            }
            test = negate(ast, test, true);
            consequent = kept;
        }
        if let Some(kept) = alternate
            && let Some(argument) = logical_not_argument(ast, test)
            && !statements::ends_in_open_if(ast, kept)
        {
            test = argument;
            // A block kept around an `if` without `else` is not needed in
            // the alternate.
            alternate = Some(statements::unwrap_block(ast, consequent));
            consequent = kept;
        }
        if alternate.is_none() {
            while let StmtData::IfStatement(inner) = ast.stmt_data(consequent)
                && inner.alternate(ast).is_none()
            {
                let inner_test = inner.test(ast);
                test = chain(ast, span, test, inner_test, LogicalOperator::And);
                consequent = inner.consequent(ast);
            }
        }
        match (
            ast.stmt_data(consequent),
            alternate.map(|kept| ast.stmt_data(kept)),
        ) {
            (StmtData::ExpressionStatement(kept), None) => {
                let expression = guard(ast, span, test, kept.expression(ast), true);
                return Some(ast.stmt_expression_statement(span, expression));
            }
            (StmtData::ExpressionStatement(left), Some(StmtData::ExpressionStatement(right))) => {
                let (left, right) = (left.expression(ast), right.expression(ast));
                let expression = conditional(ast, span, test, left, right);
                return Some(ast.stmt_expression_statement(span, expression));
            }
            (StmtData::ReturnStatement(left), Some(StmtData::ReturnStatement(right)))
                if left.argument(ast).is_some() && right.argument(ast).is_some() =>
            {
                let (left, right) = (left.argument(ast)?, right.argument(ast)?);
                let expression = conditional(ast, span, test, left, right);
                return Some(ast.stmt_return_statement(span, Some(expression)));
            }
            _ => {}
        }
        if test != statement.test(ast) {
            statement.set_test(ast, test);
        }
        if consequent != statement.consequent(ast) {
            statement.set_consequent(ast, consequent);
        }
        if alternate != statement.alternate(ast) {
            statement.set_alternate(ast, alternate);
        }
        None
    }

    /// Shortens a conditional expression: `!a ? b : c` becomes `a ? c : b`,
    /// and a choice between `true` and `false` becomes a negation.
    pub(super) fn compress_conditional(
        &self,
        ast: &mut Ast<'_>,
        node: Expr,
        conditional: ConditionalExpression,
    ) -> Option<Expr> {
        let test = conditional.test(ast);
        let consequent = conditional.consequent(ast);
        let alternate = conditional.alternate(ast);
        let span = node.span(ast);
        if let Some(choice) = boolean_choice(ast, span, test, consequent, alternate) {
            // Without dead-code removal, the test can be a constant.
            if let ExprData::UnaryExpression(unary) = ast.expr_data(choice) {
                let argument = self.fold_built(ast, unary.argument(ast));
                unary.set_argument(ast, argument);
            }
            return Some(self.fold_built(ast, choice));
        }
        let argument = logical_not_argument(ast, test)?;
        Some(Expr::ConditionalExpression(ast.conditional_expression(
            span, argument, alternate, consequent,
        )))
    }

    /// Folds an expression built from compressed operands, as the
    /// traversal would have folded it.
    fn fold_built(&self, ast: &mut Ast<'_>, expr: Expr) -> Expr {
        if !self.options.evaluate {
            return expr;
        }
        self.fold(ast, expr).unwrap_or(expr)
    }

    /// Removes returns that end a function body. A trailing `return;` is
    /// removed, and the statements that follow `if (a) return;` move into
    /// `if (!a) { ... }` when they declare no block-scoped names.
    ///
    /// Runs before the body list is compressed. The moved statements are
    /// compressed once, as the body of the new block.
    pub(super) fn compress_function_tail(
        &mut self,
        ast: &mut Ast<'_>,
        list: SubRange<Stmt>,
    ) -> Option<SubRange<Stmt>> {
        let mut statements = Vec::new();
        self.flatten_held_blocks(ast, list, &mut statements);
        let mut changed = false;
        if let Some(&last) = statements.last()
            && matches!(ast.stmt_data(last), StmtData::ReturnStatement(statement)
                if statement.argument(ast).is_none())
        {
            statements.pop();
            changed = true;
        }
        let mut index = statements.len();
        while index > 0 {
            index -= 1;
            let Some(test) = return_guard(ast, statements[index]) else {
                continue;
            };
            let tail = &statements[index + 1..];
            if tail
                .iter()
                .any(|statement| super::dead_code::is_lexical(ast, *statement))
            {
                continue;
            }
            let body = if tail.is_empty() {
                None
            } else {
                Some(ast.add_sub_range(tail.iter().copied()))
            };
            // Consecutive guards join their tests, so the tail is rebuilt
            // once for all of them.
            let end = statements[index].span(ast).end;
            let mut tests = vec![test];
            while index > 0
                && let Some(test) = return_guard(ast, statements[index - 1])
            {
                tests.push(test);
                index -= 1;
            }
            let span = Span::new(statements[index].span(ast).start, end);
            let test = tests
                .into_iter()
                .rev()
                .reduce(|left, right| chain(ast, span, left, right, LogicalOperator::Or))
                .unwrap_or(test);
            let (test, body) = match body {
                None => (test, ast.stmt_empty_statement(span)),
                Some(body) => {
                    let block = ast.block_statement(span, body);
                    self.compress_held_block(ast, block);
                    let body = statements::unwrap_block(ast, Stmt::BlockStatement(block));
                    (negate(ast, test, true), body)
                }
            };
            let replacement = ast.stmt_if_statement(span, test, body, None);
            let replacement = self.compress_if(ast, replacement).unwrap_or(replacement);
            let replacement = if self.options.side_effects || self.options.negate_iife {
                self.drop_statement_values(ast, replacement)
                    .unwrap_or(replacement)
            } else {
                replacement
            };
            statements.truncate(index);
            statements.push(replacement);
            changed = true;
        }
        changed.then(|| ast.add_sub_range(statements))
    }

    /// Appends the statements of `list` to `statements`, with the
    /// statements of blocks that the list would merge in their place.
    fn flatten_held_blocks(&self, ast: &Ast<'_>, list: SubRange<Stmt>, statements: &mut Vec<Stmt>) {
        for id in list.iter() {
            let statement = ast.get_node_in_sub_range(id);
            match ast.stmt_data(statement) {
                StmtData::BlockStatement(block)
                    if self.options.blocks && !statements::declares_lexical(ast, block) =>
                {
                    self.flatten_held_blocks(ast, block.body(ast), statements);
                }
                _ => statements.push(statement),
            }
        }
    }

    /// Shortens a strict equality whose operands have the same type, such
    /// as `typeof a === "b"`, to `==`.
    pub(super) fn loosen_equality(&self, ast: &mut Ast<'_>, binary: BinaryExpression) {
        let operator = match binary.operator(ast) {
            BinaryOperator::StrictEqual => BinaryOperator::Equal,
            BinaryOperator::StrictNotEqual => BinaryOperator::NotEqual,
            _ => return,
        };
        let left = primitive_type(ast, binary.left(ast));
        if left.is_some() && left == primitive_type(ast, binary.right(ast)) {
            binary.set_operator(ast, operator);
        }
    }
}

/// Joins `return b;` to a preceding `if (a) return c;` as
/// `return a ? c : b;`.
pub(super) fn join_returns(ast: &mut Ast<'_>, previous: Stmt, statement: Stmt) -> Option<Stmt> {
    let StmtData::ReturnStatement(last) = ast.stmt_data(statement) else {
        return None;
    };
    let StmtData::IfStatement(branch) = ast.stmt_data(previous) else {
        return None;
    };
    if branch.alternate(ast).is_some() {
        return None;
    }
    let StmtData::ReturnStatement(first) = ast.stmt_data(branch.consequent(ast)) else {
        return None;
    };
    let consequent = first.argument(ast)?;
    let alternate = last.argument(ast)?;
    let span = Span::new(previous.span(ast).start, statement.span(ast).end);
    let test = branch.test(ast);
    let expression = conditional(ast, span, test, consequent, alternate);
    Some(ast.stmt_return_statement(span, Some(expression)))
}

/// The test of `if (test) return;`.
fn return_guard(ast: &Ast<'_>, statement: Stmt) -> Option<Expr> {
    let StmtData::IfStatement(branch) = ast.stmt_data(statement) else {
        return None;
    };
    let returns = matches!(ast.stmt_data(branch.consequent(ast)),
        StmtData::ReturnStatement(statement) if statement.argument(ast).is_none());
    (branch.alternate(ast).is_none() && returns).then(|| branch.test(ast))
}

/// Builds `test && expr`, or `test || expr` when `when` is false, removing a
/// negation from `test` by switching the operator.
fn guard(ast: &mut Ast<'_>, span: Span, test: Expr, expr: Expr, when: bool) -> Expr {
    let (test, when) = match logical_not_argument(ast, test) {
        Some(argument) => (argument, !when),
        None => (test, when),
    };
    let operator = if when {
        LogicalOperator::And
    } else {
        LogicalOperator::Or
    };
    chain(ast, span, test, expr, operator)
}

/// Builds `left && right` for `&&` or `||`. A right operand with the same
/// operator is reassociated to the left, so `a && (b && c)` prints as
/// `a && b && c`.
fn chain(
    ast: &mut Ast<'_>,
    span: Span,
    left: Expr,
    right: Expr,
    operator: LogicalOperator,
) -> Expr {
    if let ExprData::LogicalExpression(inner) = ast.expr_data(right)
        && inner.operator(ast) == operator
    {
        let (middle, last) = (inner.left(ast), inner.right(ast));
        let left = chain(ast, span, left, middle, operator);
        return Expr::LogicalExpression(ast.logical_expression(span, left, last, operator));
    }
    Expr::LogicalExpression(ast.logical_expression(span, left, right, operator))
}

/// Builds `test ? consequent : alternate`, removing a negation from `test`
/// by swapping the branches.
fn conditional(
    ast: &mut Ast<'_>,
    span: Span,
    test: Expr,
    consequent: Expr,
    alternate: Expr,
) -> Expr {
    if let Some(choice) = boolean_choice(ast, span, test, consequent, alternate) {
        return choice;
    }
    let (test, consequent, alternate) = match logical_not_argument(ast, test) {
        Some(argument) => (argument, alternate, consequent),
        None => (test, consequent, alternate),
    };
    Expr::ConditionalExpression(ast.conditional_expression(span, test, consequent, alternate))
}

/// Builds `test ? true : false` as `!!test`, or as `test` when it is a
/// boolean, and `test ? false : true` as the negation of `test`.
fn boolean_choice(
    ast: &mut Ast<'_>,
    span: Span,
    test: Expr,
    consequent: Expr,
    alternate: Expr,
) -> Option<Expr> {
    let (Some(Value::Boolean(left)), Some(Value::Boolean(right))) =
        (Value::of(ast, consequent), Value::of(ast, alternate))
    else {
        return None;
    };
    if left == right {
        return None;
    }
    if !left {
        return Some(negate(ast, test, false));
    }
    if primitive_type(ast, test) == Some(PrimitiveType::Boolean) {
        return Some(test);
    }
    let negated = not(ast, span, test);
    Some(not(ast, span, negated))
}

/// The operand of `!a`. Only the truthiness of the result may be used.
fn logical_not_argument(ast: &Ast<'_>, expr: Expr) -> Option<Expr> {
    match ast.expr_data(skip_parentheses(ast, expr)) {
        ExprData::UnaryExpression(unary) if unary.operator(ast) == UnaryOperator::LogicalNot => {
            Some(unary.argument(ast))
        }
        _ => None,
    }
}

/// Builds the shortest negation of `expr`. With `truthy`, only the
/// truthiness of the result is used, so `!a` negates to `a`.
pub(super) fn negate(ast: &mut Ast<'_>, expr: Expr, truthy: bool) -> Expr {
    let span = expr.span(ast);
    match ast.expr_data(skip_parentheses(ast, expr)) {
        ExprData::UnaryExpression(unary)
            if truthy && unary.operator(ast) == UnaryOperator::LogicalNot =>
        {
            unary.argument(ast)
        }
        ExprData::BinaryExpression(binary) => match negated_equality(binary.operator(ast)) {
            Some(operator) => {
                let (left, right) = (binary.left(ast), binary.right(ast));
                Expr::BinaryExpression(ast.binary_expression(span, left, right, operator))
            }
            None => not(ast, span, expr),
        },
        ExprData::SequenceExpression(sequence) => {
            let mut items: Vec<_> = sequence
                .expressions(ast)
                .iter()
                .map(|id| ast.get_node_in_sub_range(id))
                .collect();
            let last = items.len() - 1;
            items[last] = negate(ast, items[last], truthy);
            statements::sequence(ast, &items)
        }
        _ => not(ast, span, expr),
    }
}

/// The equality operator whose result is the negation of `operator`.
pub(super) fn negated_equality(operator: BinaryOperator) -> Option<BinaryOperator> {
    Some(match operator {
        BinaryOperator::Equal => BinaryOperator::NotEqual,
        BinaryOperator::NotEqual => BinaryOperator::Equal,
        BinaryOperator::StrictEqual => BinaryOperator::StrictNotEqual,
        BinaryOperator::StrictNotEqual => BinaryOperator::StrictEqual,
        _ => return None,
    })
}

fn not(ast: &mut Ast<'_>, span: Span, expr: Expr) -> Expr {
    Expr::UnaryExpression(ast.unary_expression(span, expr, UnaryOperator::LogicalNot))
}

/// Whether a statement does nothing.
fn is_empty(ast: &Ast<'_>, statement: Stmt) -> bool {
    match ast.stmt_data(statement) {
        StmtData::EmptyStatement(_) => true,
        StmtData::BlockStatement(block) => block.body(ast).is_empty(),
        _ => false,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PrimitiveType {
    String,
    Boolean,
    Number,
}

/// The type of the value of `expr`, when it is known to be a string, a
/// boolean, or a number.
fn primitive_type(ast: &Ast<'_>, expr: Expr) -> Option<PrimitiveType> {
    match ast.expr_data(skip_parentheses(ast, expr)) {
        ExprData::StringLiteral(_) | ExprData::TemplateLiteral(_) => Some(PrimitiveType::String),
        ExprData::BooleanLiteral(_) => Some(PrimitiveType::Boolean),
        ExprData::NumericLiteral(_) => Some(PrimitiveType::Number),
        ExprData::UnaryExpression(unary) => match unary.operator(ast) {
            UnaryOperator::Typeof => Some(PrimitiveType::String),
            UnaryOperator::LogicalNot | UnaryOperator::Delete => Some(PrimitiveType::Boolean),
            UnaryOperator::Negate
                if matches!(
                    ast.expr_data(unary.argument(ast)),
                    ExprData::NumericLiteral(_)
                ) =>
            {
                Some(PrimitiveType::Number)
            }
            _ => None,
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
            | BinaryOperator::Instanceof => Some(PrimitiveType::Boolean),
            _ => None,
        },
        _ => None,
    }
}
