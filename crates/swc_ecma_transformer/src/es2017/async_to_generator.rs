//! ES2017 async-function lowering.
//!
//! The output shape follows SWC's async-to-generator transform: reuse the
//! original function and its scope as the generator, then place it in a new
//! wrapper function.
//! The traversal state keeps `await`, `this`, and `arguments` attached to the
//! nearest ordinary function while allowing nested arrows to inherit them.

use swc_next_ecma_ast::{
    Argument, ArrowFunctionBody, ArrowFunctionExpression, AssignmentOperator, AssignmentTarget,
    BinaryOperator, BindingPattern, DUMMY_SP, Expr, FormalParameterItem, FormalParameterKind,
    FormalParameterPattern, FormalParameters, Function, FunctionBody, FunctionType, GetSpan,
    LogicalOperator, MemberExpression, NodeId, NodeKindData, ObjectPropertyKind, PropertyKey,
    PropertyKind, ReferenceId, ScopeId, SimpleAssignmentTarget, Span, Stmt, SubRange,
    TraverseAction, UnaryOperator, UpdateOperator, VariableKind,
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
    wrapper: Option<Function>,
    wrapper_scope: Option<ScopeId>,
    helper_scope: Option<ScopeId>,
    arrow_wrapper: Option<ArrowFunctionExpression>,
    source_scope: Option<ScopeId>,
    super_helpers: Vec<WrapperHelper>,
    lexical_helpers: Vec<WrapperHelper>,
}

