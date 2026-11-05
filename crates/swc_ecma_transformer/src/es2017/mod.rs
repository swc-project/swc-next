mod async_to_generator;

use swc_next_ecma_ast::{
    ArrowFunctionExpression, Expr, Function, SimpleAssignmentTarget, TraverseAction,
};

use crate::context::TransformCtx;

#[derive(Debug, Default, Clone, Copy)]
pub struct ES2017Options {
    pub async_to_generator: bool,
}

pub(crate) struct ES2017 {
    async_to_generator: Option<async_to_generator::AsyncToGenerator>,
}

impl ES2017 {
    pub(crate) fn new(options: ES2017Options, ignore_function_length: bool) -> Self {
        Self {
            async_to_generator: options
                .async_to_generator
                .then(|| async_to_generator::AsyncToGenerator::new(ignore_function_length)),
        }
    }

    pub(crate) fn enter_function(&mut self, function: Function, ctx: &mut TransformCtx<'_, '_>) {
        if let Some(transform) = &mut self.async_to_generator {
            transform.enter_function(function, ctx);
        }
    }

    pub(crate) fn exit_function(
        &mut self,
        function: Function,
        ctx: &mut TransformCtx<'_, '_>,
    ) -> TraverseAction<Function> {
        self.async_to_generator
            .as_mut()
            .map_or(TraverseAction::Continue, |transform| {
                transform.exit_function(function, ctx)
            })
    }

    pub(crate) fn enter_arrow_function_expression(
        &mut self,
        arrow: ArrowFunctionExpression,
        ctx: &mut TransformCtx<'_, '_>,
    ) {
        if let Some(transform) = &mut self.async_to_generator {
            transform.enter_arrow_function_expression(arrow, ctx);
        }
    }

    pub(crate) fn exit_arrow_function_expression(
        &mut self,
        arrow: ArrowFunctionExpression,
        ctx: &mut TransformCtx<'_, '_>,
    ) -> TraverseAction<ArrowFunctionExpression> {
        self.async_to_generator
            .as_mut()
            .map_or(TraverseAction::Continue, |transform| {
                transform.exit_arrow_function_expression(arrow, ctx)
            })
    }

    pub(crate) fn exit_expr<'a>(
        &mut self,
        expression: Expr,
        ctx: &mut TransformCtx<'_, 'a>,
    ) -> TraverseAction<Expr> {
        self.async_to_generator
            .as_mut()
            .map_or(TraverseAction::Continue, |transform| {
                transform.exit_expr(expression, ctx)
            })
    }

    pub(crate) fn exit_simple_assignment_target(
        &mut self,
        node: SimpleAssignmentTarget,
        ctx: &mut TransformCtx<'_, '_>,
    ) -> TraverseAction<SimpleAssignmentTarget> {
        self.async_to_generator
            .as_mut()
            .map_or(TraverseAction::Continue, |transform| {
                transform.exit_simple_assignment_target(node, ctx)
            })
    }
}

#[cfg(test)]
mod tests;
