/// SWC's `optional_chaining::Config`. All flags default to `false`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// Assume values cannot be the special `document.all` object.
    pub no_document_all: bool,
    /// Assume property getters have no side effects.
    ///
    /// The singular spelling follows SWC's optional-chaining configuration.
    pub pure_getter: bool,
}

use crate::common::{build, resolver::context::Resolver};
use swc_next_ecma_ast::*;

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/optional_chaining_impl.rs#L9>.
pub(crate) fn optional_chaining_impl(ast: &mut Ast<'_>, resolver: &mut Resolver, options: Config) {
    let mut pass = OptionalChaining {
        ast,
        resolver,
        options,
        vars: Vec::new(),
    };
    pass.ast.root_program().visit_mut_with(&mut pass);
}

struct OptionalChaining<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    options: Config,
    vars: Vec<VariableDeclarator>,
}

#[derive(Clone, Copy)]
enum Memo {
    Cache(BindingIdentifier),
    Raw(Expr),
}
enum Gathering {
    Member(MemberExpression, Option<Memo>),
    Call(CallExpression, Option<Memo>),
}

impl OptionalChaining<'_, '_> {
    fn memo_expr(&mut self, memo: Memo) -> Expr {
        match memo {
            Memo::Cache(binding) => self.resolver.reference(self.ast, binding),
            Memo::Raw(expression) => build::clone_expr(self.ast, self.resolver, expression),
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/optional_chaining_impl.rs#L359>.
    fn memoize(&mut self, expression: Expr, call: bool) -> Memo {
        fn simple(ast: &Ast<'_>, expression: Expr) -> bool {
            match ast.expr_data(expression) {
                ExprData::IdentifierReference(_) | ExprData::ThisExpression(_) => true,
                ExprData::MemberExpression(member)
                    if !member.computed(ast)
                        && !member.optional(ast)
                        && !chain_part(ast, member.object(ast)) =>
                {
                    member.object(ast).is_super(ast) || simple(ast, member.object(ast))
                }
                _ => false,
            }
        }
        let resolved = expression
            .as_identifier_reference(self.ast)
            .is_some_and(|id| {
                self.resolver.node_scopes[id.node_id()] != self.resolver.unresolved()
            });
        if resolved || call && self.options.pure_getter && simple(self.ast, expression) {
            return Memo::Raw(expression);
        }
        let name = build::alias_ident_for(self.ast, expression, "_this");
        let binding = self.resolver.private_binding(self.ast, &name);
        self.vars.push(self.ast.variable_declarator(
            DUMMY_SP,
            BindingPattern::BindingIdentifier(binding),
            None,
            None,
            false,
        ));
        Memo::Cache(binding)
    }

    /// Next puts an optional chain around the full member/call spine; SWC puts
    /// `OptChainExpr` around each participating link. Plain accesses before the
    /// first optional operation are the base, rather than chain links.
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/optional_chaining_impl.rs#L175>.
    fn gather(&mut self, mut current: Expr) -> (Expr, Vec<Gathering>) {
        let mut chain = Vec::new();
        loop {
            let next = match self.ast.expr_data(current) {
                ExprData::MemberExpression(member) => {
                    let next = member.object(self.ast);
                    if member.computed(self.ast) {
                        let key = member.property(self.ast);
                        if let Some(key) = key.visit_mut_with(self) {
                            member.set_property(self.ast, key);
                        }
                    }
                    let memo = member.optional(self.ast).then(|| self.memoize(next, false));
                    chain.push(Gathering::Member(member, memo));
                    next
                }
                ExprData::CallExpression(call) => {
                    let next = call.callee(self.ast);
                    if let Some(arguments) = call.arguments(self.ast).visit_mut_with(self) {
                        call.set_arguments(self.ast, arguments);
                    }
                    let memo = call.optional(self.ast).then(|| self.memoize(next, true));
                    chain.push(Gathering::Call(call, memo));
                    next
                }
                _ => unreachable!("an optional chain has a member or call spine"),
            };
            if chain_part(self.ast, next) {
                current = next;
            } else {
                next.visit_mut_children_with(self);
                return (next, chain);
            }
        }
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/optional_chaining_impl.rs#L235>.
    fn construct(&mut self, base: Expr, chain: Vec<Gathering>, delete: bool) -> Expr {
        let mut current = base;
        let mut conditionals = Vec::new();
        let mut receiver = None;
        for part in chain.into_iter().rev() {
            current = match part {
                Gathering::Member(member, None) => {
                    receiver = None;
                    self.ast.expr_member_expression(
                        member.span(self.ast),
                        current,
                        member.property(self.ast),
                        member.computed(self.ast),
                        false,
                    )
                }
                Gathering::Call(call, None) => {
                    receiver = None;
                    self.ast.expr_call_expression(
                        call.span(self.ast),
                        current,
                        call.type_arguments(self.ast),
                        call.arguments(self.ast),
                        false,
                        call.pure(self.ast),
                    )
                }
                Gathering::Member(member, Some(memo)) => {
                    let test = self.init_and_eq_null_or_undefined(memo, current);
                    conditionals.push(test);
                    receiver = Some(memo);
                    let object = self.memo_expr(memo);
                    self.ast.expr_member_expression(
                        member.span(self.ast),
                        object,
                        member.property(self.ast),
                        member.computed(self.ast),
                        false,
                    )
                }
                Gathering::Call(call, Some(memo)) => {
                    let mut arguments: Vec<_> = call
                        .arguments(self.ast)
                        .iter()
                        .map(|slot| self.ast.get_node_in_sub_range(slot))
                        .collect();
                    let method = if let Some(member) = current.as_member_expression(self.ast) {
                        if member.object(self.ast).is_super(self.ast) {
                            arguments
                                .insert(0, Argument::Expr(self.ast.expr_this_expression(DUMMY_SP)));
                        } else {
                            let this = receiver.unwrap_or_else(|| {
                                let object = member.object(self.ast);
                                let this = self.memoize(object, true);
                                if let Memo::Cache(binding) = this {
                                    let left = self.resolver.reference(self.ast, binding);
                                    let object = build::assign(self.ast, left, object);
                                    member.set_object(self.ast, object);
                                }
                                this
                            });
                            arguments.insert(0, Argument::Expr(self.memo_expr(this)));
                        }
                        true
                    } else {
                        false
                    };
                    conditionals.push(self.init_and_eq_null_or_undefined(memo, current));
                    let callee = self.memo_expr(memo);
                    let callee = if method {
                        build::member(self.ast, callee, "call")
                    } else {
                        callee
                    };
                    let arguments = self.ast.add_sub_range(arguments);
                    receiver = None;
                    self.ast.expr_call_expression(
                        call.span(self.ast),
                        callee,
                        call.type_arguments(self.ast),
                        arguments,
                        false,
                        call.pure(self.ast),
                    )
                }
            };
        }
        if delete {
            current = self
                .ast
                .expr_unary_expression(DUMMY_SP, current, UnaryOperator::Delete);
        }
        for test in conditionals.into_iter().rev() {
            let consequent = if delete {
                self.ast.expr_boolean_literal(DUMMY_SP, true)
            } else {
                build::undefined(self.ast)
            };
            current = self
                .ast
                .expr_conditional_expression(DUMMY_SP, test, consequent, current);
        }
        current
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/optional_chaining_impl.rs#L425>.
    fn init_and_eq_null_or_undefined(&mut self, memo: Memo, init: Expr) -> Expr {
        let left = match memo {
            Memo::Cache(binding) => {
                let left = self.resolver.reference(self.ast, binding);
                build::assign(self.ast, left, init)
            }
            Memo::Raw(_) => self.memo_expr(memo),
        };
        let null = self.ast.expr_null_literal(DUMMY_SP);
        let null_test = self.ast.expr_binary_expression(
            DUMMY_SP,
            left,
            null,
            if self.options.no_document_all {
                BinaryOperator::Equal
            } else {
                BinaryOperator::StrictEqual
            },
        );
        if self.options.no_document_all {
            return null_test;
        }
        let left = self.memo_expr(memo);
        let undefined = build::undefined(self.ast);
        let undefined_test =
            self.ast
                .expr_binary_expression(DUMMY_SP, left, undefined, BinaryOperator::StrictEqual);
        self.ast
            .expr_logical_expression(DUMMY_SP, null_test, undefined_test, LogicalOperator::Or)
    }

    fn declarations(&mut self) -> Stmt {
        let vars = self.ast.add_sub_range(std::mem::take(&mut self.vars));
        Stmt::Declaration(Decl::VariableDeclaration(self.ast.variable_declaration(
            DUMMY_SP,
            VariableKind::Var,
            vars,
            false,
        )))
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/optional_chaining_impl.rs#L101>.
    fn initializer(&mut self, expression: Expr) -> Expr {
        let outer = std::mem::take(&mut self.vars);
        let expression = expression.visit_mut_with(self).unwrap_or(expression);
        let expression = if self.vars.is_empty() {
            expression
        } else {
            let declaration = self.declarations();
            let result = self.ast.stmt_return_statement(DUMMY_SP, Some(expression));
            let body = self.ast.add_sub_range([declaration, result]);
            let body = self.ast.function_body(DUMMY_SP, SubRange::empty(), body);
            let params = self.ast.formal_parameters(
                DUMMY_SP,
                SubRange::empty(),
                None,
                FormalParameterKind::ArrowFormalParameters,
            );
            let arrow = self.ast.expr_arrow_function_expression(
                DUMMY_SP,
                false,
                false,
                None,
                params,
                None,
                ArrowFunctionBody::FunctionBody(body),
            );
            build::call(self.ast, arrow, [], false)
        };
        self.vars = outer;
        expression
    }
}

fn chain_part(ast: &Ast<'_>, expression: Expr) -> bool {
    match ast.expr_data(expression) {
        ExprData::MemberExpression(member) => {
            member.optional(ast) || chain_part(ast, member.object(ast))
        }
        ExprData::CallExpression(call) => call.optional(ast) || chain_part(ast, call.callee(ast)),
        _ => false,
    }
}

impl<'ast> VisitMut<'ast> for OptionalChaining<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/optional_chaining_impl.rs#L74>.
    fn visit_mut_expr(&mut self, expression: Expr) -> Option<Expr> {
        if let Some(chain) = expression.as_chain_expression(self.ast) {
            let (base, parts) = self.gather(chain.expression(self.ast));
            return Some(self.construct(base, parts, false));
        }
        if let Some(unary) = expression.as_unary_expression(self.ast)
            && unary.operator(self.ast) == UnaryOperator::Delete
            && let Some(chain) = unary.argument(self.ast).as_chain_expression(self.ast)
        {
            let (base, parts) = self.gather(chain.expression(self.ast));
            return Some(self.construct(base, parts, true));
        }
        expression.visit_mut_children_with(self)
    }
    fn visit_mut_stmts(&mut self, statements: SubRange<Stmt>) -> Option<SubRange<Stmt>> {
        let outer = std::mem::take(&mut self.vars);
        let statements = statements
            .visit_mut_children_with(self)
            .unwrap_or(statements);
        let mut output: Vec<_> = statements
            .iter()
            .map(|slot| self.ast.get_node_in_sub_range(slot))
            .collect();
        if !self.vars.is_empty() {
            let declaration = self.declarations();
            let index = output
                .iter()
                .take_while(|statement| {
                    statement
                        .as_expression_statement(self.ast)
                        .is_some_and(|statement| {
                            statement.expression(self.ast).is_string_literal(self.ast)
                        })
                })
                .count();
            output.insert(index, declaration);
        }
        self.vars = outer;
        Some(self.ast.add_sub_range(output))
    }
    fn visit_mut_arrow_function_body(
        &mut self,
        body: ArrowFunctionBody,
    ) -> Option<ArrowFunctionBody> {
        let Some(expression) = body.as_expr(self.ast) else {
            return body.visit_mut_children_with(self);
        };
        let result = self.ast.stmt_return_statement(DUMMY_SP, Some(expression));
        let statements = self.ast.add_sub_range([result]);
        let function_body = self
            .ast
            .function_body(DUMMY_SP, SubRange::empty(), statements);
        function_body.visit_mut_with(self);
        let statements = function_body.body(self.ast);
        Some(if statements.len() == 1 {
            let statement = self
                .ast
                .get_node_in_sub_range(statements.iter().next().unwrap());
            ArrowFunctionBody::Expr(
                statement
                    .as_return_statement(self.ast)
                    .unwrap()
                    .argument(self.ast)
                    .unwrap(),
            )
        } else {
            ArrowFunctionBody::FunctionBody(function_body)
        })
    }
    fn visit_mut_arrow_function_expression(
        &mut self,
        arrow: ArrowFunctionExpression,
    ) -> Option<ArrowFunctionExpression> {
        arrow.visit_mut_children_with(self);
        arrow.set_expression(self.ast, arrow.body(self.ast).is_expr(self.ast));
        None
    }
    fn visit_mut_assignment_pattern(
        &mut self,
        pattern: AssignmentPattern,
    ) -> Option<AssignmentPattern> {
        let right = self.initializer(pattern.right(self.ast));
        pattern.set_right(self.ast, right);
        if let Some(left) = pattern.left(self.ast).visit_mut_with(self) {
            pattern.set_left(self.ast, left);
        }
        None
    }
    fn visit_mut_formal_parameter(
        &mut self,
        parameter: FormalParameter,
    ) -> Option<FormalParameter> {
        if let Some(initializer) = parameter.initializer(self.ast) {
            let initializer = self.initializer(initializer);
            parameter.set_initializer(self.ast, Some(initializer));
        }
        if let Some(pattern) = parameter.pattern(self.ast).visit_mut_with(self) {
            parameter.set_pattern(self.ast, pattern);
        }
        None
    }
}

/// SWC's class-properties visitor uses the same optional-chain pass locally,
/// then takes its temporaries into the private-access visitor's variable list.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/private_field.rs#L216>.
pub(crate) fn expression(
    ast: &mut Ast<'_>,
    resolver: &mut Resolver,
    options: Config,
    expression: Expr,
) -> (Expr, Vec<VariableDeclarator>) {
    resolver.register(ast);
    let mut pass = OptionalChaining {
        ast,
        resolver,
        options,
        vars: Vec::new(),
    };
    let expression = expression.visit_mut_with(&mut pass).unwrap_or(expression);
    (expression, pass.vars)
}
