use std::{convert::Infallible, hash::BuildHasher};

use swc_next_allocator::IndexSlice;

use super::*;
use crate::{Scopes, SymbolTables, Symbols, ts::TsImportAlias};

#[derive(Clone, Copy)]
struct Binding {
    symbol: SymbolId,
    flags: SymbolFlags,
}

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    struct Spaces: u8 {
        const VALUE = 1 << 0;
        const TYPE = 1 << 1;
        const NAMESPACE = 1 << 2;
        const TYPEOF = 1 << 3;
    }
}

#[derive(Clone, Copy)]
struct AliasMeaning {
    spaces: Spaces,
    owner: Option<SymbolId>,
    type_only: bool,
}

impl AliasMeaning {
    fn direct(binding: Binding) -> Self {
        let flags = binding.flags;
        let mut spaces = Spaces::empty();
        spaces.set(
            Spaces::VALUE,
            flags.contains(SymbolFlags::EXPORT_VALUE) || flags.visible_in(ReferenceSpace::Value),
        );
        spaces.set(
            Spaces::TYPEOF,
            flags.contains(SymbolFlags::EXPORT_VALUE) || flags.visible_in(ReferenceSpace::Typeof),
        );
        spaces.set(Spaces::TYPE, flags.visible_in(ReferenceSpace::Type));
        spaces.set(
            Spaces::NAMESPACE,
            flags.visible_in(ReferenceSpace::Namespace),
        );
        Self {
            spaces,
            owner: flags.in_namespace_space().then_some(binding.symbol),
            type_only: flags.contains(SymbolFlags::TYPE_IMPORT),
        }
    }

    fn unknown(type_only: bool) -> Self {
        // Like tsc's unknownSymbol, an unknown target leaves its alias visible.
        let mut spaces = Spaces::all();
        spaces.set(Spaces::VALUE, !type_only);
        Self {
            spaces,
            owner: None,
            type_only,
        }
    }

    fn restrict_value(&mut self, type_only: bool) {
        if type_only {
            self.spaces.remove(Spaces::VALUE);
            self.type_only = true;
        }
    }

    fn visible_in(self, space: ReferenceSpace) -> bool {
        let spaces = match space {
            ReferenceSpace::Value => Spaces::VALUE,
            ReferenceSpace::Type => Spaces::TYPE,
            ReferenceSpace::Namespace => Spaces::NAMESPACE,
            ReferenceSpace::Typeof => Spaces::TYPEOF,
            ReferenceSpace::Any => Spaces::all(),
        };
        self.spaces.intersects(spaces)
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
        direct.spaces |= alias.spaces;
        if direct.owner.is_none() {
            direct.owner = alias.owner;
            direct.type_only = alias.type_only;
        }
        direct
    }

    fn visible_in(self, alias: Option<AliasMeaning>, space: ReferenceSpace) -> bool {
        if alias.is_some() && self.flags.intersects(SymbolFlags::ANY_IMPORT) {
            return self.meaning(alias).visible_in(space);
        }
        self.flags.visible_in(space)
            || (self.flags.contains(SymbolFlags::EXPORT_VALUE)
                && matches!(space, ReferenceSpace::Value | ReferenceSpace::Typeof))
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
        let result = self.lookup_with(scope, name, space, |binding| {
            Ok::<_, Infallible>(binding.visible_in(None, space))
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
        mut visible: impl FnMut(Binding) -> Result<bool, E>,
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
                if !shadowed && visible(binding)? {
                    return Ok(Some(binding));
                }
                hidden = self.shadowed_internal_names.get(&symbol).copied();
            }

            if let Some(binding) = self.public_binding(current, name, local)
                && visible(binding)?
            {
                return Ok(Some(binding));
            }
            if barrier {
                break;
            }
            ancestor = scope.parent;
        }
        Ok(None)
    }
}

struct ResolvedAlias {
    target: TsImportAlias,
    meaning: AliasMeaning,
    root: Option<Binding>,
}

