//! Classify inferable expressions and direct returns without rewriting the AST.

use swc_next_ecma_ast::{
    ArrayExpression, ArrowFunctionExpression, Ast, Class, Expr, ExprData, Function, FunctionBody,
    IdentifierReference, ObjectExpression, ReturnStatement, StmtData, TemplateElement,
    TemplateLiteral, TsLiteral, TsType, TsTypeData, TsTypeNameData, UnaryOperator, Visit,
    VisitWith,
};
use swc_next_ecma_semantic::Semantic;

use super::DeclarationSurface;

pub(crate) fn unparenthesized(mut expression: Expr, ast: &Ast<'_>) -> Expr {
    while let ExprData::ParenthesizedExpression(parenthesized) = ast.expr_data(expression) {
        expression = parenthesized.expression(ast);
    }
    expression
}

pub(crate) fn first_template_quasi(template: TemplateLiteral, ast: &Ast<'_>) -> TemplateElement {
    let first = template
        .quasis(ast)
        .first()
        .expect("every template literal has at least one quasi");
    ast.get_node_in_sub_range(first)
}

pub(crate) enum ExportExpression {
    Reference(IdentifierReference),
    Class(Class),
    Inferred(Expr),
}

pub(crate) fn export_expression(expression: Expr, ast: &Ast<'_>) -> ExportExpression {
    let expression = unparenthesized(expression, ast);
    match ast.expr_data(expression) {
        ExprData::IdentifierReference(reference) => ExportExpression::Reference(reference),
        ExprData::Class(class) => ExportExpression::Class(class),
        _ => ExportExpression::Inferred(expression),
    }
}

pub(crate) enum InferredType {
    Annotation(TsType),
    OptionalParameter(TsType),
    Literal(TsLiteral),
    Template(TemplateElement),
    String,
    Number,
    BigInt,
    Boolean,
    Null,
    Undefined,
    Object(ObjectExpression, bool),
    Array(ArrayExpression),
    Function(Function),
    Arrow(ArrowFunctionExpression),
    InvalidArray,
    InvalidClass,
    Unsupported,
}

