//! Compression of statement lists and statement bodies.
//!
//! A list is rewritten once, when the traversal exits the node that owns it.
//! Its statements are final by then, so one forward scan joins neighbors
//! without visiting any statement again.

use swc_next_ecma_ast::{
    Ast, BlockStatement, DeclData, Expr, ExprData, ForStatement, ForStatementInit,
    ForStatementInitData, GetSpan, IfStatement, SetSpan, Span, Stmt, StmtData, SubRange,
    VariableDeclaration, VariableDeclarator, VariableKind,
};

use super::dead_code::is_lexical;
use crate::CompressOptions;

/// Reusable buffers for rewriting one statement list at a time.
#[derive(Default)]
pub(super) struct StatementList {
    statements: Vec<Stmt>,
    /// Expressions of the open run of expression statements.
    expressions: Vec<Expr>,
    /// The first statement of the open run of expression statements.
    expression_run: Option<Stmt>,
    /// Number of statements in the open expression run.
    expression_count: usize,
    /// Declarators of the open run of declarations.
    declarators: Vec<VariableDeclarator>,
    /// The first declaration of the open run of declarations.
    declaration_run: Option<(Stmt, VariableDeclaration)>,
    /// Number of statements in the open declaration run.
    declaration_count: usize,
    changed: bool,
}

impl StatementList {
    /// Rewrites `list`, and returns the new list if anything changed.
    ///
    /// With `blocks`, empty statements are removed, and blocks that declare
    /// no block-scoped names are merged into the list. With `sequences`,
    /// adjacent expression statements are joined with the comma operator and
    /// moved into a following `if`, `return`, `throw`, `switch`, or `for`
    /// statement. With `join_vars`, adjacent declarations of the same kind
    /// are joined, and `var` declarations move into a following `for` loop.
    pub(super) fn compress(
        &mut self,
        ast: &mut Ast<'_>,
        options: &CompressOptions,
        list: SubRange<Stmt>,
    ) -> Option<SubRange<Stmt>> {
        if !options.blocks && !options.sequences && !options.join_vars {
            return None;
        }
        debug_assert!(self.statements.is_empty() && !self.changed);
        for id in list.iter() {
            let statement = ast.get_node_in_sub_range(id);
            self.push(ast, options, statement);
        }
        self.flush_expressions(ast);
        self.flush_declarations(ast);
        let changed = std::mem::take(&mut self.changed);
        let list = changed.then(|| ast.add_sub_range(self.statements.iter().copied()));
        self.statements.clear();
        list
    }

    fn push(&mut self, ast: &mut Ast<'_>, options: &CompressOptions, statement: Stmt) {
        match ast.stmt_data(statement) {
            StmtData::EmptyStatement(_) if options.blocks => {
                self.changed = true;
            }
            // Blocks made by dead-code removal have not been compressed, so
            // they may contain blocks too.
            StmtData::BlockStatement(block) if options.blocks && !declares_lexical(ast, block) => {
                self.changed = true;
                for id in block.body(ast).iter() {
                    let statement = ast.get_node_in_sub_range(id);
                    self.push(ast, options, statement);
                }
            }
            StmtData::ExpressionStatement(expression) if options.sequences => {
                self.flush_declarations(ast);
                if self.expression_run.is_none() {
                    self.expression_run = Some(statement);
                }
                self.expression_count += 1;
                push_expressions(ast, expression.expression(ast), &mut self.expressions);
            }
            StmtData::Declaration(declaration) if options.join_vars => {
                self.flush_expressions(ast);
                let DeclData::VariableDeclaration(declaration) = ast.decl_data(declaration) else {
                    self.flush_declarations(ast);
                    self.statements.push(statement);
                    return;
                };
                if !joinable(ast, declaration) {
                    self.flush_declarations(ast);
                    self.statements.push(statement);
                    return;
                }
                if let Some((_, first)) = self.declaration_run
                    && first.kind(ast) != declaration.kind(ast)
                {
                    self.flush_declarations(ast);
                }
                if self.declaration_run.is_none() {
                    self.declaration_run = Some((statement, declaration));
                }
                self.declaration_count += 1;
                self.declarators.extend(
                    declaration
                        .declarators(ast)
                        .iter()
                        .map(|id| ast.get_node_in_sub_range(id)),
                );
            }
            _ => {
                self.absorb_expressions(ast, statement);
                self.absorb_declarations(ast, statement);
                self.statements.push(statement);
            }
        }
    }

    /// Closes the open expression run as one expression statement.
    fn flush_expressions(&mut self, ast: &mut Ast<'_>) {
        let Some(first) = self.expression_run.take() else {
            return;
        };
        let count = std::mem::take(&mut self.expression_count);
        if count == 1 {
            self.expressions.clear();
            self.statements.push(first);
            return;
        }
        self.changed = true;
        let expression = self.take_sequence(ast);
        let span = expression.span(ast);
        self.statements
            .push(ast.stmt_expression_statement(span, expression));
    }

