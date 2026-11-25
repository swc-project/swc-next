/// SWC's `async_to_generator::Config`. All flags default to `false`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AsyncToGeneratorOptions {
    /// Allow lowering to change a function's observable `length`.
    pub ignore_function_length: bool,
}

use crate::common::{
    TraverseCtx, build as b,
    function::fn_env_hoister::{FnEnvHoister, init_this},
    resolver::context::Resolver,
};
use std::collections::HashMap;
use swc_next_ecma_ast::*;

mod loop_head_scope;

use self::loop_head_scope::LoopHeadScope;

#[derive(Clone, Copy)]
enum AwaitForMode {
    NativeAsync,
    Generator,
    AsyncGenerator,
}

#[derive(Default)]
struct FnState {
    should_transform: bool,
    is_generator: bool,
    is_arrow: bool,
    await_for_mode: Option<AwaitForMode>,
    use_this: bool,
    use_arguments: bool,
    use_super: bool,
    in_constructor: bool,
}
pub(crate) fn pass(
    context: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    transform_async_to_generator: bool,
    options: AsyncToGeneratorOptions,
) {
    let program = context.ast.root_program();
    program.visit_mut_with(&mut AsyncToGeneratorPass {
        context,
        transform_async_to_generator,
        options,
        fn_state: None,
        fn_state_stack: Vec::new(),
        constructors: HashMap::new(),
        this_var: None,
    });
}

struct AsyncToGeneratorPass<'a, 'ctx, 'ast, 'r> {
    context: &'a mut TraverseMutCtx<'ctx, 'ast, TraverseCtx<'r>>,
    transform_async_to_generator: bool,
    options: AsyncToGeneratorOptions,
    fn_state: Option<FnState>,
    fn_state_stack: Vec<Option<FnState>>,
    constructors: HashMap<NodeId, bool>,
    this_var: Option<BindingIdentifier>,
}

impl AsyncToGeneratorPass<'_, '_, '_, '_> {
    fn push(&mut self, state: FnState) {
        self.fn_state_stack.push(self.fn_state.take());
        self.fn_state = Some(state);
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/lib.rs#L175>.
impl<'ast> VisitMut<'ast> for AsyncToGeneratorPass<'_, '_, 'ast, '_> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.context.ast
    }