struct WrapperHelper {
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
        let (wrapper, wrapper_scope) = info.map_or((None, None), |_| {
            let (wrapper, scope) = prepare_function_wrapper(function, ctx);
            (Some(wrapper), Some(scope))
        });
        self.function = Some(FunctionState {
            should_transform: info.is_some(),
            use_this: info.is_some_and(|info| info.uses_this),
            use_arguments: info.is_some_and(|info| info.uses_arguments),
            wrapper,
            wrapper_scope,
            helper_scope: wrapper_scope,
            source_scope: wrapper_scope.map(|_| ctx.current_scope()),
            ..FunctionState::default()
        });
    }

    pub(crate) fn exit_function(
        &mut self,
        function: Function,
        ctx: &mut TransformCtx<'_, '_>,
    ) -> TraverseAction<Function> {
        let state = self.function.take().expect("matching function state");
        self.function = self.function_stack.pop().expect("function state stack");
        if !state.should_transform {
            return TraverseAction::Continue;
        }

        let original_params = function.params(ctx.ast());
        let wrapper = state.wrapper.expect("prepared async wrapper");
        let wrapper_scope = state.wrapper_scope.expect("prepared async wrapper scope");
        let wrapper_params = if self.ignore_function_length {
            empty_parameters_with_kind(original_params.kind(ctx.ast()), ctx)
        } else {
            placeholder_parameters(original_params, wrapper_scope, ctx)
        };
        wrapper.set_params(ctx.ast, wrapper_params);

        let body = function.body(ctx.ast());
        let wrapper_directives = strict_directives(body, ctx);
        if function.function_type(ctx.ast()) == FunctionType::FunctionExpression {
            ctx.transfer_function_expression_name(function, wrapper);
        }
        configure_generator(function, ctx);

        let wrapped = ctx.call_async_to_generator(Expr::Function(function), wrapper_scope);
        let forwards_arguments = !original_params.items(ctx.ast()).is_empty()
            || original_params.rest(ctx.ast()).is_some()
            || state.use_arguments;
        let invocation = if forwards_arguments {
            call_with_this_and_arguments(wrapped, wrapper_scope, ctx)
        } else if state.use_this {
            let this = Expr::ThisExpression(ctx.add_node(|ast| ast.this_expression(DUMMY_SP)));
            call_member(wrapped, "call", [Argument::Expr(this)], ctx)
        } else {
            Expr::CallExpression(ctx.add_node(|ast| {
                ast.call_expression(DUMMY_SP, wrapped, None, SubRange::empty(), false)
            }))
        };
        let wrapper_body = wrapper.body(ctx.ast());
        wrapper_body.set_directives(ctx.ast, wrapper_directives);
        write_wrapper_body(
            wrapper_body,
            invocation,
            state.super_helpers,
            wrapper_scope,
            ctx,
        );
        TraverseAction::ReplaceSkip(wrapper)
    }

    pub(crate) fn enter_arrow_function_expression(
        &mut self,
        arrow: ArrowFunctionExpression,
        ctx: &mut TransformCtx<'_, '_>,
    ) {
        let Some(info) = ctx
            .take_async_function_info(arrow.node_id())
            .filter(|_| arrow.r#async(ctx.ast()))
        else {
            return;
        };
        let arrow_scope = ctx.current_scope();
        let original = ctx.semantic().scope(arrow_scope);
        let parent_scope = original.parent.expect("arrow scope parent");
        let flags = ScopeFlags {
            strict: ctx.semantic().scope(parent_scope).flags.strict,
            ..Default::default()
        };
        let wrapper = create_empty_arrow(arrow.span(ctx.ast()), ctx);
        let wrapper_scope = insert_wrapper_scope(wrapper.node_id(), flags, ctx);
        let parent = self.function.take();
        let helper_scope = parent
            .as_ref()
            .and_then(|state| {
                let scope = state.source_scope?;
                let owner = ctx.semantic().scope(scope).node;
                let mut path = ctx.ancestors();
                if crosses_class_initializer(owner, &mut path, &mut None, ctx) {
                    return None;
                }
                state.helper_scope
            })
            .unwrap_or(wrapper_scope);
        self.function_stack.push(parent);
        self.function = Some(FunctionState {
            should_transform: true,
            use_this: info.uses_this,
            has_parameter_expressions: info.has_parameter_expressions,
            arrow_wrapper: Some(wrapper),
            source_scope: Some(arrow_scope),
            wrapper_scope: Some(wrapper_scope),
            helper_scope: Some(helper_scope),
            ..FunctionState::default()
        });
    }

    pub(crate) fn exit_arrow_function_expression(
        &mut self,
        arrow: ArrowFunctionExpression,
        ctx: &mut TransformCtx<'_, '_>,
    ) -> TraverseAction<ArrowFunctionExpression> {
        if !arrow.r#async(ctx.ast()) {
            return TraverseAction::Continue;
        }
        let mut state = self.function.take().expect("matching async arrow state");
        let mut parent = self.function_stack.pop().expect("async arrow state stack");
        if let Some(parent) = &mut parent
            && state.helper_scope != state.wrapper_scope
        {
            parent.use_this |= state.use_this;
            parent.super_helpers.append(&mut state.super_helpers);
        }
        self.function = parent;

        let wrapper = state.arrow_wrapper.expect("prepared arrow wrapper");
        let wrapper_scope = state.wrapper_scope.expect("prepared wrapper scope");
        let generator_scope = ctx.current_scope();
        let params = arrow.params(ctx.ast());
        let span = arrow.span(ctx.ast());
        wrapper.set_type_parameters(ctx.ast, arrow.type_parameters(ctx.ast()));
        wrapper.set_return_type(ctx.ast, arrow.return_type(ctx.ast()));
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
        ctx.inject_statements_to(body.node_id());
        if state.has_parameter_expressions
            && ctx
                .semantic()
                .scope(generator_scope)
                .function_body
                .is_none()
        {
            // A concise arrow did not own a separate body scope. Its generated
            // block needs one now that parameter expressions stay in the generator.
            let flags = ctx.semantic().scope(generator_scope).flags;
            let body_scope = ctx.create_child_scope(
                generator_scope,
                body.node_id(),
                ScopeKind::FunctionBody,
                flags,
            );
            ctx.move_subtree_to_scope(body, generator_scope, body_scope);
        }
        initialize_arrow_arguments_var(body, generator_scope, ctx);
        let forwarded = arrow_wrapper_parameters(
            wrapper,
            params,
            wrapper_scope,
            self.ignore_function_length,
            ctx,
        );
        let generator = ctx.ast.function(
            span,
            FunctionType::FunctionExpression,
            None,
            None,
            None,
            params,
            None,
            body,
            true,
            false,
        );
        // SAFETY: The generator is a detached final allocation. The typed arrow
        // hook returns a replacement immediately, so only expression edges or
        // the new Function handle will observe the changed kind.
        let generator = unsafe { ctx.ast.replace_data(arrow, generator) };
        configure_generator(generator, ctx);
        let flags = ctx.semantic().scope(generator_scope).flags;
        ctx.convert_scope(
            generator_scope,
            generator.node_id(),
            ScopeKind::Function,
            ScopeFlags {
                arguments_barrier: true,
                ..flags
            },
        );
        let wrapped = ctx.call_async_to_generator(Expr::Function(generator), wrapper_scope);
        let invocation = call_expression(wrapped, forwarded, ctx);

        state.super_helpers.append(&mut state.lexical_helpers);
        if state.super_helpers.is_empty() {
            wrapper.set_expression(ctx.ast, true);
            wrapper.set_body(ctx.ast, ArrowFunctionBody::Expr(invocation));
        } else {
            let body = wrapper
                .body(ctx.ast())
                .as_function_body(ctx.ast())
                .expect("prepared wrapper body");
            write_wrapper_body(body, invocation, state.super_helpers, wrapper_scope, ctx);
        }
        TraverseAction::ReplaceSkip(wrapper)
    }

    pub(crate) fn exit_expr<'a>(
        &mut self,
        expression: Expr,
        ctx: &mut TransformCtx<'_, 'a>,
    ) -> TraverseAction<Expr> {
        if let Some(replacement) = self.capture_lexical_value(expression, ctx) {
            return TraverseAction::ReplaceSkip(replacement);
        }
        let Some(state) = &mut self.function else {
            return TraverseAction::Continue;
        };
        if !state.should_transform {
            return TraverseAction::Continue;
        }

        if let Some(call) = expression.as_call_expression(ctx.ast())
            && let Some(member) = call.callee(ctx.ast()).as_member_expression(ctx.ast())
            && member.object(ctx.ast()).is_super(ctx.ast())
            && state.owns_lexical_super(ctx)
        {
            state.use_this = true;
            let getter = create_super_getter(member, state, ctx);
            let mut arguments = Vec::with_capacity(call.arguments(ctx.ast()).len() + 1);
            let this = Expr::ThisExpression(ctx.add_node(|ast| ast.this_expression(DUMMY_SP)));
            let receiver = self.capture_lexical_value(this, ctx).unwrap_or(this);
            arguments.push(Argument::Expr(receiver));
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
            && state.owns_lexical_super(ctx)
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
            && state.owns_lexical_super(ctx)
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
            && state.owns_lexical_super(ctx)
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

    fn capture_lexical_value(
        &mut self,
        expression: Expr,
        ctx: &mut TransformCtx<'_, '_>,
    ) -> Option<Expr> {
        self.function.as_ref()?.arrow_wrapper?;
        let ast = ctx.ast();
        let (base, reference) = match ast.kind_data(expression.node_id()) {
            NodeKindData::ThisExpression(_) => ("_this", None),
            NodeKindData::MetaProperty(meta) if ast.get_utf8(meta.meta(ast).name(ast)) == "new" => {
                ("_newtarget", None)
            }
            NodeKindData::IdentifierReference(identifier)
                if ast.get_utf8(identifier.name(ast)) == "arguments" =>
            {
                // Moving the whole unary expression preserves typeof on an
                // unbound name and delete on a lexical binding.
                if let Some(parent) = ctx.parent()
                    && let NodeKindData::UnaryExpression(unary) = ast.kind_data(parent)
                    && matches!(
                        unary.operator(ast),
                        UnaryOperator::Typeof | UnaryOperator::Delete
                    )
                {
                    return None;
                }
                (
                    "_arguments",
                    Some(ctx.semantic().reference_of(identifier.node_id())?),
                )
            }
            NodeKindData::UnaryExpression(unary)
                if matches!(
                    unary.operator(ast),
                    UnaryOperator::Typeof | UnaryOperator::Delete
                ) =>
            {
                let identifier = unary.argument(ast).as_identifier_reference(ast)?;
                if ast.get_utf8(identifier.name(ast)) != "arguments" {
                    return None;
                }
                (
                    "_arguments",
                    Some(ctx.semantic().reference_of(identifier.node_id())?),
                )
            }
            _ => return None,
        };
        let owner = self.capture_owner(reference, ctx)?;
        let scope = owner.wrapper_scope.expect("lexical capture wrapper");
        let (getter, getter_scope) = create_helper_arrow(scope, ctx);
        let initializer = finish_helper_arrow(getter, getter.params(ctx.ast()), expression, ctx);
        if let Some(reference) = reference {
            ctx.move_reference_to_scope(reference, getter_scope);
        }
        let binding = add_lexical_helper(owner, scope, base, initializer, ctx);
        if let Some(parent) = ctx.parent()
            && let NodeKindData::ObjectProperty(property) = ctx.ast.kind_data(parent)
            && property.value(ctx.ast()).node_id() == expression.node_id()
            && property.shorthand(ctx.ast())
        {
            property.set_shorthand(ctx.ast, false);
        }
        let read =
            Expr::IdentifierReference(binding.reference_read(ctx, ctx.current_scope(), DUMMY_SP));
        Some(call_expression(read, Vec::new(), ctx))
    }

    pub(crate) fn exit_simple_assignment_target(
        &mut self,
        target: SimpleAssignmentTarget,
        ctx: &mut TransformCtx<'_, '_>,
    ) -> TraverseAction<SimpleAssignmentTarget> {
        let Some(identifier) = target.as_identifier_reference(ctx.ast()) else {
            return TraverseAction::Continue;
        };
        if ctx.ast.get_utf8(identifier.name(ctx.ast())) != "arguments" {
            return TraverseAction::Continue;
        }
        let Some(id) = ctx.semantic().reference_of(identifier.node_id()) else {
            return TraverseAction::Continue;
        };
        let reference = ctx.semantic().reference(id);
        if !reference.flags.is_write() {
            return TraverseAction::Continue;
        }
        let Some(owner) = self.capture_owner(Some(id), ctx) else {
            return TraverseAction::Continue;
        };
        let scope = owner.wrapper_scope.expect("lexical capture wrapper");

        // An accessor retains the original assignment/update syntax, including
        // destructuring, short-circuiting, and the value of a postfix update.
        // Its callbacks are arrows because a method has its own arguments.
        let (setter, setter_scope) = create_helper_arrow(scope, ctx);
        let value = BoundIdentifier::named(
            ctx,
            setter_scope,
            "_value",
            SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
        );
        let params = parameters_from_bindings(&[value], ctx);
        let value = Expr::IdentifierReference(value.reference_read(ctx, setter_scope, DUMMY_SP));
        let assignment = ctx.add_node(|ast| {
            ast.assignment_expression(
                DUMMY_SP,
                AssignmentTarget::SimpleAssignmentTarget(target),
                value,
                AssignmentOperator::Assign,
            )
        });
        let setter =
            finish_helper_arrow(setter, params, Expr::AssignmentExpression(assignment), ctx);
        ctx.move_reference_to_scope(id, setter_scope);
        ctx.set_reference_flags(id, reference.flags.write_only());
        let setter = add_lexical_helper(owner, scope, "_setArguments", setter, ctx);
        let mut properties = Vec::with_capacity(2);
        if reference.flags.is_read() {
            let (getter, getter_scope) = create_helper_arrow(scope, ctx);
            let identifier = ctx.add_node(|ast| ast.identifier_reference(DUMMY_SP, reference.name));
            ctx.register_reference(
                getter_scope,
                identifier,
                reference.symbol,
                ReferenceFlags::default(),
            );
            let getter = finish_helper_arrow(
                getter,
                getter.params(ctx.ast()),
                Expr::IdentifierReference(identifier),
                ctx,
            );
            let getter = add_lexical_helper(owner, scope, "_getArguments", getter, ctx);
            properties.push(arguments_accessor(scope, PropertyKind::Get, getter, ctx));
        }
        properties.push(arguments_accessor(scope, PropertyKind::Set, setter, ctx));
        let properties = ctx.ast.add_sub_range(properties);
        let object =
            Expr::ObjectExpression(ctx.add_node(|ast| ast.object_expression(DUMMY_SP, properties)));
        let binding = add_lexical_helper(owner, scope, "_argumentsRef", object, ctx);
        let object =
            Expr::IdentifierReference(binding.reference_read(ctx, ctx.current_scope(), DUMMY_SP));
        let name = ctx.ast.add_utf8("value");
        let key =
            PropertyKey::IdentifierName(ctx.add_node(|ast| ast.identifier_name(DUMMY_SP, name)));
        let member = ctx.add_node(|ast| ast.member_expression(DUMMY_SP, object, key, false, false));
        TraverseAction::ReplaceSkip(SimpleAssignmentTarget::MemberExpression(member))
    }

    /// Capture at the outermost converted arrow that the lexical value crosses.
    /// This also keeps a nested async arrow from seeing a generated arguments object.
    fn capture_owner(
        &mut self,
        reference: Option<ReferenceId>,
        ctx: &TransformCtx<'_, '_>,
    ) -> Option<&mut FunctionState> {
        let symbol_scope = reference
            .and_then(|id| ctx.semantic().reference(id).symbol)
            .map(|symbol| ctx.semantic().scope_of_symbol(symbol));
        let mut owner = None;
        let mut path = ctx.ancestors();
        let mut child = None;
        for state in std::iter::once(&mut self.function).chain(self.function_stack.iter_mut().rev())
        {
            let Some(state) = state else { break };
            if state.arrow_wrapper.is_none() {
                break;
            }
            let arrow_scope = state.source_scope.expect("original arrow scope");
            let node = ctx.semantic().scope(arrow_scope).node;
            if crosses_class_initializer(node, &mut path, &mut child, ctx) {
                return owner;
            }
            let mut scope = symbol_scope;
            while let Some(current) = scope {
                if current == arrow_scope {
                    return owner;
                }
                scope = ctx.semantic().scope(current).parent;
            }
            owner = Some(state);
        }
        owner
    }
}

fn crosses_class_initializer(
    owner: NodeId,
    path: &mut impl Iterator<Item = NodeId>,
    child: &mut Option<NodeId>,
    ctx: &TransformCtx<'_, '_>,
) -> bool {
    for ancestor in path {
        if ancestor == owner {
            *child = Some(owner);
            return false;
        }
        match ctx.ast.kind_data(ancestor) {
            NodeKindData::StaticBlock(_) => return true,
            NodeKindData::PropertyDefinition(field)
                if field.value(ctx.ast()).map(|value| value.node_id()) == *child =>
            {
                return true;
            }
            _ => *child = Some(ancestor),
        }
    }
    false
}

impl FunctionState {
    fn owns_lexical_super(&self, ctx: &TransformCtx<'_, '_>) -> bool {
        let scope = self.source_scope.expect("original async scope");
        let owner = ctx.semantic().scope(scope).node;
        !crosses_class_initializer(owner, &mut ctx.ancestors(), &mut None, ctx)
    }
}

fn prepare_function_wrapper(
    function: Function,
    ctx: &mut TransformCtx<'_, '_>,
) -> (Function, ScopeId) {
    let function_scope = ctx.current_scope();
    let flags = ctx.semantic().scope(function_scope).flags;
    let params = empty_parameters_with_kind(function.params(ctx.ast()).kind(ctx.ast()), ctx);
    let body =
        ctx.add_node(|ast| ast.function_body(DUMMY_SP, SubRange::empty(), SubRange::empty()));
    let function_type = function.function_type(ctx.ast());
    let declaration_name = if function_type == FunctionType::FunctionExpression {
        None
    } else {
        function.id(ctx.ast())
    };
    let wrapper = ctx.add_node(|ast| {
        ast.function(
            function.span(ast),
            function_type,
            declaration_name,
            function.type_parameters(ast),
            function.this_param(ast),
            params,
            function.return_type(ast),
            body,
            false,
            false,
        )
    });
    let wrapper_scope = insert_wrapper_scope(wrapper.node_id(), flags, ctx);
    (wrapper, wrapper_scope)
}

fn insert_wrapper_scope(
    owner: NodeId,
    flags: ScopeFlags,
    ctx: &mut TransformCtx<'_, '_>,
) -> ScopeId {
    let scope = ctx.current_scope();
    let parent = ctx
        .semantic()
        .scope(scope)
        .parent
        .expect("function scope parent");
    let wrapper = ctx.create_child_scope(parent, owner, ScopeKind::Function, flags);
    ctx.reparent_scope(scope, wrapper);
    wrapper
}

fn configure_generator(function: Function, ctx: &mut TransformCtx<'_, '_>) {
    function.set_async(ctx.ast, false);
    function.set_generator(ctx.ast, true);
    function.set_function_type(ctx.ast, FunctionType::FunctionExpression);
    function.set_id(ctx.ast, None);
    function.set_type_parameters(ctx.ast, None);
    function.set_this_param(ctx.ast, None);
    function.set_return_type(ctx.ast, None);
    function
        .params(ctx.ast)
        .set_kind(ctx.ast, FormalParameterKind::FormalParameters);
}

fn write_wrapper_body(
    body: FunctionBody,
    invocation: Expr,
    helpers: Vec<WrapperHelper>,
    scope: ScopeId,
    ctx: &mut TransformCtx<'_, '_>,
) {
    let helpers = wrapper_helpers_declaration(helpers, scope, ctx);
    let ret =
        Stmt::ReturnStatement(ctx.add_node(|ast| ast.return_statement(DUMMY_SP, Some(invocation))));
    let statements = ctx.ast.add_sub_range(helpers.into_iter().chain([ret]));
    body.set_body(ctx.ast, statements);
}

fn initialize_arrow_arguments_var(
    body: FunctionBody,
    function: ScopeId,
    ctx: &mut TransformCtx<'_, '_>,
) {
    let semantic = ctx.semantic();
    let scope = semantic.scope(function).function_body.unwrap_or(function);
    let Some(symbol) = semantic.binding(scope, b"arguments") else {
        return;
    };
    let flags = semantic.symbol(symbol).flags;
    if !flags.contains(SymbolFlags::FUNCTION_SCOPED_VAR)
        || flags.intersects(SymbolFlags::FUNCTION | SymbolFlags::PARAMETER)
        || semantic
            .binding(function, b"arguments")
            .is_some_and(|parameter| {
                semantic
                    .symbol(parameter)
                    .flags
                    .contains(SymbolFlags::PARAMETER)
            })
    {
        return;
    }
    // A sloppy arrow's local var starts as undefined. A generator initializes
    // the same spelling with its implicit arguments object.
    let name = ctx.ast.add_utf8("arguments");
    let identifier = ctx.add_node(|ast| ast.identifier_reference(DUMMY_SP, name));
    ctx.register_reference(
        scope,
        identifier,
        Some(symbol),
        ReferenceFlags::default().write_only(),
    );
    let zero = ctx.add_node(|ast| ast.numeric_literal(DUMMY_SP, 0.0, None));
    let undefined = Expr::UnaryExpression(ctx.add_node(|ast| {
        ast.unary_expression(DUMMY_SP, Expr::NumericLiteral(zero), UnaryOperator::Void)
    }));
    let assignment = Expr::AssignmentExpression(ctx.add_node(|ast| {
        ast.assignment_expression(
            DUMMY_SP,
            AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::IdentifierReference(
                identifier,
            )),
            undefined,
            AssignmentOperator::Assign,
        )
    }));
    let statement = Stmt::ExpressionStatement(
        ctx.add_node(|ast| ast.expression_statement(DUMMY_SP, assignment)),
    );
    let rest = body.body(ctx.ast());
    let statements = std::iter::once(statement)
        .chain(rest.iter().map(|id| ctx.ast.get_node_in_sub_range(id)))
        .collect::<Vec<_>>();
    let statements = ctx.ast.add_sub_range(statements);
    body.set_body(ctx.ast, statements);
}

