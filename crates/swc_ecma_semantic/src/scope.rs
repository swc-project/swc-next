use oxc_index::Idx;
use swc_next_allocator::{Allocator, vec::Vec as ArenaVec};
use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, Class, ClassType, Directive, Function, FunctionBody,
    FunctionType, NodeId, NodeKindData, ScopeId, SourceType,
};

use crate::ecmascript;

/// A single lexical scope in the scope tree.
///
/// ## Example
/// ```ts
/// function f() { { var a; let b; } }
/// //       ^ creates a function scope, a hoist target
/// //             ^ creates a block scope where `b` lives,
/// //               while `a` hoists to the function scope
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Scope {
    /// The AST node that created this scope.
    pub node: NodeId,
    /// Parent scope, or `None` for the root.
    pub parent: Option<ScopeId>,
    /// Nearest ancestor (or self) where `var` declarations hoist to.
    pub hoist_target: ScopeId,
    pub kind: ScopeKind,
    pub flags: ScopeFlags,
}

const _: () = assert!(std::mem::size_of::<Scope>() == 16);

/// What kind of JavaScript construct created a scope.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ScopeKind {
    /// Section 16.1.7 GlobalDeclarationInstantiation.
    Global,
    /// Section 16.2.1.6 ModuleDeclarationEnvironmentSetup.
    /// Modules are always strict.
    Module,
    /// Section 10.2.11 FunctionDeclarationInstantiation.
    /// Also used for the synthetic CommonJS top-level function wrapper.
    Function,
    /// Section 14.2.2 Block runtime semantics, 14.2.3 BlockDeclarationInstantiation.
    Block,
    /// Section 14.11 WithStatement evaluation. This object environment has
    /// no static bindings, but can intercept value-name resolution before
    /// the lexical scopes outside it are reached.
    With,
    /// Section 15.7.14 ClassDefinitionEvaluation.
    Class,
    /// Section 15.7.11 ClassStaticBlockDefinitionEvaluation.
    StaticBlock,
    /// Intermediate scope holding a named function/class expression's
    /// name (sections 15.2.5 and 15.7.14). The name binds in its own
    /// environment between the outer scope and the body, so it never
    /// conflicts with same-named bindings inside the body:
    ///
    /// ```text
    /// outer scope (x lives here)
    ///   expression-name scope (foo lives here)
    ///     function scope (body bindings live here)
    /// ```
    ExpressionName,
    /// TS namespace body. Acts as a var-hoist target so vars
    /// declared inside don't escape to the surrounding scope.
    TsModule,
    /// Separate var environment of a function body whose parameter
    /// list contains expressions (10.2.11 FunctionDeclarationInstantiation
    /// step 28):
    ///
    /// ```ts
    /// function f(a = () => x) { var x }
    /// //       ^ function scope: a
    /// //                        ^ function-body scope: x,
    /// //                          invisible to the default's closure
    /// ```
    FunctionBody,
}

impl ScopeKind {
    /// Returns whether `var` declarations hoist to this scope kind.
    pub fn is_hoist_target(self) -> bool {
        matches!(
            self,
            Self::Global
                | Self::Module
                | Self::Function
                | Self::StaticBlock
                | Self::TsModule
                | Self::FunctionBody
        )
    }
}

/// Scope-level flags.
#[derive(Debug, Default, Clone, Copy)]
pub struct ScopeFlags {
    /// Whether this scope is in strict mode.
    pub strict: bool,
    /// Whether this scope or one of its descendants contains a call that can
    /// invoke the intrinsic eval function directly.
    pub direct_eval: bool,
}

impl ScopeFlags {
    /// True when direct eval can observe bindings owned by this scope.
    pub fn contains_direct_eval(self) -> bool {
        self.direct_eval
    }
}

const _: () = assert!(std::mem::size_of::<ScopeFlags>() == 2);

/// All scopes of a tree, indexed by [`ScopeId`].
#[derive(Debug)]
pub struct ScopeTree<'a> {
    pub(crate) list: ArenaVec<'a, Scope>,
}

impl<'a> ScopeTree<'a> {
    /// The scope with the given id.
    #[inline]
    pub fn get(&self, id: ScopeId) -> Scope {
        assert!(id.index() < self.list.len());
        self.list[id.index()]
    }

    /// Walks from `start` up to the root, yielding `start` first.
    pub fn ancestors(&self, start: Option<ScopeId>) -> AncestorIterator<'_> {
        assert!(start.is_none_or(|id| id.index() < self.list.len()));
        AncestorIterator {
            list: &self.list,
            current: start,
        }
    }

    /// Marks `start` and every lexical ancestor as observable by direct eval.
    pub(crate) fn mark_direct_eval(&mut self, start: ScopeId) {
        let mut current = Some(start);
        // Each scope is marked at most once. Later calls stop at the first
        // marked ancestor, so all calls together are O(scopes + eval calls).
        while let Some(id) = current {
            let scope = &mut self.list[id.index()];
            if scope.flags.direct_eval {
                break;
            }
            scope.flags.direct_eval = true;
            current = scope.parent;
        }
    }
}

