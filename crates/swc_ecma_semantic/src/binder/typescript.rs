//! TypeScript binding, member visibility, and alias resolution.
//!
//! Syntax-to-data map:
//! - `namespace N { export const x = 1 }`: `N` owns a table keyed by `"x"`.
//! - `enum E { A }`: `E` owns a table keyed by `"A"`.
//! - `namespace N { export { X as Y } }`: `"Y"` has an export-alias entry.
//! - `import A = N.X`: `A` has its own symbol and an import-alias index entry.
//!
//! Reopened namespace bodies share their owner's table. A view selects which
//! entries are visible inside a body; alias targets share forwarding tails.
//! `TsBindings` retains query data; `TsBuildState` and resolvers hold build scratch.
//! TypeScript-specific records, binding, lookup and alias resolution live here.

use std::{borrow::Cow, hash::BuildHasher, ops::ControlFlow};

use hashbrown::{HashMap, HashSet};
use rustc_hash::FxBuildHasher;

use swc_next_allocator::{
    Allocator, IndexSlice,
    boxed::Box as ArenaBox,
    hash_map::HashMap as ArenaHashMap,
    vec::{ArenaIndexVec, ArenaVec},
};

use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentOperator, Ast, AstUtf8, AstWtf8, BindingIdentifier, ClassType,
    ExportDefaultDeclarationKindData, ExportNamedDeclaration, ImportOrExportKind, MemberExpression,
    ModuleExportName, ModuleExportNameData, NodeId, NodeKind, NodeKindData, ReferenceId, ScopeId,
    SymbolId, TsEnumDeclaration, TsEnumMember, TsFunction, TsGlobalDeclaration,
    TsImportEqualsDeclaration, TsInterfaceDeclaration, TsModuleDeclaration, TsModuleName,
    TsModuleNameData, TsModuleReferenceData, TsNamespaceExportDeclaration, TsQualifiedName,
    TsQualifiedNameLeft, TsQualifiedNameLeftData, TsTypeAliasDeclaration, TsTypeName,
    TsTypeNameData, TsTypePredicate, UnaryOperator,
};

use super::lookup::{
    LexicalLookup, NameLookupQuery, lookup_lexical_name, lookup_scope_binding, lookup_scope_name,
};
use super::{
    DeclarationConflictCandidate, DeclarationConflictKind, DeclarationDestination, Excludes,
    ExportState, NameResolution, PendingBinding, PendingDeclaration, PreparedDeclaration, Range,
    Reference, ReferenceCandidates, ReferenceFlags, ReferenceSpace, SymbolFlags, SymbolTracker,
    UseRanges, import_pending, merge_symbol_flags,
};
use crate::{
    DeclarationId, Declarations, ScopeMap, ScopeMaps, Scopes, Semantic, Symbols, SyntaxChecks,
    ecmascript::unwrap_parentheses,
    scope::{ScopeKind, ScopeTree},
};

swc_next_allocator::define_nonmax_u32_index_type! {
    /// The member table owned by `N` in `namespace N { ... }`, or by an enum/module.
    pub(super) struct TsMemberEnvironmentId;
}

swc_next_allocator::define_nonmax_u32_index_type! {
    /// A shared direct-plus-alias target, such as both meanings of exported `X`.
    pub(super) struct TsExportMemberId;
}

swc_next_allocator::define_nonmax_u32_index_type! {
    /// One local export specifier, such as `X as Y` in `export { X as Y }`.
    pub(super) struct TsExportAliasId;
}

/// Member/export names within one namespace, enum, ambient module, or TS source file.
///
/// ```ts
/// namespace N { export const x = 1 } // N's table: "x" -> declaration of x
/// enum E { A }                      // E's table: "A" -> declaration of A
/// ```
/// Container names `N` and `E` are owners in [`TsMemberEnvironments::owners`].
pub(super) type TsMemberMap<'a> = ArenaHashMap<'a, &'a [u8], TsMemberEntry>;
/// Shared combinations referenced by aliases, indexed by [`TsExportMemberId`].
pub(super) type TsExportMembers = IndexSlice<TsExportMemberId, [TsExportMember]>;
/// Local export specifiers (`export { X as Y }`), indexed by [`TsExportAliasId`].
pub(super) type TsExportAliases = IndexSlice<TsExportAliasId, [TsExportAlias]>;

/// One owner's member table; both bodies of `namespace N {} namespace N {}` share it.
#[derive(Debug)]
pub(super) struct TsMemberEnvironment<'a> {
    /// Exported/member names: `"x"` in `namespace N { export const x = 1 }`.
    pub(super) members: TsMemberMap<'a>,
    /// Contains local export specifiers: `export { X }` or `export { X as Y }`.
    /// Includes type-only and unresolved exports. `import A = N.X` alone leaves this false.
    pub(super) has_aliases: bool,
}

bitflags::bitflags! {
    /// Which declarations of a merged symbol belong to this member entry.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) struct TsMemberKinds: u8 {
        /// Namespace/module member, e.g. `namespace N { export const x = 1 }`.
        const MODULE = 1 << 0;
        /// Enum member, e.g. `enum E { A }` contributes `A`.
        const ENUM = 1 << 1;
        /// Has a non-import namespace/module declaration, e.g. `export interface X {}`.
        const DECLARATION = 1 << 2;
    }
}

bitflags::bitflags! {
    /// Contexts that can refer to a member; one class name can occupy several.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) struct TsMemberSpaces: u8 {
        /// Expression use: `const result = X`.
        const VALUE = 1 << 0;
        /// Type use: `let result: X`.
        const TYPE = 1 << 1;
        /// Qualified-name prefix: `type Result = N.X` looks up `N` here.
        const NAMESPACE = 1 << 2;
        /// Type query: `type Result = typeof X`.
        const TYPEOF = 1 << 3;
    }
}

impl TsMemberSpaces {
    pub(super) fn from_flags(flags: SymbolFlags) -> Self {
        let mut spaces = Self::empty();
        spaces.set(Self::VALUE, flags.visible_in(ReferenceSpace::Value));
        spaces.set(Self::TYPE, flags.visible_in(ReferenceSpace::Type));
        spaces.set(Self::NAMESPACE, flags.visible_in(ReferenceSpace::Namespace));
        spaces.set(Self::TYPEOF, flags.visible_in(ReferenceSpace::Typeof));
        spaces
    }

    pub(super) fn visible_in(self, space: ReferenceSpace) -> bool {
        match space {
            ReferenceSpace::Value => self.contains(Self::VALUE),
            ReferenceSpace::Type => self.contains(Self::TYPE),
            ReferenceSpace::Namespace => self.contains(Self::NAMESPACE),
            ReferenceSpace::Typeof => self.contains(Self::TYPEOF),
            ReferenceSpace::Any => true,
        }
    }
}

/// One public name, which can have both a declaration and an export specifier.
///
/// ```ts
/// namespace N {
///   export interface X {} // "X".direct is this interface
///   const value = 1;
///   export { value as X }; // "X".alias forwards to value
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub(super) struct TsMemberEntry {
    /// A declared member, including `export import A = N.X` and enum members.
    pub(super) direct: Option<TsMemberDeclaration>,
    /// A local export specifier under this name, e.g. `value as X` above.
    pub(super) alias: Option<TsExportAliasId>,
}

impl TsMemberEntry {
    pub(super) fn target(self, aliases: &TsExportAliases) -> Option<TsResolvedImportTarget> {
        let direct = self.direct.map(|declaration| declaration.symbol);
        let alias = self.alias.and_then(|id| match aliases[id].resolution {
            TsExportAliasResolution::Resolved(target) => Some(target),
            TsExportAliasResolution::Unresolved | TsExportAliasResolution::NonLocal => None,
            TsExportAliasResolution::Pending | TsExportAliasResolution::Resolving => {
                unreachable!("export aliases are resolved before querying bindings")
            }
        });
        if direct.is_none() && alias.is_none() {
            return None;
        }
        Some(TsResolvedImportTarget {
            direct,
            alias,
            type_only_export: alias.and_then(|alias| alias.type_only_export),
            property_value: PropertyValue::Absent,
        })
    }
}

/// The declaration part of a member entry, such as `export class C {}`.
#[derive(Debug, Clone, Copy)]
pub(super) struct TsMemberDeclaration {
    /// Canonical identity shared by compatible declarations in reopened bodies.
    pub(super) symbol: SymbolId,
    /// Namespace/module, enum, and non-import declaration contributions.
    pub(super) kinds: TsMemberKinds,
    /// Visibility as a namespace/module member; enum visibility is fixed by its view.
    pub(super) module_spaces: TsMemberVisibility,
}

/// Per-space visibility after resolving members such as `export import A = N.X`.
/// An opaque target can be definitely a type but only possibly a runtime value.
#[derive(Debug, Clone, Copy)]
pub(super) struct TsMemberVisibility {
    /// All spaces the member may occupy, including uncertain alias contributions.
    pub(super) possible: TsMemberSpaces,
    /// Spaces the member is known to occupy; always a subset of `possible`.
    pub(super) certain: TsMemberSpaces,
}

impl TsMemberVisibility {
    pub(super) fn certain(spaces: TsMemberSpaces) -> Self {
        Self {
            possible: spaces,
            certain: spaces,
        }
    }

    pub(super) fn insert(&mut self, other: Self) {
        self.possible |= other.possible;
        self.certain |= other.certain;
    }

    pub(super) fn in_space<const PRECISE_IMPORT_ALIASES: bool>(
        self,
        space: ReferenceSpace,
    ) -> BindingVisibility {
        if self.certain.visible_in(space) {
            BindingVisibility::Present
        } else if PRECISE_IMPORT_ALIASES && self.possible.visible_in(space) {
            // Only internal import-alias refinement introduces uncertain spaces.
            BindingVisibility::Unknown
        } else {
            BindingVisibility::Absent
        }
    }
}

/// Whether a candidate supplies the requested meaning, e.g. `X` in `let v: X`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BindingVisibility {
    /// Does not supply this space; continue searching.
    Absent,
    /// Supplies this space; stop searching.
    Present,
    /// May supply this space; retain the candidate and search outer scopes too.
    Unknown,
}

/// One `export { X as Y }` / `export { X }` specifier, not an `import A = N.X`.
#[derive(Debug, Clone, Copy)]
pub(super) struct TsExportAlias {
    /// `export type { X as Y }` or `export { type X as Y }`.
    pub(super) type_only: bool,
    /// The meaning reached by looking up the local name `X`.
    pub(super) resolution: TsExportAliasResolution,
    /// The `X as Y` specifier node, used for diagnostics and type-only provenance.
    pub(super) node: NodeId,
}

/// Resolution of the local name in `export { X as Y }`.
#[derive(Debug, Clone, Copy)]
pub(super) enum TsExportAliasResolution {
    /// Not visited yet; construction only.
    Pending,
    /// On the dependency stack; construction only, also used to detect cycles.
    Resolving,
    /// Found a declaration or a shared combination of public contributions.
    Resolved(TsResolvedExportAliasTarget),
    /// No usable target, including an alias cycle with no declaration to reach.
    Unresolved,
    /// Found a script-global name outside the permitted export container.
    NonLocal,
}

/// How a body reads an owner's shared member table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TsMemberViewKind {
    /// Namespace body: `namespace N { ... }`.
    Module,
    /// Ambient body: `declare namespace N { ... }` or `declare module "pkg" { ... }`.
    AmbientModule,
    /// A TypeScript source module, such as a file containing `export const x = 1`.
    SourceFile,
    /// Enum initializer: `enum E { A, B = A }` looks up `A` through this view.
    Enum,
}

impl TsMemberViewKind {
    pub(super) fn accepts(self, kinds: TsMemberKinds) -> bool {
        match self {
            Self::Module | Self::AmbientModule | Self::SourceFile => {
                kinds.contains(TsMemberKinds::MODULE)
            }
            Self::Enum => kinds.contains(TsMemberKinds::ENUM),
        }
    }

    pub(super) fn spaces(self, entry: TsMemberDeclaration) -> TsMemberVisibility {
        match self {
            Self::Module | Self::AmbientModule | Self::SourceFile => entry.module_spaces,
            // Every enum contribution has exactly the enum-member declaration spaces.
            Self::Enum => {
                TsMemberVisibility::certain(TsMemberSpaces::from_flags(SymbolFlags::ENUM_MEMBER))
            }
        }
    }
}

/// An owner's table plus the lookup rules for one body.
/// `namespace N { ... } namespace N { ... }` uses the same environment in both views.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TsMemberView {
    pub(super) environment: TsMemberEnvironmentId,
    pub(super) kind: TsMemberViewKind,
}

impl TsMemberView {
    #[inline(always)]
    pub(super) fn candidates(
        self,
        environments: &TsMemberEnvironments<'_>,
        hash: u64,
        name: &[u8],
    ) -> Option<TsMemberEntry> {
        let members = &environments.environments[self.environment].members;
        let (_, entry) = members
            .raw_entry()
            .from_hash(hash, |candidate| *candidate == name)?;
        let mut entry = *entry;
        entry.direct = entry
            .direct
            .filter(|direct| self.kind.accepts(direct.kinds));
        if self.kind == TsMemberViewKind::Enum {
            entry.alias = None;
        }
        // Pure export specifiers are not locals of source files or ambient bodies.
        let declaration_only = matches!(
            self.kind,
            TsMemberViewKind::AmbientModule | TsMemberViewKind::SourceFile
        );
        if declaration_only
            && entry.alias.is_some()
            && !entry
                .direct
                .is_some_and(|direct| direct.kinds.contains(TsMemberKinds::DECLARATION))
        {
            return None;
        }
        Some(entry)
    }

    #[inline(always)]
    pub(super) fn lookup(
        self,
        environments: &TsMemberEnvironments<'_>,
        hash: u64,
        name: &[u8],
    ) -> Option<TsMemberLookup> {
        let entry = self.candidates(environments, hash, name)?;
        let target = entry.target(&environments.aliases)?;
        let direct_spaces = entry
            .direct
            .map(|entry| self.kind.spaces(entry))
            .unwrap_or_else(|| TsMemberVisibility::certain(TsMemberSpaces::empty()));
        Some(TsMemberLookup {
            target,
            direct_spaces,
        })
    }
}

/// One view's lookup of a member name, e.g. `X` inside a reopened `namespace N`.
pub(super) struct TsMemberLookup {
    /// Selected declaration and export-alias contributions for `X`.
    pub(super) target: TsResolvedImportTarget,
    /// The direct declaration's spaces in this view; alias spaces are resolved separately.
    pub(super) direct_spaces: TsMemberVisibility,
}

impl TsMemberLookup {
    pub(super) fn visit_candidates<const PRECISE_IMPORT_ALIASES: bool>(
        &self,
        symbols: &Symbols,
        import_alias_spaces: &TsImportAliasSpaces,
        members: &TsExportMembers,
        space: ReferenceSpace,
        mut accept: impl FnMut(SymbolId, BindingVisibility) -> bool,
    ) -> bool {
        if let Some(symbol) = self.target.direct {
            let visibility = self.direct_spaces.in_space::<PRECISE_IMPORT_ALIASES>(space);
            if accept(symbol, visibility) {
                return true;
            }
        }
        let Some(alias) = self.target.alias else {
            return false;
        };
        for contribution in alias
            .target
            .public_contributions(members, self.target.type_only_export)
        {
            if contribution.type_only_export.is_some() && space == ReferenceSpace::Value {
                continue;
            }
            let visibility = if PRECISE_IMPORT_ALIASES
                && let Some(spaces) =
                    precise_import_alias_spaces(symbols, import_alias_spaces, contribution.symbol)
            {
                // Namespace-body lookup uses ModuleMember, excluding property
                // values even when a qualified alias could access them.
                spaces
                    .difference(
                        TsResolvedAliasSpaces::PROPERTY_VALUE
                            | TsResolvedAliasSpaces::UNKNOWN_PROPERTY_VALUE,
                    )
                    .visibility(space, symbols[contribution.symbol].flags)
            } else {
                symbol_visibility::<PRECISE_IMPORT_ALIASES>(
                    symbols,
                    import_alias_spaces,
                    contribution.symbol,
                    space,
                )
            };
            if accept(contribution.symbol, visibility) {
                return true;
            }
        }
        false
    }
}

/// All namespace/enum/module tables and the export targets they share.
#[derive(Debug)]
pub(super) struct TsMemberEnvironments<'a> {
    /// Owner symbol -> table ID: `N` in `namespace N`, `E` in `enum E`.
    pub(super) owners: ArenaHashMap<'a, SymbolId, TsMemberEnvironmentId>,
    /// Member tables indexed by owner-assigned environment ID.
    pub(super) environments: ArenaIndexVec<'a, TsMemberEnvironmentId, TsMemberEnvironment<'a>>,
    /// Shared direct-plus-alias combinations, such as the two meanings of exported `X`.
    pub(super) targets: ArenaIndexVec<'a, TsExportMemberId, TsExportMember>,
    /// Local export specifiers, e.g. `export { X as Y }`.
    pub(super) aliases: ArenaIndexVec<'a, TsExportAliasId, TsExportAlias>,
}

impl<'a> TsMemberEnvironments<'a> {
    pub(super) fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            owners: ArenaHashMap::new_in(allocator),
            environments: ArenaIndexVec::new_in(allocator),
            targets: ArenaIndexVec::new_in(allocator),
            aliases: ArenaIndexVec::new_in(allocator),
        }
    }

    pub(super) fn ensure(
        &mut self,
        owner: SymbolId,
        allocator: &'a Allocator,
    ) -> TsMemberEnvironmentId {
        *self.owners.entry(owner).or_insert_with(|| {
            let environment = self.environments.next_idx();
            self.environments.push(TsMemberEnvironment {
                members: TsMemberMap::new_in(allocator),
                has_aliases: false,
            });
            environment
        })
    }

    pub(super) fn get(&self, owner: SymbolId) -> Option<&TsMemberEnvironment<'a>> {
        let environment = *self.owners.get(&owner)?;
        Some(&self.environments[environment])
    }

    pub(super) fn member(
        &self,
        environment: TsMemberEnvironmentId,
        name: &[u8],
    ) -> Option<SymbolId> {
        let entry = self.environments[environment].members.get(name)?;
        entry.direct.map(|direct| direct.symbol)
    }
}

/// `A`'s resolved spaces in `import A = N.X`, indexed by A's own symbol.
/// `None` keeps ordinary declaration-flag lookup; it does not mean an unknown target.
pub(super) type TsImportAliasSpaces = IndexSlice<SymbolId, [Option<TsResolvedAliasSpaces>]>;

/// Completed results for internal `import A = N.X` aliases; `A` keeps its own binding.
#[derive(Debug)]
pub(super) struct TsImportAliasIndex<'a> {
    /// Type/value/namespace meanings supplied by A's target, indexed by A's symbol.
    pub(super) spaces: ArenaIndexVec<'a, SymbolId, Option<TsResolvedAliasSpaces>>,
    /// Unique terminal symbol after forwarding; absent for opaque, cyclic or composite targets.
    pub(super) targets: ArenaIndexVec<'a, SymbolId, Option<SymbolId>>,
}

impl<'a> TsImportAliasIndex<'a> {
    pub(super) fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            spaces: ArenaIndexVec::new_in(allocator),
            targets: ArenaIndexVec::new_in(allocator),
        }
    }
}

/// TypeScript query data retained after building; excludes pending work and container stacks.
#[derive(Debug)]
pub(crate) struct TsBindings<'a> {
    /// Source-file exports such as `export class C {}` and `export default class {}`.
    pub(super) source_exports: Option<TsSourceExports<'a>>,
    /// Body-local name -> public declaration links for named/default exports.
    pub(super) local_exports: TsLocalExportMap<'a>,
    /// Shared member tables for `namespace N`, `enum E`, ambient modules and the source file.
    pub(super) environments: TsMemberEnvironments<'a>,
    /// Which member views apply at each lexical scope, including inherited lookup boundaries.
    pub(super) scope_views: Option<TsScopeViewIndex<'a>>,
    /// Views referenced by `scope_views` ranges; reopened bodies share an environment.
    pub(super) views: ArenaVec<'a, TsMemberView>,
    /// Resolved meanings of `import A = N.X`; separate from `export { X as Y }` records.
    pub(super) import_aliases: TsImportAliasIndex<'a>,
    /// Extra value targets for references selecting an alias-backed member name.
    pub(super) reference_values: Option<ArenaBox<'a, TsReferenceValues<'a>>>,
}

impl<'a> TsBindings<'a> {
    pub(super) fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            source_exports: None,
            local_exports: ArenaHashMap::new_in(allocator),
            environments: TsMemberEnvironments::new_in(allocator),
            scope_views: None,
            views: ArenaVec::new_in(allocator),
            import_aliases: TsImportAliasIndex::new_in(allocator),
            reference_values: None,
        }
    }
}

