//! Optional cleanup for SWC's Flow marker on the shared TypeScript AST.
use super::transform::Transform;
use crate::resolver::{Id, Resolver};
use std::collections::{HashMap, HashSet};
use swc_next_ecma_ast::*;

impl Transform<'_, '_> {
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L2011>.
    pub(super) fn flow_component_arrow(&mut self, declaration: VariableDeclarator) {
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
    pub(super) fn flow_method_key(&mut self, method: MethodDefinition) {
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
            && constructor_key(self.ast, method.key(self.ast))
        {
            let key = computed_constructor(self.ast);
            method.set_key(self.ast, key);
            method.set_computed(self.ast, true);
        }
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L692>.
    pub(super) fn flow_accessor_key(&mut self, property: PropertyDefinition) {
        if property.r#static(self.ast)
            && !property.computed(self.ast)
            && constructor_key(self.ast, property.key(self.ast))
        {
            let key = computed_constructor(self.ast);
            property.set_key(self.ast, key);
            property.set_computed(self.ast, true);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform.rs#L904>.
    pub(super) fn normalize_flow_await(&mut self) {
        let mut collector = AwaitBindings {
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
fn constructor_key(ast: &Ast<'_>, key: PropertyKey) -> bool {
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

struct AwaitBindings<'a, 'ast> {
    ast: &'a Ast<'ast>,
    resolver: &'a Resolver,
    used: HashSet<String>,
    seen: HashSet<Id>,
    bindings: Vec<Id>,
}
impl<'ast> Visit<'ast> for AwaitBindings<'_, 'ast> {
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
