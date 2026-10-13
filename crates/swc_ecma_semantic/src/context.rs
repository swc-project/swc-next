use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, Class, Directive, Function, FunctionBody, NodeId,
};

use crate::{
    Semantic,
    binder::{ReferenceFlags, ReferenceSpace, SymbolTracker},
    module_record::ModuleRecordBuilder,
    scope::{DecoratorSite, ScopeTracker},
};

/// Walk context combining the current node, scope tracker, symbol tracker,
/// and reference state. Visitor hooks use it to inspect surrounding source
/// structure or to query and contribute symbols.
pub(crate) struct SemanticContext<'a> {
    pub scope: ScopeTracker<'a>,
    pub symbols: SymbolTracker<'a>,
    pub module_record: ModuleRecordBuilder<'a>,
    // Nested type syntax saves and restores this default through the visitor.
    default_reference_flags: ReferenceFlags,
    ts_namespace_depth: u32,
    reference_flags: ReferenceFlags,
}

impl<'a> SemanticContext<'a> {
    pub fn init(tree: &Ast<'a>, build_module_record: bool) -> Self {
        let allocator = tree.allocator();
        Self {
            scope: ScopeTracker::init(tree),
            symbols: SymbolTracker::init(tree),
            module_record: ModuleRecordBuilder::new(allocator, build_module_record),
            default_reference_flags: ReferenceFlags::default(),
            ts_namespace_depth: 0,
            reference_flags: ReferenceFlags::default(),
        }
    }

    /// True when the walker is currently inside a TS type-only subtree.
    pub fn in_type_position(&self) -> bool {
        self.default_reference_flags.space == ReferenceSpace::Type
    }

    /// True when the walker is currently inside a TS namespace body.
    pub fn in_ts_namespace(&self) -> bool {
        self.ts_namespace_depth > 0
    }

    #[inline]
    pub fn enter_node(&mut self) -> ReferenceFlags {
        let flags = self.reference_flags;
        self.reference_flags = self.default_reference_flags();
        flags
    }

    #[inline]
    pub fn default_reference_flags(&self) -> ReferenceFlags {
        self.default_reference_flags
    }

    #[inline]
    pub fn replace_reference_flags(&mut self, flags: ReferenceFlags) -> ReferenceFlags {
        std::mem::replace(&mut self.reference_flags, flags)
    }

    #[inline]
    pub fn enter_type_position(&mut self) -> ReferenceFlags {
        let flags = ReferenceFlags::default()
            .with_space(ReferenceSpace::Type)
            .no_access();
        std::mem::replace(&mut self.default_reference_flags, flags)
    }

    #[inline]
    pub fn exit_type_position(&mut self, previous: ReferenceFlags) {
        debug_assert!(self.in_type_position());
        self.default_reference_flags = previous;
        self.reference_flags = previous;
    }

    pub fn enter_directive(&mut self, tree: &Ast, directive: Directive) {
        self.scope.enter_directive(tree, directive);
    }

    pub fn enter_function(&mut self, tree: &Ast, function: Function) {
        self.scope.enter_function(tree, function);
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

    pub fn enter_class(&mut self, class: Class) {
        self.scope.enter_class(class);
    }

    pub fn enter_static_block(&mut self, node: NodeId) {
        self.scope.enter_static_block(node);
    }

    pub fn enter_decorator(&mut self, node: NodeId, site: DecoratorSite) {
        self.scope.enter_decorator(node, site);
    }

    pub fn exit_scope(&mut self, node: NodeId) {
        self.scope.exit_scope(node);
    }

    pub fn exit_decorator(&mut self) {
        self.scope.exit_decorator();
    }

    pub fn finalize(self, tree: &Ast<'a>) -> Semantic<'a> {
        let scopes = self.scope.into_scope_tree();
        self.symbols.finalize(tree, scopes, self.module_record)
    }
}
