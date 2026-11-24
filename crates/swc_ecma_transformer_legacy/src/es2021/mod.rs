//! ES2021 transform options.

/// Selects ES2021 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2021Options {
    /// Lower `&&=`, `||=`, and `??=` using SWC's `logical_assignments` pass.
    /// Nullish coalescing produced by `??=` is selected separately through
    /// [`ES2020Options::nullish_coalescing`](crate::es2020::ES2020Options::nullish_coalescing).
    pub logical_assignments: bool,
}

pub(crate) mod logical_assignment_operators;
