use std::{
    collections::hash_map::Entry, convert::Infallible, hash::BuildHasher, iter::once, mem::take,
};

use bitflags::bitflags;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_allocator::IndexSlice;
use swc_next_ecma_ast::{
    ClassElement, ClassElementData, Expr, IdentifierName, ModuleExportNameData, PropertyKeyData,
    TsImportEqualsDeclaration, TsModuleBlock, TsModuleBody, TsModuleBodyData,
};

use super::*;
use crate::{
    Scopes, SymbolTables, Symbols,
    ts::{TsAlias, TsContainer, TsMember, TsMemberTable},
};

/// Temporary TypeScript binding data used to collect declarations and resolve aliases.
#[derive(Default)]
pub(super) struct TsBindingState<'a> {
    /// Ambient namespace scopes whose declarations can be exported implicitly.
    implicit_exports: FxHashSet<ScopeId>,
    /// Unresolved targets keyed by alias symbol. In `import A = N.E`,
    /// the symbol ID of `A` maps to the reference to `N` and the member name `E`.
    aliases: FxHashMap<SymbolId, Alias>,
    /// Maps each initializer's root reference ID to its alias symbol ID.
    /// In `import A = N.E`, the reference ID of `N` maps to the symbol ID of `A`.
    initializers: FxHashMap<ReferenceId, SymbolId>,
    /// Static properties without symbol IDs. `None` means a computed key may match any name.
    unbound_class_statics: FxHashMap<SymbolId, Option<FxHashSet<String>>>,
    /// Candidate property writes to bind after collecting declarations.
    expando_assignments: Vec<ExpandoAssignment>,
    /// Reusable set of local value `import =` names for one ambient namespace block.
    namespace_imports: FxHashSet<&'a str>,
}

/// The unresolved target of an internal import-equals or a namespace export alias.
struct Alias {
    /// Reference to the leftmost identifier, such as `N` in `import A = N.E`.
    root: ReferenceId,
    /// Member names after the root in lookup order, such as `E` in `import A = N.E`.
    members: Vec<AstUtf8>,
}

/// A dotted assignment that may add a property to a function declaration.
struct ExpandoAssignment {
    /// Receiver expression, such as `a.b` in `a.b.c = value`.
    object: Expr,
    /// Identifier of the assigned property, such as `c` in `a.b.c = value`.
    property: IdentifierName,
    /// Lexical scope of the assignment and the first scope searched.
    scope: ScopeId,
    /// Nearest non-block scope used as a fallback for receiver lookup.
    container: ScopeId,
}

/// How a declaration contributes to a TypeScript container member.
pub(super) enum TsMemberKind {
    /// An enum member declaration.
    Enum,
    /// An explicit or implicit namespace export.
    Namespace,
    /// A named static class member.
    ClassStatic,
    /// A function property introduced by a dotted assignment.
    Expando,
}

