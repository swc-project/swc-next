//! Additional TypeScript relationships between the existing semantic IDs.
//!
//! Symbols, declarations, references, lexical bindings, and declaration flags
//! continue to use `SymbolTracker`'s existing storage. `TsBindings` holds the
//! relationships that storage does not express.

use swc_next_allocator::{Allocator, hash_map::HashMap};
use swc_next_ecma_ast::{ScopeId, SymbolId};

use crate::{Semantic, SymbolFlags};

/// TypeScript relationships supplementing `SymbolTracker`.
#[derive(Debug)]
pub(crate) struct TsBindings<'a> {
    /// TypeScript container body scopes and their canonical owners.
    pub(crate) containers: HashMap<'a, ScopeId, TsContainer>,
    /// Enum members, namespace exports, class statics, and expandos by container symbol.
    pub(crate) container_members: HashMap<'a, SymbolId, TsMemberTable<'a>>,
    /// Local symbol -> canonical export symbol for directly exported declarations.
    pub(crate) export_symbols: HashMap<'a, SymbolId, SymbolId>,
    /// Import-equals and namespace export aliases -> target resolution.
    pub(crate) aliases: HashMap<'a, SymbolId, TsAlias>,
}

impl<'a> TsBindings<'a> {
    pub(crate) fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            containers: HashMap::new_in(allocator),
            container_members: HashMap::new_in(allocator),
            export_symbols: HashMap::new_in(allocator),
            aliases: HashMap::new_in(allocator),
        }
    }

    pub(crate) fn ts_owner(&self, scope: ScopeId) -> Option<SymbolId> {
        self.containers.get(&scope).map(|container| container.owner)
    }

    pub(crate) fn member(&self, owner: SymbolId, name: &[u8]) -> Option<TsMember> {
        self.container_members.get(&owner)?.get(name).copied()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TsContainer {
    pub(crate) owner: SymbolId,
    pub(crate) ambient: bool,
}

pub(crate) type TsMemberTable<'a> = HashMap<'a, &'a [u8], TsMember>;

/// A member of a TypeScript container.
#[derive(Debug, Clone, Copy)]
pub struct TsMember {
    /// Canonical member identity.
    pub symbol: SymbolId,
    /// Flags contributed as an enum member; `None` means absent.
    pub enum_flags: Option<SymbolFlags>,
    /// Flags contributed by explicit or implicit namespace exports; `None` means absent.
    pub namespace_flags: Option<SymbolFlags>,
    /// Flags contributed by named class static declarations; `None` means absent.
    pub class_static_flags: Option<SymbolFlags>,
    /// Whether an assignment introduced this value property.
    pub expando: bool,
}

impl TsMember {
    /// Combined declaration flags for a qualified lookup.
    pub fn flags(self) -> SymbolFlags {
        let mut flags = self.enum_flags.unwrap_or_default()
            | self.namespace_flags.unwrap_or_default()
            | self.class_static_flags.unwrap_or_default();
        if self.expando {
            flags.insert(SymbolFlags::PROPERTY);
        }
        flags
    }
}

/// The result of resolving a TypeScript import-equals or namespace export alias.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsAlias {
    /// Known direct target; visibility follows the target and alias-chain restrictions.
    Target(SymbolId),
    /// The target could not be resolved from the available declarations.
    /// A reference can still resolve to the alias itself.
    Unknown,
}

impl<'a> Semantic<'a> {
    /// Returns the canonical container symbol for an enum or namespace body scope.
    pub fn ts_container_owner(&self, scope: ScopeId) -> Option<SymbolId> {
        self.ts_bindings.ts_owner(scope)
    }

    /// Finds a named member on a canonical container. Use [`TsMember::flags`]
    /// for qualified lookup, including static members and expando properties.
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

    /// Returns the recorded resolution of an import-equals or namespace export alias.
    pub fn ts_alias(&self, symbol: SymbolId) -> Option<TsAlias> {
        self.ts_bindings.aliases.get(&symbol).copied()
    }
}
