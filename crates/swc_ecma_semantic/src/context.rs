use swc_next_allocator::vec::Vec as ArenaVec;
use swc_next_ecma_ast::{Ast, AstNode, NodeId, NodeKind, ScopeId, TraverseCtx};

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
    pub fn init(tree: &Ast<'a>) -> Self {
        let allocator = tree.allocator();
        let mut node_scopes = ArenaVec::with_capacity_in(tree.node_count(), allocator);
        let mut node_parents = ArenaVec::with_capacity_in(tree.node_count(), allocator);
        for _ in 0..tree.node_count() {
            node_scopes.push(ScopeId::ROOT);
            node_parents.push(None);
        }
        Self {
            scope: ScopeTracker::init(tree),
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

    pub fn enter(&mut self, tree: &Ast, node: NodeId, kind: NodeKind, parent: Option<NodeId>) {
        self.scope.enter(tree, node, parent, kind);
        self.node_scopes[node.index()] = self.scope.current;
        self.node_parents[node.index()] = parent;
        if is_type_context(kind) {
            self.type_position_depth += 1;
        }
        self.symbols
            .set_binding_context(tree, node, kind, parent, &self.scope);
    }

    pub fn post_enter(
        &mut self,
        tree: &Ast,
        node: NodeId,
        kind: NodeKind,
        reference_context: RefContext,
    ) {
        self.symbols
            .declare_bindings(tree, node, kind, &self.scope, reference_context);
    }

    pub fn post_enter_binding(&mut self, tree: &Ast, node: NodeId, space: ReferenceSpace) {
        self.post_enter(
            tree,
            node,
            NodeKind::BindingIdentifier,
            RefContext {
                is_write: false,
                space,
            },
        );
    }

    pub fn exit(&mut self, node: NodeId, kind: NodeKind) {
        self.symbols.exit(kind);
        self.scope.exit(node, kind);
        if is_type_context(kind) {
            assert!(self.type_position_depth > 0);
            self.type_position_depth -= 1;
        }
    }

    pub fn finalize(self, tree: &Ast) -> Semantic<'a> {
        let node_scopes = self.node_scopes.into_bump_slice();
        let node_parents = self.node_parents.into_bump_slice();
        let scopes = self.scope.to_scope_tree();
        self.symbols
            .finalize(tree, scopes, node_scopes, node_parents)
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
        match tree.node_kind(parent) {
            NodeKind::TsQualifiedName => {
                qualified_left = true;
                child = parent;
            }
            NodeKind::TsPropertySignature => {
                let signature = unsafe {
                    swc_next_ecma_ast::TsPropertySignature::from_node_id_unchecked(parent)
                };
                if signature.computed(tree) && signature.key(tree).node_id() == child {
                    return ReferenceSpace::Typeof;
                }
                return type_reference_space(in_type_position, qualified_left);
            }
            NodeKind::TsMethodSignature => {
                let signature =
                    unsafe { swc_next_ecma_ast::TsMethodSignature::from_node_id_unchecked(parent) };
                if signature.computed(tree) && signature.key(tree).node_id() == child {
                    return ReferenceSpace::Typeof;
                }
                return type_reference_space(in_type_position, qualified_left);
            }
            NodeKind::TsTypeQuery => return ReferenceSpace::Typeof,
            NodeKind::ExportSpecifier => {
                let specifier =
                    unsafe { swc_next_ecma_ast::ExportSpecifier::from_node_id_unchecked(parent) };
                return if specifier.local(tree).node_id() == child {
                    ReferenceSpace::Any
                } else {
                    ReferenceSpace::Value
                };
            }
            NodeKind::ExportDefaultDeclaration
            | NodeKind::TsExportAssignment
            | NodeKind::TsImportEqualsDeclaration => return ReferenceSpace::Any,
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
        match tree.node_kind(parent) {
            NodeKind::AssignmentExpression => {
                let assignment = unsafe {
                    swc_next_ecma_ast::AssignmentExpression::from_node_id_unchecked(parent)
                };
                return assignment.left(tree).node_id() == child;
            }
            NodeKind::UpdateExpression => {
                let update =
                    unsafe { swc_next_ecma_ast::UpdateExpression::from_node_id_unchecked(parent) };
                return update.argument(tree).node_id() == child;
            }
            NodeKind::ForInStatement => {
                let statement =
                    unsafe { swc_next_ecma_ast::ForInStatement::from_node_id_unchecked(parent) };
                return statement.left(tree).node_id() == child;
            }
            NodeKind::ForOfStatement => {
                let statement =
                    unsafe { swc_next_ecma_ast::ForOfStatement::from_node_id_unchecked(parent) };
                return statement.left(tree).node_id() == child;
            }
            NodeKind::ArrayPattern
            | NodeKind::ObjectPattern
            | NodeKind::ParenthesizedExpression
            | NodeKind::TsNonNullExpression => child = parent,
            NodeKind::BindingProperty => {
                let property =
                    unsafe { swc_next_ecma_ast::BindingProperty::from_node_id_unchecked(parent) };
                if property.value(tree).node_id() != child {
                    return false;
                }
                child = parent;
            }
            NodeKind::AssignmentPattern => {
                let pattern =
                    unsafe { swc_next_ecma_ast::AssignmentPattern::from_node_id_unchecked(parent) };
                if pattern.left(tree).node_id() != child {
                    return false;
                }
                child = parent;
            }
            NodeKind::BindingRestElement => {
                let rest = unsafe {
                    swc_next_ecma_ast::BindingRestElement::from_node_id_unchecked(parent)
                };
                if rest.argument(tree).node_id() != child {
                    return false;
                }
                child = parent;
            }
            NodeKind::TsAsExpression => {
                let expression =
                    unsafe { swc_next_ecma_ast::TsAsExpression::from_node_id_unchecked(parent) };
                if expression.expression(tree).node_id() != child {
                    return false;
                }
                child = parent;
            }
            NodeKind::TsSatisfiesExpression => {
                let expression = unsafe {
                    swc_next_ecma_ast::TsSatisfiesExpression::from_node_id_unchecked(parent)
                };
                if expression.expression(tree).node_id() != child {
                    return false;
                }
                child = parent;
            }
            NodeKind::TsTypeAssertion => {
                let expression =
                    unsafe { swc_next_ecma_ast::TsTypeAssertion::from_node_id_unchecked(parent) };
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
