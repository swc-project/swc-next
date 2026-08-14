use swc_next_allocator::vec::Vec as ArenaVec;
use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, Class, Directive, Function, FunctionBody, NodeId, ScopeId,
};

use crate::{
    Semantic,
    binder::{ReferenceFlags, ReferenceSpace, SymbolTracker},
    module_record::ModuleRecordBuilder,
    scope::ScopeTracker,
};

pub(crate) struct SemanticVisitCtx<'traverse, 'ast> {
    pub state: SemanticContext<'ast>,
    pub ast: &'traverse Ast<'ast>,
    current_node: Option<NodeId>,
}

impl<'traverse, 'ast> SemanticVisitCtx<'traverse, 'ast> {
    pub fn new(ast: &'traverse Ast<'ast>, state: SemanticContext<'ast>) -> Self {
        Self {
            state,
            ast,
            current_node: None,
        }
    }

    pub fn enter_node(&mut self, node: NodeId) -> ReferenceFlags {
        let parent = self.current_node;
        self.current_node = Some(node);
        self.state.enter_node(node, parent)
    }

    pub fn leave_node(&mut self, node: NodeId) {
        debug_assert_eq!(self.current_node, Some(node));
        self.current_node = self.state.node_parents[node.index()];
    }

    pub fn parent(&self) -> Option<NodeId> {
        self.current_node
            .and_then(|node| self.state.node_parents[node.index()])
    }

    pub fn current_node(&self) -> Option<NodeId> {
        self.current_node
    }

    pub fn ancestors(&self) -> impl Iterator<Item = NodeId> + '_ {
        std::iter::successors(self.current_node, |node| {
            self.state.node_parents[node.index()]
        })
    }

    pub fn finalize(self) -> Semantic<'ast> {
        self.state.finalize(self.ast)
    }
}

/// Walk context combining the current node, scope tracker, symbol tracker,
/// and reference state. Visitor hooks use it to inspect surrounding source
/// structure or to query and contribute symbols.
pub(crate) struct SemanticContext<'a> {
    pub scope: ScopeTracker<'a>,
    pub symbols: SymbolTracker<'a>,
    pub module_record: ModuleRecordBuilder<'a>,
    // depth of TS type-only context. inspect via `in_type_position()`.
    type_position_depth: u32,
    ts_namespace_depth: u32,
    reference_flags: ReferenceFlags,
    // per-node tables carried into the final `Semantic`
    node_scopes: ArenaVec<'a, ScopeId>,
    node_parents: ArenaVec<'a, Option<NodeId>>,
}

impl<'a> SemanticContext<'a> {
    pub fn init(tree: &Ast<'a>) -> Self {
        let allocator = tree.allocator();
        let mut node_scopes = ArenaVec::with_capacity_in(tree.node_count(), allocator);
        let mut node_parents = ArenaVec::with_capacity_in(tree.node_count(), allocator);
        node_scopes.resize(tree.node_count(), ScopeId::ROOT);
        node_parents.resize(tree.node_count(), None);
        Self {
            scope: ScopeTracker::init(tree),
            symbols: SymbolTracker::init(tree),
            module_record: ModuleRecordBuilder::new(allocator),
            type_position_depth: 0,
            ts_namespace_depth: 0,
            reference_flags: ReferenceFlags::default(),
            node_scopes,
            node_parents,
        }
    }

    /// True when the walker is currently inside a TS type-only subtree.
    pub fn in_type_position(&self) -> bool {
        self.type_position_depth > 0
    }

    /// True when the walker is currently inside a TS namespace body.
    pub fn in_ts_namespace(&self) -> bool {
        self.ts_namespace_depth > 0
    }

    #[inline]
    pub fn enter_node(&mut self, node: NodeId, parent: Option<NodeId>) -> ReferenceFlags {
        self.node_scopes[node.index()] = self.scope.current;
        self.node_parents[node.index()] = parent;
        let flags = self.reference_flags;
        self.reference_flags = self.default_reference_flags();
        flags
    }

