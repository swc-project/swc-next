mod static_blocks;

use swc_next_ecma_ast::{Class, ClassBody, Expr, TraverseAction};

use crate::{context::TransformCtx, proposal::ProposalOptions};

pub struct ES2022 {
    static_blocks: Option<static_blocks::StaticBlocks>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ES2022Options {
    pub static_blocks: bool,
}

impl ES2022 {
    pub(crate) fn new(options: ES2022Options, proposal: ProposalOptions) -> Self {
        Self {
            static_blocks: options
                .static_blocks
                .then(|| static_blocks::StaticBlocks::new(proposal.prevent_extensions)),
        }
    }

    pub(crate) fn enter_class_body<'a>(&mut self, body: ClassBody, ctx: &mut TransformCtx<'_, 'a>) {
        if let Some(static_blocks) = &mut self.static_blocks {
            static_blocks.enter_class_body(body, ctx);
        }
    }

    pub(crate) fn exit_class<'a>(
        &mut self,
        class: Class,
        ctx: &mut TransformCtx<'_, 'a>,
    ) -> TraverseAction<Expr> {
        if let Some(static_blocks) = &mut self.static_blocks {
            return static_blocks.exit_class(class, ctx);
        }
        TraverseAction::Continue
    }
}