impl TsBindings<'_> {
    pub(super) fn collect_binding_values<const PRECISE_IMPORT_ALIASES: bool>(
        &self,
        value_targets: &TsValueTargetLookup<'_, '_>,
        name: &[u8],
        space: ReferenceSpace,
        bindings: impl Iterator<Item = NameBinding>,
        uncertain: bool,
        targets: &mut Vec<NameResolution>,
    ) {
        let spaces = value_targets.import_alias_spaces;
        let hash = value_targets.scope_maps[ScopeId::ROOT]
            .hasher()
            .hash_one(name);
        let mut accept = |symbol, visibility| {
            let value = value_targets.target::<PRECISE_IMPORT_ALIASES>(symbol, name, space);
            if let Some(candidate) = value.candidate() {
                let value = if uncertain || visibility == BindingVisibility::Unknown {
                    NameResolution::Uncertain { candidate }
                } else {
                    value
                };
                if !targets.contains(&value) {
                    targets.push(value);
                }
            }
            visibility == BindingVisibility::Present
        };
        for binding in bindings {
            if let Some(view) = binding.member {
                let member = view
                    .lookup(&self.environments, hash, name)
                    .expect("the selected member remains in its environment");
                member.visit_candidates::<PRECISE_IMPORT_ALIASES>(
                    value_targets.symbols,
                    spaces,
                    &self.environments.targets,
                    space,
                    &mut accept,
                );
            } else {
                accept(binding.symbol, BindingVisibility::Present);
            }
        }
    }

    pub(crate) fn reference_values(&self, reference: ReferenceId) -> Option<&[NameResolution]> {
        self.reference_values.as_ref()?.get(reference)
    }

    pub(crate) fn import_alias_target(&self, alias: SymbolId) -> Option<SymbolId> {
        self.import_aliases.targets.get(alias).copied().flatten()
    }

    pub(crate) fn source_module_symbol(&self) -> Option<SymbolId> {
        self.source_exports.as_ref().map(|source| source.owner)
    }

    pub(crate) fn default_export_symbol(&self) -> Option<SymbolId> {
        self.source_exports
            .as_ref()?
            .default
            .as_ref()
            .map(|default| default.symbol)
    }

    pub(crate) fn source_binding(&self, name: &[u8]) -> Option<SymbolId> {
        if name == b"default" {
            return None;
        }
        let source = self.source_exports.as_ref()?;
        self.environments.member(source.environment, name)
    }

    pub(crate) fn source_bindings<'s>(
        &'s self,
        locals: &'s ScopeMap<'s>,
    ) -> Option<TsSourceBindingIterator<'s>> {
        let source = self.source_exports.as_ref()?;
        let members = &self.environments.environments[source.environment].members;
        Some(TsSourceBindingIterator {
            inner: members.iter(),
            locals,
        })
    }

    pub(crate) fn namespace_members(&self, owner: SymbolId) -> impl Iterator<Item = SymbolId> + '_ {
        self.members(owner, TsMemberViewKind::Module)
    }

    pub(crate) fn namespace_member(&self, owner: SymbolId, name: &[u8]) -> Option<SymbolId> {
        self.member(owner, name, TsMemberViewKind::Module)
    }

    pub(crate) fn namespace_has_export_aliases(&self, owner: SymbolId) -> bool {
        self.environments
            .get(owner)
            .is_some_and(|environment| environment.has_aliases)
    }

    pub(crate) fn namespace_value_targets<'s>(
        &'s self,
        symbols: &'s Symbols,
        declarations: &'s Declarations,
        scope_maps: &'s ScopeMaps<'_>,
        owner: SymbolId,
        name: &'s [u8],
    ) -> impl Iterator<Item = NameResolution> + 's {
        let values = TsValueTargetLookup {
            locals: &self.local_exports,
            symbols,
            declarations,
            scope_maps,
            import_alias_spaces: self.import_aliases.spaces.as_slice(),
        };
        let target = self.environments.get(owner).and_then(|environment| {
            environment
                .members
                .get(name)?
                .target(&self.environments.aliases)
        });
        target
            .into_iter()
            .flat_map(|target| target.public_contributions(&self.environments.targets))
            .filter_map(move |contribution| {
                if contribution.type_only_export.is_some() {
                    return None;
                }
                values.value(contribution.symbol, name, ReferenceSpace::Value, false)
            })
    }

    pub(crate) fn enum_members(&self, owner: SymbolId) -> impl Iterator<Item = SymbolId> + '_ {
        self.members(owner, TsMemberViewKind::Enum)
    }

    pub(crate) fn enum_member(&self, owner: SymbolId, name: &[u8]) -> Option<SymbolId> {
        self.member(owner, name, TsMemberViewKind::Enum)
    }

    pub(super) fn members(
        &self,
        owner: SymbolId,
        kind: TsMemberViewKind,
    ) -> impl Iterator<Item = SymbolId> + '_ {
        let environment = self.environments.get(owner);
        environment
            .into_iter()
            .flat_map(|environment| environment.members.values())
            .filter_map(|entry| entry.direct)
            .filter(move |direct| kind.accepts(direct.kinds))
            .map(|direct| direct.symbol)
    }

    pub(super) fn member(
        &self,
        owner: SymbolId,
        name: &[u8],
        kind: TsMemberViewKind,
    ) -> Option<SymbolId> {
        let environment = self.environments.get(owner)?;
        let direct = environment.members.get(name)?.direct?;
        kind.accepts(direct.kinds).then_some(direct.symbol)
    }
}

impl Semantic<'_> {
    /// Value bindings associated with a reference's selected bindings.
    /// Includes the alias contributions of a selected public member and follows
    /// local TypeScript ExportValue links without changing lexical identity.
    /// Alias space information determines whether each target is a definite
    /// value or only a possible one. Results are never Unresolved.
    ///
    /// This queries value meaning, not runtime usage: a `typeof` or explicit
    /// type-only export can still name a value. Access flags and emit options
    /// determine whether it is needed at runtime. Import aliases remain bindings;
    /// this does not replace them with their final property or declaration target.
    /// `ast` must be the tree analyzed to produce this model.
    pub fn reference_value_targets<'s>(
        &'s self,
        ast: &'s Ast<'_>,
        id: ReferenceId,
    ) -> impl Iterator<Item = NameResolution> + 's {
        let reference = self.reference(id);
        let name = ast.get_utf8(reference.name).as_bytes();
        let values = TsValueTargetLookup {
            locals: &self.typescript.local_exports,
            symbols: &self.symbols,
            declarations: &self.declarations,
            scope_maps: &self.scope_maps,
            import_alias_spaces: self.typescript.import_aliases.spaces.as_slice(),
        };
        let mut member_values = self.typescript.reference_values(id).map(<[_]>::iter);
        let mut candidates = self.reference_candidates(id);
        std::iter::from_fn(move || {
            if let Some(values) = &mut member_values {
                return values.next().copied();
            }
            candidates.find_map(|symbol| {
                values.value(
                    symbol,
                    name,
                    reference.flags.space.value_target_space(),
                    reference.flags.is_uncertain(),
                )
            })
        })
    }

    /// Looks up a name and reports whether its target is known or only possible.
    /// Unknown contributions do not prove that an outer binding is shadowed.
    /// An uncertain result names the nearest candidate, following the same scope
    /// and declaration-space rules as [`Self::lookup`].
    pub fn resolve_name(
        &self,
        scope: ScopeId,
        name: &[u8],
        space: ReferenceSpace,
    ) -> NameResolution {
        let query = NameLookupQuery { scope, name, space };
        let lexical = LexicalLookup {
            scopes: &self.scopes.list,
            symbols: &self.symbols,
            internal_names: &self.internal_names,
            scope_maps: &self.scope_maps,
        };
        let bindings = &self.typescript;
        if !bindings.import_aliases.spaces.is_empty() {
            lookup_name_in_scopes::<true, true>(
                lexical,
                bindings,
                bindings.scope_views.as_ref(),
                query,
                |_| {},
            )
            .resolution()
        } else {
            lookup_name_in_scopes::<true, false>(
                lexical,
                bindings,
                bindings.scope_views.as_ref(),
                query,
                |_| {},
            )
            .resolution()
        }
    }
}

/// Source-file declarations such as `export const x = 1` without a separate local binding.
pub(crate) struct TsSourceBindingIterator<'a> {
    pub(super) inner: swc_next_allocator::hash_map::Iter<'a, &'a [u8], TsMemberEntry>,
    pub(super) locals: &'a ScopeMap<'a>,
}

impl Iterator for TsSourceBindingIterator<'_> {
    type Item = SymbolId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find_map(|(&name, member)| {
            let direct = member.direct?;
            let own = name != b"default" && !self.locals.contains_key(name);
            own.then_some(direct.symbol)
        })
    }
}

/// The target of `export { X as Y }`: a symbol or a shared combination of meanings.
/// `Y` and `Z` in `export { X as Y }; export { Y as Z }` can share the same target.
#[derive(Debug, Clone, Copy)]
pub(super) enum TsExportTarget {
    /// `lexical` is X's identity for declaration conflicts; `public` follows its
    /// local-to-exported-value link for public access. Usually they are identical.
    Symbols { lexical: SymbolId, public: SymbolId },
    /// Both parts of a name such as `export interface X {}; export { value as X }`.
    Member(TsExportMemberId),
}

/// A combined export name referenced by another alias.
/// For `export interface X {}; export { value as X }; export { X as Y }`, Y's
/// target contains the interface as `direct` and the value export as `alias`.
#[derive(Debug)]
pub(super) struct TsExportMember {
    /// The declaration under the combined name, e.g. `interface X` above.
    pub(super) direct: SymbolId,
    /// The export specifier's target, e.g. `value as X` above.
    pub(super) alias: TsResolvedExportAliasTarget,
}

/// Resolved export forwarding, with any type-only restriction on its path.
#[derive(Debug, Clone, Copy)]
pub(super) struct TsResolvedExportAliasTarget {
    pub(super) target: TsExportTarget,
    /// A crossed `export type { X as Y }` specifier; retained for access checks/diagnostics.
    pub(super) type_only_export: Option<NodeId>,
}

impl TsResolvedExportAliasTarget {
    pub(super) fn target(target: TsExportTarget) -> Self {
        Self {
            target,
            type_only_export: None,
        }
    }

    pub(super) fn symbol(symbol: SymbolId) -> Self {
        Self::target(TsExportTarget::Symbols {
            lexical: symbol,
            public: symbol,
        })
    }
}

/// One declaration reached by expanding a public export name's contributions.
#[derive(Clone, Copy)]
pub(super) struct TsExportContribution {
    pub(super) symbol: SymbolId,
    /// The type-only export edge through which this declaration was reached, if any.
    pub(super) type_only_export: Option<NodeId>,
}

/// Walks the direct declaration and forwarded meanings of an exported name.
/// For interface X plus `export { value as X }`, yields both declarations without copying them.
pub(super) struct TsExportContributions<'a> {
    /// Initial direct declaration, yielded before following alias targets.
    direct: Option<SymbolId>,
    /// Next forwarding target; `Member` nodes share the remaining path.
    target: Option<TsExportTarget>,
    members: &'a TsExportMembers,
    /// Select lexical conflict meaning before crossing into a public member target.
    conflict: bool,
    /// Type-only restriction inherited while walking forwarding edges.
    type_only_export: Option<NodeId>,
}

impl TsExportTarget {
    pub(super) fn public_contributions<'a>(
        self,
        members: &'a TsExportMembers,
        type_only_export: Option<NodeId>,
    ) -> TsExportContributions<'a> {
        TsExportContributions {
            direct: None,
            target: Some(self),
            members,
            conflict: false,
            type_only_export,
        }
    }

    pub(super) fn conflict_contributions<'a>(
        self,
        members: &'a TsExportMembers,
    ) -> TsExportContributions<'a> {
        TsExportContributions {
            direct: None,
            target: Some(self),
            members,
            conflict: true,
            type_only_export: None,
        }
    }
}

impl Iterator for TsExportContributions<'_> {
    type Item = TsExportContribution;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(symbol) = self.direct.take() {
            return Some(TsExportContribution {
                symbol,
                type_only_export: None,
            });
        }
        let target = self.target.take()?;
        let type_only_export = self.type_only_export;
        let symbol = match target {
            TsExportTarget::Symbols { lexical, public } => {
                if self.conflict {
                    lexical
                } else {
                    public
                }
            }
            TsExportTarget::Member(member) => {
                let member = &self.members[member];
                self.target = Some(member.alias.target);
                // The member is already a public export symbol. Its alias edge
                // crosses the exported-value bridge, even when the caller is
                // checking a new alias against the member's declaration spaces.
                self.conflict = false;
                self.type_only_export = self.type_only_export.or(member.alias.type_only_export);
                member.direct
            }
        };
        Some(TsExportContribution {
            symbol,
            type_only_export,
        })
    }
}

/// Meaning of `N.X` selected for `import A = N.X`, or for a shared-member lookup.
/// A name can combine a direct declaration, an export specifier, and a property value.
#[derive(Clone, Copy)]
pub(super) struct TsResolvedImportTarget {
    /// Declared member `X`, e.g. `namespace N { export interface X {} }`.
    pub(super) direct: Option<SymbolId>,
    /// Forwarded meaning, e.g. `namespace N { export { value as X } }`.
    pub(super) alias: Option<TsResolvedExportAliasTarget>,
    /// Type-only export specifier encountered on the qualified/forwarding path.
    pub(super) type_only_export: Option<NodeId>,
    /// Property value meaning, e.g. `C.x` from `class C { static x = 1 }` or `C.prototype`.
    /// Includes implicit properties without a declaration symbol or member AST node.
    pub(super) property_value: PropertyValue,
}

impl TsResolvedImportTarget {
    pub(super) fn needs_value_projection(self) -> bool {
        self.alias.is_some_and(|alias| match alias.target {
            TsExportTarget::Member(_) => true,
            TsExportTarget::Symbols { public, .. } => match self.direct {
                Some(direct) => direct != public,
                None => self.type_only_export.is_some(),
            },
        })
    }

    pub(super) fn symbol(symbol: SymbolId) -> Self {
        Self {
            direct: Some(symbol),
            alias: None,
            type_only_export: None,
            property_value: PropertyValue::Absent,
        }
    }

    pub(super) fn property_value(value: PropertyValue) -> Self {
        Self {
            direct: None,
            alias: None,
            type_only_export: None,
            property_value: value,
        }
    }

    pub(super) fn non_symbol_spaces(self) -> TsResolvedAliasSpaces {
        match self.property_value {
            PropertyValue::Absent => TsResolvedAliasSpaces::empty(),
            PropertyValue::Present => TsResolvedAliasSpaces::PROPERTY_VALUE,
            PropertyValue::Unknown => TsResolvedAliasSpaces::UNKNOWN_PROPERTY_VALUE,
        }
    }

    // Declaration checks use the immediate lexical meaning of a pure alias.
    // Public lookup follows export-value bridges. Both walk the same shared graph.
    pub(super) fn contributions<'a>(
        self,
        members: &'a TsExportMembers,
        conflict: bool,
    ) -> TsExportContributions<'a> {
        TsExportContributions {
            direct: self.direct,
            target: self.alias.map(|alias| alias.target),
            members,
            conflict: conflict
                && self.direct.is_none()
                && self.property_value == PropertyValue::Absent,
            type_only_export: self.type_only_export,
        }
    }

    pub(super) fn public_contributions<'a>(
        self,
        members: &'a TsExportMembers,
    ) -> TsExportContributions<'a> {
        self.contributions(members, false)
    }

    pub(super) fn public_symbols<'a>(
        self,
        members: &'a TsExportMembers,
    ) -> impl Iterator<Item = SymbolId> + 'a {
        self.contributions(members, false)
            .map(|contribution| contribution.symbol)
    }

    pub(super) fn conflict_symbols<'a>(
        self,
        members: &'a TsExportMembers,
    ) -> impl Iterator<Item = SymbolId> + 'a {
        self.contributions(members, true)
            .map(|contribution| contribution.symbol)
    }

    pub(super) fn with_type_only_export(mut self, node: Option<NodeId>) -> Self {
        self.type_only_export = self.type_only_export.or(node);
        self
    }
}

bitflags::bitflags! {
    /// Meanings of A's target in `import A = N.X`, separate from A's declaration flags.
    /// For `namespace N { export class X {} }`, A has both TYPE and VALUE meanings.
    /// These results must not be written back into [`SymbolFlags`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) struct TsResolvedAliasSpaces: u8 {
        /// Declared runtime value, e.g. exported `const X = 1`.
        const VALUE = 1 << 0;
        /// Type declaration, e.g. exported `interface X {}`.
        const TYPE = 1 << 1;
        /// Namespace declaration, e.g. exported `namespace X {}`.
        const NAMESPACE = 1 << 2;
        /// Opaque target such as an unavailable external namespace's `External.X`.
        /// May supply any space, but proves no declaration-space conflict.
        const UNKNOWN = 1 << 3;
        /// Known property, e.g. `class C { static x = 1 }` supplies `C.x`.
        /// Usable by qualified aliases, excluded from namespace-body member lookup.
        const PROPERTY_VALUE = 1 << 4;
        /// Unknown computed property, e.g. `class C { static [key] = 1 }` may supply `C.x`.
        /// Allows possible values only; proves no existence or type/namespace meaning.
        const UNKNOWN_PROPERTY_VALUE = 1 << 5;
    }
}

impl TsResolvedAliasSpaces {
    pub(super) fn known(self) -> Self {
        self.difference(Self::UNKNOWN | Self::UNKNOWN_PROPERTY_VALUE)
    }

    pub(super) fn visibility(self, space: ReferenceSpace, flags: SymbolFlags) -> BindingVisibility {
        if self.known().visible_in(space, flags) {
            BindingVisibility::Present
        } else if self.visible_in(space, flags) {
            BindingVisibility::Unknown
        } else {
            BindingVisibility::Absent
        }
    }

    pub(super) fn may_have(self, space: Self) -> bool {
        self.intersects(space | Self::UNKNOWN)
            || (space.contains(Self::PROPERTY_VALUE) && self.contains(Self::UNKNOWN_PROPERTY_VALUE))
    }

    pub(super) fn may_have_value(self) -> bool {
        self.may_have(Self::VALUE | Self::PROPERTY_VALUE)
    }

    pub(super) fn declaration_spaces(mut self) -> Self {
        if self.contains(Self::PROPERTY_VALUE) {
            self.remove(Self::PROPERTY_VALUE);
            self.insert(Self::VALUE);
        }
        self.intersection(Self::VALUE | Self::TYPE | Self::NAMESPACE)
    }

    pub(super) fn visible_in(self, space: ReferenceSpace, flags: SymbolFlags) -> bool {
        match space {
            ReferenceSpace::Value => {
                // Type-only restricts the import contribution, not an
                // independent value declaration merged into the same symbol.
                flags.in_value_space()
                    || (flags.contains(SymbolFlags::IMPORT) && self.may_have_value())
            }
            ReferenceSpace::Typeof => self.may_have_value(),
            ReferenceSpace::Type => self.may_have(Self::TYPE),
            ReferenceSpace::Namespace => self.may_have(Self::NAMESPACE),
            ReferenceSpace::Any => true,
        }
    }
}

/// Cached value targets for references selecting a name with export-alias contributions.
/// In `export interface X {}; export { value as X }`, expression `X` can reach `value`.
/// Ordinary references use their canonical candidates without extra entries here.
#[derive(Debug)]
pub(super) struct TsReferenceValues<'a> {
    /// Reference ID -> window in `targets`; an empty window means no value target.
    pub(super) ranges: ArenaHashMap<'a, ReferenceId, Range>,
    /// Concatenated value targets for the recorded references.
    pub(super) targets: ArenaVec<'a, NameResolution>,
}

impl<'a> TsReferenceValues<'a> {
    pub(super) fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            ranges: ArenaHashMap::new_in(allocator),
            targets: ArenaVec::new_in(allocator),
        }
    }

    pub(super) fn get(&self, reference: ReferenceId) -> Option<&[NameResolution]> {
        let range = self.ranges.get(&reference)?;
        let start = range.start as usize;
        Some(&self.targets[start..start + range.len as usize])
    }

    pub(super) fn record(&mut self, reference: ReferenceId, targets: &[NameResolution]) {
        self.ranges.insert(
            reference,
            Range {
                start: self.targets.len() as u32,
                len: targets.len() as u32,
            },
        );
        self.targets.extend(targets.iter().copied());
    }
}

/// Scope -> member views used to read shared namespace/enum declarations.
/// A `namespace A.B { ... }` body can have views of both B and its enclosing A.
#[derive(Debug)]
pub(super) struct TsScopeViewIndex<'a> {
    /// Views attached to each scope, as windows in [`TsBindings::views`].
    pub(super) ranges: ArenaIndexVec<'a, ScopeId, Range>,
    /// Inherited lookup boundaries, precomputed for every scope.
    pub(super) lookup: ArenaIndexVec<'a, ScopeId, TsScopeLookup>,
}

/// Lookup rules inherited by nested scopes, e.g. a function inside `namespace N`.
#[derive(Debug, Default, Clone, Copy)]
pub(super) struct TsScopeLookup {
    /// Nearest scope with member views, including self; bounds the lexical-only search.
    pub(super) members: Option<ScopeId>,
    /// A TS signature/conditional-type scope on the ancestor path restricts lexical lookup.
    pub(super) restricted_lexical: bool,
}

/// A TypeScript source module's exports, e.g. `export class C {}` and `export default class {}`.
/// Named exports use the common member table; default declarations use its `"default"` key.
#[derive(Debug)]
pub(super) struct TsSourceExports<'a> {
    /// Synthetic symbol owning the file's export table.
    pub(super) owner: SymbolId,
    pub(super) environment: TsMemberEnvironmentId,
    /// Ambient source context where declarations are exported without an `export` modifier.
    pub(super) implicit_exports: bool,
    /// Default class/function/interface declarations; excludes `export default expression`.
    pub(super) default: Option<TsDefaultExport<'a>>,
}

/// Shared `"default"` declaration identity, including anonymous `export default class {}`.
#[derive(Debug)]
pub(super) struct TsDefaultExport<'a> {
    pub(super) symbol: SymbolId,
    /// Local name from `export default class C {}`; absent for anonymous declarations.
    /// The first named function takes precedence over a previously recorded non-function.
    pub(super) first_local: Option<&'a [u8]>,
    /// Whether `first_local` came from `export default function f() {}`.
    pub(super) first_local_is_function: bool,
}

/// A body's local-name link to an exported declaration, e.g. C in `export class C {}`.
#[derive(Debug, Clone, Copy)]
pub(super) struct TsExportLocal {
    /// Exported declaration selected for this body's local name.
    pub(super) target: SymbolId,
    /// Whether that declaration is a function; preserves TS's function-first binding order.
    pub(super) target_is_function: bool,
    /// At least one export declaration under this local name supplies a runtime value.
    pub(super) exported_value: bool,
    /// Default-export declaration facts used when checking the local name for conflicts.
    pub(super) default_declarations: Option<TsDefaultLocal>,
}

/// Local declaration facts for named defaults such as `export default class C {}`.
#[derive(Debug, Clone, Copy)]
pub(super) struct TsDefaultLocal {
    pub(super) node: NodeId,
    pub(super) flags: SymbolFlags,
}

