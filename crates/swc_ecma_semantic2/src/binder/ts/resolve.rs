use std::{convert::Infallible, hash::BuildHasher};

use swc_next_allocator::IndexSlice;

use super::*;
use crate::{
    Scopes, SymbolTables, Symbols,
    ts::{TsImportAlias, TsImportAliasSpace as Spaces},
};

#[derive(Clone, Copy)]
struct Binding {
    symbol: SymbolId,
    flags: SymbolFlags,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Visibility {
    Absent,
    Possible,
    Definite,
}

#[derive(Clone, Copy, Default)]
struct AliasMeaning {
    definite: Spaces,
    possible: Spaces,
    owner: Option<SymbolId>,
    type_only: bool,
}

impl AliasMeaning {
    fn direct(binding: Binding) -> Self {
        let flags = binding.flags;
        let mut definite = Spaces::empty();
        definite.set(
            Spaces::VALUE,
            flags.contains(SymbolFlags::EXPORT_VALUE) || flags.visible_in(ReferenceSpace::Value),
        );
        definite.set(
            Spaces::TYPEOF,
            flags.contains(SymbolFlags::EXPORT_VALUE) || flags.visible_in(ReferenceSpace::Typeof),
        );
        definite.set(Spaces::TYPE, flags.visible_in(ReferenceSpace::Type));
        definite.set(
            Spaces::NAMESPACE,
            flags.visible_in(ReferenceSpace::Namespace),
        );
        Self {
            definite,
            owner: flags.in_namespace_space().then_some(binding.symbol),
            type_only: flags.contains(SymbolFlags::TYPE_IMPORT),
            ..Self::default()
        }
    }

    fn restrict_value(&mut self, type_only: bool) {
        if type_only {
            self.definite.remove(Spaces::VALUE);
            self.possible.remove(Spaces::VALUE);
            self.type_only = true;
        }
    }

    fn spaces(self) -> Spaces {
        self.definite | self.possible
    }

    fn visible_in(self, space: ReferenceSpace) -> Visibility {
        let spaces = match space {
            ReferenceSpace::Value => Spaces::VALUE,
            ReferenceSpace::Type => Spaces::TYPE,
            ReferenceSpace::Namespace => Spaces::NAMESPACE,
            ReferenceSpace::Typeof => Spaces::TYPEOF,
            ReferenceSpace::Any => Spaces::all(),
        };
        if self.definite.intersects(spaces) {
            Visibility::Definite
        } else if self.possible.intersects(spaces) {
            Visibility::Possible
        } else {
            Visibility::Absent
        }
    }
}

impl Binding {
    fn meaning(self, alias: Option<AliasMeaning>) -> AliasMeaning {
        let Some(alias) = alias.filter(|_| self.flags.intersects(SymbolFlags::ANY_IMPORT)) else {
            return AliasMeaning::direct(self);
        };
        let mut direct = AliasMeaning::direct(Self {
            flags: self.flags - SymbolFlags::ANY_IMPORT,
            ..self
        });
        direct.definite |= alias.definite;
        direct.possible = alias.possible;
        if direct.owner.is_none() {
            direct.owner = alias.owner;
            direct.type_only = alias.type_only;
        }
        direct
    }
}

/// The lexical table and the TypeScript contributions visible from each scope.
pub(crate) struct LexicalLookup<'s, 'a> {
    pub(crate) scopes: &'s Scopes,
    pub(crate) symbols: &'s Symbols,
    pub(crate) symbol_tables: &'s SymbolTables<'a>,
    pub(crate) shadowed_internal_names: &'s ShadowedInternalNames<'a>,
    pub(crate) ts: &'s TsBindings<'a>,
}

impl LexicalLookup<'_, '_> {
    pub(crate) fn lookup(
        &self,
        scope: ScopeId,
        name: &[u8],
        space: ReferenceSpace,
    ) -> Option<SymbolId> {
        let result = self.lookup_with(scope, name, space, &mut Vec::new(), |binding| {
            Ok::<_, Infallible>(visibility(binding, None, space))
        });
        result.unwrap().map(|binding| binding.symbol)
    }