pub(crate) fn inferred_type(
    expression: Expr,
    is_const: bool,
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
    surface: &DeclarationSurface,
) -> InferredType {
    let expression = unparenthesized(expression, ast);
    match ast.expr_data(expression) {
        ExprData::TsAsExpression(assertion) => asserted_type(
            assertion.expression(ast),
            assertion.type_annotation(ast),
            ast,
            semantic,
            surface,
        ),
        ExprData::TsTypeAssertion(assertion) => asserted_type(
            assertion.expression(ast),
            assertion.type_annotation(ast),
            ast,
            semantic,
            surface,
        ),
        ExprData::TsNonNullExpression(_) => InferredType::Unsupported,
        ExprData::StringLiteral(literal) => {
            if is_const {
                InferredType::Literal(TsLiteral::StringLiteral(literal))
            } else {
                InferredType::String
            }
        }
        ExprData::NumericLiteral(literal) => {
            if is_const {
                InferredType::Literal(TsLiteral::NumericLiteral(literal))
            } else {
                InferredType::Number
            }
        }
        ExprData::BigIntLiteral(literal) => {
            if is_const {
                InferredType::Literal(TsLiteral::BigIntLiteral(literal))
            } else {
                InferredType::BigInt
            }
        }
        ExprData::BooleanLiteral(literal) => {
            if is_const {
                InferredType::Literal(TsLiteral::BooleanLiteral(literal))
            } else {
                InferredType::Boolean
            }
        }
        ExprData::TemplateLiteral(_) if !is_const => InferredType::String,
        ExprData::TemplateLiteral(template) if template.expressions(ast).is_empty() => {
            InferredType::Template(first_template_quasi(template, ast))
        }
        ExprData::NullLiteral(_) => InferredType::Null,
        ExprData::IdentifierReference(identifier) => {
            if ast.get_utf8(identifier.name(ast)) == "undefined"
                && semantic
                    .reference_of(identifier.node_id())
                    .is_some_and(|id| semantic.reference(id).symbol.is_none())
            {
                InferredType::Undefined
            } else if let Some(reference) = semantic.reference_of(identifier.node_id())
                && let Some(symbol) = semantic.reference(reference).resolution().symbol()
                && let Some(annotation) = surface.annotations.get(&symbol)
            {
                if annotation.optional {
                    InferredType::OptionalParameter(annotation.annotation)
                } else {
                    InferredType::Annotation(annotation.annotation)
                }
            } else {
                InferredType::Unsupported
            }
        }
        ExprData::UnaryExpression(unary)
            if matches!(
                unary.operator(ast),
                UnaryOperator::Negate | UnaryOperator::Positive
            ) =>
        {
            match ast.expr_data(unparenthesized(unary.argument(ast), ast)) {
                ExprData::NumericLiteral(number) => {
                    if !is_const {
                        InferredType::Number
                    } else if unary.operator(ast) == UnaryOperator::Positive {
                        InferredType::Literal(TsLiteral::NumericLiteral(number))
                    } else {
                        InferredType::Literal(TsLiteral::UnaryExpression(unary))
                    }
                }
                ExprData::BigIntLiteral(_) => {
                    if is_const {
                        InferredType::Literal(TsLiteral::UnaryExpression(unary))
                    } else {
                        InferredType::BigInt
                    }
                }
                _ => InferredType::Unsupported,
            }
        }
        ExprData::ObjectExpression(object) => InferredType::Object(object, is_const),
        ExprData::ArrayExpression(array) => {
            if is_const {
                InferredType::Array(array)
            } else {
                InferredType::InvalidArray
            }
        }
        ExprData::Function(function) => InferredType::Function(function),
        ExprData::ArrowFunctionExpression(function) => InferredType::Arrow(function),
        ExprData::Class(_) if !is_const => InferredType::InvalidClass,
        _ => InferredType::Unsupported,
    }
}

fn asserted_type(
    expression: Expr,
    annotation: TsType,
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
    surface: &DeclarationSurface,
) -> InferredType {
    let is_const = if let TsTypeData::TsTypeReference(reference) = ast.ts_type_data(annotation)
        && let TsTypeNameData::IdentifierReference(identifier) =
            ast.ts_type_name_data(reference.type_name(ast))
    {
        ast.get_utf8(identifier.name(ast)) == "const"
    } else {
        false
    };
    if is_const {
        inferred_type(expression, true, ast, semantic, surface)
    } else {
        InferredType::Annotation(annotation)
    }
}

/// Only a single, direct return expression is syntactically inferable. Nested returns
/// require control-flow analysis and must not be mistaken for an unconditional return.
pub(crate) fn return_expression(body: FunctionBody, ast: &Ast<'_>) -> Option<Expr> {
    let mut direct = None;
    for statement in body.body(ast).iter() {
        let statement = ast.get_node_in_sub_range(statement);
        if let StmtData::ReturnStatement(statement) = ast.stmt_data(statement) {
            if direct.is_some() {
                return None;
            }
            direct = Some(statement);
        }
    }
    let direct = direct?;
    let mut collector = ReturnCollector {
        ast,
        direct,
        nested: false,
    };
    body.visit_with(&mut collector);
    if collector.nested {
        None
    } else {
        direct.argument(ast)
    }
}

struct ReturnCollector<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    direct: ReturnStatement,
    nested: bool,
}
impl<'ast> Visit<'ast> for ReturnCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }
    fn visit_return_statement(&mut self, statement: ReturnStatement) {
        self.nested |= statement != self.direct;
    }
    fn visit_function(&mut self, _: Function) {}
    fn visit_arrow_function_expression(&mut self, _: ArrowFunctionExpression) {}
    fn visit_class(&mut self, _: Class) {}
}
