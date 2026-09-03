//! Lexical and shared TypeScript member lookup, including uncertain candidates.

use std::hash::BuildHasher;

use oxc_index::Idx;
use swc_next_ecma_ast::{ScopeId, SymbolId};

use super::{
    BindingVisibility, NameResolution, ReferenceSpace, ScopeMap, Symbol, SymbolFlags,
    TsLocalExportMap, TsMemberEnvironments, TsMemberView, TsResolvedAliasSpaces, TsScopeViewIndex,
    TsSourceExports,
};
use crate::scope::{Scope, ScopeKind};

#[derive(Clone, Copy)]
pub(super) struct NameLookupQuery<'a> {
    pub(super) scope: ScopeId,
    pub(super) name: &'a [u8],
    pub(super) space: ReferenceSpace,
}

impl NameLookupQuery<'_> {
    fn arguments_barrier(self) -> bool {
        self.name == b"arguments"
            && matches!(self.space, ReferenceSpace::Value | ReferenceSpace::Typeof)
    }
}

#[derive(Clone, Copy)]
pub(super) struct LexicalLookup<'lookup, 'arena> {
    pub(super) scopes: &'lookup [Scope],
    pub(super) symbols: &'lookup [Symbol],
    pub(super) scope_maps: &'lookup [ScopeMap<'arena>],
}

pub(super) struct TsNameLookup<'lookup, 'arena> {
    pub(super) source_exports: Option<&'lookup TsSourceExports<'arena>>,
    pub(super) local_exports: &'lookup TsLocalExportMap<'arena>,
    pub(super) environments: &'lookup TsMemberEnvironments<'arena>,
    pub(super) views: &'lookup [TsMemberView],
    // Missing entries have no refinement; declaration flags remain the fallback.
    pub(super) import_alias_spaces: &'lookup [Option<TsResolvedAliasSpaces>],
}

/// A canonical declaration together with the member view that supplied it.
/// Only alias-backed views need context beyond the declaration's own symbol.
#[derive(Clone, Copy)]
pub(super) struct NameBinding {
    pub(super) symbol: SymbolId,
    pub(super) member: Option<TsMemberView>,
}

impl NameBinding {
    pub(super) fn symbol(symbol: SymbolId) -> Self {
        Self {
            symbol,
            member: None,
        }
    }
}

#[derive(Default)]
pub(super) struct NameLookupResult {
    pub(super) binding: Option<NameBinding>,
    uncertain: bool,
}

impl NameLookupResult {
    pub(super) fn resolution(&self) -> NameResolution {
        match self.binding {
            Some(binding) if self.uncertain => NameResolution::Uncertain {
                candidate: binding.symbol,
            },
            Some(binding) => NameResolution::Resolved(binding.symbol),
            None => NameResolution::Unresolved,
        }
    }

    fn lexical(resolution: NameResolution) -> Self {
        Self {
            binding: resolution.candidate().map(NameBinding::symbol),
            uncertain: matches!(resolution, NameResolution::Uncertain { .. }),
        }
    }

    // A possible contribution does not stop the search: every outer fallback
    // must reach the reference's candidate set and reverse use index as well.
    fn accept(
        &mut self,
        binding: NameBinding,
        visibility: BindingVisibility,
        emit: &mut impl FnMut(NameBinding),
    ) -> bool {
        if visibility == BindingVisibility::Absent {
            return false;
        }
        if let Some(first) = self.binding {
            if binding.symbol != first.symbol || binding.member != first.member {
                emit(binding);
            }
        } else {
            self.binding = Some(binding);
        }
        self.uncertain |= visibility == BindingVisibility::Unknown;
        visibility == BindingVisibility::Present
    }
}

#[inline(always)]
pub(super) fn lookup_name_in_scopes<
    const INCLUDE_GLOBAL: bool,
    const PRECISE_IMPORT_ALIASES: bool,
>(
    lexical: LexicalLookup<'_, '_>,
    ts: &TsNameLookup<'_, '_>,
    scope_views: Option<&TsScopeViewIndex<'_>>,
    query: NameLookupQuery<'_>,
    mut emit: impl FnMut(NameBinding),
) -> NameLookupResult {
    let Some(scope_views) = scope_views else {
        return NameLookupResult::lexical(if PRECISE_IMPORT_ALIASES {
            lookup_lexical_name_with_import_aliases::<INCLUDE_GLOBAL>(
                lexical,
                ts.import_alias_spaces,
                query,
                |symbol| emit(NameBinding::symbol(symbol)),
            )
        } else {
            lookup_lexical_name::<INCLUDE_GLOBAL, true, true>(lexical, query)
        });
    };
    lookup_name_with_ts_views::<INCLUDE_GLOBAL, PRECISE_IMPORT_ALIASES, false>(
        lexical,
        ts,
        scope_views,
        query,
        emit,
    )
}