fn arrow_wrapper_parameters(
    wrapper: ArrowFunctionExpression,
    original: FormalParameters,
    scope: ScopeId,
    ignore_function_length: bool,
    ctx: &mut TransformCtx<'_, '_>,
) -> Vec<Argument> {
    let count = original.items(ctx.ast()).len();
    if count == 0 && original.rest(ctx.ast()).is_none() {
        return Vec::new();
    }
    let params = if ignore_function_length {
        wrapper.params(ctx.ast())
    } else {
        placeholder_parameters(original, scope, ctx)
    };
    let items = params.items(ctx.ast());
    let mut forwarded = Vec::with_capacity(items.len() + 1);
    for item in items.iter() {
        let parameter = ctx
            .ast
            .get_node_in_sub_range(item)
            .as_formal_parameter(ctx.ast())
            .unwrap();
        let identifier = parameter
            .pattern(ctx.ast())
            .as_binding_identifier(ctx.ast())
            .unwrap();
        let symbol = ctx.semantic().symbol_of(identifier.node_id()).unwrap();
        let binding = BoundIdentifier { identifier, symbol };
        let reference = binding.reference_read(ctx, scope, DUMMY_SP);
        forwarded.push(Argument::Expr(Expr::IdentifierReference(reference)));
    }
    if items.len() < count || original.rest(ctx.ast()).is_some() {
        let binding = BoundIdentifier::named(
            ctx,
            scope,
            "_args",
            SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
        );
        let rest = ctx.add_node(|ast| {
            ast.formal_parameter_rest(
                DUMMY_SP,
                SubRange::empty(),
                FormalParameterPattern::BindingIdentifier(binding.identifier),
                None,
            )
        });
        params.set_rest(ctx.ast, Some(rest));
        let reference = Expr::IdentifierReference(binding.reference_read(ctx, scope, DUMMY_SP));
        forwarded.push(Argument::SpreadElement(
            ctx.add_node(|ast| ast.spread_element(DUMMY_SP, reference)),
        ));
    }
    wrapper.set_params(ctx.ast, params);
    forwarded
}

