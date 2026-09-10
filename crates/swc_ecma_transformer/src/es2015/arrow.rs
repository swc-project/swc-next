use serde::Deserialize;
use swc_next_ecma_ast::{
    ArrowFunctionExpression, AstUtf8, CallExpression, DUMMY_SP, Expr, Function, GetSpan,
    IdentifierReference, MetaProperty, NodeId, Program, StaticBlock, ThisExpression,
};

use crate::{context::Mutation, es2015::util::fn_env_hoister::FnEnvHoister};

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ArrowFunctionsOptions {
    pub spec: bool,
}

struct ArrowState {
    old_in_arrow: bool,
    wraps_property_initializer: bool,
    property_initializer_owner: Option<NodeId>,
    preserve_lexical_super: bool,
    static_block_environment: bool,
}

/// Compile ES2015 arrow functions to function expressions.
pub(super) struct ArrowFunctions {
    _options: ArrowFunctionsOptions,
    hoister: FnEnvHoister,
    arrow_state_stack: Vec<ArrowState>,
    function_state_stack: Vec<(bool, bool)>,
    static_block_state_stack: Vec<bool>,
    constructor_captures: rustc_hash::FxHashMap<NodeId, AstUtf8>,
}

impl ArrowFunctions {
    pub fn new(options: ArrowFunctionsOptions) -> Self {
        Self {
            _options: options,
            hoister: FnEnvHoister::new(),
            arrow_state_stack: Vec::new(),
            function_state_stack: Vec::new(),
            static_block_state_stack: Vec::new(),
            constructor_captures: Default::default(),
        }
    }

    fn convert_arrow(arrow: ArrowFunctionExpression, ctx: &mut Mutation<'_, '_, '_>) -> Function {
        use swc_next_ecma_ast::{
            DUMMY_SP, FormalParameterKind, FunctionType, GetSpan, Stmt, SubRange,
        };
        let (span, params, body, is_async) = (
            arrow.span(ctx.ast()),
            arrow.params(ctx.ast()),
            arrow.body(ctx.ast()),
            arrow.r#async(ctx.ast()),
        );
        ctx.set_parameter_kind(params, FormalParameterKind::FormalParameters);
        let body = if let Some(body) = body.as_function_body(ctx.ast()) {
            body
        } else {
            let expression = body
                .as_expr(ctx.ast())
                .expect("arrow body is an expression or a function body");
            let statement = ctx.add_node(ctx.current_scope(), |ast| {
                ast.return_statement(DUMMY_SP, Some(expression))
            });
            let items = ctx.add_sub_range([Stmt::ReturnStatement(statement)]);
            ctx.add_node(ctx.current_scope(), |ast| {
                ast.function_body(DUMMY_SP, SubRange::empty(), items)
            })
        };
        let scope = ctx
            .scope_for_node(arrow.node_id())
            .expect("arrow scope exists");
        let (function, _) = ctx.add_scope_node(
            crate::context::ScopeOptions {
                parent: ctx
                    .semantic()
                    .scope(scope)
                    .parent
                    .expect("arrow has a parent scope"),
                kind: swc_next_ecma_semantic::scope::ScopeKind::Function,
                reuse: Some(scope),
                environment: None,
            },
            |ast| {
                ast.function(
                    span,
                    FunctionType::FunctionExpression,
                    None,
                    None,
                    None,
                    params,
                    None,
                    body,
                    false,
                    is_async,
                )
            },
        );
        function
    }

    pub fn exit_program(&mut self, node: Program, ctx: &mut Mutation<'_, '_, '_>) {
        let frame = self.hoister.take_frame();
        self.hoister
            .enqueue_captures(node.node_id(), frame, false, ctx);
        debug_assert!(self.arrow_state_stack.is_empty());
        debug_assert!(self.function_state_stack.is_empty());
        debug_assert!(self.static_block_state_stack.is_empty());
    }

    pub fn enter_function(&mut self, node: Function, ctx: &mut Mutation<'_, '_, '_>) {
        let derived_constructor = ctx
            .semantic()
            .class()
            .is_derived_constructor(node.node_id());
        let old_in_arrow = self.hoister.set_in_arrow(false);
        self.function_state_stack
            .push((old_in_arrow, derived_constructor));
        self.hoister.push_frame();
        if ctx
            .semantic()
            .class()
            .constructor_needs_this_capture(node.node_id())
        {
            let name = self.hoister.capture_this(ctx);
            self.constructor_captures.insert(node.node_id(), name);
        }
    }