    #[inline]
    pub fn default_reference_flags(&self) -> ReferenceFlags {
        ReferenceFlags::default().with_space(if self.in_type_position() {
            ReferenceSpace::Type
        } else {
            ReferenceSpace::Value
        })
    }

    #[inline]
    pub fn replace_reference_flags(&mut self, flags: ReferenceFlags) -> ReferenceFlags {
        std::mem::replace(&mut self.reference_flags, flags)
    }

    #[inline]
    pub fn enter_type_position(&mut self) {
        self.type_position_depth += 1;
    }

    #[inline]
    pub fn exit_type_position(&mut self) {
        debug_assert!(self.type_position_depth > 0);
        self.type_position_depth -= 1;
        self.reference_flags = self.default_reference_flags();
    }

    pub fn enter_directive(&mut self, tree: &Ast, directive: Directive) {
        self.scope.enter_directive(tree, directive);
    }

    pub fn enter_function(&mut self, tree: &Ast, function: Function, annex_b_if_body: bool) {
        self.scope.enter_function(tree, function, annex_b_if_body);
    }

    pub fn enter_ts_function(&mut self, node: NodeId) {
        self.scope.enter_ts_function(node);
    }

    pub fn enter_arrow_function_expression(&mut self, tree: &Ast, arrow: ArrowFunctionExpression) {
        self.scope.enter_arrow_function_expression(tree, arrow);
    }

    pub fn enter_function_body(&mut self, tree: &Ast, body: FunctionBody, parent: Option<NodeId>) {
        self.scope.enter_function_body(tree, body, parent);
    }

    pub fn enter_block_statement(&mut self, tree: &Ast, node: NodeId) {
        self.scope.enter_block_statement(tree, node);
    }

    pub fn enter_with_statement(&mut self, node: NodeId) {
        self.scope.enter_with_scope(node);
        // The object expression evaluates in the enclosing environment, so
        // the with scope is created only after visiting it. Keep the creator
        // node associated with the scope it creates.
        self.node_scopes[node.index()] = self.scope.current;
    }

    pub fn exit_with_statement(&mut self, node: NodeId) {
        self.scope.exit_with_scope(node);
    }

    pub fn enter_switch_case(&mut self, parent: Option<NodeId>) {
        self.scope.enter_switch_case(parent);
    }

    pub fn enter_ts_enum_body(&mut self, node: NodeId) {
        self.scope.enter_ts_enum_body(node);
    }

    fn enter_block_scope(&mut self, node: NodeId) {
        self.scope.enter_block_scope(node);
    }

    pub fn enter_for_statement(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_for_in_statement(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_for_of_statement(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_catch_clause(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_interface_declaration(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_type_alias_declaration(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_function_type(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_constructor_type(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_method_signature(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_call_signature_declaration(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_construct_signature_declaration(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_index_signature(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_mapped_type(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_conditional_type(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_module_block(&mut self, node: NodeId) {
        self.ts_namespace_depth += 1;
        self.scope.enter_ts_module_block(node);
    }

    pub fn exit_ts_module_block(&mut self, node: NodeId) {
        debug_assert!(self.ts_namespace_depth > 0);
        self.scope.exit_scope(node);
        self.ts_namespace_depth -= 1;
    }

    pub fn enter_class(&mut self, tree: &Ast, class: Class) {
        self.scope.enter_class(tree, class);
    }

    pub fn enter_static_block(&mut self, node: NodeId) {
        self.scope.enter_static_block(node);
    }

    pub fn enter_decorator(&mut self) {
        self.scope.enter_decorator();
    }

    pub fn exit_scope(&mut self, node: NodeId) {
        self.scope.exit_scope(node);
    }

    pub fn exit_decorator(&mut self) {
        self.scope.exit_decorator();
    }

    pub fn finalize(self, tree: &Ast) -> Semantic<'a> {
        let (scopes, has_with_statement) = self.scope.into_scope_tree();
        self.symbols.finalize(
            tree,
            scopes,
            has_with_statement,
            self.node_scopes,
            self.node_parents,
            self.module_record,
        )
    }
}