#[inline(always)]
pub(super) fn lookup_name_with_ts_views<
    const INCLUDE_GLOBAL: bool,
    const PRECISE_IMPORT_ALIASES: bool,
    const MEMBER_VALUES: bool,
>(
    lexical: LexicalLookup<'_, '_>,
    ts: &TsNameLookup<'_, '_>,
    scope_views: &TsScopeViewIndex<'_>,
    query: NameLookupQuery<'_>,
    emit: impl FnMut(NameBinding),
) -> NameLookupResult {
    if scope_views.lookup[query.scope.index()].restricted_lexical {
        lookup_name_with_ts_views_policy::<
            INCLUDE_GLOBAL,
            PRECISE_IMPORT_ALIASES,
            MEMBER_VALUES,
            true,
        >(lexical, ts, scope_views, query, emit)
    } else {
        lookup_name_with_ts_views_policy::<
            INCLUDE_GLOBAL,
            PRECISE_IMPORT_ALIASES,
            MEMBER_VALUES,
            false,
        >(lexical, ts, scope_views, query, emit)
    }
}

#[inline(always)]
fn lookup_name_with_ts_views_policy<
    const INCLUDE_GLOBAL: bool,
    const PRECISE_IMPORT_ALIASES: bool,
    const MEMBER_VALUES: bool,
    const TS_LOOKUP: bool,
>(
    lexical: LexicalLookup<'_, '_>,
    ts: &TsNameLookup<'_, '_>,
    scope_views: &TsScopeViewIndex<'_>,
    query: NameLookupQuery<'_>,
    mut emit: impl FnMut(NameBinding),
) -> NameLookupResult {
    let LexicalLookup {
        scopes,
        symbols,
        scope_maps,
    } = lexical;
    let arguments_barrier = query.arguments_barrier();
    let symbol_mask = if TS_LOOKUP {
        query.space.symbol_mask()
    } else {
        None
    };
    let NameLookupQuery { scope, name, space } = query;
    let Some(member_scope) = scope_views.lookup[scope.index()].members else {
        return NameLookupResult::lexical(if PRECISE_IMPORT_ALIASES {
            lookup_lexical_name_with_import_aliases::<INCLUDE_GLOBAL>(
                lexical,
                ts.import_alias_spaces,
                query,
                |symbol| emit(NameBinding::symbol(symbol)),
            )
        } else {
            lookup_lexical_name::<INCLUDE_GLOBAL, TS_LOOKUP, true>(lexical, query)
        });
    };
    let hash = scope_maps[ScopeId::ROOT.index()].hasher().hash_one(name);
    let mut state = NameLookupResult::default();
    let mut ancestor = Some(scope);
    let mut child_kind = ScopeKind::Block;
    // Ordinary lexical bindings can finish before an export/member overlay
    // introduces additional candidates or uncertainty.
    if !PRECISE_IMPORT_ALIASES {
        while let Some(current) = ancestor {
            if !INCLUDE_GLOBAL && current == ScopeId::ROOT {
                return NameLookupResult::default();
            }
            let binding = lookup_scope_binding::<TS_LOOKUP>(
                scopes,
                scope_maps,
                current,
                child_kind,
                query,
                hash,
                |symbol| {
                    let flags = symbols[symbol.index()].flags;
                    if TS_LOOKUP {
                        binding_visible_from(child_kind, flags, space)
                            && symbol_mask.is_none_or(|mask| flags.intersects(mask))
                    } else {
                        flags.visible_in(space)
                    }
                },
            );
            if let Some(symbol) = binding {
                return NameLookupResult::lexical(NameResolution::Resolved(symbol));
            }
            if current == member_scope {
                break;
            }
            if arguments_barrier && scopes[current.index()].flags.arguments_barrier {
                return NameLookupResult::default();
            }
            child_kind = scopes[current.index()].kind;
            ancestor = scopes[current.index()].parent;
        }
    }
    while let Some(current) = ancestor {
        if !INCLUDE_GLOBAL && current == ScopeId::ROOT {
            break;
        }
        // The lexical prefix already rejected this scope with the same rules.
        let binding = if PRECISE_IMPORT_ALIASES || current != member_scope {
            lookup_scope_binding::<TS_LOOKUP>(
                scopes,
                scope_maps,
                current,
                child_kind,
                query,
                hash,
                |symbol| {
                    if TS_LOOKUP
                        && !binding_visible_from(child_kind, symbols[symbol.index()].flags, space)
                    {
                        return false;
                    }
                    let visibility = symbol_visibility::<PRECISE_IMPORT_ALIASES>(
                        symbols,
                        ts.import_alias_spaces,
                        symbol,
                        space,
                    );
                    state.accept(NameBinding::symbol(symbol), visibility, &mut emit)
                },
            )
        } else {
            None
        };
        if binding.is_some() {
            return state;
        }
        if current == ScopeId::MODULE
            && let Some(symbol) = ts
                .source_exports
                .and_then(|export| export.lookup(name, space, symbols, ts.local_exports))
            && state.accept(
                NameBinding::symbol(symbol),
                BindingVisibility::Present,
                &mut emit,
            )
        {
            return state;
        }
        let range = scope_views.ranges[current.index()];
        let start = range.start as usize;
        for view in &ts.views[start..start + range.len as usize] {
            let Some(member) = view.lookup(ts.environments, hash, name) else {
                continue;
            };
            if member.visit_candidates::<PRECISE_IMPORT_ALIASES>(
                symbols,
                ts.import_alias_spaces,
                space,
                |symbol, visibility| {
                    let binding = NameBinding {
                        symbol,
                        member: (MEMBER_VALUES && member.target.needs_value_projection())
                            .then_some(*view),
                    };
                    state.accept(binding, visibility, &mut emit)
                },
            ) {
                return state;
            }
        }
        if arguments_barrier && scopes[current.index()].flags.arguments_barrier {
            break;
        }
        child_kind = scopes[current.index()].kind;
        ancestor = scopes[current.index()].parent;
    }
    state
}

