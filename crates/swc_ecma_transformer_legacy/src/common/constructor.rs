use crate::common::build as utils;
use swc_next_ecma_ast::*;
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_utils/src/constructor.rs#L9>.
pub(crate) fn inject_after_super(
    ast: &mut Ast<'_>,
    resolver: &mut crate::common::resolver::context::Resolver,
    body: FunctionBody,
    expressions: Vec<Expr>,
) {
    if expressions.is_empty() {
        return;
    }
    let mut injector = Injector {
        ast,
        resolver,
        expressions,
        ignore_return_value: false,
        injected: false,
    };
    body.visit_mut_with(&mut injector);
    if !injector.injected {
        let mut statements: Vec<_> = injector
            .expressions
            .iter()
            .map(|&expression| injector.ast.stmt_expression_statement(DUMMY_SP, expression))
            .collect();
        statements.extend(
            body.body(injector.ast)
                .iter()
                .map(|slot| injector.ast.get_node_in_sub_range(slot)),
        );
        let statements = injector.ast.add_sub_range(statements);
        body.set_body(injector.ast, statements);
    }
}

struct Injector<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut crate::common::resolver::context::Resolver,
    expressions: Vec<Expr>,
    ignore_return_value: bool,
    injected: bool,
}
impl<'ast> VisitMut<'ast> for Injector<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_function(&mut self, _: Function) -> Option<Function> {
        None
    }
    fn visit_mut_method_definition(
        &mut self,
        method: MethodDefinition,
    ) -> Option<MethodDefinition> {
        if method.kind(self.ast) != MethodDefinitionKind::Constructor {
            method.visit_mut_children_with(self);
        }
        None
    }
    fn visit_mut_expression_statement(
        &mut self,
        statement: ExpressionStatement,
    ) -> Option<ExpressionStatement> {
        let old = std::mem::replace(&mut self.ignore_return_value, true);
        statement.visit_mut_children_with(self);
        self.ignore_return_value = old;
        None
    }
    fn visit_mut_sequence_expression(
        &mut self,
        sequence: SequenceExpression,
    ) -> Option<SequenceExpression> {
        let expressions = sequence.expressions(self.ast);
        for (index, slot) in expressions.iter().enumerate() {
            let expression = self.ast.get_node_in_sub_range(slot);
            let old = self.ignore_return_value;
            if index + 1 != expressions.len() {
                self.ignore_return_value = true;
            }
            if let Some(replacement) = expression.visit_mut_with(self) {
                expressions.replace_slot(self.ast, slot, replacement);
            }
            self.ignore_return_value = old;
        }
        None
    }
    /// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_utils/src/constructor.rs#L72>.
    fn visit_mut_expr(&mut self, expression: Expr) -> Option<Expr> {
        let ignore = self.ignore_return_value;
        if !expression.is_parenthesized_expression(self.ast)
            && !expression.is_sequence_expression(self.ast)
        {
            self.ignore_return_value = false;
        }
        expression.visit_mut_children_with(self);
        self.ignore_return_value = ignore;
        let call = expression.as_call_expression(self.ast)?;
        if !call.callee(self.ast).is_super(self.ast) {
            return None;
        }
        self.injected = true;
        let mut expressions = vec![expression];
        for &expression in &self.expressions {
            expressions.push(utils::clone_expr(self.ast, self.resolver, expression));
        }
        Some(if ignore {
            let expressions = self.ast.add_sub_range(expressions);
            self.ast.expr_sequence_expression(DUMMY_SP, expressions)
        } else {
            let elements = self.ast.add_sub_range(
                expressions
                    .into_iter()
                    .map(|expression| Some(Argument::Expr(expression))),
            );
            let array = self.ast.expr_array_expression(DUMMY_SP, elements);
            let zero = self.ast.expr_numeric_literal(DUMMY_SP, 0.0, None);
            utils::computed(self.ast, array, zero)
        })
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L1577>.
pub(crate) fn default_constructor(
    ast: &mut Ast<'_>,
    resolver: &mut crate::common::resolver::context::Resolver,
    derived: bool,
) -> MethodDefinition {
    let params = utils::parameters(ast, []);
    let mut statements = Vec::new();
    if derived {
        let args = resolver.private_binding(ast, "args");
        let rest = ast.formal_parameter_rest(
            DUMMY_SP,
            SubRange::empty(),
            FormalParameterPattern::BindingIdentifier(args),
            None,
        );
        params.set_rest(ast, Some(rest));
        let args = resolver.reference(ast, args);
        let spread = ast.spread_element(DUMMY_SP, args);
        let arguments = ast.add_sub_range([Argument::SpreadElement(spread)]);
        let super_ = ast.expr_super(DUMMY_SP);
        let call = ast.expr_call_expression(DUMMY_SP, super_, None, arguments, false, false);
        statements.push(ast.stmt_expression_statement(DUMMY_SP, call));
    }
    let body = ast.add_sub_range(statements);
    let body = ast.function_body(DUMMY_SP, SubRange::empty(), body);
    let function = ast.function(
        DUMMY_SP,
        FunctionType::FunctionExpression,
        None,
        None,
        None,
        params,
        None,
        body,
        false,
        false,
    );
    let name = ast.add_utf8("constructor");
    let key = PropertyKey::IdentifierName(ast.identifier_name(DUMMY_SP, name));
    ast.method_definition(
        DUMMY_SP,
        SubRange::empty(),
        key,
        function,
        MethodDefinitionKind::Constructor,
        false,
        false,
        false,
        false,
        false,
        Accessibility::None,
    )
}
