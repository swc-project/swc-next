use swc_next_allocator::vec::ArenaIndexVec;
use swc_next_ecma_ast::{Ast, NodeId, ScopeId, SourceType};

use crate::Scopes;

/// A lexical environment or decorator evaluation context in the scope tree.
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
    /// False when a transform has removed this scope.
    pub active: bool,
    /// The AST node that created this scope.
    pub node: NodeId,
    /// Parent scope, or `None` for the root.
    pub parent: Option<ScopeId>,
    /// Nearest ancestor (or self) where `var` declarations hoist to.
    pub hoist_target: ScopeId,
    /// Separate body environment when this function has parameter expressions.
    pub function_body: Option<ScopeId>,
    pub kind: ScopeKind,
    pub flags: ScopeFlags,
}

/// The lexical environment represented by a scope.
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
    /// no static bindings.
    With,
    /// Section 15.7.14 ClassDefinitionEvaluation.
    Class,
    /// A class expression's decorators use its name environment and inherit
    /// strictness from the enclosing code.
    ClassDecorator,
    /// Section 15.7.11 ClassStaticBlockDefinitionEvaluation.
    StaticBlock,
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
    /// Stops lookup of an outer `arguments` binding: ordinary functions provide
    /// their own implicit binding, while static blocks disallow it entirely.
    pub arguments_barrier: bool,
}

/// All scopes of a tree, indexed by [`ScopeId`].
#[derive(Debug)]
pub struct ScopeTree<'a> {
    pub(crate) list: ArenaIndexVec<'a, ScopeId, Scope>,
}

impl<'a> ScopeTree<'a> {
    pub(crate) fn init(tree: &Ast<'a>) -> Self {
        let root = tree.root_program();
        let root_strict = root.directives(tree).iter().any(|directive| {
            let directive = tree.get_node_in_sub_range(directive);
            tree.get_utf8(directive.value(tree)) == "use strict"
        });
        let allocator = tree.allocator();
        let estimated_scopes = 16usize.max(tree.node_count() / 16);
        let mut scopes = Self {
            list: ArenaIndexVec::with_capacity_in(estimated_scopes, allocator),
        };
        scopes.push_root(root.node_id(), tree.source_type(), root_strict);
        scopes
    }

    fn push_root(&mut self, root_node: NodeId, source_type: SourceType, root_strict: bool) {
        debug_assert!(self.list.is_empty());
        self.list.push(Scope {
            active: true,
            node: root_node,
            parent: None,
            hoist_target: ScopeId::ROOT,
            function_body: None,
            kind: ScopeKind::Global,
            flags: ScopeFlags {
                strict: source_type == SourceType::Script && root_strict,
                ..ScopeFlags::default()
            },
        });

        match source_type {
            SourceType::Module => {
                self.push_scope(
                    ScopeId::ROOT,
                    ScopeKind::Module,
                    root_node,
                    ScopeFlags {
                        strict: true,
                        ..ScopeFlags::default()
                    },
                );
            }
            SourceType::CommonJs => {
                self.push_scope(
                    ScopeId::ROOT,
                    ScopeKind::Function,
                    root_node,
                    ScopeFlags {
                        strict: root_strict,
                        arguments_barrier: true,
                    },
                );
            }
            SourceType::Script => {}
            SourceType::Unambiguous => {
                unreachable!("unambiguous source type must be resolved before semantic analysis")
            }
        }
    }

    pub(crate) fn push_scope(
        &mut self,
        parent: ScopeId,
        kind: ScopeKind,
        node: NodeId,
        flags: ScopeFlags,
    ) -> ScopeId {
        // The current length is the about-to-be-assigned id and must fit u32.
        debug_assert!(self.list.len() < u32::MAX as usize);
        let id = self.list.next_idx();
        let hoist_target = self.get(parent).hoist_target;
        self.list.push(Scope {
            active: true,
            node,
            parent: Some(parent),
            hoist_target: if kind.is_hoist_target() {
                id
            } else {
                hoist_target
            },
            function_body: None,
            kind,
            flags,
        });
        id
    }

    /// The scope with the given id.
    #[inline]
    pub fn get(&self, id: ScopeId) -> Scope {
        self.list[id]
    }

    /// Walks from `start` up to the root, yielding `start` first.
    pub fn ancestors(&self, start: Option<ScopeId>) -> AncestorIterator<'_> {
        AncestorIterator {
            list: &self.list,
            current: start,
        }
    }
}

/// Yields each scope id from a starting scope up to the root.
pub struct AncestorIterator<'a> {
    list: &'a Scopes,
    current: Option<ScopeId>,
}

impl Iterator for AncestorIterator<'_> {
    type Item = ScopeId;

    /// The next scope id, or `None` past the root.
    fn next(&mut self) -> Option<Self::Item> {
        let id = self.current?;
        self.current = self.list[id].parent;
        Some(id)
    }
}
