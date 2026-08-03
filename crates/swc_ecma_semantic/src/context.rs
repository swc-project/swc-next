use std::mem::MaybeUninit;

use swc_next_allocator::vec::Vec as ArenaVec;
use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, Class, Directive, Function, FunctionBody, NodeId, Program,
    ScopeId,
};

use crate::{
    Semantic,
    binder::{ReferenceFlags, ReferenceSpace, SymbolTracker},
    scope::{ScopeKind, ScopeTracker},
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
        // SAFETY: `enter_node` initialized this slot before visiting children.
        self.current_node = unsafe { *self.state.node_parents[node.index()].assume_init_ref() };
    }

    pub fn parent(&self) -> Option<NodeId> {
        self.current_node.and_then(|node| {
            // SAFETY: Every current node has already passed through `enter_node`.
            unsafe { *self.state.node_parents[node.index()].assume_init_ref() }
        })
    }

    pub fn current_node(&self) -> Option<NodeId> {
        self.current_node
    }

    pub fn ancestors(&self) -> impl Iterator<Item = NodeId> + '_ {
        std::iter::successors(self.current_node, |node| {
            // SAFETY: Ancestors were initialized when traversal entered them.
            unsafe { *self.state.node_parents[node.index()].assume_init_ref() }
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
    // depth of TS type-only context. inspect via `in_type_position()`.
    type_position_depth: u32,
    reference_flags: ReferenceFlags,
    // per-node tables carried into the final `Semantic`
    node_scopes: ArenaVec<'a, MaybeUninit<ScopeId>>,
    node_parents: ArenaVec<'a, MaybeUninit<Option<NodeId>>>,
}

impl<'a> SemanticContext<'a> {
    pub fn init(root: Program, tree: &Ast<'a>) -> Self {
        let allocator = tree.allocator();
        let node_scopes = uninit_node_table(tree.node_count(), allocator);
        let node_parents = uninit_node_table(tree.node_count(), allocator);
        Self {
            scope: ScopeTracker::init(root, tree),
            symbols: SymbolTracker::init(tree),
            type_position_depth: 0,
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
        self.scope
            .ancestors(Some(self.scope.current))
            .any(|id| self.scope.get(id).kind == ScopeKind::TsModule)
    }

    #[inline]
    pub fn enter_node(&mut self, node: NodeId, parent: Option<NodeId>) -> ReferenceFlags {
        self.node_scopes[node.index()].write(self.scope.current);
        self.node_parents[node.index()].write(parent);
        let flags = self.reference_flags;
        self.reference_flags = self.default_reference_flags();
        flags
    }

    #[inline]
    pub fn default_reference_flags(&self) -> ReferenceFlags {
        ReferenceFlags {
            write: false,
            space: if self.in_type_position() {
                ReferenceSpace::Type
            } else {
                ReferenceSpace::Value
            },
            _reserved: 0,
        }
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
        assert!(self.type_position_depth > 0);
        self.type_position_depth -= 1;
        self.reference_flags = self.default_reference_flags();
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
        self.scope.enter_ts_module_block(node);
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
        // SAFETY: The visitor enters every AST node exactly once, initializing
        // both complete per-node tables before finalization.
        let node_scopes = unsafe { assume_init_node_table(self.node_scopes) };
        let node_parents = unsafe { assume_init_node_table(self.node_parents) };
        let scopes = self.scope.into_scope_tree();
        self.symbols
            .finalize(tree, scopes, node_scopes, node_parents)
    }
}

#[inline]
fn uninit_node_table<'a, T>(
    length: usize,
    allocator: &'a swc_next_allocator::Allocator,
) -> ArenaVec<'a, MaybeUninit<T>> {
    let mut table = ArenaVec::with_capacity_in(length, allocator);
    // SAFETY: `MaybeUninit` permits uninitialized elements. Traversal writes
    // every node's slot before it can be read.
    unsafe { table.set_len(length) };
    table
}

/// Reuses a fully initialized `MaybeUninit` allocation without copying it.
///
/// # Safety
///
/// Every element in `table` must contain a valid `T`.
#[inline(always)]
unsafe fn assume_init_node_table<'a, T>(
    mut table: ArenaVec<'a, MaybeUninit<T>>,
) -> ArenaVec<'a, T> {
    let allocator = table.bump();
    let length = table.len();
    let capacity = table.capacity();
    let pointer = table.as_mut_ptr().cast::<T>();
    // SAFETY: `MaybeUninit<T>` has the same layout as `T`; the caller promises
    // every element is initialized, all allocation metadata is unchanged, and
    // arena vectors have no destructor that could invalidate the allocation.
    unsafe { ArenaVec::from_raw_parts_in(pointer, length, capacity, allocator) }
}