/// An unresolved dependency suspends an alias while the dependency is resolved.
/// This keeps forward chains and cycles off the Rust call stack.
struct AliasResolver<'s, 'a> {
    lookup: &'s LexicalLookup<'s, 'a>,
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
            let type_only = binding.flags.contains(SymbolFlags::TYPE_IMPORT);
            return Ok(Some(AliasMeaning::unknown(type_only)));
        }
        Err(symbol)
    }

    fn target_meaning(&self, binding: Binding) -> Result<AliasMeaning, SymbolId> {
        let mut meaning = binding.meaning(self.alias_meaning(binding)?);
        if let Some(export) = self.lookup.export(binding.symbol) {
            let export = export.meaning(self.alias_meaning(export)?);
            meaning.spaces |= export.spaces;
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
        let lookup = |space| -> Result<Option<Binding>, SymbolId> {
            self.lookup.lookup_with(root.scope, name, space, |binding| {
                Ok(binding.visible_in(self.alias_meaning(binding)?, space))
            })
        };
        let mut binding = lookup(ReferenceSpace::Namespace)?;
        if binding.is_none() && alias.members.is_empty() {
            binding = lookup(ReferenceSpace::Any)?;
        }
        let type_only = self.lookup.symbols[symbol]
            .flags
            .contains(SymbolFlags::TYPE_IMPORT);
        let (target, mut meaning) = match binding {
            Some(binding) => self.resolve_path(binding, &alias.members)?,
            None => (TsImportAlias::Unknown, AliasMeaning::unknown(false)),
        };
        meaning.restrict_value(type_only);
        Ok(ResolvedAlias {
            target,
            meaning,
            root: binding,
        })
    }

    fn resolve_path(
        &self,
        mut binding: Binding,
        members: &[AstUtf8],
    ) -> Result<(TsImportAlias, AliasMeaning), SymbolId> {
        let mut meaning = self.target_meaning(binding)?;
        for &name in members {
            let name = self.tree.get_utf8(name).as_bytes();
            let type_only = meaning.type_only;
            let Some(member) = meaning
                .owner
                .and_then(|owner| self.lookup.ts.member(owner, name))
            else {
                return Ok((TsImportAlias::Unknown, AliasMeaning::unknown(type_only)));
            };
            binding = Binding {
                symbol: member.symbol,
                flags: member.flags(),
            };
            meaning = self.target_meaning(binding)?;
            meaning.restrict_value(type_only);
        }
        let symbol = binding.symbol;
        let declared_meanings = SymbolFlags::VALUE_SPACE
            | SymbolFlags::TYPE_SPACE
            | SymbolFlags::NAMESPACE_SPACE
            | SymbolFlags::EXPORT_VALUE;
        let target = if binding.flags.intersects(SymbolFlags::ANY_IMPORT)
            && !binding.flags.intersects(declared_meanings)
            && (self.active.contains(&symbol)
                || self
                    .resolved
                    .get(&symbol)
                    .is_some_and(|alias| alias.target == TsImportAlias::Unknown))
        {
            TsImportAlias::Unknown
        } else {
            TsImportAlias::Target(symbol)
        };
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
                lookup: &lookup,
                tree,
                references: &self.references,
                aliases: &self.ts_state.aliases,
                active: FxHashSet::default(),
                resolved: FxHashMap::default(),
            };
            aliases = resolver.resolve_all(&self.ts_state.initializers);
        }

        for (index, reference) in self.references.iter_mut().enumerate() {
            let id = ReferenceId::from_usize(index);
            let initializer = self
                .ts_state
                .initializers
                .get(&id)
                .map(|symbol| &aliases[symbol]);
            let space = reference.flags.space;
            let name = tree.get_utf8(reference.name).as_bytes();
            let binding = if let Some(alias) = initializer {
                alias.root
            } else {
                lookup
                    .lookup_with(reference.scope, name, space, |binding| {
                        let alias = aliases.get(&binding.symbol).map(|alias| alias.meaning);
                        Ok::<_, Infallible>(binding.visible_in(alias, space))
                    })
                    .unwrap()
            };
            reference.symbol = binding.map(|binding| binding.symbol);
            if let Some(alias) = initializer {
                if !alias.meaning.visible_in(ReferenceSpace::Value) {
                    reference.flags = reference.flags.no_access();
                }
            } else if space == ReferenceSpace::Any
                && let Some(binding) = binding
            {
                let alias = aliases.get(&binding.symbol).map(|alias| alias.meaning);
                if !binding.visible_in(alias, ReferenceSpace::Value) {
                    reference.flags = reference.flags.no_access();
                }
            }
        }

        for (symbol, alias) in aliases {
            self.ts_bindings.import_aliases.insert(symbol, alias.target);
        }
    }
}