    /// Builds one expression from the open expression run.
    fn take_sequence(&mut self, ast: &mut Ast<'_>) -> Expr {
        let expression = sequence(ast, &self.expressions);
        self.expressions.clear();
        expression
    }

    /// Moves the open expression run into the leading expression of
    /// `statement`, which runs right after it.
    fn absorb_expressions(&mut self, ast: &mut Ast<'_>, statement: Stmt) {
        if self.expression_run.is_none() {
            return;
        }
        // The expression the run joins, or `None` for an empty `for`
        // initializer.
        let target = match ast.stmt_data(statement) {
            StmtData::IfStatement(statement) => Some(statement.test(ast)),
            StmtData::ReturnStatement(statement) if statement.argument(ast).is_some() => {
                statement.argument(ast)
            }
            StmtData::ThrowStatement(statement) => Some(statement.argument(ast)),
            StmtData::SwitchStatement(statement) => Some(statement.discriminant(ast)),
            StmtData::ForStatement(statement) => match statement
                .init(ast)
                .map(|init| ast.for_statement_init_data(init))
            {
                None => None,
                Some(ForStatementInitData::Expr(init)) => Some(init),
                // Only the declaration's initializers are expressions.
                Some(ForStatementInitData::VariableDeclaration(_)) => {
                    self.flush_expressions(ast);
                    return;
                }
            },
            _ => {
                self.flush_expressions(ast);
                return;
            }
        };
        self.expression_run = None;
        self.expression_count = 0;
        self.changed = true;
        if let Some(target) = target {
            push_expressions(ast, target, &mut self.expressions);
        }
        let expression = self.take_sequence(ast);
        match ast.stmt_data(statement) {
            StmtData::IfStatement(statement) => statement.set_test(ast, expression),
            StmtData::ReturnStatement(statement) => statement.set_argument(ast, Some(expression)),
            StmtData::ThrowStatement(statement) => statement.set_argument(ast, expression),
            StmtData::SwitchStatement(statement) => statement.set_discriminant(ast, expression),
            StmtData::ForStatement(statement) => {
                statement.set_init(ast, Some(ForStatementInit::Expr(expression)));
            }
            _ => unreachable!(),
        }
    }

    /// Closes the open declaration run as one declaration.
    fn flush_declarations(&mut self, ast: &mut Ast<'_>) {
        let Some((statement, declaration)) = self.declaration_run.take() else {
            return;
        };
        let count = std::mem::take(&mut self.declaration_count);
        if count > 1 {
            self.changed = true;
            // The first declaration keeps its node, so hoisting records of
            // removed branches still find every name it now declares.
            set_declarators(ast, declaration, &self.declarators);
        }
        self.declarators.clear();
        self.statements.push(statement);
    }

    /// Moves an open `var` run into the initializer of a following `for`.
    fn absorb_declarations(&mut self, ast: &mut Ast<'_>, statement: Stmt) {
        let Some((_, first)) = self.declaration_run else {
            return;
        };
        let StmtData::ForStatement(for_statement) = ast.stmt_data(statement) else {
            self.flush_declarations(ast);
            return;
        };
        if first.kind(ast) != VariableKind::Var {
            // A lexical declaration would become scoped to the loop.
            self.flush_declarations(ast);
            return;
        }
        match for_init_declaration(ast, for_statement) {
            Some(Some(declaration)) => {
                self.declarators.extend(
                    declaration
                        .declarators(ast)
                        .iter()
                        .map(|id| ast.get_node_in_sub_range(id)),
                );
                set_declarators(ast, declaration, &self.declarators);
            }
            Some(None) => {
                set_declarators(ast, first, &self.declarators);
                for_statement.set_init(ast, Some(ForStatementInit::VariableDeclaration(first)));
            }
            None => {
                self.flush_declarations(ast);
                return;
            }
        }
        self.declaration_run = None;
        self.declaration_count = 0;
        self.declarators.clear();
        self.changed = true;
    }
}

/// Returns `Some(None)` for a `for` loop without an initializer, and
/// `Some(Some(_))` for one that declares `var` names. Returns `None`
/// otherwise.
fn for_init_declaration(
    ast: &Ast<'_>,
    statement: ForStatement,
) -> Option<Option<VariableDeclaration>> {
    let Some(init) = statement.init(ast) else {
        return Some(None);
    };
    match ast.for_statement_init_data(init) {
        ForStatementInitData::VariableDeclaration(declaration)
            if declaration.kind(ast) == VariableKind::Var && !declaration.declare(ast) =>
        {
            Some(Some(declaration))
        }
        _ => None,
    }
}

