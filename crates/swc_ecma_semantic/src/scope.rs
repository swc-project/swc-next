use oxc_index::Idx;
use swc_next_allocator::{Allocator, vec::Vec as ArenaVec};
use swc_next_ecma_ast::{
    Ast, Class, ClassType, Function, FunctionBody, FunctionType, NodeId, NodeKindData, Program,
    ScopeId, SourceType,
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
}

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
    // scopes saved while a decorator subtree retargets `current`,
    // one entry per nested decorator. see `enter` on `Decorator`.
    decorator_saved: ArenaVec<'a, ScopeId>,
}

impl<'a> ScopeTracker<'a> {
    pub fn init(root: Program, tree: &Ast<'a>) -> Self {
        assert!(tree.node_count() > 0);

        let allocator = tree.allocator();
        let estimated_scopes = 16usize.max(tree.node_count() / 16);
        let mut tracker = Self {
            allocator,
            scopes: ArenaVec::with_capacity_in(estimated_scopes, allocator),
            current: ScopeId::ROOT,
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
                self.push_scope(ScopeKind::Module, root_node, ScopeFlags { strict: true });
            }
            SourceType::CommonJs => {
                self.push_scope(ScopeKind::Function, root_node, ScopeFlags::default());
            }
            SourceType::Script => {}
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

    pub fn enter(&mut self, tree: &Ast, index: NodeId, parent: Option<NodeId>) {
        match tree.kind_data(index) {
            NodeKindData::Directive(directive) => {
                if tree.string(directive.value(tree)) == b"use strict" {
                    self.current_mut().flags.strict = true;
                }
            }
            NodeKindData::Function(function) => {
                let flags = if self.has_retroactive_use_strict(tree, function.body(tree)) {
                    ScopeFlags { strict: true }
                } else {
                    self.inherit_strict_flag()
                };
                if is_named_function_expression(tree, function) {
                    // The expression-name scope sits between outer and body.
                    self.push_scope(ScopeKind::ExpressionName, index, flags);
                }
                self.push_scope(ScopeKind::Function, index, flags);
            }
            NodeKindData::TsFunction(_) => {
                self.push_scope(ScopeKind::Function, index, self.inherit_strict_flag());
            }
            NodeKindData::ArrowFunctionExpression(arrow) => {
                let body = arrow.body(tree).node_id();
                let flags = match tree.kind_data(body) {
                    NodeKindData::FunctionBody(body)
                        if self.has_retroactive_use_strict(tree, body) =>
                    {
                        ScopeFlags { strict: true }
                    }
                    _ => self.inherit_strict_flag(),
                };
                self.push_scope(ScopeKind::Function, index, flags);
            }
            NodeKindData::FunctionBody(_) => {
                // Pushed only when the parent function's parameter list
                // contains expressions; see ScopeKind::FunctionBody.
                let Some(parent) = parent else { return };
                let params = match tree.kind_data(parent) {
                    NodeKindData::Function(function) => function.params(tree),
                    NodeKindData::ArrowFunctionExpression(arrow) => arrow.params(tree),
                    _ => return,
                };
                if ecmascript::find_parameter_expression(tree, params).is_some() {
                    self.push_scope(ScopeKind::FunctionBody, index, self.inherit_strict_flag());
                }
            }
            NodeKindData::BlockStatement(_) => {
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
                        catch.body(tree).node_id() == index
                            && catch.param(tree).is_none_or(|param| {
                                ecmascript::find_pattern_expression(tree, Some(param.node_id()))
                                    .is_none()
                            })
                    });
                if !shares_catch_scope {
                    self.push_scope(ScopeKind::Block, index, self.inherit_strict_flag());
                }
            }
            NodeKindData::SwitchCase(_) => {
                // One case-block scope per switch, shared by every case and
                // created at the first case. Per 14.12.4, the discriminant
                // evaluates before this environment exists:
                //
                //   switch (x) { case 1: let x }
                //   //      ^ outer scope    ^ case-block scope
                if self.current_scope().node != parent.expect("a switch case has a parent") {
                    self.push_scope(
                        ScopeKind::Block,
                        parent.expect("a switch case has a parent"),
                        self.inherit_strict_flag(),
                    );
                }
            }
            NodeKindData::TsEnumBody(_) => {
                // TSC resolves unqualified member references inside the body.
                self.push_scope(ScopeKind::Block, index, self.inherit_strict_flag());
            }
            NodeKindData::ForStatement(_)
            | NodeKindData::ForInStatement(_)
            | NodeKindData::ForOfStatement(_)
            | NodeKindData::CatchClause(_)
            | NodeKindData::TsInterfaceDeclaration(_)
            | NodeKindData::TsTypeAliasDeclaration(_)
            | NodeKindData::TsFunctionType(_)
            | NodeKindData::TsConstructorType(_)
            | NodeKindData::TsMethodSignature(_)
            | NodeKindData::TsCallSignatureDeclaration(_)
            | NodeKindData::TsConstructSignatureDeclaration(_)
            | NodeKindData::TsIndexSignature(_)
            | NodeKindData::TsMappedType(_)
            | NodeKindData::TsConditionalType(_) => {
                self.push_scope(ScopeKind::Block, index, self.inherit_strict_flag());
            }
            NodeKindData::TsModuleBlock(_) => {
                self.push_scope(ScopeKind::TsModule, index, self.inherit_strict_flag());
            }
            NodeKindData::Class(class) => {
                // Classes are always strict mode (section 15.7.14).
                let flags = ScopeFlags { strict: true };
                if is_named_class_expression(tree, class) {
                    self.push_scope(ScopeKind::ExpressionName, index, flags);
                }
                self.push_scope(ScopeKind::Class, index, flags);
            }
            NodeKindData::StaticBlock(_) => {
                self.push_scope(ScopeKind::StaticBlock, index, self.inherit_strict_flag());
            }
            NodeKindData::Decorator(_) => {
                // Decorators evaluate in the scope enclosing the class:
                // neither its type parameters nor an expression's own name is
                // visible, so retarget `current` for the decorator subtree.
                self.decorator_saved.push(self.current);
                self.current = self.decorator_eval_scope();
            }
            _ => {}
        }
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
            if tree.string(directive.value(tree)) == b"use strict" {
                return true;
            }
        }
        false
    }

    #[inline]
    fn inherit_strict_flag(&self) -> ScopeFlags {
        ScopeFlags {
            strict: self.current_scope().flags.strict,
        }
    }

    // A node pops exactly the scopes recording it as creator. Two for a
    // named function or class expression, zero for a shared or never-created
    // scope. Cannot drift from `enter`.
    pub fn exit(&mut self, index: NodeId, data: NodeKindData) {
        match data {
            NodeKindData::Program(_) => {}
            NodeKindData::Decorator(_) => {
                // Every decorator enter saved exactly one scope.
                self.current = self
                    .decorator_saved
                    .pop()
                    .expect("every decorator enter saved exactly one scope");
            }
            _ => {
                while self.current.index() != 0 && self.current_scope().node == index {
                    self.pop_scope();
                }
            }
        }
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

    /// Walks from `start` up to the root, yielding `start` first.
    pub fn ancestors(&self, start: Option<ScopeId>) -> AncestorIterator<'_> {
        assert!(start.is_none_or(|id| id.index() < self.scopes.len()));
        AncestorIterator {
            list: &self.scopes,
            current: start,
        }
    }

    /// Finalizes into an immutable [`ScopeTree`].
    pub fn into_scope_tree(self) -> ScopeTree<'a> {
        ScopeTree { list: self.scopes }
    }
}

fn is_named_function_expression(tree: &Ast<'_>, function: Function) -> bool {
    function.function_type(tree) == FunctionType::FunctionExpression && function.id(tree).is_some()
}

fn is_named_class_expression(tree: &Ast<'_>, class: Class) -> bool {
    class.class_type(tree) != ClassType::ClassDeclaration && class.id(tree).is_some()
}