#[inline(always)]
pub(super) fn lookup_lexical_name<
    const INCLUDE_GLOBAL: bool,
    const TS_LOOKUP: bool,
    const FILTER_BY_SPACE: bool,
>(
    lexical: LexicalLookup<'_, '_>,
    query: NameLookupQuery<'_>,
) -> NameResolution {
    let LexicalLookup {
        scopes,
        symbols,
        scope_maps,
    } = lexical;
    let arguments_barrier = query.arguments_barrier();
    let NameLookupQuery { scope, name, space } = query;
    let hash = scope_maps[ScopeId::ROOT.index()].hasher().hash_one(name);
    let mut ancestor = Some(scope);
    let mut child_kind = ScopeKind::Block;
    while let Some(current) = ancestor {
        if !INCLUDE_GLOBAL && current == ScopeId::ROOT {
            break;
        }
        if let Some(symbol) = lookup_scope_binding::<TS_LOOKUP>(
            scopes,
            scope_maps,
            current,
            child_kind,
            query,
            hash,
            |symbol| {
                let flags = symbols[symbol.index()].flags;
                (!TS_LOOKUP || binding_visible_from(child_kind, flags, space))
                    && (!FILTER_BY_SPACE || flags.visible_in(space))
            },
        ) {
            return NameResolution::Resolved(symbol);
        }
        if arguments_barrier && scopes[current.index()].flags.arguments_barrier {
            break;
        }
        child_kind = scopes[current.index()].kind;
        ancestor = scopes[current.index()].parent;
    }
    NameResolution::Unresolved
}

#[inline(always)]
pub(super) fn lookup_lexical_name_with_import_aliases<const INCLUDE_GLOBAL: bool>(
    lexical: LexicalLookup<'_, '_>,
    import_alias_spaces: &[Option<TsResolvedAliasSpaces>],
    query: NameLookupQuery<'_>,
    mut emit: impl FnMut(SymbolId),
) -> NameResolution {
    let LexicalLookup {
        scopes,
        symbols,
        scope_maps,
    } = lexical;
    let arguments_barrier = query.arguments_barrier();
    let NameLookupQuery { scope, name, space } = query;
    let hash = scope_maps[ScopeId::ROOT.index()].hasher().hash_one(name);
    let mut state = NameLookupResult::default();
    let mut ancestor = Some(scope);
    let mut child_kind = ScopeKind::Block;
    while let Some(current) = ancestor {
        if !INCLUDE_GLOBAL && current == ScopeId::ROOT {
            break;
        }
        let binding = lookup_scope_binding::<true>(
            scopes,
            scope_maps,
            current,
            child_kind,
            query,
            hash,
            |symbol| {
                if !binding_visible_from(child_kind, symbols[symbol.index()].flags, space) {
                    return false;
                }
                let visibility =
                    symbol_visibility::<true>(symbols, import_alias_spaces, symbol, space);
                state.accept(NameBinding::symbol(symbol), visibility, &mut |binding| {
                    emit(binding.symbol)
                })
            },
        );
        if binding.is_some() {
            return state.resolution();
        }
        if arguments_barrier && scopes[current.index()].flags.arguments_barrier {
            break;
        }
        child_kind = scopes[current.index()].kind;
        ancestor = scopes[current.index()].parent;
    }
    state.resolution()
}