/// `(body scope, local name)` -> exported declaration link.
/// For `namespace N { export class C {} }`, the key uses that body's scope and `"C"`.
/// A second `namespace N { ... }` shares N's public table, but does not inherit this local link.
pub(super) type TsLocalExportMap<'a> = ArenaHashMap<'a, (ScopeId, &'a [u8]), TsExportLocal>;

impl TsExportLocal {
    pub(super) fn exported_value(self) -> Option<SymbolId> {
        self.exported_value.then_some(self.target)
    }
}

pub(super) fn ts_exported_value(
    locals: &TsLocalExportMap<'_>,
    symbols: &Symbols,
    declarations: &Declarations,
    scope_maps: &ScopeMaps<'_>,
    lexical: SymbolId,
    name: &[u8],
) -> Option<SymbolId> {
    let first = symbols[lexical].declaration;
    let scope = declarations[first].scope;
    let target = locals.get(&(scope, name))?.exported_value()?;
    (scope_maps[scope].get(name).copied() == Some(lexical)).then_some(target)
}

/// Read-only lookup of value meaning: `X` from `export { value as X }`, or `typeof A`.
/// Follows local export links and import-alias spaces while preserving binding identities.
pub(super) struct TsValueTargetLookup<'lookup, 'arena> {
    pub(super) locals: &'lookup TsLocalExportMap<'arena>,
    pub(super) symbols: &'lookup Symbols,
    pub(super) declarations: &'lookup Declarations,
    pub(super) scope_maps: &'lookup ScopeMaps<'arena>,
    pub(super) import_alias_spaces: &'lookup TsImportAliasSpaces,
}

impl TsValueTargetLookup<'_, '_> {
    fn value(
        &self,
        symbol: SymbolId,
        name: &[u8],
        space: ReferenceSpace,
        uncertain: bool,
    ) -> Option<NameResolution> {
        match self.target::<true>(symbol, name, space) {
            NameResolution::Unresolved => None,
            NameResolution::Resolved(candidate) if uncertain => {
                Some(NameResolution::Uncertain { candidate })
            }
            target => Some(target),
        }
    }

    pub(super) fn reference_flags<const PRECISE_IMPORT_ALIASES: bool>(
        &self,
        tree: &Ast<'_>,
        reference: Reference,
        resolved: NameResolution,
    ) -> ReferenceFlags {
        debug_assert_eq!(reference.flags.space, ReferenceSpace::Any);
        let Some(symbol) = resolved.candidate() else {
            return reference.flags;
        };
        let name = tree.get_utf8(reference.name).as_bytes();
        let target = self.target::<PRECISE_IMPORT_ALIASES>(symbol, name, ReferenceSpace::Value);
        if target.candidate().is_some() {
            reference.flags
        } else {
            reference.flags.no_access()
        }
    }

    pub(super) fn target<const PRECISE_IMPORT_ALIASES: bool>(
        &self,
        symbol: SymbolId,
        name: &[u8],
        space: ReferenceSpace,
    ) -> NameResolution {
        debug_assert!(matches!(
            space,
            ReferenceSpace::Value | ReferenceSpace::Typeof
        ));
        let symbol = ts_exported_value(
            self.locals,
            self.symbols,
            self.declarations,
            self.scope_maps,
            symbol,
            name,
        )
        .unwrap_or(symbol);
        match symbol_visibility::<PRECISE_IMPORT_ALIASES>(
            self.symbols,
            self.import_alias_spaces,
            symbol,
            space,
        ) {
            BindingVisibility::Absent => NameResolution::Unresolved,
            BindingVisibility::Present => NameResolution::Resolved(symbol),
            BindingVisibility::Unknown => NameResolution::Uncertain { candidate: symbol },
        }
    }
}

impl TsSourceExports<'_> {
    pub(super) fn lookup(
        &self,
        name: &[u8],
        space: ReferenceSpace,
        symbols: &Symbols,
        locals: &TsLocalExportMap<'_>,
    ) -> Option<SymbolId> {
        if matches!(space, ReferenceSpace::Value | ReferenceSpace::Typeof)
            && let Some(symbol) = locals
                .get(&(ScopeId::MODULE, name))
                .and_then(|local| local.exported_value())
            && symbols[symbol].flags.visible_in(space)
        {
            return Some(symbol);
        }
        let default = self.default.as_ref()?;
        (default.first_local == Some(name) && symbols[default.symbol].flags.visible_in(space))
            .then_some(default.symbol)
    }
}

/// Current namespace/enum/module body while walking its declarations.
#[derive(Clone, Copy)]
pub(super) enum TsContainerContext {
    /// An external module body for which this analyzer did not establish member ownership.
    UnsupportedExternalModule {
        /// Module body AST node, absent for a bodyless ambient declaration.
        body: Option<NodeId>,
    },
    Member {
        /// Shared owner table, e.g. N's table for `namespace N { ... }`.
        environment: TsMemberEnvironmentId,
        /// This body's lookup views, e.g. B then A for `namespace A.B { ... }`.
        views: Range,
        /// Scope receiving projected members, e.g. ROOT for `declare global { ... }`.
        projection_scope: Option<ScopeId>,
        /// Body AST node; absent for a bodyless declaration such as `declare module "pkg";`.
        body: Option<NodeId>,
        /// Lexical scope of this particular body, distinct from the shared owner table.
        body_scope: Option<ScopeId>,
        /// Ambient declarations can become members without an explicit `export` modifier.
        implicit_exports: bool,
        /// Whether local export specifiers contribute to this body's public table.
        public_exports: bool,
    },
}

impl TsContainerContext {
    fn body(self) -> Option<NodeId> {
        match self {
            Self::UnsupportedExternalModule { body } | Self::Member { body, .. } => body,
        }
    }
}

/// Deferred local-name lookup for `export { X as Y }` in a namespace or source module.
#[derive(Clone, Copy)]
pub(super) struct PendingTsNamespaceExportAlias<'a> {
    /// Table receiving the exported name Y.
    pub(super) environment: TsMemberEnvironmentId,
    /// `"Y"` in `export { X as Y }`.
    pub(super) exported_name: &'a [u8],
    /// Reference to X, resolved after all bodies and exports have been collected.
    pub(super) local_reference: ReferenceId,
    /// Earlier export specifier for Y, when this is a duplicate.
    pub(super) previous: Option<NodeId>,
}

/// Deferred target lookup for internal `import A = N.X`; excludes `import A = require("pkg")`.
#[derive(Clone, Copy)]
pub(super) struct TsImportAliasTarget {
    /// The new local binding A, whose identity survives target resolution.
    pub(super) alias: SymbolId,
    /// The complete `import A = N.X` declaration.
    pub(super) declaration: TsImportEqualsDeclaration,
    /// N's reference in `N.X`; identifier-name segments such as X have no reference ID.
    pub(super) root_reference: ReferenceId,
}

/// Diagnostics from local `export { X as Y }` and internal `import A = N.X` resolution.
#[derive(Clone, Copy)]
pub(crate) enum TsAliasIssue {
    MissingExport {
        name: AstUtf8,
        node: NodeId,
    },
    NonLocalExport {
        name: AstUtf8,
        node: NodeId,
    },
    ExportConflict {
        node: NodeId,
        previous: NodeId,
        duplicate: bool,
    },
    ImportConflict {
        name: AstWtf8,
        node: NodeId,
        previous: NodeId,
        previous_flags: SymbolFlags,
    },
    TypeOnlyExport {
        node: NodeId,
        export: NodeId,
    },
}

bitflags::bitflags! {
    /// TypeScript checker declaration spaces used when validating that merged
    /// declarations are either all exported or all local. These are narrower
    /// than reference/export capabilities: in particular, an unresolved
    /// import does not claim every space merely because it can be referenced
    /// in every space.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) struct TsDeclarationSpaces: u8 {
        const VALUE = 1 << 0;
        const TYPE = 1 << 1;
        const NAMESPACE = 1 << 2;
    }
}

/// Build-only records for TS syntax; completed queries retain [`TsBindings`] instead.
pub(super) struct TsBuildState<'a> {
    /// Class/interface occurrence scope -> merged owner symbol for `<T>` parameters.
    pub(super) type_parameter_owners: ArenaHashMap<'a, ScopeId, SymbolId>,
    /// Merged owner -> parameter-name table, e.g. C -> `"T"` in `class C<T> {}`.
    pub(super) type_parameters: ArenaHashMap<'a, SymbolId, ScopeMap<'a>>,
    /// Shared owner of `declare global { ... }` declarations.
    pub(super) global_owner: Option<SymbolId>,
    /// Body scope -> window in `ts.views`, collected before building the dense scope index.
    pub(super) scope_views: ArenaVec<'a, (ScopeId, Range)>,
    /// Pending `export { X as Y }` local-name lookups.
    pub(super) export_aliases: ArenaVec<'a, PendingTsNamespaceExportAlias<'a>>,
    /// Pending internal `import A = N.X` target lookups.
    pub(super) import_aliases: ArenaVec<'a, TsImportAliasTarget>,
    /// Property assignments such as `function f() {} f.x = 1`, with their owning context.
    pub(super) expando_assignments: ArenaVec<'a, (MemberExpression, TsExpandoBindingSite)>,
    /// Nested namespace/enum/module bodies currently being visited.
    pub(super) containers: Vec<TsContainerContext>,
    /// Cached namespace-body state: type-only, const-enum-only, or runtime-instantiated.
    pub(super) module_instances: HashMap<NodeId, TsModuleInstanceMemo, FxBuildHasher>,
    /// Enum owner symbols while visiting `enum E { A, B = A }` initializers.
    pub(super) enums: Vec<SymbolId>,
}

impl<'a> TsBuildState<'a> {
    pub(super) fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            type_parameter_owners: ArenaHashMap::new_in(allocator),
            type_parameters: ArenaHashMap::new_in(allocator),
            global_owner: None,
            scope_views: ArenaVec::new_in(allocator),
            export_aliases: ArenaVec::new_in(allocator),
            import_aliases: ArenaVec::new_in(allocator),
            expando_assignments: ArenaVec::new_in(allocator),
            containers: Vec::new(),
            module_instances: HashMap::with_hasher(FxBuildHasher),
            enums: Vec::new(),
        }
    }
}

/// Completes `export { X as Y }` records in place, following X across merged bodies.
/// A pending inner export suspends lookup instead of exposing an outer namesake.
struct TsExportAliasResolver<'tracker, 'a> {
    tracker: &'tracker SymbolTracker<'a>,
    tree: &'tracker Ast<'a>,
    scopes: &'tracker Scopes,
    scope_views: Option<&'tracker TsScopeViewIndex<'a>>,
    /// Export specifiers being resolved; returned to the member tables when complete.
    aliases: ArenaIndexVec<'a, TsExportAliasId, TsExportAlias>,
    /// Export-alias ID -> shared combination node for a direct-plus-alias name.
    member_targets: Vec<Option<TsExportMemberId>>,
    /// Combination nodes reused by forwarding names such as Y and Z.
    members: ArenaIndexVec<'a, TsExportMemberId, TsExportMember>,
}

/// Dependency results for `import A = N.X; import B = A`: B inherits A's meanings.
struct TsImportAliasSpaceGraph {
    /// Resolved target per import-equals declaration, in pending-declaration order.
    /// Shared by propagation, diagnostics and finalization.
    targets: Vec<Option<TsResolvedImportTarget>>,
    /// Public meanings per distinct alias symbol, indexed by [`TsImportAliasInfo::index`].
    public: Vec<TsResolvedAliasSpaces>,
    /// Declaration-conflict meanings using the same node indexes; computed only on demand.
    conflict: Vec<TsResolvedAliasSpaces>,
}

/// One distinct local A in `import A = N.X`, together with its dependency-graph index.
#[derive(Clone, Copy)]
struct TsImportAliasInfo {
    target: TsImportAliasTarget,
    index: usize,
}

/// Build-time DFS state for an internal `import A = N.X` target.
#[derive(Clone, Copy)]
enum TsImportTargetState {
    Pending,
    Resolving,
    Resolved {
        /// Full meaning of N.X, retained even when it has no namespace meaning.
        target: Option<TsResolvedImportTarget>,
        /// Cached namespace interpretation when A is used in a longer path such as A.Y.
        namespace: TsNamespaceTargetResolution,
    },
}

/// Whether an import target supplies namespace meaning for a qualified-name prefix.
#[derive(Clone, Copy)]
enum TsNamespaceTargetResolution {
    /// Known namespace owner; retain all of its public export contributions.
    Resolved(TsResolvedImportTarget),
    /// Opaque or cyclic alias; no namespace owner can be established.
    Unknown,
    /// No namespace meaning; separate property lookup may still supply class/function members.
    NotNamespace,
}

impl TsNamespaceTargetResolution {
    pub(super) fn target(self) -> Option<TsResolvedImportTarget> {
        match self {
            Self::Resolved(target) => Some(target),
            Self::Unknown | Self::NotNamespace => None,
        }
    }
}

/// Resolves internal `import A = N.X` declarations after exports are known.
/// Target/property facts are shared by diagnostics and final indexes; DFS and
/// propagation scratch are discarded after building [`TsBindings`].
struct TsImportAliasResolver<'tracker, 'a> {
    tracker: &'tracker SymbolTracker<'a>,
    tree: &'tracker Ast<'a>,
    scopes: &'tracker Scopes,
    scope_views: Option<&'tracker TsScopeViewIndex<'a>>,
    /// A's local symbol -> declaration and distinct-alias graph index.
    aliases: HashMap<SymbolId, TsImportAliasInfo, FxBuildHasher>,
    /// Target-resolution DFS states indexed by distinct-alias graph index.
    alias_targets: Vec<TsImportTargetState>,
    /// Known/possible values of class/function properties such as `C.x` and `f.x`.
    property_values: TsPropertyValues<'tracker>,
    space_graph: TsImportAliasSpaceGraph,
}

/// Ownership context of an expando assignment such as `function f() {} f.x = 1`.
/// TS checks the enclosing block and declaration container for the receiver f.
/// A container without locals (for example an object literal) is a barrier.
#[derive(Clone, Copy)]
pub(super) struct TsExpandoBindingSite {
    /// Enclosing declaration container, e.g. source file, function or namespace body.
    pub(super) container: Option<ScopeId>,
    /// Nearest enclosing block scope, falling back to `container` when there is none.
    pub(super) block: Option<ScopeId>,
}

/// Whether syntax proves a requested class/function property exists, e.g. `C.x`.
/// Unknown computed names can add possible values, never type/namespace meaning.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum PropertyValue {
    #[default]
    /// No collected own property supplies this name; this does not check inherited properties.
    Absent,
    /// Known name, e.g. `class C { static x = 1 }` supplies `C.x`.
    Present,
    /// Unresolved computed name, e.g. `class C { static [key] = 1 }` may supply `C.x`.
    Unknown,
}

/// Class/function owner -> own property names from static members and expando assignments.
/// `class C { static x = 1 }` records C -> `"x"`; classes also supply `"prototype"`.
#[derive(Default)]
struct TsPropertyValues<'tree> {
    owners: HashMap<SymbolId, PropertyNames<'tree>, FxBuildHasher>,
}

/// Property names belonging to one owner, e.g. the static members of class C.
#[derive(Default)]
struct PropertyNames<'tree> {
    /// Syntactically known names such as `"x"` from `static x` or `f.x = 1`.
    known: HashSet<Cow<'tree, [u8]>, FxBuildHasher>,
    /// At least one eligible computed name could not be resolved, e.g. `static [key]`.
    unknown: bool,
}

/// Resolves assignment receivers such as f in `f.x = 1` to their declaration owner.
struct TsMemberFactsLookup<'lookup, 'arena> {
    tracker: &'lookup SymbolTracker<'arena>,
    scopes: &'lookup Scopes,
    scope_views: Option<&'lookup TsScopeViewIndex<'arena>>,
}

impl<'tree> TsPropertyValues<'tree> {
    pub(super) fn record(&mut self, owner: SymbolId, name: Option<Cow<'tree, [u8]>>) {
        let members = self.owners.entry(owner).or_default();
        if let Some(name) = name {
            members.known.insert(name);
        } else {
            members.unknown = true;
        }
    }

    pub(super) fn get(&self, owner: SymbolId, name: &[u8]) -> PropertyValue {
        let Some(members) = self.owners.get(&owner) else {
            return PropertyValue::Absent;
        };
        if members.known.contains(name) {
            PropertyValue::Present
        } else if members.unknown {
            PropertyValue::Unknown
        } else {
            PropertyValue::Absent
        }
    }
}

/// TypeScript's final module-instance states. `ConstEnumOnly` has value
/// meaning during binding, while remaining distinguishable from a namespace
/// that necessarily emits a runtime object.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum TsModuleInstanceState {
    NonInstantiated,
    ConstEnumOnly,
    Instantiated,
}

impl TsModuleInstanceState {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Instantiated, _) | (_, Self::Instantiated) => Self::Instantiated,
            (Self::ConstEnumOnly, _) | (_, Self::ConstEnumOnly) => Self::ConstEnumOnly,
            (Self::NonInstantiated, Self::NonInstantiated) => Self::NonInstantiated,
        }
    }
}

/// Computes the same three final states used by TypeScript's
/// `getModuleInstanceState`. The traversal is syntactic because the state is
/// needed while the module symbol itself is being bound. Local export
/// specifiers search the current and enclosing module blocks, matching the
/// binder's forward-reference behavior.
struct TsModuleInstanceAnalyzer<'tree, 'ast, 'scratch> {
    tree: &'tree Ast<'ast>,
    module_states: &'scratch mut HashMap<NodeId, TsModuleInstanceMemo, FxBuildHasher>,
}

#[derive(Clone, Copy)]
pub(super) enum TsModuleInstanceMemo {
    Computing,
    Complete(TsModuleInstanceState),
}

struct TsModuleBlockPath<'a> {
    node: NodeId,
    parent: Option<&'a TsModuleBlockPath<'a>>,
}

impl<'a> SymbolTracker<'a> {
    pub(super) fn init_ts_source_exports(&mut self, tree: &Ast<'a>) {
        let program = tree.root_program();
        let (owner, _) = self.declare_symbol(
            tree,
            PendingDeclaration {
                name: tree.empty_wtf8(),
                node: program.node_id(),
                binding: PendingBinding {
                    flags: SymbolFlags::VALUE_MODULE | SymbolFlags::NAMESPACE_MODULE,
                    excludes: SymbolFlags::empty(),
                    scope: ScopeId::ROOT,
                },
            },
            None,
            false,
        );
        let environment = self.ts.environments.ensure(owner, self.allocator);
        let start = self.ts.views.len();
        self.ts.views.push(TsMemberView {
            environment,
            kind: TsMemberViewKind::SourceFile,
        });
        self.record_ts_scope_views(
            ScopeId::MODULE,
            Range {
                start: start as u32,
                len: 1,
            },
        );
        let implicit_exports = self.ambient
            && !program.body(tree).iter().any(|statement| {
                let statement = tree.get_node_in_sub_range(statement).node_id();
                ts_statement_is_export_declaration(tree, statement)
            });
        self.ts.source_exports = Some(TsSourceExports {
            owner,
            environment,
            implicit_exports,
            default: None,
        });
    }

    pub fn enter_ts_function(
        &mut self,
        tree: &Ast<'a>,
        function: TsFunction,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        self.push_saved_context();
        let target = scope
            .get(current_scope)
            .parent
            .expect("a TypeScript function scope has a parent");
        self.pending = PendingBinding {
            flags: SymbolFlags::FUNCTION | SymbolFlags::AMBIENT,
            excludes: Excludes::FUNCTION,
            scope: target,
        };
        if function.id(tree).is_none() {
            self.declare_anonymous_default(tree, function.node_id());
        }
    }

    pub fn enter_ts_module_block(&mut self, current_scope: ScopeId) {
        self.export_state = ExportState::None;
        let context = self
            .ts_build
            .containers
            .last_mut()
            .expect("a TypeScript module block has a context");
        let TsContainerContext::Member {
            views, body_scope, ..
        } = context
        else {
            return;
        };
        *body_scope = Some(current_scope);
        let views = *views;
        self.record_ts_scope_views(current_scope, views);
    }

    pub fn enter_ts_enum_body(&mut self, current_scope: ScopeId) {
        self.export_state = ExportState::None;
        let owner = *self
            .ts_build
            .enums
            .last()
            .expect("an enum body has an owner");
        let environment = self.ts.environments.ensure(owner, self.allocator);
        let start = self.ts.views.len();
        self.ts.views.push(TsMemberView {
            environment,
            kind: TsMemberViewKind::Enum,
        });
        self.ts_build.scope_views.push((
            current_scope,
            Range {
                start: start as u32,
                len: 1,
            },
        ));
    }

    pub fn enter_ts_enum_members(&mut self, owner: SymbolId) {
        self.ts_build.enums.push(owner);
    }

    pub fn exit_ts_enum_members(&mut self, owner: SymbolId) {
        let exited = self.ts_build.enums.pop();
        debug_assert_eq!(exited, Some(owner));
    }

    pub fn enter_ts_module_body(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsModuleDeclaration,
    ) -> Vec<PreparedDeclaration> {
        let name = declaration.id(tree);
        let (declarations, views) = self.prepare_ts_module_views(tree, name);
        let body = declaration.body(tree).map(|body| body.node_id());
        let context = if views.len == 0 {
            TsContainerContext::UnsupportedExternalModule { body }
        } else {
            // The body's own environment precedes qualified-name ancestors.
            let environment = self.ts.views[views.start as usize].environment;
            TsContainerContext::Member {
                environment,
                views,
                projection_scope: None,
                body,
                body_scope: None,
                implicit_exports: self.ambient
                    && !ts_module_has_export_declarations(tree, declaration.body(tree)),
                public_exports: true,
            }
        };
        self.ts_build.containers.push(context);
        declarations
    }

    pub fn exit_ts_module_body(&mut self) {
        self.ts_build
            .containers
            .pop()
            .expect("a TypeScript module body owner exists");
    }

    pub fn enter_ts_import_equals_declaration(
        &mut self,
        import_kind: ImportOrExportKind,
        current_scope: ScopeId,
    ) {
        self.push_saved_context();
        self.pending = import_pending(import_kind, current_scope);
        self.pending.flags |= SymbolFlags::IMPORT_EQUALS;
    }

