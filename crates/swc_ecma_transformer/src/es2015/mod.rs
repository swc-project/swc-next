use swc_next_ecma_ast::{
    ArrowFunctionExpression, CallExpression, Function, IdentifierReference, MetaProperty, NodeId,
    ObjectProperty, Program, RegExpLiteral, StaticBlock, ThisExpression,
};

use crate::{TransformFeatureFlags, context::Mutation};

pub mod arrow;
pub mod shorthand;
pub mod sticky_regex;
pub(crate) mod util;

pub struct ES2015 {
    features: TransformFeatureFlags,
    arrow: arrow::ArrowFunctions,
}

#[derive(Default)]
pub struct ES2015Options {
    pub arrow_functions: Option<arrow::ArrowFunctionsOptions>,
    pub shorthand_properties: bool,
    pub sticky_regex: bool,
}

impl ES2015 {
    pub(crate) fn new(options: ES2015Options, features: TransformFeatureFlags) -> Self {
        Self {
            features,
            arrow: arrow::ArrowFunctions::new(options.arrow_functions.unwrap_or_default()),
        }
    }
}

impl ES2015 {
    pub(crate) fn exit_program(&mut self, node: Program, ctx: &mut Mutation<'_, '_, '_>) {
        if self
            .features
            .contains(TransformFeatureFlags::ARROW_FUNCTIONS)
        {
            self.arrow.exit_program(node, ctx);
        }
    }

    pub(crate) fn enter_function(&mut self, node: Function, ctx: &mut Mutation<'_, '_, '_>) {
        if self
            .features
            .contains(TransformFeatureFlags::ARROW_FUNCTIONS)
        {
            self.arrow.enter_function(node, ctx);
        }
    }

    pub(crate) fn exit_call_expression(
        &mut self,
        node: CallExpression,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        if self
            .features
            .contains(TransformFeatureFlags::ARROW_FUNCTIONS)
        {
            self.arrow.exit_call_expression(node, ctx);
        }
    }

    pub(crate) fn exit_function(&mut self, node: Function, ctx: &mut Mutation<'_, '_, '_>) {
        if self
            .features
            .contains(TransformFeatureFlags::ARROW_FUNCTIONS)
        {
            self.arrow.exit_function(node, ctx);
        }
    }

    pub(crate) fn enter_static_block(&mut self) {
        if self
            .features
            .contains(TransformFeatureFlags::ARROW_FUNCTIONS)
        {
            self.arrow.enter_static_block();
        }
    }

    pub(crate) fn exit_static_block(&mut self, node: StaticBlock, ctx: &mut Mutation<'_, '_, '_>) {
        if self
            .features
            .contains(TransformFeatureFlags::ARROW_FUNCTIONS)
        {
            self.arrow.exit_static_block(node, ctx);
        }
    }

    pub(crate) fn enter_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        property_initializer_owner: Option<NodeId>,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        if self
            .features
            .contains(TransformFeatureFlags::ARROW_FUNCTIONS)
        {
            self.arrow
                .enter_arrow_function_expression(node, property_initializer_owner, ctx);
        }
    }

    pub(crate) fn exit_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        if self
            .features
            .contains(TransformFeatureFlags::ARROW_FUNCTIONS)
        {
            self.arrow.exit_arrow_function_expression(node, ctx);
        }
    }

    pub(crate) fn exit_this_expression(
        &mut self,
        node: ThisExpression,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        if self
            .features
            .contains(TransformFeatureFlags::ARROW_FUNCTIONS)
        {
            self.arrow.exit_this_expression(node, ctx);
        }
    }

    pub(crate) fn enter_identifier_reference(
        &mut self,
        node: IdentifierReference,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        if self
            .features
            .contains(TransformFeatureFlags::ARROW_FUNCTIONS)
        {
            self.arrow.enter_identifier_reference(node, ctx);
        }
    }

    pub(crate) fn exit_meta_property(
        &mut self,
        node: MetaProperty,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        if self
            .features
            .contains(TransformFeatureFlags::ARROW_FUNCTIONS)
        {
            self.arrow.exit_meta_property(node, ctx);
        }
    }

    pub(crate) fn exit_object_property(
        &mut self,
        node: ObjectProperty,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        if self
            .features
            .contains(TransformFeatureFlags::SHORTHAND_PROPERTIES)
        {
            shorthand::Shorthand.exit_object_property(node, ctx);
        }
    }

    pub(crate) fn exit_reg_exp_literal(
        &mut self,
        node: RegExpLiteral,
        ctx: &mut Mutation<'_, '_, '_>,
    ) {
        if self.features.contains(TransformFeatureFlags::STICKY_REGEX) {
            sticky_regex::StickyRegex.exit_reg_exp_literal(node, ctx);
        }
    }
}
