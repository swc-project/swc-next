//! Resolves internal import aliases and namespace export aliases before reference binding.

use std::{hash::BuildHasher, ops::ControlFlow};

use hashbrown::{HashMap, HashSet};
use rustc_hash::FxBuildHasher;
use swc_next_allocator::{Allocator, vec::ArenaIndexVec};
use swc_next_ecma_ast::{
    Ast, ImportOrExportKind, NodeId, ReferenceId, ScopeId, SymbolId, TsModuleReferenceData,
    TsQualifiedNameLeft, TsQualifiedNameLeftData, TsTypeName, TsTypeNameData,
};

use super::lookup::{LexicalLookup, NameLookupQuery, lookup_lexical_name};
use super::member_facts::{
    PropertyValue, TsMemberFactsLookup, TsPropertyValues, collect_ts_property_values,
};
use super::{
    DeclarationConflictCandidate, DeclarationConflictKind, Range, ReferenceSpace, SymbolFlags,
    SymbolTracker, TsExportAliasResolution, TsExportMember, TsExportTarget,
    TsImportAliasConflictIssue, TsImportAliasIndex, TsImportAliasIssue, TsImportAliasTarget,
    TsMemberKinds, TsMemberSpaces, TsMemberVisibility, TsNamespaceExportAliasIssue,
    TsNamespaceExportAliasLookup, TsResolvedAliasSpaces, TsResolvedExportAliasTarget,
    TsResolvedImportTarget, TsScopeViewIndex, ts_resolved_alias_spaces,
};
use crate::{Scopes, SyntaxChecks, scope::ScopeTree};

impl<'a> SymbolTracker<'a> {
    pub(crate) fn resolve_ts_aliases(
        &mut self,
        tree: &Ast<'a>,
        scopes: &ScopeTree<'a>,
        checks: SyntaxChecks,
    ) -> (Vec<TsNamespaceExportAliasIssue>, Vec<TsImportAliasIssue>) {
        self.resolve_ts_namespace_export_aliases(tree, scopes);
        let export_issues = if checks.ts_export_aliases() {
            self.check_ts_namespace_export_aliases(tree, scopes)
        } else {
            Vec::new()
        };
        let import_issues = self.resolve_ts_import_aliases(tree, scopes, checks);
        (export_issues, import_issues)
    }

    #[cold]
    #[inline(never)]
    fn resolve_ts_namespace_export_aliases(&mut self, tree: &Ast<'a>, scopes: &ScopeTree<'a>) {
        if self.pending_ts_namespace_export_aliases.is_empty() {
            return;
        }
        self.sync_scope_maps(scopes.list.len());
        let scope_views = self
            .ts_scope_view_index
            .take()
            .or_else(|| self.finalize_ts_scope_views(&scopes.list));
        let states =
            TsExportAliasResolver::new(self, tree, &scopes.list, scope_views.as_ref()).resolve();
        for (index, state) in states.into_iter().enumerate() {
            let alias = self.pending_ts_namespace_export_aliases[index];
            if alias.previous.is_some() {
                continue;
            }
            let resolution = match state {
                TsExportAliasState::Resolved(target) => TsExportAliasResolution::Resolved(target),
                TsExportAliasState::Missing => TsExportAliasResolution::Unresolved,
                TsExportAliasState::NonLocal => TsExportAliasResolution::NonLocal,
                TsExportAliasState::Pending | TsExportAliasState::Resolving => {
                    unreachable!("export alias dependencies are complete")
                }
            };
            let entry = self
                .ts_member_environments
                .export_aliases_mut(alias.environment)
                .get_mut(alias.exported_name)
                .expect("a pending namespace export alias has a public entry");
            entry.resolution = resolution;
        }
        self.ts_scope_view_index = scope_views;
    }