    pub fn enter_ts_interface_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsInterfaceDeclaration,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        // The id binds in the surrounding scope. Type-parameter and
        // body bindings live in a scope pushed by the builder.
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags::INTERFACE.with(
                SymbolFlags::AMBIENT,
                declaration.declare(tree) || self.ambient,
            ),
            excludes: Excludes::INTERFACE,
            scope: scope
                .get(current_scope)
                .parent
                .expect("an interface scope has a parent"),
        };
    }

    pub fn enter_ts_type_alias_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsTypeAliasDeclaration,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags::TYPE_ALIAS.with(
                SymbolFlags::AMBIENT,
                declaration.declare(tree) || self.ambient,
            ),
            excludes: Excludes::TYPE_ALIAS,
            scope: scope
                .get(current_scope)
                .parent
                .expect("a type alias scope has a parent"),
        };
    }

    pub fn enter_ts_enum_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsEnumDeclaration,
        current_scope: ScopeId,
    ) {
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: if declaration.is_const(tree) {
                SymbolFlags::CONST_ENUM
            } else {
                SymbolFlags::REGULAR_ENUM
            }
            .with(
                SymbolFlags::AMBIENT,
                declaration.declare(tree) || self.ambient,
            ),
            excludes: if declaration.is_const(tree) {
                Excludes::CONST_ENUM
            } else {
                Excludes::REGULAR_ENUM
            },
            scope: current_scope,
        };
    }

    pub fn enter_ts_module_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsModuleDeclaration,
        current_scope: ScopeId,
    ) {
        self.push_saved_context();
        if matches!(
            tree.ts_module_name_data(declaration.id(tree)),
            TsModuleNameData::StringLiteral(_)
        ) {
            if declaration.declare(tree) {
                self.ambient = true;
            }
            return;
        }
        let state = ts_module_instance_state(
            tree,
            declaration,
            &self.ts_build.containers,
            &mut self.ts_build.module_instances,
        );
        let instantiated = state != TsModuleInstanceState::NonInstantiated;
        let mut flags = SymbolFlags::NAMESPACE_MODULE.with(SymbolFlags::VALUE_MODULE, instantiated);
        flags.set(
            SymbolFlags::CONST_ENUM_ONLY_MODULE,
            state == TsModuleInstanceState::ConstEnumOnly,
        );
        self.pending = PendingBinding {
            flags: flags.with(
                SymbolFlags::AMBIENT,
                declaration.declare(tree) || self.ambient,
            ),
            excludes: if instantiated {
                Excludes::VALUE_MODULE
            } else {
                Excludes::NAMESPACE_MODULE
            },
            scope: current_scope,
        };
        // `declare` makes the whole body ambient.
        if declaration.declare(tree) {
            self.ambient = true;
        }
    }

    pub fn enter_ts_global_declaration(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsGlobalDeclaration,
    ) {
        self.push_saved_context();
        let identifier = declaration.id(tree);
        let existing = self.ts_build.global_owner;
        let (owner, _) = self.declare_symbol(
            tree,
            PendingDeclaration {
                name: identifier.name(tree).into(),
                node: identifier.node_id(),
                binding: PendingBinding {
                    flags: SymbolFlags::NAMESPACE_MODULE | SymbolFlags::AMBIENT,
                    excludes: Excludes::NAMESPACE_MODULE,
                    scope: ScopeId::ROOT,
                },
            },
            existing,
            true,
        );
        let views = self.append_ts_namespace_views([owner], TsMemberViewKind::Module);
        if existing.is_none() {
            self.ts_build.global_owner = Some(owner);
        }
        self.ambient = true;
        let environment = self.ts.views[views.start as usize].environment;
        self.ts_build.containers.push(TsContainerContext::Member {
            environment,
            views,
            projection_scope: Some(ScopeId::ROOT),
            body: Some(declaration.body(tree).node_id()),
            body_scope: None,
            implicit_exports: true,
            public_exports: false,
        });
    }

    pub fn exit_ts_global_declaration(&mut self) {
        self.ts_build
            .containers
            .pop()
            .expect("a TypeScript global body context exists");
        self.exit_saved_context();
    }

    pub fn enter_ts_type_parameter(
        &mut self,
        tree: &Ast<'a>,
        parent: Option<NodeId>,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        // `TsTypeParameter` wraps `<T>` and `infer U`.
        self.push_saved_context();
        // `infer` declares in the enclosing conditional scope so the
        // true branch sees it, even from inside a signature type.
        let target = if parent.is_some_and(|parent| tree.node_kind(parent) == NodeKind::TsInferType)
        {
            infer_owner(scope, current_scope)
        } else if scope.get(current_scope).kind == ScopeKind::TsTypeParameters {
            scope
                .get(current_scope)
                .parent
                .expect("type parameters have an owning declaration scope")
        } else {
            current_scope
        };
        self.pending = PendingBinding {
            flags: SymbolFlags::TYPE_PARAMETER,
            excludes: Excludes::TYPE_PARAMETER,
            scope: target,
        };
        // TS7 stores class/interface parameters in the merged owner's Members,
        // but only exposes parameters declared by the current occurrence. Keep
        // that identity index separate from both lexical scopes and exports.
        let owner = match tree.kind_data(scope.get(target).node) {
            NodeKindData::Class(class) if class.class_type(tree) == ClassType::ClassDeclaration => {
                if let Some(id) = class.id(tree) {
                    self.symbol_of_node(id.node_id())
                } else {
                    self.default_export_symbol()
                }
            }
            NodeKindData::TsInterfaceDeclaration(interface) => {
                self.symbol_of_node(interface.id(tree).node_id())
            }
            _ => None,
        };
        if let Some(owner) = owner {
            self.ts_build.type_parameter_owners.insert(target, owner);
        }
        self.export_state = ExportState::None;
    }

    pub fn enter_ts_mapped_type(
        &mut self,
        tree: &Ast<'a>,
        parent: Option<NodeId>,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        // Mapped type keys `[K in T]` are bare binding identifiers, so the
        // same pending context as a type parameter applies.
        self.push_saved_context();
        let target = if parent.is_some_and(|parent| tree.node_kind(parent) == NodeKind::TsInferType)
        {
            infer_owner(scope, current_scope)
        } else {
            current_scope
        };
        self.pending = PendingBinding {
            flags: SymbolFlags::TYPE_PARAMETER,
            excludes: Excludes::TYPE_PARAMETER,
            scope: target,
        };
        self.export_state = ExportState::None;
    }

    pub fn enter_ts_function_type(&mut self, current_scope: ScopeId) {
        self.enter_signature_type(current_scope);
    }

    pub fn enter_ts_constructor_type(&mut self, current_scope: ScopeId) {
        self.enter_signature_type(current_scope);
    }

    pub fn enter_ts_method_signature(&mut self, current_scope: ScopeId) {
        self.enter_signature_type(current_scope);
    }

    pub fn enter_ts_call_signature_declaration(&mut self, current_scope: ScopeId) {
        self.enter_signature_type(current_scope);
    }

    pub fn enter_ts_construct_signature_declaration(&mut self, current_scope: ScopeId) {
        self.enter_signature_type(current_scope);
    }

    pub fn enter_ts_index_signature(&mut self, current_scope: ScopeId) {
        self.enter_signature_type(current_scope);
    }

    fn enter_signature_type(&mut self, current_scope: ScopeId) {
        // Erasure does not remove signature parameters from the value namespace.
        // FormalParameters uses the same role; index signatures bind directly.
        self.push_saved_context();
        self.pending = PendingBinding {
            flags: SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
            excludes: Excludes::PARAMETER,
            scope: current_scope,
        };
        self.export_state = ExportState::None;
    }

    pub(crate) fn prepare_ts_enum_member(
        &mut self,
        tree: &Ast<'a>,
        member: TsEnumMember,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) -> Option<PreparedDeclaration> {
        self.sync_scope_maps_from(scope);
        // Members are lexically visible inside the enum body:
        //
        //   enum E { a, b = a }
        //   //              ^ resolves to the member
        //
        // Member names are not BindingIdentifiers, so they declare here.
        // Literal computed names denote the same member as their unwrapped
        // string or no-substitution template form.
        let member_name = member.id(tree).node_id();
        let name = text_member_name(tree, member_name, false)?;
        let owner = *self
            .ts_build
            .enums
            .last()
            .expect("an enum member has an owner");
        let flags = SymbolFlags::ENUM_MEMBER.with(SymbolFlags::AMBIENT, self.ambient);
        let environment = self.ts.environments.ensure(owner, self.allocator);
        let name_text = tree.get_wtf8(name).as_bytes();
        let shared = self.ts.environments.member(environment, name_text);
        let local = self.own_binding(current_scope, name_text);
        let prepared = PreparedDeclaration {
            declaration: PendingDeclaration {
                name,
                node: member.node_id(),
                binding: PendingBinding {
                    flags,
                    excludes: Excludes::ENUM_MEMBER,
                    scope: current_scope,
                },
            },
            destination: DeclarationDestination::TsMember {
                environment,
                body_scope: Some(current_scope),
                projection_scope: None,
            },
            local,
            shared,
            existing: shared,
            default_local: None,
        };
        Some(prepared)
    }

    pub(crate) fn declare_prepared_ts_enum_member(
        &mut self,
        tree: &Ast<'a>,
        prepared: PreparedDeclaration,
    ) {
        debug_assert!(prepared.flags().contains(SymbolFlags::ENUM_MEMBER));
        self.declare_prepared(tree, prepared);
    }

    fn record_ts_scope_views(&mut self, scope: ScopeId, views: Range) {
        self.ts_build.scope_views.push((scope, views));
    }

    fn append_ts_namespace_views(
        &mut self,
        owners: impl IntoIterator<Item = SymbolId>,
        kind: TsMemberViewKind,
    ) -> Range {
        let start = self.ts.views.len();
        for owner in owners {
            let environment = self.ts.environments.ensure(owner, self.allocator);
            self.ts.views.push(TsMemberView { environment, kind });
        }
        Range {
            start: start as u32,
            len: (self.ts.views.len() - start) as u32,
        }
    }

    #[inline(always)]
    pub(super) fn pending_ts_namespace_destination(
        &self,
    ) -> Option<(TsMemberEnvironmentId, Option<ScopeId>)> {
        let TsContainerContext::Member {
            environment,
            projection_scope,
            body_scope,
            implicit_exports,
            ..
        } = *self.ts_build.containers.last()?
        else {
            return None;
        };
        let implicitly_exported = implicit_exports
            && body_scope == Some(self.pending.scope)
            && !self.pending.flags.intersects(SymbolFlags::ANY_IMPORT);
        (self.export_state.is_exported() || implicitly_exported)
            .then_some((environment, projection_scope))
    }

    pub(super) fn pending_ts_source_destination(&self) -> Option<TsMemberEnvironmentId> {
        if self.pending.scope != ScopeId::MODULE {
            return None;
        }
        let source = self.ts.source_exports.as_ref()?;
        let implicit =
            source.implicit_exports && !self.pending.flags.intersects(SymbolFlags::ANY_IMPORT);
        (self.export_state.is_exported() || implicit).then_some(source.environment)
    }

    pub(super) fn current_ts_namespace_export_environment(
        &self,
        scope: ScopeId,
    ) -> Option<TsMemberEnvironmentId> {
        if scope == ScopeId::MODULE
            && let Some(source) = &self.ts.source_exports
        {
            return Some(source.environment);
        }
        let TsContainerContext::Member {
            environment,
            body_scope,
            public_exports,
            ..
        } = *self.ts_build.containers.last()?
        else {
            return None;
        };
        (public_exports && body_scope == Some(scope)).then_some(environment)
    }

    #[inline(always)]
    pub(super) fn current_ts_body_member(&self, scope: ScopeId, name: &[u8]) -> Option<SymbolId> {
        if scope == ScopeId::MODULE
            && let Some(source) = &self.ts.source_exports
        {
            return self.ts.environments.member(source.environment, name);
        }
        let TsContainerContext::Member {
            environment,
            body_scope,
            ..
        } = *self.ts_build.containers.last()?
        else {
            return None;
        };
        if body_scope != Some(scope) {
            return None;
        }
        let member = self.ts.environments.member(environment, name)?;
        self.symbol_declared_in_scope(member, scope)
            .then_some(member)
    }

    #[inline(always)]
    fn prepare_ts_module_views(
        &mut self,
        tree: &Ast<'a>,
        name: TsModuleName,
    ) -> (Vec<PreparedDeclaration>, Range) {
        let kind = if self.ambient {
            TsMemberViewKind::AmbientModule
        } else {
            TsMemberViewKind::Module
        };
        match tree.ts_module_name_data(name) {
            TsModuleNameData::BindingIdentifier(identifier) => {
                let owner = self.node_semantic[identifier.node_id()].symbol();
                let views = owner.map_or_else(Range::default, |owner| {
                    self.append_ts_namespace_views([owner], kind)
                });
                (Vec::new(), views)
            }
            TsModuleNameData::StringLiteral(_) => (Vec::new(), Range::default()),
            TsModuleNameData::TsQualifiedName(qualified) => {
                let mut members = Vec::new();
                let Some(root) = collect_ts_module_path(tree, qualified, &mut members) else {
                    return (Vec::new(), Range::default());
                };
                let Some(mut owner) = self.node_semantic[root].symbol() else {
                    return (Vec::new(), Range::default());
                };
                let mut owners = Vec::with_capacity(members.len() + 1);
                owners.push(owner);
                let mut declarations = Vec::new();
                for (name, node) in members {
                    let parent = owner;
                    let mut flags = self.pending.flags;
                    flags.insert(SymbolFlags::EXPORTED);
                    let declaration = PendingDeclaration {
                        name,
                        node,
                        binding: PendingBinding {
                            flags,
                            ..self.pending
                        },
                    };
                    let (next_owner, prepared) =
                        self.declare_ts_namespace_member(tree, parent, declaration, None);
                    owner = next_owner;
                    if prepared.shared.is_some() {
                        declarations.push(prepared);
                    }
                    owners.push(owner);
                }
                let views = self.append_ts_namespace_views(owners.into_iter().rev(), kind);
                (declarations, views)
            }
        }
    }

    pub fn record_ts_global_export(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsNamespaceExportDeclaration,
        current_scope: ScopeId,
    ) {
        if current_scope != ScopeId::MODULE || tree.lang() != swc_next_ecma_ast::Lang::Dts {
            return;
        }
        let identifier = declaration.id(tree);
        self.declare_symbol(
            tree,
            PendingDeclaration {
                name: identifier.name(tree).into(),
                node: identifier.node_id(),
                binding: PendingBinding {
                    flags: SymbolFlags::UMD_EXPORT,
                    excludes: SymbolFlags::empty(),
                    scope: current_scope,
                },
            },
            None,
            true,
        );
    }

    pub(super) fn pending_type_parameter_owner(&self) -> Option<SymbolId> {
        if !self.pending.flags.contains(SymbolFlags::TYPE_PARAMETER) {
            return None;
        }
        self.ts_build
            .type_parameter_owners
            .get(&self.pending.scope)
            .copied()
    }

    pub(super) fn pending_is_default_export(&self, tree: &Ast) -> bool {
        tree.is_ts()
            && self.ts.source_exports.is_some()
            && self.export_state.is_default()
            && self.pending.scope == ScopeId::MODULE
            && self
                .pending
                .flags
                .intersects(SymbolFlags::FUNCTION | SymbolFlags::CLASS | SymbolFlags::INTERFACE)
    }

    pub(super) fn declare_anonymous_default(&mut self, tree: &Ast<'a>, node: NodeId) {
        if !self.pending_is_default_export(tree) {
            return;
        }
        let declaration = PendingDeclaration {
            name: tree.empty_wtf8(),
            node,
            binding: PendingBinding {
                flags: self.pending.flags | SymbolFlags::EXPORTED | SymbolFlags::IS_DEFAULT,
                ..self.pending
            },
        };
        self.declare_default_export(tree, declaration, false);
    }

    pub(super) fn declare_default_export(
        &mut self,
        tree: &Ast<'a>,
        declaration: PendingDeclaration,
        named: bool,
    ) -> DeclarationId {
        let mut source = self
            .ts
            .source_exports
            .take()
            .expect("a default export belongs to a source module");
        let environment = source.environment;
        let existing = source.default.as_ref().map(|default| default.symbol);
        let merges =
            existing.is_none_or(|existing| self.declaration_merges(existing, declaration.binding));
        let (symbol, declaration_id) = self.declare_symbol(tree, declaration, existing, named);
        let default = source.default.get_or_insert(TsDefaultExport {
            symbol,
            first_local: None,
            first_local_is_function: false,
        });
        if named {
            let name = tree.get_wtf8(declaration.name).as_bytes();
            let function = declaration.binding.flags.contains(SymbolFlags::FUNCTION);
            if default.first_local.is_none() || function && !default.first_local_is_function {
                default.first_local = Some(name);
                default.first_local_is_function = function;
            }
        }
        self.ts.source_exports = Some(source);
        if merges {
            self.record_ts_member(
                environment,
                b"default",
                symbol,
                TsMemberKinds::MODULE,
                declaration.binding.flags,
            );
        }
        if named {
            self.record_ts_export_local(tree, declaration, symbol);
        }
        declaration_id
    }

    fn record_ts_export_local(
        &mut self,
        tree: &Ast<'a>,
        declaration: PendingDeclaration,
        symbol: SymbolId,
    ) {
        let flags = declaration.binding.flags;
        // TS7 declares exported import-equals aliases only in Exports.
        if flags.intersects(SymbolFlags::ANY_IMPORT) {
            return;
        }
        let name = tree.get_wtf8(declaration.name).as_bytes();
        let function = flags.contains(SymbolFlags::FUNCTION);
        let scope = declaration.binding.scope;
        let local = self
            .ts
            .local_exports
            .entry((scope, name))
            .or_insert(TsExportLocal {
                target: symbol,
                target_is_function: function,
                exported_value: false,
                default_declarations: None,
            });
        local.exported_value |= flags.in_value_space();
        // declareModuleMember replaces LocalSymbol.ExportSymbol on every
        // export declaration. TS7 visits statement-list functions first, then
        // all other statements, each in source order.
        if local.target_is_function || !function {
            local.target = symbol;
            local.target_is_function = function;
        }
        if flags.contains(SymbolFlags::IS_DEFAULT) && scope == ScopeId::MODULE {
            match &mut local.default_declarations {
                Some(local) => local.flags = merge_symbol_flags(local.flags, flags),
                local @ None => {
                    *local = Some(TsDefaultLocal {
                        node: declaration.node,
                        flags,
                    })
                }
            }
        }
    }

    /// An infer parameter is visible in its own constraint as well as the
    /// conditional true branch, without exposing its siblings' infer names.
    pub fn expose_ts_infer_parameter(
        &mut self,
        tree: &Ast<'a>,
        name: BindingIdentifier,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        self.sync_scope_maps_from(scope);
        let symbol = self
            .symbol_of_node(name.node_id())
            .expect("the infer parameter is already declared");
        self.insert_scope_binding(tree, current_scope, name.name(tree).into(), symbol);
    }

    #[cold]
    #[inline(never)]
    pub(super) fn declare_ts_context_binding(
        &mut self,
        tree: &Ast<'a>,
        name: AstWtf8,
        node: NodeId,
    ) -> DeclarationId {
        let plan = self.prepare_declaration(tree, name, node);
        self.declare_prepared(tree, plan)
    }

    fn declare_ts_namespace_member(
        &mut self,
        tree: &Ast<'a>,
        owner: SymbolId,
        declaration: PendingDeclaration,
        actual_scope: Option<ScopeId>,
    ) -> (SymbolId, PreparedDeclaration) {
        let name = declaration.name;
        let name_text = tree.get_wtf8(name).as_bytes();
        let environment = self.ts.environments.ensure(owner, self.allocator);
        let shared_existing = self.ts.environments.member(environment, name_text);
        let local_existing = actual_scope.and_then(|scope| self.own_binding(scope, name_text));
        let target = actual_scope.unwrap_or(declaration.binding.scope);
        let mut declaration = declaration;
        declaration.binding.scope = target;
        let prepared = PreparedDeclaration {
            declaration,
            destination: DeclarationDestination::TsMember {
                environment,
                body_scope: actual_scope,
                projection_scope: None,
            },
            local: local_existing,
            shared: shared_existing,
            existing: shared_existing,
            default_local: None,
        };
        let declaration_id = self.declare_prepared(tree, prepared);
        let symbol = self.declarations[declaration_id].symbol;
        (symbol, prepared)
    }

    pub(super) fn declare_ts_member_in_environment(
        &mut self,
        tree: &Ast<'a>,
        environment: TsMemberEnvironmentId,
        body_scope: Option<ScopeId>,
        projection_scope: Option<ScopeId>,
        mut declaration: PendingDeclaration,
        existing: Option<SymbolId>,
    ) -> DeclarationId {
        let name = declaration.name;
        let name_text = tree.get_wtf8(name).as_bytes();
        let kind = if declaration.binding.flags.contains(SymbolFlags::ENUM_MEMBER) {
            TsMemberKinds::ENUM
        } else {
            TsMemberKinds::MODULE
        };
        let symbol_scope = body_scope.unwrap_or(declaration.binding.scope);
        declaration.binding.scope = symbol_scope;
        let merges =
            existing.is_none_or(|existing| self.declaration_merges(existing, declaration.binding));
        let (id, declaration_id) = self.declare_symbol(tree, declaration, existing, true);

        if merges {
            self.record_ts_member(environment, name_text, id, kind, declaration.binding.flags);
            if let Some(scope) = projection_scope {
                if let Some(projected) = self.binding(scope, name_text) {
                    debug_assert_eq!(projected, id);
                } else {
                    self.insert_scope_binding(tree, scope, name, id);
                }
            }
        }
        if kind == TsMemberKinds::MODULE && body_scope.is_some() {
            self.record_ts_export_local(tree, declaration, id);
        }
        declaration_id
    }

    fn record_ts_member(
        &mut self,
        environment: TsMemberEnvironmentId,
        name: &'a [u8],
        symbol: SymbolId,
        kind: TsMemberKinds,
        flags: SymbolFlags,
    ) {
        let mut kinds = kind;
        let spaces = if kind == TsMemberKinds::MODULE {
            if !flags.intersects(SymbolFlags::ANY_IMPORT) {
                kinds.insert(TsMemberKinds::DECLARATION);
            }
            TsMemberSpaces::from_flags(flags)
        } else {
            TsMemberSpaces::empty()
        };
        let declaration = TsMemberDeclaration {
            symbol,
            kinds,
            module_spaces: TsMemberVisibility::certain(spaces),
        };
        let members = &mut self.ts.environments.environments[environment].members;
        let hash = members.hasher().hash_one(name);
        match members.raw_entry_mut().from_key_hashed_nocheck(hash, name) {
            hashbrown::hash_map::RawEntryMut::Occupied(mut entry) => {
                let entry = entry.get_mut();
                if let Some(direct) = &mut entry.direct {
                    debug_assert_eq!(direct.symbol, symbol);
                    direct.kinds.insert(kinds);
                    direct.module_spaces.insert(declaration.module_spaces);
                } else {
                    entry.direct = Some(declaration);
                }
            }
            hashbrown::hash_map::RawEntryMut::Vacant(entry) => {
                entry.insert_hashed_nocheck(
                    hash,
                    name,
                    TsMemberEntry {
                        direct: Some(declaration),
                        alias: None,
                    },
                );
            }
        }
    }

    pub fn record_ts_import_equals_target(
        &mut self,
        tree: &Ast<'a>,
        declaration: TsImportEqualsDeclaration,
    ) {
        let alias = self
            .symbol_of_node(declaration.id(tree).node_id())
            .expect("an import-equals binding has a symbol");
        let Some(root) = crate::ecmascript::import_equals_reference_node(tree, declaration) else {
            return;
        };
        // A type name visits its root reference followed only by identifier names.
        let root_reference = self.references.last_idx();
        debug_assert_eq!(self.references[root_reference].node, root);
        self.ts_build.import_aliases.push(TsImportAliasTarget {
            alias,
            declaration,
            root_reference,
        });
    }

    /// Save property assignments with TS7's binder containers. These are not
    /// lexical queries: crossing a function or namespace changes ownership.
    pub fn record_ts_expando_assignment(
        &mut self,
        tree: &Ast<'a>,
        node: AssignmentExpression,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
        ancestors: impl Iterator<Item = NodeId>,
    ) {
        if !tree.lang().is_typescript() || node.operator(tree) != AssignmentOperator::Assign {
            return;
        }
        let left = crate::ecmascript::unwrap_parentheses(tree, node.left(tree).node_id());
        let NodeKindData::MemberExpression(member) = tree.kind_data(left) else {
            return;
        };
        if !is_expando_assignment(tree, member) {
            return;
        }
        let site = TsExpandoBindingSite::new(tree, scope, current_scope, ancestors);
        self.ts_build.expando_assignments.push((member, site));
    }

    pub fn add_ts_type_predicate_reference(
        &mut self,
        tree: &Ast<'a>,
        predicate: TsTypePredicate,
        scope: &ScopeTree<'a>,
        current_scope: ScopeId,
    ) {
        self.sync_scope_maps_from(scope);
        // `param is T` parses `param` as an IdentifierName but it
        // references the enclosing function or signature parameter. Its
        // binding identity is preserved even when the signature is erased.
        let parameter_name = predicate.parameter_name(tree);
        let parameter_node = parameter_name.node_id();
        let NodeKindData::IdentifierName(parameter_name) = tree.kind_data(parameter_node) else {
            return;
        };
        let name = parameter_name.name(tree);
        let parameter_scope = if scope.get(current_scope).kind == ScopeKind::TsSignatureReturn {
            scope
                .get(current_scope)
                .parent
                .expect("a return annotation has an owning signature")
        } else {
            current_scope
        };
        let Some(parameter) = self.own_binding(parameter_scope, tree.get_utf8(name).as_bytes())
        else {
            return;
        };
        if !self
            .symbol(parameter)
            .flags
            .contains(SymbolFlags::PARAMETER)
        {
            return;
        }
        self.add_reference(
            tree,
            name,
            current_scope,
            parameter_node,
            ReferenceFlags::default()
                .with_space(ReferenceSpace::Typeof)
                .no_access(),
        );
    }

    #[inline]
    pub fn in_ambient_context(&self) -> bool {
        self.ambient || self.pending.flags.contains(SymbolFlags::AMBIENT)
    }

    pub(crate) fn in_unsupported_ts_external_module(&self) -> bool {
        matches!(
            self.ts_build.containers.last(),
            Some(TsContainerContext::UnsupportedExternalModule { .. })
        )
    }

    fn default_export_symbol(&self) -> Option<SymbolId> {
        self.ts
            .source_exports
            .as_ref()?
            .default
            .as_ref()
            .map(|default| default.symbol)
    }

    fn symbol_declared_in_scope(&self, symbol: SymbolId, scope: ScopeId) -> bool {
        let first = self.symbols[symbol].declaration;
        if self.declarations[first].scope == scope {
            return true;
        }
        self.redeclarations
            .get(&symbol)
            .is_some_and(|declarations| {
                declarations
                    .iter()
                    .any(|declaration| self.declarations[*declaration].scope == scope)
            })
    }

    /// Checks whether an existing symbol has an overlapping declaration in
    /// the same binding scope whose exported/local visibility differs from
    /// the incoming declaration.
    /// TypeScript permits that difference across merged namespace bodies and
    /// across disjoint declaration spaces, but not within one body and space.
    #[cold]
    #[inline(never)]
    pub(crate) fn has_same_scope_export_visibility_conflict(
        &self,
        prepared: PreparedDeclaration,
        symbol: SymbolId,
    ) -> bool {
        // Default exports are checked by TypeScript's separate TS2652 rule.
        // This path implements only the named-export/local TS2395 invariant.
        let flags = prepared.flags();
        let target = prepared.target();
        if flags.contains(SymbolFlags::IS_DEFAULT) {
            return false;
        }
        let incoming_exported = flags.contains(SymbolFlags::EXPORTED);
        let incoming_spaces = ts_declaration_spaces(flags);
        if incoming_spaces.is_empty() {
            return false;
        }
        let first = self.symbols[symbol].declaration;
        if self.declaration_has_export_visibility_conflict(
            first,
            target,
            incoming_exported,
            incoming_spaces,
        ) {
            return true;
        }
        self.redeclarations
            .get(&symbol)
            .is_some_and(|declarations| {
                declarations.iter().copied().any(|declaration| {
                    self.declaration_has_export_visibility_conflict(
                        declaration,
                        target,
                        incoming_exported,
                        incoming_spaces,
                    )
                })
            })
    }

    fn declaration_has_export_visibility_conflict(
        &self,
        declaration: DeclarationId,
        scope: ScopeId,
        incoming_exported: bool,
        incoming_spaces: TsDeclarationSpaces,
    ) -> bool {
        let declaration = self.declarations[declaration];
        declaration.scope == scope
            && !declaration.flags.contains(SymbolFlags::IS_DEFAULT)
            && declaration.flags.contains(SymbolFlags::EXPORTED) != incoming_exported
            && ts_declaration_spaces(declaration.flags).intersects(incoming_spaces)
    }

    /// Follow a recorded local ExportValue link without changing the local
    /// symbol used for conflict checks or qualified namespace ownership.
    pub(super) fn ts_exported_value(&self, lexical: SymbolId, name: &[u8]) -> Option<SymbolId> {
        ts_exported_value(
            &self.ts.local_exports,
            &self.symbols,
            &self.declarations,
            &self.scope_maps,
            lexical,
            name,
        )
    }

    fn ts_member_declared_in_scope(
        &self,
        scope: ScopeId,
        name: &[u8],
        scope_views: &TsScopeViewIndex<'_>,
    ) -> Option<SymbolId> {
        let range = scope_views.ranges[scope];
        if range.len == 0 {
            return None;
        }
        // The body's own container view precedes qualified-name ancestors.
        // Restrict the result to a contribution declared in this body, since
        // the environment itself is shared by all merged bodies.
        let view = self.ts.views[range.start as usize];
        let members = &self.ts.environments.environments[view.environment].members;
        let direct = members.get(name)?.direct?;
        let visible =
            view.kind.accepts(direct.kinds) && self.symbol_declared_in_scope(direct.symbol, scope);
        visible.then_some(direct.symbol)
    }

    #[cold]
    #[inline(never)]
    fn ts_public_member(&self, owner: SymbolId, name: &[u8]) -> Option<TsResolvedImportTarget> {
        // TypeScript's public export symbol can combine a direct member and
        // an export alias under one name. Keep both contributions here rather
        // than mutating either canonical symbol with the other's meanings.
        let environment = self.ts.environments.get(owner)?;
        environment
            .members
            .get(name)?
            .target(&self.ts.environments.aliases)
    }

    #[cold]
    #[inline(never)]
    pub(super) fn finalize_ts_scope_views(&self, scopes: &Scopes) -> Option<TsScopeViewIndex<'a>> {
        if self.ts_build.scope_views.is_empty() {
            return None;
        }
        let scope_count = scopes.len();
        let mut ranges = ArenaIndexVec::with_capacity_in(scope_count, self.allocator);
        ranges.resize(scope_count, Range::default());
        for &(scope, range) in &self.ts_build.scope_views {
            debug_assert_eq!(ranges[scope].len, 0);
            ranges[scope] = range;
        }
        let mut lookup: ArenaIndexVec<ScopeId, TsScopeLookup> =
            ArenaIndexVec::with_capacity_in(scope_count, self.allocator);
        for (id, scope) in scopes.iter_enumerated() {
            let parent = scope
                .parent
                .map(|parent| lookup[parent])
                .unwrap_or_default();
            lookup.push(TsScopeLookup {
                members: if ranges[id].len != 0 {
                    Some(id)
                } else {
                    parent.members
                },
                restricted_lexical: scope.kind.is_ts_lookup_view() || parent.restricted_lexical,
            });
        }
        Some(TsScopeViewIndex { ranges, lookup })
    }

    pub(super) fn resolve_references_with_ts_views<const PRECISE_IMPORT_ALIASES: bool>(
        &mut self,
        tree: &Ast<'a>,
        scopes: &ScopeTree<'_>,
        scope_views: &TsScopeViewIndex<'_>,
        use_ranges: &mut UseRanges,
        additional_targets: &mut ReferenceCandidates<'a>,
    ) -> TsReferenceValues<'a> {
        let needs_values = {
            let members = &self.ts.environments;
            self.ts_build.export_aliases.iter().any(|alias| {
                let environment = &members.environments[alias.environment];
                let entry = environment.members[alias.exported_name];
                entry
                    .target(&members.aliases)
                    .is_some_and(TsResolvedImportTarget::needs_value_projection)
            })
        };
        if needs_values {
            self.resolve_references_with_ts_views_policy::<PRECISE_IMPORT_ALIASES, true>(
                tree,
                scopes,
                scope_views,
                use_ranges,
                additional_targets,
            )
        } else {
            self.resolve_references_with_ts_views_policy::<PRECISE_IMPORT_ALIASES, false>(
                tree,
                scopes,
                scope_views,
                use_ranges,
                additional_targets,
            )
        }
    }

    #[cold]
    #[inline(never)]
    fn resolve_references_with_ts_views_policy<
        const PRECISE_IMPORT_ALIASES: bool,
        const MEMBER_VALUES: bool,
    >(
        &mut self,
        tree: &Ast<'a>,
        scopes: &ScopeTree<'_>,
        scope_views: &TsScopeViewIndex<'_>,
        use_ranges: &mut UseRanges,
        additional_targets: &mut ReferenceCandidates<'a>,
    ) -> TsReferenceValues<'a> {
        let mut reference_values = TsReferenceValues::new_in(self.allocator);
        let import_alias_spaces = self.ts.import_aliases.spaces.as_slice();
        let lexical = LexicalLookup {
            scopes: &scopes.list,
            symbols: &self.symbols,
            internal_names: &self.internal_names,
            scope_maps: &self.scope_maps,
        };
        let ts = &self.ts;
        let mut candidates = Vec::new();
        let mut bindings = Vec::new();
        let mut values = Vec::new();
        for (id, record) in self.references.iter_mut_enumerated() {
            candidates.clear();
            bindings.clear();
            let reference = *record;
            let name = tree.get_utf8(reference.name).as_bytes();
            let query = NameLookupQuery {
                scope: reference.scope,
                name,
                space: reference.flags.space,
            };
            let result = lookup_name_with_ts_views::<true, PRECISE_IMPORT_ALIASES, MEMBER_VALUES>(
                lexical,
                ts,
                scope_views,
                query,
                |binding| {
                    if MEMBER_VALUES {
                        bindings.push(binding);
                    } else {
                        candidates.push(binding.symbol);
                    }
                },
            );
            let resolved = result.resolution();
            let first = result.binding();
            let has_member = MEMBER_VALUES
                && (first.is_some_and(|binding| binding.member.is_some())
                    || bindings.iter().any(|binding| binding.member.is_some()));
            let selected = first.into_iter().chain(bindings.iter().copied());
            if has_member {
                values.clear();
                let value_lookup = TsValueTargetLookup {
                    locals: &self.ts.local_exports,
                    symbols: &self.symbols,
                    declarations: &self.declarations,
                    scope_maps: &self.scope_maps,
                    import_alias_spaces,
                };
                self.ts.collect_binding_values::<PRECISE_IMPORT_ALIASES>(
                    &value_lookup,
                    name,
                    query.space.value_target_space(),
                    selected,
                    matches!(resolved, NameResolution::Uncertain { .. }),
                    &mut values,
                );
                if reference.flags.space == ReferenceSpace::Any && values.is_empty() {
                    record.flags = reference.flags.no_access();
                }
                reference_values.record(id, &values);
            } else if reference.flags.space == ReferenceSpace::Any {
                record.flags = {
                    let value_lookup = TsValueTargetLookup {
                        locals: &self.ts.local_exports,
                        symbols: &self.symbols,
                        declarations: &self.declarations,
                        scope_maps: &self.scope_maps,
                        import_alias_spaces,
                    };
                    value_lookup
                        .reference_flags::<PRECISE_IMPORT_ALIASES>(tree, reference, resolved)
                };
            }
            if MEMBER_VALUES {
                candidates.extend(bindings.iter().map(|binding| binding.symbol));
            }
            additional_targets.record(id, record, resolved, &mut candidates, use_ranges);
        }
        reference_values
    }
}