    fn binding(&self, symbol: SymbolId) -> Binding {
        let mut flags = self.symbols[symbol].flags;
        if flags.contains(SymbolFlags::EXPORT_VALUE)
            && !self
                .ts
                .export_symbols
                .get(&symbol)
                .is_some_and(|&export| self.symbols[export].flags.in_value_space())
        {
            flags.remove(SymbolFlags::EXPORT_VALUE);
        }
        Binding { symbol, flags }
    }

    fn export(&self, local: SymbolId) -> Option<Binding> {
        let &symbol = self.ts.export_symbols.get(&local)?;
        Some(self.binding(symbol))
    }

    fn public_binding(
        &self,
        scope: ScopeId,
        name: &[u8],
        local: Option<SymbolId>,
    ) -> Option<Binding> {
        let kind = self.scopes[scope].kind;
        match kind {
            ScopeKind::Module => self.export(local?),
            ScopeKind::TsEnum | ScopeKind::TsModule => {
                let owner = self.ts.ts_owner(scope)?;
                let member = self.ts.member(owner, name)?;
                let flags = if kind == ScopeKind::TsEnum {
                    member.enum_flags?
                } else {
                    member.namespace_flags?
                };
                Some(Binding {
                    symbol: member.symbol,
                    flags,
                })
            }
            _ => None,
        }
    }

    fn lookup_with<E>(
        &self,
        scope: ScopeId,
        name: &[u8],
        space: ReferenceSpace,
        candidates: &mut Vec<Binding>,
        mut visibility: impl FnMut(Binding) -> Result<Visibility, E>,
    ) -> Result<Option<Binding>, E> {
        let hash = self.symbol_tables[ScopeId::ROOT].hasher().hash_one(name);
        let is_arguments =
            name == b"arguments" && matches!(space, ReferenceSpace::Value | ReferenceSpace::Typeof);
        let mut ancestor = Some(scope);
        while let Some(current) = ancestor {
            let scope = &self.scopes[current];
            let barrier = is_arguments && scope.flags.arguments_barrier;
            let entry = self.symbol_tables[current]
                .raw_entry()
                .from_hash(hash, |candidate| *candidate == name);
            let local = entry.map(|(_, &symbol)| symbol);
            let mut hidden = local;
            while let Some(symbol) = hidden {
                let binding = self.binding(symbol);
                let shadowed = barrier
                    && binding.flags.intersects(
                        SymbolFlags::FUNCTION_EXPRESSION | SymbolFlags::ANNEX_B_IF_FUNCTION,
                    );
                if !shadowed && add_binding(binding, visibility(binding)?, candidates) {
                    return Ok(finish_lookup(binding, candidates));
                }
                hidden = self.shadowed_internal_names.get(&symbol).copied();
            }

            if let Some(binding) = self.public_binding(current, name, local)
                && add_binding(binding, visibility(binding)?, candidates)
            {
                return Ok(finish_lookup(binding, candidates));
            }
            if barrier {
                break;
            }
            ancestor = scope.parent;
        }
        Ok(None)
    }
}

fn visibility(binding: Binding, alias: Option<AliasMeaning>, space: ReferenceSpace) -> Visibility {
    if alias.is_none() {
        if binding.flags.visible_in(space)
            || (binding.flags.contains(SymbolFlags::EXPORT_VALUE)
                && matches!(space, ReferenceSpace::Value | ReferenceSpace::Typeof))
        {
            return Visibility::Definite;
        }
        return Visibility::Absent;
    }
    binding.meaning(alias).visible_in(space)
}

fn insert_candidate(binding: Binding, candidates: &mut Vec<Binding>) {
    if let Some(candidate) = candidates.iter_mut().find(|c| c.symbol == binding.symbol) {
        candidate.flags |= binding.flags;
    } else {
        candidates.push(binding);
    }
}

fn add_binding(binding: Binding, visibility: Visibility, candidates: &mut Vec<Binding>) -> bool {
    match visibility {
        Visibility::Absent => false,
        Visibility::Possible => {
            insert_candidate(binding, candidates);
            false
        }
        Visibility::Definite => true,
    }
}

fn finish_lookup(binding: Binding, candidates: &mut Vec<Binding>) -> Option<Binding> {
    if candidates.is_empty() {
        return Some(binding);
    }
    insert_candidate(binding, candidates);
    None
}

struct ResolvedAlias {
    target: TsImportAlias,
    meaning: AliasMeaning,
    root: Option<Binding>,
    candidates: Vec<Binding>,
}

impl ResolvedAlias {
    fn visibility(&self, binding: Binding, space: ReferenceSpace) -> Visibility {
        if space == ReferenceSpace::Any
            && matches!(self.target, TsImportAlias::Target(_))
            && !self.meaning.spaces().is_empty()
        {
            return Visibility::Definite;
        }
        visibility(binding, Some(self.meaning), space)
    }

