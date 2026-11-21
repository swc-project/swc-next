//! ES2016 transform options.

/// Selects ES2016 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2016Options {
    /// Lower `**` and `**=` using SWC's `exponentiation` pass.
    pub exponentiation: bool,
}
