//! ES2022 transform options. RegExp features are configured separately.

use swc_next_ecma_ast::{Diagnostic, TraverseMutCtx, TraverseMutWith};

use crate::common::{TraverseCtx, var_declarations::VarDeclarations};

use self::{
    class_fields_use_set::class_fields_use_set, class_properties::class_properties,
    private_property_in_object::PrivatePropertyInObjectPass,
};

mod class_fields_use_set;
mod class_properties;
pub use class_fields_use_set::ClassFieldsUseSetOptions;

pub use class_properties::Config as ClassPropertiesOptions;

/// Selects ES2022 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2022Options {
    /// Convert class-field initialization from define to assignment semantics.
    pub class_fields_use_set: Option<ClassFieldsUseSetOptions>,
    /// Lower class static blocks before class-property lowering.
    pub static_blocks: bool,
    /// Lower public/private class fields and private methods/accessors.
    /// `None` disables the pass.
    pub class_properties: Option<ClassPropertiesOptions>,
    /// Lower private-brand checks (`#field in object`).
    pub private_in_object: bool,
}

mod class_static_block;

mod private_property_in_object;

pub(crate) mod optional_chaining_impl;

pub(crate) fn transform(
    ctx: &mut TraverseMutCtx<'_, '_, TraverseCtx<'_>>,
    options: ES2022Options,
) -> Vec<Diagnostic> {
    if let Some(config) = options.class_fields_use_set {
        class_fields_use_set(ctx.ast, ctx.state.resolver, config);
    }
    if options.static_blocks {
        class_static_block::pass(ctx.ast, &mut ctx.state.static_blocks);
    }
    let diagnostics = options
        .class_properties
        .map(|config| class_properties(ctx.ast, &mut ctx.state, config))
        .unwrap_or_default();
    if options.private_in_object {
        let program = ctx.ast.root_program();
        let _ = program.traverse_mut_with(
            &mut VarDeclarations {
                hook: PrivatePropertyInObjectPass::new(),
                scope_arrow_exprs: false,
            },
            ctx,
        );
    }
    diagnostics
}