    fn check_ts_namespace_export_aliases(
        &self,
        tree: &Ast<'a>,
        scopes: &ScopeTree<'a>,
    ) -> Vec<TsNamespaceExportAliasIssue> {
        if self.pending_ts_namespace_export_aliases.is_empty() {
            return Vec::new();
        }
        // All public targets are resolved before this pass, including aliases
        // reached through another namespace. Validate each entry once.
        let mut resolver =
            TsImportAliasResolver::new(self, tree, &scopes.list, self.ts_scope_view_index.as_ref());
        let mut issues = Vec::new();
        for alias in &self.pending_ts_namespace_export_aliases {
            if let Some(previous) = alias.previous {
                issues.push(TsNamespaceExportAliasIssue::Duplicate {
                    node: alias.node,
                    previous,
                });
                continue;
            }
            let entry = self
                .ts_member_environments
                .environment_export_aliases(alias.environment)
                .get(alias.exported_name)
                .expect("a pending namespace export alias has a public entry");
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
                    issues.push(TsNamespaceExportAliasIssue::Missing {
                        name,
                        node: alias.node,
                    });
                    continue;
                }
                TsExportAliasResolution::NonLocal => {
                    let name = self.references[alias.local_reference].name;
                    issues.push(TsNamespaceExportAliasIssue::NonLocal {
                        name,
                        node: alias.node,
                    });
                    continue;
                }
            };
            let Some(direct) = self
                .ts_member_environments
                .member(alias.environment, alias.exported_name)
            else {
                continue;
            };
            // Conflict checks use the lexical target; public access may follow
            // a separate exported-value bridge.
            let alias_spaces = target.target.conflict_contributions().fold(
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
                issues.push(TsNamespaceExportAliasIssue::Conflict {
                    node: alias.node,
                    previous: self.first_decl_of(direct),
                });
            }
        }
        issues
    }

    fn resolve_ts_import_aliases(
        &mut self,
        tree: &Ast<'a>,
        scopes: &ScopeTree<'a>,
        checks: SyntaxChecks,
    ) -> Vec<TsImportAliasIssue> {
        if self.ts_import_alias_targets.is_empty() {
            return Vec::new();
        }
        self.sync_scope_maps(scopes.list.len());
        let scope_views = self
            .ts_scope_view_index
            .take()
            .or_else(|| self.finalize_ts_scope_views(&scopes.list));
        let (issues, aliases, module_spaces, erased_targets) = {
            let mut resolver =
                TsImportAliasResolver::new(self, tree, &scopes.list, scope_views.as_ref());
            let mut issues = Vec::new();
            if checks.ts_import_aliases() {
                for (index, &alias) in self.ts_import_alias_targets.iter().enumerate() {
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
                        issues.push(TsImportAliasIssue::TypeOnlyExport {
                            node: alias.declaration.module_reference(tree).node_id(),
                            export,
                        });
                    }
                    if !checks.ts_import_alias_conflicts() {
                        continue;
                    }
                    let alias_name = self.symbols[alias.alias].name;
                    let alias_scope = self.references[alias.root_reference].scope;
                    let body_member = scope_views
                        .as_ref()
                        .and_then(|scope_views| {
                            self.ts_member_declared_in_scope(
                                alias_scope,
                                tree.get_wtf8(alias_name).as_bytes(),
                                scope_views,
                            )
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
                            declaration.node != alias.declaration.id(tree).node_id()
                                && !declaration.flags.intersects(SymbolFlags::ANY_IMPORT)
                                && ts_resolved_alias_spaces(declaration.flags)
                                    .intersects(target_spaces)
                        });
                    if let Some(previous) = conflict {
                        issues.push(TsImportAliasIssue::Conflict(TsImportAliasConflictIssue {
                            name: alias_name,
                            node: alias.declaration.id(tree).node_id(),
                            previous: previous.node,
                            previous_flags: previous.flags,
                        }));
                    }
                }
            }
            let (module_spaces, erased_targets) = resolver.resolve_alias_contributions();
            let aliases = resolver.into_public_index(self.allocator, self.symbols.len());
            (issues, aliases, module_spaces, erased_targets)
        };
        // Refine only namespace contributions. Enum members keep their own
        // visibility, even when their canonical symbol also contains an alias.
        if !module_spaces.is_empty() {
            for environment in &mut self.ts_member_environments.environments {
                for entry in environment.members.values_mut() {
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
        self.ts_import_aliases = Some(aliases);
        self.ts_scope_view_index = scope_views;
        issues
    }
}

#[derive(Clone, Copy)]
enum TsExportAliasState<'a> {
    Pending,
    Resolving,
    Resolved(TsResolvedExportAliasTarget<'a>),
    Missing,
    NonLocal,
}

// Construction state is separate from the final public alias entries. A
// pending candidate suspends lookup instead of exposing an outer namesake.
struct TsExportAliasResolver<'tracker, 'a> {
    tracker: &'tracker SymbolTracker<'a>,
    tree: &'tracker Ast<'a>,
    scopes: &'tracker Scopes,
    scope_views: Option<&'tracker TsScopeViewIndex<'a>>,
    indices: Option<HashMap<NodeId, usize, FxBuildHasher>>,
    states: Vec<TsExportAliasState<'a>>,
    member_targets: HashMap<NodeId, &'a TsExportMember<'a>, FxBuildHasher>,
}

impl<'tracker, 'a> TsExportAliasResolver<'tracker, 'a> {
    fn new(
        tracker: &'tracker SymbolTracker<'a>,
        tree: &'tracker Ast<'a>,
        scopes: &'tracker Scopes,
        scope_views: Option<&'tracker TsScopeViewIndex<'a>>,
    ) -> Self {
        let aliases = &tracker.pending_ts_namespace_export_aliases;
        Self {
            tracker,
            tree,
            scopes,
            scope_views,
            indices: None,
            states: vec![TsExportAliasState::Pending; aliases.len()],
            member_targets: HashMap::with_hasher(FxBuildHasher),
        }
    }

    fn resolve(mut self) -> Vec<TsExportAliasState<'a>> {
        let mut pending = Vec::new();
        for root in 0..self.states.len() {
            if self.tracker.pending_ts_namespace_export_aliases[root]
                .previous
                .is_some()
                || !matches!(self.states[root], TsExportAliasState::Pending)
            {
                continue;
            }
            let mut index = root;
            self.states[index] = TsExportAliasState::Resolving;
            loop {
                match self.lookup_target(index) {
                    ControlFlow::Break(dependency) => {
                        self.states[dependency] = TsExportAliasState::Resolving;
                        pending.push(index);
                        index = dependency;
                    }
                    ControlFlow::Continue(result) => {
                        self.states[index] = match result {
                            TsNamespaceExportAliasLookup::Resolved(mut target) => {
                                let alias = self.tracker.pending_ts_namespace_export_aliases[index];
                                if alias.type_only {
                                    target.type_only_export = Some(alias.node);
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
                                TsExportAliasState::Resolved(target)
                            }
                            TsNamespaceExportAliasLookup::Missing => TsExportAliasState::Missing,
                            TsNamespaceExportAliasLookup::NonLocal => TsExportAliasState::NonLocal,
                        };
                        let Some(parent) = pending.pop() else {
                            break;
                        };
                        index = parent;
                    }
                }
            }
        }
        self.states
    }

    fn lookup_target(
        &mut self,
        index: usize,
    ) -> ControlFlow<usize, TsNamespaceExportAliasLookup<'a>> {
        let alias = self.tracker.pending_ts_namespace_export_aliases[index];
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
                return ControlFlow::Continue(TsNamespaceExportAliasLookup::Resolved(
                    TsResolvedExportAliasTarget::symbol(symbol),
                ));
            }
            if current == ScopeId::MODULE
                && let Some(symbol) = self.tracker.ts_source_exports.as_ref().and_then(|source| {
                    source.lookup(
                        name,
                        ReferenceSpace::Any,
                        &self.tracker.symbols,
                        &self.tracker.ts_local_exports,
                    )
                })
            {
                return ControlFlow::Continue(TsNamespaceExportAliasLookup::Resolved(
                    TsResolvedExportAliasTarget::symbol(symbol),
                ));
            }
            let range = self
                .scope_views
                .map_or_else(Range::default, |views| views.ranges[current]);
            let start = range.start as usize;
            for view in &self.tracker.ts_member_views[start..start + range.len as usize] {
                let Some(candidate) =
                    view.candidates(&self.tracker.ts_member_environments, hash, name)
                else {
                    continue;
                };
                let direct = candidate.direct.map(|entry| entry.symbol);
                let Some(alias) = candidate.alias else {
                    if let Some(direct) = direct {
                        return ControlFlow::Continue(TsNamespaceExportAliasLookup::Resolved(
                            TsResolvedExportAliasTarget::symbol(direct),
                        ));
                    }
                    continue;
                };
                return self.resolve_member_target(direct, alias.node);
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
            return ControlFlow::Continue(TsNamespaceExportAliasLookup::Missing);
        };
        // TS7 checks the target's declaration container, not the scope where
        // lookup found it. Global augmentations project their members into
        // ROOT while preserving each member's declaration in the module body.
        let declaration = self.tracker.symbols[symbol].declaration;
        let declaration_scope = self.tracker.declarations[declaration].scope;
        let target = if declaration_scope == ScopeId::ROOT {
            TsNamespaceExportAliasLookup::NonLocal
        } else {
            TsNamespaceExportAliasLookup::Resolved(TsResolvedExportAliasTarget::symbol(symbol))
        };
        ControlFlow::Continue(target)
    }

    fn resolve_member_target(
        &mut self,
        direct: Option<SymbolId>,
        node: NodeId,
    ) -> ControlFlow<usize, TsNamespaceExportAliasLookup<'a>> {
        let indices = self.indices.get_or_insert_with(|| {
            let aliases = &self.tracker.pending_ts_namespace_export_aliases;
            let mut indices = HashMap::with_capacity_and_hasher(aliases.len(), FxBuildHasher);
            for (index, alias) in aliases.iter().enumerate() {
                if alias.previous.is_none() {
                    indices.insert(alias.node, index);
                }
            }
            indices
        });
        let dependency = indices[&node];
        let target = match self.states[dependency] {
            TsExportAliasState::Pending => return ControlFlow::Break(dependency),
            TsExportAliasState::Resolved(target) => target,
            // Failed aliases still occupy this public name. Its own declaration
            // meaning remains, but lookup cannot expose an outer namesake.
            state => {
                let result = if let Some(direct) = direct {
                    TsNamespaceExportAliasLookup::Resolved(TsResolvedExportAliasTarget::symbol(
                        direct,
                    ))
                } else if matches!(state, TsExportAliasState::NonLocal) {
                    TsNamespaceExportAliasLookup::NonLocal
                } else {
                    TsNamespaceExportAliasLookup::Missing
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
            return ControlFlow::Continue(TsNamespaceExportAliasLookup::Resolved(target));
        };
        if matches!(target.target, TsExportTarget::Symbols { public, .. } if public == direct) {
            return ControlFlow::Continue(TsNamespaceExportAliasLookup::Resolved(
                TsResolvedExportAliasTarget::symbol(direct),
            ));
        }
        let member = self.member_targets.entry(node).or_insert_with(|| {
            self.tracker.allocator.alloc(TsExportMember {
                direct,
                alias: target,
            })
        });
        ControlFlow::Continue(TsNamespaceExportAliasLookup::Resolved(
            TsResolvedExportAliasTarget::target(TsExportTarget::Member(member)),
        ))
    }
}