/// Yields each scope id from a starting scope up to the root.
pub struct AncestorIterator<'a> {
    list: &'a [Scope],
    current: Option<ScopeId>,
}

impl Iterator for AncestorIterator<'_> {
    type Item = ScopeId;

    /// The next scope id, or `None` past the root.
    fn next(&mut self) -> Option<Self::Item> {
        let id = self.current?;
        assert!(id.index() < self.list.len());
        self.current = self.list[id.index()].parent;
        Some(id)
    }
}

pub(crate) struct ScopeTracker<'a> {
    allocator: &'a Allocator,
    pub(crate) scopes: ArenaVec<'a, Scope>,
    // active scope. follow the parent chain to walk the path.
    pub(crate) current: ScopeId,
    // Whether finalization needs to account for a with object environment.
    // This is transient builder state, not part of the finalized scope tree.
    has_with_statement: bool,
    // Active `with` scopes, used only while walking their statement bodies.
    // The heap allocation is therefore paid only by files containing `with`.
    active_with_scopes: std::vec::Vec<ScopeId>,
    // scopes saved while a decorator subtree retargets `current`,
    // one entry per nested decorator. see `enter` on `Decorator`.
    decorator_saved: ArenaVec<'a, ScopeId>,
}

impl<'a> ScopeTracker<'a> {
    pub fn init(tree: &Ast<'a>) -> Self {
        assert!(tree.node_count() > 0);

        let root = tree.root_program();
        let allocator = tree.allocator();
        let estimated_scopes = 16usize.max(tree.node_count() / 16);
        let mut tracker = Self {
            allocator,
            scopes: ArenaVec::with_capacity_in(estimated_scopes, allocator),
            current: ScopeId::ROOT,
            has_with_statement: false,
            active_with_scopes: std::vec::Vec::new(),
            decorator_saved: ArenaVec::new_in(allocator),
        };
        tracker.push_root(root.node_id(), tree.source_type());
        tracker
    }

    fn push_root(&mut self, root_node: NodeId, source_type: SourceType) {
        assert!(self.scopes.is_empty());
        self.scopes.push(Scope {
            node: root_node,
            parent: None,
            hoist_target: ScopeId::ROOT,
            kind: ScopeKind::Global,
            flags: ScopeFlags::default(),
        });
        self.current = ScopeId::ROOT;

        match source_type {
            SourceType::Module => {
                self.push_scope(
                    ScopeKind::Module,
                    root_node,
                    ScopeFlags {
                        strict: true,
                        ..ScopeFlags::default()
                    },
                );
            }
            SourceType::CommonJs => {
                self.push_scope(ScopeKind::Function, root_node, ScopeFlags::default());
            }
            SourceType::Script => {}
            SourceType::Unambiguous => {
                unreachable!("unambiguous source type must be resolved before semantic analysis")
            }
        }
    }

    pub fn push_scope(&mut self, kind: ScopeKind, node: NodeId, flags: ScopeFlags) {
        let _allocator = self.allocator;
        // The current length is the about-to-be-assigned id and must fit u32.
        assert!(self.scopes.len() < u32::MAX as usize);
        let id = ScopeId::from_usize(self.scopes.len());
        let parent = self.current_scope();
        self.scopes.push(Scope {
            node,
            parent: Some(self.current),
            hoist_target: if kind.is_hoist_target() {
                id
            } else {
                parent.hoist_target
            },
            kind,
            flags,
        });
        self.current = id;
    }

    fn pop_scope(&mut self) {
        // Root and its source-type wrapper are never popped directly by the ordinary walk.
        assert_ne!(self.current.index(), 0);
        self.current = self
            .current_scope()
            .parent
            .expect("non-root scopes have a parent");
    }

    pub fn enter_directive(&mut self, tree: &Ast, directive: Directive) {
        if tree.get_utf8(directive.value(tree)) == "use strict" {
            self.current_mut().flags.strict = true;
        }
    }

