//! ES2017 async-function lowering.
//!
//! The output shape follows SWC's async-to-generator transform: retain the
//! original function as a wrapper and run its former body through the helper.
//! The traversal state keeps `await`, `this`, and `arguments` attached to the
//! nearest ordinary function while allowing nested arrows to inherit them.

use swc_next_ecma_ast::{
    Argument, ArrowFunctionBody, ArrowFunctionExpression, AssignmentOperator, AssignmentTarget,
    BinaryOperator, BindingPattern, DUMMY_SP, Expr, FormalParameterItem, FormalParameterKind,
    FormalParameterPattern, FormalParameters, Function, FunctionBody, FunctionType, GetSpan,
    LogicalOperator, MemberExpression, NodeKindData, PropertyKey, SimpleAssignmentTarget, Stmt,
    SubRange, TraverseAction, UpdateOperator, VariableKind,
};
use swc_next_ecma_semantic::{
    ReferenceFlags, SymbolFlags,
    scope::{ScopeFlags, ScopeKind},
};

use crate::context::{BoundIdentifier, TransformCtx};

#[derive(Default)]
struct FunctionState {
    should_transform: bool,
    use_this: bool,
    use_arguments: bool,
    has_parameter_expressions: bool,
    super_helpers: Vec<SuperHelper>,
}

struct SuperHelper {
    binding: BoundIdentifier,
    initializer: Expr,
}

pub(crate) struct AsyncToGenerator {
    ignore_function_length: bool,
    function: Option<FunctionState>,
    function_stack: Vec<Option<FunctionState>>,
}

impl AsyncToGenerator {
    pub(crate) fn new(ignore_function_length: bool) -> Self {
        Self {
            ignore_function_length,
            function: None,
            function_stack: Vec::new(),
        }
    }