fn is_late_bindable_name(tree: &Ast<'_>, mut node: NodeId) -> bool {
    // Identifiers and dotted paths can name static members through their types.
    loop {
        node = match tree.kind_data(node) {
            NodeKindData::IdentifierReference(_) => return true,
            NodeKindData::MemberExpression(member) if !member.computed(tree) => {
                member.object(tree).node_id()
            }
            NodeKindData::ParenthesizedExpression(parentheses) => {
                parentheses.expression(tree).node_id()
            }
            NodeKindData::ChainExpression(chain) => chain.expression(tree).node_id(),
            _ => return false,
        };
    }
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
        let (key, computed, declared) = match tree.class_element_data(node) {
            ClassElementData::PropertyDefinition(property) if property.r#static(tree) => (
                property.key(tree),
                property.computed(tree),
                property.declare(tree) || property.r#abstract(tree),
            ),
            ClassElementData::MethodDefinition(method) if method.r#static(tree) => {
                (method.key(tree), method.computed(tree), false)
            }
            ClassElementData::TsMethodDefinition(method) if method.r#static(tree) => (
                method.key(tree),
                method.computed(tree),
                method.r#abstract(tree),
            ),
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
        let name = match tree.kind_data(key.node_id()) {
            NodeKindData::IdentifierName(identifier) if !computed => identifier.name(tree).into(),
            NodeKindData::StringLiteral(literal) => literal.value(tree),
            NodeKindData::NumericLiteral(literal) => {
                if let Some(names) = self
                    .ts_state
                    .unbound_class_statics
                    .entry(owner)
                    .or_insert_with(|| Some(FxHashSet::default()))
                {
                    let mut buffer = zmij_ecma::Buffer::new();
                    names.insert(buffer.format(literal.value(tree)).to_owned());
                }
                return;
            }
            _ if computed && is_late_bindable_name(tree, key.node_id()) => {
                self.ts_state.unbound_class_statics.insert(owner, None);
                return;
            }
            _ => return,
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

    #[cold]
    #[inline(never)]
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

/// A namespace body's contribution to the namespace's value binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum NamespaceState {
    /// The body contributes no value binding.
    NonInstantiated,
    /// The body's value contributions come only from const enums.
    ConstEnumOnly,
    /// The body contributes a value binding independently of const enums.
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
        let export = self.export_state.is_named();
        let import = binding.flags.intersects(SymbolFlags::IMPORT);
        let implicit_import = self.ts_state.implicit_exports.contains(&binding.scope);
        export || (!import && implicit_import)
    }

    fn source_export_symbol(&self, local: SymbolId) -> Option<SymbolId> {
        if let Some(&export) = self.ts_bindings.export_symbols.get(&local) {
            return Some(export);
        }
        let flags = self.symbols[local].flags;
        if flags.contains(SymbolFlags::IMPORT_EQUALS | SymbolFlags::EXPORTED) {
            return Some(local);
        }
        None
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

        let mut symbol = match local {
            Some(local) => {
                let export_symbols = self.ts_bindings.export_symbols.get(&local);
                export_symbols.copied().unwrap_or(local)
            }
            None => {
                let owner = self.ts_bindings.ts_owner(scope)?;
                self.ts_bindings.member(owner, root)?.symbol
            }
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

impl SymbolTracker<'_> {
    pub(super) fn resolve_references(&mut self, tree: &Ast<'_>, scopes: &ScopeTree<'_>) {
        if !self.ts_state.aliases.is_empty() {
            self.resolve_references_with_aliases(tree, scopes);
            return;
        }

        let lookup = LexicalLookup {
            scopes: &scopes.list,
            symbols: &self.symbols,
            symbol_tables: &self.symbol_tables,
            shadowed_internal_names: &self.shadowed_internal_names,
            ts: &self.ts_bindings,
        };

        // Recovered or edited JavaScript ASTs can contain TypeScript bindings.
        let all_value_bindings = !tree.is_ts()
            && !self.references.is_empty()
            && self.ts_bindings.containers.is_empty()
            && self.symbols.iter().all(|symbol| {
                symbol.flags.contains(SymbolFlags::EXPORT_VALUE)
                    || symbol.flags.visible_in(ReferenceSpace::Value)
            });

        // Without recorded alias targets, visibility follows declaration flags.
        for reference in self.references.iter_mut() {
            let name = tree.get_utf8(reference.name).as_bytes();
            let space = reference.flags.space;
            // An implicit arguments object can hide an outer binding.
            if all_value_bindings && space == ReferenceSpace::Value && name != b"arguments" {
                reference.symbol = lookup.lookup_javascript_value(reference.scope, name);
                continue;
            }
            let binding = lookup.lookup(reference.scope, name, space);
            reference.symbol = binding.map(|(symbol, _)| symbol);
            if space == ReferenceSpace::Any
                && reference.flags.is_read()
                && let Some((_, flags)) = binding
                && !flags.contains(SymbolFlags::EXPORT_VALUE)
                && !flags.visible_in(ReferenceSpace::Value)
            {
                reference.flags = reference.flags.no_access();
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn resolve_references_with_aliases(&mut self, tree: &Ast<'_>, scopes: &ScopeTree<'_>) {
        let lookup = LexicalLookup {
            scopes: &scopes.list,
            symbols: &self.symbols,
            symbol_tables: &self.symbol_tables,
            shadowed_internal_names: &self.shadowed_internal_names,
            ts: &self.ts_bindings,
        };

        // Resolve alias dependencies once, before deciding which bindings each reference can see.
        let mut resolver = AliasResolver {
            lookup: &lookup,
            tree,
            aliases: &self.ts_state.aliases,
            unbound_class_statics: &self.ts_state.unbound_class_statics,
            resolved: FxHashMap::default(),
        };
        resolver.resolve_all(&self.references);

        // Keep the identity found at the use site. The reverse index is built from these IDs.
        for (id, reference) in self.references.iter_mut_enumerated() {
            let initializer = self.ts_state.initializers.get(&id).copied();
            resolver.resolve_reference(reference, initializer);
        }

        // Only direct alias links outlive the binder.
        for (symbol, alias) in resolver.resolved {
            self.ts_bindings.aliases.insert(symbol, alias.target);
        }
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
    fn lookup_javascript_value(&self, start: ScopeId, name: &[u8]) -> Option<SymbolId> {
        let hash = self.symbol_tables[ScopeId::ROOT].hasher().hash_one(name);
        let mut scope = Some(start);
        while let Some(id) = scope {
            if let Some((_, &symbol)) = self.symbol_tables[id]
                .raw_entry()
                .from_hash(hash, |candidate| *candidate == name)
            {
                return Some(symbol);
            }
            scope = self.scopes[id].parent;
        }
        None
    }

    pub(crate) fn lookup(
        &self,
        scope: ScopeId,
        name: &[u8],
        space: ReferenceSpace,
    ) -> Option<(SymbolId, SymbolFlags)> {
        self.lookup_inner(scope, name, space, |_, flags| {
            Ok::<_, Infallible>(flags.visible_in(space))
        })
        .unwrap()
    }

    /// Walk outwards until a binding contributes the requested meaning.
    /// The visibility check can suspend alias resolution on a dependency,
    /// as in `import A = B.C` while `import B = N` is still unresolved.
    fn lookup_inner<E>(
        &self,
        start: ScopeId,
        name: &[u8],
        space: ReferenceSpace,
        mut visible: impl FnMut(SymbolId, SymbolFlags) -> Result<bool, E>,
    ) -> Result<Option<(SymbolId, SymbolFlags)>, E> {
        let arguments = name == b"arguments"
            && matches!(
                space,
                ReferenceSpace::Value | ReferenceSpace::Typeof | ReferenceSpace::Any
            );
        let hash = self.symbol_tables[ScopeId::ROOT].hasher().hash_one(name);
        for id in successors(Some(start), |&id| self.scopes[id].parent) {
            let scope = self.scopes[id];
            let barrier = arguments && scope.flags.arguments_barrier;
            if let Some((_, &local)) = self.symbol_tables[id]
                .raw_entry()
                .from_hash(hash, |candidate| *candidate == name)
            {
                let flags = self.symbols[local].flags;

                // The implicit arguments object hides the function expression's name.
                // `const f = function arguments() { return arguments; };`
                if barrier && flags.intersects(SymbolFlags::INTERNAL_NAME) {
                    return Ok(None);
                }

                // The local binding matches the requested space.
                // `const x = 1; x;` or `export const x = 1; x;`
                let exported_value = flags.contains(SymbolFlags::EXPORT_VALUE)
                    && matches!(space, ReferenceSpace::Value | ReferenceSpace::Typeof);
                if exported_value || visible(local, flags)? {
                    return Ok(Some((local, flags)));
                }

                // The hidden name still supplies the requested meaning.
                // `const f = function n() { type n = string; return n; };`
                if let Some(&internal) = self.shadowed_internal_names.get(&local) {
                    let flags = self.symbols[internal].flags;
                    if !barrier && visible(internal, flags)? {
                        return Ok(Some((internal, flags)));
                    }
                }

                // The module's export symbol supplies the missing meaning.
                // `export interface T {} type U = T;`
                if scope.kind == ScopeKind::Module
                    && let Some(&export) = self.ts.export_symbols.get(&local)
                {
                    let flags = self.symbols[export].flags;
                    if visible(export, flags)? {
                        return Ok(Some((export, flags)));
                    }
                }
            }

            // The arguments barrier blocks lookup in outer scopes.
            // `var arguments; function f() { return arguments; }`
            if barrier {
                return Ok(None);
            }

            // A shared TS member is visible across merged bodies.
            // `enum E { A = 1 } enum E { B = A }`
            // `namespace N { export type T = string; } namespace N { type U = T; }`
            if matches!(scope.kind, ScopeKind::TsEnum | ScopeKind::TsModule)
                && let Some((symbol, flags)) = self.lookup_container_member(id, scope.kind, name)
                && visible(symbol, flags)?
            {
                std::hint::cold_path();
                return Ok(Some((symbol, flags)));
            }
        }
        Ok(None)
    }

    #[cold]
    #[inline(never)]
    fn lookup_container_member(
        &self,
        scope: ScopeId,
        kind: ScopeKind,
        name: &[u8],
    ) -> Option<(SymbolId, SymbolFlags)> {
        let container = self.ts.containers.get(&scope)?;
        let member = self.ts.member(container.owner, name)?;
        // Merged bodies share only the contributions of their own kind.
        let flags = match kind {
            ScopeKind::TsEnum => member.enum_flags,
            ScopeKind::TsModule => member.namespace_flags,
            _ => None,
        }?;
        if container.ambient
            && flags.contains(SymbolFlags::ALIAS)
            && !flags.intersects(
                SymbolFlags::IMPORT_EQUALS
                    | SymbolFlags::VALUE_SPACE
                    | SymbolFlags::TYPE_SPACE
                    | SymbolFlags::NAMESPACE_SPACE,
            )
        {
            // A bare name can see a merged declaration with its alias meanings.
            // An export specifier on its own only introduces a public name.
            return None;
        }
        Some((member.symbol, flags))
    }
}

bitflags! {
    #[derive(Clone, Copy)]
    struct Spaces: u8 {
        const VALUE = 1;
        const TYPE = 2;
        const NAMESPACE = 4;
        const TYPEOF = 8;
    }
}

impl Spaces {
    fn declared(flags: SymbolFlags) -> Self {
        let mut spaces = Self::empty();
        spaces.set(Self::VALUE | Self::TYPEOF, flags.in_value_space());
        spaces.set(Self::TYPE, flags.in_type_space());
        spaces.set(Self::NAMESPACE, flags.in_namespace_space());
        spaces
    }

    fn visible_in(self, space: ReferenceSpace) -> bool {
        match space {
            ReferenceSpace::Value => self.contains(Self::VALUE),
            ReferenceSpace::Type => self.contains(Self::TYPE),
            ReferenceSpace::Namespace => self.contains(Self::NAMESPACE),
            ReferenceSpace::Typeof => self.contains(Self::TYPEOF),
            ReferenceSpace::Any => true,
        }
    }
}

#[derive(Clone, Copy)]
struct QualifiedOwner {
    symbol: SymbolId,
    type_only: bool,
}

/// The direct target is public. The remaining fields describe what a use
/// through this particular alias path can see.
#[derive(Clone, Copy)]
struct ResolvedAlias {
    target: TsAlias,
    spaces: Spaces,
    namespace_owner: Option<QualifiedOwner>,
    property_owner: Option<QualifiedOwner>,
    /// Restriction to carry when no owner is known for a qualified lookup.
    type_only: bool,
}

impl ResolvedAlias {
    const UNKNOWN: Self = Self {
        target: TsAlias::Unknown,
        spaces: Spaces::all(),
        namespace_owner: None,
        property_owner: None,
        type_only: false,
    };

    const UNBOUND_PROPERTY: Self = Self {
        spaces: Spaces::VALUE.union(Spaces::TYPEOF),
        ..Self::UNKNOWN
    };

    fn restrict_value(&mut self) {
        self.type_only = true;
        self.spaces.remove(Spaces::VALUE);
        for owner in [&mut self.namespace_owner, &mut self.property_owner]
            .into_iter()
            .flatten()
        {
            owner.type_only = true;
        }
    }
}

struct AliasResolver<'s, 'a> {
    lookup: &'s LexicalLookup<'s, 'a>,
    tree: &'s Ast<'a>,
    aliases: &'s FxHashMap<SymbolId, Alias>,
    unbound_class_statics: &'s FxHashMap<SymbolId, Option<FxHashSet<String>>>,
    resolved: FxHashMap<SymbolId, ResolvedAlias>,
}

impl AliasResolver<'_, '_> {
    fn resolve_all(&mut self, references: &IndexSlice<ReferenceId, [Reference]>) {
        let mut stack = Vec::new();
        let mut active = FxHashSet::default();
        for &symbol in self.aliases.keys() {
            if self.resolved.contains_key(&symbol) {
                continue;
            }
            stack.push(symbol);
            active.insert(symbol);
            while let Some(&current) = stack.last() {
                let alias = &self.aliases[&current];
                match self.resolve_alias(alias, references[alias.root]) {
                    Ok(result) => {
                        self.resolved.insert(current, result);
                        stack.pop();
                        active.remove(&current);
                    }
                    // Finish a forward dependency, then retry the suspended alias.
                    Err(dependency) if active.insert(dependency) => stack.push(dependency),
                    Err(dependency) => {
                        // Every alias on this cycle has an unknown target.
                        while let Some(cyclic) = stack.pop() {
                            self.resolved.insert(cyclic, ResolvedAlias::UNKNOWN);
                            active.remove(&cyclic);
                            if cyclic == dependency {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn find(
        &self,
        reference: &Reference,
        space: ReferenceSpace,
    ) -> Result<Option<(SymbolId, SymbolFlags)>, SymbolId> {
        let name = self.tree.get_utf8(reference.name).as_bytes();
        self.lookup
            .lookup_inner(reference.scope, name, space, |symbol, flags| {
                if space == ReferenceSpace::Any || !flags.contains(SymbolFlags::ALIAS) {
                    return Ok(flags.visible_in(space));
                }
                Ok(self.meaning(symbol, flags)?.spaces.visible_in(space))
            })
    }

    fn root(
        &self,
        reference: &Reference,
        bare: bool,
    ) -> Result<Option<(SymbolId, SymbolFlags)>, SymbolId> {
        let space = reference.flags.space;
        let binding = self.find(reference, space)?;
        if binding.is_none() && bare && space == ReferenceSpace::Namespace {
            return self.find(reference, ReferenceSpace::Any);
        }
        Ok(binding)
    }

    fn resolve_alias(
        &self,
        alias: &Alias,
        reference: Reference,
    ) -> Result<ResolvedAlias, SymbolId> {
        // Import-equals prefers a namespace root. A bare name may denote any declaration.
        let Some((symbol, flags)) = self.root(&reference, alias.members.is_empty())? else {
            return Ok(ResolvedAlias::UNKNOWN);
        };
        let mut result = self.meaning(symbol, flags)?;
        if let Some(&export) = self.lookup.ts.export_symbols.get(&symbol) {
            // Keep the local identity while including the public declaration's meanings.
            let exported = self.meaning(export, self.lookup.symbols[export].flags)?;
            result.spaces |= exported.spaces;
            result.namespace_owner = result.namespace_owner.or(exported.namespace_owner);
            result.property_owner = result.property_owner.or(exported.property_owner);
            result.type_only |= exported.type_only;
        }
        for &name in &alias.members {
            result = self.qualified_member(result, self.tree.get_utf8(name))?;
        }
        Ok(result)
    }

    fn qualified_member(
        &self,
        qualifier: ResolvedAlias,
        name: &str,
    ) -> Result<ResolvedAlias, SymbolId> {
        // Search the namespace owner before the aliased class or function.
        for owner in [qualifier.namespace_owner, qualifier.property_owner]
            .into_iter()
            .flatten()
        {
            if let Some(mut result) = self.member_meaning(owner.symbol, name)? {
                if owner.type_only {
                    result.restrict_value();
                }
                return Ok(result);
            }
        }

        let mut result = ResolvedAlias::UNKNOWN;
        let type_only = qualifier
            .namespace_owner
            .or(qualifier.property_owner)
            .map_or(qualifier.type_only, |owner| owner.type_only);
        if type_only {
            result.restrict_value();
        }
        Ok(result)
    }

    fn member_meaning(
        &self,
        owner: SymbolId,
        name: &str,
    ) -> Result<Option<ResolvedAlias>, SymbolId> {
        let member = self.lookup.ts.member(owner, name.as_bytes());
        let unbound_property = self
            .unbound_class_statics
            .get(&owner)
            .is_some_and(|names| names.as_ref().is_none_or(|names| names.contains(name)));
        match (member, unbound_property) {
            (None, false) => Ok(None),
            (None, true) => Ok(Some(ResolvedAlias::UNBOUND_PROPERTY)),
            (Some(member), _) => {
                let mut result = self.meaning(member.symbol, member.flags())?;
                if unbound_property {
                    // The declared member supplies its own meanings, but its
                    // symbol alone does not account for the static property.
                    result.spaces |= ResolvedAlias::UNBOUND_PROPERTY.spaces;
                    result.target = TsAlias::Unknown;
                }
                Ok(Some(result))
            }
        }
    }

    fn meaning(&self, symbol: SymbolId, flags: SymbolFlags) -> Result<ResolvedAlias, SymbolId> {
        // Resolve only the alias contribution. Merged declarations keep their
        // own meanings, including a value merged with a type-only import.
        let mut result = if flags.contains(SymbolFlags::ALIAS) {
            let mut alias = if let Some(&resolved) = self.resolved.get(&symbol) {
                resolved
            } else if self.aliases.contains_key(&symbol) {
                return Err(symbol);
            } else {
                // An external import itself is known even though its members are not.
                ResolvedAlias {
                    target: TsAlias::Target(symbol),
                    ..ResolvedAlias::UNKNOWN
                }
            };
            if flags.contains(SymbolFlags::TYPE_ONLY) {
                alias.restrict_value();
            }
            alias
        } else {
            ResolvedAlias {
                target: TsAlias::Target(symbol),
                spaces: Spaces::empty(),
                ..ResolvedAlias::UNKNOWN
            }
        };
        if let TsAlias::Target(_) = result.target {
            result.target = TsAlias::Target(symbol);
        }
        result.spaces |= Spaces::declared(flags);
        // Direct declarations do not inherit restrictions from merged aliases.
        let owner = Some(QualifiedOwner {
            symbol,
            type_only: false,
        });
        if flags.in_namespace_space() {
            result.namespace_owner = owner;
        }
        if flags.intersects(SymbolFlags::CLASS | SymbolFlags::FUNCTION) {
            result.property_owner = owner;
        }
        Ok(result)
    }

    fn resolve_reference(&self, reference: &mut Reference, initializer: Option<SymbolId>) {
        let bare = initializer.is_some_and(|symbol| self.aliases[&symbol].members.is_empty());
        let binding = self.root(reference, bare).unwrap();
        reference.symbol = binding.map(|(symbol, _)| symbol);

        // Runtime access for an initializer depends on its final member. For
        // export expressions and specifiers it depends on the lexical binding.
        if !reference.flags.is_read() {
            return;
        }
        let reads_value = if let Some(symbol) = initializer {
            self.resolved[&symbol].spaces.contains(Spaces::VALUE)
        } else if reference.flags.space == ReferenceSpace::Any
            && let Some((symbol, flags)) = binding
        {
            flags.contains(SymbolFlags::EXPORT_VALUE)
                || self
                    .meaning(symbol, flags)
                    .unwrap()
                    .spaces
                    .contains(Spaces::VALUE)
        } else {
            true
        };
        if !reads_value {
            reference.flags = reference.flags.no_access();
        }
    }
}
