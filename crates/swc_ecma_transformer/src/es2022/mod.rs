mod static_blocks;

use swc_next_ecma_ast::{Class, ClassBody};

use crate::{TransformFeatureFlags, context::Mutation};

pub struct ES2022 {
    features: TransformFeatureFlags,
    static_blocks: static_blocks::StaticBlocks,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ES2022Options {
    pub static_blocks: bool,
}

impl ES2022 {
    pub(crate) fn is_empty(&self) -> bool {
        self.static_blocks.is_empty()
    }

    pub(crate) fn new(features: TransformFeatureFlags) -> Self {
        Self {
            features,
            static_blocks: static_blocks::StaticBlocks::new(features),
        }
    }

    pub(crate) fn enter_class_body<'a>(&mut self, body: ClassBody, ctx: &mut Mutation<'_, '_, 'a>) {
        if self
            .features
            .contains(TransformFeatureFlags::ES2022_STATIC_BLOCKS)
        {
            self.static_blocks.enter_class_body(body, ctx);
        }
    }

    pub(crate) fn exit_class<'a>(&mut self, class: Class, ctx: &mut Mutation<'_, '_, 'a>) {
        if self
            .features
            .contains(TransformFeatureFlags::ES2022_STATIC_BLOCKS)
        {
            self.static_blocks.exit_class(class, ctx);
        }
    }
}