    pub fn enter_function(&mut self, tree: &Ast, function: Function, annex_b_if_body: bool) {
        let flags = if self.has_retroactive_use_strict(tree, function.body(tree)) {
            ScopeFlags {
                strict: true,
                ..ScopeFlags::default()
            }
        } else {
            self.inherit_strict_flag()
        };
        // Annex B.3.3 permits a FunctionDeclaration directly in a sloppy
        // if/else clause. Its lexical binding is distinct from bindings in
        // the enclosing scope, just as if the clause had an implicit block.
        // Use the function node as the scope creator so `exit_scope` removes
        // this scope together with the function scope below.
        if annex_b_if_body && !flags.strict {
            self.push_scope(ScopeKind::Block, function.node_id(), flags);
        }
        if is_named_function_expression(tree, function) {
            // The expression-name scope sits between outer and body.
            self.push_scope(ScopeKind::ExpressionName, function.node_id(), flags);
        }
        self.push_scope(ScopeKind::Function, function.node_id(), flags);
    }

    pub fn enter_ts_function(&mut self, node: NodeId) {
        self.push_scope(ScopeKind::Function, node, self.inherit_strict_flag());
    }

    pub fn enter_arrow_function_expression(&mut self, tree: &Ast, arrow: ArrowFunctionExpression) {
        let body = arrow.body(tree).node_id();
        let flags = match tree.kind_data(body) {
            NodeKindData::FunctionBody(body) if self.has_retroactive_use_strict(tree, body) => {
                ScopeFlags {
                    strict: true,
                    ..ScopeFlags::default()
                }
            }
            _ => self.inherit_strict_flag(),
        };
        self.push_scope(ScopeKind::Function, arrow.node_id(), flags);
    }

    pub fn enter_function_body(&mut self, tree: &Ast, body: FunctionBody, parent: Option<NodeId>) {
        // Pushed only when the parent function's parameter list
        // contains expressions; see ScopeKind::FunctionBody.
        let Some(parent) = parent else { return };
        let params = match tree.kind_data(parent) {
            NodeKindData::Function(function) => function.params(tree),
            NodeKindData::ArrowFunctionExpression(arrow) => arrow.params(tree),
            _ => return,
        };
        if ecmascript::find_parameter_expression(tree, params).is_some() {
            self.push_scope(
                ScopeKind::FunctionBody,
                body.node_id(),
                self.inherit_strict_flag(),
            );
        }
    }

    pub fn enter_block_statement(&mut self, tree: &Ast, node: NodeId) {
        // 14.15.2 gives catch parameters and the block separate
        // environments. Folding them is unobservable and makes the
        // 14.15.1 conflict a single-scope lookup, except when a
        // parameter expression runs before the block environment:
        //
        //   catch (e)       { let e }   // shared, conflict found
        //   catch ([e = 1]) { let e }   // split; default cannot see let
        let catch = match tree.kind_data(self.current_scope().node) {
            NodeKindData::CatchClause(catch) => Some(catch),
            _ => None,
        };
        let shares_catch_scope = self.current_scope().kind == ScopeKind::Block
            && catch.is_some_and(|catch| {
                catch.body(tree).node_id() == node
                    && catch.param(tree).is_none_or(|param| {
                        ecmascript::find_pattern_expression(tree, Some(param.node_id())).is_none()
                    })
            });
        if !shares_catch_scope {
            self.enter_block_scope(node);
        }
    }

    pub fn enter_switch_case(&mut self, parent: Option<NodeId>) {
        // One case-block scope per switch, shared by every case and
        // created at the first case. Per 14.12.4, the discriminant
        // evaluates before this environment exists:
        //
        //   switch (x) { case 1: let x }
        //   //      ^ outer scope    ^ case-block scope
        let parent = parent.expect("a switch case has a parent");
        if self.current_scope().node != parent {
            self.enter_block_scope(parent);
        }
    }

    pub fn enter_ts_enum_body(&mut self, node: NodeId) {
        // TSC resolves unqualified member references inside the body.
        self.enter_block_scope(node);
    }

    pub fn enter_block_scope(&mut self, node: NodeId) {
        self.push_scope(ScopeKind::Block, node, self.inherit_strict_flag());
    }

    pub fn enter_with_scope(&mut self, node: NodeId) {
        self.has_with_statement = true;
        self.push_scope(ScopeKind::With, node, self.inherit_strict_flag());
        self.active_with_scopes.push(self.current);
    }

    pub fn exit_with_scope(&mut self, node: NodeId) {
        let scope = self.current_scope();
        assert_eq!(scope.node, node);
        assert_eq!(scope.kind, ScopeKind::With);
        assert_eq!(self.active_with_scopes.pop(), Some(self.current));
        self.pop_scope();
    }

    pub fn enter_ts_module_block(&mut self, node: NodeId) {
        self.push_scope(ScopeKind::TsModule, node, self.inherit_strict_flag());
    }

    pub fn enter_class(&mut self, tree: &Ast, class: Class) {
        // Classes are always strict mode (section 15.7.14).
        let flags = ScopeFlags {
            strict: true,
            ..ScopeFlags::default()
        };
        if is_named_class_expression(tree, class) {
            self.push_scope(ScopeKind::ExpressionName, class.node_id(), flags);
        }
        self.push_scope(ScopeKind::Class, class.node_id(), flags);
    }

