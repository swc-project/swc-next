use swc_next_allocator::vec::Vec as ArenaVec;
use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, BindingIdentifier, Class, Directive, Function, FunctionBody,
    NodeId, NodeKindData, Program, ScopeId,
};

use crate::{
    Semantic,
    binder::{RefContext, ReferenceSpace, SymbolTracker},
    scope::{ScopeKind, ScopeTracker},
};

pub(crate) struct SemanticVisitCtx<'traverse, 'ast> {
    pub state: SemanticContext<'ast>,
    pub ast: &'traverse Ast<'ast>,
    path: Vec<NodeId>,
}

impl<'traverse, 'ast> SemanticVisitCtx<'traverse, 'ast> {
    pub fn new(ast: &'traverse Ast<'ast>, state: SemanticContext<'ast>) -> Self {
        Self {
            state,
            ast,
            path: Vec::new(),
        }
    }

    pub fn enter_node(&mut self, node: NodeId) {
        let parent = self.path.last().copied();
        self.path.push(node);
        self.state.enter_node(node, parent);
    }

    pub fn leave_node(&mut self, node: NodeId) {
        self.state.exit(self.ast, node);
        let actual = self.path.pop();
        debug_assert_eq!(actual, Some(node));
    }

    pub fn path(&self) -> &[NodeId] {
        &self.path
    }

    pub fn parent(&self) -> Option<NodeId> {
        self.path.iter().rev().nth(1).copied()
    }

    pub fn current_node(&self) -> Option<NodeId> {
        self.path.last().copied()
    }

    pub fn ancestors(&self) -> impl DoubleEndedIterator<Item = NodeId> + '_ {
        self.path.iter().rev().copied()
    }

    pub fn finalize(self) -> Semantic<'ast> {
        self.state.finalize(self.ast)
    }
}

/// Walk context combining the path stack, scope tracker, and symbol
/// tracker. Visitor hooks use it to inspect surrounding source structure
/// or to query and contribute symbols.
pub(crate) struct SemanticContext<'a> {
    pub scope: ScopeTracker<'a>,
    pub symbols: SymbolTracker<'a>,
    // depth of TS type-only context. inspect via `in_type_position()`.
    type_position_depth: u32,
    // per-node tables carried into the final `Semantic`
    node_scopes: ArenaVec<'a, ScopeId>,
    node_parents: ArenaVec<'a, Option<NodeId>>,
}

