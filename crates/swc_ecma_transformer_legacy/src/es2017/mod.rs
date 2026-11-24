//! ES2017 transform options.

pub(crate) mod async_to_generator;

pub use async_to_generator::AsyncToGeneratorOptions;

/// Selects ES2017 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2017Options {
    /// Lower async functions and async generators, matching SWC's standalone
    /// Lower ordinary async functions and async generators.
    pub async_to_generator: Option<AsyncToGeneratorOptions>,
    /// Lower async generators and `for await` while preserving ordinary async functions.
    /// If both passes are enabled, `async_to_generator` supplies their options.
    pub async_generator_functions: Option<AsyncToGeneratorOptions>,
}
