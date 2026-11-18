//! Additional TypeScript relationships between the existing semantic IDs.
//!
//! Symbols, declarations, references, lexical bindings, and declaration flags
//! continue to use `SymbolTracker`'s existing storage. `TsBindings` holds the
//! relationships that storage does not express.

use swc_next_allocator::{
    Allocator,
    hash_map::HashMap,
    vec::{ArenaIndexVec, ArenaVec},
};
use swc_next_ecma_ast::{ReferenceId, ScopeId, SymbolId};

use crate::{Semantic, SymbolFlags, binder::Range};

/// TypeScript relationships supplementing `SymbolTracker`.
#[derive(Debug)]
pub(crate) struct TsBindings<'a> {
    /// Body scope -> canonical container symbol; exported containers use their export symbol.
    pub(crate) container_owners: HashMap<'a, ScopeId, SymbolId>,
    /// Shared enum members and namespace exports, indexed by container symbol.
    pub(crate) container_members: HashMap<'a, SymbolId, TsMemberTable<'a>>,
    /// Local symbol -> canonical export symbol for directly exported declarations.
    pub(crate) export_symbols: HashMap<'a, SymbolId, SymbolId>,
    /// Import alias -> direct target or possible declaration spaces.
    pub(crate) import_aliases: HashMap<'a, SymbolId, TsImportAlias>,
    /// Candidate ranges when `Reference.symbol` is `None`: nonempty means uncertain,
    /// empty means unresolved. Definite references have empty ranges.
    pub(crate) reference_symbol_ranges: ArenaIndexVec<'a, ReferenceId, Range>,
    /// Candidates deduplicated per reference, also included in each symbol's uses.
    pub(crate) reference_symbols: ArenaVec<'a, SymbolId>,
}

impl<'a> TsBindings<'a> {
    pub(crate) fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            container_owners: HashMap::new_in(allocator),
            container_members: HashMap::new_in(allocator),
            export_symbols: HashMap::new_in(allocator),
            import_aliases: HashMap::new_in(allocator),
            reference_symbol_ranges: ArenaIndexVec::new_in(allocator),
            reference_symbols: ArenaVec::new_in(allocator),
        }
    }

    pub(crate) fn ts_owner(&self, scope: ScopeId) -> Option<SymbolId> {
        self.container_owners.get(&scope).copied()
    }

    pub(crate) fn member(&self, owner: SymbolId, name: &[u8]) -> Option<TsMember> {
        self.container_members.get(&owner)?.get(name).copied()
    }

    pub(crate) fn candidates(&self, id: ReferenceId) -> &[SymbolId] {
        let Some(range) = self.reference_symbol_ranges.get(id) else {
            return &[];
        };
        let start = range.start as usize;
        &self.reference_symbols[start..start + range.len as usize]
    }
}

pub(crate) type TsMemberTable<'a> = HashMap<'a, &'a [u8], TsMember>;

/// A shared member with at least one enum or namespace contribution.
#[derive(Debug, Clone, Copy)]
pub struct TsMember {
    /// Canonical identity shared by both contributions.
    pub symbol: SymbolId,
    /// Flags contributed as an enum member; `None` means absent.
    pub enum_flags: Option<SymbolFlags>,
    /// Flags contributed by explicit or implicit namespace exports; `None` means absent.
    pub namespace_flags: Option<SymbolFlags>,
}

/// A TypeScript import alias's direct target or possible declaration spaces.
#[derive(Debug, Clone, Copy)]
pub enum TsImportAlias {
    /// Known direct target; visibility follows the target and alias-chain restrictions.
    Target(SymbolId),
    /// Possible spaces without a known target; empty means no usable target.
    Unknown(TsImportAliasSpace),
}

bitflags::bitflags! {
    /// Possible declaration spaces for an alias with no known direct target.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct TsImportAliasSpace: u8 {
        const VALUE = 1 << 0;
        const TYPE = 1 << 1;
        const NAMESPACE = 1 << 2;
        const TYPEOF = 1 << 3;
    }
}

impl<'a> Semantic<'a> {
    /// Returns the canonical container symbol for an enum or namespace body scope.
    pub fn ts_container_owner(&self, scope: ScopeId) -> Option<SymbolId> {
        self.ts_bindings.ts_owner(scope)
    }

    /// Finds a shared enum or namespace member by name on a canonical container.
    /// Each contribution's flags describe its visibility in the corresponding body.
    pub fn ts_container_member(&self, owner: SymbolId, name: &[u8]) -> Option<TsMember> {
        self.ts_bindings.member(owner, name)
    }

    /// Iterates the shared members of a canonical container in unspecified order.
    pub fn ts_container_members(
        &self,
        owner: SymbolId,
    ) -> impl Iterator<Item = (&'a [u8], TsMember)> + '_ {
        self.ts_bindings
            .container_members
            .get(&owner)
            .into_iter()
            .flat_map(|members| members.iter().map(|(&name, &member)| (name, member)))
    }

    /// Returns the canonical export symbol linked to a local symbol.
    pub fn ts_export_symbol(&self, local: SymbolId) -> Option<SymbolId> {
        self.ts_bindings.export_symbols.get(&local).copied()
    }

    /// Returns the recorded resolution of an import alias.
    pub fn ts_import_alias(&self, symbol: SymbolId) -> Option<TsImportAlias> {
        self.ts_bindings.import_aliases.get(&symbol).copied()
    }

    /// Returns the deduplicated candidates for an uncertain reference in unspecified order.
    /// Definite and unresolved references have no candidates.
    pub fn reference_candidates(&self, id: ReferenceId) -> &[SymbolId] {
        self.ts_bindings.candidates(id)
    }
}
