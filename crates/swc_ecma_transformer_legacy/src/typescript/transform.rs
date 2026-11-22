//! SWC's TypeScript erasure and runtime lowering before #12455.
use super::{
    ImportsNotUsedAsValues, TypeScriptOptions, retain, semantic::SemanticInfo, ts_enum, utils,
};
use crate::resolver::{Id, Resolver};
use std::collections::{HashMap, HashSet};
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
    pub(super) fn program(&mut self) {
        let program = self.ast.root_program();
        let mut body: Vec<_> = program
            .body(self.ast)
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        if self.ast.is_module() {
            if !self.options.verbatim_module_syntax {
                body.retain(|&statement| self.strip_module_item(statement));
            }
            self.import_export_assignments(&mut body);
        }
        let body = self.ast.add_sub_range(body);
        let body = self.statement_list(body, self.ast.is_module());
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
            self.normalize_flow_await();
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
    pub(super) fn statement_list(
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
            StmtData::Declaration(declaration) => retain::declaration(self.ast, declaration),
            StmtData::ExportNamedDeclaration(export) => {
                export.export_kind(self.ast) == ImportOrExportKind::Value
                    && export.declaration(self.ast).is_none_or(|declaration| {
                        self.in_namespace && declaration.is_variable_declaration(self.ast)
                            || retain::declaration(self.ast, declaration)
                    })
            }
            StmtData::EmptyStatement(_) => true,
            _ => retain::concrete(self.ast, statement),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L733>.
    fn strip_module_item(&mut self, statement: Stmt) -> bool {
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
                            && self.semantic.pure_type(&self.resolver.id(
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
                    self.semantic.pure_type(&self.resolver.id(
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
                    self.strip_namespace(module);
                    true
                }
                _ => true,
            },
            _ => true,
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L835>.
    fn strip_namespace(&mut self, module: TsModuleDeclaration) {
        match module
            .body(self.ast)
            .map(|body| self.ast.ts_module_body_data(body))
        {
            Some(TsModuleBodyData::TsModuleDeclaration(module)) => self.strip_namespace(module),
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
                        self.strip_namespace(module);
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
    fn declaration(&mut self, declaration: Decl, exported: bool) -> Option<Stmt> {
        match self.ast.decl_data(declaration) {
            DeclData::TsModuleDeclaration(module) => Some(self.namespace(module, exported)),
            DeclData::TsEnumDeclaration(enumeration) => self.enumeration(enumeration, exported),
            DeclData::TsImportEqualsDeclaration(import) if self.in_namespace => {
                self.namespace_import(import, exported)
            }
            DeclData::VariableDeclaration(variable) => {
                for slot in variable.declarators(self.ast).iter() {
                    let declarator = self.ast.get_node_in_sub_range(slot);
                    let old = std::mem::replace(&mut self.rewrite, exported);
                    if self.options.flow_syntax {
                        self.flow_component_arrow(declarator);
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
    pub(super) fn rewrite_reference(&mut self, node: NodeId, name: AstUtf8) -> Option<Expr> {
        if !self.rewrite {
            return None;
        }
        let id = self.resolver.id(node, name, self.ast);
        let owner = self.semantic.exported_binding.get(&id)?.as_ref()?.clone();
        let object = utils::reference(self.ast, self.resolver, &owner);
        Some(utils::member(self.ast, object, &id.name))
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L1645>.
    fn inline_enum(&mut self, expression: Expr) -> Option<Expr> {
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
        let name = ts_enum::static_member(self.ast, member)?;
        let value = self.semantic.enum_record.get(&(id, name))?;
        value.constant().then(|| value.expression(self.ast))
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
                self.declaration(declaration, false)
                    .unwrap_or_else(|| self.empty()),
            ),
            StmtData::ExportNamedDeclaration(export) if export.declaration(self.ast).is_some() => {
                let declaration = export.declaration(self.ast).unwrap();
                let lowered = self.declaration(declaration, true);
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
        Some(self.statement_list(statements, false))
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
        expression = self.inline_enum(expression).unwrap_or(expression);
        expression.visit_mut_children_with(self);
        if let Some(id) = expression.as_identifier_reference(self.ast)
            && let Some(replacement) = self.rewrite_reference(id.node_id(), id.name(self.ast))
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
            && let Some(replacement) = self.rewrite_reference(id.node_id(), id.name(self.ast))
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
        pattern.visit_mut_children_with(self);
        if let Some(id) = pattern.as_binding_identifier(self.ast)
            && let Some(replacement) = self.rewrite_reference(id.node_id(), id.name(self.ast))
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
            self.flow_component_arrow(declarator);
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
        self.class_body(body);
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