    pub(crate) fn enter_function(&mut self, function: Function, ctx: &mut TransformCtx<'_, '_>) {
        let info = ctx
            .take_async_function_info(function.node_id())
            .filter(|_| function.r#async(ctx.ast()) && !function.generator(ctx.ast()));
        self.function_stack.push(self.function.take());
        self.function = Some(FunctionState {
            should_transform: info.is_some(),
            use_this: info.is_some_and(|info| info.uses_this),
            use_arguments: info.is_some_and(|info| info.uses_arguments),
            has_parameter_expressions: info.is_some_and(|info| info.has_parameter_expressions),
            ..FunctionState::default()
        });
    }

    pub(crate) fn exit_function(&mut self, function: Function, ctx: &mut TransformCtx<'_, '_>) {
        let mut state = self.function.take().expect("matching function state");
        self.function = self.function_stack.pop().expect("function state stack");
        if !state.should_transform {
            prepend_super_helpers(function.body(ctx.ast()), state.super_helpers, ctx);
            return;
        }

        let original_params = function.params(ctx.ast());
        let wrapper_scope = ctx.current_scope();
        state.use_arguments |= could_potentially_throw(original_params, ctx);
        state.use_arguments |= !state.super_helpers.is_empty();
        let inner_params = if state.use_arguments {
            let outer_params = if self.ignore_function_length {
                empty_parameters(ctx)
            } else {
                placeholder_parameters(original_params, ctx)
            };
            function.set_params(ctx.ast, outer_params);
            original_params.set_kind(ctx.ast, FormalParameterKind::FormalParameters);
            original_params
        } else {
            empty_parameters(ctx)
        };
        let body = function.body(ctx.ast());
        let wrapper_directives = strict_directives(body, ctx);
        let invocation = make_async_generator_invocation(&state, inner_params, body, ctx);
        let statement = Stmt::ReturnStatement(
            ctx.add_node(|ast| ast.return_statement(DUMMY_SP, Some(invocation))),
        );
        let mut statements = Vec::with_capacity(2);
        if let Some(helpers) = super_helpers_declaration(state.super_helpers, wrapper_scope, ctx) {
            statements.push(helpers);
        }
        statements.push(statement);
        let statements = ctx.ast.add_sub_range(statements);
        let wrapper_body =
            ctx.add_node(|ast| ast.function_body(DUMMY_SP, wrapper_directives, statements));

        function.set_async(ctx.ast, false);
        function.set_generator(ctx.ast, false);
        function.set_body(ctx.ast, wrapper_body);
        if state.has_parameter_expressions && !state.use_arguments {
            create_wrapper_body_scope(wrapper_scope, wrapper_body, ctx);
        }
    }

    pub(crate) fn enter_arrow_function_expression(
        &mut self,
        arrow: ArrowFunctionExpression,
        ctx: &mut TransformCtx<'_, '_>,
    ) {
        let info = ctx
            .take_async_function_info(arrow.node_id())
            .filter(|_| arrow.r#async(ctx.ast()));
        if info.is_none() {
            return;
        }
        self.function_stack.push(self.function.take());
        self.function = Some(FunctionState {
            should_transform: true,
            use_this: info.is_some_and(|info| info.uses_this),
            use_arguments: info.is_some_and(|info| info.uses_arguments),
            has_parameter_expressions: info.is_some_and(|info| info.has_parameter_expressions),
            ..FunctionState::default()
        });
    }

    pub(crate) fn exit_arrow_function_expression(
        &mut self,
        arrow: ArrowFunctionExpression,
        ctx: &mut TransformCtx<'_, '_>,
    ) {
        if !arrow.r#async(ctx.ast()) {
            return;
        }
        let mut state = self.function.take().expect("matching async arrow state");
        let mut parent = self.function_stack.pop().expect("async arrow state stack");
        if let Some(parent) = &mut parent {
            parent.super_helpers.append(&mut state.super_helpers);
        }
        self.function = parent;

        let wrapper_scope = ctx.current_scope();
        let arrow_body = arrow.body(ctx.ast());
        let body = if let Some(body) = arrow_body.as_function_body(ctx.ast()) {
            body
        } else {
            let expression = arrow_body
                .as_expr(ctx.ast())
                .expect("arrow body must be a function body or expression");
            let statement = Stmt::ReturnStatement(
                ctx.add_node(|ast| ast.return_statement(DUMMY_SP, Some(expression))),
            );
            let statements = ctx.ast.add_sub_range([statement]);
            ctx.add_node(|ast| ast.function_body(DUMMY_SP, SubRange::empty(), statements))
        };
        let params = empty_parameters(ctx);
        let invocation = make_async_generator_invocation(&state, params, body, ctx);

        arrow.set_async(ctx.ast, false);
        if let Some(helpers) = super_helpers_declaration(state.super_helpers, wrapper_scope, ctx) {
            let ret = Stmt::ReturnStatement(
                ctx.add_node(|ast| ast.return_statement(DUMMY_SP, Some(invocation))),
            );
            let statements = ctx.ast.add_sub_range([helpers, ret]);
            let body =
                ctx.add_node(|ast| ast.function_body(DUMMY_SP, SubRange::empty(), statements));
            arrow.set_expression(ctx.ast, false);
            arrow.set_body(ctx.ast, ArrowFunctionBody::FunctionBody(body));
        } else {
            arrow.set_expression(ctx.ast, true);
            arrow.set_body(ctx.ast, ArrowFunctionBody::Expr(invocation));
        }
        if state.has_parameter_expressions
            && let Some(body) = arrow.body(ctx.ast()).as_function_body(ctx.ast())
        {
            create_wrapper_body_scope(wrapper_scope, body, ctx);
        }
    }

    pub(crate) fn exit_expr<'a>(
        &mut self,
        expression: Expr,
        ctx: &mut TransformCtx<'_, 'a>,
    ) -> TraverseAction<Expr> {
        let Some(state) = &mut self.function else {
            return TraverseAction::Continue;
        };
        if !state.should_transform {
            return TraverseAction::Continue;
        }

        if let Some(call) = expression.as_call_expression(ctx.ast())
            && let Some(member) = call.callee(ctx.ast()).as_member_expression(ctx.ast())
            && member.object(ctx.ast()).is_super(ctx.ast())
        {
            state.use_this = true;
            let getter = create_super_getter(member, state, ctx);
            let mut arguments = Vec::with_capacity(call.arguments(ctx.ast()).len() + 1);
            let this = Expr::ThisExpression(ctx.add_node(|ast| ast.this_expression(DUMMY_SP)));
            arguments.push(Argument::Expr(this));
            arguments.extend(
                call.arguments(ctx.ast())
                    .iter()
                    .map(|argument| ctx.ast.get_node_in_sub_range(argument)),
            );
            let replacement = call_member_vec(getter, "call", arguments, ctx);
            return TraverseAction::ReplaceSkip(replacement);
        }
        if let Some(assignment) = expression.as_assignment_expression(ctx.ast())
            && let Some(member) = assignment
                .left(ctx.ast())
                .as_simple_assignment_target(ctx.ast())
                .and_then(|target| target.as_member_expression(ctx.ast()))
            && member.object(ctx.ast()).is_super(ctx.ast())
        {
            let operator = assignment.operator(ctx.ast());
            let replacement = if operator == AssignmentOperator::Assign {
                create_super_setter(member, assignment.right(ctx.ast()), None, state, ctx)
            } else {
                create_super_compound_assignment(
                    member,
                    assignment.right(ctx.ast()),
                    operator,
                    state,
                    ctx,
                )
            };
            return TraverseAction::ReplaceSkip(replacement);
        }
        if let Some(update) = expression.as_update_expression(ctx.ast())
            && let Some(member) = update.argument(ctx.ast()).as_member_expression(ctx.ast())
            && member.object(ctx.ast()).is_super(ctx.ast())
        {
            let replacement = create_super_update(
                member,
                update.operator(ctx.ast()),
                update.prefix(ctx.ast()),
                state,
                ctx,
            );
            return TraverseAction::ReplaceSkip(replacement);
        }
        if let Some(member) = expression.as_member_expression(ctx.ast())
            && member.object(ctx.ast()).is_super(ctx.ast())
        {
            if super_member_is_parent_operand(member, ctx) {
                return TraverseAction::Continue;
            }
            let replacement = create_super_getter(member, state, ctx);
            return TraverseAction::ReplaceSkip(replacement);
        }
        let Some(await_expression) = expression.as_await_expression(ctx.ast()) else {
            return TraverseAction::Continue;
        };
        let span = await_expression.span(ctx.ast());
        let argument = await_expression.argument(ctx.ast());
        let yield_expression = Expr::YieldExpression(
            ctx.add_node(|ast| ast.yield_expression(span, Some(argument), false)),
        );
        let yield_expression = ctx.inherits_comments(yield_expression, await_expression);
        TraverseAction::ReplaceSkip(yield_expression)
    }
}

