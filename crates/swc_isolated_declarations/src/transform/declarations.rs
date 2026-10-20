//! Declaration retention, variable emit, and generated default exports.

use std::borrow::Cow;

use swc_next_ecma_ast::{
    Ast, AstInternedUtf8, BindingIdentifier, BindingPattern, BindingPatternData, ClassType, Decl,
    DeclData, Diagnostic, ExportDefaultDeclarationKind, ExportDefaultDeclarationKindData, Expr,
    ExprData, GetSpan, Stmt, SymbolId, TsModuleDeclarationKind, TsModuleNameData,
    VariableDeclaration, VariableDeclarator, VariableKind, Visit, VisitWith,
};
use swc_next_ecma_semantic::SymbolFlags;

use super::diagnostics::{
    BINDING_ELEMENT_NOT_SUPPORTED, DEFAULT_EXPORT_TYPE_REQUIRED, VARIABLE_TYPE_REQUIRED,
};
use super::enums::transform_enum;
use super::modules::ModuleScopeKind;
use super::{DeclContext, Transformer};
use crate::SymbolUsage;
use crate::surface::{ExportExpression, export_expression, namespace_binding, unparenthesized};

pub(super) struct ExportExpressionBinding {
    declaration: Stmt,
    pub(super) name: AstInternedUtf8,
    order: ExportOrder,
}

enum ExportOrder {
    BeforeBinding,
    AfterBinding,
}

impl ExportExpressionBinding {
    pub(super) fn into_statements(self, export: Stmt) -> [Stmt; 2] {
        match self.order {
            ExportOrder::BeforeBinding => [export, self.declaration],
            ExportOrder::AfterBinding => [self.declaration, export],
        }
    }
}

