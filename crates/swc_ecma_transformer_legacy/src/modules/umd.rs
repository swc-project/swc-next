use super::{
    ModuleType, module_ref_rewriter::rewrite_import_bindings, syntax_strip, util::ModuleCtx,
};

mod emit;

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/umd.rs#L36>.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/umd.rs#L77>.
pub(super) fn umd(cx: &mut ModuleCtx<'_, '_>, filename: Option<&str>) {
    let mut module = syntax_strip::lower(cx, ModuleType::Umd);
    rewrite_import_bindings(&mut module.body, cx, &module.import_map);
    emit::emit(cx, module, filename.unwrap_or("input.js"));
}
