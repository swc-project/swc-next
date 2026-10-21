use crate::bridge;

/// Rust transforms applied between JavaScript plugin stages.
#[bridge(object)]
#[derive(Default)]
pub struct TransformPassOptions {
    /// Lower TypeScript syntax. Defaults to true for TypeScript input.
    /// Uses the transformer's default TypeScript options; JSX is preserved.
    pub typescript: Option<bool>,
    /// Lower class static blocks. Defaults to false.
    pub static_blocks: Option<bool>,
}
