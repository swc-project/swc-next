//! Runtime namespace and enum emission, following SWC before #12455.
use super::{
    transform::Transform,
    ts_enum::{Computer, EvalCtx, Value},
    utils,
};
use crate::resolver::Id;
use std::collections::HashSet;
use swc_next_allocator::wtf8::Wtf8Buf;
use swc_next_ecma_ast::*;

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
    pub(super) fn namespace(&mut self, module: TsModuleDeclaration, exported: bool) -> Stmt {
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
                let body = self.statement_list(block.body(self.ast), true);
                self.namespace_block(body)
            }
            TsModuleBodyData::TsModuleDeclaration(module) => vec![self.namespace(module, true)],
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
    fn namespace_block(&mut self, body: SubRange<Stmt>) -> Vec<Stmt> {
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
    pub(super) fn namespace_import(
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
        let init = self.entity_expression(reference);
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
    pub(super) fn enumeration(
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
            let name = utils::enum_name(self.ast, member.id(self.ast));
            let mut value = self
                .semantic
                .enum_record
                .get(&(id.clone(), name.clone()))
                .unwrap()
                .clone();
            if matches!(value, Value::Opaque(_))
                && let Some(init) = member.initializer(self.ast)
            {
                let init = init.visit_mut_with(self).unwrap_or(init);
                value = Computer {
                    enum_id: &id,
                    resolver: self.resolver,
                    record: &self.semantic.enum_record,
                    ambient: &self.semantic.ambient_enum_record,
                    const_vars: &self.semantic.const_vars,
                    const_only: None,
                    ambient_const_only: None,
                }
                .compute(self.ast, init, EvalCtx::RECOMPUTE);
                if let Value::Opaque(expression) = &mut value {
                    let mut rewriter = EnumMemberRewriter {
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
            if !remove || !value.constant() {
                members.push((name, value, member.span(self.ast)));
            }
        }
        if members.is_empty() && enumeration.is_const(self.ast) {
            return None;
        }
        let opaque = members
            .iter()
            .any(|(_, value, _)| matches!(value, Value::Opaque(_)));
        let mut statements = Vec::new();
        for (name, value, span) in members {
            let string = matches!(value, Value::String(_));
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
    pub(super) fn jsx_name(&mut self, name: JsxElementName) -> Option<JsxElementName> {
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
    resolver: &mut crate::resolver::Resolver,
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
    resolver: &mut crate::resolver::Resolver,
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
struct EnumMemberRewriter<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut crate::resolver::Resolver,
    id: &'a Id,
    names: &'a HashSet<Wtf8Buf>,
}
impl EnumMemberRewriter<'_, '_> {
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
impl<'ast> VisitMut<'ast> for EnumMemberRewriter<'_, 'ast> {
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
