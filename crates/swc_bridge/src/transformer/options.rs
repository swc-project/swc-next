use crate::bridge;

/// Rust transforms applied between JavaScript plugin stages.
#[bridge(object)]
#[derive(Default)]
pub struct TransformPassOptions {
    /// Lower TypeScript syntax. Defaults to true for TypeScript input.
    /// JSX is preserved.
    pub typescript: Option<bool>,
    /// Lower class static blocks. Defaults to false.
    pub static_blocks: Option<bool>,
    /// Preserve value imports/exports and disable const-enum inlining and removal.
    /// Only applies when lowering TypeScript. Defaults to false.
    pub verbatim_module_syntax: Option<bool>,
    /// Preserve const-enum declarations while still allowing inlining.
    /// Only applies when lowering TypeScript. Defaults to false.
    pub preserve_const_enums: Option<bool>,
}
