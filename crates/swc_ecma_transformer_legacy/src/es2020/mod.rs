//! ES2020 transform options.

mod nullish_coalescing;
mod optional_chaining;

pub use nullish_coalescing::NullishCoalescingOptions;
pub use optional_chaining::OptionalChainingOptions;

/// Selects ES2020 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2020Options {
    /// Lower `export * as namespace from "module"`.
    pub export_namespace_from: bool,
    /// Lower `??`. `None` disables the pass.
    pub nullish_coalescing: Option<NullishCoalescingOptions>,
    /// Lower optional chaining. `None` disables the pass.
    pub optional_chaining: Option<OptionalChainingOptions>,
}