fn make_async_generator_invocation(
    state: &FunctionState,
    params: FormalParameters,
    body: FunctionBody,
    ctx: &mut TransformCtx<'_, '_>,
) -> Expr {
    let wrapper_scope = ctx.current_scope();
    let generator = ctx.add_node(|ast| {
        ast.function(
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
        )
    });
    let wrapper_flags = ctx.semantic().scope(wrapper_scope).flags;
    let generator_scope = ctx.create_child_scope(
        wrapper_scope,
        generator.node_id(),
        ScopeKind::Function,
        ScopeFlags {
            arguments_barrier: true,
            ..wrapper_flags
        },
    );
    ctx.move_subtree_to_scope(params, wrapper_scope, generator_scope);
    ctx.move_subtree_to_scope(body, wrapper_scope, generator_scope);
    let wrapped = ctx.call_async_to_generator(Expr::Function(generator), wrapper_scope);

    if state.use_arguments {
        let this = Expr::ThisExpression(ctx.add_node(|ast| ast.this_expression(DUMMY_SP)));
        let name = ctx.ast.add_utf8("arguments");
        let arguments = ctx.add_node(|ast| ast.identifier_reference(DUMMY_SP, name));
        let target = ctx.semantic().binding(wrapper_scope, b"arguments");
        ctx.register_reference(wrapper_scope, arguments, target, ReferenceFlags::default());
        return call_member(
            wrapped,
            "apply",
            [
                Argument::Expr(this),
                Argument::Expr(Expr::IdentifierReference(arguments)),
            ],
            ctx,
        );
    }
    if state.use_this {
        let this = Expr::ThisExpression(ctx.add_node(|ast| ast.this_expression(DUMMY_SP)));
        return call_member(wrapped, "call", [Argument::Expr(this)], ctx);
    }
    let call =
        ctx.add_node(|ast| ast.call_expression(DUMMY_SP, wrapped, None, SubRange::empty(), false));
    Expr::CallExpression(call)
}

fn create_wrapper_body_scope(
    wrapper_scope: swc_next_ecma_ast::ScopeId,
    body: FunctionBody,
    ctx: &mut TransformCtx<'_, '_>,
) {
    debug_assert!(ctx.semantic().scope(wrapper_scope).function_body.is_none());
    let flags = ctx.semantic().scope(wrapper_scope).flags;
    let body_scope = ctx.create_child_scope(
        wrapper_scope,
        body.node_id(),
        ScopeKind::FunctionBody,
        flags,
    );
    ctx.move_subtree_to_scope(body, wrapper_scope, body_scope);
}

