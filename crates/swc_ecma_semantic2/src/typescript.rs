//! Additional TypeScript relationships between the existing semantic IDs.
//!
//! Symbols, declarations, references, lexical bindings, and declaration flags
//! continue to use `SymbolTracker`'s existing storage. These side tables
//! in `TsBinder` hold only the relationships that storage does not express.
//! Binding scratch data need not become part of `Semantic`.

#![expect(
    dead_code,
    reason = "side-table definitions precede TypeScript analysis integration"
)]

use swc_next_allocator::{
    hash_map::HashMap,
    vec::{ArenaIndexVec, ArenaVec},
};
use swc_next_ecma_ast::{ReferenceId, ScopeId, SymbolId};

use crate::{SymbolFlags, binder::Range};

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

pub(crate) type TsMemberTable<'a> = HashMap<'a, &'a [u8], TsMember>;

/// A shared member with at least one enum or namespace contribution.
#[derive(Debug)]
pub(crate) struct TsMember {
    /// Canonical identity shared by both contributions.
    pub(crate) symbol: SymbolId,
    /// Flags contributed as an enum member; `None` means absent.
    pub(crate) enum_flags: Option<SymbolFlags>,
    /// Flags contributed by explicit or implicit namespace exports; `None` means absent.
    pub(crate) namespace_flags: Option<SymbolFlags>,
}

#[derive(Debug)]
pub(crate) enum TsImportAlias {
    /// Known direct target; visibility follows the target and alias-chain restrictions.
    Target(SymbolId),
    /// Possible spaces without a known target; empty means no usable target.
    Unknown(TsImportAliasSpace),
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) struct TsImportAliasSpace: u8 {
        const VALUE = 1 << 0;
        const TYPE = 1 << 1;
        const NAMESPACE = 1 << 2;
        const TYPEOF = 1 << 3;
    }
}