    fn new(
        target: Option<SymbolId>,
        meaning: AliasMeaning,
        root: Option<Binding>,
        candidates: Vec<Binding>,
    ) -> Self {
        let mut meaning = meaning;
        let target = match target {
            Some(symbol) => TsImportAlias::Target(symbol),
            None => {
                meaning.possible |= meaning.definite;
                meaning.definite = Spaces::empty();
                TsImportAlias::Unknown(meaning.possible)
            }
        };
        Self {
            target,
            meaning,
            root,
            candidates,
        }
    }
}

/// An unresolved dependency suspends an alias while the dependency is resolved.
/// This keeps forward chains and cycles off the Rust call stack.
struct AliasResolver<'s, 'a> {
    lookup: LexicalLookup<'s, 'a>,
    tree: &'s Ast<'a>,
    references: &'s IndexSlice<ReferenceId, [Reference]>,
    aliases: &'s FxHashMap<SymbolId, ImportEquals>,
    active: FxHashSet<SymbolId>,
    resolved: FxHashMap<SymbolId, ResolvedAlias>,
}

impl AliasResolver<'_, '_> {
    fn resolve_all(
        mut self,
        initializers: &FxHashMap<ReferenceId, SymbolId>,
    ) -> FxHashMap<SymbolId, ResolvedAlias> {
        let mut stack = Vec::new();
        for (id, _) in self.references.iter_enumerated() {
            let Some(&symbol) = initializers.get(&id) else {
                continue;
            };
            if self.resolved.contains_key(&symbol) {
                continue;
            }
            self.active.insert(symbol);
            stack.push(symbol);
            while let Some(&symbol) = stack.last() {
                match self.resolve(symbol) {
                    Ok(alias) => {
                        self.resolved.insert(symbol, alias);
                        self.active.remove(&symbol);
                        stack.pop();
                    }
                    Err(dependency) => {
                        self.active.insert(dependency);
                        stack.push(dependency);
                    }
                }
            }
        }
        self.resolved
    }

    fn alias_meaning(&self, binding: Binding) -> Result<Option<AliasMeaning>, SymbolId> {
        let symbol = binding.symbol;
        if !binding.flags.intersects(SymbolFlags::ANY_IMPORT) || !self.aliases.contains_key(&symbol)
        {
            return Ok(None);
        }
        if let Some(alias) = self.resolved.get(&symbol) {
            return Ok(Some(alias.meaning));
        }
        if self.active.contains(&symbol) {
            return Ok(Some(AliasMeaning::default()));
        }
        Err(symbol)
    }

