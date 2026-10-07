use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, AstUtf8, BindingIdentifier, DUMMY_SP, Expr, IdentifierReference,
    NodeId, Program, PropertyKey, ScopeId, Span, SymbolId,
};
mod construction;
mod expressions;
mod mutation;
mod names;
mod scopes;
mod statement_injector;

use crate::state::TransformerState;
pub(crate) use mutation::ClassElementAction;
pub use mutation::StatementList;
pub(crate) use scopes::ScopeOptions;
pub(crate) use statement_injector::StatementInjector;
use swc_next_ecma_semantic::Semantic;

/// Borrow AST and transform state together for synchronized mutations.
pub trait Mutate<'s, 'a> {
    fn mutate(&mut self) -> Mutation<'_, 's, 'a>;
}

impl<'s, 'a> Mutate<'s, 'a>
    for swc_next_ecma_ast::TraverseMutCtx<'_, 'a, TransformerState<'s, 'a>>
{
    fn mutate(&mut self) -> Mutation<'_, 's, 'a> {
        Mutation {
            ast: self.ast,
            state: &mut self.state,
        }
    }
}

/// An atomic mutation interface borrowing the traversal's AST, state and path.
pub struct Mutation<'c, 's, 'a> {
    ast: &'c mut Ast<'a>,
    state: &'c mut TransformerState<'s, 'a>,
}

impl<'c, 's, 'a> Mutation<'c, 's, 'a> {
    pub fn assumptions(&self) -> &crate::assumptions::Assumptions {
        &self.state.assumptions
    }

