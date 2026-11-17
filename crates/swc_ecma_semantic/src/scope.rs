use swc_next_allocator::vec::ArenaIndexVec;
use swc_next_ecma_ast::{Ast, NodeId, ScopeId, SourceType};

use crate::Scopes;

/// A lexical environment or a TypeScript name-lookup view in the scope tree.
/// Lookup views restrict visible declarations without introducing a runtime
/// environment; see the corresponding [`ScopeKind`] variants.
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
    /// Signature lookup can inspect it without exposing it to initializers.
    pub function_body: Option<ScopeId>,
    pub kind: ScopeKind,
    pub flags: ScopeFlags,
}

/// The lexical environment or name-lookup view represented by a scope.
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
    /// Lookup view for a class expression's decorators. Retains the class's
    /// name and type parameters, but inherits strictness from outside the class.
    ClassDecorator,
    /// Section 15.7.11 ClassStaticBlockDefinitionEvaluation.
    StaticBlock,
    /// TS namespace body. Acts as a var-hoist target so vars
    /// declared inside don't escape to the surrounding scope.
    TsModule,
    /// Type-parameter-list lookup view. The owning declaration contributes
    /// type parameters; its parameter variables and body-local types are hidden.
    TsTypeParameters,
    /// Return-annotation lookup view. The owning declaration contributes
    /// parameters and type parameters. Body functions and let/const variables
    /// can contribute value meaning, but body var declarations and local types
    /// (including classes and enums queried through `typeof`) remain hidden.
    TsSignatureReturn,
    /// Parameter-region lookup view. Type queries exclude body-local variables
    /// and types, but can see body functions. Runtime expressions retain the
    /// owning function's JavaScript parameter environment.
    TsSignatureParameter,
    /// Extends-clause lookup view. Infer declarations belong to the conditional
    /// scope but are only visible from its true branch.
    TsConditionalExtends,
    /// Local visibility of an infer parameter inside its own constraint.
    /// Its declaration identity still belongs to the conditional scope.
    TsInferConstraint,
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
    pub(crate) fn is_ts_lookup_view(self) -> bool {
        matches!(
            self,
            Self::TsTypeParameters
                | Self::TsSignatureReturn
                | Self::TsSignatureParameter
                | Self::TsConditionalExtends
                | Self::TsInferConstraint
        )
    }

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