impl<'ast> Transformer<'_, 'ast> {
    pub(super) fn transform_decl(
        &mut self,
        declaration: Decl,
        context: DeclContext,
        ast: &mut Ast<'ast>,
    ) -> Option<Decl> {
        match ast.decl_data(declaration) {
            DeclData::VariableDeclaration(declaration) => {
                self.transform_variable_declaration(declaration, context, ast)
            }
            DeclData::Function(function) => {
                let binding = function.id(ast);
                if !context.retain_all
                    && !binding.is_some_and(|id| self.binding_is_used_as(id, SymbolUsage::VALUE))
                {
                    return None;
                }
                let return_type = self.transform_function_signature(function, ast);
                let signature = ast.ts_function(
                    function.span(ast),
                    binding,
                    function.type_parameters(ast),
                    function.this_param(ast),
                    function.params(ast),
                    return_type,
                    false,
                    false,
                    context.declare,
                );
                Some(Decl::TsFunction(signature))
            }
            DeclData::Class(class) => {
                let binding = class.id(ast);
                if !context.retain_all && !binding.is_some_and(|id| self.binding_is_used(id)) {
                    return None;
                }
                self.transform_class(class, context, ast);
                Some(declaration)
            }
            DeclData::TsFunction(function) => {
                let binding = function.id(ast);
                if !context.retain_all
                    && !binding.is_some_and(|id| self.binding_is_used_as(id, SymbolUsage::VALUE))
                {
                    return None;
                }
                self.transform_ts_function_signature(function, ast);
                function.set_declare(ast, context.declare);
                Some(declaration)
            }
            DeclData::TsTypeAliasDeclaration(type_alias) => {
                if !context.retain_all
                    && !self.binding_is_used_as(type_alias.id(ast), SymbolUsage::TYPE)
                {
                    return None;
                }
                Some(declaration)
            }
            DeclData::TsInterfaceDeclaration(interface) => {
                if !context.retain_all
                    && !self.binding_is_used_as(interface.id(ast), SymbolUsage::TYPE)
                {
                    return None;
                }
                Some(declaration)
            }
            DeclData::TsEnumDeclaration(ts_enum) => {
                if !context.retain_all && !self.binding_is_used(ts_enum.id(ast)) {
                    return None;
                }
                let ambient = context.ambient || ts_enum.declare(ast);
                transform_enum(ts_enum, ambient, &mut self.diagnostics, self.semantic, ast);
                ts_enum.set_declare(ast, context.declare);
                Some(declaration)
            }
            DeclData::TsModuleDeclaration(module) => {
                let name = module.id(ast);
                let ambient_body = context.namespace(module, ast).ambient;
                if !context.retain_all
                    && namespace_binding(name, ast)
                        .is_some_and(|binding| !self.binding_is_used(binding))
                {
                    return None;
                }
                module.set_declare(ast, context.declare);
                if matches!(
                    ast.ts_module_name_data(name),
                    TsModuleNameData::BindingIdentifier(_)
                ) {
                    module.set_kind(ast, TsModuleDeclarationKind::Namespace);
                }
                self.transform_module_body(module.body(ast), ambient_body, ast);
                Some(declaration)
            }
            DeclData::TsGlobalDeclaration(global) => {
                global.set_declare(ast, context.declare);
                let block = global.body(ast);
                self.transform_module_block(
                    block,
                    DeclContext::AMBIENT_MODULE,
                    ModuleScopeKind::Global,
                    ast,
                );
                Some(declaration)
            }
            DeclData::TsImportEqualsDeclaration(import) => {
                if !context.retain_all && !self.binding_is_used(import.id(ast)) {
                    return None;
                }
                Some(declaration)
            }
        }
    }

    fn transform_variable_declaration(
        &mut self,
        declaration: VariableDeclaration,
        context: DeclContext,
        ast: &mut Ast<'ast>,
    ) -> Option<Decl> {
        let declarators = declaration.declarators(ast);
        if declarators.is_empty() {
            return None;
        }

        let kind = declaration.kind(ast);
        let mut transformed = Vec::new();
        for declarator in declarators.iter() {
            let declarator = ast.get_node_in_sub_range(declarator);
            let pattern = declarator.id(ast);
            let BindingPatternData::BindingIdentifier(binding) = ast.binding_pattern_data(pattern)
            else {
                self.report_unsupported_binding_pattern(pattern, context, ast);
                continue;
            };
            if !context.retain_all && !self.binding_is_used_as(binding, SymbolUsage::VALUE) {
                continue;
            }
            self.transform_variable_declarator(declarator, kind, ast);
            transformed.push(declarator);
        }
        if transformed.is_empty() && !context.retain_all {
            return None;
        }
        let declarators = ast.add_sub_range(transformed);
        declaration.set_declarators(ast, declarators);
        declaration.set_declare(ast, context.declare);
        if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing) {
            declaration.set_kind(ast, VariableKind::Const);
        }
        Some(Decl::VariableDeclaration(declaration))
    }

    fn report_unsupported_binding_pattern(
        &mut self,
        pattern: BindingPattern,
        context: DeclContext,
        ast: &Ast<'ast>,
    ) {
        let mut collector = BindingCollector::new(ast);
        pattern.visit_with(&mut collector);
        if !context.retain_all
            && !collector
                .bindings
                .iter()
                .any(|binding| self.binding_is_used_as(*binding, SymbolUsage::VALUE))
        {
            return;
        }
        self.diagnostics.extend(
            collector
                .bindings
                .into_iter()
                .filter(|binding| {
                    context.retain_all
                        || self
                            .semantic
                            .symbol_of(binding.node_id())
                            .is_some_and(|symbol| self.semantic.declarations(symbol).len() == 1)
                })
                .map(|binding| Diagnostic::error(BINDING_ELEMENT_NOT_SUPPORTED, binding.span(ast))),
        );
    }

    fn transform_variable_declarator(
        &mut self,
        declarator: VariableDeclarator,
        kind: VariableKind,
        ast: &mut Ast<'ast>,
    ) {
        declarator.set_definite(ast, false);
        if declarator.type_annotation(ast).is_some() {
            declarator.set_init(ast, None);
            return;
        }
        let Some(initializer) = declarator.init(ast) else {
            self.diagnostics.push(Diagnostic::error(
                VARIABLE_TYPE_REQUIRED,
                declarator.span(ast),
            ));
            return;
        };
        let initializer = unparenthesized(initializer, ast);
        if kind == VariableKind::Const && self.is_literal_initializer(initializer, ast) {
            let initializer = self.normalize_literal_initializer(initializer, ast);
            declarator.set_init(ast, Some(initializer));
            return;
        }
        let inferred = match ast.expr_data(initializer) {
            ExprData::TemplateLiteral(template)
                if kind == VariableKind::Const && !template.expressions(ast).is_empty() =>
            {
                None
            }
            _ => self.infer_type_from_expr(initializer, ast),
        };
        let inferred = inferred.unwrap_or_else(|| {
            self.diagnostics.push(Diagnostic::error(
                VARIABLE_TYPE_REQUIRED,
                initializer.span(ast),
            ));
            ast.ts_type_ts_unknown_keyword(initializer.span(ast))
        });
        let annotation = ast.ts_type_annotation(initializer.span(ast), inferred);
        declarator.set_type_annotation(ast, Some(annotation));
        declarator.set_init(ast, None);
    }

    pub(super) fn transform_default_declaration(
        &mut self,
        declaration: ExportDefaultDeclarationKind,
        context: DeclContext,
        ast: &mut Ast<'ast>,
    ) -> ExportDefaultDeclarationKind {
        match ast.export_default_declaration_kind_data(declaration) {
            ExportDefaultDeclarationKindData::Function(function) => {
                let return_type = self.transform_function_signature(function, ast);
                ExportDefaultDeclarationKind::TsFunction(ast.ts_function(
                    function.span(ast),
                    function.id(ast),
                    function.type_parameters(ast),
                    function.this_param(ast),
                    function.params(ast),
                    return_type,
                    false,
                    false,
                    false,
                ))
            }
            ExportDefaultDeclarationKindData::Class(class) => {
                self.transform_class(
                    class,
                    DeclContext {
                        declare: false,
                        ..context
                    },
                    ast,
                );
                declaration
            }
            ExportDefaultDeclarationKindData::TsFunction(function) => {
                self.transform_ts_function_signature(function, ast);
                function.set_declare(ast, false);
                declaration
            }
            ExportDefaultDeclarationKindData::TsInterfaceDeclaration(_) => {
                // The shared type-surface pass handles its body after retention.
                declaration
            }
            _ => declaration,
        }
    }

    pub(super) fn transform_export_expression_binding(
        &mut self,
        expression: Expr,
        context: DeclContext,
        ast: &mut Ast<'ast>,
    ) -> ExportExpressionBinding {
        let expression = unparenthesized(expression, ast);
        let span = expression.span(ast);
        let name = self.fresh_default_name(ast);
        let binding = ast.binding_identifier(span, name);

        // Only references retained by dependency analysis can escape the class's
        // original name scope. References in erased method bodies do not matter.
        let class = match export_expression(expression, ast) {
            ExportExpression::Class(class) => Some(class),
            _ => None,
        };
        if let Some(class) = class
            && class
                .id(ast)
                .and_then(|id| self.semantic.symbol_of(id.node_id()))
                .is_none_or(|symbol| !self.used.contains(symbol))
        {
            class.set_class_type(ast, ClassType::ClassDeclaration);
            class.set_id(ast, Some(binding));
            self.transform_class(class, context, ast);
            return ExportExpressionBinding {
                declaration: Stmt::Declaration(Decl::Class(class)),
                name,
                order: ExportOrder::BeforeBinding,
            };
        }

        let (type_annotation, initializer) = if self.is_literal_initializer(expression, ast) {
            (None, Some(expression))
        } else {
            let ty = self
                .infer_type_from_expr(expression, ast)
                .unwrap_or_else(|| {
                    self.diagnostics.push(Diagnostic::error(
                        DEFAULT_EXPORT_TYPE_REQUIRED,
                        expression.span(ast),
                    ));
                    ast.ts_type_ts_any_keyword(expression.span(ast))
                });
            (Some(ast.ts_type_annotation(span, ty)), None)
        };
        let declarator = ast.variable_declarator(
            span,
            BindingPattern::BindingIdentifier(binding),
            type_annotation,
            initializer,
            false,
        );
        let declarators = ast.add_sub_range([declarator]);
        let declaration =
            ast.variable_declaration(span, VariableKind::Const, declarators, context.declare);
        let statement = Stmt::Declaration(Decl::VariableDeclaration(declaration));
        ExportExpressionBinding {
            declaration: statement,
            name,
            order: ExportOrder::AfterBinding,
        }
    }

    fn fresh_default_name(&mut self, ast: &mut Ast<'ast>) -> AstInternedUtf8 {
        let names = self.reserved_names.get_or_insert_with(|| {
            let symbols = self
                .semantic
                .iter_symbols()
                .map(|entry| ast.get_wtf8(entry.symbol.name).as_bytes());
            let references = self
                .semantic
                .iter_references()
                .map(|entry| ast.get_utf8(entry.reference.name).as_bytes());
            symbols
                .chain(references)
                .filter(|name| name.starts_with(b"_default"))
                .collect()
        });
        loop {
            let suffix = self.next_default_id;
            self.next_default_id += 1;
            let candidate = if suffix == 0 {
                Cow::Borrowed("_default")
            } else {
                Cow::Owned(format!("_default_{suffix}"))
            };
            if !names.contains(candidate.as_bytes()) {
                // The increasing suffix already keeps generated names distinct.
                return ast.add_interned_utf8(&candidate);
            }
        }
    }

    pub(super) fn binding_is_used(&self, binding: BindingIdentifier) -> bool {
        self.semantic
            .symbol_of(binding.node_id())
            .is_some_and(|symbol| {
                self.binding_contributes_to_export(binding, symbol) && self.used.contains(symbol)
            })
    }

    pub(super) fn binding_is_used_as(
        &self,
        binding: BindingIdentifier,
        usage: SymbolUsage,
    ) -> bool {
        self.semantic
            .symbol_of(binding.node_id())
            .is_some_and(|symbol| {
                self.binding_contributes_to_export(binding, symbol)
                    && (usage.contains(SymbolUsage::TYPE) && self.used.contains_type(symbol)
                        || usage.contains(SymbolUsage::VALUE) && self.used.contains_value(symbol))
            })
    }

    fn binding_contributes_to_export(&self, binding: BindingIdentifier, symbol: SymbolId) -> bool {
        let mut current_flags = None;
        let mut exported_flags = Vec::new();
        for declaration in self.semantic.declarations(symbol) {
            let exported = declaration.flags.contains(SymbolFlags::EXPORTED);
            if exported {
                exported_flags.push(declaration.flags);
            }
            if declaration.node == binding.node_id() {
                current_flags = Some(declaration.flags);
            }
        }
        let Some(current_flags) = current_flags else {
            return true;
        };
        if current_flags.contains(SymbolFlags::EXPORTED) {
            return true;
        }
        let declaration_kinds = SymbolFlags::VARIABLE
            | SymbolFlags::FUNCTION
            | SymbolFlags::CLASS
            | SymbolFlags::REGULAR_ENUM
            | SymbolFlags::CONST_ENUM
            | SymbolFlags::INTERFACE
            | SymbolFlags::TYPE_ALIAS
            | SymbolFlags::NAMESPACE_MODULE;
        let current_kind = current_flags & declaration_kinds;
        !exported_flags
            .into_iter()
            .any(|flags| (flags & declaration_kinds).intersects(current_kind))
    }
}
struct BindingCollector<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    bindings: Vec<BindingIdentifier>,
}

impl<'visit, 'ast> BindingCollector<'visit, 'ast> {
    fn new(ast: &'visit Ast<'ast>) -> Self {
        Self {
            ast,
            bindings: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for BindingCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_binding_identifier(&mut self, binding: BindingIdentifier) {
        self.bindings.push(binding);
    }
}
