//! Syntax facts collected during semantic traversal for structural transforms.
//!
//! These facts describe the source tree only. A transform consumes the entry
//! for a rewritten owner and keeps scope, binding, and reference indexes in
//! sync through `ScopeModel` mutation APIs.

use rustc_hash::FxHashMap;
use swc_next_allocator::{Allocator, hash_map::HashMap as ArenaHashMap, vec::ArenaVec};
use swc_next_ecma_ast::{ArrowFunctionExpression, Ast, Function, NodeId};

use crate::{ReferenceSpace, Semantic};

/// Lexical-environment facts for one ordinary async function or async arrow.
#[derive(Debug, Clone, Copy)]
pub struct AsyncFunctionInfo {
    pub node: NodeId,
    pub uses_this: bool,
    pub uses_arguments: bool,
    pub has_parameter_expressions: bool,
}

/// Semantic facts shared by transforms.
#[derive(Debug)]
pub struct TransformInfo<'a> {
    async_functions: ArenaHashMap<'a, NodeId, AsyncFunctionInfo>,
    async_functions_post_order: ArenaVec<'a, AsyncFunctionInfo>,
}

impl<'a> TransformInfo<'a> {
    pub(crate) fn empty(allocator: &'a Allocator) -> Self {
        Self {
            async_functions: ArenaHashMap::new_in(allocator),
            async_functions_post_order: ArenaVec::new_in(allocator),
        }
    }

    /// Facts for `node`, when it is an ordinary async function or async arrow.
    pub fn async_function(&self, node: NodeId) -> Option<AsyncFunctionInfo> {
        self.async_functions.get(&node).copied()
    }

    /// Async owners in child-before-parent order.
    pub fn async_functions_post_order(&self) -> &[AsyncFunctionInfo] {
        &self.async_functions_post_order
    }

    pub(crate) fn take_async_function(&mut self, node: NodeId) -> Option<AsyncFunctionInfo> {
        let info = self.async_functions.remove(&node)?;
        self.async_functions_post_order
            .retain(|candidate| candidate.node != node);
        Some(info)
    }
}

#[derive(Default)]
pub(crate) struct TransformInfoBuilder {
    boundaries: Vec<Option<NodeId>>,
    async_functions: FxHashMap<NodeId, AsyncFunctionInfoBuilder>,
    async_functions_post_order: Vec<NodeId>,
}

#[derive(Default)]
struct AsyncFunctionInfoBuilder {
    lexical_parent: Option<NodeId>,
    uses_this: bool,
    argument_candidates: Vec<NodeId>,
    uses_arguments: bool,
    has_parameter_expressions: bool,
}

impl TransformInfoBuilder {
    pub(crate) fn enter_function(&mut self, function: Function, ast: &Ast<'_>) {
        let node = function.node_id();
        let transform = function.r#async(ast) && !function.generator(ast);
        let has_parameter_expressions = transform
            && crate::ecmascript::find_parameter_expression(ast, function.params(ast)).is_some();
        self.enter_boundary(node, transform, has_parameter_expressions);
    }

    pub(crate) fn leave_function(&mut self, function: Function, ast: &Ast<'_>) {
        let transform = function.r#async(ast) && !function.generator(ast);
        self.leave_boundary(function.node_id(), transform);
    }

    pub(crate) fn enter_arrow(&mut self, arrow: ArrowFunctionExpression, ast: &Ast<'_>) {
        if arrow.r#async(ast) {
            let has_parameter_expressions =
                crate::ecmascript::find_parameter_expression(ast, arrow.params(ast)).is_some();
            self.enter_boundary(arrow.node_id(), true, has_parameter_expressions);
        }
    }

    pub(crate) fn leave_arrow(&mut self, arrow: ArrowFunctionExpression, ast: &Ast<'_>) {
        if arrow.r#async(ast) {
            self.leave_boundary(arrow.node_id(), true);
        }
    }

    fn enter_boundary(&mut self, node: NodeId, transform: bool, has_parameter_expressions: bool) {
        let owner = transform.then_some(node);
        if transform {
            let lexical_parent = self.boundaries.last().copied().flatten();
            let previous = self.async_functions.insert(
                node,
                AsyncFunctionInfoBuilder {
                    lexical_parent,
                    has_parameter_expressions,
                    ..AsyncFunctionInfoBuilder::default()
                },
            );
            debug_assert!(previous.is_none());
        }
        self.boundaries.push(owner);
    }

    fn leave_boundary(&mut self, node: NodeId, transform: bool) {
        let expected = transform.then_some(node);
        debug_assert_eq!(self.boundaries.pop(), Some(expected));
        if transform {
            self.async_functions_post_order.push(node);
        }
    }

    pub(crate) fn record_this(&mut self) {
        let Some(owner) = self.boundaries.last().copied().flatten() else {
            return;
        };
        self.async_functions
            .get_mut(&owner)
            .expect("active async owner")
            .uses_this = true;
    }

    pub(crate) fn record_identifier_reference(
        &mut self,
        identifier: swc_next_ecma_ast::IdentifierReference,
        ast: &Ast<'_>,
    ) {
        let Some(owner) = self.boundaries.last().copied().flatten() else {
            return;
        };
        if ast.get_utf8(identifier.name(ast)) == "arguments" {
            self.async_functions
                .get_mut(&owner)
                .expect("active async owner")
                .argument_candidates
                .push(identifier.node_id());
        }
    }

    pub(crate) fn build<'a>(
        mut self,
        semantic: &Semantic<'a>,
        allocator: &'a Allocator,
    ) -> TransformInfo<'a> {
        debug_assert!(self.boundaries.is_empty());
        let mut async_functions = ArenaHashMap::new_in(allocator);
        let mut async_functions_post_order =
            ArenaVec::with_capacity_in(self.async_functions_post_order.len(), allocator);
        for node in self.async_functions_post_order {
            let mut builder = self
                .async_functions
                .remove(&node)
                .expect("recorded async owner");
            builder.uses_arguments |= builder.argument_candidates.iter().any(|candidate| {
                semantic.reference_of(*candidate).is_some_and(|reference| {
                    let reference = semantic.reference(reference);
                    reference.flags.space == ReferenceSpace::Value && reference.symbol.is_none()
                })
            });
            let info = AsyncFunctionInfo {
                node,
                uses_this: builder.uses_this,
                uses_arguments: builder.uses_arguments,
                has_parameter_expressions: builder.has_parameter_expressions,
            };
            if let Some(parent) = builder.lexical_parent {
                let parent = self
                    .async_functions
                    .get_mut(&parent)
                    .expect("lexical async parent");
                parent.uses_this |= info.uses_this;
                parent.uses_arguments |= info.uses_arguments;
            }
            async_functions.insert(node, info);
            async_functions_post_order.push(info);
        }
        debug_assert!(self.async_functions.is_empty());
        TransformInfo {
            async_functions,
            async_functions_post_order,
        }
    }
}