fn joinable(ast: &Ast<'_>, declaration: VariableDeclaration) -> bool {
    matches!(
        declaration.kind(ast),
        VariableKind::Var | VariableKind::Let | VariableKind::Const
    ) && !declaration.declare(ast)
}

fn set_declarators(
    ast: &mut Ast<'_>,
    declaration: VariableDeclaration,
    declarators: &[VariableDeclarator],
) {
    let start = declaration.span(ast).start;
    let end = declarators.last().map_or(start, |last| last.span(ast).end);
    let declarators = ast.add_sub_range(declarators.iter().copied());
    declaration.set_declarators(ast, declarators);
    declaration.set_span(ast, Span::new(start, end.max(declaration.span(ast).end)));
}

/// Appends `expression` to `expressions`, splicing in the items of a
/// sequence.
fn push_expressions(ast: &Ast<'_>, expression: Expr, expressions: &mut Vec<Expr>) {
    match ast.expr_data(expression) {
        ExprData::SequenceExpression(sequence) => expressions.extend(
            sequence
                .expressions(ast)
                .iter()
                .map(|id| ast.get_node_in_sub_range(id)),
        ),
        _ => expressions.push(expression),
    }
}

/// Joins `expressions` with the comma operator.
fn sequence(ast: &mut Ast<'_>, expressions: &[Expr]) -> Expr {
    match expressions {
        [expression] => *expression,
        [first, .., last] => {
            let span = Span::new(first.span(ast).start, last.span(ast).end);
            let expressions = ast.add_sub_range(expressions.iter().copied());
            Expr::SequenceExpression(ast.sequence_expression(span, expressions))
        }
        [] => unreachable!(),
    }
}

/// Whether a block's statements declare block-scoped names.
fn declares_lexical(ast: &Ast<'_>, block: BlockStatement) -> bool {
    block
        .body(ast)
        .iter()
        .any(|id| is_lexical(ast, ast.get_node_in_sub_range(id)))
}

/// Replaces a block body with its only statement, or with an empty statement
/// when it has none. Blocks that declare block-scoped names are kept.
pub(super) fn unwrap_block(ast: &mut Ast<'_>, body: Stmt) -> Stmt {
    let StmtData::BlockStatement(block) = ast.stmt_data(body) else {
        return body;
    };
    let statements = block.body(ast);
    match statements.len() {
        0 => ast.stmt_empty_statement(body.span(ast)),
        1 => {
            let statement = ast.get_node_in_sub_range(statements.first().unwrap());
            if is_lexical(ast, statement) {
                body
            } else {
                statement
            }
        }
        _ => body,
    }
}

/// Unwraps the bodies of an `if` statement.
///
/// When an `else` follows, a consequent that ends in an `if` without `else`
/// is kept in, or wrapped in, a block, so the `else` does not move to the
/// inner `if`. Bodies inside the consequent may have been unwrapped already.
pub(super) fn unwrap_if(ast: &mut Ast<'_>, statement: IfStatement) {
    let consequent = statement.consequent(ast);
    let unwrapped = unwrap_block(ast, consequent);
    let Some(alternate) = statement.alternate(ast) else {
        statement.set_consequent(ast, unwrapped);
        return;
    };
    let unwrapped_alternate = unwrap_block(ast, alternate);
    statement.set_alternate(ast, Some(unwrapped_alternate));
    if !ends_in_open_if(ast, unwrapped) {
        statement.set_consequent(ast, unwrapped);
    } else if unwrapped == consequent {
        let span = consequent.span(ast);
        let body = ast.add_sub_range([consequent]);
        let block = ast.stmt_block_statement(span, body);
        statement.set_consequent(ast, block);
    }
}

/// Whether `statement` ends in an `if` without `else`, which would take an
/// `else` that follows `statement`.
fn ends_in_open_if(ast: &Ast<'_>, statement: Stmt) -> bool {
    match ast.stmt_data(statement) {
        StmtData::IfStatement(statement) => statement
            .alternate(ast)
            .is_none_or(|alternate| ends_in_open_if(ast, alternate)),
        StmtData::LabeledStatement(statement) => ends_in_open_if(ast, statement.body(ast)),
        StmtData::WhileStatement(statement) => ends_in_open_if(ast, statement.body(ast)),
        StmtData::ForStatement(statement) => ends_in_open_if(ast, statement.body(ast)),
        StmtData::ForInStatement(statement) => ends_in_open_if(ast, statement.body(ast)),
        StmtData::ForOfStatement(statement) => ends_in_open_if(ast, statement.body(ast)),
        StmtData::WithStatement(statement) => ends_in_open_if(ast, statement.body(ast)),
        _ => false,
    }
}
