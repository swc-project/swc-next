//! Removal of branches that a constant condition never runs.
//!
//! Removed code may declare `var` bindings, which are hoisted to the enclosing
//! function and stay observable. The compressor records `var` declarations
//! while it traverses each branch, so removing a branch keeps those names
//! without walking the removed code again.

use std::ops::Range;

use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{
    Ast, BindingIdentifier, BindingPattern, BindingPatternData, DeclData, Expr, ExprData, GetSpan,
    NodeId, Span, Stmt, StmtData, VariableDeclaration, VariableDeclarator, VariableKind,
};

use super::value::skip_parentheses;

/// A hoisted declaration in an open function, in source order.
#[derive(Clone, Copy)]
pub(super) enum Hoisted {
    Var(VariableDeclaration),
    /// A function declaration. In a block, its hoisting depends on strict
    /// mode and on conflicting lexical names, so branches that contain one
    /// are kept.
    Function,
}

/// An open `if`, `while`, or `for` statement.
pub(super) struct Branch {
    /// Index of the statement's first entry in the hoisted list.
    pub(super) start: usize,
    /// The `else` statement of an `if`.
    pub(super) alternate: Option<NodeId>,
    /// Index of the alternate's first entry, set when it is entered.
    pub(super) middle: Option<usize>,
}

impl Branch {
    /// Entries in the consequent of an `if`, or in the body of a loop.
    pub(super) fn consequent(&self, end: usize) -> Range<usize> {
        self.start..self.middle.unwrap_or(end)
    }

    /// Entries in the alternate of an `if`.
    pub(super) fn alternate(&self, end: usize) -> Range<usize> {
        self.middle.unwrap_or(end)..end
    }
}

/// Returns the bindings that the `var` declarations in `entries` introduce,
/// excluding `exclude` and names that `keep` rejects, without duplicates.
///
/// Returns `None` if an entry is a function declaration or a binding cannot be
/// named.
pub(super) fn hoisted_bindings(
    ast: &Ast<'_>,
    entries: &[Hoisted],
    exclude: Option<VariableDeclaration>,
    keep: impl Fn(BindingIdentifier) -> bool,
) -> Option<Vec<BindingIdentifier>> {
    let mut bindings = Vec::new();
    for entry in entries {
        let Hoisted::Var(declaration) = *entry else {
            return None;
        };
        if exclude.is_some_and(|exclude| exclude.node_id() == declaration.node_id()) {
            continue;
        }
        for id in declaration.declarators(ast).iter() {
            let declarator = ast.get_node_in_sub_range(id);
            collect_bindings(ast, declarator.id(ast), &mut bindings)?;
        }
    }
    let mut names = FxHashSet::default();
    bindings.retain(|binding| keep(*binding) && names.insert(ast.get_utf8(binding.name(ast))));
    Some(bindings)
}

fn collect_bindings(
    ast: &Ast<'_>,
    pattern: BindingPattern,
    bindings: &mut Vec<BindingIdentifier>,
) -> Option<()> {
    match ast.binding_pattern_data(pattern) {
        BindingPatternData::BindingIdentifier(binding) => bindings.push(binding),
        BindingPatternData::SimpleAssignmentTarget(_) => return None,
        BindingPatternData::AssignmentPattern(pattern) => {
            collect_bindings(ast, pattern.left(ast), bindings)?;
        }
        BindingPatternData::BindingRestElement(rest) => {
            collect_bindings(ast, rest.argument(ast), bindings)?;
        }
        BindingPatternData::ArrayPattern(pattern) => {
            for id in pattern.elements(ast).iter() {
                if let Some(element) = ast.get_node_in_sub_range(id) {
                    collect_bindings(ast, element, bindings)?;
                }
            }
            if let Some(rest) = pattern.rest(ast) {
                collect_bindings(ast, rest.argument(ast), bindings)?;
            }
        }
        BindingPatternData::ObjectPattern(pattern) => {
            for id in pattern.properties(ast).iter() {
                let property = ast.get_node_in_sub_range(id);
                collect_bindings(ast, property.value(ast), bindings)?;
            }
            if let Some(rest) = pattern.rest(ast) {
                collect_bindings(ast, rest.argument(ast), bindings)?;
            }
        }
    }
    Some(())
}

/// Builds uninitialized declarators that reuse the names of `bindings`.
pub(super) fn declarators(
    ast: &mut Ast<'_>,
    bindings: &[BindingIdentifier],
) -> Vec<VariableDeclarator> {
    bindings
        .iter()
        .map(|binding| {
            let span = binding.span(ast);
            let name = binding.name(ast);
            let id = ast.binding_pattern_binding_identifier(span, name);
            ast.variable_declarator(span, id, None, None, false)
        })
        .collect()
}

pub(super) fn var_statement(
    ast: &mut Ast<'_>,
    span: Span,
    declarators: Vec<VariableDeclarator>,
) -> Stmt {
    let declarators = ast.add_sub_range(declarators);
    ast.stmt_decl_variable_declaration(span, VariableKind::Var, declarators, false)
}

/// Whether `statement` declares a block-scoped name. Function declarations
/// count: in a block they are block-scoped.
pub(super) fn is_lexical(ast: &Ast<'_>, statement: Stmt) -> bool {
    let StmtData::Declaration(declaration) = ast.stmt_data(statement) else {
        return false;
    };
    !matches!(ast.decl_data(declaration), DeclData::VariableDeclaration(declaration)
        if declaration.kind(ast) == VariableKind::Var)
}

/// Joins statements that replace a removed statement into one statement.
///
/// `hoisted` comes first. A kept block is merged into the result unless it
/// declares block-scoped names, which could conflict with hoisted names. A
/// declaration is wrapped in a block, because a function declaration directly
/// in an `if` behaves like one in a block.
pub(super) fn join(
    ast: &mut Ast<'_>,
    span: Span,
    hoisted: Option<Stmt>,
    kept: Option<Stmt>,
) -> Stmt {
    let mut statements: Vec<Stmt> = hoisted.into_iter().collect();
    if let Some(kept) = kept {
        match ast.stmt_data(kept) {
            StmtData::BlockStatement(block)
                if !statements.is_empty()
                    && !block
                        .body(ast)
                        .iter()
                        .any(|id| is_lexical(ast, ast.get_node_in_sub_range(id))) =>
            {
                statements.extend(
                    block
                        .body(ast)
                        .iter()
                        .map(|id| ast.get_node_in_sub_range(id)),
                );
            }
            _ => statements.push(kept),
        }
    }
    match statements.as_slice() {
        [] => ast.stmt_empty_statement(span),
        [statement] if !is_lexical(ast, *statement) => *statement,
        _ => {
            let body = ast.add_sub_range(statements);
            ast.stmt_block_statement(span, body)
        }
    }
}

/// Whether replacing a callee, tag, or `delete` or `typeof` operand with
/// `expr` would change the call receiver or the operand reference.
pub(super) fn is_reference(ast: &Ast<'_>, expr: Expr) -> bool {
    matches!(
        ast.expr_data(skip_parentheses(ast, expr)),
        ExprData::IdentifierReference(_)
            | ExprData::MemberExpression(_)
            | ExprData::ChainExpression(_)
    )
}