    fn target_meaning(&self, binding: Binding) -> Result<AliasMeaning, SymbolId> {
        let alias = self.alias_meaning(binding)?;
        let mut meaning = binding.meaning(alias);
        if alias.is_none() && binding.flags.intersects(SymbolFlags::ANY_IMPORT) {
            // A direct import is its own lexical binding. The meaning of its
            // external target is only possible when used through another alias.
            let direct = Binding {
                flags: binding.flags - SymbolFlags::ANY_IMPORT,
                ..binding
            };
            meaning.possible = meaning.definite;
            meaning.definite = AliasMeaning::direct(direct).definite;
        }
        if let Some(export) = self.lookup.export(binding.symbol) {
            let export = export.meaning(self.alias_meaning(export)?);
            meaning.definite |= export.definite;
            meaning.possible |= export.possible;
            if meaning.owner.is_none() {
                meaning.owner = export.owner;
                meaning.type_only |= export.type_only;
            }
        }
        Ok(meaning)
    }

    fn resolve(&self, symbol: SymbolId) -> Result<ResolvedAlias, SymbolId> {
        let alias = &self.aliases[&symbol];
        let root = self.references[alias.root];
        let name = self.tree.get_utf8(root.name).as_bytes();
        let mut candidates = Vec::new();
        let lookup = |space, candidates: &mut Vec<_>| {
            self.lookup
                .lookup_with(root.scope, name, space, candidates, |binding| {
                    let meaning = self.alias_meaning(binding)?;
                    let visibility = match self.resolved.get(&binding.symbol) {
                        Some(alias) => alias.visibility(binding, space),
                        None => visibility(binding, meaning, space),
                    };
                    Ok::<_, SymbolId>(visibility)
                })
        };
        let mut binding = lookup(ReferenceSpace::Namespace, &mut candidates)?;
        if binding.is_none() && candidates.is_empty() && alias.members.is_empty() {
            binding = lookup(ReferenceSpace::Any, &mut candidates)?;
        }
        let type_only = self.lookup.symbols[symbol]
            .flags
            .contains(SymbolFlags::TYPE_IMPORT);
        if let Some(binding) = binding {
            let (target, mut meaning) = self.resolve_path(binding, &alias.members)?;
            meaning.restrict_value(type_only);
            return Ok(ResolvedAlias::new(
                target,
                meaning,
                Some(binding),
                candidates,
            ));
        }

        let mut meaning = AliasMeaning::default();
        for &binding in &candidates {
            let (_, path) = self.resolve_path(binding, &alias.members)?;
            let spaces = path.spaces();
            if spaces.is_empty() {
                continue;
            }
            let previous = meaning.spaces();
            if previous.is_empty() {
                meaning = path;
                continue;
            }
            if spaces.contains(Spaces::NAMESPACE) {
                if !previous.contains(Spaces::NAMESPACE) {
                    meaning.owner = path.owner;
                } else if meaning.owner != path.owner {
                    meaning.owner = None;
                }
            }
            meaning.possible |= spaces;
            meaning.type_only &= path.type_only;
        }
        meaning.restrict_value(type_only);
        Ok(ResolvedAlias::new(None, meaning, None, candidates))
    }

