use oxc_index::{Idx, IndexVec};
use swc_experimental_ecma_ast::{NodeId, ScopeId};

/// Kind of JavaScript or TypeScript construct that created a scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeKind {
    /// ECMAScript GlobalDeclarationInstantiation.
    Global,
    /// Module declaration environment. Modules are always strict.
    Module,
    /// Function declaration instantiation.
    Function,
    /// Block runtime semantics and BlockDeclarationInstantiation.
    Block,
    /// Class definition evaluation.
    Class,
    /// Class static block evaluation.
    StaticBlock,
    /// Intermediate environment containing a named function or class
    /// expression's name.
    ///
    /// This sits between the outer scope and the body scope so the expression
    /// name cannot conflict with a same-named binding inside the body.
    ExpressionName,
    /// TypeScript namespace body.
    ///
    /// This is a `var` hoist target so variables do not escape into the
    /// surrounding scope.
    TsModule,
}

impl ScopeKind {
    /// Returns whether `var` declarations hoist to this scope kind.
    pub fn is_hoist_target(self) -> bool {
        matches!(
            self,
            Self::Global | Self::Module | Self::Function | Self::StaticBlock | Self::TsModule
        )
    }
}

/// Scope-level flags.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScopeFlags {
    /// Whether this scope is in strict mode.
    pub strict: bool,
}

/// A single lexical scope in the scope tree.
///
/// ## Example
///
/// ```ts
/// function f() { { var a; let b; } }
/// //       ^ function scope and hoist target
/// //             ^ block scope where `b` lives; `a` hoists outward
/// ```
#[derive(Clone, Debug)]
pub struct Scope {
    /// AST node that created this scope.
    pub node: NodeId,
    /// Parent scope, or `None` for the root.
    pub parent: Option<ScopeId>,
    /// Nearest ancestor, or this scope itself, where `var` declarations hoist.
    pub hoist_target: ScopeId,
    /// First directly nested scope, if any.
    pub first_child: Option<ScopeId>,
    /// Next scope with the same parent, if any.
    pub next_sibling: Option<ScopeId>,
    /// Construct that created this scope.
    pub kind: ScopeKind,
    /// Scope-level semantic flags.
    pub flags: ScopeFlags,
    /// Cached strict-mode state.
    pub strict: bool,
    /// Whether this scope contains a direct call to `eval`.
    pub has_direct_eval: bool,
    /// Whether this scope contains a `with` statement.
    pub has_with: bool,
    /// Whether dynamic name lookup makes lexical renaming unsafe.
    pub rename_unsafe: bool,
}

/// All scopes for one source file, indexed by [`ScopeId`].
#[derive(Clone, Debug)]
pub struct ScopeTree {
    scopes: IndexVec<ScopeId, Scope>,
    root: ScopeId,
    top_level: ScopeId,
}

impl ScopeTree {
    pub(crate) fn new(root_node: NodeId, module: bool, script_strict: bool) -> Self {
        let mut scopes = IndexVec::new();
        let root: ScopeId = scopes.push(Scope {
            node: root_node,
            parent: None,
            hoist_target: ScopeId::from_usize(0),
            first_child: None,
            next_sibling: None,
            kind: ScopeKind::Global,
            flags: ScopeFlags {
                strict: script_strict,
            },
            strict: script_strict,
            has_direct_eval: false,
            has_with: false,
            rename_unsafe: false,
        });
        debug_assert_eq!(root.index(), 0);
        let top_level = if module {
            let module_scope = scopes.push(Scope {
                node: root_node,
                parent: Some(root),
                hoist_target: ScopeId::from_usize(1),
                first_child: None,
                next_sibling: None,
                kind: ScopeKind::Module,
                flags: ScopeFlags { strict: true },
                strict: true,
                has_direct_eval: false,
                has_with: false,
                rename_unsafe: false,
            });
            debug_assert_eq!(module_scope.index(), 1);
            module_scope
        } else {
            root
        };
        Self {
            scopes,
            root,
            top_level,
        }
    }

    /// Returns the global root scope.
    pub fn root_scope_id(&self) -> ScopeId {
        self.root
    }

    /// Returns the source file's top-level scope.
    ///
    /// This is the module scope for modules and the global scope for scripts.
    pub fn top_level_scope_id(&self) -> ScopeId {
        self.top_level
    }

    /// Returns the number of scopes.
    pub fn len(&self) -> usize {
        self.scopes.len()
    }

    /// Returns whether the scope tree is empty.
    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }

    /// Returns the scope with the given ID.
    pub fn get(&self, id: ScopeId) -> &Scope {
        &self.scopes[id]
    }

    pub(crate) fn get_mut(&mut self, id: ScopeId) -> &mut Scope {
        &mut self.scopes[id]
    }

    /// Iterates scopes in creation order.
    pub fn iter(&self) -> impl Iterator<Item = (ScopeId, &Scope)> {
        self.scopes.iter_enumerated()
    }

    /// Walks from `start` to the root, yielding `start` first.
    pub fn ancestors(&self, start: ScopeId) -> ScopeAncestors<'_> {
        ScopeAncestors {
            tree: self,
            next: Some(start),
        }
    }

    pub(crate) fn push(
        &mut self,
        parent: ScopeId,
        node: NodeId,
        kind: ScopeKind,
        flags: ScopeFlags,
    ) -> ScopeId {
        let next_id = ScopeId::from_usize(self.scopes.len());
        let hoist_target = if kind.is_hoist_target() {
            next_id
        } else {
            self.scopes[parent].hoist_target
        };
        let child = self.scopes.push(Scope {
            node,
            parent: Some(parent),
            hoist_target,
            first_child: None,
            next_sibling: None,
            kind,
            flags,
            strict: flags.strict,
            has_direct_eval: false,
            has_with: false,
            rename_unsafe: false,
        });
        if let Some(mut sibling) = self.scopes[parent].first_child {
            while let Some(next) = self.scopes[sibling].next_sibling {
                sibling = next;
            }
            self.scopes[sibling].next_sibling = Some(child);
        } else {
            self.scopes[parent].first_child = Some(child);
        }
        child
    }
}

/// Iterator over a scope and its ancestors.
pub struct ScopeAncestors<'tree> {
    tree: &'tree ScopeTree,
    next: Option<ScopeId>,
}

impl Iterator for ScopeAncestors<'_> {
    type Item = ScopeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        self.next = self.tree.get(current).parent;
        Some(current)
    }
}
