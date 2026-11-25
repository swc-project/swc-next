use std::{borrow::Cow, iter, mem, ops::Range};

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
    used_symbols: ArenaVec<'ast, SymbolId>,
    // Positions of uses in this scope and its descendants.
    uses: Range<usize>,
    pub(in crate::common::rename) queue: ArenaVec<'ast, SymbolId>,
    pub(in crate::common::rename) queued: ArenaHashSet<'ast, SymbolId>,
    pub(in crate::common::rename) reserved: ArenaHashSet<'ast, &'ast str>,
    pub(in crate::common::rename) children: ArenaVec<'ast, Scope<'ast>>,
}

impl<'ast> Scope<'ast> {
    pub(in crate::common::rename) fn new(function: bool, allocator: &'ast Allocator) -> Self {
        Self {
            function,
            used_symbols: allocator.vec(),
            uses: 0..0,
            queue: allocator.vec(),
            queued: ArenaHashSet::new_in(allocator),
            reserved: ArenaHashSet::new_in(allocator),
            children: allocator.vec(),
        }
    }

    /// SWC declaration queue: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L52>.
    /// SWC var hoisting: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/mod.rs#L53>.
    pub(in crate::common::rename) fn add_decl(&mut self, id: SymbolId) {
        self.add_usage(id);
        if self.queued.insert(id) {
            self.queue.push(id);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L74>.
    pub(in crate::common::rename) fn add_usage(&mut self, id: SymbolId) {
        if self.used_symbols.last().copied() != Some(id) {
            self.used_symbols.push(id);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L91>.
    pub(in crate::common::rename) fn prepare_renaming(
        &mut self,
        symbols: &mut ScopeSymbolsBuilder<'ast>,
    ) {
        let start = symbols.add_scope(&self.used_symbols);
        for child in &mut self.children {
            child.prepare_renaming(symbols);
            self.reserved.extend(child.reserved.iter().copied());
        }
        self.uses = start..symbols.symbols.len();
    }

    /// SWC scope traversal: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L102>.
    /// SWC name assignment: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_base/src/rename/analyzer/scope.rs#L140>.
    pub(in crate::common::rename) fn rename_in_normal_mode(
        &self,
        ast: &Ast<'ast>,
        resolver: &Resolver,
        unresolved: &ArenaHashSet<'ast, &'ast str>,
        assignments: &mut Assignments<'ast>,
        symbols: &ScopeSymbols<'ast>,
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
                if assignments.conflicts(&name, &self.uses, symbols) {
                    continue;
                }
                assignments.insert(id, name);
                break;
            }
        }
        for child in &self.children {
            child.rename_in_normal_mode(ast, resolver, unresolved, assignments, symbols);
        }
    }
}

pub(in crate::common::rename) struct ScopeSymbolsBuilder<'ast> {
    symbols: ArenaVec<'ast, SymbolId>,
    // Each nonempty scope starts at a distinct offset in symbols.
    last_scope_start: ArenaIndexVec<'ast, SymbolId, usize>,
    counts: ArenaIndexVec<'ast, SymbolId, usize>,
}

impl<'ast> ScopeSymbolsBuilder<'ast> {
    pub(in crate::common::rename) fn new(symbol_count: usize, allocator: &'ast Allocator) -> Self {
        Self {
            symbols: ArenaVec::with_capacity_in(symbol_count, allocator),
            last_scope_start: ArenaIndexVec::from_iter_in(
                iter::repeat_n(usize::MAX, symbol_count),
                allocator,
            ),
            counts: ArenaIndexVec::from_iter_in(iter::repeat_n(0, symbol_count), allocator),
        }
    }

    fn add_scope(&mut self, symbols: &[SymbolId]) -> usize {
        let start = self.symbols.len();
        for &symbol in symbols {
            if self.last_scope_start[symbol] != start {
                self.last_scope_start[symbol] = start;
                self.counts[symbol] += 1;
                self.symbols.push(symbol);
            }
        }
        start
    }

    pub(in crate::common::rename) fn finish(self) -> ScopeSymbols<'ast> {
        let Self {
            symbols,
            counts: mut ends,
            ..
        } = self;
        // Turn counts into write offsets. Filling the index advances them to
        // the exclusive end of each symbol's sorted positions.
        let mut start = 0;
        for count in &mut ends {
            start += mem::replace(count, start);
        }
        let mut positions = ArenaVec::from_iter_in(iter::repeat_n(0, start), symbols.bump());
        for (position, &symbol) in symbols.iter().enumerate() {
            positions[ends[symbol]] = position;
            ends[symbol] += 1;
        }
        ScopeSymbols {
            symbols,
            ends,
            positions,
        }
    }
}

pub(in crate::common::rename) struct ScopeSymbols<'ast> {
    symbols: ArenaVec<'ast, SymbolId>,
    ends: ArenaIndexVec<'ast, SymbolId, usize>,
    positions: ArenaVec<'ast, usize>,
}

impl ScopeSymbols<'_> {
    fn contains(&self, scope: &Range<usize>, symbol: SymbolId) -> bool {
        let ends = self.ends.as_raw_slice();
        let start = if symbol.index() == 0 {
            0
        } else {
            ends[symbol.index() - 1]
        };
        let positions = &self.positions[start..self.ends[symbol]];
        let first = positions.partition_point(|&position| position < scope.start);
        positions
            .get(first)
            .is_some_and(|&position| position < scope.end)
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
            assigned: ArenaIndexVec::from_iter_in(iter::repeat_n(None, symbol_count), allocator),
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
    fn conflicts(&self, name: &str, scope: &Range<usize>, symbols: &ScopeSymbols<'ast>) -> bool {
        let Some(group) = self.reverse.get(name) else {
            return false;
        };
        // The symbol being assigned is absent from all groups.
        if group.len() > scope.len() {
            let first = Some(group[0]);
            symbols.symbols[scope.clone()]
                .iter()
                .any(|&other| self.assigned[other] == first)
        } else {
            group.iter().any(|&other| symbols.contains(scope, other))
        }
    }
}
