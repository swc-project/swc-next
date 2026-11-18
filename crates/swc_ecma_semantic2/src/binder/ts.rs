use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{TsImportEqualsDeclaration, TsModuleBlock};

use super::*;
use crate::ts::{TsMember, TsMemberTable};

mod resolve;

pub(crate) use resolve::LexicalLookup;

#[derive(Default)]
pub(super) struct TsBindingState {
    implicit_exports: FxHashSet<ScopeId>,
    aliases: FxHashMap<SymbolId, ImportEquals>,
    initializers: FxHashMap<ReferenceId, SymbolId>,
}

struct ImportEquals {
    root: ReferenceId,
    members: Vec<AstUtf8>,
}

impl<'a> SymbolTracker<'a> {
    pub(crate) fn next_reference(&self) -> ReferenceId {
        self.references.next_idx()
    }

    pub(crate) fn record_import_equals(
        &mut self,
        tree: &Ast<'a>,
        node: TsImportEqualsDeclaration,
        root: ReferenceId,
    ) {
        let Some(symbol) = self.symbol_of_node(node.id(tree).node_id()) else {
            return;
        };
        let Some(reference) = self.references.get_mut(root) else {
            return;
        };
        let mut current = node.module_reference(tree).node_id();
        let mut members = Vec::new();
        while let NodeKindData::TsQualifiedName(name) = tree.kind_data(current) {
            members.push(name.right(tree).name(tree));
            current = name.left(tree).node_id();
        }
        if reference.node != current {
            return;
        }
        reference.flags.space = ReferenceSpace::Namespace;
        if node.import_kind(tree) == ImportOrExportKind::Type {
            reference.flags = reference.flags.no_access();
        }
        members.reverse();
        if let std::collections::hash_map::Entry::Vacant(entry) =
            self.ts_state.aliases.entry(symbol)
        {
            entry.insert(ImportEquals { root, members });
            self.ts_state.initializers.insert(root, symbol);
        }
    }

    pub(crate) fn enter_ts_container(
        &mut self,
        scope: ScopeId,
        owner: Option<SymbolId>,
        implicit_exports: bool,
    ) {
        if let Some(owner) = owner {
            self.ts_bindings.container_owners.insert(scope, owner);
            if implicit_exports {
                self.ts_state.implicit_exports.insert(scope);
            }
        }
        self.export_state = ExportState::None;
    }

    pub(crate) fn enter_dotted_namespace(&mut self, scope: ScopeId) {
        self.pending.scope = scope;
        self.export_state = ExportState::Named;
    }

    pub(crate) fn declare_dotted_namespace(
        &mut self,
        tree: &Ast<'a>,
        scopes: &ScopeTree<'a>,
        name: AstUtf8,
        node: NodeId,
    ) {
        self.sync_symbol_tables_from(scopes);
        self.declare(tree, name.into(), node);
    }

    pub(super) fn declare_ts_export(
        &mut self,
        tree: &Ast<'a>,
        mut declaration: PendingDeclaration,
        local: Option<SymbolId>,
    ) -> Option<DeclarationId> {
        let scope = declaration.binding.scope;
        let owner = self.ts_bindings.ts_owner(scope);
        let source_module = tree.is_module() && scope == ScopeId::MODULE;
        if owner.is_none() && !source_module {
            return None;
        }

        let binding = declaration.binding;
        if binding.flags.contains(SymbolFlags::IS_DEFAULT) || !self.ts_direct_export(binding) {
            return None;
        }
        declaration.binding.flags.insert(SymbolFlags::EXPORTED);
        let flags = declaration.binding.flags;
        let is_alias = flags.contains(SymbolFlags::IMPORT_EQUALS);
        let local_name = declaration.name;
        let (export, id) = if let Some(owner) = owner {
            self.declare_ts_member(tree, owner, declaration, false)
        } else {
            let existing =
                local.and_then(|symbol| self.source_export(symbol).or(is_alias.then_some(symbol)));
            self.declare_symbol(tree, declaration, existing, true)
        };

        if local == Some(export) {
            return Some(id);
        }
        if is_alias {
            if local.is_none() {
                self.insert_scope_binding(tree, scope, local_name, export);
            }
        } else {
            let local_flags =
                SymbolFlags::empty().with(SymbolFlags::EXPORT_VALUE, flags.in_value_space());
            let local = match local {
                Some(local) => {
                    self.symbols[local].flags |= local_flags;
                    self.add_redeclaration(local, id);
                    local
                }
                None => {
                    let local = self.symbols.push(Symbol {
                        name: local_name,
                        flags: local_flags,
                        declaration: id,
                    });
                    self.insert_scope_binding(tree, scope, local_name, local);
                    local
                }
            };
            self.ts_bindings.export_symbols.insert(local, export);
        }

        Some(id)
    }

