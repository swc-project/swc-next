use swc_next_ecma_ast::{Argument, Ast, AstUtf8, DUMMY_SP, Expr, ExprData, PropertyKey};

pub(super) fn call_expression<I>(ast: &mut Ast<'_>, callee: AstUtf8, arguments: I) -> Expr
where
    I: IntoIterator<Item = Argument>,
{
    let callee = ast.expr_identifier_reference(DUMMY_SP, callee);
    let arguments = ast.add_sub_range(arguments);
    ast.expr_call_expression(DUMMY_SP, callee, None, arguments, false, false)
}

pub(super) fn qualified_name_expression(ast: &mut Ast<'_>, name: &str) -> Expr {
    let mut parts = name.split('.');
    let root = parts.next().unwrap_or(name);
    let mut expression = if root == "import" && parts.next() == Some("meta") {
        let import = ast.add_utf8("import");
        let meta = ast.add_utf8("meta");
        let import = ast.identifier_name(DUMMY_SP, import);
        let meta = ast.identifier_name(DUMMY_SP, meta);
        ast.expr_meta_property(DUMMY_SP, import, meta)
    } else {
        let root = ast.add_utf8(root);
        ast.expr_identifier_reference(DUMMY_SP, root)
    };
    for part in parts {
        let part = ast.add_utf8(part);
        let part = ast.identifier_name(DUMMY_SP, part);
        expression = ast.expr_member_expression(
            DUMMY_SP,
            expression,
            PropertyKey::IdentifierName(part),
            false,
            false,
        );
    }
    expression
}

pub(super) fn runtime_expression(mut expression: Expr, ast: &Ast<'_>) -> Expr {
    loop {
        expression = match ast.expr_data(expression) {
            ExprData::ParenthesizedExpression(wrapper) => wrapper.expression(ast),
            ExprData::TsAsExpression(wrapper) => wrapper.expression(ast),
            ExprData::TsSatisfiesExpression(wrapper) => wrapper.expression(ast),
            ExprData::TsNonNullExpression(wrapper) => wrapper.expression(ast),
            ExprData::TsTypeAssertion(wrapper) => wrapper.expression(ast),
            ExprData::TsInstantiationExpression(wrapper) => wrapper.expression(ast),
            _ => return expression,
        };
    }
}