    pub fn exit_function(&mut self, node: Function, ctx: &mut Mutation<'_, '_, '_>) {
        let frame = self.hoister.take_frame();
        let (old_in_arrow, derived_constructor) = self
            .function_state_stack
            .pop()
            .expect("a function traversal state exists");
        let body = node.body(ctx.ast());
        self.constructor_captures.remove(&node.node_id());
        self.hoister
            .enqueue_captures(body.node_id(), frame, derived_constructor, ctx);
        self.hoister.set_in_arrow(old_in_arrow);
    }

    pub fn exit_call_expression(&mut self, call: CallExpression, ctx: &mut Mutation<'_, '_, '_>) {
        let Some(owner) = ctx
            .semantic()
            .class()
            .super_call_constructor(call.node_id())
        else {
            return;
        };
        let Some(&name) = self.constructor_captures.get(&owner) else {
            return;
        };
        let this = Expr::ThisExpression(
            ctx.add_node(ctx.current_scope(), |ast| ast.this_expression(DUMMY_SP)),
        );
        let assignment = ctx.assign_generated(name, this, DUMMY_SP, ctx.current_scope());
        let span = call.span(ctx.ast());
        let items = ctx.add_sub_range([Expr::CallExpression(call), assignment]);
        let expression = Expr::SequenceExpression(ctx.add_node(ctx.current_scope(), |ast| {
            ast.sequence_expression(span, items)
        }));
        ctx.wrap_current_expr(expression);
    }

    pub fn enter_static_block(&mut self) {
        let old_in_arrow = self.hoister.set_in_arrow(false);
        self.static_block_state_stack.push(old_in_arrow);
        self.hoister.push_frame();
    }

    pub fn exit_static_block(&mut self, node: StaticBlock, ctx: &mut Mutation<'_, '_, '_>) {
        let frame = self.hoister.take_frame();
        let old_in_arrow = self
            .static_block_state_stack
            .pop()
            .expect("a static block traversal state exists");
        self.hoister
            .enqueue_captures(node.node_id(), frame, false, ctx);
        self.hoister.set_in_arrow(old_in_arrow);
    }

    pub fn enter_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        property_initializer_owner: Option<NodeId>,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        let class = ctx.semantic().class();
        let old_in_arrow = self.hoister.set_in_arrow(true);
        let inherited_property_initializer_owner = old_in_arrow
            .then(|| {
                self.arrow_state_stack
                    .last()
                    .and_then(|state| state.property_initializer_owner)
            })
            .flatten();
        let wraps_property_initializer = property_initializer_owner.is_some()
            && property_initializer_owner != inherited_property_initializer_owner;
        if wraps_property_initializer {
            self.hoister.push_property_frame();
        }
        let static_block_environment = class.is_static_block_environment_arrow(node.node_id())
            || (old_in_arrow
                && self
                    .arrow_state_stack
                    .last()
                    .is_some_and(|state| state.static_block_environment));
        self.arrow_state_stack.push(ArrowState {
            old_in_arrow,
            wraps_property_initializer,
            property_initializer_owner: property_initializer_owner
                .or(inherited_property_initializer_owner),
            preserve_lexical_super: class.arrow_preserves_lexical_super(node.node_id()),
            static_block_environment,
        });
    }

    pub fn exit_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        let state = self
            .arrow_state_stack
            .pop()
            .expect("an arrow traversal state exists");
        self.hoister.set_in_arrow(state.old_in_arrow);

        if state.preserve_lexical_super {
            if state.wraps_property_initializer {
                let frame = self.hoister.take_frame();
                let replacement = self.hoister.wrap_generated_arrow(
                    Expr::ArrowFunctionExpression(node),
                    frame,
                    ctx,
                );
                ctx.wrap_current_expr(replacement);
            }
            return;
        }

        let function = Self::convert_arrow(node, ctx);
        let mut replacement = Expr::Function(function);
        if state.wraps_property_initializer {
            let frame = self.hoister.take_frame();
            replacement = self.hoister.wrap_generated_arrow(replacement, frame, ctx);
        }
        ctx.replace_current_expr_preserving_children(replacement);
    }

    pub fn exit_this_expression(&mut self, node: ThisExpression, ctx: &mut Mutation<'_, '_, '_>) {
        if let Some(replacement) = self.hoister.transform_this_expression(node, ctx) {
            ctx.replace_current_expr(replacement);
        }
    }

    pub fn enter_identifier_reference(
        &mut self,
        node: IdentifierReference,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        self.hoister.visit_identifier_reference(node, ctx);
    }

    pub fn exit_meta_property(&mut self, node: MetaProperty, ctx: &mut Mutation<'_, '_, '_>) {
        if let Some(replacement) = self.hoister.transform_meta_property(
            node,
            self.arrow_state_stack
                .last()
                .is_some_and(|state| state.static_block_environment),
            ctx,
        ) {
            ctx.replace_current_expr(replacement);
        }
    }
}