fn super_member_is_parent_operand(member: MemberExpression, ctx: &TransformCtx<'_, '_>) -> bool {
    let Some(parent) = ctx.parent() else {
        return false;
    };
    match ctx.ast().kind_data(parent) {
        NodeKindData::CallExpression(call) => call.callee(ctx.ast()).node_id() == member.node_id(),
        NodeKindData::AssignmentExpression(assignment) => assignment
            .left(ctx.ast())
            .as_simple_assignment_target(ctx.ast())
            .and_then(|target| target.as_member_expression(ctx.ast()))
            .is_some_and(|target| target.node_id() == member.node_id()),
        NodeKindData::UpdateExpression(update) => update
            .argument(ctx.ast())
            .as_member_expression(ctx.ast())
            .is_some_and(|target| target.node_id() == member.node_id()),
        _ => false,
    }
}

fn create_super_getter(
    member: MemberExpression,
    state: &mut FunctionState,
    ctx: &mut TransformCtx<'_, '_>,
) -> Expr {
    create_super_getter_with_argument(member, None, state, ctx)
}

fn create_super_getter_with_argument(
    member: MemberExpression,
    computed_argument: Option<Expr>,
    state: &mut FunctionState,
    ctx: &mut TransformCtx<'_, '_>,
) -> Expr {
    let scope = ctx.current_scope();
    let (base, params, body, arguments) = if member.computed(ctx.ast()) {
        let property = member
            .property(ctx.ast())
            .as_expr(ctx.ast())
            .expect("computed super property expression");
        let parameter = BoundIdentifier::named(
            ctx,
            scope,
            "_prop",
            SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
        );
        let params = parameters_from_bindings(&[parameter], ctx);
        let property_reference =
            Expr::IdentifierReference(parameter.reference_read(ctx, scope, DUMMY_SP));
        let super_expression = Expr::Super(ctx.add_node(|ast| ast.super_(DUMMY_SP)));
        let span = member.span(ctx.ast());
        let body = Expr::MemberExpression(ctx.add_node(|ast| {
            ast.member_expression(
                span,
                super_expression,
                PropertyKey::Expr(property_reference),
                true,
                false,
            )
        }));
        (
            "_superprop_get".to_owned(),
            params,
            body,
            vec![Argument::Expr(computed_argument.unwrap_or(property))],
        )
    } else {
        let name = member
            .property(ctx.ast())
            .as_identifier_name(ctx.ast())
            .map(|identifier| ctx.ast().get_utf8(identifier.name(ctx.ast())))
            .unwrap_or("property");
        (
            format!("_superprop_get_{name}"),
            empty_parameters(ctx),
            clone_static_super_member(member, ctx),
            Vec::new(),
        )
    };
    let initializer = super_helper_arrow(params, body, ctx);
    let binding = BoundIdentifier::named(ctx, scope, &base, SymbolFlags::FUNCTION_SCOPED_VAR);
    let callee = Expr::IdentifierReference(binding.reference_read(ctx, scope, DUMMY_SP));
    state.super_helpers.push(SuperHelper {
        binding,
        initializer,
    });
    call_expression(callee, arguments, ctx)
}