fn infer_owner(scope: &ScopeTree<'_>, current_scope: ScopeId) -> ScopeId {
    // Like TS7's getInferTypeContainer, select the nearest conditional whose
    // extends subtree contains the declaration, even across nested branches.
    for id in scope.ancestors(Some(current_scope)) {
        let region = scope.get(id);
        if region.kind == ScopeKind::TsConditionalExtends {
            return region
                .parent
                .expect("an extends view has a conditional owner");
        }
    }
    current_scope
}

fn collect_ts_module_path(
    tree: &Ast<'_>,
    qualified: TsQualifiedName,
    members: &mut Vec<(AstWtf8, NodeId)>,
) -> Option<NodeId> {
    let root = match tree.ts_qualified_name_left_data(qualified.left(tree)) {
        TsQualifiedNameLeftData::BindingIdentifier(identifier) => identifier.node_id(),
        TsQualifiedNameLeftData::TsQualifiedName(left) => {
            collect_ts_module_path(tree, left, members)?
        }
        TsQualifiedNameLeftData::IdentifierReference(_)
        | TsQualifiedNameLeftData::IdentifierName(_)
        | TsQualifiedNameLeftData::ThisExpression(_) => return None,
    };
    let right = qualified.right(tree);
    members.push((right.name(tree).into(), right.node_id()));
    Some(root)
}

pub(super) fn ts_module_export_name(tree: &Ast<'_>, name: ModuleExportName) -> AstWtf8 {
    match tree.module_export_name_data(name) {
        ModuleExportNameData::IdentifierName(identifier) => identifier.name(tree).into(),
        ModuleExportNameData::StringLiteral(literal) => literal.value(tree),
    }
}

// TypeScript gives an ambient namespace body an implicit export context unless
// it contains an export declaration or export assignment. `export as
// namespace` does not change that context.
fn ts_module_has_export_declarations(
    tree: &Ast<'_>,
    body: Option<swc_next_ecma_ast::TsModuleBody>,
) -> bool {
    let Some(body) = body else {
        return false;
    };
    match tree.kind_data(body.node_id()) {
        NodeKindData::TsModuleBlock(block) => block.body(tree).iter().any(|statement| {
            let statement = tree.get_node_in_sub_range(statement).node_id();
            ts_statement_is_export_declaration(tree, statement)
        }),
        NodeKindData::TsModuleDeclaration(declaration) => {
            ts_module_has_export_declarations(tree, declaration.body(tree))
        }
        _ => false,
    }
}

fn ts_statement_is_export_declaration(tree: &Ast, statement: NodeId) -> bool {
    match tree.kind_data(statement) {
        NodeKindData::ExportNamedDeclaration(export) => export.declaration(tree).is_none(),
        NodeKindData::ExportDefaultDeclaration(export) => {
            export.declaration(tree).as_expr(tree).is_some()
        }
        NodeKindData::ExportAllDeclaration(_) | NodeKindData::TsExportAssignment(_) => true,
        _ => false,
    }
}

pub(super) fn symbol_flags_clear_const_enum_only(flags: SymbolFlags) -> bool {
    flags.intersects(SymbolFlags::FUNCTION | SymbolFlags::CLASS | SymbolFlags::REGULAR_ENUM)
        || flags.contains(SymbolFlags::VALUE_MODULE)
            && !flags.contains(SymbolFlags::CONST_ENUM_ONLY_MODULE)
}

fn ts_declaration_spaces(flags: SymbolFlags) -> TsDeclarationSpaces {
    if flags.intersects(SymbolFlags::ANY_IMPORT) {
        // TypeScript derives import-clause and namespace-import spaces from
        // the resolved target. This single-file binder intentionally does not
        // invent those target meanings when module resolution is unavailable.
        return TsDeclarationSpaces::empty();
    }

    let mut spaces = TsDeclarationSpaces::empty();
    if flags.intersects(SymbolFlags::INTERFACE | SymbolFlags::TYPE_ALIAS | SymbolFlags::ENUM_MEMBER)
    {
        spaces.insert(TsDeclarationSpaces::TYPE);
    }
    if flags.intersects(SymbolFlags::CLASS | SymbolFlags::REGULAR_ENUM | SymbolFlags::CONST_ENUM) {
        spaces.insert(TsDeclarationSpaces::VALUE | TsDeclarationSpaces::TYPE);
    }
    if flags.contains(SymbolFlags::VALUE_MODULE) {
        spaces.insert(TsDeclarationSpaces::VALUE | TsDeclarationSpaces::NAMESPACE);
    } else if flags.contains(SymbolFlags::NAMESPACE_MODULE) {
        spaces.insert(TsDeclarationSpaces::NAMESPACE);
    }
    if flags.intersects(
        SymbolFlags::FUNCTION_SCOPED_VAR
            | SymbolFlags::BLOCK_SCOPED_VAR
            | SymbolFlags::FUNCTION
            | SymbolFlags::ENUM_MEMBER,
    ) {
        spaces.insert(TsDeclarationSpaces::VALUE);
    }
    spaces
}

fn ts_resolved_alias_spaces(flags: SymbolFlags) -> TsResolvedAliasSpaces {
    let spaces = ts_declaration_spaces(flags);
    let mut resolved = TsResolvedAliasSpaces::empty();
    resolved.set(
        TsResolvedAliasSpaces::VALUE,
        spaces.contains(TsDeclarationSpaces::VALUE),
    );
    resolved.set(
        TsResolvedAliasSpaces::TYPE,
        spaces.contains(TsDeclarationSpaces::TYPE),
    );
    resolved.set(
        TsResolvedAliasSpaces::NAMESPACE,
        spaces.contains(TsDeclarationSpaces::NAMESPACE),
    );
    if flags.intersects(SymbolFlags::REGULAR_ENUM | SymbolFlags::CONST_ENUM) {
        resolved.insert(TsResolvedAliasSpaces::NAMESPACE);
    }
    resolved
}

impl<'a> SymbolTracker<'a> {
    pub(crate) fn resolve_ts_aliases(
        &mut self,
        tree: &Ast<'a>,
        scopes: &ScopeTree<'a>,
        checks: SyntaxChecks,
    ) -> Vec<TsAliasIssue> {
        if self.ts_build.export_aliases.is_empty() && self.ts_build.import_aliases.is_empty() {
            return Vec::new();
        }

        // ======================= Environments =======================
        let scopes = &scopes.list;
        self.sync_scope_maps(scopes.len());
        let scope_views = self.finalize_ts_scope_views(scopes);

        // ======================= Export targets =======================
        if !self.ts_build.export_aliases.is_empty() {
            let aliases = std::mem::replace(
                &mut self.ts.environments.aliases,
                ArenaIndexVec::new_in(self.allocator),
            );
            let resolver =
                TsExportAliasResolver::new(self, tree, scopes, scope_views.as_ref(), aliases);
            let (aliases, members) = resolver.resolve();
            self.ts.environments.aliases = aliases;
            self.ts.environments.targets = members;
        }

        // ======================= Alias meanings =======================
        let issues = {
            // Target lookup, property facts and space propagation are shared by
            // both diagnostics and the completed model.
            let mut resolver = TsImportAliasResolver::new(self, tree, scopes, scope_views.as_ref());
            let mut issues = Vec::new();
            if checks.ts_export_aliases() {
                let members = &self.ts.environments;
                for (index, alias) in self.ts_build.export_aliases.iter().enumerate() {
                    let id = TsExportAliasId::from_usize(index);
                    let entry = &members.aliases[id];
                    if let Some(previous) = alias.previous {
                        issues.push(TsAliasIssue::ExportConflict {
                            node: entry.node,
                            previous,
                            duplicate: true,
                        });
                        continue;
                    }
                    let target = match entry.resolution {
                        TsExportAliasResolution::Resolved(target) => target,
                        TsExportAliasResolution::Unresolved => {
                            // Erased exports need no runtime binding. Preserve the
                            // unresolved alias for semantic queries, but leave missing
                            // type names to the type checker.
                            if entry.type_only {
                                continue;
                            }
                            let name = self.references[alias.local_reference].name;
                            issues.push(TsAliasIssue::MissingExport {
                                name,
                                node: entry.node,
                            });
                            continue;
                        }
                        TsExportAliasResolution::Pending | TsExportAliasResolution::Resolving => {
                            unreachable!("export alias dependencies are complete")
                        }
                        TsExportAliasResolution::NonLocal => {
                            let name = self.references[alias.local_reference].name;
                            issues.push(TsAliasIssue::NonLocalExport {
                                name,
                                node: entry.node,
                            });
                            continue;
                        }
                    };
                    let Some(direct) = members.member(alias.environment, alias.exported_name)
                    else {
                        continue;
                    };
                    // Conflict checks use the lexical target; public access may follow
                    // a separate exported-value bridge.
                    let contributions = target.target.conflict_contributions(&members.targets);
                    let alias_spaces = contributions.fold(
                        TsResolvedAliasSpaces::empty(),
                        |spaces, contribution| {
                            spaces | resolver.resolved_symbol_conflict_spaces(contribution.symbol)
                        },
                    );
                    let direct_spaces = resolver.resolved_symbol_conflict_spaces(direct);
                    let direct_flags = self.symbols[direct].flags;
                    let conflict = direct_flags.intersects(SymbolFlags::ANY_IMPORT)
                        || alias_spaces.intersects(direct_spaces);
                    if conflict {
                        issues.push(TsAliasIssue::ExportConflict {
                            node: entry.node,
                            previous: self.first_decl_of(direct),
                            duplicate: false,
                        });
                    }
                }
            }

            if checks.ts_import_aliases() {
                for (index, &alias) in self.ts_build.import_aliases.iter().enumerate() {
                    // Import aliases inherit the union of all public target
                    // contributions. Type-only provenance is kept separately
                    // because it constrains access without changing the target's
                    // declaration spaces.
                    let Some(target) = resolver.space_graph.targets[index] else {
                        continue;
                    };
                    if checks.ts_type_only_import_aliases()
                        && let Some(export) = target.type_only_export
                    {
                        issues.push(TsAliasIssue::TypeOnlyExport {
                            node: alias.declaration.module_reference(tree).node_id(),
                            export,
                        });
                    }
                    if !checks.ts_import_alias_conflicts() {
                        continue;
                    }
                    let alias_name = self.symbols[alias.alias].name;
                    let name = tree.get_wtf8(alias_name).as_bytes();
                    let node = alias.declaration.id(tree).node_id();
                    let alias_scope = self.references[alias.root_reference].scope;
                    let body_member = scope_views
                        .as_ref()
                        .and_then(|scope_views| {
                            self.ts_member_declared_in_scope(alias_scope, name, scope_views)
                        })
                        .filter(|&member| member != alias.alias)
                        .map(|symbol| DeclarationConflictCandidate {
                            symbol,
                            kind: DeclarationConflictKind::Scope(alias_scope),
                        });
                    if body_member.is_none() && !self.redeclarations.contains_key(&alias.alias) {
                        continue;
                    }
                    let target_spaces = resolver.resolved_target_conflict_spaces(target);
                    let candidates = std::iter::once(DeclarationConflictCandidate {
                        symbol: alias.alias,
                        kind: DeclarationConflictKind::Symbol,
                    })
                    .chain(body_member);
                    let conflict = candidates
                        .flat_map(|candidate| self.conflict_candidate_declarations(candidate))
                        .find(|declaration| {
                            declaration.node != node
                                && !declaration.flags.intersects(SymbolFlags::ANY_IMPORT)
                                && ts_resolved_alias_spaces(declaration.flags)
                                    .intersects(target_spaces)
                        });
                    if let Some(previous) = conflict {
                        issues.push(TsAliasIssue::ImportConflict {
                            name: alias_name,
                            node,
                            previous: previous.node,
                            previous_flags: previous.flags,
                        });
                    }
                }
            }

            let (module_spaces, erased_targets) = resolver.resolve_alias_contributions();
            let aliases = resolver.into_index(self.allocator, self.symbols.len());

            // ======================= Publish =======================
            // Refine only namespace contributions. Enum members keep their own
            // visibility, even when their canonical symbol also contains an alias.
            if !module_spaces.is_empty() {
                for environment in &mut self.ts.environments.environments {
                    for entry in environment
                        .members
                        .values_mut()
                        .filter_map(|entry| entry.direct.as_mut())
                    {
                        if entry.kinds.contains(TsMemberKinds::MODULE)
                            && let Some(spaces) = module_spaces[entry.symbol.index()]
                        {
                            entry.module_spaces = spaces;
                        }
                    }
                }
            }
            for reference in erased_targets {
                let reference = &mut self.references[reference];
                reference.flags = reference.flags.no_access();
            }
            self.ts.import_aliases = aliases;
            issues
        };
        self.ts.scope_views = scope_views;
        issues
    }
}