    fn ts_direct_export(&self, binding: PendingBinding) -> bool {
        self.export_state.is_named()
            || (!binding.flags.intersects(SymbolFlags::ANY_IMPORT)
                && self.ts_state.implicit_exports.contains(&binding.scope))
    }

    fn source_export(&self, local: SymbolId) -> Option<SymbolId> {
        if let Some(&export) = self.ts_bindings.export_symbols.get(&local) {
            return Some(export);
        }
        let flags = self.symbols[local].flags;
        flags
            .contains(SymbolFlags::IMPORT_EQUALS | SymbolFlags::EXPORTED)
            .then_some(local)
    }

    pub(crate) fn ts_export_conflict(
        &self,
        tree: &Ast<'_>,
        name: &[u8],
        binding: PendingBinding,
    ) -> Option<(NodeId, SymbolFlags)> {
        if !self.ts_direct_export(binding) || self.export_state.is_default() {
            return None;
        }
        let scope = binding.scope;
        let (symbol, flags) = if let Some(owner) = self.ts_bindings.ts_owner(scope) {
            let member = self.ts_bindings.member(owner, name)?;
            (member.symbol, member.namespace_flags?)
        } else if tree.is_module() && scope == ScopeId::MODULE {
            let symbol = self.source_export(self.own_binding(scope, name)?)?;
            (symbol, self.symbols[symbol].flags)
        } else {
            return None;
        };
        if !flags.intersects(binding.excludes) {
            return None;
        }
        let node = self
            .declarations_of(symbol)
            .find(|declaration| declaration.flags.intersects(binding.excludes))
            .map_or_else(
                || self.first_decl_of(symbol),
                |declaration| declaration.node,
            );
        Some((node, flags))
    }

    pub(crate) fn local_declaration_flags(&self, symbol: SymbolId, scope: ScopeId) -> SymbolFlags {
        let flags = self.symbols[symbol].flags;
        if !self.ts_bindings.export_symbols.contains_key(&symbol) {
            return flags;
        }
        self.declarations_of(symbol)
            .filter(|declaration| declaration.symbol != symbol && declaration.scope == scope)
            .fold(flags, |flags, declaration| flags | declaration.flags)
    }

    fn declarations_of(&self, symbol: SymbolId) -> impl Iterator<Item = Declaration> {
        std::iter::once(self.symbols[symbol].declaration)
            .chain(
                self.redeclarations
                    .get(&symbol)
                    .into_iter()
                    .flatten()
                    .copied(),
            )
            .map(|id| self.declarations[id])
    }

    pub(super) fn declare_ts_member(
        &mut self,
        tree: &Ast<'a>,
        owner: SymbolId,
        declaration: PendingDeclaration,
        enum_member: bool,
    ) -> (SymbolId, DeclarationId) {
        let name = tree.get_wtf8(declaration.name).as_bytes();
        let previous = self.ts_bindings.member(owner, name);
        let (symbol, id) = self.declare_symbol(
            tree,
            declaration,
            previous.map(|member| member.symbol),
            true,
        );
        let table = self
            .ts_bindings
            .container_members
            .entry(owner)
            .or_insert_with(|| TsMemberTable::new_in(self.allocator));
        let member = table.entry(name).or_insert(TsMember {
            symbol,
            enum_flags: None,
            namespace_flags: None,
        });
        let contribution = if enum_member {
            &mut member.enum_flags
        } else {
            &mut member.namespace_flags
        };
        let flags = declaration.binding.flags - SymbolFlags::CONST_ENUM_ONLY_MODULE;
        match *contribution {
            None => *contribution = Some(flags),
            Some(previous) if !previous.intersects(declaration.binding.excludes) => {
                *contribution = Some(merge_symbol_flags(previous, flags));
            }
            _ => {}
        }
        (symbol, id)
    }
}