    fn resolve_path(
        &self,
        root: Binding,
        members: &[AstUtf8],
    ) -> Result<(Option<SymbolId>, AliasMeaning), SymbolId> {
        let mut target = Some(root.symbol);
        let mut meaning = self.target_meaning(root)?;
        for (index, &name) in members.iter().enumerate() {
            let name = self.tree.get_utf8(name).as_bytes();
            let last = index + 1 == members.len();
            let owner = meaning.owner;
            let mut possible = if owner.is_none() && meaning.possible.contains(Spaces::NAMESPACE) {
                Spaces::all()
            } else {
                Spaces::empty()
            };
            let member = owner.and_then(|owner| self.lookup.ts.member(owner, name));
            let type_only = meaning.type_only;
            (target, meaning) = match member {
                Some(member) => {
                    let flags = member.enum_flags.unwrap_or_default()
                        | member.namespace_flags.unwrap_or_default();
                    let binding = Binding {
                        symbol: member.symbol,
                        flags,
                    };
                    (Some(binding.symbol), self.target_meaning(binding)?)
                }
                None => (None, AliasMeaning::default()),
            };
            if last && let Some(owner) = owner {
                let flags = self.lookup.symbols[owner].flags;
                if flags.intersects(SymbolFlags::CLASS | SymbolFlags::FUNCTION) {
                    possible |= (Spaces::VALUE | Spaces::TYPEOF) - meaning.definite;
                }
            }
            if type_only {
                possible.remove(Spaces::VALUE);
            }
            if !possible.is_empty() {
                target = None;
            }
            meaning.possible |= possible;
            meaning.restrict_value(type_only);
        }
        Ok((target, meaning))
    }
}

impl SymbolTracker<'_> {
    pub(in crate::binder) fn resolve_references(&mut self, tree: &Ast<'_>, scopes: &ScopeTree<'_>) {
        let lookup = LexicalLookup {
            scopes: &scopes.list,
            symbols: &self.symbols,
            symbol_tables: &self.symbol_tables,
            shadowed_internal_names: &self.shadowed_internal_names,
            ts: &self.ts_bindings,
        };
        let mut aliases = FxHashMap::default();
        if !self.ts_state.aliases.is_empty() {
            let resolver = AliasResolver {
                lookup,
                tree,
                references: &self.references,
                aliases: &self.ts_state.aliases,
                active: FxHashSet::default(),
                resolved: FxHashMap::default(),
            };
            aliases = resolver.resolve_all(&self.ts_state.initializers);
            for (&symbol, alias) in &aliases {
                self.ts_bindings.import_aliases.insert(symbol, alias.target);
            }
        }

        let mut candidates = Vec::new();
        let ts = &mut self.ts_bindings;
        for (index, reference) in self.references.iter_mut().enumerate() {
            let id = ReferenceId::from_usize(index);
            let lookup = LexicalLookup {
                scopes: &scopes.list,
                symbols: &self.symbols,
                symbol_tables: &self.symbol_tables,
                shadowed_internal_names: &self.shadowed_internal_names,
                ts,
            };
            let initializer = self
                .ts_state
                .initializers
                .get(&id)
                .map(|symbol| &aliases[symbol]);
            let space = reference.flags.space;
            let name = tree.get_utf8(reference.name).as_bytes();
            let binding = if let Some(alias) = initializer {
                candidates.extend_from_slice(&alias.candidates);
                alias.root
            } else {
                lookup
                    .lookup_with(reference.scope, name, space, &mut candidates, |binding| {
                        let alias = aliases.get(&binding.symbol);
                        let visibility = match alias {
                            Some(alias) => alias.visibility(binding, space),
                            None => visibility(binding, None, space),
                        };
                        Ok::<_, Infallible>(visibility)
                    })
                    .unwrap()
            };
            reference.symbol = binding.map(|binding| binding.symbol);
            if let Some(alias) = initializer {
                if !alias.meaning.spaces().contains(Spaces::VALUE)
                    && (binding.is_some()
                        || !candidates.is_empty()
                        || lookup.lookup(reference.scope, name, space).is_some())
                {
                    reference.flags = reference.flags.no_access();
                }
            } else if space == ReferenceSpace::Any && (binding.is_some() || !candidates.is_empty())
            {
                let reads_value =
                    binding
                        .into_iter()
                        .chain(candidates.iter().copied())
                        .any(|binding| {
                            let alias = aliases.get(&binding.symbol).map(|alias| alias.meaning);
                            binding.meaning(alias).spaces().contains(Spaces::VALUE)
                        });
                if !reads_value {
                    reference.flags = reference.flags.no_access();
                }
            }
            if !candidates.is_empty() {
                let ranges = &mut ts.reference_symbol_ranges;
                if ranges.len() <= id.index() {
                    ranges.resize(id.index() + 1, Range::default());
                }
                ranges[id] = Range {
                    start: ts.reference_symbols.len() as u32,
                    len: candidates.len() as u32,
                };
                for binding in candidates.drain(..) {
                    ts.reference_symbols.push(binding.symbol);
                }
            }
        }
    }
}
