use std::{collections::hash_map::Entry, iter::once, mem::take};

use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
    ClassElement, ClassElementData, Expr, IdentifierName, ModuleExportNameData, PropertyKeyData,
    TsImportEqualsDeclaration, TsModuleBlock, TsModuleBody, TsModuleBodyData,
};

use super::*;
use crate::ts::{TsContainer, TsMember, TsMemberTable};

mod resolve;

pub(crate) use resolve::LexicalLookup;

#[derive(Default)]
pub(super) struct TsBindingState<'a> {
    implicit_exports: FxHashSet<ScopeId>,
    aliases: FxHashMap<SymbolId, Alias>,
    initializers: FxHashMap<ReferenceId, SymbolId>,
    expando_assignments: Vec<ExpandoAssignment>,
    namespace_imports: FxHashSet<&'a str>,
}

struct Alias {
    root: ReferenceId,
    members: Vec<AstUtf8>,
}

struct ExpandoAssignment {
    object: Expr,
    property: IdentifierName,
    scope: ScopeId,
    container: ScopeId,
}

pub(super) enum TsMemberKind {
    Enum,
    Namespace,
    ClassStatic,
    Expando,
}

// TypeScript binding and traversal context.
impl<'a> SymbolTracker<'a> {
    pub(crate) fn enter_ts_container(
        &mut self,
        scope: ScopeId,
        owner: Option<SymbolId>,
        implicit_exports: bool,
    ) {
        if let Some(owner) = owner {
            let ambient = self.in_ambient_context();
            let container = &mut self.ts_bindings.containers;
            container.insert(scope, TsContainer { owner, ambient });
            if implicit_exports {
                self.ts_state.implicit_exports.insert(scope);
            }
        }
        self.export_state = ExportState::None;
    }

    pub(crate) fn enter_ts_dotted_namespace(&mut self, scope: ScopeId, owner: Option<SymbolId>) {
        self.enter_ts_container(scope, owner, false);
        self.pending.scope = scope;
        self.export_state = ExportState::Named;
    }

    pub(crate) fn declare_ts_namespace_segment(
        &mut self,
        tree: &Ast<'a>,
        scopes: &ScopeTree<'a>,
        name: AstUtf8,
        node: NodeId,
    ) {
        self.sync_symbol_tables_from(scopes);
        self.declare(tree, name.into(), node);
    }