fn call_with_this_and_arguments(
    wrapped: Expr,
    wrapper_scope: ScopeId,
    ctx: &mut TransformCtx<'_, '_>,
) -> Expr {
    let this = Expr::ThisExpression(ctx.add_node(|ast| ast.this_expression(DUMMY_SP)));
    let name = ctx.ast.add_utf8("arguments");
    let arguments = ctx.add_node(|ast| ast.identifier_reference(DUMMY_SP, name));
    let target = ctx.semantic().binding(wrapper_scope, b"arguments");
    ctx.register_reference(wrapper_scope, arguments, target, ReferenceFlags::default());
    call_member(
        wrapped,
        "apply",
        [
            Argument::Expr(this),
            Argument::Expr(Expr::IdentifierReference(arguments)),
        ],
        ctx,
    )
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
    let helper_scope = state.helper_scope.expect("super helper destination scope");
    let (arrow, arrow_scope) = create_helper_arrow(helper_scope, ctx);
    let (base, params, body, arguments) = if member.computed(ctx.ast()) {
        let property = member
            .property(ctx.ast())
            .as_expr(ctx.ast())
            .expect("computed super property expression");
        let parameter = BoundIdentifier::named(
            ctx,
            arrow_scope,
            "_prop",
            SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
        );
        let params = parameters_from_bindings(&[parameter], ctx);
        let property_reference =
            Expr::IdentifierReference(parameter.reference_read(ctx, arrow_scope, DUMMY_SP));
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
    let initializer = finish_helper_arrow(arrow, params, body, ctx);
    let binding =
        BoundIdentifier::named(ctx, helper_scope, &base, SymbolFlags::FUNCTION_SCOPED_VAR);
    let callee = Expr::IdentifierReference(binding.reference_read(ctx, scope, DUMMY_SP));
    state.super_helpers.push(WrapperHelper {
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
    let helper_scope = state.helper_scope.expect("super helper destination scope");
    let (arrow, arrow_scope) = create_helper_arrow(helper_scope, ctx);
    let value_parameter = BoundIdentifier::named(
        ctx,
        arrow_scope,
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
            arrow_scope,
            "_prop",
            SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
        );
        let parameters = parameters_from_bindings(&[property_parameter, value_parameter], ctx);
        let property_reference = Expr::IdentifierReference(property_parameter.reference_read(
            ctx,
            arrow_scope,
            DUMMY_SP,
        ));
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
        Expr::IdentifierReference(value_parameter.reference_read(ctx, arrow_scope, DUMMY_SP));
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
    let initializer = finish_helper_arrow(arrow, parameters, assignment, ctx);
    let binding =
        BoundIdentifier::named(ctx, helper_scope, &base, SymbolFlags::FUNCTION_SCOPED_VAR);
    let callee = Expr::IdentifierReference(binding.reference_read(ctx, scope, DUMMY_SP));
    state.super_helpers.push(WrapperHelper {
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
    let helper_scope = state.helper_scope.expect("super helper destination scope");
    let (arrow, arrow_scope) = create_helper_arrow(helper_scope, ctx);
    let (base, params, target_member, arguments) = if member.computed(ctx.ast()) {
        let property = member
            .property(ctx.ast())
            .as_expr(ctx.ast())
            .expect("computed super property expression");
        let property_parameter = BoundIdentifier::named(
            ctx,
            arrow_scope,
            "_prop",
            SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
        );
        let params = parameters_from_bindings(&[property_parameter], ctx);
        let property_reference = Expr::IdentifierReference(property_parameter.reference_read(
            ctx,
            arrow_scope,
            DUMMY_SP,
        ));
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
    let initializer = finish_helper_arrow(arrow, params, update, ctx);
    let binding =
        BoundIdentifier::named(ctx, helper_scope, &base, SymbolFlags::FUNCTION_SCOPED_VAR);
    let callee = Expr::IdentifierReference(binding.reference_read(ctx, scope, DUMMY_SP));
    state.super_helpers.push(WrapperHelper {
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

fn create_helper_arrow(
    parent: ScopeId,
    ctx: &mut TransformCtx<'_, '_>,
) -> (ArrowFunctionExpression, ScopeId) {
    let arrow = create_empty_arrow(DUMMY_SP, ctx);
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
    (arrow, scope)
}

fn create_empty_arrow(span: Span, ctx: &mut TransformCtx<'_, '_>) -> ArrowFunctionExpression {
    let params = empty_parameters_with_kind(FormalParameterKind::ArrowFormalParameters, ctx);
    let body =
        ctx.add_node(|ast| ast.function_body(DUMMY_SP, SubRange::empty(), SubRange::empty()));
    ctx.add_node(|ast| {
        ast.arrow_function_expression(
            span,
            false,
            false,
            None,
            params,
            None,
            ArrowFunctionBody::FunctionBody(body),
        )
    })
}

fn finish_helper_arrow(
    arrow: ArrowFunctionExpression,
    params: FormalParameters,
    body: Expr,
    ctx: &mut TransformCtx<'_, '_>,
) -> Expr {
    arrow.set_expression(ctx.ast, true);
    arrow.set_params(ctx.ast, params);
    arrow.set_body(ctx.ast, ArrowFunctionBody::Expr(body));
    Expr::ArrowFunctionExpression(arrow)
}

fn add_lexical_helper(
    state: &mut FunctionState,
    scope: ScopeId,
    name: &str,
    initializer: Expr,
    ctx: &mut TransformCtx<'_, '_>,
) -> BoundIdentifier {
    let binding = BoundIdentifier::named(ctx, scope, name, SymbolFlags::FUNCTION_SCOPED_VAR);
    state.lexical_helpers.push(WrapperHelper {
        binding,
        initializer,
    });
    binding
}

fn arguments_accessor(
    parent: ScopeId,
    kind: PropertyKind,
    callback: BoundIdentifier,
    ctx: &mut TransformCtx<'_, '_>,
) -> ObjectPropertyKind {
    let body =
        ctx.add_node(|ast| ast.function_body(DUMMY_SP, SubRange::empty(), SubRange::empty()));
    let params = empty_parameters(ctx);
    let function = ctx.add_node(|ast| {
        ast.function(
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
        )
    });
    let flags = ScopeFlags {
        arguments_barrier: true,
        ..ctx.semantic().scope(parent).flags
    };
    let scope = ctx.create_child_scope(parent, function.node_id(), ScopeKind::Function, flags);
    let args = if kind == PropertyKind::Set {
        let value = BoundIdentifier::named(
            ctx,
            scope,
            "_value",
            SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
        );
        let params = parameters_from_bindings(&[value], ctx);
        params.set_kind(ctx.ast, FormalParameterKind::FormalParameters);
        function.set_params(ctx.ast, params);
        vec![Argument::Expr(Expr::IdentifierReference(
            value.reference_read(ctx, scope, DUMMY_SP),
        ))]
    } else {
        Vec::new()
    };
    let callee = Expr::IdentifierReference(callback.reference_read(ctx, scope, DUMMY_SP));
    let call = call_expression(callee, args, ctx);
    let statement =
        Stmt::ReturnStatement(ctx.add_node(|ast| ast.return_statement(DUMMY_SP, Some(call))));
    let statements = ctx.ast.add_sub_range([statement]);
    body.set_body(ctx.ast, statements);
    let name = ctx.ast.add_utf8("value");
    let key = PropertyKey::IdentifierName(ctx.add_node(|ast| ast.identifier_name(DUMMY_SP, name)));
    ObjectPropertyKind::ObjectProperty(ctx.add_node(|ast| {
        ast.object_property(
            DUMMY_SP,
            key,
            Expr::Function(function),
            kind,
            false,
            false,
            false,
        )
    }))
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

fn wrapper_helpers_declaration(
    helpers: Vec<WrapperHelper>,
    destination: ScopeId,
    ctx: &mut TransformCtx<'_, '_>,
) -> Option<Stmt> {
    if helpers.is_empty() {
        return None;
    }
    let declarators: Vec<_> = helpers
        .into_iter()
        .map(|helper| {
            debug_assert_eq!(
                ctx.semantic().scope_of_symbol(helper.binding.symbol),
                destination
            );
            if let Some(arrow) = helper.initializer.as_arrow_function_expression(ctx.ast()) {
                let scope = ctx
                    .semantic()
                    .scope_of(arrow.node_id())
                    .expect("helper arrow scope");
                debug_assert_eq!(ctx.semantic().scope(scope).parent, Some(destination));
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
    empty_parameters_with_kind(FormalParameterKind::FormalParameters, ctx)
}

fn empty_parameters_with_kind(
    kind: FormalParameterKind,
    ctx: &mut TransformCtx<'_, '_>,
) -> FormalParameters {
    ctx.add_node(|ast| ast.formal_parameters(DUMMY_SP, SubRange::empty(), None, kind))
}

fn placeholder_parameters(
    params: FormalParameters,
    scope: ScopeId,
    ctx: &mut TransformCtx<'_, '_>,
) -> FormalParameters {
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
