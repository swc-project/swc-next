//! Lexical scope walking and binding lookup shared by JavaScript and TypeScript.

use std::hash::BuildHasher;

use swc_next_ecma_ast::{ScopeId, SymbolId};

use super::typescript::{binding_visible_from, is_signature_query};
use super::{NameResolution, ReferenceSpace, SymbolFlags};
use crate::{ScopeMaps, Scopes, Symbols, scope::ScopeKind};

/// One identifier lookup: `type R = N.X` queries name `N` in namespace space.
#[derive(Clone, Copy)]
pub(super) struct NameLookupQuery<'a> {
    pub(super) scope: ScopeId,
    pub(super) name: &'a [u8],
    pub(super) space: ReferenceSpace,
}

impl NameLookupQuery<'_> {
    pub(super) fn arguments_barrier(self) -> bool {
        self.name == b"arguments"
            && matches!(self.space, ReferenceSpace::Value | ReferenceSpace::Typeof)
    }
}

/// Borrowed lexical scope/symbol indexes used before consulting shared TS member tables.
#[derive(Clone, Copy)]
pub(super) struct LexicalLookup<'lookup, 'arena> {
    pub(super) scopes: &'lookup Scopes,
    pub(super) symbols: &'lookup Symbols,
    pub(super) internal_names: &'lookup crate::InternalNames<'arena>,
    pub(super) scope_maps: &'lookup ScopeMaps<'arena>,
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
        ..
    } = lexical;
    let arguments_barrier = query.arguments_barrier();
    let NameLookupQuery { scope, name, space } = query;
    let hash = scope_maps[ScopeId::ROOT].hasher().hash_one(name);
    let mut ancestor = Some(scope);
    let mut child_kind = ScopeKind::Block;
    while let Some(current) = ancestor {
        if !INCLUDE_GLOBAL && current == ScopeId::ROOT {
            break;
        }
        if let Some(symbol) =
            lookup_scope_binding::<TS_LOOKUP>(lexical, current, child_kind, query, hash, |symbol| {
                let flags = symbols[symbol].flags;
                (!TS_LOOKUP || binding_visible_from(child_kind, flags, space))
                    && (!FILTER_BY_SPACE || flags.visible_in(space))
            })
        {
            return NameResolution::Resolved(symbol);
        }
        if arguments_barrier && scopes[current].flags.arguments_barrier {
            break;
        }
        child_kind = scopes[current].kind;
        ancestor = scopes[current].parent;
    }
    NameResolution::Unresolved
}

// Signature queries inspect permitted declarations in a split body before
// walking outward. Runtime parameter initializers never take this edge.
#[inline(always)]
pub(super) fn lookup_scope_binding<const TS_LOOKUP: bool>(
    lexical: LexicalLookup<'_, '_>,
    current: ScopeId,
    child: ScopeKind,
    query: NameLookupQuery<'_>,
    hash: u64,
    mut accept: impl FnMut(SymbolId) -> bool,
) -> Option<SymbolId> {
    let LexicalLookup {
        scopes,
        symbols,
        scope_maps,
        ..
    } = lexical;
    // The implicit arguments object shadows an expression or if-branch name,
    // even though both are represented by the same function scope.
    let arguments_barrier = query.arguments_barrier() && scopes[current].flags.arguments_barrier;
    let mut accept_visible = |symbol: SymbolId| {
        (!arguments_barrier
            || !symbols[symbol]
                .flags
                .intersects(SymbolFlags::FUNCTION_EXPRESSION | SymbolFlags::ANNEX_B_IF_FUNCTION))
            && accept(symbol)
    };
    if TS_LOOKUP
        && is_signature_query(child, query.space)
        && let Some(body) = scopes[current].function_body
        && let Some(symbol) = lookup_scope_name(scope_maps, body, query.name, hash)
        && accept_visible(symbol)
    {
        return Some(symbol);
    }
    let symbol = lookup_scope_name(scope_maps, current, query.name, hash)?;
    if accept_visible(symbol) {
        return Some(symbol);
    }
    // Flattened internal names remain an outer fallback when a same-named
    // type parameter or a declaration hidden from a signature is rejected.
    let &internal_name = lexical
        .internal_names
        .get(&(current, symbols[symbol].name))?;
    if internal_name == symbol {
        return None;
    }
    accept_visible(internal_name).then_some(internal_name)
}

#[inline(always)]
pub(super) fn lookup_scope_name(
    scope_maps: &ScopeMaps<'_>,
    scope: ScopeId,
    name: &[u8],
    hash: u64,
) -> Option<SymbolId> {
    let bindings = &scope_maps[scope];
    if bindings.is_empty() {
        return None;
    }
    let (_, &symbol) = bindings
        .raw_entry()
        .from_hash(hash, |candidate| *candidate == name)?;
    Some(symbol)
}