    pub(super) fn try_declare_ts_export(
        &mut self,
        tree: &Ast<'a>,
        mut declaration: PendingDeclaration,
    ) -> Option<DeclarationId> {
        let scope = declaration.binding.scope;
        let owner = self.ts_bindings.ts_owner(scope);
        let source_module = tree.is_module() && scope == ScopeId::MODULE;
        if owner.is_none() && !source_module {
            return None;
        }

        let binding = declaration.binding;
        if binding.flags.contains(SymbolFlags::IS_DEFAULT) || !self.is_ts_direct_export(binding) {
            return None;
        }
        declaration.binding.flags.insert(SymbolFlags::EXPORTED);
        let flags = declaration.binding.flags;
        let is_alias = flags.contains(SymbolFlags::IMPORT_EQUALS);
        let local_name = declaration.name;
        let text = tree.get_wtf8(local_name).as_bytes();
        let local = self.own_binding(scope, text).filter(|&id| {
            let flags = self.symbol(id).flags;
            !flags.intersects(SymbolFlags::INTERNAL_NAME)
        });
        let (export, id) = if let Some(owner) = owner {
            self.declare_ts_member(tree, owner, declaration, TsMemberKind::Namespace)
        } else {
            let existing = local.and_then(|symbol| {
                let export = self.source_export_symbol(symbol);
                export.or(is_alias.then_some(symbol))
            });
            let (symbol, id) = self.declare_symbol(tree, declaration, existing);
            self.record_symbol(declaration.node, symbol);
            (symbol, id)
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

    pub(crate) fn declare_ts_namespace_export_alias(
        &mut self,
        tree: &Ast<'a>,
        scopes: &ScopeTree<'a>,
        specifier: ExportSpecifier,
        root: ReferenceId,
        type_only: bool,
    ) {
        if !tree.is_ts() {
            return;
        }
        let scope = self.references[root].scope;
        let Some(container) = self.ts_bindings.containers.get(&scope).copied() else {
            return;
        };
        if !container.ambient || scopes.get(scope).kind != ScopeKind::TsModule {
            return;
        }
        let name = match tree.module_export_name_data(specifier.exported(tree)) {
            ModuleExportNameData::IdentifierName(name) => name.name(tree).into(),
            ModuleExportNameData::StringLiteral(name) => name.value(tree),
        };
        let alias_flags = if type_only {
            SymbolFlags::ALIAS | SymbolFlags::TYPE_ONLY
        } else {
            SymbolFlags::ALIAS
        };
        self.sync_symbol_tables_from(scopes);
        let (symbol, _) = self.declare_ts_member(
            tree,
            container.owner,
            PendingDeclaration {
                name,
                node: specifier.node_id(),
                binding: PendingBinding {
                    flags: alias_flags | SymbolFlags::EXPORTED | SymbolFlags::AMBIENT,
                    excludes: SymbolFlags::ALIAS,
                    scope,
                },
            },
            TsMemberKind::Namespace,
        );
        self.record_alias(symbol, root, Vec::new());
    }

    pub(crate) fn record_ts_import_equals_alias(
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
        self.record_alias(symbol, root, members);
    }

    fn record_alias(&mut self, symbol: SymbolId, root: ReferenceId, members: Vec<AstUtf8>) {
        if let Entry::Vacant(entry) = self.ts_state.aliases.entry(symbol) {
            entry.insert(Alias { root, members });
            self.ts_state.initializers.insert(root, symbol);
        }
    }

    pub(super) fn declare_ts_member(
        &mut self,
        tree: &Ast<'a>,
        owner: SymbolId,
        declaration: PendingDeclaration,
        kind: TsMemberKind,
    ) -> (SymbolId, DeclarationId) {
        let name = tree.get_wtf8(declaration.name).as_bytes();
        let member = self.ts_bindings.member(owner, name);
        let existing = member.map(|member| member.symbol);
        let (symbol, id) = self.declare_symbol(tree, declaration, existing);
        self.record_symbol(declaration.node, symbol);
        let table = self
            .ts_bindings
            .container_members
            .entry(owner)
            .or_insert_with(|| TsMemberTable::new_in(self.allocator));
        let member = table.entry(name).or_insert(TsMember {
            symbol,
            enum_flags: None,
            namespace_flags: None,
            class_static_flags: None,
            expando: false,
        });
        let contribution = match kind {
            TsMemberKind::Enum => &mut member.enum_flags,
            TsMemberKind::Namespace => &mut member.namespace_flags,
            TsMemberKind::ClassStatic => &mut member.class_static_flags,
            TsMemberKind::Expando => {
                member.expando = true;
                return (symbol, id);
            }
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

    pub(crate) fn declare_ts_class_static_member(
        &mut self,
        tree: &Ast<'a>,
        node: ClassElement,
        scopes: &ScopeTree<'a>,
        scope: ScopeId,
    ) {
        let (key, declared) = match tree.class_element_data(node) {
            ClassElementData::PropertyDefinition(property)
                if property.r#static(tree) && !property.computed(tree) =>
            {
                (
                    property.key(tree),
                    property.declare(tree) || property.r#abstract(tree),
                )
            }
            ClassElementData::MethodDefinition(method)
                if method.r#static(tree) && !method.computed(tree) =>
            {
                (method.key(tree), false)
            }
            ClassElementData::TsMethodDefinition(method)
                if method.r#static(tree) && !method.computed(tree) =>
            {
                (method.key(tree), method.r#abstract(tree))
            }
            _ => return,
        };
        let name = match tree.kind_data(key.node_id()) {
            NodeKindData::IdentifierName(identifier) => identifier.name(tree).into(),
            NodeKindData::StringLiteral(literal) => literal.value(tree),
            _ => return,
        };
        let NodeKindData::Class(class) = tree.kind_data(scopes.get(scope).node) else {
            return;
        };
        let Some(owner) = class
            .id(tree)
            .and_then(|id| self.symbol_of_node(id.node_id()))
        else {
            return;
        };
        self.sync_symbol_tables_from(scopes);
        self.declare_ts_member(
            tree,
            owner,
            PendingDeclaration {
                name,
                node: node.node_id(),
                binding: PendingBinding {
                    flags: SymbolFlags::PROPERTY
                        .with(SymbolFlags::AMBIENT, declared || self.in_ambient_context()),
                    excludes: SymbolFlags::empty(),
                    scope,
                },
            },
            TsMemberKind::ClassStatic,
        );
    }

    pub(crate) fn record_ts_expando_assignment(
        &mut self,
        object: Expr,
        property: IdentifierName,
        scope: ScopeId,
        container: ScopeId,
    ) {
        self.ts_state.expando_assignments.push(ExpandoAssignment {
            object,
            property,
            scope,
            container,
        });
    }

    pub(super) fn declare_ts_expando_members(&mut self, tree: &Ast<'a>, scopes: &ScopeTree<'_>) {
        // The receiver can be declared after the write. Bind named members once
        // all declaration tables exist, before resolving import aliases.
        let mut path = Vec::new();
        for assignment in take(&mut self.ts_state.expando_assignments) {
            path.clear();
            let mut object = assignment.object.node_id();
            while let NodeKindData::MemberExpression(member) = tree.kind_data(object) {
                let PropertyKeyData::IdentifierName(property) =
                    tree.property_key_data(member.property(tree))
                else {
                    break;
                };
                if member.computed(tree) || member.optional(tree) {
                    break;
                }
                path.push(property.name(tree));
                object = member.object(tree).node_id();
            }
            let NodeKindData::IdentifierReference(root) = tree.kind_data(object) else {
                continue;
            };
            let root = tree.get_utf8(root.name(tree)).as_bytes();
            let mut receiver = self.expando_receiver(tree, scopes, assignment.scope, root, &path);
            if receiver.is_none() && assignment.scope != assignment.container {
                receiver = self.expando_receiver(tree, scopes, assignment.container, root, &path);
            }
            let Some(owner) = receiver.filter(|&symbol| self.is_expando_function(symbol)) else {
                continue;
            };

            let name = assignment.property.name(tree);
            if self
                .ts_bindings
                .member(owner, tree.get_utf8(name).as_bytes())
                .is_some_and(|member| !member.expando)
            {
                continue;
            }
            self.declare_ts_member(
                tree,
                owner,
                PendingDeclaration {
                    name: name.into(),
                    node: assignment.property.node_id(),
                    binding: PendingBinding {
                        flags: SymbolFlags::PROPERTY,
                        excludes: SymbolFlags::empty(),
                        scope: assignment.scope,
                    },
                },
                TsMemberKind::Expando,
            );
        }
    }
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

// Redeclaration checks, lookups, and namespace classification.
impl<'a> SymbolTracker<'a> {
    fn is_ts_direct_export(&self, binding: PendingBinding) -> bool {
        self.export_state.is_named()
            || (!binding.flags.intersects(SymbolFlags::IMPORT)
                && self.ts_state.implicit_exports.contains(&binding.scope))
    }

    fn source_export_symbol(&self, local: SymbolId) -> Option<SymbolId> {
        if let Some(&export) = self.ts_bindings.export_symbols.get(&local) {
            return Some(export);
        }
        let flags = self.symbols[local].flags;
        flags
            .contains(SymbolFlags::IMPORT_EQUALS | SymbolFlags::EXPORTED)
            .then_some(local)
    }

    /// Finds a conflict for an explicit or implicit named TypeScript export.
    /// Namespace blocks share a member table, so the conflict can come from
    /// another block of the same namespace. The redeclaration check uses the
    /// returned node and flags for its diagnostic.
    pub(crate) fn ts_export_conflict(
        &self,
        tree: &Ast<'_>,
        name: &[u8],
        binding: PendingBinding,
    ) -> Option<(NodeId, SymbolFlags)> {
        if !self.is_ts_direct_export(binding) || self.export_state.is_default() {
            return None;
        }
        let scope = binding.scope;
        let (symbol, flags) = if let Some(owner) = self.ts_bindings.ts_owner(scope) {
            let member = self.ts_bindings.member(owner, name)?;
            (member.symbol, member.namespace_flags?)
        } else if tree.is_module() && scope == ScopeId::MODULE {
            let symbol = self.source_export_symbol(self.own_binding(scope, name)?)?;
            (symbol, self.symbols[symbol].flags)
        } else {
            return None;
        };
        if !flags.intersects(binding.excludes) {
            return None;
        }
        for declaration in self.declarations_of(symbol) {
            if declaration.flags.intersects(binding.excludes) {
                return Some((declaration.node, flags));
            }
        }
        Some((self.first_decl_of(symbol), flags))
    }

    /// Returns the flags of an existing binding for the local redeclaration check.
    /// For a local symbol linked to an export, this also includes the flags of
    /// attached export declarations from the same scope.
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
        once(self.symbols[symbol].declaration)
            .chain(
                self.redeclarations
                    .get(&symbol)
                    .into_iter()
                    .flatten()
                    .copied(),
            )
            .map(|id| self.declarations[id])
    }

    fn expando_receiver(
        &self,
        tree: &Ast<'_>,
        scopes: &ScopeTree<'_>,
        scope: ScopeId,
        root: &[u8],
        path: &[AstUtf8],
    ) -> Option<SymbolId> {
        let local = self.own_binding(scope, root).or_else(|| {
            let scope = scopes.get(scope);
            if scope.kind == ScopeKind::FunctionBody {
                self.own_binding(scope.parent.unwrap(), root)
            } else {
                None
            }
        });
        let mut symbol = if let Some(local) = local {
            self.ts_bindings
                .export_symbols
                .get(&local)
                .copied()
                .unwrap_or(local)
        } else {
            let owner = self.ts_bindings.ts_owner(scope)?;
            self.ts_bindings.member(owner, root)?.symbol
        };

        for &name in path.iter().rev() {
            if !self.is_expando_function(symbol) {
                return None;
            }
            symbol = self
                .ts_bindings
                .member(symbol, tree.get_utf8(name).as_bytes())?
                .symbol;
        }
        Some(symbol)
    }

    fn is_expando_function(&self, symbol: SymbolId) -> bool {
        let flags = self.symbols[symbol].flags;
        flags.contains(SymbolFlags::FUNCTION) && !flags.contains(SymbolFlags::FUNCTION_EXPRESSION)
    }

    pub(super) fn namespace_state(
        &mut self,
        tree: &Ast<'a>,
        body: Option<TsModuleBody>,
        ambient: bool,
    ) -> NamespaceState {
        use NamespaceState::*;

        let Some(body) = body else {
            return NonInstantiated;
        };
        let block = match tree.ts_module_body_data(body) {
            TsModuleBodyData::TsModuleBlock(block) => block,
            TsModuleBodyData::TsModuleDeclaration(child) => {
                return self.namespace_state(
                    tree,
                    child.body(tree),
                    ambient || child.declare(tree),
                );
            }
        };
        if !block.directives(tree).is_empty() {
            return Instantiated;
        }

        let mut state = NonInstantiated;
        let mut has_local_exports = false;
        for id in block.body(tree).iter() {
            let statement = tree.kind_data(tree.get_node_in_sub_range(id).node_id());
            state = state.max(self.statement_state(tree, statement, ambient));
            if state == Instantiated {
                return state;
            }
            if ambient
                && !has_local_exports
                && let NodeKindData::ExportNamedDeclaration(export) = statement
            {
                has_local_exports = export.export_kind(tree) == ImportOrExportKind::Value
                    && export.source(tree).is_none()
                    && !export.specifiers(tree).is_empty();
            }
        }
        if has_local_exports && self.exports_local_import_equals(tree, block) {
            return Instantiated;
        }
        state
    }

    fn statement_state(
        &mut self,
        tree: &Ast<'a>,
        statement: NodeKindData,
        ambient: bool,
    ) -> NamespaceState {
        use NamespaceState::*;

        match statement {
            NodeKindData::TsInterfaceDeclaration(_)
            | NodeKindData::TsTypeAliasDeclaration(_)
            | NodeKindData::TsImportEqualsDeclaration(_)
            | NodeKindData::ImportDeclaration(_)
            | NodeKindData::TsGlobalDeclaration(_)
            | NodeKindData::TsNamespaceExportDeclaration(_) => NonInstantiated,
            NodeKindData::TsEnumDeclaration(declaration) if declaration.is_const(tree) => {
                ConstEnumOnly
            }
            NodeKindData::TsModuleDeclaration(module) => {
                self.namespace_state(tree, module.body(tree), ambient || module.declare(tree))
            }
            NodeKindData::ExportNamedDeclaration(export) => {
                if export.export_kind(tree) == ImportOrExportKind::Type {
                    return NonInstantiated;
                }
                let Some(declaration) = export.declaration(tree) else {
                    if export.source(tree).is_some() {
                        return Instantiated;
                    }
                    return NonInstantiated;
                };
                let declaration = tree.kind_data(declaration.node_id());
                if let NodeKindData::TsImportEqualsDeclaration(import) = declaration
                    && import.import_kind(tree) == ImportOrExportKind::Value
                {
                    return Instantiated;
                }
                self.statement_state(tree, declaration, ambient)
            }
            NodeKindData::ExportDefaultDeclaration(export) => {
                let declaration = tree.kind_data(export.declaration(tree).node_id());
                self.statement_state(tree, declaration, ambient)
            }
            NodeKindData::ExportAllDeclaration(export)
                if export.export_kind(tree) == ImportOrExportKind::Type =>
            {
                NonInstantiated
            }
            _ => Instantiated,
        }
    }

    // A local import-equals may supply the value of an ambient namespace's export.
    // Classify it before binding, as with a directly exported import-equals.
    fn exports_local_import_equals(&mut self, tree: &Ast<'a>, block: TsModuleBlock) -> bool {
        let imports = &mut self.ts_state.namespace_imports;
        imports.clear();
        let statements = block.body(tree);
        for id in statements.iter() {
            let node = tree.get_node_in_sub_range(id).node_id();
            if let NodeKindData::TsImportEqualsDeclaration(import) = tree.kind_data(node)
                && import.import_kind(tree) == ImportOrExportKind::Value
            {
                imports.insert(tree.get_utf8(import.id(tree).name(tree)));
            }
        }
        if imports.is_empty() {
            return false;
        }

        for id in statements.iter() {
            let node = tree.get_node_in_sub_range(id).node_id();
            let NodeKindData::ExportNamedDeclaration(export) = tree.kind_data(node) else {
                continue;
            };
            if export.export_kind(tree) == ImportOrExportKind::Type || export.source(tree).is_some()
            {
                continue;
            }
            for id in export.specifiers(tree).iter() {
                let specifier = tree.get_node_in_sub_range(id);
                if specifier.export_kind(tree) == ImportOrExportKind::Type {
                    continue;
                }
                if let Some(local) = specifier.local(tree).as_identifier_name(tree)
                    && imports.contains(tree.get_utf8(local.name(tree)))
                {
                    return true;
                }
            }
        }
        false
    }
}
