use swc_next_ecma_ast::{Ast, ObjectProperty, RegExpLiteral};
use swc_next_ecma_semantic::Semantic;

use crate::state::TransformerCtx;

pub mod arrow;
pub mod shorthand;
pub mod sticky_regex;
pub(crate) mod util;

pub struct ES2015 {
    semantic_program: Option<SemanticProgramTransforms>,
    traverse: TraverseTransforms,
}

pub(crate) struct SemanticProgramTransforms {
    arrow: arrow::ArrowFunctions,
}

pub(crate) struct TraverseTransforms {
    shorthand_properties: bool,
    sticky_regex: bool,
}

#[derive(Default)]
pub struct ES2015Options {
    pub arrow_functions: Option<arrow::ArrowFunctionsOptions>,
    pub shorthand_properties: bool,
    pub sticky_regex: bool,
}

impl ES2015 {
    pub fn new(options: ES2015Options) -> Self {
        let ES2015Options {
            arrow_functions,
            shorthand_properties,
            sticky_regex,
        } = options;
        Self {
            semantic_program: arrow_functions.map(|options| SemanticProgramTransforms {
                arrow: arrow::ArrowFunctions::new(options),
            }),
            traverse: TraverseTransforms {
                shorthand_properties,
                sticky_regex,
            },
        }
    }

    pub(crate) fn into_phases(self) -> (Option<SemanticProgramTransforms>, TraverseTransforms) {
        (self.semantic_program, self.traverse)
    }
}

impl SemanticProgramTransforms {
    pub(crate) fn transform<'a>(&mut self, ast: &mut Ast<'a>, semantic: &Semantic) {
        self.arrow.transform(ast, semantic);
    }
}

impl TraverseTransforms {
    pub(crate) fn is_enabled(&self) -> bool {
        self.shorthand_properties || self.sticky_regex
    }

    pub(crate) fn exit_object_property(
        &mut self,
        node: ObjectProperty,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.shorthand_properties {
            shorthand::Shorthand.exit_object_property(node, ctx);
        }
    }

    pub(crate) fn exit_reg_exp_literal(
        &mut self,
        node: RegExpLiteral,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.sticky_regex {
            sticky_regex::StickyRegex.exit_reg_exp_literal(node, ctx);
        }
    }
}