fn create_super_setter(
    member: MemberExpression,
    value: Expr,
    computed_argument: Option<Expr>,
    state: &mut FunctionState,
    ctx: &mut TransformCtx<'_, '_>,
) -> Expr {
    let scope = ctx.current_scope();
    let value_parameter = BoundIdentifier::named(
        ctx,
        scope,
        "_value",
        SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
    );
    let (base, parameters, target_member, arguments) = if member.computed(ctx.ast()) {
        let property = member
            .property(ctx.ast())
            .as_expr(ctx.ast())
            .expect("computed super property expression");
        let property_parameter = BoundIdentifier::named(
            ctx,
            scope,
            "_prop",
            SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
        );
        let parameters = parameters_from_bindings(&[property_parameter, value_parameter], ctx);
        let property_reference =
            Expr::IdentifierReference(property_parameter.reference_read(ctx, scope, DUMMY_SP));
        let super_expression = Expr::Super(ctx.add_node(|ast| ast.super_(DUMMY_SP)));
        let span = member.span(ctx.ast());
        let target = ctx.add_node(|ast| {
            ast.member_expression(
                span,
                super_expression,
                PropertyKey::Expr(property_reference),
                true,
                false,
            )
        });
        (
            "_superprop_set".to_owned(),
            parameters,
            target,
            vec![
                Argument::Expr(computed_argument.unwrap_or(property)),
                Argument::Expr(value),
            ],
        )
    } else {
        let name = member
            .property(ctx.ast())
            .as_identifier_name(ctx.ast())
            .map(|identifier| ctx.ast().get_utf8(identifier.name(ctx.ast())))
            .unwrap_or("property");
        (
            format!("_superprop_set_{name}"),
            parameters_from_bindings(&[value_parameter], ctx),
            clone_static_super_member_handle(member, ctx),
            vec![Argument::Expr(value)],
        )
    };
    let value_reference =
        Expr::IdentifierReference(value_parameter.reference_read(ctx, scope, DUMMY_SP));
    let target = AssignmentTarget::SimpleAssignmentTarget(
        SimpleAssignmentTarget::MemberExpression(target_member),
    );
    let assignment = Expr::AssignmentExpression(ctx.add_node(|ast| {
        ast.assignment_expression(
            DUMMY_SP,
            target,
            value_reference,
            AssignmentOperator::Assign,
        )
    }));
    let initializer = super_helper_arrow(parameters, assignment, ctx);
    let binding = BoundIdentifier::named(ctx, scope, &base, SymbolFlags::FUNCTION_SCOPED_VAR);
    let callee = Expr::IdentifierReference(binding.reference_read(ctx, scope, DUMMY_SP));
    state.super_helpers.push(SuperHelper {
        binding,
        initializer,
    });
    call_expression(callee, arguments, ctx)
}

fn create_super_compound_assignment(
    member: MemberExpression,
    right: Expr,
    operator: AssignmentOperator,
    state: &mut FunctionState,
    ctx: &mut TransformCtx<'_, '_>,
) -> Expr {
    let scope = ctx.current_scope();
    let key = member.computed(ctx.ast()).then(|| {
        let property = member
            .property(ctx.ast())
            .as_expr(ctx.ast())
            .expect("computed super property expression");
        let key = ctx.generate_var(scope, "_key");
        let assignment = key.assign(ctx, scope, member.span(ctx.ast()), property);
        (key, assignment)
    });
    let getter_argument = key.as_ref().map(|(_, assignment)| *assignment);
    let getter = create_super_getter_with_argument(member, getter_argument, state, ctx);
    let setter_argument = key
        .as_ref()
        .map(|(key, _)| Expr::IdentifierReference(key.reference_read(ctx, scope, DUMMY_SP)));

    if let Some(operator) = assignment_binary_operator(operator) {
        let value = Expr::BinaryExpression(
            ctx.add_node(|ast| ast.binary_expression(DUMMY_SP, getter, right, operator)),
        );
        return create_super_setter(member, value, setter_argument, state, ctx);
    }

    let operator = assignment_logical_operator(operator)
        .expect("compound assignment must be binary or logical");
    let current = ctx.generate_var(scope, "_value");
    let current_assignment = current.assign(ctx, scope, DUMMY_SP, getter);
    let current_reference = Expr::IdentifierReference(current.reference_read(ctx, scope, DUMMY_SP));
    let setter = create_super_setter(member, right, setter_argument, state, ctx);
    let logical = Expr::LogicalExpression(
        ctx.add_node(|ast| ast.logical_expression(DUMMY_SP, current_reference, setter, operator)),
    );
    let expressions = ctx.ast.add_sub_range([current_assignment, logical]);
    Expr::SequenceExpression(ctx.add_node(|ast| ast.sequence_expression(DUMMY_SP, expressions)))
}

