//! SWC's TypeScript erasure and runtime lowering before #12455.
use super::{
    ImportsNotUsedAsValues, TsImportExportAssignConfig, TypeScriptOptions, retain,
    semantic::SemanticInfo,
    ts_enum,
    ts_enum::{EnumValueComputer, EvalCtx, TsEnumRecordValue},
    utils,
};
use crate::common::{
    constructor::inject_after_super,
    resolver::context::{Id, Resolver},
};
use std::collections::{HashMap, HashSet};
use swc_next_allocator::wtf8::Wtf8Buf;
use swc_next_ecma_ast::*;

pub(super) struct Transform<'a, 'ast> {
    pub(super) ast: &'a mut Ast<'ast>,
    pub(super) resolver: &'a mut Resolver,
    pub(super) options: &'a TypeScriptOptions,
    pub(super) semantic: SemanticInfo,
    pub(super) diagnostics: Vec<Diagnostic>,
    pub(super) in_namespace: bool,
    pub(super) namespace_id: Option<Id>,
    pub(super) is_lhs: bool,
    pub(super) rewrite: bool,
    pub(super) decl_id_record: HashSet<Id>,
    pub(super) var_list: Vec<Id>,
    pub(super) export_var_list: Vec<Id>,
    comment_moves: [HashMap<u32, u32>; 2],
}

