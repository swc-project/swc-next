//! Isolated type inference from expressions and function return statements.

use swc_next_ecma_ast::{
    ArgumentData, ArrayExpression, ArrowFunctionBodyData, ArrowFunctionExpression, Ast, Diagnostic,
    Expr, ExprData, Function, GetSpan, TsLiteral, TsLiteralData, TsTupleElement, TsType,
    TsTypeAnnotation, TsTypeOperatorKind, UnaryOperator,
};

use super::Transformer;
use super::diagnostics::{
    ARRAY_REQUIRES_CONST, ARRAY_SPREAD_NOT_SUPPORTED, CLASS_EXPRESSION_NOT_SUPPORTED,
    EXPRESSION_TYPE_REQUIRED, FUNCTION_RETURN_TYPE_REQUIRED,
};
use crate::surface::{
    InferredType, first_template_quasi, inferred_type, return_expression, unparenthesized,
};

impl<'ast> Transformer<'_, 'ast> {
    pub(super) fn infer_type_from_expr(
        &mut self,
        expression: Expr,
        ast: &mut Ast<'ast>,
    ) -> Option<TsType> {
        self.emit_inferred_type(expression, false, ast)
    }

    pub(super) fn infer_const_type(
        &mut self,
        expression: Expr,
        ast: &mut Ast<'ast>,
    ) -> Option<TsType> {
        self.emit_inferred_type(expression, true, ast)
    }

    fn emit_inferred_type(
        &mut self,
        expression: Expr,
        is_const: bool,
        ast: &mut Ast<'ast>,
    ) -> Option<TsType> {
        let span = expression.span(ast);
        let ty = match inferred_type(expression, is_const, ast, self.semantic, &self.surface) {
            InferredType::Annotation(annotation) => annotation,
            InferredType::OptionalParameter(annotation) => {
                let undefined = ast.ts_type_ts_undefined_keyword(span);
                let types = ast.add_sub_range([annotation, undefined]);
                ast.ts_type_ts_union_type(span, types)
            }
            InferredType::Literal(literal) => {
                if let TsLiteralData::UnaryExpression(unary) = ast.ts_literal_data(literal) {
                    let argument = unparenthesized(unary.argument(ast), ast);
                    unary.set_argument(ast, argument);
                }
                ast.ts_type_ts_literal_type(span, literal)
            }
            InferredType::Template(quasi) => {
                let literal = ast.string_literal(span, quasi.cooked(ast), ast.empty_utf8());
                ast.ts_type_ts_literal_type(span, TsLiteral::StringLiteral(literal))
            }
            InferredType::String => ast.ts_type_ts_string_keyword(span),
            InferredType::Number => ast.ts_type_ts_number_keyword(span),
            InferredType::BigInt => ast.ts_type_ts_big_int_keyword(span),
            InferredType::Boolean => ast.ts_type_ts_boolean_keyword(span),
            InferredType::Null => ast.ts_type_ts_null_keyword(span),
            InferredType::Undefined => ast.ts_type_ts_undefined_keyword(span),
            InferredType::Object(object, is_const) => self.infer_object_type(object, is_const, ast),
            InferredType::Array(array) => self.infer_const_array_type(array, ast),
            InferredType::InvalidArray => {
                self.diagnostics
                    .push(Diagnostic::error(ARRAY_REQUIRES_CONST, span));
                ast.ts_type_ts_unknown_keyword(span)
            }
            InferredType::InvalidClass => {
                self.diagnostics
                    .push(Diagnostic::error(CLASS_EXPRESSION_NOT_SUPPORTED, span));
                ast.ts_type_ts_unknown_keyword(span)
            }
            InferredType::Function(function) => {
                let Some(return_type) = self.transform_function_signature(function, ast) else {
                    return Some(ast.ts_type_ts_unknown_keyword(span));
                };
                ast.ts_type_ts_function_type(
                    span,
                    function.type_parameters(ast),
                    function.this_param(ast),
                    function.params(ast),
                    return_type,
                )
            }
            InferredType::Arrow(function) => {
                let return_type = self.infer_arrow_return_type(function, ast);
                self.transform_formal_parameters(function.params(ast), ast);
                let Some(return_type) = return_type else {
                    self.diagnostics.push(Diagnostic::error(
                        FUNCTION_RETURN_TYPE_REQUIRED,
                        function.span(ast),
                    ));
                    return Some(ast.ts_type_ts_unknown_keyword(span));
                };
                ast.ts_type_ts_function_type(
                    span,
                    function.type_parameters(ast),
                    None,
                    function.params(ast),
                    return_type,
                )
            }
            InferredType::Unsupported => return None,
        };
        Some(ty)
    }

    fn infer_const_array_type(&mut self, array: ArrayExpression, ast: &mut Ast<'ast>) -> TsType {
        let span = array.span(ast);
        let elements = array.elements(ast);
        let mut element_types = Vec::with_capacity(elements.len());
        for element in elements.iter() {
            let element = ast.get_node_in_sub_range(element);
            let Some(element) = element else {
                let ty = ast.ts_type_ts_undefined_keyword(span);
                element_types.push(TsTupleElement::TsType(ty));
                continue;
            };
            let expression = match ast.argument_data(element) {
                ArgumentData::Expr(expression) => expression,
                ArgumentData::SpreadElement(spread) => {
                    self.diagnostics.push(Diagnostic::error(
                        ARRAY_SPREAD_NOT_SUPPORTED,
                        spread.span(ast),
                    ));
                    continue;
                }
            };
            let Some(ty) = self.infer_const_type(expression, ast) else {
                self.diagnostics.push(Diagnostic::error(
                    EXPRESSION_TYPE_REQUIRED,
                    expression.span(ast),
                ));
                continue;
            };
            element_types.push(TsTupleElement::TsType(ty));
        }
        let element_types = ast.add_sub_range(element_types);
        let tuple = ast.ts_type_ts_tuple_type(span, element_types);
        ast.ts_type_ts_type_operator(span, TsTypeOperatorKind::Readonly, tuple)
    }

    pub(super) fn infer_function_return_type(
        &mut self,
        function: Function,
        ast: &mut Ast<'ast>,
    ) -> Option<TsTypeAnnotation> {
        let body = function.body(ast);
        let expression = return_expression(body, ast)?;
        let ty = self.infer_type_from_expr(expression, ast)?;
        Some(ast.ts_type_annotation(function.span(ast), ty))
    }

    fn infer_arrow_return_type(
        &mut self,
        function: ArrowFunctionExpression,
        ast: &mut Ast<'ast>,
    ) -> Option<TsTypeAnnotation> {
        if let Some(return_type) = function.return_type(ast) {
            return Some(return_type);
        }
        if function.r#async(ast) {
            return None;
        }
        match ast.arrow_function_body_data(function.body(ast)) {
            ArrowFunctionBodyData::Expr(expression) => {
                let ty = self.infer_type_from_expr(expression, ast)?;
                Some(ast.ts_type_annotation(expression.span(ast), ty))
            }
            ArrowFunctionBodyData::FunctionBody(body) => {
                let expression = return_expression(body, ast)?;
                let ty = self.infer_type_from_expr(expression, ast)?;
                Some(ast.ts_type_annotation(function.span(ast), ty))
            }
        }
    }

    pub(super) fn is_literal_initializer(&self, expression: Expr, ast: &Ast<'ast>) -> bool {
        let expression = unparenthesized(expression, ast);
        match ast.expr_data(expression) {
            ExprData::StringLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::BooleanLiteral(_) => true,
            ExprData::TemplateLiteral(template) => template.expressions(ast).is_empty(),
            ExprData::UnaryExpression(unary) => {
                matches!(
                    unary.operator(ast),
                    UnaryOperator::Negate | UnaryOperator::Positive
                ) && matches!(
                    ast.expr_data(unparenthesized(unary.argument(ast), ast)),
                    ExprData::NumericLiteral(_) | ExprData::BigIntLiteral(_)
                )
            }
            _ => false,
        }
    }

    pub(super) fn normalize_literal_initializer(
        &self,
        expression: Expr,
        ast: &mut Ast<'ast>,
    ) -> Expr {
        let expression = unparenthesized(expression, ast);
        match ast.expr_data(expression) {
            ExprData::UnaryExpression(unary) => {
                let argument = unparenthesized(unary.argument(ast), ast);
                if unary.operator(ast) == UnaryOperator::Positive
                    && matches!(ast.expr_data(argument), ExprData::NumericLiteral(_))
                {
                    return argument;
                }
                unary.set_argument(ast, argument);
                expression
            }
            ExprData::TemplateLiteral(template) if template.expressions(ast).is_empty() => {
                let quasi = first_template_quasi(template, ast);
                let raw = ast.add_utf8("");
                ast.expr_string_literal(expression.span(ast), quasi.cooked(ast), raw)
            }
            _ => expression,
        }
    }
}
