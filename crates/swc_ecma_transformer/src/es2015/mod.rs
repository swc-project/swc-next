use swc_experimental_ecma_ast::{ObjectProperty, Program, RegExpLiteral, TraverseCtx};
use swc_experimental_ecma_semantic::Semantic;

mod arrow;
mod shorthand;
mod sticky_regex;
pub(crate) mod util;

pub struct ES2015 {
    arrow: arrow::ArrowFunctions,
    shorthand: shorthand::Shorthand,
    sticky_regex: sticky_regex::StickyRegex,
}

#[derive(Default)]
pub struct ES2015Options {
    arrow_functions: arrow::ArrowFunctionsOptions,
}

impl ES2015 {
    pub fn new(options: ES2015Options) -> Self {
        Self {
            arrow: arrow::ArrowFunctions::new(options.arrow_functions),
            shorthand: shorthand::Shorthand,
            sticky_regex: sticky_regex::StickyRegex,
        }
    }
}

impl ES2015 {
    pub(crate) fn transform<'a>(
        &mut self,
        node: Program,
        semantic: &Semantic,
        ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.arrow.transform(node, ctx.ast_mut(), semantic);
    }

    pub(crate) fn exit_object_property(
        &mut self,
        node: ObjectProperty,
        ctx: &mut TraverseCtx<'_, '_>,
    ) {
        self.shorthand.exit_object_property(node, ctx);
    }

    pub(crate) fn exit_reg_exp_literal(
        &mut self,
        node: RegExpLiteral,
        ctx: &mut TraverseCtx<'_, '_>,
    ) {
        self.sticky_regex.exit_reg_exp_literal(node, ctx);
    }
}