impl<'tracker, 'a> TsExportAliasResolver<'tracker, 'a> {
    fn new(
        tracker: &'tracker SymbolTracker<'a>,
        tree: &'tracker Ast<'a>,
        scopes: &'tracker Scopes,
        scope_views: Option<&'tracker TsScopeViewIndex<'a>>,
        aliases: ArenaIndexVec<'a, TsExportAliasId, TsExportAlias>,
    ) -> Self {
        let count = aliases.len();
        Self {
            tracker,
            tree,
            scopes,
            scope_views,
            aliases,
            member_targets: vec![None; count],
            members: ArenaIndexVec::new_in(tracker.allocator),
        }
    }

    fn resolve(
        mut self,
    ) -> (
        ArenaIndexVec<'a, TsExportAliasId, TsExportAlias>,
        ArenaIndexVec<'a, TsExportMemberId, TsExportMember>,
    ) {
        let mut pending = Vec::new();
        for root in 0..self.aliases.len() {
            let id = TsExportAliasId::from_usize(root);
            if self.tracker.ts_build.export_aliases[root]
                .previous
                .is_some()
            {
                self.aliases[id].resolution = TsExportAliasResolution::Unresolved;
                continue;
            }
            if !matches!(
                self.aliases[id].resolution,
                TsExportAliasResolution::Pending
            ) {
                continue;
            }
            let mut index = root;
            self.aliases[id].resolution = TsExportAliasResolution::Resolving;
            loop {
                match self.lookup_target(index) {
                    ControlFlow::Break(dependency) => {
                        let id = TsExportAliasId::from_usize(dependency);
                        self.aliases[id].resolution = TsExportAliasResolution::Resolving;
                        pending.push(index);
                        index = dependency;
                    }
                    ControlFlow::Continue(result) => {
                        let mut result = result;
                        if let TsExportAliasResolution::Resolved(target) = &mut result {
                            let alias = self.tracker.ts_build.export_aliases[index];
                            let id = TsExportAliasId::from_usize(index);
                            let entry = self.aliases[id];
                            if entry.type_only {
                                target.type_only_export = Some(entry.node);
                            }
                            if let TsExportTarget::Symbols {
                                lexical,
                                ref mut public,
                            } = target.target
                            {
                                let reference = self.tracker.references[alias.local_reference];
                                let name = self.tree.get_utf8(reference.name).as_bytes();
                                if let Some(exported) =
                                    self.tracker.ts_exported_value(lexical, name)
                                {
                                    *public = exported;
                                }
                            }
                        }
                        let id = TsExportAliasId::from_usize(index);
                        self.aliases[id].resolution = result;
                        let Some(parent) = pending.pop() else {
                            break;
                        };
                        index = parent;
                    }
                }
            }
        }
        (self.aliases, self.members)
    }

    fn lookup_target(&mut self, index: usize) -> ControlFlow<usize, TsExportAliasResolution> {
        let alias = self.tracker.ts_build.export_aliases[index];
        let reference = self.tracker.references[alias.local_reference];
        // Export specifiers select a name across all declaration spaces.
        debug_assert_eq!(reference.flags.space, ReferenceSpace::Any);
        let name = self.tree.get_utf8(reference.name).as_bytes();
        let hash = self.tracker.scope_maps[ScopeId::ROOT]
            .hasher()
            .hash_one(name);
        let mut ancestor = Some(reference.scope);
        while let Some(current) = ancestor {
            if current == ScopeId::ROOT {
                break;
            }
            if let Some((_, &symbol)) = self.tracker.scope_maps[current]
                .raw_entry()
                .from_hash(hash, |candidate| *candidate == name)
            {
                return ControlFlow::Continue(TsExportAliasResolution::Resolved(
                    TsResolvedExportAliasTarget::symbol(symbol),
                ));
            }
            if current == ScopeId::MODULE
                && let Some(symbol) = self.tracker.ts.source_exports.as_ref().and_then(|source| {
                    source.lookup(
                        name,
                        ReferenceSpace::Any,
                        &self.tracker.symbols,
                        &self.tracker.ts.local_exports,
                    )
                })
            {
                return ControlFlow::Continue(TsExportAliasResolution::Resolved(
                    TsResolvedExportAliasTarget::symbol(symbol),
                ));
            }
            let range = self
                .scope_views
                .map_or_else(Range::default, |views| views.ranges[current]);
            let start = range.start as usize;
            for view in &self.tracker.ts.views[start..start + range.len as usize] {
                let Some(candidate) = view.candidates(&self.tracker.ts.environments, hash, name)
                else {
                    continue;
                };
                let direct = candidate.direct.map(|entry| entry.symbol);
                let Some(alias) = candidate.alias else {
                    if let Some(direct) = direct {
                        return ControlFlow::Continue(TsExportAliasResolution::Resolved(
                            TsResolvedExportAliasTarget::symbol(direct),
                        ));
                    }
                    continue;
                };
                return self.resolve_member_target(direct, alias);
            }
            ancestor = self.scopes[current].parent;
        }
        let global = lookup_lexical_name::<true, true, true>(
            LexicalLookup {
                scopes: self.scopes,
                symbols: &self.tracker.symbols,
                internal_names: &self.tracker.internal_names,
                scope_maps: &self.tracker.scope_maps,
            },
            NameLookupQuery {
                scope: ScopeId::ROOT,
                name,
                space: ReferenceSpace::Any,
            },
        )
        .candidate();
        let Some(symbol) = global else {
            return ControlFlow::Continue(TsExportAliasResolution::Unresolved);
        };
        // TS7 checks the target's declaration container, not the scope where
        // lookup found it. Global augmentations project their members into
        // ROOT while preserving each member's declaration in the module body.
        let declaration = self.tracker.symbols[symbol].declaration;
        let declaration_scope = self.tracker.declarations[declaration].scope;
        let target = if declaration_scope == ScopeId::ROOT {
            TsExportAliasResolution::NonLocal
        } else {
            TsExportAliasResolution::Resolved(TsResolvedExportAliasTarget::symbol(symbol))
        };
        ControlFlow::Continue(target)
    }

    fn resolve_member_target(
        &mut self,
        direct: Option<SymbolId>,
        alias: TsExportAliasId,
    ) -> ControlFlow<usize, TsExportAliasResolution> {
        let dependency = alias.index();
        let target = match self.aliases[alias].resolution {
            TsExportAliasResolution::Pending => return ControlFlow::Break(dependency),
            TsExportAliasResolution::Resolved(target) => target,
            // Failed aliases still occupy this public name. Its own declaration
            // meaning remains, but lookup cannot expose an outer namesake.
            state => {
                let result = if let Some(direct) = direct {
                    TsExportAliasResolution::Resolved(TsResolvedExportAliasTarget::symbol(direct))
                } else if matches!(state, TsExportAliasResolution::NonLocal) {
                    TsExportAliasResolution::NonLocal
                } else {
                    TsExportAliasResolution::Unresolved
                };
                return ControlFlow::Continue(result);
            }
        };
        let Some(direct) = direct else {
            let mut target = target;
            if let TsExportTarget::Symbols {
                ref mut lexical,
                public,
            } = target.target
            {
                *lexical = public;
            }
            return ControlFlow::Continue(TsExportAliasResolution::Resolved(target));
        };
        if matches!(target.target, TsExportTarget::Symbols { public, .. } if public == direct) {
            return ControlFlow::Continue(TsExportAliasResolution::Resolved(
                TsResolvedExportAliasTarget::symbol(direct),
            ));
        }
        let member = *self.member_targets[dependency].get_or_insert_with(|| {
            self.members.push(TsExportMember {
                direct,
                alias: target,
            })
        });
        ControlFlow::Continue(TsExportAliasResolution::Resolved(
            TsResolvedExportAliasTarget::target(TsExportTarget::Member(member)),
        ))
    }
}

impl<'tracker, 'a> TsImportAliasResolver<'tracker, 'a> {
    fn new(
        tracker: &'tracker SymbolTracker<'a>,
        tree: &'tracker Ast<'a>,
        scopes: &'tracker Scopes,
        scope_views: Option<&'tracker TsScopeViewIndex<'a>>,
    ) -> Self {
        let mut aliases =
            HashMap::with_capacity_and_hasher(tracker.ts_build.import_aliases.len(), FxBuildHasher);
        for &target in &tracker.ts_build.import_aliases {
            let index = aliases.len();
            aliases
                .entry(target.alias)
                .or_insert(TsImportAliasInfo { target, index });
        }
        let property_values = if aliases.is_empty() {
            TsPropertyValues::default()
        } else {
            collect_ts_property_values(
                tree,
                scopes,
                &tracker.ts_build.expando_assignments,
                &TsMemberFactsLookup {
                    tracker,
                    scopes,
                    scope_views,
                },
            )
        };
        let mut resolver = Self {
            tracker,
            tree,
            scopes,
            scope_views,
            aliases,
            alias_targets: Vec::new(),
            property_values,
            space_graph: TsImportAliasSpaceGraph {
                targets: Vec::with_capacity(tracker.ts_build.import_aliases.len()),
                public: Vec::new(),
                conflict: Vec::new(),
            },
        };
        resolver.space_graph.public = resolver.compute_spaces::<false>();
        resolver
    }

    fn conflict_spaces(&mut self) -> &[TsResolvedAliasSpaces] {
        if self.space_graph.conflict.is_empty() {
            self.space_graph.conflict = self.compute_spaces::<true>();
        }
        &self.space_graph.conflict
    }

    fn compute_spaces<const CONFLICT: bool>(&mut self) -> Vec<TsResolvedAliasSpaces> {
        let mut spaces = vec![TsResolvedAliasSpaces::empty(); self.aliases.len()];
        for (&symbol, alias) in &self.aliases {
            spaces[alias.index] = self.non_import_spaces(symbol);
        }
        let mut edges = Vec::new();
        let tracker = self.tracker;
        let members = &tracker.ts.environments.targets;
        for (index, &alias) in tracker.ts_build.import_aliases.iter().enumerate() {
            let target = if CONFLICT {
                self.space_graph.targets[index]
            } else {
                let target = self.resolve_target(alias);
                self.space_graph.targets.push(target);
                target
            };
            let dependent = self.aliases[&alias.alias].index;
            let Some(target) = target else {
                spaces[dependent] |= TsResolvedAliasSpaces::UNKNOWN;
                continue;
            };
            spaces[dependent] |= target.non_symbol_spaces();
            for contribution in target.contributions(members, CONFLICT) {
                self.add_space_dependency(&mut spaces, &mut edges, dependent, contribution.symbol);
            }
        }
        propagate_alias_spaces(&mut spaces, edges);
        spaces
    }

    fn resolved_target_conflict_spaces(
        &mut self,
        target: TsResolvedImportTarget,
    ) -> TsResolvedAliasSpaces {
        target
            .conflict_symbols(&self.tracker.ts.environments.targets)
            .fold(
                target.non_symbol_spaces().declaration_spaces(),
                |spaces, symbol| spaces | self.resolved_symbol_conflict_spaces(symbol),
            )
    }

    fn resolved_symbol_conflict_spaces(&mut self, root: SymbolId) -> TsResolvedAliasSpaces {
        self.resolved_symbol_spaces(root, true).declaration_spaces()
    }

    fn resolve_alias_contributions(
        &mut self,
    ) -> (Vec<Option<TsMemberVisibility>>, Vec<ReferenceId>) {
        let mut module_spaces = Vec::new();
        let mut erased_targets = Vec::new();
        for index in 0..self.tracker.ts_build.import_aliases.len() {
            let alias = self.tracker.ts_build.import_aliases[index];
            let target = self.space_graph.targets[index];
            let redeclarations = self.tracker.redeclarations.get(&alias.alias);
            let spaces = target.map(|target| {
                if redeclarations.is_none() {
                    // A standalone alias contributes exactly its target's spaces.
                    let index = self.aliases[&alias.alias].index;
                    self.space_graph.public[index]
                } else {
                    // Merged symbols also contain non-alias contributions.
                    target
                        .public_symbols(&self.tracker.ts.environments.targets)
                        .fold(target.non_symbol_spaces(), |spaces, symbol| {
                            spaces | self.resolved_symbol_spaces(symbol, false)
                        })
                }
            });
            if let (Some(target), Some(spaces)) = (target, spaces)
                && !self.may_have_runtime_value_target(target, spaces)
            {
                erased_targets.push(alias.root_reference);
            }
            // Local aliases are resolved by the lexical index. Only exported
            // symbols can require refinement in a shared member environment.
            if !self.tracker.symbols[alias.alias]
                .flags
                .contains(SymbolFlags::EXPORTED)
            {
                continue;
            }
            if module_spaces.is_empty() {
                module_spaces.resize(self.tracker.symbols.len(), None);
            }
            let module = module_spaces[alias.alias.index()].get_or_insert_with(|| {
                let Some(redeclarations) = self.tracker.redeclarations.get(&alias.alias) else {
                    return TsMemberVisibility::certain(TsMemberSpaces::empty());
                };
                let first = self.tracker.symbols[alias.alias].declaration;
                let certain = std::iter::once(first)
                    .chain(redeclarations.iter().copied())
                    .map(|id| self.tracker.declarations[id].flags)
                    .filter(|flags| {
                        !flags.intersects(SymbolFlags::ANY_IMPORT | SymbolFlags::ENUM_MEMBER)
                    })
                    .fold(TsMemberSpaces::empty(), |spaces, flags| {
                        spaces | TsMemberSpaces::from_flags(flags)
                    });
                TsMemberVisibility::certain(certain)
            });
            let type_only = alias.declaration.import_kind(self.tree) == ImportOrExportKind::Type;
            let Some(spaces) = spaces else {
                let flags = if type_only {
                    SymbolFlags::TYPE_IMPORT
                } else {
                    SymbolFlags::IMPORT
                };
                module.possible |= TsMemberSpaces::from_flags(flags);
                continue;
            };
            let visibility = |spaces: TsResolvedAliasSpaces| {
                let mut visible = TsMemberSpaces::empty();
                visible.set(
                    TsMemberSpaces::VALUE,
                    spaces.may_have(TsResolvedAliasSpaces::VALUE) && !type_only,
                );
                visible.set(
                    TsMemberSpaces::TYPEOF,
                    spaces.may_have(TsResolvedAliasSpaces::VALUE),
                );
                visible.set(
                    TsMemberSpaces::TYPE,
                    spaces.may_have(TsResolvedAliasSpaces::TYPE),
                );
                visible.set(
                    TsMemberSpaces::NAMESPACE,
                    spaces.may_have(TsResolvedAliasSpaces::NAMESPACE),
                );
                visible
            };
            module.insert(TsMemberVisibility {
                possible: visibility(spaces),
                certain: visibility(spaces.known()),
            });
        }
        (module_spaces, erased_targets)
    }