fn create_super_update(
    member: MemberExpression,
    operator: UpdateOperator,
    prefix: bool,
    state: &mut FunctionState,
    ctx: &mut TransformCtx<'_, '_>,
) -> Expr {
    let scope = ctx.current_scope();
    let (base, params, target_member, arguments) = if member.computed(ctx.ast()) {
        let property = member
            .property(ctx.ast())
            .as_expr(ctx.ast())
            .expect("computed super property expression");
        let property_parameter = BoundIdentifier::named(
            ctx,
            scope,
            "_prop",
            SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
        );
        let params = parameters_from_bindings(&[property_parameter], ctx);
        let property_reference =
            Expr::IdentifierReference(property_parameter.reference_read(ctx, scope, DUMMY_SP));
        let super_expression = Expr::Super(ctx.add_node(|ast| ast.super_(DUMMY_SP)));
        let span = member.span(ctx.ast());
        let target = ctx.add_node(|ast| {
            ast.member_expression(
                span,
                super_expression,
                PropertyKey::Expr(property_reference),
                true,
                false,
            )
        });
        (
            "_superprop_update".to_owned(),
            params,
            target,
            vec![Argument::Expr(property)],
        )
    } else {
        let name = member
            .property(ctx.ast())
            .as_identifier_name(ctx.ast())
            .map(|identifier| ctx.ast().get_utf8(identifier.name(ctx.ast())))
            .unwrap_or("property");
        (
            format!("_superprop_update_{name}"),
            empty_parameters(ctx),
            clone_static_super_member_handle(member, ctx),
            Vec::new(),
        )
    };
    let target = SimpleAssignmentTarget::MemberExpression(target_member);
    let update = Expr::UpdateExpression(
        ctx.add_node(|ast| ast.update_expression(DUMMY_SP, target, operator, prefix)),
    );
    let initializer = super_helper_arrow(params, update, ctx);
    let binding = BoundIdentifier::named(ctx, scope, &base, SymbolFlags::FUNCTION_SCOPED_VAR);
    let callee = Expr::IdentifierReference(binding.reference_read(ctx, scope, DUMMY_SP));
    state.super_helpers.push(SuperHelper {
        binding,
        initializer,
    });
    call_expression(callee, arguments, ctx)
}

fn clone_static_super_member(member: MemberExpression, ctx: &mut TransformCtx<'_, '_>) -> Expr {
    Expr::MemberExpression(clone_static_super_member_handle(member, ctx))
}

fn clone_static_super_member_handle(
    member: MemberExpression,
    ctx: &mut TransformCtx<'_, '_>,
) -> MemberExpression {
    let property = member
        .property(ctx.ast())
        .as_identifier_name(ctx.ast())
        .expect("noncomputed super property identifier");
    let name = property.name(ctx.ast());
    let property =
        PropertyKey::IdentifierName(ctx.add_node(|ast| ast.identifier_name(DUMMY_SP, name)));
    let super_expression = Expr::Super(ctx.add_node(|ast| ast.super_(DUMMY_SP)));
    let span = member.span(ctx.ast());
    ctx.add_node(|ast| ast.member_expression(span, super_expression, property, false, false))
}

fn assignment_binary_operator(operator: AssignmentOperator) -> Option<BinaryOperator> {
    Some(match operator {
        AssignmentOperator::AddAssign => BinaryOperator::Add,
        AssignmentOperator::SubtractAssign => BinaryOperator::Subtract,
        AssignmentOperator::MultiplyAssign => BinaryOperator::Multiply,
        AssignmentOperator::DivideAssign => BinaryOperator::Divide,
        AssignmentOperator::ModuloAssign => BinaryOperator::Modulo,
        AssignmentOperator::ExponentAssign => BinaryOperator::Exponent,
        AssignmentOperator::LeftShiftAssign => BinaryOperator::LeftShift,
        AssignmentOperator::RightShiftAssign => BinaryOperator::RightShift,
        AssignmentOperator::UnsignedRightShiftAssign => BinaryOperator::UnsignedRightShift,
        AssignmentOperator::BitwiseOrAssign => BinaryOperator::BitwiseOr,
        AssignmentOperator::BitwiseXorAssign => BinaryOperator::BitwiseXor,
        AssignmentOperator::BitwiseAndAssign => BinaryOperator::BitwiseAnd,
        _ => return None,
    })
}

fn assignment_logical_operator(operator: AssignmentOperator) -> Option<LogicalOperator> {
    Some(match operator {
        AssignmentOperator::LogicalOrAssign => LogicalOperator::Or,
        AssignmentOperator::LogicalAndAssign => LogicalOperator::And,
        AssignmentOperator::NullishAssign => LogicalOperator::NullishCoalescing,
        _ => return None,
    })
}