// Signature queries inspect permitted declarations in a split body before
// walking outward. Runtime parameter initializers never take this edge.
#[inline(always)]
fn lookup_scope_binding<const TS_LOOKUP: bool>(
    scopes: &[Scope],
    scope_maps: &[ScopeMap<'_>],
    current: ScopeId,
    child: ScopeKind,
    query: NameLookupQuery<'_>,
    hash: u64,
    mut accept: impl FnMut(SymbolId) -> bool,
) -> Option<SymbolId> {
    if TS_LOOKUP
        && is_signature_query(child, query.space)
        && let Some(body) = scopes[current.index()].function_body
        && let Some(symbol) = lookup_scope_name(scope_maps, body, query.name, hash)
        && accept(symbol)
    {
        return Some(symbol);
    }
    let symbol = lookup_scope_name(scope_maps, current, query.name, hash)?;
    accept(symbol).then_some(symbol)
}

#[inline(always)]
pub(super) fn lookup_scope_name(
    scope_maps: &[ScopeMap<'_>],
    scope: ScopeId,
    name: &[u8],
    hash: u64,
) -> Option<SymbolId> {
    let bindings = &scope_maps[scope.index()];
    if bindings.is_empty() {
        return None;
    }
    let (_, &symbol) = bindings
        .raw_entry()
        .from_hash(hash, |candidate| *candidate == name)?;
    Some(symbol)
}

fn is_signature_query(kind: ScopeKind, space: ReferenceSpace) -> bool {
    if kind == ScopeKind::TsSignatureParameter
        && matches!(space, ReferenceSpace::Value | ReferenceSpace::Any)
    {
        return false;
    }
    matches!(
        kind,
        ScopeKind::TsTypeParameters
            | ScopeKind::TsSignatureReturn
            | ScopeKind::TsSignatureParameter
    )
}

fn binding_visible_from(child: ScopeKind, flags: SymbolFlags, space: ReferenceSpace) -> bool {
    if child == ScopeKind::TsConditionalExtends {
        return !flags.contains(SymbolFlags::TYPE_PARAMETER);
    }
    if !is_signature_query(child, space) {
        return true;
    }
    // A class or enum has type meaning even when requested through `typeof`.
    // A function merged with an interface does not: only the meanings matched
    // by this query participate in the signature's local-type restriction.
    let meanings = match space {
        ReferenceSpace::Type => SymbolFlags::TYPE_SPACE,
        ReferenceSpace::Value | ReferenceSpace::Typeof => SymbolFlags::VALUE_SPACE,
        ReferenceSpace::Namespace => SymbolFlags::NAMESPACE_SPACE,
        ReferenceSpace::Any => SymbolFlags::all(),
    };
    if flags.intersects(meanings & SymbolFlags::TYPE_SPACE) {
        return flags.contains(SymbolFlags::TYPE_PARAMETER);
    }
    if child == ScopeKind::TsSignatureParameter
        && flags.intersects(SymbolFlags::VARIABLE)
        && !flags.contains(SymbolFlags::PARAMETER)
    {
        return false;
    }
    // Return annotations and type-parameter constraints can see body let/const
    // declarations, but body var declarations remain hidden. Function
    // declarations are visible in all three signature regions.
    if flags.contains(SymbolFlags::FUNCTION_SCOPED_VAR) {
        return child != ScopeKind::TsTypeParameters && flags.contains(SymbolFlags::PARAMETER);
    }
    true
}

#[inline]
pub(super) fn symbol_visibility<const PRECISE_IMPORT_ALIASES: bool>(
    symbols: &[Symbol],
    import_alias_spaces: &[Option<TsResolvedAliasSpaces>],
    symbol: SymbolId,
    space: ReferenceSpace,
) -> BindingVisibility {
    let flags = symbols[symbol.index()].flags;
    if PRECISE_IMPORT_ALIASES
        && let Some(spaces) = precise_import_alias_spaces(symbols, import_alias_spaces, symbol)
    {
        return spaces.visibility(space, flags);
    }
    if flags.visible_in(space) {
        BindingVisibility::Present
    } else {
        BindingVisibility::Absent
    }
}

#[inline]
pub(super) fn precise_import_alias_spaces(
    symbols: &[Symbol],
    import_alias_spaces: &[Option<TsResolvedAliasSpaces>],
    symbol: SymbolId,
) -> Option<TsResolvedAliasSpaces> {
    let flags = symbols[symbol.index()].flags;
    if !flags.intersects(SymbolFlags::ANY_IMPORT) {
        return None;
    }
    import_alias_spaces.get(symbol.index()).copied().flatten()
}