    pub fn enter_static_block(&mut self, node: NodeId) {
        self.push_scope(ScopeKind::StaticBlock, node, self.inherit_strict_flag());
    }

    pub fn enter_decorator(&mut self) {
        // Decorators evaluate in the scope enclosing the class:
        // neither its type parameters nor an expression's own name is
        // visible, so retarget `current` for the decorator subtree.
        self.decorator_saved.push(self.current);
        self.current = self.decorator_eval_scope();
    }

    // the parent of the nearest class scope, past a named class
    // expression's name scope
    fn decorator_eval_scope(&self) -> ScopeId {
        for id in self.ancestors(Some(self.current)) {
            if self.get(id).kind != ScopeKind::Class {
                continue;
            }
            let Some(parent) = self.get(id).parent else {
                break;
            };
            if self.get(parent).kind == ScopeKind::ExpressionName {
                return self.get(parent).parent.unwrap_or(parent);
            }
            return parent;
        }
        self.current
    }

    // "use strict" applies retroactively to the whole function including
    // its parameter list, so peek into the body and set the flag at
    // scope-creation time, before any parameter is visited
    fn has_retroactive_use_strict(&self, tree: &Ast, body: FunctionBody) -> bool {
        for directive in body.directives(tree).iter() {
            let directive = tree.get_node_in_sub_range(directive);
            if tree.get_utf8(directive.value(tree)) == "use strict" {
                return true;
            }
        }
        false
    }

    #[inline]
    fn inherit_strict_flag(&self) -> ScopeFlags {
        ScopeFlags {
            strict: self.current_scope().flags.strict,
            ..ScopeFlags::default()
        }
    }

    // A node pops exactly the scopes recording it as creator. Two for a
    // named function or class expression, zero for a shared or never-created
    // scope. Cannot drift from `enter`.
    pub fn exit_scope(&mut self, node: NodeId) {
        while self.current.index() != 0 && self.current_scope().node == node {
            self.pop_scope();
        }
    }

    pub fn exit_decorator(&mut self) {
        // Every decorator enter saved exactly one scope.
        self.current = self
            .decorator_saved
            .pop()
            .expect("every decorator enter saved exactly one scope");
    }

    /// The current scope.
    #[inline]
    pub fn current_scope(&self) -> Scope {
        self.get(self.current)
    }

    #[inline]
    // Mutable view used for the tracker's strict-mode bookkeeping.
    fn current_mut(&mut self) -> &mut Scope {
        &mut self.scopes[self.current.index()]
    }

    /// The scope where a `var` declared here would land.
    #[inline]
    pub fn hoist_target(&self) -> ScopeId {
        self.current_scope().hoist_target
    }

    /// The scope with the given id.
    #[inline]
    pub fn get(&self, id: ScopeId) -> Scope {
        assert!(id.index() < self.scopes.len());
        self.scopes[id.index()]
    }

    /// Whether the current scope is in strict mode.
    #[inline]
    pub fn is_strict(&self) -> bool {
        self.current_scope().flags.strict
    }

    /// Whether resolving a `var` initializer from the current scope crosses
    /// the nearest active `with` object environment before reaching its
    /// hoist target.
    #[inline]
    pub fn var_write_crosses_with(&self) -> bool {
        let Some(&with_scope) = self.active_with_scopes.last() else {
            return false;
        };
        let hoist_target = self.hoist_target();
        // Both ids are on the active ancestor chain. Scopes are allocated on
        // entry, so an outer hoist target precedes the intervening with scope.
        debug_assert!(
            self.ancestors(Some(self.current))
                .any(|scope| scope == with_scope)
        );
        hoist_target.index() < with_scope.index()
    }

    /// Walks from `start` up to the root, yielding `start` first.
    pub fn ancestors(&self, start: Option<ScopeId>) -> AncestorIterator<'_> {
        assert!(start.is_none_or(|id| id.index() < self.scopes.len()));
        AncestorIterator {
            list: &self.scopes,
            current: start,
        }
    }

    /// Finalizes into an immutable [`ScopeTree`].
    pub fn into_scope_tree(self) -> (ScopeTree<'a>, bool) {
        assert!(self.active_with_scopes.is_empty());
        (ScopeTree { list: self.scopes }, self.has_with_statement)
    }
}

fn is_named_function_expression(tree: &Ast<'_>, function: Function) -> bool {
    function.function_type(tree) == FunctionType::FunctionExpression && function.id(tree).is_some()
}

fn is_named_class_expression(tree: &Ast<'_>, class: Class) -> bool {
    class.class_type(tree) != ClassType::ClassDeclaration && class.id(tree).is_some()
}