fn super_helper_arrow(
    params: FormalParameters,
    body: Expr,
    ctx: &mut TransformCtx<'_, '_>,
) -> Expr {
    let parent = ctx.current_scope();
    let arrow = ctx.add_node(|ast| {
        ast.arrow_function_expression(
            DUMMY_SP,
            true,
            false,
            None,
            params,
            None,
            ArrowFunctionBody::Expr(body),
        )
    });
    let flags = ctx.semantic().scope(parent).flags;
    let scope = ctx.create_child_scope(
        parent,
        arrow.node_id(),
        ScopeKind::Function,
        ScopeFlags {
            arguments_barrier: false,
            ..flags
        },
    );
    ctx.move_subtree_to_scope(params, parent, scope);
    ctx.move_subtree_to_scope(body, parent, scope);
    Expr::ArrowFunctionExpression(arrow)
}

fn parameters_from_bindings(
    bindings: &[BoundIdentifier],
    ctx: &mut TransformCtx<'_, '_>,
) -> FormalParameters {
    let parameters: Vec<_> = bindings
        .iter()
        .map(|binding| {
            FormalParameterItem::FormalParameter(ctx.add_node(|ast| {
                ast.formal_parameter(
                    DUMMY_SP,
                    SubRange::empty(),
                    FormalParameterPattern::BindingIdentifier(binding.identifier),
                    None,
                    None,
                    false,
                )
            }))
        })
        .collect();
    let parameters = ctx.ast.add_sub_range(parameters);
    ctx.add_node(|ast| {
        ast.formal_parameters(
            DUMMY_SP,
            parameters,
            None,
            FormalParameterKind::ArrowFormalParameters,
        )
    })
}

fn call_expression(callee: Expr, arguments: Vec<Argument>, ctx: &mut TransformCtx<'_, '_>) -> Expr {
    let arguments = ctx.ast.add_sub_range(arguments);
    Expr::CallExpression(
        ctx.add_node(|ast| ast.call_expression(DUMMY_SP, callee, None, arguments, false)),
    )
}

fn super_helpers_declaration(
    helpers: Vec<SuperHelper>,
    destination: swc_next_ecma_ast::ScopeId,
    ctx: &mut TransformCtx<'_, '_>,
) -> Option<Stmt> {
    if helpers.is_empty() {
        return None;
    }
    let declarators: Vec<_> = helpers
        .into_iter()
        .map(|helper| {
            let source = ctx.semantic().scope_of_symbol(helper.binding.symbol);
            if source != destination {
                ctx.move_binding_to_scope(helper.binding.identifier, source, destination);
            }
            let arrow = helper
                .initializer
                .as_arrow_function_expression(ctx.ast())
                .expect("super helper initializer is an arrow");
            let scope = ctx
                .semantic()
                .scope_of(arrow.node_id())
                .expect("super helper arrow scope");
            if ctx.semantic().scope(scope).parent != Some(destination) {
                ctx.reparent_scope(scope, destination);
            }
            ctx.add_node(|ast| {
                ast.variable_declarator(
                    DUMMY_SP,
                    BindingPattern::BindingIdentifier(helper.binding.identifier),
                    None,
                    Some(helper.initializer),
                    false,
                )
            })
        })
        .collect();
    let declarators = ctx.ast.add_sub_range(declarators);
    Some(Stmt::Declaration(
        swc_next_ecma_ast::Decl::VariableDeclaration(ctx.add_node(|ast| {
            ast.variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false)
        })),
    ))
}

fn prepend_super_helpers(
    body: FunctionBody,
    helpers: Vec<SuperHelper>,
    ctx: &mut TransformCtx<'_, '_>,
) {
    let function_scope = ctx.current_scope();
    let destination = ctx
        .semantic()
        .scope(function_scope)
        .function_body
        .unwrap_or(function_scope);
    let Some(declaration) = super_helpers_declaration(helpers, destination, ctx) else {
        return;
    };
    let mut statements = Vec::with_capacity(body.body(ctx.ast()).len() + 1);
    statements.push(declaration);
    statements.extend(
        body.body(ctx.ast())
            .iter()
            .map(|statement| ctx.ast.get_node_in_sub_range(statement)),
    );
    let statements = ctx.ast.add_sub_range(statements);
    body.set_body(ctx.ast, statements);
}

fn strict_directives(
    body: FunctionBody,
    ctx: &mut TransformCtx<'_, '_>,
) -> SubRange<swc_next_ecma_ast::Directive> {
    let Some(directive) = body.directives(ctx.ast()).iter().find_map(|directive| {
        let directive = ctx.ast.get_node_in_sub_range(directive);
        (ctx.ast().get_utf8(directive.value(ctx.ast())) == "use strict").then_some(directive)
    }) else {
        return SubRange::empty();
    };
    let expression = directive.expression(ctx.ast());
    let expression = ctx
        .add_node(|ast| ast.string_literal(DUMMY_SP, expression.value(ast), expression.raw(ast)));
    let directive = ctx.add_node(|ast| ast.directive(DUMMY_SP, expression, directive.value(ast)));
    ctx.ast.add_sub_range([directive])
}