    fn visit_mut_class(&mut self, node: Class) -> Option<Class> {
        let ctx = &mut *self.context;
        for slot in node.body(ctx.ast).body(ctx.ast).iter() {
            if let Some(method) = ctx
                .ast
                .get_node_in_sub_range(slot)
                .as_method_definition(ctx.ast)
                && method.kind(ctx.ast) == MethodDefinitionKind::Constructor
            {
                self.constructors.insert(
                    method.value(ctx.ast).node_id(),
                    node.super_class(ctx.ast).is_some(),
                );
            }
        }
        node.visit_mut_children_with(self)
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2017/async_to_generator.rs#L157>.
    fn visit_mut_function(&mut self, node: Function) -> Option<Function> {
        let ctx = &mut *self.context;
        if let Some(derived) = self.constructors.get(&node.node_id()) {
            if *derived {
                self.push(FnState {
                    in_constructor: true,
                    ..Default::default()
                });
            }
        } else {
            let generator = node.generator(ctx.ast);
            let is_async = node.r#async(ctx.ast);
            self.push(FnState {
                should_transform: is_async && (generator || self.transform_async_to_generator),
                is_generator: generator,
                await_for_mode: is_async.then_some(if generator {
                    AwaitForMode::AsyncGenerator
                } else if self.transform_async_to_generator {
                    AwaitForMode::Generator
                } else {
                    AwaitForMode::NativeAsync
                }),
                ..Default::default()
            });
        }

        let replacement = node.visit_mut_children_with(self);
        let node = replacement.unwrap_or(node);
        let ctx = &mut *self.context;
        if let Some(derived) = self.constructors.get(&node.node_id()) {
            if *derived {
                let state = self.fn_state.take().unwrap();
                if state.use_this {
                    let binding = self.this_var.take().unwrap();
                    let body = node.body(ctx.ast);
                    init_this(ctx.ast, ctx.state.resolver, body, binding);
                    let decl = Stmt::Declaration(b::variable(
                        ctx.ast,
                        VariableKind::Var,
                        binding,
                        None,
                        DUMMY_SP,
                    ));
                    let statements = b::prepend_stmts(ctx.ast, [decl], body.body(ctx.ast));
                    body.set_body(ctx.ast, statements);
                }
                self.fn_state = self.fn_state_stack.pop().unwrap();
            }
            return replacement;
        }
        let mut state = self.fn_state.take().unwrap();
        self.fn_state = self.fn_state_stack.pop().unwrap();
        if !state.should_transform {
            return replacement;
        }
        let body = node.body(ctx.ast);
        let (body, mut statements) = if state.use_super {
            FnEnvHoister::visit(ctx.ast, ctx.state.resolver, body)
        } else {
            (body, Vec::new())
        };
        node.set_async(ctx.ast, false);
        node.set_generator(ctx.ast, false);
        state.use_arguments |=
            could_potentially_throw(ctx.ast, ctx.state.resolver, node.params(ctx.ast));
        let params = if state.use_arguments {
            let original = node.params(ctx.ast);
            let count = if self.options.ignore_function_length {
                0
            } else {
                original
                    .items(ctx.ast)
                    .iter()
                    .filter(|&slot| {
                        let parameter = ctx
                            .ast
                            .get_node_in_sub_range(slot)
                            .as_formal_parameter(ctx.ast)
                            .unwrap();
                        parameter.initializer(ctx.ast).is_none()
                    })
                    .count()
            };
            let bindings: Vec<_> = (0..count)
                .map(|index| {
                    ctx.state
                        .resolver
                        .private_binding(ctx.ast, &format!("_{index}"))
                })
                .collect();
            let params = b::parameters(ctx.ast, bindings);
            node.set_params(ctx.ast, params);
            original
        } else {
            b::parameters(ctx.ast, [])
        };
        let result = make_fn_ref(ctx, &state, params, body);
        statements.push(ctx.ast.stmt_return_statement(DUMMY_SP, Some(result)));
        let statements = ctx.ast.add_sub_range(statements);
        let body = ctx
            .ast
            .function_body(DUMMY_SP, SubRange::empty(), statements);
        node.set_body(ctx.ast, body);
        replacement
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2017/async_to_generator.rs#L198>.
    fn visit_mut_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
    ) -> Option<ArrowFunctionExpression> {
        let ctx = &mut *self.context;
        if node.r#async(ctx.ast) {
            let in_constructor = self
                .fn_state
                .as_ref()
                .is_some_and(|state| state.in_constructor);
            self.push(FnState {
                should_transform: self.transform_async_to_generator,
                is_arrow: true,
                await_for_mode: Some(if self.transform_async_to_generator {
                    AwaitForMode::Generator
                } else {
                    AwaitForMode::NativeAsync
                }),
                in_constructor,
                ..Default::default()
            });
        }

        let replacement = node.visit_mut_children_with(self);
        let node = replacement.unwrap_or(node);
        let ctx = &mut *self.context;
        if !node.r#async(ctx.ast) {
            return replacement;
        }
        let state = self.fn_state.take().unwrap();
        self.fn_state = self.fn_state_stack.pop().unwrap();
        if let Some(parent) = &mut self.fn_state
            && (state.should_transform || parent.should_transform || parent.is_arrow)
        {
            parent.use_this |= state.use_this;
            parent.use_arguments |= state.use_arguments;
            parent.use_super |= state.use_super;
        }
        if !state.should_transform {
            return replacement;
        }
        let body = node.body(ctx.ast);
        let (mut body, mut statements) = if state.use_super {
            FnEnvHoister::visit(ctx.ast, ctx.state.resolver, body)
        } else {
            (body, Vec::new())
        };
        if state.in_constructor && state.use_this {
            let binding = *self
                .this_var
                .get_or_insert_with(|| ctx.state.resolver.private_binding(ctx.ast, "_this"));
            body = replace_this_in_arrow_function_body(ctx.ast, ctx.state.resolver, body, binding);
        }
        let body = match ctx.ast.arrow_function_body_data(body) {
            ArrowFunctionBodyData::FunctionBody(body) => body,
            ArrowFunctionBodyData::Expr(expression) => {
                let result = ctx.ast.stmt_return_statement(DUMMY_SP, Some(expression));
                let statements = ctx.ast.add_sub_range([result]);
                ctx.ast
                    .function_body(DUMMY_SP, SubRange::empty(), statements)
            }
        };
        let params = b::parameters(ctx.ast, []);
        let result = make_fn_ref(ctx, &state, params, body);
        let body = if state.use_super {
            statements.push(ctx.ast.stmt_expression_statement(DUMMY_SP, result));
            let statements = ctx.ast.add_sub_range(statements);
            ArrowFunctionBody::FunctionBody(ctx.ast.function_body(
                DUMMY_SP,
                SubRange::empty(),
                statements,
            ))
        } else {
            ArrowFunctionBody::Expr(result)
        };
        node.set_body(ctx.ast, body);
        node.set_expression(ctx.ast, body.is_expr(ctx.ast));
        node.set_async(ctx.ast, false);
        replacement
    }
    fn visit_mut_super(&mut self, _: Super) -> Option<Super> {
        if let Some(state) = &mut self.fn_state {
            state.use_super = true;
        }
        None
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2017/async_to_generator.rs#L347>.
    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        let replacement = node.visit_mut_children_with(self);
        let node = replacement.unwrap_or(node);
        let ctx = &mut *self.context;
        let Some(state) = &mut self.fn_state else {
            return replacement;
        };
        if state.should_transform || state.is_arrow {
            if node.is_this_expression(ctx.ast) {
                state.use_this = true;
            }
            if node
                .as_identifier_reference(ctx.ast)
                .is_some_and(|id| ctx.ast.get_utf8(id.name(ctx.ast)) == "arguments")
            {
                state.use_arguments = true;
            }
        }
        if !state.should_transform {
            return replacement;
        }
        if let Some(await_) = node.as_await_expression(ctx.ast) {
            let mut argument = await_.argument(ctx.ast);
            if state.is_generator {
                let helper = ctx.state.helper(ctx.ast, "await_async_generator");
                argument = b::call(ctx.ast, helper, [argument], false);
            }
            return Some(Expr::YieldExpression(ctx.ast.yield_expression(
                await_.span(ctx.ast),
                Some(argument),
                false,
            )));
        }
        if let Some(yield_) = node.as_yield_expression(ctx.ast)
            && yield_.delegate(ctx.ast)
            && let Some(argument) = yield_.argument(ctx.ast)
        {
            let helper = ctx.state.helper(ctx.ast, "async_iterator");
            let argument = b::call(ctx.ast, helper, [argument], false);
            let helper = ctx.state.helper(ctx.ast, "async_generator_delegate");
            let argument = b::call(ctx.ast, helper, [argument], false);
            return Some(Expr::YieldExpression(ctx.ast.yield_expression(
                yield_.span(ctx.ast),
                Some(argument),
                true,
            )));
        }
        replacement
    }
    fn visit_mut_stmt(&mut self, node: Stmt) -> Option<Stmt> {
        let Some(FnState {
            await_for_mode: Some(mode),
            ..
        }) = self.fn_state
        else {
            return node.visit_mut_children_with(self);
        };
        let labeled = if node.is_labeled_statement(self.context.ast) {
            handle_await_for(self.context, node, mode)
        } else {
            None
        };
        let node = labeled.unwrap_or(node);
        let replacement = node.visit_mut_children_with(self).or(labeled);
        let node = replacement.unwrap_or(node);
        if node.is_for_of_statement(self.context.ast)
            && let Some(loop_stmt) = handle_await_for(self.context, node, mode)
        {
            return Some(loop_stmt);
        }
        replacement
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2017/async_to_generator.rs#L451>.
fn make_fn_ref(
    ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    state: &FnState,
    params: FormalParameters,
    body: FunctionBody,
) -> Expr {
    let function = Expr::Function(ctx.ast.function(
        DUMMY_SP,
        FunctionType::FunctionExpression,
        None,
        None,
        None,
        params,
        None,
        body,
        true,
        false,
    ));
    let helper = ctx.state.helper(
        ctx.ast,
        if state.is_generator {
            "wrap_async_generator"
        } else {
            "async_to_generator"
        },
    );
    let function = b::call(ctx.ast, helper, [function], false);
    if state.use_arguments {
        let callee = b::member(ctx.ast, function, "apply");
        let this = ctx.ast.expr_this_expression(DUMMY_SP);
        let arguments = ctx
            .state
            .resolver
            .marked_reference(ctx.ast, "arguments", true);
        b::call(ctx.ast, callee, [this, arguments], false)
    } else if state.use_this && !state.in_constructor {
        let callee = b::member(ctx.ast, function, "call");
        let this = ctx.ast.expr_this_expression(DUMMY_SP);
        b::call(ctx.ast, callee, [this], false)
    } else {
        b::call(ctx.ast, function, [], false)
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2017/async_to_generator.rs#L497>.
fn could_potentially_throw(ast: &Ast<'_>, resolver: &Resolver, params: FormalParameters) -> bool {
    for slot in params.items(ast).iter() {
        let item = ast.get_node_in_sub_range(slot);
        let parameter = item.as_formal_parameter(ast).unwrap();
        if let Some(init) = parameter.initializer(ast) {
            match ast.expr_data(init) {
                ExprData::IdentifierReference(id)
                    if ast.get_utf8(id.name(ast)) == "undefined"
                        && resolver.is_unresolved(id.node_id()) => {}
                ExprData::NullLiteral(_)
                | ExprData::BooleanLiteral(_)
                | ExprData::NumericLiteral(_)
                | ExprData::BigIntLiteral(_)
                | ExprData::StringLiteral(_)
                | ExprData::Function(_)
                | ExprData::ArrowFunctionExpression(_) => {}
                _ => return true,
            }
        } else if !parameter.pattern(ast).is_binding_identifier(ast) {
            return true;
        }
    }
    params
        .rest(ast)
        .is_some_and(|rest| !rest.argument(ast).is_binding_identifier(ast))
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2017/async_to_generator.rs#L530>.
fn handle_await_for(
    ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    mut statement: Stmt,
    mode: AwaitForMode,
) -> Option<Stmt> {
    let mut labels = Vec::new();
    while let Some(label) = statement.as_labeled_statement(ctx.ast) {
        labels.push((label.span(ctx.ast), label.label(ctx.ast)));
        statement = label.body(ctx.ast);
    }
    let loop_ = statement.as_for_of_statement(ctx.ast)?;
    if !loop_.r#await(ctx.ast) {
        return None;
    }
    let native_async = matches!(mode, AwaitForMode::NativeAsync);
    let loop_head_scope = if native_async {
        LoopHeadScope::new(ctx.ast, ctx.state.resolver, loop_)
    } else {
        None
    };
    let value = ctx.state.resolver.private_binding(ctx.ast, "_value");
    let iterator = ctx.state.resolver.private_binding(ctx.ast, "_iterator");
    let next_method = native_async.then(|| ctx.state.resolver.private_binding(ctx.ast, "_next"));
    let return_method =
        native_async.then(|| ctx.state.resolver.private_binding(ctx.ast, "_return"));
    let iterator_error = ctx
        .state
        .resolver
        .private_binding(ctx.ast, "_iteratorError");
    let step = ctx.state.resolver.private_binding(ctx.ast, "_step");
    let did_error = ctx
        .state
        .resolver
        .private_binding(ctx.ast, "_didIteratorError");
    let abrupt = ctx
        .state
        .resolver
        .private_binding(ctx.ast, "_iteratorAbruptCompletion");
    let err = ctx.state.resolver.private_binding(ctx.ast, "err");
    let reference = ctx.state.resolver.reference(ctx.ast, step);
    let get_value = b::member(ctx.ast, reference, "value");
    let first = Stmt::Declaration(b::variable(
        ctx.ast,
        VariableKind::Let,
        value,
        Some(get_value),
        DUMMY_SP,
    ));
    let reference = ctx.state.resolver.reference(ctx.ast, value);
    let next = match ctx.ast.for_statement_left_data(loop_.left(ctx.ast)) {
        ForStatementLeftData::VariableDeclaration(decl) => {
            let original = ctx
                .ast
                .get_node_in_sub_range(decl.declarators(ctx.ast).iter().next().unwrap());
            let declarator = ctx.ast.variable_declarator(
                DUMMY_SP,
                original.id(ctx.ast),
                None,
                Some(reference),
                false,
            );
            b::declaration(ctx.ast, decl.kind(ctx.ast), vec![declarator])
        }
        ForStatementLeftData::AssignmentTarget(target) => {
            let expr = ctx.ast.expr_assignment_expression(
                DUMMY_SP,
                target,
                reference,
                AssignmentOperator::Assign,
            );
            ctx.ast.stmt_expression_statement(DUMMY_SP, expr)
        }
    };
    let mut body = Vec::new();
    if let Some(next_method) = next_method {
        let next = call_iterator_method(ctx, iterator, next_method);
        let next = await_iteration(ctx, next, mode);
        let reference = ctx.state.resolver.reference(ctx.ast, step);
        let assignment = b::assign(ctx.ast, reference, next);
        body.push(ctx.ast.stmt_expression_statement(DUMMY_SP, assignment));
        body.push(check_iterator_object(ctx, step));
        let reference = ctx.state.resolver.reference(ctx.ast, step);
        let done = b::member(ctx.ast, reference, "done");
        let incomplete = ctx
            .ast
            .expr_unary_expression(DUMMY_SP, done, UnaryOperator::LogicalNot);
        let reference = ctx.state.resolver.reference(ctx.ast, abrupt);
        let assignment = b::assign(ctx.ast, reference, incomplete);
        body.push(ctx.ast.stmt_expression_statement(DUMMY_SP, assignment));
        let reference = ctx.state.resolver.reference(ctx.ast, abrupt);
        let done = ctx
            .ast
            .expr_unary_expression(DUMMY_SP, reference, UnaryOperator::LogicalNot);
        let stop = ctx.ast.stmt_break_statement(DUMMY_SP, None);
        body.push(ctx.ast.stmt_if_statement(DUMMY_SP, done, stop, None));
    }
    body.extend([first, next]);
    let original = loop_.body(ctx.ast);
    if let Some(block) = original.as_block_statement(ctx.ast) {
        body.extend(
            block
                .body(ctx.ast)
                .iter()
                .map(|slot| ctx.ast.get_node_in_sub_range(slot)),
        );
    } else {
        body.push(original);
    }
    let body = Stmt::BlockStatement(b::block(ctx.ast, body));
    let helper = ctx.state.helper(ctx.ast, "async_iterator");
    let init = b::call(ctx.ast, helper, [loop_.right(ctx.ast)], false);
    let mut try_stmts = Vec::new();
    let mut init_var_decls = Vec::new();
    if let Some(next_method) = next_method {
        if let Some(loop_head_scope) = loop_head_scope {
            loop_head_scope.init_iterator(
                ctx.ast,
                ctx.state.resolver,
                iterator,
                init,
                &mut try_stmts,
            );
        } else {
            try_stmts.push(Stmt::Declaration(b::variable(
                ctx.ast,
                VariableKind::Var,
                iterator,
                Some(init),
                DUMMY_SP,
            )));
        }
        try_stmts.push(check_iterator_object(ctx, iterator));
        let reference = ctx.state.resolver.reference(ctx.ast, iterator);
        let next = b::member(ctx.ast, reference, "next");
        init_var_decls.push(b::declarator(ctx.ast, next_method, Some(next)));
    } else {
        init_var_decls.push(b::declarator(ctx.ast, iterator, Some(init)));
    }
    init_var_decls.push(b::declarator(ctx.ast, step, None));
    let init = b::declaration(ctx.ast, VariableKind::Var, init_var_decls)
        .as_declaration(ctx.ast)
        .unwrap()
        .as_variable_declaration(ctx.ast)
        .unwrap();
    let test = (!native_async).then(|| {
        let reference = ctx.state.resolver.reference(ctx.ast, iterator);
        let callee = b::member(ctx.ast, reference, "next");
        let next = b::call(ctx.ast, callee, [], false);
        let next = await_iteration(ctx, next, mode);
        let reference = ctx.state.resolver.reference(ctx.ast, step);
        let next = b::assign(ctx.ast, reference, next);
        let done = b::member(ctx.ast, next, "done");
        let done = ctx
            .ast
            .expr_unary_expression(DUMMY_SP, done, UnaryOperator::LogicalNot);
        let reference = ctx.state.resolver.reference(ctx.ast, abrupt);
        b::assign(ctx.ast, reference, done)
    });
    let reference = ctx.state.resolver.reference(ctx.ast, abrupt);
    let false_ = ctx.ast.expr_boolean_literal(DUMMY_SP, false);
    let update = b::assign(ctx.ast, reference, false_);
    let mut for_ = Stmt::ForStatement(ctx.ast.for_statement(
        DUMMY_SP,
        Some(ForStatementInit::VariableDeclaration(init)),
        test,
        Some(update),
        body,
    ));
    for (span, label) in labels.into_iter().rev() {
        for_ = Stmt::LabeledStatement(ctx.ast.labeled_statement(span, label, for_));
    }
    try_stmts.push(for_);
    let try_body = b::block(ctx.ast, try_stmts);
    let reference = ctx.state.resolver.reference(ctx.ast, did_error);
    let true_ = ctx.ast.expr_boolean_literal(DUMMY_SP, true);
    let mark = b::assign(ctx.ast, reference, true_);
    let mark = ctx.ast.stmt_expression_statement(DUMMY_SP, mark);
    let left = ctx.state.resolver.reference(ctx.ast, iterator_error);
    let right = ctx.state.resolver.reference(ctx.ast, err);
    let store = b::assign(ctx.ast, left, right);
    let store = ctx.ast.stmt_expression_statement(DUMMY_SP, store);
    let catch_body = b::block(ctx.ast, [mark, store]);
    let catch = ctx.ast.catch_clause(
        DUMMY_SP,
        Some(BindingPattern::BindingIdentifier(err)),
        None,
        catch_body,
    );
    let error = ctx.state.resolver.reference(ctx.ast, iterator_error);
    let throw = ctx.ast.stmt_throw_statement(DUMMY_SP, error);
    let throw_body = Stmt::BlockStatement(b::block(ctx.ast, [throw]));
    let condition = ctx.state.resolver.reference(ctx.ast, did_error);
    let throw = ctx
        .ast
        .stmt_if_statement(DUMMY_SP, condition, throw_body, None);
    let inner_finally = b::block(ctx.ast, [throw]);
    let call = if let Some(return_method) = return_method {
        call_iterator_method(ctx, iterator, return_method)
    } else {
        let reference = ctx.state.resolver.reference(ctx.ast, iterator);
        let return_ = b::member(ctx.ast, reference, "return");
        b::call(ctx.ast, return_, [], false)
    };
    let awaited = await_iteration(ctx, call, mode);
    let close_body = if native_async {
        let reference = ctx.state.resolver.reference(ctx.ast, step);
        let assignment = b::assign(ctx.ast, reference, awaited);
        let assignment = ctx.ast.stmt_expression_statement(DUMMY_SP, assignment);
        let validate = check_iterator_object(ctx, step);
        b::block(ctx.ast, [assignment, validate])
    } else {
        let awaited = ctx.ast.stmt_expression_statement(DUMMY_SP, awaited);
        b::block(ctx.ast, [awaited])
    };
    let reference = ctx.state.resolver.reference(ctx.ast, iterator);
    let return_ = b::member(ctx.ast, reference, "return");
    let return_ = if let Some(return_method) = return_method {
        let reference = ctx.state.resolver.reference(ctx.ast, return_method);
        b::assign(ctx.ast, reference, return_)
    } else {
        return_
    };
    let null = ctx.ast.expr_null_literal(DUMMY_SP);
    let test = ctx
        .ast
        .expr_binary_expression(DUMMY_SP, return_, null, BinaryOperator::NotEqual);
    let reference = ctx.state.resolver.reference(ctx.ast, abrupt);
    let test = ctx
        .ast
        .expr_logical_expression(DUMMY_SP, reference, test, LogicalOperator::And);
    let close = ctx
        .ast
        .stmt_if_statement(DUMMY_SP, test, Stmt::BlockStatement(close_body), None);
    let close = b::block(ctx.ast, [close]);
    let inner_try =
        Stmt::TryStatement(
            ctx.ast
                .try_statement(DUMMY_SP, close, None, Some(inner_finally)),
        );
    let finally = b::block(ctx.ast, [inner_try]);
    let try_ = Stmt::TryStatement(ctx.ast.try_statement(
        loop_.span(ctx.ast),
        try_body,
        Some(catch),
        Some(finally),
    ));
    let false_ = ctx.ast.expr_boolean_literal(DUMMY_SP, false);
    let abrupt = b::declarator(ctx.ast, abrupt, Some(false_));
    let false_ = ctx.ast.expr_boolean_literal(DUMMY_SP, false);
    let did_error = b::declarator(ctx.ast, did_error, Some(false_));
    let error = b::declarator(ctx.ast, iterator_error, None);
    let mut declarations = vec![abrupt, did_error, error];
    if let Some(return_method) = return_method {
        declarations.push(b::declarator(ctx.ast, return_method, None));
    }
    let declarations = b::declaration(ctx.ast, VariableKind::Var, declarations);
    Some(Stmt::BlockStatement(b::block(
        ctx.ast,
        [declarations, try_],
    )))
}

fn call_iterator_method(
    ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    iterator: BindingIdentifier,
    method: BindingIdentifier,
) -> Expr {
    let reflect = ctx
        .state
        .resolver
        .marked_reference(ctx.ast, "Reflect", true);
    let apply = b::member(ctx.ast, reflect, "apply");
    let method = ctx.state.resolver.reference(ctx.ast, method);
    let receiver = ctx.state.resolver.reference(ctx.ast, iterator);
    let args = ctx.ast.expr_array_expression(DUMMY_SP, SubRange::empty());
    b::call(ctx.ast, apply, [method, receiver, args], false)
}

fn check_iterator_object(
    ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    value: BindingIdentifier,
) -> Stmt {
    let mut is_not_type = |kind| {
        let value = ctx.state.resolver.reference(ctx.ast, value);
        let actual = ctx
            .ast
            .expr_unary_expression(DUMMY_SP, value, UnaryOperator::Typeof);
        let expected = b::string(ctx.ast, kind);
        ctx.ast
            .expr_binary_expression(DUMMY_SP, actual, expected, BinaryOperator::StrictNotEqual)
    };
    let not_object = is_not_type("object");
    let not_function = is_not_type("function");
    let primitive =
        ctx.ast
            .expr_logical_expression(DUMMY_SP, not_object, not_function, LogicalOperator::And);
    let reference = ctx.state.resolver.reference(ctx.ast, value);
    let null = ctx.ast.expr_null_literal(DUMMY_SP);
    let is_null =
        ctx.ast
            .expr_binary_expression(DUMMY_SP, reference, null, BinaryOperator::StrictEqual);
    let invalid =
        ctx.ast
            .expr_logical_expression(DUMMY_SP, is_null, primitive, LogicalOperator::Or);
    let error = ctx
        .state
        .resolver
        .marked_reference(ctx.ast, "TypeError", true);
    let message = b::string(ctx.ast, "Iterator result is not an object");
    let arguments = ctx.ast.add_sub_range([Argument::Expr(message)]);
    let error = ctx
        .ast
        .expr_new_expression(DUMMY_SP, error, None, arguments);
    let throw = ctx.ast.stmt_throw_statement(DUMMY_SP, error);
    ctx.ast.stmt_if_statement(DUMMY_SP, invalid, throw, None)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2017/async_to_generator.rs#L1143>.
fn await_iteration(
    ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    mut expression: Expr,
    mode: AwaitForMode,
) -> Expr {
    if matches!(mode, AwaitForMode::NativeAsync) {
        return Expr::AwaitExpression(ctx.ast.await_expression(DUMMY_SP, expression));
    }
    if matches!(mode, AwaitForMode::AsyncGenerator) {
        let helper = ctx.state.helper(ctx.ast, "await_async_generator");
        expression = b::call(ctx.ast, helper, [expression], false);
    }
    Expr::YieldExpression(ctx.ast.yield_expression(DUMMY_SP, Some(expression), false))
}

struct ReplaceThis<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    binding: BindingIdentifier,
}
impl<'ast> VisitMut<'ast> for ReplaceThis<'_, 'ast> {
    fn visit_mut_function(&mut self, _: Function) -> Option<Function> {
        None
    }

    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }
    fn visit_mut_class(&mut self, _: Class) -> Option<Class> {
        None
    }
    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        node.visit_mut_children_with(self);
        node.is_this_expression(self.ast)
            .then(|| self.resolver.reference(self.ast, self.binding))
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transformer/src/es2017/async_to_generator.rs#L1181>.
fn replace_this_in_arrow_function_body(
    ast: &mut Ast<'_>,
    resolver: &mut Resolver,
    node: ArrowFunctionBody,
    binding: BindingIdentifier,
) -> ArrowFunctionBody {
    node.visit_mut_with(&mut ReplaceThis {
        ast,
        resolver,
        binding,
    })
    .unwrap_or(node)
}