impl<'a, 'ast> Transform<'a, 'ast> {
    pub(super) fn new(
        ast: &'a mut Ast<'ast>,
        resolver: &'a mut Resolver,
        options: &'a TypeScriptOptions,
        semantic: SemanticInfo,
    ) -> Self {
        Self {
            ast,
            resolver,
            options,
            semantic,
            diagnostics: Vec::new(),
            in_namespace: false,
            namespace_id: None,
            is_lhs: false,
            rewrite: true,
            decl_id_record: HashSet::new(),
            var_list: Vec::new(),
            export_var_list: Vec::new(),
            comment_moves: Default::default(),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L135>.
    pub(super) fn visit_mut_program(&mut self) {
        let program = self.ast.root_program();
        let mut body: Vec<_> = program
            .body(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        if self.ast.is_module() {
            if !self.options.verbatim_module_syntax {
                body.retain(|&statement| self.strip_module_items_with_semantic(statement));
            }
            self.visit_mut_for_ts_import_export(&mut body);
        }
        let body = self.ast.add_sub_range(body);
        let body = self.visit_mut_stmts_impl(body, self.ast.is_module());
        let mut body: Vec<_> = body
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        if self.ast.is_module() && !self.export_var_list.is_empty() {
            let ids = std::mem::take(&mut self.export_var_list);
            let declaration = self.variables(ids);
            body.push(utils::export(self.ast, declaration));
        }
        let body = self.ast.add_sub_range(body);
        program.set_body(self.ast, body);
        if self.options.flow_syntax && self.ast.is_module() {
            self.normalize_module_await_bindings();
        }
        self.ast.retarget_comments(|position, offset| {
            let moves = &self.comment_moves[position as usize];
            let mut target = offset;
            while let Some(&next) = moves.get(&target) {
                target = next;
            }
            (target != offset).then_some(target)
        });
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L170>.
    pub(super) fn visit_mut_stmts_impl(
        &mut self,
        statements: SubRange<Stmt>,
        module: bool,
    ) -> SubRange<Stmt> {
        let saved = std::mem::take(&mut self.var_list);
        let mut output = Vec::new();
        for slot in statements.iter() {
            let statement = self.ast.get_node_in_sub_range(slot);
            let original_empty = statement.is_empty_statement(self.ast);
            if module && (original_empty || !self.retain_statement(statement)) {
                continue;
            }
            let statement = statement.visit_mut_with(self).unwrap_or(statement);
            if (!statement.is_empty_statement(self.ast) || original_empty)
                && (module
                    || !statement.is_empty_statement(self.ast)
                    || statement.span(self.ast) != DUMMY_SP)
            {
                output.push(statement);
            }
        }
        let variables = std::mem::replace(&mut self.var_list, saved);
        if !variables.is_empty() {
            output.push(Stmt::Declaration(self.variables(variables)));
        }
        self.ast.add_sub_range(output)
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L2104>.
    fn variables(&mut self, ids: Vec<Id>) -> Decl {
        let declarations: Vec<_> = ids
            .into_iter()
            .map(|id| {
                let binding = self.resolver.binding(self.ast, &id);
                self.ast.variable_declarator(
                    DUMMY_SP,
                    BindingPattern::BindingIdentifier(binding),
                    None,
                    None,
                    false,
                )
            })
            .collect();
        let declarations = self.ast.add_sub_range(declarations);
        Decl::VariableDeclaration(self.ast.variable_declaration(
            DUMMY_SP,
            VariableKind::Var,
            declarations,
            false,
        ))
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L348>.
    fn retain_statement(&self, statement: Stmt) -> bool {
        match self.ast.stmt_data(statement) {
            StmtData::Declaration(declaration) => retain::should_retain_decl(self.ast, declaration),
            StmtData::ExportNamedDeclaration(export) => {
                export.export_kind(self.ast) == ImportOrExportKind::Value
                    && export.declaration(self.ast).is_none_or(|declaration| {
                        self.in_namespace && declaration.is_variable_declaration(self.ast)
                            || retain::should_retain_decl(self.ast, declaration)
                    })
            }
            StmtData::EmptyStatement(_) => true,
            _ => retain::concrete(self.ast, statement),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L733>.
    fn strip_module_items_with_semantic(&mut self, statement: Stmt) -> bool {
        match self.ast.stmt_data(statement) {
            StmtData::ImportDeclaration(import)
                if import.import_kind(self.ast) == ImportOrExportKind::Value
                    && !import.specifiers(self.ast).is_empty() =>
            {
                let specifiers: Vec<_> = import
                    .specifiers(self.ast)
                    .iter()
                    .filter_map(|slot| {
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
                        let id = self.resolver.id(id.node_id(), id.name(self.ast), self.ast);
                        (!type_only
                            && !self.semantic.id_value.contains(&id)
                            && self.semantic.usage.contains(&id))
                        .then_some(specifier)
                    })
                    .collect();
                let retain = !specifiers.is_empty()
                    || self.options.import_not_used_as_values == ImportsNotUsedAsValues::Preserve;
                let specifiers = self.ast.add_sub_range(specifiers);
                import.set_specifiers(self.ast, specifiers);
                retain
            }
            StmtData::ExportNamedDeclaration(export) if export.declaration(self.ast).is_none() => {
                if export.export_kind(self.ast) == ImportOrExportKind::Type {
                    return false;
                }
                let specifiers: Vec<_> = export
                    .specifiers(self.ast)
                    .iter()
                    .filter_map(|slot| {
                        let specifier = self.ast.get_node_in_sub_range(slot);
                        if specifier.export_kind(self.ast) == ImportOrExportKind::Type {
                            return None;
                        }
                        if export.source(self.ast).is_none()
                            && let Some(id) = specifier.local(self.ast).as_identifier_name(self.ast)
                            && self.semantic.has_pure_type(&self.resolver.id(
                                id.node_id(),
                                id.name(self.ast),
                                self.ast,
                            ))
                        {
                            return None;
                        }
                        Some(specifier)
                    })
                    .collect();
                let retain = !specifiers.is_empty();
                let specifiers = self.ast.add_sub_range(specifiers);
                export.set_specifiers(self.ast, specifiers);
                retain
            }
            StmtData::ExportDefaultDeclaration(export) => !export
                .declaration(self.ast)
                .as_expr(self.ast)
                .and_then(|expression| expression.as_identifier_reference(self.ast))
                .is_some_and(|id| {
                    self.semantic.has_pure_type(&self.resolver.id(
                        id.node_id(),
                        id.name(self.ast),
                        self.ast,
                    ))
                }),
            StmtData::Declaration(declaration) => match self.ast.decl_data(declaration) {
                DeclData::TsImportEqualsDeclaration(import) => {
                    import.import_kind(self.ast) != ImportOrExportKind::Type
                        && self
                            .semantic
                            .usage
                            .contains(&self.binding_id(import.id(self.ast)))
                }
                DeclData::TsModuleDeclaration(module) => {
                    self.strip_namespace_module_items_with_semantic(module);
                    true
                }
                _ => true,
            },
            _ => true,
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L835>.
    fn strip_namespace_module_items_with_semantic(&mut self, module: TsModuleDeclaration) {
        match module
            .body(self.ast)
            .map(|body| self.ast.ts_module_body_data(body))
        {
            Some(TsModuleBodyData::TsModuleDeclaration(module)) => {
                self.strip_namespace_module_items_with_semantic(module)
            }
            Some(TsModuleBodyData::TsModuleBlock(block)) => {
                let statements: Vec<_> = block
                    .body(self.ast)
                    .iter()
                    .filter_map(|slot| {
                        let statement = self.ast.get_node_in_sub_range(slot);
                        if let Some(import) =
                            statement.as_declaration(self.ast).and_then(|declaration| {
                                declaration.as_ts_import_equals_declaration(self.ast)
                            })
                            && (import.import_kind(self.ast) == ImportOrExportKind::Type
                                || !self
                                    .semantic
                                    .namespace_import_equals_usage
                                    .contains(&import.node_id()))
                        {
                            return None;
                        }
                        Some(statement)
                    })
                    .collect();
                for &statement in &statements {
                    if let Some(module) = statement
                        .as_declaration(self.ast)
                        .and_then(|declaration| declaration.as_ts_module_declaration(self.ast))
                    {
                        self.strip_namespace_module_items_with_semantic(module);
                    }
                }
                let statements = self.ast.add_sub_range(statements);
                block.set_body(self.ast, statements);
            }
            None => {}
        }
    }

    pub(super) fn binding_id(&self, id: BindingIdentifier) -> Id {
        self.resolver.id(id.node_id(), id.name(self.ast), self.ast)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L872>.
    fn fold_decl(&mut self, declaration: Decl, exported: bool) -> Option<Stmt> {
        match self.ast.decl_data(declaration) {
            DeclData::TsModuleDeclaration(module) => {
                Some(self.transform_ts_module(module, exported))
            }
            DeclData::TsEnumDeclaration(enumeration) => {
                self.transform_ts_enum(enumeration, exported)
            }
            DeclData::TsImportEqualsDeclaration(import) if self.in_namespace => {
                self.namespace_import(import, exported)
            }
            DeclData::VariableDeclaration(variable) => {
                for slot in variable.declarators(self.ast).iter() {
                    let declarator = self.ast.get_node_in_sub_range(slot);
                    let old = std::mem::replace(&mut self.rewrite, exported);
                    if self.options.flow_syntax {
                        self.convert_flow_component_arrow(declarator);
                    }
                    declarator.set_type_annotation(self.ast, None);
                    declarator.set_definite(self.ast, false);
                    if let Some(pattern) = declarator.id(self.ast).visit_mut_with(self) {
                        declarator.set_id(self.ast, pattern);
                    }
                    self.rewrite = old;
                    if let Some(init) = declarator.init(self.ast).visit_mut_with(self) {
                        declarator.set_init(self.ast, init);
                    }
                }
                Some(Stmt::Declaration(declaration))
            }
            _ => {
                declaration.visit_mut_children_with(self);
                let id = match self.ast.decl_data(declaration) {
                    DeclData::Function(function) => function.id(self.ast),
                    DeclData::Class(class) => class.id(self.ast),
                    _ => None,
                };
                if let Some(id) = id {
                    self.decl_id_record.insert(self.binding_id(id));
                }
                Some(Stmt::Declaration(declaration))
            }
        }
    }

    pub(super) fn empty(&mut self) -> Stmt {
        Stmt::EmptyStatement(self.ast.empty_statement(DUMMY_SP))
    }

    /// SWC `ExportQuery`: ordinary references and write targets use the same query.
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1888>.
    pub(super) fn query_ref(&mut self, node: NodeId, name: AstUtf8) -> Option<Expr> {
        if !self.rewrite {
            return None;
        }
        let id = self.resolver.id(node, name, self.ast);
        let owner = self.semantic.exported_binding.get(&id)?.as_ref()?.clone();
        let object = utils::reference(self.ast, self.resolver, &owner);
        Some(utils::member(self.ast, object, &id.name))
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1645>.
    fn enter_expr_for_inline_enum(&mut self, expression: Expr) -> Option<Expr> {
        if self.is_lhs {
            return None;
        }
        let member = expression.as_member_expression(self.ast)?;
        if member.optional(self.ast) {
            return None;
        }
        let id = member.object(self.ast).as_identifier_reference(self.ast)?;
        let id = self.resolver.id(id.node_id(), id.name(self.ast), self.ast);
        if self.options.ts_enum_is_mutable && !self.semantic.const_enum.contains(&id) {
            return None;
        }
        let name = ts_enum::static_enum_member_name(self.ast, member)?;
        let value = self.semantic.enum_record.get(&(id, name))?;
        value.is_const().then(|| value.expression(self.ast))
    }
}

impl<'ast> VisitMut<'ast> for Transform<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L348>.
    fn visit_mut_stmt(&mut self, statement: Stmt) -> Option<Stmt> {
        if !self.retain_statement(statement) {
            return Some(self.empty());
        }
        match self.ast.stmt_data(statement) {
            StmtData::Declaration(declaration) => Some(
                self.fold_decl(declaration, false)
                    .unwrap_or_else(|| self.empty()),
            ),
            StmtData::ExportNamedDeclaration(export) if export.declaration(self.ast).is_some() => {
                let declaration = export.declaration(self.ast).unwrap();
                let lowered = self.fold_decl(declaration, true);
                Some(match lowered {
                    Some(statement) => {
                        if let Some(declaration) = statement.as_declaration(self.ast) {
                            export.set_declaration(self.ast, Some(declaration));
                            Stmt::ExportNamedDeclaration(export)
                        } else {
                            statement
                        }
                    }
                    None => self.empty(),
                })
            }
            _ => statement.visit_mut_children_with(self),
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L306>.
    fn visit_mut_stmts(&mut self, statements: SubRange<Stmt>) -> Option<SubRange<Stmt>> {
        Some(self.visit_mut_stmts_impl(statements, false))
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L448>.
    fn visit_mut_expr(&mut self, expression: Expr) -> Option<Expr> {
        let mut expression = expression;
        loop {
            let inner = match self.ast.expr_data(expression) {
                ExprData::TsAsExpression(wrapper) => wrapper.expression(self.ast),
                ExprData::TsSatisfiesExpression(wrapper) => wrapper.expression(self.ast),
                ExprData::TsTypeAssertion(wrapper) => wrapper.expression(self.ast),
                ExprData::TsNonNullExpression(wrapper) => wrapper.expression(self.ast),
                ExprData::TsInstantiationExpression(wrapper) => wrapper.expression(self.ast),
                _ => break,
            };
            let from = expression.span(self.ast);
            let to = inner.span(self.ast);
            if from.start != to.start {
                self.comment_moves[CommentPosition::Leading as usize].insert(from.start, to.start);
            }
            if from.end != to.end {
                self.comment_moves[CommentPosition::Trailing as usize].insert(from.end, to.end);
            }
            expression = inner;
        }
        expression = self
            .enter_expr_for_inline_enum(expression)
            .unwrap_or(expression);
        expression.visit_mut_children_with(self);
        if let Some(id) = expression.as_identifier_reference(self.ast)
            && let Some(replacement) = self.query_ref(id.node_id(), id.name(self.ast))
        {
            return Some(replacement);
        }
        Some(expression)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L503>.
    fn visit_mut_simple_assignment_target(
        &mut self,
        target: SimpleAssignmentTarget,
    ) -> Option<SimpleAssignmentTarget> {
        let inner = match self.ast.simple_assignment_target_data(target) {
            SimpleAssignmentTargetData::TsAsExpression(wrapper) => {
                Some(wrapper.expression(self.ast))
            }
            SimpleAssignmentTargetData::TsSatisfiesExpression(wrapper) => {
                Some(wrapper.expression(self.ast))
            }
            SimpleAssignmentTargetData::TsTypeAssertion(wrapper) => {
                Some(wrapper.expression(self.ast))
            }
            SimpleAssignmentTargetData::TsNonNullExpression(wrapper) => {
                Some(wrapper.expression(self.ast))
            }
            _ => None,
        };
        if let Some(inner) = inner {
            let inner = self.visit_mut_expr(inner).unwrap_or(inner);
            return Some(utils::target(self.ast, utils::unwrap(self.ast, inner)));
        }
        target.visit_mut_children_with(self);
        if let Some(id) = target.as_identifier_reference(self.ast)
            && let Some(replacement) = self.query_ref(id.node_id(), id.name(self.ast))
        {
            return Some(utils::target(self.ast, replacement));
        }
        None
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L468>.
    fn visit_mut_assignment_expression(
        &mut self,
        assignment: AssignmentExpression,
    ) -> Option<AssignmentExpression> {
        let old = std::mem::replace(&mut self.is_lhs, true);
        if let Some(left) = assignment.left(self.ast).visit_mut_with(self) {
            assignment.set_left(self.ast, left);
        }
        self.is_lhs = false;
        if let Some(right) = assignment.right(self.ast).visit_mut_with(self) {
            assignment.set_right(self.ast, right);
        }
        self.is_lhs = old;
        None
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L484>.
    fn visit_mut_update_expression(
        &mut self,
        update: UpdateExpression,
    ) -> Option<UpdateExpression> {
        let old = std::mem::replace(&mut self.is_lhs, true);
        update.visit_mut_children_with(self);
        self.is_lhs = old;
        None
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L497>.
    fn visit_mut_member_expression(
        &mut self,
        member: MemberExpression,
    ) -> Option<MemberExpression> {
        let old = std::mem::replace(&mut self.is_lhs, false);
        member.visit_mut_children_with(self);
        self.is_lhs = old;
        None
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L476>.
    fn visit_mut_assignment_pattern(
        &mut self,
        assignment: AssignmentPattern,
    ) -> Option<AssignmentPattern> {
        let old = std::mem::replace(&mut self.is_lhs, true);
        if let Some(left) = assignment.left(self.ast).visit_mut_with(self) {
            assignment.set_left(self.ast, left);
        }
        self.is_lhs = false;
        if let Some(right) = assignment.right(self.ast).visit_mut_with(self) {
            assignment.set_right(self.ast, right);
        }
        self.is_lhs = old;
        None
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L440>.
    fn visit_mut_binding_pattern(&mut self, pattern: BindingPattern) -> Option<BindingPattern> {
        if let Some(replacement) = pattern.visit_mut_children_with(self) {
            return Some(replacement);
        }
        if let Some(id) = pattern.as_binding_identifier(self.ast)
            && let Some(replacement) = self.query_ref(id.node_id(), id.name(self.ast))
        {
            return Some(BindingPattern::SimpleAssignmentTarget(utils::target(
                self.ast,
                replacement,
            )));
        }
        None
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L421>.
    fn visit_mut_object_property(&mut self, property: ObjectProperty) -> Option<ObjectProperty> {
        property.visit_mut_children_with(self);
        if property.shorthand(self.ast)
            && !property.value(self.ast).is_identifier_reference(self.ast)
        {
            property.set_shorthand(self.ast, false);
        }
        None
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L536>.
    fn visit_mut_binding_property(&mut self, property: BindingProperty) -> Option<BindingProperty> {
        property.visit_mut_children_with(self);
        utils::binding_shorthand(self.ast, property);
        None
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L633>.
    fn visit_mut_formal_parameter(
        &mut self,
        parameter: FormalParameter,
    ) -> Option<FormalParameter> {
        parameter.set_type_annotation(self.ast, None);
        parameter.set_optional(self.ast, false);
        parameter.visit_mut_children_with(self)
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L110>.
    fn visit_mut_formal_parameter_rest(
        &mut self,
        rest: FormalParameterRest,
    ) -> Option<FormalParameterRest> {
        rest.set_type_annotation(self.ast, None);
        rest.visit_mut_children_with(self)
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L110>.
    fn visit_mut_catch_clause(&mut self, catch: CatchClause) -> Option<CatchClause> {
        catch.set_type_annotation(self.ast, None);
        catch.visit_mut_children_with(self)
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L429>.
    fn visit_mut_variable_declarator(
        &mut self,
        declarator: VariableDeclarator,
    ) -> Option<VariableDeclarator> {
        if self.options.flow_syntax {
            self.convert_flow_component_arrow(declarator);
        }
        declarator.set_type_annotation(self.ast, None);
        declarator.set_definite(self.ast, false);
        let old = std::mem::replace(&mut self.rewrite, false);
        if let Some(pattern) = declarator.id(self.ast).visit_mut_with(self) {
            declarator.set_id(self.ast, pattern);
        }
        self.rewrite = old;
        if let Some(init) = declarator.init(self.ast).visit_mut_with(self) {
            declarator.set_init(self.ast, init);
        }
        None
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L558>.
    fn visit_mut_class(&mut self, class: Class) -> Option<Class> {
        class.set_type_parameters(self.ast, None);
        class.set_super_type_arguments(self.ast, None);
        class.set_implements(self.ast, SubRange::empty());
        class.set_abstract(self.ast, false);
        class.visit_mut_children_with(self);
        None
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L194>.
    fn visit_mut_class_body(&mut self, body: ClassBody) -> Option<ClassBody> {
        let mut elements: Vec<_> =
            body.body(self.ast)
                .iter()
                .map(|slot| self.ast.get_node_in_sub_range(slot))
                .filter(|element| match self.ast.class_element_data(*element) {
                    ClassElementData::TsIndexSignature(_)
                    | ClassElementData::TsMethodDefinition(_) => false,
                    ClassElementData::MethodDefinition(method) => !method.r#abstract(self.ast),
                    ClassElementData::PropertyDefinition(property) => {
                        !property.declare(self.ast) && !property.r#abstract(self.ast)
                    }
                    _ => true,
                })
                .collect();
        let mut properties = Vec::new();
        let mut initializers = Vec::new();
        let mut constructor = None;
        for &element in &elements {
            if let Some(method) = element.as_method_definition(self.ast)
                && method.kind(self.ast) == MethodDefinitionKind::Constructor
            {
                constructor = Some(method.value(self.ast));
                self.reorder_class_prop_decls_and_inits(
                    method.value(self.ast),
                    &mut properties,
                    &mut initializers,
                );
            }
            element.visit_mut_with(self);
        }
        if !properties.is_empty() {
            if !self.options.native_class_properties {
                self.reorder_class_prop_decls(&elements, &mut initializers);
            }
            if let Some(constructor) = constructor {
                inject_after_super(
                    self.ast,
                    self.resolver,
                    constructor.body(self.ast),
                    initializers,
                );
            }
            let mut fields = Vec::new();
            for id in properties {
                let name = self.ast.add_utf8(&id.name);
                let key = self.ast.identifier_name(DUMMY_SP, name);
                fields.push(self.ast.class_element_property_definition(
                    DUMMY_SP,
                    SubRange::empty(),
                    PropertyKey::IdentifierName(key),
                    None,
                    None,
                    false,
                    false,
                    false,
                    false,
                    false,
                    false,
                    false,
                    false,
                    false,
                    Accessibility::None,
                ));
            }
            fields.append(&mut elements);
            elements = fields;
        }
        let elements = self.ast.add_sub_range(elements);
        body.set_body(self.ast, elements);
        None
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L564>.
    fn visit_mut_method_definition(
        &mut self,
        method: MethodDefinition,
    ) -> Option<MethodDefinition> {
        if self.options.flow_syntax {
            self.flow_method_key(method);
        }
        method.set_override(self.ast, false);
        method.set_optional(self.ast, false);
        method.set_abstract(self.ast, false);
        method.set_accessibility(self.ast, Accessibility::None);
        method.visit_mut_children_with(self)
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L579>.
    fn visit_mut_property_definition(
        &mut self,
        property: PropertyDefinition,
    ) -> Option<PropertyDefinition> {
        if self.options.flow_syntax && property.accessor(self.ast) {
            self.flow_accessor_key(property);
        }
        property.set_type_annotation(self.ast, None);
        property.set_override(self.ast, false);
        property.set_optional(self.ast, false);
        property.set_definite(self.ast, false);
        property.set_readonly(self.ast, false);
        property.set_abstract(self.ast, false);
        property.set_accessibility(self.ast, Accessibility::None);
        property.visit_mut_children_with(self)
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L653>.
    fn visit_mut_function(&mut self, function: Function) -> Option<Function> {
        function.set_type_parameters(self.ast, None);
        function.set_this_param(self.ast, None);
        function.set_return_type(self.ast, None);
        function.visit_mut_children_with(self)
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L110>.
    fn visit_mut_arrow_function_expression(
        &mut self,
        arrow: ArrowFunctionExpression,
    ) -> Option<ArrowFunctionExpression> {
        arrow.set_type_parameters(self.ast, None);
        arrow.set_return_type(self.ast, None);
        arrow.visit_mut_children_with(self)
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L110>.
    fn visit_mut_call_expression(&mut self, call: CallExpression) -> Option<CallExpression> {
        call.set_type_arguments(self.ast, None);
        call.visit_mut_children_with(self)
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L110>.
    fn visit_mut_new_expression(&mut self, expression: NewExpression) -> Option<NewExpression> {
        expression.set_type_arguments(self.ast, None);
        expression.visit_mut_children_with(self)
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L110>.
    fn visit_mut_tagged_template_expression(
        &mut self,
        template: TaggedTemplateExpression,
    ) -> Option<TaggedTemplateExpression> {
        template.set_type_arguments(self.ast, None);
        template.visit_mut_children_with(self)
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L110>.
    fn visit_mut_jsx_opening_element(
        &mut self,
        element: JsxOpeningElement,
    ) -> Option<JsxOpeningElement> {
        element.set_type_arguments(self.ast, None);
        element.visit_mut_children_with(self)
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L606>.
    fn visit_mut_import_declaration(
        &mut self,
        import: ImportDeclaration,
    ) -> Option<ImportDeclaration> {
        let specifiers: Vec<_> = import
            .specifiers(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .filter(|specifier| {
                !specifier
                    .as_import_specifier(self.ast)
                    .is_some_and(|specifier| {
                        specifier.import_kind(self.ast) == ImportOrExportKind::Type
                    })
            })
            .collect();
        let specifiers = self.ast.add_sub_range(specifiers);
        import.set_specifiers(self.ast, specifiers);
        None
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L590>.
    fn visit_mut_export_specifiers(
        &mut self,
        specifiers: SubRange<ExportSpecifier>,
    ) -> Option<SubRange<ExportSpecifier>> {
        let specifiers: Vec<_> = specifiers
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .filter(|specifier| specifier.export_kind(self.ast) == ImportOrExportKind::Value)
            .collect();
        Some(self.ast.add_sub_range(specifiers))
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L388>.
    fn visit_mut_export_default_declaration(
        &mut self,
        export: ExportDefaultDeclaration,
    ) -> Option<ExportDefaultDeclaration> {
        export.visit_mut_children_with(self);
        let id = match self
            .ast
            .export_default_declaration_kind_data(export.declaration(self.ast))
        {
            ExportDefaultDeclarationKindData::Function(function) => function.id(self.ast),
            ExportDefaultDeclarationKindData::Class(class) => class.id(self.ast),
            _ => None,
        };
        if let Some(id) = id {
            self.decl_id_record.insert(self.binding_id(id));
        }
        None
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L520>.
    fn visit_mut_jsx_element_name(&mut self, name: JsxElementName) -> Option<JsxElementName> {
        self.jsx_name(name)
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L110>.
    fn visit_mut_ts_type(&mut self, _: TsType) -> Option<TsType> {
        None
    }
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L675>.
    fn visit_mut_ts_import_equals_declaration(
        &mut self,
        _: TsImportEqualsDeclaration,
    ) -> Option<TsImportEqualsDeclaration> {
        None
    }
}

impl Transform<'_, '_> {
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L236>.
    fn reorder_class_prop_decls_and_inits(
        &mut self,
        function: Function,
        properties: &mut Vec<crate::common::resolver::context::Id>,
        initializers: &mut Vec<Expr>,
    ) {
        let parameters = function.params(self.ast);
        for slot in parameters.items(self.ast).iter() {
            let item = self.ast.get_node_in_sub_range(slot);
            let Some(property) = item.as_ts_parameter_property(self.ast) else {
                continue;
            };
            let parameter = property.parameter(self.ast);
            parameter.set_decorators(self.ast, property.decorators(self.ast));
            let binding = parameter
                .pattern(self.ast)
                .as_binding_identifier(self.ast)
                .expect("parameter properties have identifier patterns");
            properties.push(self.binding_id(binding));
            let name = self.ast.get_utf8(binding.name(self.ast)).to_owned();
            let object = self.ast.expr_this_expression(DUMMY_SP);
            let left = utils::member(self.ast, object, &name);
            let right = self.resolver.reference(self.ast, binding);
            initializers.push(utils::assign(self.ast, left, right));
            parameters.items(self.ast).replace_slot(
                self.ast,
                slot,
                FormalParameterItem::FormalParameter(parameter),
            );
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1492>.
    fn reorder_class_prop_decls(
        &mut self,
        elements: &[ClassElement],
        initializers: &mut Vec<Expr>,
    ) {
        for &element in elements {
            let Some(property) = element.as_property_definition(self.ast) else {
                continue;
            };
            if property.r#static(self.ast) || property.accessor(self.ast) {
                continue;
            }
            let Some(value) = property.value(self.ast) else {
                continue;
            };
            let mut key = property.key(self.ast);
            if property.computed(self.ast)
                && let Some(expression) = key.as_expr(self.ast)
            {
                let literal = matches!(
                    self.ast.expr_data(expression),
                    ExprData::StringLiteral(_)
                        | ExprData::NumericLiteral(_)
                        | ExprData::BigIntLiteral(_)
                        | ExprData::BooleanLiteral(_)
                        | ExprData::NullLiteral(_)
                        | ExprData::RegExpLiteral(_)
                );
                if !literal {
                    let name = utils::alias_ident_for(self.ast, expression, "_key");
                    let binding = self.resolver.private_binding(self.ast, &name);
                    self.var_list.push(self.binding_id(binding));
                    let left = self.resolver.reference(self.ast, binding);
                    let assignment = utils::assign(self.ast, left, expression);
                    property.set_key(self.ast, PropertyKey::Expr(assignment));
                    key = PropertyKey::Expr(self.resolver.reference(self.ast, binding));
                }
            }
            let object = self.ast.expr_this_expression(DUMMY_SP);
            let member = self.ast.expr_member_expression(
                DUMMY_SP,
                object,
                key,
                property.computed(self.ast),
                false,
            );
            initializers.push(utils::assign(self.ast, member, value));
            property.set_value(self.ast, None);
        }
    }
}

impl Transform<'_, '_> {
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L2011>.
    fn convert_flow_component_arrow(&mut self, declaration: VariableDeclarator) {
        let Some(id) = declaration.id(self.ast).as_binding_identifier(self.ast) else {
            return;
        };
        let Some(annotation) = declaration.type_annotation(self.ast) else {
            return;
        };
        let mut ty = annotation.type_annotation(self.ast);
        while let Some(parenthesized) = ty.as_ts_parenthesized_type(self.ast) {
            ty = parenthesized.type_annotation(self.ast);
        }
        let Some(function_type) = ty.as_ts_function_type(self.ast) else {
            return;
        };
        let params = function_type.params(self.ast);
        if params.items(self.ast).len() != 1 || params.rest(self.ast).is_some() {
            return;
        }
        let parameter = self
            .ast
            .get_node_in_sub_range(params.items(self.ast).iter().next().unwrap());
        let Some(parameter) = parameter.as_formal_parameter(self.ast) else {
            return;
        };
        let Some(object) = parameter.pattern(self.ast).as_object_pattern(self.ast) else {
            return;
        };
        if function_type.span(self.ast) == DUMMY_SP
            || object.span(self.ast) != function_type.span(self.ast)
            || parameter.type_annotation(self.ast).is_some()
        {
            return;
        }
        let Some(arrow) = declaration
            .init(self.ast)
            .and_then(|init| init.as_arrow_function_expression(self.ast))
        else {
            return;
        };
        let body = match self.ast.arrow_function_body_data(arrow.body(self.ast)) {
            ArrowFunctionBodyData::FunctionBody(body) => body,
            ArrowFunctionBodyData::Expr(expression) => {
                let span = expression.span(self.ast);
                let statement = self.ast.stmt_return_statement(span, Some(expression));
                let statements = self.ast.add_sub_range([statement]);
                self.ast.function_body(span, SubRange::empty(), statements)
            }
        };
        let id = self.binding_id(id);
        let binding = self.resolver.binding(self.ast, &id);
        let params = arrow.params(self.ast);
        params.set_kind(self.ast, FormalParameterKind::FormalParameters);
        let function = self.ast.function(
            arrow.span(self.ast),
            FunctionType::FunctionExpression,
            Some(binding),
            arrow.type_parameters(self.ast),
            None,
            params,
            arrow.return_type(self.ast),
            body,
            false,
            arrow.r#async(self.ast),
        );
        declaration.set_init(self.ast, Some(Expr::Function(function)));
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L692>.
    fn flow_method_key(&mut self, method: MethodDefinition) {
        let function = method.value(self.ast);
        let special = function.r#async(self.ast)
            || function.generator(self.ast)
            || matches!(
                method.kind(self.ast),
                MethodDefinitionKind::Get | MethodDefinitionKind::Set
            );
        if method.r#static(self.ast)
            && special
            && !method.computed(self.ast)
            && is_constructor_key(self.ast, method.key(self.ast))
        {
            let key = computed_constructor(self.ast);
            method.set_key(self.ast, key);
            method.set_computed(self.ast, true);
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L692>.
    fn flow_accessor_key(&mut self, property: PropertyDefinition) {
        if property.r#static(self.ast)
            && !property.computed(self.ast)
            && is_constructor_key(self.ast, property.key(self.ast))
        {
            let key = computed_constructor(self.ast);
            property.set_key(self.ast, key);
            property.set_computed(self.ast, true);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L904>.
    fn normalize_module_await_bindings(&mut self) {
        let mut collector = AwaitBindingCollector {
            ast: self.ast,
            resolver: self.resolver,
            used: HashSet::new(),
            seen: HashSet::new(),
            bindings: Vec::new(),
        };
        collector.ast.root_program().visit_with(&mut collector);
        let mut used = collector.used;
        let mut renames = HashMap::new();
        let mut suffix = 0;
        for id in collector.bindings {
            let name = loop {
                let name = if suffix == 0 {
                    "_await".into()
                } else {
                    format!("_await{suffix}")
                };
                suffix += 1;
                if used.insert(name.clone()) {
                    break name;
                }
            };
            renames.insert(id, self.ast.add_utf8(&name));
        }
        if !renames.is_empty() {
            self.resolver.rename(self.ast, renames);
        }
    }
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L722>.
fn is_constructor_key(ast: &Ast<'_>, key: PropertyKey) -> bool {
    key.as_identifier_name(ast)
        .is_some_and(|id| ast.get_utf8(id.name(ast)) == "constructor")
        || key
            .as_expr(ast)
            .and_then(|expression| expression.as_string_literal(ast))
            .is_some_and(|string| ast.get_wtf8(string.value(ast)).as_str() == Some("constructor"))
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L692>.
fn computed_constructor(ast: &mut Ast<'_>) -> PropertyKey {
    let value = ast.add_utf8("constructor");
    PropertyKey::Expr(ast.expr_string_literal(DUMMY_SP, value.into(), AstUtf8::EMPTY))
}

struct AwaitBindingCollector<'a, 'ast> {
    ast: &'a Ast<'ast>,
    resolver: &'a Resolver,
    used: HashSet<String>,
    seen: HashSet<Id>,
    bindings: Vec<Id>,
}
impl<'ast> Visit<'ast> for AwaitBindingCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }
    fn visit_binding_identifier(&mut self, identifier: BindingIdentifier) {
        let id = self
            .resolver
            .id(identifier.node_id(), identifier.name(self.ast), self.ast);
        if id.name == "await" && self.seen.insert(id.clone()) {
            self.bindings.push(id.clone());
        }
        self.used.insert(id.name);
    }
    fn visit_identifier_reference(&mut self, identifier: IdentifierReference) {
        self.used
            .insert(self.ast.get_utf8(identifier.name(self.ast)).into());
    }
    fn visit_jsx_identifier(&mut self, identifier: JsxIdentifier) {
        self.used
            .insert(self.ast.get_utf8(identifier.name(self.ast)).into());
    }
    fn visit_module_export_name(&mut self, name: ModuleExportName) {
        if let Some(id) = name.as_identifier_name(self.ast) {
            self.used
                .insert(self.ast.get_utf8(id.name(self.ast)).into());
        }
    }
}

impl Transform<'_, '_> {
    /// SWC `InitArg::{get, or_empty, or_assign_empty}`.
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1014>.
    fn init_arg(&mut self, id: &Id, owner: Option<&Id>, assign: bool, or_empty: bool) -> Expr {
        let reference = utils::reference(self.ast, self.resolver, owner.unwrap_or(id));
        let reference = if owner.is_some() {
            utils::member(self.ast, reference, &id.name)
        } else {
            reference
        };
        if !assign && !or_empty {
            return reference;
        }
        let empty = self.ast.expr_object_expression(DUMMY_SP, SubRange::empty());
        let right = if assign {
            let left = utils::clone_expr(self.ast, self.resolver, reference);
            utils::assign(self.ast, left, empty)
        } else {
            empty
        };
        self.ast
            .expr_logical_expression(DUMMY_SP, reference, right, LogicalOperator::Or)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1265>.
    fn transform_ts_module(&mut self, module: TsModuleDeclaration, exported: bool) -> Stmt {
        let names = utils::namespace_names(self.ast, module.id(self.ast));
        let ids: Vec<_> = names
            .iter()
            .map(|&(node, name)| self.resolver.id(node, name, self.ast))
            .collect();
        let id = ids.first().unwrap();
        let owner = self.namespace_id.clone();
        let saved_namespace = self.namespace_id.replace(ids.last().unwrap().clone());
        let saved_in_namespace = std::mem::replace(&mut self.in_namespace, true);
        let mut statements = match self.ast.ts_module_body_data(module.body(self.ast).unwrap()) {
            TsModuleBodyData::TsModuleBlock(block) => {
                let body = self.visit_mut_stmts_impl(block.body(self.ast), true);
                self.transform_ts_module_block(body)
            }
            TsModuleBodyData::TsModuleDeclaration(module) => {
                vec![self.transform_ts_module(module, true)]
            }
        };
        self.namespace_id = saved_namespace;
        self.in_namespace = saved_in_namespace;
        for index in (0..ids.len()).rev() {
            let id = &ids[index];
            let owner = if index == 0 {
                owner.as_ref().filter(|_| exported)
            } else {
                Some(&ids[index - 1])
            };
            let argument = self.init_arg(id, owner, true, false);
            let binding = self.resolver.binding(self.ast, id);
            let function = utils::function(self.ast, binding, statements);
            let call = utils::call(self.ast, function, [argument], false);
            statements = vec![
                self.ast
                    .stmt_expression_statement(module.span(self.ast), call),
            ];
        }
        if self.decl_id_record.insert(id.clone()) {
            if exported {
                if owner.is_none() {
                    self.export_var_list.push(id.clone());
                }
            } else {
                self.var_list.push(id.clone());
            }
        }
        statements.pop().unwrap()
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1360>.
    fn transform_ts_module_block(&mut self, body: SubRange<Stmt>) -> Vec<Stmt> {
        let mut statements = Vec::new();
        for slot in body.iter() {
            let statement = self.ast.get_node_in_sub_range(slot);
            if let Some(export) = statement.as_export_named_declaration(self.ast) {
                let Some(declaration) = export.declaration(self.ast) else {
                    self.diagnostics.push(Diagnostic::error(
                        "ESM-style module declarations are not permitted in a namespace.",
                        export.span(self.ast),
                    ));
                    continue;
                };
                match self.ast.decl_data(declaration) {
                    DeclData::Function(function) => {
                        statements.push(Stmt::Declaration(declaration));
                        statements.push(self.assign_export(function.id(self.ast).unwrap()));
                    }
                    DeclData::Class(class) => {
                        statements.push(Stmt::Declaration(declaration));
                        statements.push(self.assign_export(class.id(self.ast).unwrap()));
                    }
                    DeclData::VariableDeclaration(variable) => {
                        let mut expressions = Vec::new();
                        for slot in variable.declarators(self.ast).iter() {
                            let declaration = self.ast.get_node_in_sub_range(slot);
                            let Some(init) = declaration.init(self.ast) else {
                                continue;
                            };
                            let target = self.pattern_target(declaration.id(self.ast));
                            expressions.push(self.ast.expr_assignment_expression(
                                declaration.span(self.ast),
                                target,
                                init,
                                AssignmentOperator::Assign,
                            ));
                        }
                        if !expressions.is_empty() {
                            let expression = if expressions.len() == 1 {
                                expressions[0]
                            } else {
                                let expressions = self.ast.add_sub_range(expressions);
                                self.ast.expr_sequence_expression(DUMMY_SP, expressions)
                            };
                            statements.push(
                                self.ast
                                    .stmt_expression_statement(variable.span(self.ast), expression),
                            );
                        }
                    }
                    _ => unreachable!("namespace runtime exports are lowered before assembly"),
                }
            } else if matches!(
                self.ast.stmt_data(statement),
                StmtData::ImportDeclaration(_)
                    | StmtData::ExportAllDeclaration(_)
                    | StmtData::ExportDefaultDeclaration(_)
                    | StmtData::TsExportAssignment(_)
                    | StmtData::TsNamespaceExportDeclaration(_)
            ) {
                self.diagnostics.push(Diagnostic::error(
                    "ESM-style module declarations are not permitted in a namespace.",
                    statement.span(self.ast),
                ));
            } else {
                statements.push(statement);
            }
        }
        statements
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1393>.
    fn pattern_target(&mut self, pattern: BindingPattern) -> AssignmentTarget {
        match self.ast.binding_pattern_data(pattern) {
            BindingPatternData::SimpleAssignmentTarget(target) => {
                AssignmentTarget::SimpleAssignmentTarget(target)
            }
            BindingPatternData::BindingIdentifier(id) => {
                let reference = self.resolver.reference(self.ast, id);
                AssignmentTarget::SimpleAssignmentTarget(utils::target(self.ast, reference))
            }
            BindingPatternData::ArrayPattern(array) => {
                AssignmentTarget::ArrayAssignmentTarget(array)
            }
            BindingPatternData::ObjectPattern(object) => {
                AssignmentTarget::ObjectAssignmentTarget(object)
            }
            _ => unreachable!("declaration initializers have an assignment pattern"),
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1613>.
    fn assign_export(&mut self, binding: BindingIdentifier) -> Stmt {
        let owner = self.namespace_id.as_ref().unwrap().clone();
        let object = utils::reference(self.ast, self.resolver, &owner);
        let left = utils::member(
            self.ast,
            object,
            self.ast
                .get_utf8(binding.name(self.ast))
                .to_owned()
                .as_str(),
        );
        let right = self.resolver.reference(self.ast, binding);
        let assignment = utils::assign(self.ast, left, right);
        self.ast.stmt_expression_statement(DUMMY_SP, assignment)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1440>.
    fn namespace_import(
        &mut self,
        import: TsImportEqualsDeclaration,
        exported: bool,
    ) -> Option<Stmt> {
        let reference = import.module_reference(self.ast);
        if reference.is_ts_external_module_reference(self.ast) {
            self.diagnostics.push(Diagnostic::error(
                "Import declarations in a namespace cannot reference a module.",
                import.span(self.ast),
            ));
            return None;
        }
        let init = self.ts_entity_name_to_expr(reference);
        if exported {
            let owner = self.namespace_id.as_ref().unwrap().clone();
            let object = utils::reference(self.ast, self.resolver, &owner);
            let left = utils::member(self.ast, object, &self.binding_id(import.id(self.ast)).name);
            let assignment = utils::assign(self.ast, left, init);
            Some(
                self.ast
                    .stmt_expression_statement(import.span(self.ast), assignment),
            )
        } else {
            Some(Stmt::Declaration(utils::variable(
                self.ast,
                VariableKind::Const,
                import.id(self.ast),
                Some(init),
                import.span(self.ast),
            )))
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1091>.
    fn transform_ts_enum(
        &mut self,
        enumeration: TsEnumDeclaration,
        exported: bool,
    ) -> Option<Stmt> {
        let id = self.binding_id(enumeration.id(self.ast));
        let first = self.decl_id_record.insert(id.clone());
        let remove = !self.options.verbatim_module_syntax
            && enumeration.is_const(self.ast)
            && !exported
            && !self.semantic.exported_binding.contains_key(&id);
        let names: HashSet<_> = self
            .semantic
            .enum_record
            .keys()
            .filter(|(owner, _)| owner == &id)
            .map(|(_, name)| name.clone())
            .collect();
        let mut members = Vec::new();
        for slot in enumeration.body(self.ast).members(self.ast).iter() {
            let member = self.ast.get_node_in_sub_range(slot);
            let name = super::shared::enum_member_name(self.ast, member.id(self.ast));
            let mut value = self
                .semantic
                .enum_record
                .get(&(id.clone(), name.clone()))
                .unwrap()
                .clone();
            if matches!(value, TsEnumRecordValue::Opaque(_))
                && let Some(init) = member.initializer(self.ast)
            {
                let init = init.visit_mut_with(self).unwrap_or(init);
                value = EnumValueComputer {
                    enum_id: &id,
                    resolver: self.resolver,
                    record: &self.semantic.enum_record,
                    ambient: &self.semantic.ambient_enum_record,
                    const_vars: &self.semantic.const_vars,
                    const_only: None,
                    ambient_const_only: None,
                }
                .compute(self.ast, init, EvalCtx::RECOMPUTE);
                if let TsEnumRecordValue::Opaque(expression) = &mut value {
                    let mut rewriter = EnumMemberRefQuery {
                        ast: self.ast,
                        resolver: self.resolver,
                        id: &id,
                        names: &names,
                    };
                    *expression = expression
                        .visit_mut_with(&mut rewriter)
                        .unwrap_or(*expression);
                }
            }
            if !remove || !value.is_const() {
                members.push((name, value, member.span(self.ast)));
            }
        }
        if members.is_empty() && enumeration.is_const(self.ast) {
            return None;
        }
        let opaque = members
            .iter()
            .any(|(_, value, _)| matches!(value, TsEnumRecordValue::Opaque(_)));
        let mut statements = Vec::new();
        for (name, value, span) in members {
            let string = matches!(value, TsEnumRecordValue::String(_));
            let name_value = self.ast.add_wtf8(&name);
            let name = self
                .ast
                .expr_string_literal(DUMMY_SP, name_value, AstUtf8::EMPTY);
            let object = utils::reference(self.ast, self.resolver, &id);
            let left = utils::computed(self.ast, object, name);
            let value = value.expression(self.ast);
            let inner = utils::assign(self.ast, left, value);
            let expression = if string {
                inner
            } else {
                let object = utils::reference(self.ast, self.resolver, &id);
                let left = utils::computed(self.ast, object, inner);
                let name = utils::clone_expr(self.ast, self.resolver, name);
                utils::assign(self.ast, left, name)
            };
            statements.push(self.ast.stmt_expression_statement(span, expression));
        }
        let namespace_export = self.namespace_id.is_some() && exported;
        let iife = !first || namespace_export;
        if !iife {
            let result = utils::reference(self.ast, self.resolver, &id);
            statements.push(self.ast.stmt_return_statement(DUMMY_SP, Some(result)));
        }
        let kind = if exported || id.scope == self.resolver.top_level() {
            VariableKind::Var
        } else {
            VariableKind::Let
        };
        let owner = self.namespace_id.clone().filter(|_| exported);
        let argument = if !first {
            self.init_arg(&id, owner.as_ref(), false, false)
        } else if namespace_export {
            self.init_arg(&id, owner.as_ref(), true, false)
        } else if exported || kind == VariableKind::Let {
            self.ast.expr_object_expression(DUMMY_SP, SubRange::empty())
        } else {
            self.init_arg(&id, None, false, true)
        };
        let binding = self.resolver.binding(self.ast, &id);
        let function = utils::function(self.ast, binding, statements);
        let call = utils::call(self.ast, function, [argument], !iife && !opaque);
        Some(if iife {
            self.ast
                .stmt_expression_statement(enumeration.span(self.ast), call)
        } else {
            Stmt::Declaration(utils::variable(
                self.ast,
                kind,
                enumeration.id(self.ast),
                Some(call),
                enumeration.span(self.ast),
            ))
        })
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1900>.
    fn jsx_name(&mut self, name: JsxElementName) -> Option<JsxElementName> {
        if !self.rewrite {
            return None;
        }
        match self.ast.jsx_element_name_data(name) {
            JsxElementNameData::JsxIdentifier(id) => {
                let binding = self.resolver.id(id.node_id(), id.name(self.ast), self.ast);
                let owner = self
                    .semantic
                    .exported_binding
                    .get(&binding)?
                    .as_ref()?
                    .clone();
                Some(jsx_member(self.ast, self.resolver, &owner, id))
            }
            JsxElementNameData::JsxMemberExpression(member) => {
                rewrite_jsx_object(
                    self.ast,
                    self.resolver,
                    &self.semantic.exported_binding,
                    member,
                );
                None
            }
            _ => None,
        }
    }
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1900>.
fn jsx_member(
    ast: &mut Ast<'_>,
    resolver: &mut crate::common::resolver::context::Resolver,
    owner: &Id,
    property: JsxIdentifier,
) -> JsxElementName {
    let name = ast.add_utf8(&owner.name);
    let object = ast.jsx_identifier(DUMMY_SP, name);
    resolver.register(ast);
    resolver.node_scopes[object.node_id()] = owner.scope;
    JsxElementName::JsxMemberExpression(ast.jsx_member_expression(
        DUMMY_SP,
        JsxMemberExpressionObject::JsxIdentifier(object),
        property,
    ))
}

/// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1900>.
fn rewrite_jsx_object(
    ast: &mut Ast<'_>,
    resolver: &mut crate::common::resolver::context::Resolver,
    exports: &std::collections::HashMap<Id, Option<Id>>,
    member: JsxMemberExpression,
) {
    match ast.jsx_member_expression_object_data(member.object(ast)) {
        JsxMemberExpressionObjectData::JsxIdentifier(id) => {
            let binding = resolver.id(id.node_id(), id.name(ast), ast);
            if let Some(Some(owner)) = exports.get(&binding) {
                let replacement = jsx_member(ast, resolver, owner, id)
                    .as_jsx_member_expression(ast)
                    .unwrap();
                member.set_object(
                    ast,
                    JsxMemberExpressionObject::JsxMemberExpression(replacement),
                );
            }
        }
        JsxMemberExpressionObjectData::JsxMemberExpression(member) => {
            rewrite_jsx_object(ast, resolver, exports, member)
        }
    }
}

/// SWC's `EnumMemberRefQuery` uses only unresolved identifiers declared by this
/// enum; lexical shadows retain their original resolver context.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1915>.
struct EnumMemberRefQuery<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut crate::common::resolver::context::Resolver,
    id: &'a Id,
    names: &'a HashSet<Wtf8Buf>,
}
impl EnumMemberRefQuery<'_, '_> {
    fn matches(&self, node: NodeId, name: AstUtf8) -> bool {
        self.resolver.node_scopes[node] == self.resolver.unresolved()
            && self
                .names
                .contains(&Wtf8Buf::from_str(self.ast.get_utf8(name)))
    }
    fn query(&mut self, node: NodeId, name: AstUtf8) -> Option<Expr> {
        if !self.matches(node, name) {
            return None;
        }
        let object = utils::reference(self.ast, self.resolver, self.id);
        let name = self.ast.get_utf8(name).to_owned();
        Some(utils::member(self.ast, object, &name))
    }
}
impl<'ast> VisitMut<'ast> for EnumMemberRefQuery<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_expr(&mut self, expression: Expr) -> Option<Expr> {
        expression.visit_mut_children_with(self);
        let id = expression.as_identifier_reference(self.ast)?;
        self.query(id.node_id(), id.name(self.ast))
    }
    fn visit_mut_simple_assignment_target(
        &mut self,
        target: SimpleAssignmentTarget,
    ) -> Option<SimpleAssignmentTarget> {
        target.visit_mut_children_with(self);
        let id = target.as_identifier_reference(self.ast)?;
        let expression = self.query(id.node_id(), id.name(self.ast))?;
        Some(utils::target(self.ast, expression))
    }
    fn visit_mut_object_property(&mut self, property: ObjectProperty) -> Option<ObjectProperty> {
        property.visit_mut_children_with(self);
        if property.shorthand(self.ast)
            && !property.value(self.ast).is_identifier_reference(self.ast)
        {
            property.set_shorthand(self.ast, false);
        }
        None
    }
    fn visit_mut_binding_property(&mut self, property: BindingProperty) -> Option<BindingProperty> {
        property.visit_mut_children_with(self);
        utils::binding_shorthand(self.ast, property);
        None
    }
    fn visit_mut_jsx_member_expression_object(
        &mut self,
        object: JsxMemberExpressionObject,
    ) -> Option<JsxMemberExpressionObject> {
        object.visit_mut_children_with(self);
        let id = object.as_jsx_identifier(self.ast)?;
        if !self.matches(id.node_id(), id.name(self.ast)) {
            return None;
        }
        let member = jsx_member(self.ast, self.resolver, self.id, id)
            .as_jsx_member_expression(self.ast)
            .unwrap();
        Some(JsxMemberExpressionObject::JsxMemberExpression(member))
    }
    fn visit_mut_jsx_element_name(&mut self, name: JsxElementName) -> Option<JsxElementName> {
        name.visit_mut_children_with(self);
        let id = name.as_jsx_identifier(self.ast)?;
        if !self.matches(id.node_id(), id.name(self.ast)) {
            return None;
        }
        Some(jsx_member(self.ast, self.resolver, self.id, id))
    }
}

impl Transform<'_, '_> {
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1625>.
    fn ts_entity_name_to_expr(&mut self, reference: TsModuleReference) -> Expr {
        fn qualified(ast: &mut Ast<'_>, name: TsQualifiedName) -> Expr {
            let object = match ast.ts_qualified_name_left_data(name.left(ast)) {
                TsQualifiedNameLeftData::IdentifierReference(id) => Expr::IdentifierReference(id),
                TsQualifiedNameLeftData::TsQualifiedName(name) => qualified(ast, name),
                _ => unreachable!("import aliases use identifier entity names"),
            };
            let key = name.right(ast);
            ast.expr_member_expression(
                name.span(ast),
                object,
                PropertyKey::IdentifierName(key),
                false,
                false,
            )
        }
        let name = reference.as_ts_type_name(self.ast).unwrap();
        match self.ast.ts_type_name_data(name) {
            TsTypeNameData::IdentifierReference(id) => Expr::IdentifierReference(id),
            TsTypeNameData::TsQualifiedName(name) => qualified(self.ast, name),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1678>.
    fn visit_mut_for_ts_import_export(&mut self, body: &mut Vec<Stmt>) {
        let create_require = self.resolver.private_binding(self.ast, "_createRequire");
        let require = self.resolver.private_binding(self.ast, "__require");
        let mut inject = false;
        let mut assignment = None;
        for statement in body.iter_mut() {
            let (import, exported) = match self.ast.stmt_data(*statement) {
                StmtData::Declaration(declaration) => {
                    (declaration.as_ts_import_equals_declaration(self.ast), false)
                }
                StmtData::ExportNamedDeclaration(export) => (
                    export.declaration(self.ast).and_then(|declaration| {
                        declaration.as_ts_import_equals_declaration(self.ast)
                    }),
                    true,
                ),
                StmtData::TsExportAssignment(export) => {
                    assignment.get_or_insert(export);
                    *statement = self.empty();
                    continue;
                }
                _ => (None, false),
            };
            let Some(import) = import else { continue };
            if import.import_kind(self.ast) == ImportOrExportKind::Type {
                continue;
            }
            let reference = import.module_reference(self.ast);
            if reference.is_ts_type_name(self.ast) {
                let init = self.ts_entity_name_to_expr(reference);
                let declaration = utils::variable(
                    self.ast,
                    VariableKind::Const,
                    import.id(self.ast),
                    Some(init),
                    if exported {
                        DUMMY_SP
                    } else {
                        import.span(self.ast)
                    },
                );
                *statement = if exported {
                    utils::export(self.ast, declaration)
                } else {
                    Stmt::Declaration(declaration)
                };
                continue;
            }
            let external = reference.as_ts_external_module_reference(self.ast).unwrap();
            let source = Expr::StringLiteral(external.expression(self.ast));
            match self.options.import_export_assign_config {
                TsImportExportAssignConfig::Classic => {
                    let callee = self.resolver.marked_reference(self.ast, "require", true);
                    let mut init = utils::call(self.ast, callee, [source], false);
                    if exported {
                        let exports = self.resolver.marked_reference(self.ast, "exports", true);
                        let left = utils::member(
                            self.ast,
                            exports,
                            &self.binding_id(import.id(self.ast)).name,
                        );
                        init = utils::assign(self.ast, left, init);
                    }
                    *statement = Stmt::Declaration(utils::variable(
                        self.ast,
                        VariableKind::Const,
                        import.id(self.ast),
                        Some(init),
                        import.span(self.ast),
                    ));
                }
                TsImportExportAssignConfig::Preserve => {}
                TsImportExportAssignConfig::NodeNext => {
                    inject = true;
                    let callee = self.resolver.reference(self.ast, require);
                    let init = utils::call(self.ast, callee, [source], false);
                    let declaration = utils::variable(
                        self.ast,
                        VariableKind::Const,
                        import.id(self.ast),
                        Some(init),
                        if exported {
                            DUMMY_SP
                        } else {
                            import.span(self.ast)
                        },
                    );
                    *statement = if exported {
                        utils::export(self.ast, declaration)
                    } else {
                        Stmt::Declaration(declaration)
                    };
                }
                TsImportExportAssignConfig::EsNext => {
                    let mut diagnostic = Diagnostic::error(
                        "Import assignment cannot be used when targeting ECMAScript modules.",
                        import.span(self.ast),
                    );
                    diagnostic.help = Some(
                        "Consider using `import * as ns from \"mod\"`, `import {a} from \"mod\"`, `import d from \"mod\"`, or another module format instead.".into(),
                    );
                    self.diagnostics.push(diagnostic);
                }
            }
        }
        if inject {
            let name = self.ast.add_utf8("createRequire");
            let imported =
                ModuleExportName::IdentifierName(self.ast.identifier_name(DUMMY_SP, name));
            let specifier = self.ast.import_specifier(
                DUMMY_SP,
                imported,
                create_require,
                ImportOrExportKind::Value,
            );
            let specifiers = self
                .ast
                .add_sub_range([ImportDeclarationSpecifier::ImportSpecifier(specifier)]);
            let source = self.ast.add_utf8("module");
            let source = self
                .ast
                .string_literal(DUMMY_SP, source.into(), AstUtf8::EMPTY);
            let import = Stmt::ImportDeclaration(self.ast.import_declaration(
                DUMMY_SP,
                specifiers,
                source,
                SubRange::empty(),
                None,
                ImportOrExportKind::Value,
            ));
            let import_name = self.ast.add_utf8("import");
            let meta_name = self.ast.add_utf8("meta");
            let import_name = self.ast.identifier_name(DUMMY_SP, import_name);
            let meta_name = self.ast.identifier_name(DUMMY_SP, meta_name);
            let meta = self
                .ast
                .expr_meta_property(DUMMY_SP, import_name, meta_name);
            let url = utils::member(self.ast, meta, "url");
            let callee = self.resolver.reference(self.ast, create_require);
            let init = utils::call(self.ast, callee, [url], false);
            let declaration = Stmt::Declaration(utils::variable(
                self.ast,
                VariableKind::Const,
                require,
                Some(init),
                DUMMY_SP,
            ));
            body.splice(0..0, [import, declaration]);
        }
        if let Some(export) = assignment {
            match self.options.import_export_assign_config {
                TsImportExportAssignConfig::Classic => {
                    let module = self.resolver.marked_reference(self.ast, "module", true);
                    let left = utils::member(self.ast, module, "exports");
                    let right = export.expression(self.ast);
                    let expression = utils::assign(self.ast, left, right);
                    let statement = self
                        .ast
                        .stmt_expression_statement(export.span(self.ast), expression);
                    if let Some(last) = body
                        .last_mut()
                        .filter(|statement| statement.is_empty_statement(self.ast))
                    {
                        *last = statement;
                    } else {
                        body.push(statement);
                    }
                }
                TsImportExportAssignConfig::Preserve => body.push(Stmt::TsExportAssignment(export)),
                TsImportExportAssignConfig::NodeNext | TsImportExportAssignConfig::EsNext => {
                    self.diagnostics.push(Diagnostic::error(
                        "Export assignment cannot be used when targeting ECMAScript modules.",
                        export.span(self.ast),
                    ))
                }
            }
        }
    }
}
