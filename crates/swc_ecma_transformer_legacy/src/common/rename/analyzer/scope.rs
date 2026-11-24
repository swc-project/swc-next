use std::borrow::Cow;

use rustc_hash::FxHashMap;
use swc_next_allocator::{
    Allocator,
    hash_map::HashMap as ArenaHashMap,
    hash_set::HashSet as ArenaHashSet,
    vec::{ArenaIndexVec, ArenaVec},
};
use swc_next_ecma_ast::{Ast, SymbolId};

use crate::common::resolver::context::Resolver;

pub(in crate::common::rename) struct Scope<'ast> {
    pub(in crate::common::rename) function: bool,
    pub(in crate::common::rename) all: ArenaHashSet<'ast, SymbolId>,
    pub(in crate::common::rename) queue: ArenaVec<'ast, SymbolId>,
    pub(in crate::common::rename) queued: ArenaHashSet<'ast, SymbolId>,
    pub(in crate::common::rename) reserved: ArenaHashSet<'ast, &'ast str>,
    pub(in crate::common::rename) children: ArenaVec<'ast, Scope<'ast>>,
}

impl<'ast> Scope<'ast> {
    pub(in crate::common::rename) fn new(function: bool, allocator: &'ast Allocator) -> Self {
        Self {
            function,
            all: ArenaHashSet::new_in(allocator),
            queue: allocator.vec(),
            queued: ArenaHashSet::new_in(allocator),
            reserved: ArenaHashSet::new_in(allocator),
            children: allocator.vec(),
        }
    }

    /// SWC declaration queue: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L52>.
    /// SWC var hoisting: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/mod.rs#L53>.
    pub(in crate::common::rename) fn add_decl(&mut self, id: SymbolId) {
        self.all.insert(id);
        if self.queued.insert(id) {
            self.queue.push(id);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L74>.
    pub(in crate::common::rename) fn add_usage(&mut self, id: SymbolId) {
        self.all.insert(id);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L91>.
    pub(in crate::common::rename) fn prepare_renaming(&mut self) {
        for child in &mut self.children {
            child.prepare_renaming();
            self.all.extend(child.all.iter().copied());
            self.reserved.extend(child.reserved.iter().copied());
        }
    }

    /// SWC scope traversal: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L102>.
    /// SWC name assignment: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L140>.
    pub(in crate::common::rename) fn rename_in_normal_mode(
        &self,
        ast: &Ast<'ast>,
        resolver: &Resolver,
        unresolved: &ArenaHashSet<'ast, &'ast str>,
        assignments: &mut Assignments<'ast>,
    ) {
        let mut latest = ArenaHashMap::<&str, usize>::new_in(ast.allocator());
        for &id in &self.queue {
            let original = ast.get_utf8(resolver.symbol_name(id));
            if original == "eval" || assignments.assigned[id].is_some() {
                continue;
            }
            let index = latest.entry(original).or_default();
            loop {
                let name = if *index == 0 {
                    Cow::Borrowed(original)
                } else {
                    Cow::Owned(format!("{original}{index}"))
                };
                *index += 1;
                if unresolved.contains(name.as_ref()) || self.reserved.contains(name.as_ref()) {
                    continue;
                }
                if assignments.conflicts(&name, id, &self.all) {
                    continue;
                }
                assignments.insert(id, name);
                break;
            }
        }
        for child in &self.children {
            child.rename_in_normal_mode(ast, resolver, unresolved, assignments);
        }
    }
}

pub(in crate::common::rename) struct Assignments<'ast> {
    reverse: FxHashMap<Cow<'ast, str>, ArenaVec<'ast, SymbolId>>,
    // The first symbol assigned to a name identifies that group in both indexes.
    assigned: ArenaIndexVec<'ast, SymbolId, Option<SymbolId>>,
    pub(in crate::common::rename) renames: FxHashMap<SymbolId, String>,
}

impl<'ast> Assignments<'ast> {
    pub(in crate::common::rename) fn new(symbol_count: usize, allocator: &'ast Allocator) -> Self {
        Self {
            reverse: FxHashMap::default(),
            assigned: ArenaIndexVec::from_iter_in(
                std::iter::repeat_n(None, symbol_count),
                allocator,
            ),
            renames: FxHashMap::default(),
        }
    }

    fn insert(&mut self, id: SymbolId, name: Cow<'ast, str>) {
        if let Cow::Owned(name) = &name {
            self.renames.insert(id, name.clone());
        }
        let allocator = self.assigned.bump();
        let group = self.reverse.entry(name).or_insert_with(|| allocator.vec());
        self.assigned[id] = Some(group.first().copied().unwrap_or(id));
        group.push(id);
    }

    /// SWC conflict check: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L199>.
    fn conflicts(&self, name: &str, id: SymbolId, all: &ArenaHashSet<'ast, SymbolId>) -> bool {
        let Some(group) = self.reverse.get(name) else {
            return false;
        };
        // The set intersection can be checked from either index.
        if group.len() > all.len() {
            let first = Some(group[0]);
            all.iter()
                .any(|&other| other != id && self.assigned[other] == first)
        } else {
            group
                .iter()
                .any(|&other| other != id && all.contains(&other))
        }
    }
}
