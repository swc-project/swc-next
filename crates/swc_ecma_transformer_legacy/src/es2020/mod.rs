//! ES2020 transform options.

pub(crate) mod nullish_coalescing;

pub use crate::es2022::optional_chaining_impl::Config as OptionalChainingOptions;
pub use nullish_coalescing::Config as NullishCoalescingOptions;

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

pub(crate) mod export_namespace_from;
