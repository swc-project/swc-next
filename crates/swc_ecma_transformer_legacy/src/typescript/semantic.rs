//! SWC's usage, export, enum and namespace-alias collection before #12455.
use super::{
    TypeScriptOptions, retain,
    ts_enum::{EnumValueComputer, EvalCtx, TsEnumRecord, TsEnumRecordValue},
    utils,
};
use crate::common::resolver::context::{Id, Resolver};
use std::collections::{HashMap, HashSet};
use swc_next_ecma_ast::*;

#[derive(Default)]
pub(super) struct SemanticInfo {
    pub(super) usage: HashSet<Id>,
    pub(super) id_type: HashSet<Id>,
    pub(super) id_value: HashSet<Id>,
    pub(super) exported_binding: HashMap<Id, Option<Id>>,
    pub(super) enum_record: TsEnumRecord,
    pub(super) ambient_enum_record: TsEnumRecord,
    pub(super) const_enum: HashSet<Id>,
    pub(super) namespace_import_equals_usage: HashSet<NodeId>,
    pub(super) const_vars: HashMap<Id, TsEnumRecordValue>,
}

impl SemanticInfo {
    pub(super) fn has_pure_type(&self, id: &Id) -> bool {
        self.id_type.contains(id) && !self.id_value.contains(id)
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L48>.
pub(super) fn analyze_program(
    ast: &mut Ast<'_>,
    resolver: &mut Resolver,
    options: &TypeScriptOptions,
    seed: HashSet<Id>,
) -> SemanticInfo {
    let mut analyzer = SemanticAnalyzer {
        ast,
        resolver,
        options,
        info: SemanticInfo {
            usage: seed,
            ..Default::default()
        },
        import_chain: HashMap::new(),
        namespace_blocks: Vec::new(),
        namespace_id: None,
        skip_transform_info: false,
        exported: false,
    };
    analyzer.collect_module();
    analyzer.ast.root_program().visit_with(&mut analyzer);
    analyze_import_chain(&mut analyzer.info.usage, &mut analyzer.import_chain);
    analyzer.resolver.register(analyzer.ast);
    analyzer.info
}

#[derive(Default)]
struct NamespaceBlock {
    usage: HashSet<Id>,
    import_chain: HashMap<Id, Id>,
    imports: Vec<(Id, NodeId, bool, bool)>,
}

/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L100>.
fn analyze_import_chain(usage: &mut HashSet<Id>, chain: &mut HashMap<Id, Id>) {
    let mut new_usage = HashSet::new();
    for id in usage.iter() {
        let mut next = chain.remove(id);
        while let Some(id) = next {
            next = chain.remove(&id);
            new_usage.insert(id);
        }
        if chain.is_empty() {
            break;
        }
    }
    usage.extend(new_usage);
}

struct SemanticAnalyzer<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    options: &'a TypeScriptOptions,
    info: SemanticInfo,
    import_chain: HashMap<Id, Id>,
    namespace_blocks: Vec<NamespaceBlock>,
    namespace_id: Option<Id>,
    skip_transform_info: bool,
    exported: bool,
}

impl SemanticAnalyzer<'_, '_> {
    fn binding(&self, id: BindingIdentifier) -> Id {
        self.resolver.id(id.node_id(), id.name(self.ast), self.ast)
    }
    fn usage(&mut self, id: Id) {
        self.info.usage.insert(id.clone());
        if let Some(block) = self.namespace_blocks.last_mut() {
            block.usage.insert(id);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L151>.
    fn collect_module(&mut self) {
        if !self.ast.is_module() {
            return;
        }
        for slot in self.ast.root_program().body(self.ast).iter() {
            let statement = self.ast.get_node_in_sub_range(slot);
            match self.ast.stmt_data(statement) {
                StmtData::Declaration(declaration) => self.collect_decl(declaration),
                StmtData::ExportNamedDeclaration(export) => {
                    if let Some(declaration) = export.declaration(self.ast) {
                        self.collect_decl(declaration);
                    }
                }
                StmtData::ImportDeclaration(import) => {
                    for slot in import.specifiers(self.ast).iter() {
                        let specifier = self.ast.get_node_in_sub_range(slot);
                        let (id, type_only) =
                            match self.ast.import_declaration_specifier_data(specifier) {
                                ImportDeclarationSpecifierData::ImportSpecifier(specifier) => (
                                    specifier.local(self.ast),
                                    specifier.import_kind(self.ast) == ImportOrExportKind::Type,
                                ),
                                ImportDeclarationSpecifierData::ImportDefaultSpecifier(
                                    specifier,
                                ) => (specifier.local(self.ast), false),
                                ImportDeclarationSpecifierData::ImportNamespaceSpecifier(
                                    specifier,
                                ) => (specifier.local(self.ast), false),
                            };
                        if type_only || import.import_kind(self.ast) == ImportOrExportKind::Type {
                            self.info.id_type.insert(self.binding(id));
                        }
                    }
                }
                StmtData::ExportDefaultDeclaration(export) => {
                    let id = match self
                        .ast
                        .export_default_declaration_kind_data(export.declaration(self.ast))
                    {
                        ExportDefaultDeclarationKindData::Function(function) => {
                            function.id(self.ast)
                        }
                        ExportDefaultDeclarationKindData::Class(class) => class.id(self.ast),
                        _ => None,
                    };
                    if let Some(id) = id {
                        self.info.id_value.insert(self.binding(id));
                    }
                }
                _ => {}
            }
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L215>.
    fn collect_decl(&mut self, declaration: Decl) {
        match self.ast.decl_data(declaration) {
            DeclData::VariableDeclaration(declaration) => {
                for slot in declaration.declarators(self.ast).iter() {
                    let declaration = self.ast.get_node_in_sub_range(slot);
                    for id in utils::find_pat_ids(self.ast, declaration.id(self.ast)) {
                        self.info.id_value.insert(self.binding(id));
                    }
                }
            }
            DeclData::Function(function) => {
                if let Some(id) = function.id(self.ast) {
                    self.info.id_value.insert(self.binding(id));
                }
            }
            DeclData::TsFunction(function) => {
                if let Some(id) = function.id(self.ast) {
                    self.info.id_value.insert(self.binding(id));
                }
            }
            DeclData::Class(class) => {
                if let Some(id) = class.id(self.ast) {
                    self.info.id_value.insert(self.binding(id));
                }
            }
            DeclData::TsEnumDeclaration(enumeration) => {
                self.info
                    .id_value
                    .insert(self.binding(enumeration.id(self.ast)));
            }
            DeclData::TsModuleDeclaration(module) => {
                if let Some(&(node, name)) =
                    utils::namespace_names(self.ast, module.id(self.ast)).first()
                {
                    let id = self.resolver.id(node, name, self.ast);
                    if retain::namespace_body_is_concrete(self.ast, module) {
                        self.info.id_value.insert(id);
                    } else {
                        self.info.id_type.insert(id);
                    }
                }
            }
            DeclData::TsInterfaceDeclaration(interface) => {
                self.info
                    .id_type
                    .insert(self.binding(interface.id(self.ast)));
            }
            DeclData::TsTypeAliasDeclaration(alias) => {
                self.info.id_type.insert(self.binding(alias.id(self.ast)));
            }
            DeclData::TsImportEqualsDeclaration(import) => {
                let id = self.binding(import.id(self.ast));
                if import.import_kind(self.ast) == ImportOrExportKind::Type {
                    self.info.id_type.insert(id);
                } else {
                    self.info.id_value.insert(id);
                }
            }
            DeclData::TsGlobalDeclaration(_) => {}
        }
    }

    fn compute(
        &mut self,
        expression: Expr,
        id: &Id,
        context: EvalCtx,
        mutable: bool,
        ambient_mutable: bool,
    ) -> TsEnumRecordValue {
        EnumValueComputer {
            enum_id: id,
            resolver: self.resolver,
            record: &self.info.enum_record,
            ambient: &self.info.ambient_enum_record,
            const_vars: &self.info.const_vars,
            const_only: mutable.then_some(&self.info.const_enum),
            ambient_const_only: ambient_mutable.then_some(&self.info.const_enum),
        }
        .compute(self.ast, expression, context)
    }
}

impl<'ast> Visit<'ast> for SemanticAnalyzer<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }
    fn visit_ts_type(&mut self, _: TsType) {}
    fn visit_binding_identifier(&mut self, _: BindingIdentifier) {}
    fn visit_import_declaration(&mut self, _: ImportDeclaration) {}
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L365>.
    fn visit_decl(&mut self, declaration: Decl) {
        let skip = self.skip_transform_info;
        self.skip_transform_info |= !retain::should_retain_decl(self.ast, declaration);
        declaration.visit_children_with(self);
        self.skip_transform_info = skip;
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L376>.
    fn visit_identifier_reference(&mut self, id: IdentifierReference) {
        self.usage(self.resolver.id(id.node_id(), id.name(self.ast), self.ast));
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L376>.
    fn visit_jsx_identifier(&mut self, id: JsxIdentifier) {
        self.usage(self.resolver.id(id.node_id(), id.name(self.ast), self.ast));
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L691>.
    fn visit_jsx_element_name(&mut self, name: JsxElementName) {
        if name.as_jsx_identifier(self.ast).is_some_and(|id| {
            self.ast
                .get_utf8(id.name(self.ast))
                .starts_with(|c: char| c.is_ascii_lowercase())
        }) {
            return;
        }
        name.visit_children_with(self);
    }
    fn visit_jsx_member_expression(&mut self, member: JsxMemberExpression) {
        member.object(self.ast).visit_with(self);
    }
    fn visit_jsx_attribute(&mut self, attribute: JsxAttribute) {
        attribute.value(self.ast).visit_with(self);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L417>.
    fn visit_ts_import_equals_declaration(&mut self, import: TsImportEqualsDeclaration) {
        let id = self.binding(import.id(self.ast));
        let type_only = import.import_kind(self.ast) == ImportOrExportKind::Type;
        if let Some(block) = self.namespace_blocks.last_mut() {
            block
                .imports
                .push((id.clone(), import.node_id(), self.exported, type_only));
        }
        if !self.skip_transform_info && self.exported {
            self.info
                .exported_binding
                .insert(id.clone(), self.namespace_id.clone());
        }
        if type_only {
            return;
        }
        let Some(root) =
            super::shared::get_module_ident(self.ast, import.module_reference(self.ast))
        else {
            return;
        };
        let root = self
            .resolver
            .id(root.node_id(), root.name(self.ast), self.ast);
        if let Some(block) = self.namespace_blocks.last_mut() {
            if self.exported {
                block.usage.insert(root.clone());
                block.usage.insert(id.clone());
            } else {
                block.import_chain.insert(id.clone(), root.clone());
            }
        }
        if self.exported {
            self.usage(root);
            self.usage(id);
        } else {
            self.import_chain.insert(id, root);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L463>.
    fn visit_export_named_declaration(&mut self, export: ExportNamedDeclaration) {
        if let Some(declaration) = export.declaration(self.ast) {
            let exported = std::mem::replace(&mut self.exported, true);
            declaration.visit_with(self);
            self.exported = exported;
            if self.skip_transform_info {
                return;
            }
            let ids = match self.ast.decl_data(declaration) {
                DeclData::VariableDeclaration(var) => var
                    .declarators(self.ast)
                    .iter()
                    .flat_map(|slot| {
                        let decl = self.ast.get_node_in_sub_range(slot);
                        utils::find_pat_ids(self.ast, decl.id(self.ast))
                    })
                    .collect(),
                DeclData::TsEnumDeclaration(enumeration) => vec![enumeration.id(self.ast)],
                DeclData::TsModuleDeclaration(module) => {
                    if let Some(id) = module.id(self.ast).as_binding_identifier(self.ast) {
                        vec![id]
                    } else {
                        Vec::new()
                    }
                }
                _ => Vec::new(),
            };
            for id in ids {
                self.info
                    .exported_binding
                    .insert(self.binding(id), self.namespace_id.clone());
            }
        } else if export.export_kind(self.ast) == ImportOrExportKind::Value
            && export.source(self.ast).is_none()
        {
            for slot in export.specifiers(self.ast).iter() {
                let specifier = self.ast.get_node_in_sub_range(slot);
                if specifier.export_kind(self.ast) == ImportOrExportKind::Value
                    && let Some(id) = specifier.local(self.ast).as_identifier_name(self.ast)
                {
                    let id = self.resolver.id(id.node_id(), id.name(self.ast), self.ast);
                    if !self.skip_transform_info {
                        self.info
                            .exported_binding
                            .insert(id.clone(), self.namespace_id.clone());
                    }
                    self.usage(id);
                }
            }
        }
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L521>.
    fn visit_export_default_declaration(&mut self, export: ExportDefaultDeclaration) {
        export.visit_children_with(self);
        if !self.skip_transform_info
            && let Some(expression) = export.declaration(self.ast).as_expr(self.ast)
            && let Some(id) = expression.as_identifier_reference(self.ast)
        {
            self.info.exported_binding.insert(
                self.resolver.id(id.node_id(), id.name(self.ast), self.ast),
                self.namespace_id.clone(),
            );
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L548>.
    fn visit_ts_module_declaration(&mut self, module: TsModuleDeclaration) {
        let old = self.namespace_id.clone();
        if !self.skip_transform_info
            && let Some(&(node, name)) =
                utils::namespace_names(self.ast, module.id(self.ast)).last()
        {
            self.namespace_id = Some(self.resolver.id(node, name, self.ast));
        }
        let exported = std::mem::take(&mut self.exported);
        module.body(self.ast).visit_with(self);
        self.exported = exported;
        self.namespace_id = old;
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L573>.
    fn visit_ts_module_block(&mut self, block: TsModuleBlock) {
        self.namespace_blocks.push(NamespaceBlock::default());
        block.visit_children_with(self);
        let mut block = self.namespace_blocks.pop().unwrap();
        analyze_import_chain(&mut block.usage, &mut block.import_chain);
        for (id, node, exported, type_only) in block.imports {
            if !type_only && (exported || block.usage.contains(&id)) {
                self.info.namespace_import_equals_usage.insert(node);
            }
        }
        if let Some(parent) = self.namespace_blocks.last_mut() {
            parent.usage.extend(block.usage);
            parent.import_chain.extend(block.import_chain);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L605>.
    fn visit_variable_declaration(&mut self, declaration: VariableDeclaration) {
        for slot in declaration.declarators(self.ast).iter() {
            let declarator = self.ast.get_node_in_sub_range(slot);
            declarator.visit_with(self);
            if declaration.kind(self.ast) != VariableKind::Const
                || declarator.type_annotation(self.ast).is_some()
            {
                continue;
            }
            let Some(id) = declarator.id(self.ast).as_binding_identifier(self.ast) else {
                continue;
            };
            let Some(init) = declarator.init(self.ast) else {
                continue;
            };
            let id = self.binding(id);
            let value = self.compute(
                init,
                &id,
                EvalCtx::CONST_INIT,
                self.options.ts_enum_is_mutable,
                self.options.ts_enum_is_mutable,
            );
            if value.is_const() {
                self.info.const_vars.insert(id, value);
            }
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/semantic.rs#L641>.
    fn visit_ts_enum_declaration(&mut self, enumeration: TsEnumDeclaration) {
        enumeration.body(self.ast).visit_with(self);
        let id = self.binding(enumeration.id(self.ast));
        if enumeration.is_const(self.ast) {
            self.info.const_enum.insert(id.clone());
        }
        let ambient = self.skip_transform_info;
        let mut default = if self.options.flow_syntax && !ambient {
            TsEnumRecordValue::Void
        } else {
            TsEnumRecordValue::number(0.0)
        };
        for slot in enumeration.body(self.ast).members(self.ast).iter() {
            let member = self.ast.get_node_in_sub_range(slot);
            let name = super::shared::enum_member_name(self.ast, member.id(self.ast));
            let value = if let Some(init) = member.initializer(self.ast) {
                self.compute(
                    init,
                    &id,
                    if ambient {
                        EvalCtx::CONST_INIT
                    } else {
                        EvalCtx::MEMBER
                    },
                    ambient && self.options.ts_enum_is_mutable,
                    self.options.ts_enum_is_mutable,
                )
            } else if ambient && !enumeration.is_const(self.ast) {
                continue;
            } else if self.options.flow_syntax
                && matches!(default, TsEnumRecordValue::Void)
                && !ambient
            {
                TsEnumRecordValue::String(name.clone())
            } else {
                default.clone()
            };
            default = value.inc();
            if ambient {
                if value.is_const() {
                    self.info
                        .ambient_enum_record
                        .insert((id.clone(), name), value);
                }
            } else {
                self.info.enum_record.insert((id.clone(), name), value);
            }
        }
    }
}
