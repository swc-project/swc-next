use swc_next_allocator::vec::Vec as ArenaVec;
use swc_next_ecma_ast::{
    Ast, BindingIdentifier, NodeId, NodeKind, NodeKindData, Program, ScopeId, TraverseCtx,
};

use crate::{
    Semantic,
    binder::{RefContext, ReferenceSpace, SymbolTracker},
    scope::{ScopeKind, ScopeTracker},
};

pub(crate) type SemanticTraverseCtx<'traverse, 'ast> =
    TraverseCtx<'traverse, 'ast, SemanticContext<'ast>>;

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

    pub fn enter(&mut self, tree: &Ast, node: NodeId, parent: Option<NodeId>) {
        self.scope.enter(tree, node, parent);
        self.node_scopes[node.index()] = self.scope.current;
        self.node_parents[node.index()] = parent;
        if is_type_context(tree.node_kind(node)) {
            self.type_position_depth += 1;
        }
        self.symbols
            .set_binding_context(tree, node, parent, &self.scope);
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
        self.symbols.exit(kind);
        self.scope.exit(node, kind);
        if is_type_context(kind) {
            assert!(self.type_position_depth > 0);
            self.type_position_depth -= 1;
        }
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

fn is_type_context(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::TsTypeAnnotation
            | NodeKind::TsTypeReference
            | NodeKind::TsQualifiedName
            | NodeKind::TsTypeQuery
            | NodeKind::TsImportType
            | NodeKind::TsTypeParameter
            | NodeKind::TsTypeParameterDeclaration
            | NodeKind::TsTypeParameterInstantiation
            | NodeKind::TsLiteralType
            | NodeKind::TsTemplateLiteralType
            | NodeKind::TsArrayType
            | NodeKind::TsIndexedAccessType
            | NodeKind::TsTupleType
            | NodeKind::TsNamedTupleMember
            | NodeKind::TsOptionalType
            | NodeKind::TsRestType
            | NodeKind::TsJsDocNullableType
            | NodeKind::TsJsDocNonNullableType
            | NodeKind::TsJsDocUnknownType
            | NodeKind::TsUnionType
            | NodeKind::TsIntersectionType
            | NodeKind::TsConditionalType
            | NodeKind::TsInferType
            | NodeKind::TsTypeOperator
            | NodeKind::TsParenthesizedType
            | NodeKind::TsFunctionType
            | NodeKind::TsConstructorType
            | NodeKind::TsTypePredicate
            | NodeKind::TsTypeLiteral
            | NodeKind::TsPropertySignature
            | NodeKind::TsMethodSignature
            | NodeKind::TsCallSignatureDeclaration
            | NodeKind::TsConstructSignatureDeclaration
            | NodeKind::TsIndexSignature
            | NodeKind::TsMappedType
            | NodeKind::TsClassImplements
            | NodeKind::TsInterfaceHeritage
            | NodeKind::TsInterfaceBody
    )
}