pub(crate) fn has_explicit_exports(tree: &Ast<'_>, block: TsModuleBlock) -> bool {
    block.body(tree).iter().any(|statement| {
        let statement = tree.get_node_in_sub_range(statement);
        match tree.kind_data(statement.node_id()) {
            NodeKindData::ExportNamedDeclaration(export) => export.declaration(tree).is_none(),
            NodeKindData::ExportAllDeclaration(_)
            | NodeKindData::ExportDefaultDeclaration(_)
            | NodeKindData::TsExportAssignment(_) => true,
            _ => false,
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum NamespaceState {
    NonInstantiated,
    ConstEnumOnly,
    Instantiated,
}

impl NamespaceState {
    pub(super) fn flags(self) -> SymbolFlags {
        match self {
            Self::NonInstantiated => SymbolFlags::empty(),
            Self::ConstEnumOnly => SymbolFlags::VALUE_MODULE | SymbolFlags::CONST_ENUM_ONLY_MODULE,
            Self::Instantiated => SymbolFlags::VALUE_MODULE,
        }
    }
}

pub(super) fn namespace_state(
    tree: &Ast<'_>,
    body: Option<swc_next_ecma_ast::TsModuleBody>,
) -> NamespaceState {
    use NamespaceState::*;

    let Some(body) = body else {
        return NonInstantiated;
    };
    match tree.ts_module_body_data(body) {
        swc_next_ecma_ast::TsModuleBodyData::TsModuleBlock(block) => {
            if !block.directives(tree).is_empty() {
                return Instantiated;
            }
            let mut state = NonInstantiated;
            for statement in block.body(tree).iter() {
                let statement = tree.get_node_in_sub_range(statement).node_id();
                state = state.max(statement_state(tree, statement));
                if state == Instantiated {
                    break;
                }
            }
            state
        }
        swc_next_ecma_ast::TsModuleBodyData::TsModuleDeclaration(child) => {
            namespace_state(tree, child.body(tree))
        }
    }
}

fn statement_state(tree: &Ast<'_>, node: NodeId) -> NamespaceState {
    use NamespaceState::*;

    match tree.kind_data(node) {
        NodeKindData::TsInterfaceDeclaration(_)
        | NodeKindData::TsTypeAliasDeclaration(_)
        | NodeKindData::TsImportEqualsDeclaration(_)
        | NodeKindData::ImportDeclaration(_)
        | NodeKindData::TsGlobalDeclaration(_)
        | NodeKindData::TsNamespaceExportDeclaration(_) => NonInstantiated,
        NodeKindData::TsEnumDeclaration(declaration) if declaration.is_const(tree) => ConstEnumOnly,
        NodeKindData::TsModuleDeclaration(module) => namespace_state(tree, module.body(tree)),
        NodeKindData::ExportNamedDeclaration(export) => {
            if export.export_kind(tree) == ImportOrExportKind::Type {
                NonInstantiated
            } else if let Some(declaration) = export.declaration(tree) {
                match tree.kind_data(declaration.node_id()) {
                    NodeKindData::TsImportEqualsDeclaration(import)
                        if import.import_kind(tree) == ImportOrExportKind::Value =>
                    {
                        Instantiated
                    }
                    _ => statement_state(tree, declaration.node_id()),
                }
            } else if export.source(tree).is_some() {
                Instantiated
            } else {
                NonInstantiated
            }
        }
        NodeKindData::ExportDefaultDeclaration(export) => {
            statement_state(tree, export.declaration(tree).node_id())
        }
        NodeKindData::ExportAllDeclaration(export)
            if export.export_kind(tree) == ImportOrExportKind::Type =>
        {
            NonInstantiated
        }
        _ => Instantiated,
    }
}