impl<'a> SemanticContext<'a> {
    pub fn init(root: Program, tree: &Ast<'a>) -> Self {
        let allocator = tree.allocator();
        let mut node_scopes = ArenaVec::with_capacity_in(tree.node_count(), allocator);
        let mut node_parents = ArenaVec::with_capacity_in(tree.node_count(), allocator);
        for _ in 0..tree.node_count() {
            node_scopes.push(ScopeId::ROOT);
            node_parents.push(None);
        }
        Self {
            scope: ScopeTracker::init(root, tree),
            symbols: SymbolTracker::init(tree),
            type_position_depth: 0,
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

    pub fn enter_node(&mut self, node: NodeId, parent: Option<NodeId>) {
        self.node_scopes[node.index()] = self.scope.current;
        self.node_parents[node.index()] = parent;
    }

    #[inline]
    pub fn enter_type_position(&mut self) {
        self.type_position_depth += 1;
    }

    #[inline]
    pub fn exit_type_position(&mut self) {
        assert!(self.type_position_depth > 0);
        self.type_position_depth -= 1;
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

    pub fn post_enter(&mut self, tree: &Ast, data: NodeKindData, reference_context: RefContext) {
        self.symbols
            .declare_bindings(tree, data, &self.scope, reference_context);
    }

    pub fn post_enter_binding(
        &mut self,
        tree: &Ast,
        identifier: BindingIdentifier,
        space: ReferenceSpace,
    ) {
        self.post_enter(
            tree,
            NodeKindData::BindingIdentifier(identifier),
            RefContext {
                is_write: false,
                space,
            },
        );
    }

    pub fn exit(&mut self, tree: &Ast, node: NodeId) {
        let kind = tree.node_kind(node);
        self.scope.exit(node, kind);
    }

    pub fn finalize(self, tree: &Ast) -> Semantic<'a> {
        let scopes = self.scope.into_scope_tree();
        self.symbols
            .finalize(tree, scopes, self.node_scopes, self.node_parents)
    }
}

/// The declaration space an identifier position resolves in.
pub(crate) fn reference_space(
    tree: &Ast<'_>,
    path: &[NodeId],
    in_type_position: bool,
) -> ReferenceSpace {
    assert!(!path.is_empty());
    let mut qualified_left = false;
    let mut child = *path.last().expect("the current node is in the path");
    for &parent in path[..path.len() - 1].iter().rev() {
        match tree.kind_data(parent) {
            NodeKindData::TsQualifiedName(_) => {
                qualified_left = true;
                child = parent;
            }
            NodeKindData::TsPropertySignature(signature) => {
                if signature.computed(tree) && signature.key(tree).node_id() == child {
                    return ReferenceSpace::Typeof;
                }
                return type_reference_space(in_type_position, qualified_left);
            }
            NodeKindData::TsMethodSignature(signature) => {
                if signature.computed(tree) && signature.key(tree).node_id() == child {
                    return ReferenceSpace::Typeof;
                }
                return type_reference_space(in_type_position, qualified_left);
            }
            NodeKindData::TsTypeQuery(_) => return ReferenceSpace::Typeof,
            NodeKindData::ExportSpecifier(specifier) => {
                return if specifier.local(tree).node_id() == child {
                    ReferenceSpace::Any
                } else {
                    ReferenceSpace::Value
                };
            }
            NodeKindData::ExportDefaultDeclaration(_)
            | NodeKindData::TsExportAssignment(_)
            | NodeKindData::TsImportEqualsDeclaration(_) => return ReferenceSpace::Any,
            _ => return type_reference_space(in_type_position, qualified_left),
        }
    }
    ReferenceSpace::Value
}

fn type_reference_space(in_type_position: bool, qualified_left: bool) -> ReferenceSpace {
    if !in_type_position {
        ReferenceSpace::Value
    } else if qualified_left {
        ReferenceSpace::Namespace
    } else {
        ReferenceSpace::Type
    }
}

/// <https://tc39.es/ecma262/#sec-static-semantics-assignmenttargettype>
pub(crate) fn is_write_target(tree: &Ast<'_>, path: &[NodeId]) -> bool {
    assert!(!path.is_empty());
    let mut child = *path.last().expect("the current node is in the path");
    for &parent in path[..path.len() - 1].iter().rev() {
        assert_ne!(parent, child);
        match tree.kind_data(parent) {
            NodeKindData::AssignmentExpression(assignment) => {
                return assignment.left(tree).node_id() == child;
            }
            NodeKindData::UpdateExpression(update) => {
                return update.argument(tree).node_id() == child;
            }
            NodeKindData::ForInStatement(statement) => {
                return statement.left(tree).node_id() == child;
            }
            NodeKindData::ForOfStatement(statement) => {
                return statement.left(tree).node_id() == child;
            }
            NodeKindData::ArrayPattern(_)
            | NodeKindData::ObjectPattern(_)
            | NodeKindData::ParenthesizedExpression(_)
            | NodeKindData::TsNonNullExpression(_) => child = parent,
            NodeKindData::BindingProperty(property) => {
                if property.value(tree).node_id() != child {
                    return false;
                }
                child = parent;
            }
            NodeKindData::AssignmentPattern(pattern) => {
                if pattern.left(tree).node_id() != child {
                    return false;
                }
                child = parent;
            }
            NodeKindData::BindingRestElement(rest) => {
                if rest.argument(tree).node_id() != child {
                    return false;
                }
                child = parent;
            }
            NodeKindData::TsAsExpression(expression) => {
                if expression.expression(tree).node_id() != child {
                    return false;
                }
                child = parent;
            }
            NodeKindData::TsSatisfiesExpression(expression) => {
                if expression.expression(tree).node_id() != child {
                    return false;
                }
                child = parent;
            }
            NodeKindData::TsTypeAssertion(expression) => {
                if expression.expression(tree).node_id() != child {
                    return false;
                }
                child = parent;
            }
            _ => return false,
        }
    }
    false
}
