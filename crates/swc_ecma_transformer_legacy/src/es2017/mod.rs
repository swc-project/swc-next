//! ES2017 transform options.

mod async_to_generator;

pub use async_to_generator::AsyncToGeneratorOptions;

/// Selects ES2017 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2017Options {
    /// Lower async functions and async generators, matching SWC's standalone
    /// `async_to_generator` pass. `None` disables the pass.
    pub async_to_generator: Option<AsyncToGeneratorOptions>,
}
