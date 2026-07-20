use oxc_index::{Idx, IndexVec};
use swc_experimental_ecma_ast::{NodeId, ScopeId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeKind {
    Global,
    Module,
    Function,
    Block,
    Class,
    StaticBlock,
    ExpressionName,
    TsModule,
}

impl ScopeKind {
    pub fn is_hoist_target(self) -> bool {
        matches!(
            self,
            Self::Global | Self::Module | Self::Function | Self::StaticBlock | Self::TsModule
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScopeFlags {
    pub strict: bool,
}

#[derive(Clone, Debug)]
pub struct Scope {
    pub node: NodeId,
    pub parent: Option<ScopeId>,
    pub hoist_target: ScopeId,
    pub first_child: Option<ScopeId>,
    pub next_sibling: Option<ScopeId>,
    pub kind: ScopeKind,
    pub flags: ScopeFlags,
    pub strict: bool,
    pub has_direct_eval: bool,
    pub has_with: bool,
    pub rename_unsafe: bool,
}

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

    pub fn root_scope_id(&self) -> ScopeId {
        self.root
    }

    pub fn top_level_scope_id(&self) -> ScopeId {
        self.top_level
    }

    pub fn len(&self) -> usize {
        self.scopes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }

    pub fn get(&self, id: ScopeId) -> &Scope {
        &self.scopes[id]
    }

    pub(crate) fn get_mut(&mut self, id: ScopeId) -> &mut Scope {
        &mut self.scopes[id]
    }

    pub fn iter(&self) -> impl Iterator<Item = (ScopeId, &Scope)> {
        self.scopes.iter_enumerated()
    }

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