    fn into_index(self, allocator: &'a Allocator, symbol_count: usize) -> TsImportAliasIndex<'a> {
        if self.aliases.is_empty() {
            return TsImportAliasIndex::new_in(allocator);
        }
        let targets = self.resolved_targets();
        let public = self.space_graph.public;
        let mut resolved = ArenaIndexVec::with_capacity_in(symbol_count, allocator);
        resolved.resize(symbol_count, None);
        let mut spaces = ArenaIndexVec::with_capacity_in(symbol_count, allocator);
        // Unrefined symbols retain ordinary local-binding semantics; UNKNOWN
        // is reserved for aliases whose target meaning is genuinely opaque.
        spaces.resize(symbol_count, None);
        for (symbol, alias) in self.aliases {
            spaces[symbol] = Some(public[alias.index]);
            resolved[symbol] = targets[alias.index];
        }
        TsImportAliasIndex {
            spaces,
            targets: resolved,
        }
    }

    fn resolve_target(&mut self, target: TsImportAliasTarget) -> Option<TsResolvedImportTarget> {
        let mut pending = Vec::new();
        loop {
            let current = pending
                .last()
                .map_or(target, |symbol| self.aliases[symbol].target);
            let resolved = (|| {
                let resolved = self.try_resolve_target(current)?;
                if let Some(&symbol) = pending.last() {
                    let resolution = match resolved {
                        Some(target) => self.resolve_namespace_target(target)?,
                        None => TsNamespaceTargetResolution::Unknown,
                    };
                    self.record_target_resolution(
                        symbol,
                        TsImportTargetState::Resolved {
                            target: resolved,
                            namespace: resolution,
                        },
                    );
                }
                ControlFlow::Continue(resolved)
            })();
            match resolved {
                ControlFlow::Break(symbol) => {
                    // Only an uncached alias suspends lookup. Mark it before
                    // descending so a back edge still resolves as unknown.
                    self.record_target_resolution(symbol, TsImportTargetState::Resolving);
                    pending.push(symbol);
                }
                ControlFlow::Continue(resolved) => {
                    if pending.pop().is_none() {
                        return resolved;
                    }
                }
            }
        }
    }

    fn record_target_resolution(&mut self, symbol: SymbolId, resolution: TsImportTargetState) {
        // Reuse the alias graph's dense indices. Allocate only when a lookup
        // first suspends on an alias dependency.
        if self.alias_targets.is_empty() {
            self.alias_targets
                .resize(self.aliases.len(), TsImportTargetState::Pending);
        }
        let index = self.aliases[&symbol].index;
        self.alias_targets[index] = resolution;
    }

    // Yield an uncached alias dependency to the driver instead of
    // recursively resolving it. Completed targets are memoized, so retrying
    // a suspended lookup preserves candidate order and shadowing semantics.
    fn try_resolve_target(
        &self,
        target: TsImportAliasTarget,
    ) -> ControlFlow<SymbolId, Option<TsResolvedImportTarget>> {
        let TsModuleReferenceData::TsTypeName(name) = self
            .tree
            .ts_module_reference_data(target.declaration.module_reference(self.tree))
        else {
            return ControlFlow::Continue(None);
        };
        let scope = self.tracker.references[target.root_reference].scope;
        self.resolve_type_name(scope, name)
    }

    fn resolve_type_name(
        &self,
        scope: ScopeId,
        name: TsTypeName,
    ) -> ControlFlow<SymbolId, Option<TsResolvedImportTarget>> {
        // An unqualified import-equals target must provide namespace
        // meaning. For a qualified target, only its final member may instead
        // contribute value or type meaning.
        match self.tree.ts_type_name_data(name) {
            TsTypeNameData::IdentifierReference(identifier) => {
                let name = self.tree.get_utf8(identifier.name(self.tree)).as_bytes();
                let target = self.lookup_namespace_name(scope, name)?;
                // TS7 getSymbolFlagsEx follows a local ExportValue link when
                // deriving alias meanings. The namespace still owns qualified
                // lookup, but this alias exposes the linked export's meanings.
                let target = target.map(|mut target| {
                    if let Some(lexical) = target.direct
                        && let Some(public) = self.tracker.ts_exported_value(lexical, name)
                    {
                        target.direct = None;
                        target.alias = Some(TsResolvedExportAliasTarget::target(
                            TsExportTarget::Symbols { lexical, public },
                        ));
                    }
                    target
                });
                ControlFlow::Continue(target)
            }
            TsTypeNameData::TsQualifiedName(qualified) => {
                let Some(owner) =
                    self.resolve_qualified_name_left(scope, qualified.left(self.tree))?
                else {
                    return ControlFlow::Continue(None);
                };
                let right = qualified.right(self.tree);
                self.qualified_member(
                    owner,
                    self.tree.get_utf8(right.name(self.tree)).as_bytes(),
                    false,
                )
            }
        }
    }

    fn resolve_qualified_name_left(
        &self,
        scope: ScopeId,
        left: TsQualifiedNameLeft,
    ) -> ControlFlow<SymbolId, Option<TsResolvedImportTarget>> {
        match self.tree.ts_qualified_name_left_data(left) {
            TsQualifiedNameLeftData::IdentifierReference(identifier) => {
                let name = self.tree.get_utf8(identifier.name(self.tree)).as_bytes();
                let Some(target) = self.lookup_namespace_name(scope, name)? else {
                    return ControlFlow::Continue(None);
                };
                self.resolve_namespace_target(target)
                    .map_continue(TsNamespaceTargetResolution::target)
            }
            TsQualifiedNameLeftData::TsQualifiedName(qualified) => {
                let Some(owner) =
                    self.resolve_qualified_name_left(scope, qualified.left(self.tree))?
                else {
                    return ControlFlow::Continue(None);
                };
                let right = qualified.right(self.tree);
                self.qualified_member(
                    owner,
                    self.tree.get_utf8(right.name(self.tree)).as_bytes(),
                    true,
                )
            }
            TsQualifiedNameLeftData::BindingIdentifier(_)
            | TsQualifiedNameLeftData::IdentifierName(_)
            | TsQualifiedNameLeftData::ThisExpression(_) => ControlFlow::Continue(None),
        }
    }

    // Resolve each next segment against the whole public name. A direct hit
    // takes precedence; alias exports are fallback paths, not merged members.
    fn qualified_member(
        &self,
        target: TsResolvedImportTarget,
        name: &[u8],
        namespace: bool,
    ) -> ControlFlow<SymbolId, Option<TsResolvedImportTarget>> {
        let mut pending = vec![(
            target.public_contributions(&self.tracker.ts.environments.targets),
            target.type_only_export,
        )];
        let mut seen = HashSet::with_hasher(FxBuildHasher);
        while let Some((contributions, inherited)) = pending.last_mut() {
            let Some(contribution) = contributions.next() else {
                pending.pop();
                continue;
            };
            let type_only_export = contribution.type_only_export.or(*inherited);
            if let Some(member) = self.public_member(contribution.symbol, name) {
                let member = member.with_type_only_export(type_only_export);
                if !namespace {
                    return ControlFlow::Continue(Some(member));
                }
                match self.resolve_namespace_target(member)? {
                    TsNamespaceTargetResolution::Resolved(target) => {
                        return ControlFlow::Continue(Some(target));
                    }
                    TsNamespaceTargetResolution::Unknown => {
                        return ControlFlow::Continue(None);
                    }
                    TsNamespaceTargetResolution::NotNamespace => {}
                }
            }
            // Import aliases can themselves refer to composed public names.
            // Keep their targets intact and traverse without recursive calls.
            if let Some(alias) = self.aliases.get(&contribution.symbol)
                && seen.insert(contribution.symbol)
            {
                let resolution = self
                    .alias_targets
                    .get(alias.index)
                    .unwrap_or(&TsImportTargetState::Pending);
                if matches!(resolution, TsImportTargetState::Pending) {
                    return ControlFlow::Break(contribution.symbol);
                };
                // This is a member fallback, not another namespace-prefix
                // query. A class or function target can supply own exports.
                if let TsImportTargetState::Resolved {
                    target: Some(target),
                    ..
                } = resolution
                {
                    pending.push((
                        target.public_contributions(&self.tracker.ts.environments.targets),
                        target.type_only_export.or(type_only_export),
                    ));
                }
            }
        }
        ControlFlow::Continue(None)
    }

    fn public_member(&self, owner: SymbolId, name: &[u8]) -> Option<TsResolvedImportTarget> {
        let target = self.tracker.ts_public_member(owner, name);
        let value = self.property_values.get(owner, name);
        match (target, value) {
            (Some(mut target), value) => {
                // A known type member does not complete a partial property
                // analysis. Preserve possible value contributions alongside it.
                target.property_value = value;
                Some(target)
            }
            (None, PropertyValue::Absent) => None,
            (None, value) => Some(TsResolvedImportTarget::property_value(value)),
        }
    }

    fn lookup_namespace_name(
        &self,
        scope: ScopeId,
        name: &[u8],
    ) -> ControlFlow<SymbolId, Option<TsResolvedImportTarget>> {
        let hash = self.tracker.scope_maps[ScopeId::ROOT]
            .hasher()
            .hash_one(name);
        let mut ancestor = Some(scope);
        while let Some(current) = ancestor {
            let lexical = self.tracker.scope_maps[current]
                .raw_entry()
                .from_hash(hash, |candidate| *candidate == name)
                .map(|(_, &symbol)| symbol);
            if let Some(symbol) = lexical
                && self.has_namespace_meaning(symbol)?
            {
                return ControlFlow::Continue(Some(TsResolvedImportTarget::symbol(symbol)));
            }

            let range = self
                .scope_views
                .map_or_else(Range::default, |scope_views| scope_views.ranges[current]);
            let start = range.start as usize;
            for index in start..start + range.len as usize {
                let view = self.tracker.ts.views[index];
                let Some(member) = view.lookup(&self.tracker.ts.environments, hash, name) else {
                    continue;
                };
                if let Some(direct) = member.target.direct
                    && member
                        .direct_spaces
                        .possible
                        .contains(TsMemberSpaces::NAMESPACE)
                    && self.has_namespace_meaning(direct)?
                {
                    return ControlFlow::Continue(Some(member.target));
                }
                if let Some(alias) = member.target.alias {
                    for contribution in alias.target.public_contributions(
                        &self.tracker.ts.environments.targets,
                        member.target.type_only_export,
                    ) {
                        if self.has_namespace_meaning(contribution.symbol)? {
                            return ControlFlow::Continue(Some(member.target));
                        }
                    }
                }
            }
            ancestor = self.scopes[current].parent;
        }
        ControlFlow::Continue(None)
    }

    fn has_namespace_meaning(&self, symbol: SymbolId) -> ControlFlow<SymbolId, bool> {
        self.resolve_namespace_meaning(symbol)
            .map_continue(|resolution| {
                matches!(
                    resolution,
                    TsNamespaceTargetResolution::Resolved(_) | TsNamespaceTargetResolution::Unknown
                )
            })
    }

    fn resolve_namespace_target(
        &self,
        mut target: TsResolvedImportTarget,
    ) -> ControlFlow<SymbolId, TsNamespaceTargetResolution> {
        // Like TS7 resolveEntityName, follow aliases until the selected owner
        // itself has namespace meaning. Once it does, keep its public exports
        // intact: an alias can still supply members absent from that namespace.
        loop {
            if let Some(symbol) = target.direct {
                if self.tracker.symbols[symbol].flags.in_namespace_space() {
                    return ControlFlow::Continue(TsNamespaceTargetResolution::Resolved(target));
                }
                match self.resolve_namespace_meaning(symbol)? {
                    TsNamespaceTargetResolution::Resolved(resolved) => {
                        return ControlFlow::Continue(TsNamespaceTargetResolution::Resolved(
                            resolved.with_type_only_export(target.type_only_export),
                        ));
                    }
                    TsNamespaceTargetResolution::Unknown => {
                        return ControlFlow::Continue(TsNamespaceTargetResolution::Unknown);
                    }
                    TsNamespaceTargetResolution::NotNamespace => {}
                }
            }
            let Some(alias) = target.alias else {
                return ControlFlow::Continue(TsNamespaceTargetResolution::NotNamespace);
            };
            let inherited = target.type_only_export.or(alias.type_only_export);
            target = match alias.target {
                TsExportTarget::Symbols { public, .. } => {
                    TsResolvedImportTarget::symbol(public).with_type_only_export(inherited)
                }
                TsExportTarget::Member(member) => {
                    let member = &self.tracker.ts.environments.targets[member];
                    TsResolvedImportTarget {
                        direct: Some(member.direct),
                        alias: Some(member.alias),
                        type_only_export: inherited.or(member.alias.type_only_export),
                        property_value: PropertyValue::Absent,
                    }
                }
            };
        }
    }

    fn resolve_namespace_meaning(
        &self,
        symbol: SymbolId,
    ) -> ControlFlow<SymbolId, TsNamespaceTargetResolution> {
        let flags = self.tracker.symbols[symbol].flags;
        if flags.in_namespace_space() {
            return ControlFlow::Continue(TsNamespaceTargetResolution::Resolved(
                TsResolvedImportTarget::symbol(symbol),
            ));
        }
        if !flags.intersects(SymbolFlags::ANY_IMPORT) {
            return ControlFlow::Continue(TsNamespaceTargetResolution::NotNamespace);
        }
        let Some(alias) = self.aliases.get(&symbol) else {
            // An opaque import can shadow an outer namespace without having
            // a member owner that this single-file analysis can expand.
            return ControlFlow::Continue(TsNamespaceTargetResolution::Unknown);
        };
        match self
            .alias_targets
            .get(alias.index)
            .unwrap_or(&TsImportTargetState::Pending)
        {
            TsImportTargetState::Pending => ControlFlow::Break(symbol),
            TsImportTargetState::Resolving => {
                ControlFlow::Continue(TsNamespaceTargetResolution::Unknown)
            }
            TsImportTargetState::Resolved { namespace, .. } => ControlFlow::Continue(*namespace),
        }
    }

    fn resolved_symbol_spaces(&mut self, root: SymbolId, conflict: bool) -> TsResolvedAliasSpaces {
        let flags = self.tracker.symbols[root].flags;
        if !flags.intersects(SymbolFlags::ANY_IMPORT) {
            return ts_resolved_alias_spaces(flags);
        }
        let Some(index) = self.aliases.get(&root).map(|alias| alias.index) else {
            return self.dependency_spaces(root);
        };
        if conflict {
            self.conflict_spaces()[index]
        } else {
            self.space_graph.public[index]
        }
    }

    fn may_have_runtime_value_target(
        &mut self,
        target: TsResolvedImportTarget,
        spaces: TsResolvedAliasSpaces,
    ) -> bool {
        if !spaces.may_have_value() || target.type_only_export.is_some() {
            return false;
        }
        if target.non_symbol_spaces().may_have_value() {
            return true;
        }
        if !target
            .alias
            .is_some_and(|alias| matches!(alias.target, TsExportTarget::Member(_)))
        {
            return true;
        }
        // A merged name can expose value meaning exclusively through a type-only
        // alias. Preserve typeof lookup without counting that path at runtime.
        target
            .public_contributions(&self.tracker.ts.environments.targets)
            .any(|contribution| {
                contribution.type_only_export.is_none()
                    && self
                        .resolved_symbol_spaces(contribution.symbol, false)
                        .may_have_value()
            })
    }

    // Follow unique forwarding edges using the graph's existing alias indices.
    // Taking each edge once also terminates cycles without recursive traversal.
    fn resolved_targets(&self) -> Vec<Option<SymbolId>> {
        let mut targets = vec![None; self.aliases.len()];
        let mut dependencies = vec![None; self.aliases.len()];
        for (index, alias) in self.tracker.ts_build.import_aliases.iter().enumerate() {
            if self.tracker.redeclarations.contains_key(&alias.alias) {
                continue;
            }
            let Some(target) = self.space_graph.targets[index] else {
                continue;
            };
            if target.property_value != PropertyValue::Absent {
                continue;
            }
            let mut symbols = target.public_symbols(&self.tracker.ts.environments.targets);
            let Some(symbol) = symbols.next() else {
                continue;
            };
            if symbols.any(|other| other != symbol) {
                continue;
            }
            let index = self.aliases[&alias.alias].index;
            if let Some(dependency) = self.aliases.get(&symbol) {
                dependencies[index] = Some(dependency.index);
            } else if !self.tracker.symbols[symbol]
                .flags
                .intersects(SymbolFlags::ANY_IMPORT)
            {
                targets[index] = Some(symbol);
            }
        }
        let mut path = Vec::with_capacity(dependencies.len());
        for root in 0..dependencies.len() {
            let mut current = root;
            while let Some(next) = dependencies[current].take() {
                path.push(current);
                current = next;
            }
            let target = targets[current];
            for index in path.drain(..) {
                targets[index] = target;
            }
        }
        targets
    }

    fn add_space_dependency(
        &self,
        spaces: &mut [TsResolvedAliasSpaces],
        edges: &mut Vec<(usize, usize)>,
        dependent: usize,
        dependency: SymbolId,
    ) {
        if let Some(dependency) = self.aliases.get(&dependency) {
            edges.push((dependency.index, dependent));
        } else {
            spaces[dependent] |= self.dependency_spaces(dependency);
        }
    }

    // Only call for symbols outside the internal alias graph. Their import
    // declarations have opaque targets; internal aliases use graph edges.
    fn dependency_spaces(&self, symbol: SymbolId) -> TsResolvedAliasSpaces {
        let known = self.non_import_spaces(symbol);
        if self.tracker.symbols[symbol]
            .flags
            .intersects(SymbolFlags::ANY_IMPORT)
        {
            known | TsResolvedAliasSpaces::UNKNOWN
        } else {
            known
        }
    }

    fn non_import_spaces(&self, symbol: SymbolId) -> TsResolvedAliasSpaces {
        let flags = self.tracker.symbols[symbol].flags;
        // Without aliases these spaces are additive across merged declarations.
        if !flags.intersects(SymbolFlags::ANY_IMPORT) {
            return ts_resolved_alias_spaces(flags);
        }
        let first = self.tracker.symbols[symbol].declaration;
        std::iter::once(first)
            .chain(
                self.tracker
                    .redeclarations
                    .get(&symbol)
                    .into_iter()
                    .flatten()
                    .copied(),
            )
            .map(|declaration| self.tracker.declarations[declaration].flags)
            .filter(|flags| !flags.intersects(SymbolFlags::ANY_IMPORT))
            .fold(TsResolvedAliasSpaces::empty(), |spaces, flags| {
                spaces | ts_resolved_alias_spaces(flags)
            })
    }
}

fn propagate_alias_spaces(spaces: &mut [TsResolvedAliasSpaces], edges: Vec<(usize, usize)>) {
    if edges.is_empty() {
        return;
    }
    let mut offsets = vec![0usize; spaces.len() + 1];
    for &(dependency, _) in &edges {
        offsets[dependency + 1] += 1;
    }
    for index in 1..offsets.len() {
        offsets[index] += offsets[index - 1];
    }
    let mut cursors = offsets.clone();
    let mut dependents = vec![0usize; edges.len()];
    for (dependency, dependent) in edges {
        dependents[cursors[dependency]] = dependent;
        cursors[dependency] += 1;
    }

    // Known spaces, property provenance and uncertainty form a six-bit
    // lattice. Each node gains at most six bits, even for alias cycles.
    let mut worklist = (0..spaces.len())
        .filter(|&index| !spaces[index].is_empty())
        .collect::<Vec<_>>();
    while let Some(dependency) = worklist.pop() {
        for &dependent in &dependents[offsets[dependency]..offsets[dependency + 1]] {
            let next = spaces[dependent] | spaces[dependency];
            if next != spaces[dependent] {
                spaces[dependent] = next;
                worklist.push(dependent);
            }
        }
    }
}

// Collect own exported property contributions, not inherited object properties.
// Only syntax accepted by TS7's early or late member binding contributes.
// Eligible names requiring types remain unknown, never proof of absence.
fn collect_ts_property_values<'tree>(
    tree: &'tree Ast<'_>,
    scopes: &Scopes,
    assignments: &[(MemberExpression, TsExpandoBindingSite)],
    lookup: &TsMemberFactsLookup<'_, '_>,
) -> TsPropertyValues<'tree> {
    let mut values = TsPropertyValues::default();
    for scope in scopes {
        if scope.kind != ScopeKind::Class {
            continue;
        }
        let NodeKindData::Class(class) = tree.kind_data(scope.node) else {
            continue;
        };
        let owner = if let Some(id) = class.id(tree) {
            lookup.tracker.symbol_of_node(id.node_id())
        } else if class.class_type(tree) == ClassType::ClassDeclaration {
            lookup.tracker.default_export_symbol()
        } else {
            None
        };
        let Some(owner) = owner else {
            continue;
        };
        values.record(owner, Some(Cow::Borrowed(b"prototype")));
        for element in class.body(tree).body(tree).iter() {
            let element = tree.get_node_in_sub_range(element);
            let (key, computed) = match tree.kind_data(element.node_id()) {
                NodeKindData::PropertyDefinition(property) if property.r#static(tree) => {
                    (property.key(tree), property.computed(tree))
                }
                NodeKindData::MethodDefinition(method) if method.r#static(tree) => {
                    (method.key(tree), method.computed(tree))
                }
                NodeKindData::TsMethodDefinition(method) if method.r#static(tree) => {
                    (method.key(tree), method.computed(tree))
                }
                _ => continue,
            };
            // Parentheses are transparent to this model. TS7's class-name
            // binding is syntax-sensitive here; we intentionally use the
            // unparenthesized form for both class and assignment properties.
            let key = unwrap_parentheses(tree, key.node_id());
            let name = property_name(tree, key, computed);
            if let Some(name) = name {
                values.record(owner, Some(name));
            } else if computed && is_late_bindable_name(tree, key) {
                values.record(owner, None);
            }
        }
    }
    for &(member, site) in assignments {
        let object = member.object(tree).node_id();
        let receiver = lookup
            .expando_receiver(tree, object, site.block)
            .or_else(|| lookup.expando_receiver(tree, object, site.container));
        let Some(owner) = receiver.and_then(|symbol| lookup.function_symbol(symbol)) else {
            continue;
        };
        let property = member.property(tree).node_id();
        // As with class properties, normalize only transparent parentheses.
        let literal = unwrap_parentheses(tree, property);
        let name = property_name(tree, literal, member.computed(tree));
        if let Some(name) = name {
            // Like TS7 deferred expando binding, direct assignments do not
            // replace a non-assignment declaration with the same public name.
            if !lookup.has_declared_member(owner, &name) {
                values.record(owner, Some(name));
            }
        } else if member.computed(tree) && is_late_bindable_name(tree, property) {
            values.record(owner, None);
        }
    }
    values
}

impl TsMemberFactsLookup<'_, '_> {
    fn expando_receiver(
        &self,
        tree: &Ast,
        node: NodeId,
        scope: Option<ScopeId>,
    ) -> Option<SymbolId> {
        let node = unwrap_parentheses(tree, node);
        match tree.kind_data(node) {
            NodeKindData::IdentifierReference(identifier) => {
                self.container_binding(tree.get_utf8(identifier.name(tree)).as_bytes(), scope?)
            }
            NodeKindData::MemberExpression(member)
                if !member.computed(tree) && !member.optional(tree) =>
            {
                let receiver = self.expando_receiver(tree, member.object(tree).node_id(), scope)?;
                let owner = self.function_symbol(receiver)?;
                let name = text_member_name(tree, member.property(tree).node_id(), false)?;
                self.tracker
                    .ts_public_member(owner, tree.get_wtf8(name).as_bytes())?
                    .direct
            }
            _ => None,
        }
    }

    fn container_binding(&self, name: &[u8], scope: ScopeId) -> Option<SymbolId> {
        let hash = self.tracker.scope_maps[ScopeId::ROOT]
            .hasher()
            .hash_one(name);
        // TS has one locals table for a function, including its body when
        // ECMAScript needs separate parameter and body environments.
        let body = self.scopes[scope].function_body;
        let local = body
            .and_then(|body| lookup_scope_name(&self.tracker.scope_maps, body, name, hash))
            .or_else(|| lookup_scope_name(&self.tracker.scope_maps, scope, name, hash));
        if let Some(symbol) = local {
            let symbol = self
                .tracker
                .ts_exported_value(symbol, name)
                .unwrap_or(symbol);
            return Some(symbol);
        }
        let range = self
            .scope_views
            .map_or_else(Range::default, |views| views.ranges[scope]);
        let start = range.start as usize;
        for view in &self.tracker.ts.views[start..start + range.len as usize] {
            if let Some(candidate) = view.candidates(&self.tracker.ts.environments, hash, name)
                && let Some(direct) = candidate.direct
            {
                return Some(direct.symbol);
            }
        }
        None
    }

    fn function_symbol(&self, symbol: SymbolId) -> Option<SymbolId> {
        // Const function initializers own a different property symbol. Their
        // expandos do not augment the variable's namespace export surface.
        self.tracker
            .symbol(symbol)
            .flags
            .contains(SymbolFlags::FUNCTION)
            .then_some(symbol)
    }

    fn has_declared_member(&self, owner: SymbolId, name: &[u8]) -> bool {
        self.tracker.ts_public_member(owner, name).is_some()
    }
}

fn is_expando_assignment(tree: &Ast, mut member: MemberExpression) -> bool {
    loop {
        let object = unwrap_parentheses(tree, member.object(tree).node_id());
        if member.optional(tree) {
            return false;
        }
        match tree.kind_data(object) {
            NodeKindData::IdentifierReference(_) => return true,
            NodeKindData::MemberExpression(receiver) if !receiver.computed(tree) => {
                member = receiver
            }
            _ => return false,
        }
    }
}

impl TsExpandoBindingSite {
    pub(super) fn new(
        tree: &Ast,
        scopes: &ScopeTree<'_>,
        current_scope: ScopeId,
        ancestors: impl Iterator<Item = NodeId>,
    ) -> Self {
        let mut block_node = None;
        for node in ancestors {
            let kind = tree.kind_data(node);
            let container = matches!(
                kind,
                NodeKindData::Program(_)
                    | NodeKindData::Function(_)
                    | NodeKindData::ArrowFunctionExpression(_)
                    | NodeKindData::Class(_)
                    | NodeKindData::StaticBlock(_)
                    | NodeKindData::ObjectExpression(_)
                    | NodeKindData::TsModuleBlock(_)
                    | NodeKindData::TsEnumDeclaration(_)
                    | NodeKindData::TsInterfaceDeclaration(_)
                    | NodeKindData::TsTypeLiteral(_)
                    | NodeKindData::TsTypeAliasDeclaration(_)
                    | NodeKindData::TsMappedType(_)
                    | NodeKindData::TsIndexSignature(_)
                    | NodeKindData::TsFunction(_)
                    | NodeKindData::TsFunctionType(_)
                    | NodeKindData::TsConstructorType(_)
                    | NodeKindData::TsMethodSignature(_)
                    | NodeKindData::TsCallSignatureDeclaration(_)
                    | NodeKindData::TsConstructSignatureDeclaration(_)
                    | NodeKindData::JsxOpeningElement(_)
            );
            if container {
                let find_scope = |owner| {
                    let mut current = Some(current_scope);
                    while let Some(id) = current {
                        let scope = &scopes.list[id];
                        if scope.node == owner && !scope.kind.is_ts_lookup_view() {
                            return Some(id);
                        }
                        current = scope.parent;
                    }
                    None
                };
                let container = find_scope(node);
                let block = block_node.and_then(find_scope).or(container);
                return Self { container, block };
            }
            if block_node.is_none()
                && matches!(
                    kind,
                    NodeKindData::BlockStatement(_)
                        | NodeKindData::CatchClause(_)
                        | NodeKindData::ForStatement(_)
                        | NodeKindData::ForInStatement(_)
                        | NodeKindData::ForOfStatement(_)
                        | NodeKindData::SwitchStatement(_)
                )
            {
                block_node = Some(node);
            }
        }
        Self {
            container: None,
            block: None,
        }
    }
}