    pub fn ast(&self) -> &Ast<'a> {
        self.ast
    }
    // Normalization removes parentheses and associative containers. It
    // preserves semantic-bearing leaves and scope owners, and allocates no
    // AST nodes. Lookup scope owners and export expression roots can still change.
    pub(crate) fn normalize(&mut self) {
        crate::normalize::Normalizer::normalize(self.ast.root_program(), self.ast);
        self.state.semantic.normalize_scope_owners(self.ast);
        self.state.semantic.normalize_exports(self.ast);
    }

    pub fn semantic(&self) -> &Semantic<'a> {
        self.state.semantic
    }

    pub(crate) fn take_static_block_layout(
        &mut self,
        class: NodeId,
    ) -> swc_next_ecma_semantic::ClassStaticBlockLayout {
        self.state
            .semantic
            .class_mut()
            .take_static_block_layout(class)
            .expect("closure lowering requires static block layout information")
    }

    pub fn generate_binding(
        &mut self,
        base: &str,
        scope: ScopeId,
        flags: swc_next_ecma_semantic::SymbolFlags,
    ) -> GeneratedBinding {
        let name = self.generate_name(base, 2);
        let declaration = self.ast.binding_identifier(DUMMY_SP, name);
        let symbol = self
            .state
            .semantic
            .register_binding(self.ast, declaration, scope, flags);
        let binding = GeneratedBinding {
            declaration,
            symbol,
        };
        self.state.generated_bindings.insert(name, binding);
        binding
    }

    pub fn create_binding_reference(
        &mut self,
        symbol: SymbolId,
        scope: ScopeId,
    ) -> IdentifierReference {
        let name = self.state.semantic.symbol(symbol).name;
        let name = self
            .ast
            .get_wtf8(name)
            .as_str()
            .expect("generated identifier is UTF-8")
            .to_owned();
        let name = self.ast.add_utf8(&name);
        let node = self.ast.identifier_reference(DUMMY_SP, name);
        self.state.semantic.register_reference(
            self.ast,
            node,
            scope,
            Some(symbol),
            Default::default(),
        );
        node
    }

    pub fn create_unbound_reference(&mut self, name: &str, scope: ScopeId) -> IdentifierReference {
        let name = self.ast.add_utf8(name);
        self.observe_name(name);
        let node = self.ast.identifier_reference(DUMMY_SP, name);
        self.state
            .semantic
            .register_reference(self.ast, node, scope, None, Default::default());
        node
    }

    pub(crate) fn path(&self) -> &[NodeId] {
        &self.state.path
    }

    pub(crate) fn current_node(&self) -> Option<NodeId> {
        self.state.path.last().copied()
    }

    pub(crate) fn parent(&self) -> Option<NodeId> {
        self.state.path.iter().rev().nth(1).copied()
    }

    pub(crate) fn ancestors(&self) -> impl DoubleEndedIterator<Item = NodeId> + '_ {
        self.state.path.iter().rev().copied()
    }

    pub(crate) fn depth(&self) -> usize {
        self.state.path.len().saturating_sub(1)
    }

    /// The innermost scope owned by a node on the current traversal path.
    pub fn current_scope(&self) -> ScopeId {
        self.ancestors()
            .find_map(|node| self.state.semantic.scope_at(node))
            .unwrap_or(ScopeId::ROOT)
    }

    pub fn create_reference(&mut self, name: &str, scope: ScopeId) -> IdentifierReference {
        let symbol = self.state.semantic.lookup(
            scope,
            name.as_bytes(),
            swc_next_ecma_semantic::ReferenceSpace::Value,
        );
        let name = self.ast.add_utf8(name);
        self.observe_name(name);
        let node = self.ast.identifier_reference(DUMMY_SP, name);
        self.state
            .semantic
            .register_reference(self.ast, node, scope, symbol, Default::default());
        node
    }

    pub(crate) fn generated_reference(&mut self, name: AstUtf8, span: Span) -> IdentifierReference {
        self.generated_reference_in_scope(name, span, self.current_scope())
    }

    pub(crate) fn generated_reference_in_scope(
        &mut self,
        name: AstUtf8,
        span: Span,
        scope: ScopeId,
    ) -> IdentifierReference {
        let symbol = self.state.generated_bindings[&name].symbol;
        let node = self.ast.identifier_reference(span, name);
        self.state.semantic.register_reference(
            self.ast,
            node,
            scope,
            Some(symbol),
            Default::default(),
        );
        node
    }

    pub(crate) fn retarget_generated_reference(
        &mut self,
        node: IdentifierReference,
        name: AstUtf8,
    ) {
        let symbol = self.state.generated_bindings[&name].symbol;
        self.state
            .semantic
            .retarget_reference(node.node_id(), name, symbol);
        node.set_name(self.ast, name);
    }

    pub(crate) fn generated_declaration(&self, name: AstUtf8) -> BindingIdentifier {
        self.state.generated_bindings[&name].declaration
    }

    pub(crate) fn program_scope(&self) -> ScopeId {
        self.scope_for_node(self.ast.root_program().node_id())
            .expect("program scope exists")
    }

    pub(crate) fn helper_callee(
        &mut self,
        helper: crate::helpers::HelperKind,
        scope: ScopeId,
    ) -> Expr {
        use crate::helpers;
        let name = match self.state.helpers.name(helper) {
            Some(name) => name,
            None => {
                let prepared = helpers::prepare(helper, self);
                let name = prepared.name;
                self.state.helpers.insert(helper, prepared);
                name
            }
        };
        helpers::callee(name, helper, scope, self)
    }

    pub(crate) fn inject_helpers(&mut self, program: Program) {
        let statements = self.state.helpers.take_statements();
        for statement in statements.into_iter().flatten() {
            self.inject_statement(program.node_id(), statement);
        }
    }

    pub(crate) fn flush_statements(&mut self, container: NodeId) {
        let mut injector = std::mem::take(&mut self.state.statement_injector);
        injector.flush(container, self);
        self.state.statement_injector = injector;
        self.state.semantic.grow_node_indexes(self.ast.node_count());
    }

    pub(crate) fn flush_arrow_statements(&mut self, arrow: ArrowFunctionExpression) {
        let mut injector = std::mem::take(&mut self.state.statement_injector);
        injector.flush_arrow_expression(arrow, self);
        self.state.statement_injector = injector;
        self.state.semantic.grow_node_indexes(self.ast.node_count());
    }

    pub(crate) fn generate_var_name(&mut self, base: &str) -> AstUtf8 {
        let container = self.state.statement_injector.current_var_scope();
        let scope = self
            .state
            .semantic
            .scope_at(container)
            .expect("variable container is registered");
        self.generate_binding(
            base,
            scope,
            swc_next_ecma_semantic::SymbolFlags::FUNCTION_SCOPED_VAR,
        )
        .declaration
        .name(self.ast)
    }

    pub(crate) fn assign_generated(
        &mut self,
        name: AstUtf8,
        value: Expr,
        span: Span,
        scope: ScopeId,
    ) -> Expr {
        use swc_next_ecma_ast::{AssignmentOperator, AssignmentTarget, SimpleAssignmentTarget};
        let symbol = self.state.generated_bindings[&name].symbol;
        let node = self.ast.identifier_reference(span, name);
        self.state.semantic.register_reference(
            self.ast,
            node,
            scope,
            Some(symbol),
            swc_next_ecma_semantic::ReferenceFlags::generated_assignment(false),
        );
        let left = AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::IdentifierReference(node),
        );
        Expr::AssignmentExpression(self.add_node(scope, |ast| {
            ast.assignment_expression(span, left, value, AssignmentOperator::Assign)
        }))
    }

    pub(crate) fn next_private_name(&mut self, class: NodeId) -> PropertyKey {
        let scope = self.scope_for_node(class).expect("class scope exists");
        self.state
            .semantic
            .reserve_private_key(self.ast, class, scope)
    }

    pub(crate) fn finish(&mut self) {
        debug_assert!(self.state.statement_injector.is_empty());
        debug_assert!(self.state.helpers.is_empty());
        #[cfg(debug_assertions)]
        self.state.semantic.assert_invariants(self.ast);
    }
}

/// A generated binding keeps its declaration and symbol together while its
/// declaration is waiting for statement injection.
#[derive(Clone, Copy)]
pub struct GeneratedBinding {
    pub declaration: BindingIdentifier,
    pub symbol: SymbolId,
}

#[cfg(test)]
#[path = "../../tests/context/support.rs"]
mod tests;