fn call_member<const N: usize>(
    object: Expr,
    property: &str,
    arguments: [Argument; N],
    ctx: &mut TransformCtx<'_, '_>,
) -> Expr {
    let name = ctx.ast.add_utf8(property);
    let property =
        PropertyKey::IdentifierName(ctx.add_node(|ast| ast.identifier_name(DUMMY_SP, name)));
    let callee = Expr::MemberExpression(
        ctx.add_node(|ast| ast.member_expression(DUMMY_SP, object, property, false, false)),
    );
    let arguments = ctx.ast.add_sub_range(arguments);
    Expr::CallExpression(
        ctx.add_node(|ast| ast.call_expression(DUMMY_SP, callee, None, arguments, false)),
    )
}

fn call_member_vec(
    object: Expr,
    property: &str,
    arguments: Vec<Argument>,
    ctx: &mut TransformCtx<'_, '_>,
) -> Expr {
    let name = ctx.ast.add_utf8(property);
    let property =
        PropertyKey::IdentifierName(ctx.add_node(|ast| ast.identifier_name(DUMMY_SP, name)));
    let callee = Expr::MemberExpression(
        ctx.add_node(|ast| ast.member_expression(DUMMY_SP, object, property, false, false)),
    );
    call_expression(callee, arguments, ctx)
}

fn empty_parameters(ctx: &mut TransformCtx<'_, '_>) -> FormalParameters {
    ctx.add_node(|ast| {
        ast.formal_parameters(
            DUMMY_SP,
            SubRange::empty(),
            None,
            FormalParameterKind::FormalParameters,
        )
    })
}

fn placeholder_parameters(
    params: FormalParameters,
    ctx: &mut TransformCtx<'_, '_>,
) -> FormalParameters {
    let scope = ctx.current_scope();
    let mut placeholders = Vec::new();
    for (index, item) in params.items(ctx.ast()).iter().enumerate() {
        let item = ctx.ast.get_node_in_sub_range(item);
        let Some(parameter) = item.as_formal_parameter(ctx.ast()) else {
            break;
        };
        if parameter.initializer(ctx.ast()).is_some() {
            break;
        }
        let binding = BoundIdentifier::named(
            ctx,
            scope,
            &format!("_{index}"),
            SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
        );
        let parameter = ctx.add_node(|ast| {
            ast.formal_parameter(
                DUMMY_SP,
                SubRange::empty(),
                FormalParameterPattern::BindingIdentifier(binding.identifier),
                None,
                None,
                false,
            )
        });
        placeholders.push(FormalParameterItem::FormalParameter(parameter));
    }
    let placeholders = ctx.ast.add_sub_range(placeholders);
    ctx.add_node(|ast| ast.formal_parameters(DUMMY_SP, placeholders, None, params.kind(ast)))
}

fn could_potentially_throw(params: FormalParameters, ctx: &TransformCtx<'_, '_>) -> bool {
    let ast = ctx.ast();
    for item in params.items(ast).iter() {
        let item = ast.get_node_in_sub_range(item);
        let Some(parameter) = item.as_formal_parameter(ast) else {
            return true;
        };
        if parameter.pattern(ast).as_binding_identifier(ast).is_none() {
            return true;
        }
        let Some(initializer) = parameter.initializer(ast) else {
            continue;
        };
        if initializer.is_null_literal(ast)
            || initializer.is_boolean_literal(ast)
            || initializer.is_numeric_literal(ast)
            || initializer.is_big_int_literal(ast)
            || initializer.is_string_literal(ast)
            || initializer.is_function(ast)
            || initializer.is_arrow_function_expression(ast)
        {
            continue;
        }
        if let Some(identifier) = initializer.as_identifier_reference(ast)
            && ast.get_utf8(identifier.name(ast)) == "undefined"
            && ctx
                .semantic()
                .reference_of(identifier.node_id())
                .is_some_and(|reference| ctx.semantic().reference(reference).symbol.is_none())
        {
            continue;
        }
        return true;
    }
    params
        .rest(ast)
        .is_some_and(|rest| rest.argument(ast).as_binding_identifier(ast).is_none())
}