// Property facts own normalized names when the AST only stores a numeric value.
// A known numeric key must neither disappear nor make unrelated names uncertain.
fn property_name<'tree>(
    tree: &'tree Ast<'_>,
    node: NodeId,
    computed: bool,
) -> Option<Cow<'tree, [u8]>> {
    if let Some(name) = text_member_name(tree, node, computed) {
        return Some(Cow::Borrowed(tree.get_wtf8(name).as_bytes()));
    }
    let (number, prefix) = match tree.kind_data(node) {
        NodeKindData::NumericLiteral(number) => (number, ""),
        NodeKindData::UnaryExpression(unary) if computed => {
            let prefix = match unary.operator(tree) {
                UnaryOperator::Positive => "+",
                UnaryOperator::Negate => "-",
                _ => return None,
            };
            let argument = unwrap_parentheses(tree, unary.argument(tree).node_id());
            let NodeKindData::NumericLiteral(number) = tree.kind_data(argument) else {
                return None;
            };
            (number, prefix)
        }
        _ => return None,
    };
    let mut buffer = zmij_ecma::Buffer::new();
    let number = buffer.format(number.value(tree));
    // TS7 binds signed literal syntax using the token sign plus the normalized
    // numeric token, rather than evaluating the whole computed expression.
    let mut name = Vec::with_capacity(prefix.len() + number.len());
    name.extend_from_slice(prefix.as_bytes());
    name.extend_from_slice(number.as_bytes());
    Some(Cow::Owned(name))
}

fn text_member_name(tree: &Ast<'_>, node: NodeId, computed: bool) -> Option<AstWtf8> {
    match tree.kind_data(node) {
        NodeKindData::IdentifierName(identifier) if !computed => Some(identifier.name(tree).into()),
        NodeKindData::StringLiteral(literal) => Some(literal.value(tree)),
        NodeKindData::TemplateLiteral(template)
            if template.expressions(tree).is_empty() && template.quasis(tree).len() == 1 =>
        {
            let quasi_id = template.quasis(tree).iter().next()?;
            let quasi = tree.get_node_in_sub_range(quasi_id);
            (!quasi.is_cooked_undefined(tree)).then(|| quasi.cooked(tree))
        }
        // Expressions requiring type analysis cannot supply a known name here.
        // Private names do not contribute to the public property surface.
        _ => None,
    }
}

fn is_late_bindable_name(tree: &Ast<'_>, mut node: NodeId) -> bool {
    // Mirrors TS7 isLateBindableAST -> IsEntityNameExpression: identifiers
    // and dotted identifier paths qualify; their types remain outside this pass.
    loop {
        node = unwrap_parentheses(tree, node);
        match tree.kind_data(node) {
            NodeKindData::IdentifierReference(_) => return true,
            NodeKindData::MemberExpression(member) if !member.computed(tree) => {
                node = member.object(tree).node_id();
            }
            NodeKindData::ChainExpression(chain) => {
                node = chain.expression(tree).node_id();
            }
            _ => return false,
        }
    }
}

fn ts_module_instance_state(
    tree: &Ast<'_>,
    declaration: TsModuleDeclaration,
    outer_contexts: &[TsContainerContext],
    module_states: &mut HashMap<NodeId, TsModuleInstanceMemo, FxBuildHasher>,
) -> TsModuleInstanceState {
    // Keep the allocation across declarations, but never reuse answers from
    // another root: cycle results depend on the current traversal.
    module_states.clear();
    TsModuleInstanceAnalyzer {
        tree,
        module_states,
    }
    .module_in_contexts(declaration, outer_contexts, None)
}

impl TsModuleInstanceAnalyzer<'_, '_, '_> {
    fn module_in_contexts(
        &mut self,
        declaration: TsModuleDeclaration,
        contexts: &[TsContainerContext],
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> TsModuleInstanceState {
        let Some((context, remaining)) = contexts.split_first() else {
            return self.module(declaration, blocks);
        };
        let Some(body) = context.body() else {
            return self.module_in_contexts(declaration, remaining, blocks);
        };
        let blocks = TsModuleBlockPath {
            node: body,
            parent: blocks,
        };
        self.module_in_contexts(declaration, remaining, Some(&blocks))
    }

    fn module(
        &mut self,
        declaration: TsModuleDeclaration,
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> TsModuleInstanceState {
        let node = declaration.node_id();
        if let Some(state) = self.module_states.get(&node) {
            // An in-progress dependency is a cycle, while completed states
            // are shared across every path through the current dependency
            // graph.
            return match state {
                TsModuleInstanceMemo::Computing => TsModuleInstanceState::NonInstantiated,
                TsModuleInstanceMemo::Complete(state) => *state,
            };
        }
        self.module_states
            .insert(node, TsModuleInstanceMemo::Computing);
        let state = declaration
            .body(self.tree)
            .map_or(TsModuleInstanceState::Instantiated, |body| {
                self.module_body(body.node_id(), blocks)
            });
        self.module_states
            .insert(node, TsModuleInstanceMemo::Complete(state));
        state
    }

    fn module_body(
        &mut self,
        body: NodeId,
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> TsModuleInstanceState {
        match self.tree.kind_data(body) {
            NodeKindData::TsModuleBlock(block) => {
                if !block.directives(self.tree).is_empty() {
                    return TsModuleInstanceState::Instantiated;
                }
                let blocks = TsModuleBlockPath {
                    node: body,
                    parent: blocks,
                };
                let mut state = TsModuleInstanceState::NonInstantiated;
                for statement in block.body(self.tree).iter() {
                    let statement = self.tree.get_node_in_sub_range(statement).node_id();
                    let child = self.statement(statement, Some(&blocks));
                    state = state.combine(child);
                    if state == TsModuleInstanceState::Instantiated {
                        break;
                    }
                }
                state
            }
            NodeKindData::TsModuleDeclaration(declaration) => {
                match self.tree.ts_module_name_data(declaration.id(self.tree)) {
                    TsModuleNameData::StringLiteral(_) => TsModuleInstanceState::NonInstantiated,
                    _ => self.module(declaration, blocks),
                }
            }
            _ => TsModuleInstanceState::Instantiated,
        }
    }

    fn statement(
        &mut self,
        node: NodeId,
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> TsModuleInstanceState {
        match self.tree.kind_data(node) {
            NodeKindData::TsInterfaceDeclaration(_)
            | NodeKindData::TsTypeAliasDeclaration(_)
            | NodeKindData::ImportDeclaration(_)
            | NodeKindData::TsImportEqualsDeclaration(_) => TsModuleInstanceState::NonInstantiated,
            NodeKindData::TsEnumDeclaration(declaration) => {
                if declaration.is_const(self.tree) {
                    TsModuleInstanceState::ConstEnumOnly
                } else {
                    TsModuleInstanceState::Instantiated
                }
            }
            NodeKindData::TsModuleDeclaration(declaration) => self.module(declaration, blocks),
            NodeKindData::ExportDefaultDeclaration(export) => {
                match self
                    .tree
                    .export_default_declaration_kind_data(export.declaration(self.tree))
                {
                    ExportDefaultDeclarationKindData::TsInterfaceDeclaration(_) => {
                        TsModuleInstanceState::NonInstantiated
                    }
                    ExportDefaultDeclarationKindData::Function(_)
                    | ExportDefaultDeclarationKindData::TsFunction(_)
                    | ExportDefaultDeclarationKindData::Class(_)
                    | ExportDefaultDeclarationKindData::Expr(_) => {
                        TsModuleInstanceState::Instantiated
                    }
                }
            }
            NodeKindData::ExportNamedDeclaration(export) => self.export_declaration(export, blocks),
            _ => TsModuleInstanceState::Instantiated,
        }
    }

    fn export_declaration(
        &mut self,
        export: ExportNamedDeclaration,
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> TsModuleInstanceState {
        if let Some(declaration) = export.declaration(self.tree) {
            if matches!(
                self.tree.kind_data(declaration.node_id()),
                NodeKindData::TsImportEqualsDeclaration(_)
            ) {
                return TsModuleInstanceState::Instantiated;
            }
            return self.statement(declaration.node_id(), blocks);
        }
        if export.source(self.tree).is_some() {
            return TsModuleInstanceState::Instantiated;
        }

        let mut state = TsModuleInstanceState::NonInstantiated;
        for specifier in export.specifiers(self.tree).iter() {
            let specifier = self.tree.get_node_in_sub_range(specifier);
            let local = specifier.local(self.tree);
            let ModuleExportNameData::IdentifierName(local) =
                self.tree.module_export_name_data(local)
            else {
                return TsModuleInstanceState::Instantiated;
            };
            let name = self.tree.get_utf8(local.name(self.tree)).as_bytes();
            let target = self.local_export_target(name, blocks);
            state = state.combine(target);
            if state == TsModuleInstanceState::Instantiated {
                break;
            }
        }
        state
    }

    fn local_export_target(
        &mut self,
        name: &[u8],
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> TsModuleInstanceState {
        let mut block = blocks;
        while let Some(path) = block {
            if let Some(state) = self.named_statement_state(path.node, name, Some(path)) {
                return state;
            }
            block = path.parent;
        }
        if let Some(state) = self.named_program_statement_state(name) {
            return state;
        }
        // Like TypeScript, an unresolved syntactic target is conservatively
        // treated as a possible value.
        TsModuleInstanceState::Instantiated
    }

    fn named_program_statement_state(&mut self, name: &[u8]) -> Option<TsModuleInstanceState> {
        let statements = self
            .tree
            .root_program()
            .body(self.tree)
            .iter()
            .map(|statement| self.tree.get_node_in_sub_range(statement).node_id());
        self.named_statements_state(statements, name, None)
    }

    fn named_statement_state(
        &mut self,
        body: NodeId,
        name: &[u8],
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> Option<TsModuleInstanceState> {
        let NodeKindData::TsModuleBlock(block) = self.tree.kind_data(body) else {
            return None;
        };
        let statements = block
            .body(self.tree)
            .iter()
            .map(|statement| self.tree.get_node_in_sub_range(statement).node_id());
        self.named_statements_state(statements, name, blocks)
    }

    fn named_statements_state(
        &mut self,
        statements: impl Iterator<Item = NodeId>,
        name: &[u8],
        blocks: Option<&TsModuleBlockPath<'_>>,
    ) -> Option<TsModuleInstanceState> {
        let mut found = None;
        for statement in statements {
            let declaration = self.exported_declaration(statement);
            if !ts_module_statement_has_name(self.tree, declaration, name) {
                continue;
            }
            let mut state = self.statement(declaration, blocks);
            if matches!(
                self.tree.kind_data(declaration),
                NodeKindData::TsImportEqualsDeclaration(_)
            ) {
                state = TsModuleInstanceState::Instantiated;
            }
            found = Some(found.map_or(state, |current: TsModuleInstanceState| {
                current.combine(state)
            }));
            if found == Some(TsModuleInstanceState::Instantiated) {
                break;
            }
        }
        found
    }

    fn exported_declaration(&self, node: NodeId) -> NodeId {
        match self.tree.kind_data(node) {
            NodeKindData::ExportNamedDeclaration(export) => export
                .declaration(self.tree)
                .map_or(node, |declaration| declaration.node_id()),
            NodeKindData::ExportDefaultDeclaration(export) => {
                let declaration = export.declaration(self.tree);
                match self.tree.export_default_declaration_kind_data(declaration) {
                    ExportDefaultDeclarationKindData::Function(function) => function.node_id(),
                    ExportDefaultDeclarationKindData::TsFunction(function) => function.node_id(),
                    ExportDefaultDeclarationKindData::Class(class) => class.node_id(),
                    ExportDefaultDeclarationKindData::TsInterfaceDeclaration(interface) => {
                        interface.node_id()
                    }
                    ExportDefaultDeclarationKindData::Expr(_) => node,
                }
            }
            _ => node,
        }
    }
}

// Match TS7 NodeHasName for module instantiation. Deliberately inspect only
// direct declaration names, not BoundNames or identifiers in type annotations.
fn ts_module_statement_has_name(tree: &Ast<'_>, node: NodeId, name: &[u8]) -> bool {
    let identifier = match tree.kind_data(node) {
        NodeKindData::Function(function) => function.id(tree),
        NodeKindData::TsFunction(function) => function.id(tree),
        NodeKindData::Class(class) => class.id(tree),
        NodeKindData::TsInterfaceDeclaration(declaration) => Some(declaration.id(tree)),
        NodeKindData::TsTypeAliasDeclaration(declaration) => Some(declaration.id(tree)),
        NodeKindData::TsEnumDeclaration(declaration) => Some(declaration.id(tree)),
        NodeKindData::TsImportEqualsDeclaration(declaration) => Some(declaration.id(tree)),
        NodeKindData::TsModuleDeclaration(declaration) => {
            return ts_module_declares_name(tree, declaration.id(tree), name);
        }
        NodeKindData::VariableDeclaration(declaration) => {
            return declaration.declarators(tree).iter().any(|declarator| {
                let declarator = tree.get_node_in_sub_range(declarator);
                match tree.kind_data(declarator.id(tree).node_id()) {
                    NodeKindData::BindingIdentifier(identifier) => {
                        tree.get_utf8(identifier.name(tree)).as_bytes() == name
                    }
                    _ => false,
                }
            });
        }
        _ => None,
    };
    identifier.is_some_and(|identifier| tree.get_utf8(identifier.name(tree)).as_bytes() == name)
}

fn ts_module_declares_name(tree: &Ast<'_>, name: TsModuleName, expected: &[u8]) -> bool {
    match tree.ts_module_name_data(name) {
        TsModuleNameData::BindingIdentifier(identifier) => {
            tree.get_utf8(identifier.name(tree)).as_bytes() == expected
        }
        TsModuleNameData::TsQualifiedName(qualified) => {
            let mut left = qualified.left(tree);
            loop {
                match tree.ts_qualified_name_left_data(left) {
                    TsQualifiedNameLeftData::BindingIdentifier(identifier) => {
                        return tree.get_utf8(identifier.name(tree)).as_bytes() == expected;
                    }
                    TsQualifiedNameLeftData::TsQualifiedName(qualified) => {
                        left = qualified.left(tree);
                    }
                    TsQualifiedNameLeftData::IdentifierReference(_)
                    | TsQualifiedNameLeftData::IdentifierName(_)
                    | TsQualifiedNameLeftData::ThisExpression(_) => return false,
                }
            }
        }
        TsModuleNameData::StringLiteral(_) => false,
    }
}

/// One selected binding, optionally retaining its public-member lookup context.
/// For interface X plus `export { value as X }`, the view lets value queries find `value`.
#[derive(Clone, Copy)]
pub(super) struct NameBinding {
    pub(super) symbol: SymbolId,
    /// Present when an alias-backed member needs its view to project value contributions.
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

/// Nearest candidate and its certainty; possible outer fallbacks are emitted separately.
pub(super) struct NameLookupResult {
    resolution: NameResolution,
    /// Value-projection context of the nearest candidate, as in [`NameBinding::member`].
    member: Option<TsMemberView>,
}

impl Default for NameLookupResult {
    fn default() -> Self {
        Self::lexical(NameResolution::Unresolved)
    }
}

impl NameLookupResult {
    pub(super) fn resolution(&self) -> NameResolution {
        self.resolution
    }

    pub(super) fn binding(&self) -> Option<NameBinding> {
        self.resolution.candidate().map(|symbol| NameBinding {
            symbol,
            member: self.member,
        })
    }

    pub(super) fn lexical(resolution: NameResolution) -> Self {
        Self {
            resolution,
            member: None,
        }
    }

    // A possible contribution does not stop the search: every outer fallback
    // must reach the reference's candidate set and reverse use index as well.
    pub(super) fn accept(
        &mut self,
        binding: NameBinding,
        visibility: BindingVisibility,
        emit: &mut impl FnMut(NameBinding),
    ) -> bool {
        if visibility == BindingVisibility::Absent {
            return false;
        }
        if let Some(first) = self.resolution.candidate() {
            // A definite candidate already ended lookup. An outer fallback
            // cannot make the nearer, uncertain contribution definite.
            debug_assert!(matches!(self.resolution, NameResolution::Uncertain { .. }));
            if binding.symbol != first || binding.member != self.member {
                emit(binding);
            }
        } else {
            self.member = binding.member;
            self.resolution = if visibility == BindingVisibility::Unknown {
                NameResolution::Uncertain {
                    candidate: binding.symbol,
                }
            } else {
                NameResolution::Resolved(binding.symbol)
            };
        }
        visibility == BindingVisibility::Present
    }
}

#[inline(always)]
fn lookup_name_in_scopes<const INCLUDE_GLOBAL: bool, const PRECISE_IMPORT_ALIASES: bool>(
    lexical: LexicalLookup<'_, '_>,
    ts: &TsBindings<'_>,
    scope_views: Option<&TsScopeViewIndex<'_>>,
    query: NameLookupQuery<'_>,
    mut emit: impl FnMut(NameBinding),
) -> NameLookupResult {
    let Some(scope_views) = scope_views else {
        return NameLookupResult::lexical(if PRECISE_IMPORT_ALIASES {
            lookup_lexical_name_with_import_aliases::<INCLUDE_GLOBAL>(
                lexical,
                ts.import_aliases.spaces.as_slice(),
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
fn lookup_name_with_ts_views<
    const INCLUDE_GLOBAL: bool,
    const PRECISE_IMPORT_ALIASES: bool,
    const MEMBER_VALUES: bool,
>(
    lexical: LexicalLookup<'_, '_>,
    ts: &TsBindings<'_>,
    scope_views: &TsScopeViewIndex<'_>,
    query: NameLookupQuery<'_>,
    emit: impl FnMut(NameBinding),
) -> NameLookupResult {
    if scope_views.lookup[query.scope].restricted_lexical {
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
    ts: &TsBindings<'_>,
    scope_views: &TsScopeViewIndex<'_>,
    query: NameLookupQuery<'_>,
    mut emit: impl FnMut(NameBinding),
) -> NameLookupResult {
    let LexicalLookup {
        scopes,
        symbols,
        scope_maps,
        ..
    } = lexical;
    let arguments_barrier = query.arguments_barrier();
    let symbol_mask = if TS_LOOKUP {
        query.space.symbol_mask()
    } else {
        None
    };
    let NameLookupQuery { scope, name, space } = query;
    let Some(member_scope) = scope_views.lookup[scope].members else {
        return NameLookupResult::lexical(if PRECISE_IMPORT_ALIASES {
            lookup_lexical_name_with_import_aliases::<INCLUDE_GLOBAL>(
                lexical,
                ts.import_aliases.spaces.as_slice(),
                query,
                |symbol| emit(NameBinding::symbol(symbol)),
            )
        } else {
            lookup_lexical_name::<INCLUDE_GLOBAL, TS_LOOKUP, true>(lexical, query)
        });
    };
    let hash = scope_maps[ScopeId::ROOT].hasher().hash_one(name);
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
                lexical,
                current,
                child_kind,
                query,
                hash,
                |symbol| {
                    let flags = symbols[symbol].flags;
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
            if arguments_barrier && scopes[current].flags.arguments_barrier {
                return NameLookupResult::default();
            }
            child_kind = scopes[current].kind;
            ancestor = scopes[current].parent;
        }
    }
    while let Some(current) = ancestor {
        if !INCLUDE_GLOBAL && current == ScopeId::ROOT {
            break;
        }
        // The lexical prefix already rejected this scope with the same rules.
        let binding = if PRECISE_IMPORT_ALIASES || current != member_scope {
            lookup_scope_binding::<TS_LOOKUP>(lexical, current, child_kind, query, hash, |symbol| {
                if TS_LOOKUP && !binding_visible_from(child_kind, symbols[symbol].flags, space) {
                    return false;
                }
                let visibility = symbol_visibility::<PRECISE_IMPORT_ALIASES>(
                    symbols,
                    ts.import_aliases.spaces.as_slice(),
                    symbol,
                    space,
                );
                state.accept(NameBinding::symbol(symbol), visibility, &mut emit)
            })
        } else {
            None
        };
        if binding.is_some() {
            return state;
        }
        if current == ScopeId::MODULE
            && let Some(symbol) = ts
                .source_exports
                .as_ref()
                .and_then(|export| export.lookup(name, space, symbols, &ts.local_exports))
            && state.accept(
                NameBinding::symbol(symbol),
                BindingVisibility::Present,
                &mut emit,
            )
        {
            return state;
        }
        let range = scope_views.ranges[current];
        let start = range.start as usize;
        for view in &ts.views[start..start + range.len as usize] {
            let Some(member) = view.lookup(&ts.environments, hash, name) else {
                continue;
            };
            if member.visit_candidates::<PRECISE_IMPORT_ALIASES>(
                symbols,
                ts.import_aliases.spaces.as_slice(),
                &ts.environments.targets,
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
        if arguments_barrier && scopes[current].flags.arguments_barrier {
            break;
        }
        child_kind = scopes[current].kind;
        ancestor = scopes[current].parent;
    }
    state
}

#[inline(always)]
pub(super) fn lookup_lexical_name_with_import_aliases<const INCLUDE_GLOBAL: bool>(
    lexical: LexicalLookup<'_, '_>,
    import_alias_spaces: &TsImportAliasSpaces,
    query: NameLookupQuery<'_>,
    mut emit: impl FnMut(SymbolId),
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
    let mut state = NameLookupResult::default();
    let mut ancestor = Some(scope);
    let mut child_kind = ScopeKind::Block;
    while let Some(current) = ancestor {
        if !INCLUDE_GLOBAL && current == ScopeId::ROOT {
            break;
        }
        let binding =
            lookup_scope_binding::<true>(lexical, current, child_kind, query, hash, |symbol| {
                if !binding_visible_from(child_kind, symbols[symbol].flags, space) {
                    return false;
                }
                let visibility =
                    symbol_visibility::<true>(symbols, import_alias_spaces, symbol, space);
                state.accept(NameBinding::symbol(symbol), visibility, &mut |binding| {
                    emit(binding.symbol)
                })
            });
        if binding.is_some() {
            return state.resolution();
        }
        if arguments_barrier && scopes[current].flags.arguments_barrier {
            break;
        }
        child_kind = scopes[current].kind;
        ancestor = scopes[current].parent;
    }
    state.resolution()
}

pub(super) fn is_signature_query(kind: ScopeKind, space: ReferenceSpace) -> bool {
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

pub(super) fn binding_visible_from(
    child: ScopeKind,
    flags: SymbolFlags,
    space: ReferenceSpace,
) -> bool {
    if child == ScopeKind::TsConditionalExtends {
        return !flags.contains(SymbolFlags::TYPE_PARAMETER);
    }
    if !is_signature_query(child, space) || flags.intersects(SymbolFlags::INTERNAL_NAME) {
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
fn symbol_visibility<const PRECISE_IMPORT_ALIASES: bool>(
    symbols: &Symbols,
    import_alias_spaces: &TsImportAliasSpaces,
    symbol: SymbolId,
    space: ReferenceSpace,
) -> BindingVisibility {
    let flags = symbols[symbol].flags;
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
fn precise_import_alias_spaces(
    symbols: &Symbols,
    import_alias_spaces: &TsImportAliasSpaces,
    symbol: SymbolId,
) -> Option<TsResolvedAliasSpaces> {
    let flags = symbols[symbol].flags;
    if !flags.intersects(SymbolFlags::ANY_IMPORT) {
        return None;
    }
    import_alias_spaces.get(symbol).copied().flatten()
}
