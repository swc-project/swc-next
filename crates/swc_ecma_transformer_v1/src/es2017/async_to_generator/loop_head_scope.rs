use std::collections::HashMap;

use swc_next_ecma_ast::*;

use crate::common::{build as b, resolver::context::Resolver};

/// Uninitialized bindings visible to the iterable, separate from the bindings
/// created for each iteration.
pub(super) struct LoopHeadScope {
    bindings: Vec<BindingIdentifier>,
}

impl LoopHeadScope {
    pub(super) fn new(
        ast: &mut Ast<'_>,
        resolver: &mut Resolver,
        statement: ForOfStatement,
    ) -> Option<Self> {
        let declaration = statement.left(ast).as_variable_declaration(ast)?;
        if declaration.kind(ast) == VariableKind::Var {
            return None;
        }
        let first = declaration.declarators(ast).get_node(ast, 0)?;
        let bindings = b::find_pat_ids(ast, first.id(ast));
        let mut usages = LoopHeadUsages {
            ast,
            resolver,
            references: bindings
                .iter()
                .map(|binding| {
                    let id = resolver.symbol_of(binding.node_id());
                    (id, Vec::new())
                })
                .collect(),
            has_eval: false,
        };
        statement.right(ast).visit_with(&mut usages);
        let LoopHeadUsages {
            mut references,
            has_eval,
            ..
        } = usages;
        let mut aliases = Vec::new();
        for binding in bindings {
            let id = resolver.symbol_of(binding.node_id());
            let references = references.remove(&id).unwrap_or_default();
            if has_eval || !references.is_empty() {
                let alias = resolver.private_binding(ast, ast.get_utf8(binding.name(ast)));
                for reference in references {
                    resolver.copy_context(alias.node_id(), reference);
                }
                aliases.push(alias);
            }
        }
        (!aliases.is_empty()).then_some(Self { bindings: aliases })
    }

    pub(super) fn init_iterator(
        self,
        ast: &mut Ast<'_>,
        resolver: &mut Resolver,
        iterator: BindingIdentifier,
        init: Expr,
        statements: &mut Vec<Stmt>,
    ) {
        statements.push(Stmt::Declaration(b::variable(
            ast,
            VariableKind::Var,
            iterator,
            None,
            DUMMY_SP,
        )));
        // The lexical for-in head creates a TDZ for the RHS. Iterating over
        // null leaves those bindings uninitialized, including in closures.
        let bindings = self
            .bindings
            .into_iter()
            .map(|binding| Some(BindingPattern::BindingIdentifier(binding)))
            .collect();
        let pattern = b::array_pattern(ast, bindings);
        let declarator = ast.variable_declarator(DUMMY_SP, pattern, None, None, false);
        let declarators = ast.add_sub_range([declarator]);
        let declaration = ast.variable_declaration(DUMMY_SP, VariableKind::Let, declarators, false);
        let reference = resolver.reference(ast, iterator);
        let assignment = b::assign(ast, reference, init);
        let null = ast.expr_null_literal(DUMMY_SP);
        let sequence = b::sequence(ast, vec![assignment, null]);
        let body = Stmt::BlockStatement(b::block(ast, []));
        let left = ForStatementLeft::VariableDeclaration(declaration);
        statements.push(Stmt::ForInStatement(
            ast.for_in_statement(DUMMY_SP, left, sequence, body),
        ));
    }
}

/// Collect only those references whose scope is the loop head's scope.
struct LoopHeadUsages<'a, 'ast> {
    ast: &'a Ast<'ast>,
    resolver: &'a Resolver,
    references: HashMap<SymbolId, Vec<NodeId>>,
    has_eval: bool,
}

impl<'ast> Visit<'ast> for LoopHeadUsages<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_identifier_reference(&mut self, reference: IdentifierReference) {
        if let Some(id) = self.resolver.symbol(reference.node_id())
            && let Some(references) = self.references.get_mut(&id)
        {
            references.push(reference.node_id());
        }
    }

    fn visit_call_expression(&mut self, call: CallExpression) {
        self.has_eval |= !call.optional(self.ast)
            && b::unwrap(self.ast, call.callee(self.ast))
                .as_identifier_reference(self.ast)
                .is_some_and(|id| self.ast.get_utf8(id.name(self.ast)) == "eval");
        call.visit_children_with(self);
    }
}
