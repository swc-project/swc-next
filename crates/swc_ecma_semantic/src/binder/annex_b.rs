//! Annex B companion var bindings, finalized after every lexical declaration.

use super::*;

#[derive(Clone, Copy)]
pub(super) struct AnnexBFunction {
    pub(super) name: BindingIdentifier,
    pub(super) scope: ScopeId,
}

#[derive(Default)]
struct AnnexBScope<'a> {
    functions: Vec<usize>,
    blockers: Vec<&'a [u8]>,
    block_functions: HashMap<&'a [u8], usize, FxBuildHasher>,
}

impl<'a> SymbolTracker<'a> {
    pub(crate) fn bind_annex_b_functions(&mut self, tree: &Ast<'a>, scopes: &ScopeTree<'a>) {
        if self.annex_b_functions.is_empty() {
            return;
        }
        self.sync_scope_maps_from(scopes);

        let mut facts = HashMap::<_, AnnexBScope<'a>, FxBuildHasher>::default();
        for (index, function) in self.annex_b_functions.iter().enumerate() {
            facts
                .entry(function.scope)
                .or_default()
                .functions
                .push(index);
        }
        // Exported TypeScript members need not occur in scope_maps. Their
        // declaration occurrences still belong to the containing body, just
        // like private lexical declarations and parameters.
        for declaration in &self.declarations {
            let scope = scopes.get(declaration.scope);
            if annex_b_var_blocker(tree, scope, declaration.flags) {
                let name = self.symbols[declaration.symbol.index()].name;
                let name = tree.get_wtf8(name).as_bytes();
                let local = facts.entry(declaration.scope).or_default();
                local.blockers.push(name);
                if declaration.flags.contains(SymbolFlags::FUNCTION)
                    && !declaration
                        .flags
                        .contains(SymbolFlags::NON_ANNEX_B_FUNCTION)
                {
                    *local.block_functions.entry(name).or_default() += 1;
                }
            }
        }
        let mut eligible = vec![false; self.annex_b_functions.len()];

        // Walk the completed scope tree once. Active blockers are keyed by
        // hoist target so declarations in an outer function cannot suppress
        // a companion binding in a nested function.
        let mut children = vec![None; scopes.list.len()];
        let mut siblings = vec![None; scopes.list.len()];
        for (index, scope) in scopes.list.iter().enumerate().rev() {
            if let Some(parent) = scope.parent {
                siblings[index] = children[parent.index()];
                children[parent.index()] = Some(ScopeId::from_usize(index));
            }
        }
        let mut blocked = HashMap::<_, usize, FxBuildHasher>::default();
        let mut walk = vec![(ScopeId::ROOT, true)];
        while let Some((id, enter)) = walk.pop() {
            let scope = scopes.get(id);
            let local = facts.get(&id);
            for &name in local.into_iter().flat_map(|facts| &facts.blockers) {
                let key = (scope.hoist_target, name);
                if enter {
                    *blocked.entry(key).or_default() += 1;
                } else {
                    let count = blocked.get_mut(&key).expect("active lexical blocker");
                    *count -= 1;
                    if *count == 0 {
                        blocked.remove(&key);
                    }
                }
            }
            if !enter {
                continue;
            }
            if let Some(local) = local {
                for &index in &local.functions {
                    let name = self.annex_b_functions[index].name;
                    let text = tree.get_utf8(name.name(tree)).as_bytes();
                    let branch_local = self.symbol_of_node(name.node_id()).is_some_and(|symbol| {
                        self.symbols[symbol.index()]
                            .flags
                            .contains(SymbolFlags::ANNEX_B_IF_FUNCTION)
                    });
                    // Ordinary functions in this block may share companions.
                    // Ancestor blocks still block promotion; an if-clause's
                    // implicit block is nested inside this scope as well.
                    let same_block = if branch_local {
                        0
                    } else {
                        local.block_functions.get(text).copied().unwrap_or(0)
                    };
                    let blockers = blocked
                        .get(&(scope.hoist_target, text))
                        .copied()
                        .unwrap_or(0);
                    eligible[index] = blockers == same_block;
                }
            }
            walk.push((id, false));
            let mut child = children[id.index()];
            while let Some(id) = child {
                walk.push((id, true));
                child = siblings[id.index()];
            }
        }
        let explicit = self.declarations.len();
        for (index, eligible) in eligible.into_iter().enumerate() {
            if eligible {
                let function = self.annex_b_functions[index];
                let target = scopes.get(function.scope).hoist_target;
                self.bind_annex_b_var(tree, scopes, target, function.name);
            }
        }
        self.annex_b_functions.clear();
        self.order_annex_b_declarations(explicit);
    }