struct TsImportAliasSpaceGraph<'a> {
    // Target resolution is shared by graph construction, diagnostics, and
    // contribution finalization. Propagation changes spaces, not these targets.
    targets: Vec<Option<TsResolvedImportTarget<'a>>>,
    public: Vec<TsResolvedAliasSpaces>,
    // Empty until syntax diagnostics request the narrower conflict view.
    // A conflict lookup always has at least one internal alias.
    conflict: Vec<TsResolvedAliasSpaces>,
}

#[derive(Clone, Copy)]
struct TsImportAliasInfo {
    target: TsImportAliasTarget,
    index: usize,
}

// Keep the original target for member fallback even when it cannot serve as
// a namespace prefix. Namespace classification has its own cached result.
#[derive(Clone, Copy)]
enum TsImportTargetResolution<'a> {
    Resolving,
    Resolved {
        target: Option<TsResolvedImportTarget<'a>>,
        namespace: TsNamespaceTargetResolution<'a>,
    },
}

#[derive(Clone, Copy)]
enum TsNamespaceTargetResolution<'a> {
    Resolved(TsResolvedImportTarget<'a>),
    Unknown,
    NotNamespace,
}

impl<'a> TsNamespaceTargetResolution<'a> {
    fn target(self) -> Option<TsResolvedImportTarget<'a>> {
        match self {
            Self::Resolved(target) => Some(target),
            Self::Unknown | Self::NotNamespace => None,
        }
    }
}

