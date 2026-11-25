use rustc_hash::FxHashSet;
use swc_next_ecma_ast::*;

use crate::common::{build, resolver::context::Resolver};

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/util.rs#L7>.
pub(super) fn is_builtin_hook(name: &str) -> bool {
    matches!(
        name,
        "useState"
            | "useReducer"
            | "useEffect"
            | "useLayoutEffect"
            | "useMemo"
            | "useCallback"
            | "useRef"
            | "useContext"
            | "useImperativeHandle"
            | "useDebugValue"
    )
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/util.rs#L23>.
pub(super) fn is_body_arrow_fn(ast: &Ast<'_>, body: ArrowFunctionBody) -> bool {
    body.as_expr(ast)
        .is_some_and(|body| body.is_arrow_function_expression(ast))
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/util.rs#L43>.
pub(super) fn make_assign_stmt(
    ast: &mut Ast<'_>,
    resolver: &mut Resolver,
    handle: BindingIdentifier,
    expr: Expr,
) -> Expr {
    let span = expr.span(ast);
    let left = resolver.reference(ast, handle);
    let left = AssignmentTarget::SimpleAssignmentTarget(build::target(ast, left));
    ast.expr_assignment_expression(span, left, expr, AssignmentOperator::Assign)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/util.rs#L55>.
pub(super) fn make_call_stmt(ast: &mut Ast<'_>, handle: Expr) -> Stmt {
    let expr = make_call_expr(ast, handle);
    ast.stmt_expression_statement(DUMMY_SP, expr)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/util.rs#L63>.
pub(super) fn make_call_expr(ast: &mut Ast<'_>, handle: Expr) -> Expr {
    build::call(ast, handle, [], false)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/util.rs#L73>.
pub(super) fn is_import_or_require(ast: &Ast<'_>, expr: Expr) -> bool {
    match ast.expr_data(expr) {
        ExprData::CallExpression(call) => call
            .callee(ast)
            .as_identifier_reference(ast)
            .is_some_and(|ident| ast.get_utf8(ident.name(ast)).contains("require")),
        ExprData::ImportExpression(_) => true,
        _ => false,
    }
}

pub(super) fn span_to_snippet<'ast>(ast: &Ast<'ast>, span: Span) -> &'ast str {
    ast.source()
        .get(span.start as usize..span.end as usize)
        .unwrap_or_default()
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/util.rs#L93>.
struct UsedInJsx<'a, 'ast> {
    ast: &'a Ast<'ast>,
    resolver: &'a Resolver,
    used_in_jsx: FxHashSet<SymbolId>,
}

impl<'ast> Visit<'ast> for UsedInJsx<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/util.rs#L98>.
    fn visit_call_expression(&mut self, node: CallExpression) {
        node.visit_children_with(self);

        let ast = self.ast;
        let name = match ast.expr_data(node.callee(ast)) {
            ExprData::IdentifierReference(ident) => ident.name(ast),
            ExprData::MemberExpression(member)
                if !member.computed(ast) && !member.object(ast).is_super(ast) =>
            {
                let Some(ident) = member.property(ast).as_identifier_name(ast) else {
                    return;
                };
                ident.name(ast)
            }
            _ => return,
        };
        if matches!(
            ast.get_utf8(name),
            "createElement" | "jsx" | "jsxDEV" | "jsxs"
        ) && !build::chain_part(ast, Expr::CallExpression(node))
            && let Some(argument) = node.arguments(ast).get_node(ast, 0)
            && let Some(ident) = argument_expr(ast, argument).as_identifier_reference(ast)
            && let Some(symbol) = self.resolver.symbol(ident.node_id())
        {
            self.used_in_jsx.insert(symbol);
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/util.rs#L123>.
    fn visit_jsx_opening_element(&mut self, node: JsxOpeningElement) {
        if let Some(ident) = node.name(self.ast).as_jsx_identifier(self.ast)
            && let Some(symbol) = self.resolver.symbol(ident.node_id())
        {
            self.used_in_jsx.insert(symbol);
        }
    }

    fn visit_ts_type(&mut self, _: TsType) {}

    fn visit_ts_type_element(&mut self, _: TsTypeElement) {}
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/util.rs#L130>.
pub(super) fn collect_ident_in_jsx(
    ast: &Ast<'_>,
    resolver: &Resolver,
    items: SubRange<Stmt>,
) -> FxHashSet<SymbolId> {
    let mut visitor = UsedInJsx {
        ast,
        resolver,
        used_in_jsx: FxHashSet::default(),
    };
    items.visit_with(&mut visitor);
    visitor.used_in_jsx
}

pub(super) fn argument_expr(ast: &Ast<'_>, argument: Argument) -> Expr {
    match ast.argument_data(argument) {
        ArgumentData::Expr(expr) => expr,
        ArgumentData::SpreadElement(spread) => spread.argument(ast),
    }
}
