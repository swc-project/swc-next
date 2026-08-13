use serde::Deserialize;
use swc_next_ecma_ast::{
    Ast, Class, DUMMY_SP, Expr, FormalParameterKind, Function, FunctionType, GetSpan,
    IdentifierReference, MethodDefinition, MethodDefinitionKind, StaticBlock, Stmt, TypedSubRange,
    VisitMut, VisitMutWith,
};
use swc_next_ecma_semantic::Semantic;

use crate::es2015::util::fn_env_hoister::FnEnvHoister;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ArrowFunctionsOptions {
    pub spec: bool,
}

/// Compile ES2015 arrow functions to function expressions.
pub(super) struct ArrowFunctions {
    _options: ArrowFunctionsOptions,
}

impl ArrowFunctions {
    pub fn new(options: ArrowFunctionsOptions) -> Self {
        Self { _options: options }
    }

    pub fn transform<'a>(&mut self, ast: &mut Ast<'a>, semantic: &Semantic) {
        let program = ast.root_program();
        let mut visitor = ArrowVisitor::new(ast, semantic);
        program.visit_mut_children_with(&mut visitor);
        let frame = visitor.hoister.take_frame();
        visitor
            .hoister
            .inject_into_program(program, frame, visitor.ast);
    }
}

struct ArrowVisitor<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    hoister: FnEnvHoister<'visit>,
    in_subclass: bool,
    next_method_is_derived_constructor: Option<bool>,
}

impl<'visit, 'ast> ArrowVisitor<'visit, 'ast> {
    fn new(ast: &'visit mut Ast<'ast>, semantic: &'visit Semantic) -> Self {
        let hoister = FnEnvHoister::new(ast, semantic);
        Self {
            ast,
            hoister,
            in_subclass: false,
            next_method_is_derived_constructor: None,
        }
    }

    fn convert_arrow(&mut self, arrow: swc_next_ecma_ast::ArrowFunctionExpression) -> Function {
        let (span, params, body, is_async) = (
            arrow.span(self.ast),
            arrow.params(self.ast),
            arrow.body(self.ast),
            arrow.r#async(self.ast),
        );

        let old_in_arrow = self.hoister.set_in_arrow(true);
        let params = params.visit_mut_with(self).unwrap_or(params);
        let body = body.visit_mut_with(self).unwrap_or(body);
        self.hoister.set_in_arrow(old_in_arrow);

        params.set_kind(self.ast, FormalParameterKind::FormalParameters);
        let body = if let Some(body) = body.as_function_body(self.ast) {
            body
        } else {
            let expression = body
                .as_expr(self.ast)
                .expect("an arrow body is either a function body or an expression");
            let return_statement = self.ast.return_statement(DUMMY_SP, Some(expression));
            let body_items = self
                .ast
                .add_typed_sub_range([Stmt::ReturnStatement(return_statement)]);
            self.ast
                .function_body(DUMMY_SP, TypedSubRange::empty(), body_items)
        };

        self.ast.function(
            span,
            FunctionType::FunctionExpression,
            None,
            false,
            is_async,
            params,
            body,
            None,
            None,
        )
    }
}

impl<'ast> VisitMut<'ast> for ArrowVisitor<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        if let Some(arrow) = node.as_arrow_function_expression(self.ast) {
            Some(Expr::Function(self.convert_arrow(arrow)))
        } else if let Some(this) = node.as_this_expression(self.ast) {
            Some(self.hoister.transform_this_expression(this, self.ast))
        } else if let Some(meta) = node.as_meta_property(self.ast) {
            Some(self.hoister.transform_meta_property(meta, self.ast))
        } else {
            node.visit_mut_children_with(self)
        }
    }

    fn visit_mut_identifier_reference(
        &mut self,
        node: IdentifierReference,
    ) -> Option<IdentifierReference> {
        self.hoister.visit_identifier_reference(node, self.ast);
        None
    }

    fn visit_mut_function(&mut self, node: Function) -> Option<Function> {
        let derived_constructor = self
            .next_method_is_derived_constructor
            .take()
            .unwrap_or(false);
        let old_in_arrow = self.hoister.set_in_arrow(false);
        self.hoister.push_frame();
        node.visit_mut_children_with(self);
        let frame = self.hoister.take_frame();
        self.hoister
            .inject_into_function(node, frame, derived_constructor, self.ast);
        self.hoister.set_in_arrow(old_in_arrow);
        None
    }

    fn visit_mut_class(&mut self, node: Class) -> Option<Class> {
        let old = self.in_subclass;
        self.in_subclass = node.super_class(self.ast).is_some();
        node.visit_mut_children_with(self);
        self.in_subclass = old;
        None
    }

    fn visit_mut_method_definition(&mut self, node: MethodDefinition) -> Option<MethodDefinition> {
        let decorators = node.decorators(self.ast);
        let key = node.key(self.ast);
        if let Some(decorators) = decorators.visit_mut_with(self) {
            node.set_decorators(self.ast, decorators);
        }
        if let Some(key) = key.visit_mut_with(self) {
            node.set_key(self.ast, key);
        }

        self.next_method_is_derived_constructor =
            Some(self.in_subclass && node.kind(self.ast) == MethodDefinitionKind::Constructor);
        let value = node.value(self.ast);
        if let Some(value) = value.visit_mut_with(self) {
            node.set_value(self.ast, value);
        }
        debug_assert!(self.next_method_is_derived_constructor.is_none());
        None
    }

    fn visit_mut_static_block(&mut self, node: StaticBlock) -> Option<StaticBlock> {
        let old_in_arrow = self.hoister.set_in_arrow(false);
        self.hoister.push_frame();
        node.visit_mut_children_with(self);
        let frame = self.hoister.take_frame();
        self.hoister.inject_into_static_block(node, frame, self.ast);
        self.hoister.set_in_arrow(old_in_arrow);
        None
    }
}