/// Resolves internal `import =` targets after all namespace exports are known.
/// The resolver is deliberately ephemeral: public member storage remains split
/// between direct members and export aliases, while this view combines their
/// declaration spaces and preserves type-only provenance for diagnostics.
struct TsImportAliasResolver<'tracker, 'a> {
    tracker: &'tracker SymbolTracker<'a>,
    tree: &'tracker Ast<'a>,
    scopes: &'tracker Scopes,
    scope_views: Option<&'tracker TsScopeViewIndex<'a>>,
    aliases: HashMap<SymbolId, TsImportAliasInfo, FxBuildHasher>,
    alias_targets: Vec<Option<TsImportTargetResolution<'a>>>,
    property_values: TsPropertyValues<'tracker>,
    space_graph: TsImportAliasSpaceGraph<'a>,
}

impl<'tracker, 'a> TsImportAliasResolver<'tracker, 'a> {
    fn new(
        tracker: &'tracker SymbolTracker<'a>,
        tree: &'tracker Ast<'a>,
        scopes: &'tracker Scopes,
        scope_views: Option<&'tracker TsScopeViewIndex<'a>>,
    ) -> Self {
        let mut aliases =
            HashMap::with_capacity_and_hasher(tracker.ts_import_alias_targets.len(), FxBuildHasher);
        for &target in &tracker.ts_import_alias_targets {
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
                &tracker.ts_expando_assignments,
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
                targets: Vec::new(),
                public: Vec::new(),
                conflict: Vec::new(),
            },
        };
        resolver.space_graph = resolver.compute_space_graph();
        resolver
    }

    fn conflict_spaces(&mut self) -> &[TsResolvedAliasSpaces] {
        if self.space_graph.conflict.is_empty() {
            self.space_graph.conflict = self.compute_conflict_spaces();
        }
        &self.space_graph.conflict
    }

    fn compute_conflict_spaces(&self) -> Vec<TsResolvedAliasSpaces> {
        // Diagnostics consume the same resolved targets as public lookup.
        // Build their narrower space graph only if a conflict query needs it.
        let mut conflict = vec![TsResolvedAliasSpaces::empty(); self.aliases.len()];
        for (&symbol, alias) in &self.aliases {
            conflict[alias.index] = self.non_import_spaces(symbol);
        }
        let mut edges = Vec::new();
        for index in 0..self.tracker.ts_import_alias_targets.len() {
            let alias = self.tracker.ts_import_alias_targets[index];
            let dependent = self.aliases[&alias.alias].index;
            let Some(target) = self.space_graph.targets[index] else {
                conflict[dependent] |= TsResolvedAliasSpaces::UNKNOWN;
                continue;
            };
            conflict[dependent] |= target.non_symbol_spaces();
            for dependency in target.conflict_symbols() {
                self.add_space_dependency(&mut conflict, &mut edges, dependent, dependency);
            }
        }
        propagate_alias_spaces(&mut conflict, edges);
        conflict
    }

    fn resolved_target_conflict_spaces(
        &mut self,
        target: TsResolvedImportTarget<'a>,
    ) -> TsResolvedAliasSpaces {
        target.conflict_symbols().fold(
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
        for index in 0..self.tracker.ts_import_alias_targets.len() {
            let alias = self.tracker.ts_import_alias_targets[index];
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
                        .public_symbols()
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

    fn into_public_index(
        self,
        allocator: &'a Allocator,
        symbol_count: usize,
    ) -> TsImportAliasIndex<'a> {
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

    fn compute_space_graph(&mut self) -> TsImportAliasSpaceGraph<'a> {
        let mut public = vec![TsResolvedAliasSpaces::empty(); self.aliases.len()];
        for (&symbol, alias) in &self.aliases {
            public[alias.index] = self.non_import_spaces(symbol);
        }
        let mut targets = Vec::with_capacity(self.tracker.ts_import_alias_targets.len());
        let mut edges = Vec::new();
        for target_index in 0..self.tracker.ts_import_alias_targets.len() {
            let alias = self.tracker.ts_import_alias_targets[target_index];
            let resolved = self.resolve_target(alias);
            targets.push(resolved);
            let dependent = self.aliases[&alias.alias].index;
            let Some(resolved) = resolved else {
                public[dependent] |= TsResolvedAliasSpaces::UNKNOWN;
                continue;
            };
            public[dependent] |= resolved.non_symbol_spaces();
            for dependency in resolved.public_symbols() {
                self.add_space_dependency(&mut public, &mut edges, dependent, dependency);
            }
        }
        propagate_alias_spaces(&mut public, edges);
        TsImportAliasSpaceGraph {
            targets,
            public,
            conflict: Vec::new(),
        }
    }

    fn resolve_target(
        &mut self,
        target: TsImportAliasTarget,
    ) -> Option<TsResolvedImportTarget<'a>> {
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
                        TsImportTargetResolution::Resolved {
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
                    self.record_target_resolution(symbol, TsImportTargetResolution::Resolving);
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

    fn record_target_resolution(
        &mut self,
        symbol: SymbolId,
        resolution: TsImportTargetResolution<'a>,
    ) {
        // Reuse the alias graph's dense indices. Allocate only when a lookup
        // first suspends on an alias dependency.
        if self.alias_targets.is_empty() {
            self.alias_targets.resize(self.aliases.len(), None);
        }
        let index = self.aliases[&symbol].index;
        self.alias_targets[index] = Some(resolution);
    }

    // Yield an uncached alias dependency to the driver instead of
    // recursively resolving it. Completed targets are memoized, so retrying
    // a suspended lookup preserves candidate order and shadowing semantics.
    fn try_resolve_target(
        &self,
        target: TsImportAliasTarget,
    ) -> ControlFlow<SymbolId, Option<TsResolvedImportTarget<'a>>> {
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
    ) -> ControlFlow<SymbolId, Option<TsResolvedImportTarget<'a>>> {
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
    ) -> ControlFlow<SymbolId, Option<TsResolvedImportTarget<'a>>> {
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
        target: TsResolvedImportTarget<'a>,
        name: &[u8],
        namespace: bool,
    ) -> ControlFlow<SymbolId, Option<TsResolvedImportTarget<'a>>> {
        let mut pending = vec![(target.public_contributions(), target.type_only_export)];
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
                let Some(resolution) = self.alias_targets.get(alias.index).and_then(Option::as_ref)
                else {
                    return ControlFlow::Break(contribution.symbol);
                };
                // This is a member fallback, not another namespace-prefix
                // query. A class or function target can supply own exports.
                if let TsImportTargetResolution::Resolved {
                    target: Some(target),
                    ..
                } = resolution
                {
                    pending.push((
                        target.public_contributions(),
                        target.type_only_export.or(type_only_export),
                    ));
                }
            }
        }
        ControlFlow::Continue(None)
    }

    fn public_member(&self, owner: SymbolId, name: &[u8]) -> Option<TsResolvedImportTarget<'a>> {
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
    ) -> ControlFlow<SymbolId, Option<TsResolvedImportTarget<'a>>> {
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
                let view = self.tracker.ts_member_views[index];
                let Some(member) = view.lookup(&self.tracker.ts_member_environments, hash, name)
                else {
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
                    for contribution in alias
                        .target
                        .public_contributions(member.target.type_only_export)
                    {
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
        mut target: TsResolvedImportTarget<'a>,
    ) -> ControlFlow<SymbolId, TsNamespaceTargetResolution<'a>> {
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
                TsExportTarget::Member(member) => TsResolvedImportTarget {
                    direct: Some(member.direct),
                    alias: Some(member.alias),
                    type_only_export: inherited.or(member.alias.type_only_export),
                    property_value: PropertyValue::Absent,
                },
            };
        }
    }

    fn resolve_namespace_meaning(
        &self,
        symbol: SymbolId,
    ) -> ControlFlow<SymbolId, TsNamespaceTargetResolution<'a>> {
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
        if let Some(resolution) = self.alias_targets.get(alias.index).and_then(Option::as_ref) {
            return ControlFlow::Continue(match resolution {
                TsImportTargetResolution::Resolving => TsNamespaceTargetResolution::Unknown,
                TsImportTargetResolution::Resolved { namespace, .. } => *namespace,
            });
        }
        ControlFlow::Break(symbol)
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
        target: TsResolvedImportTarget<'a>,
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
        target.public_contributions().any(|contribution| {
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
        for (index, alias) in self.tracker.ts_import_alias_targets.iter().enumerate() {
            if self.tracker.redeclarations.contains_key(&alias.alias) {
                continue;
            }
            let Some(target) = self.space_graph.targets[index] else {
                continue;
            };
            if target.property_value != PropertyValue::Absent {
                continue;
            }
            let mut symbols = target.public_symbols();
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