    fn bind_annex_b_var(
        &mut self,
        tree: &Ast<'a>,
        scopes: &ScopeTree<'a>,
        target: ScopeId,
        name: BindingIdentifier,
    ) {
        let text = tree.get_utf8(name.name(tree)).as_bytes();
        let scope = scopes.get(target);
        // Parameter expressions introduce a separate body var environment.
        // Annex B still excludes every name from the parameter environment.
        if scope.kind == ScopeKind::FunctionBody
            && let Some(parameter) = scope
                .parent
                .and_then(|parent| self.own_binding(parent, text))
            && self.symbols[parameter.index()]
                .flags
                .contains(SymbolFlags::PARAMETER)
        {
            return;
        }
        // ExportValue links identify the body's value binding even when its
        // ordinary scope map holds a same-named type-only declaration.
        let existing = self
            .ts_local_exports
            .get(&(target, text))
            .and_then(|local| local.exported_value())
            .or_else(|| self.own_binding(target, text))
            .filter(|&symbol| {
                !self.symbols[symbol.index()]
                    .flags
                    .intersects(SymbolFlags::INTERNAL_NAME)
            });
        // FunctionDeclarationInstantiation does not create a new arguments
        // binding. Its implicit arguments object remains outside our explicit
        // declaration model; an existing body var can still receive the value.
        if text == b"arguments" && scope.kind != ScopeKind::Global && existing.is_none() {
            return;
        }
        let declaration = PendingDeclaration {
            name: name.name(tree).into(),
            node: name.node_id(),
            binding: PendingBinding {
                flags: SymbolFlags::FUNCTION_SCOPED_VAR,
                excludes: Excludes::FUNCTION_SCOPED_VAR.difference(SymbolFlags::FUNCTION),
                scope: target,
            },
        };
        // The syntax node continues to identify the lexical function. The
        // companion declaration only contributes to the outer var symbol.
        let (symbol, _) = self.declare_symbol(tree, declaration, existing, false);
        if existing.is_none() {
            self.insert_scope_binding(tree, target, declaration.name, symbol);
        }
    }

    fn order_annex_b_declarations(&mut self, explicit: usize) {
        if explicit == self.declarations.len() {
            return;
        }
        // Each companion follows its lexical declaration in the public
        // declaration iterator. Merge once instead of inserting into the
        // arena vector for every function and shifting later declarations.
        let companions = self.declarations[explicit..]
            .iter()
            .enumerate()
            .map(|(index, declaration)| (declaration.node, explicit + index))
            .collect::<HashMap<_, _, FxBuildHasher>>();
        let mut ordered = Vec::with_capacity(self.declarations.len());
        let mut remap = vec![DeclarationId::from_usize(0); self.declarations.len()];
        for (index, &declaration) in self.declarations[..explicit].iter().enumerate() {
            remap[index] = DeclarationId::from_usize(ordered.len());
            ordered.push(declaration);
            if let Some(&companion) = companions.get(&declaration.node) {
                remap[companion] = DeclarationId::from_usize(ordered.len());
                ordered.push(self.declarations[companion]);
            }
        }
        debug_assert_eq!(ordered.len(), self.declarations.len());
        self.declarations.copy_from_slice(&ordered);
        for declarations in self.redeclarations.values_mut() {
            declarations.clear();
        }
        for (index, declaration) in self.declarations.iter().enumerate().rev() {
            self.symbols[declaration.symbol.index()].declaration = DeclarationId::from_usize(index);
        }
        for (index, declaration) in self.declarations.iter().enumerate() {
            let id = DeclarationId::from_usize(index);
            if self.symbols[declaration.symbol.index()].declaration != id {
                self.redeclarations
                    .entry(declaration.symbol)
                    .or_insert_with(|| ArenaVec::new_in(self.allocator))
                    .push(id);
            }
        }
        for map in self.hoisting_variables.iter_mut().flatten() {
            for declaration in map.values_mut() {
                *declaration = remap[declaration.index()];
            }
        }
    }
}

fn annex_b_var_blocker(tree: &Ast<'_>, scope: Scope, flags: SymbolFlags) -> bool {
    if flags.contains(SymbolFlags::CATCH_VAR) {
        let NodeKindData::CatchClause(catch) = tree.kind_data(scope.node) else {
            unreachable!("catch variables belong to a catch scope");
        };
        return catch.param(tree).is_some_and(|parameter| {
            tree.node_kind(parameter.node_id()) != NodeKind::BindingIdentifier
        });
    }
    flags.intersects(SymbolFlags::BLOCK_SCOPED_LIKE.difference(SymbolFlags::FUNCTION))
        || flags.contains(SymbolFlags::PARAMETER)
        || (!scope.kind.is_hoist_target() && flags.contains(SymbolFlags::FUNCTION))
}
