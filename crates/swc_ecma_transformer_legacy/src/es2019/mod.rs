//! ES2019 transform options.

/// Selects ES2019 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2019Options {
    /// Add a binding to catch clauses that omit one.
    pub optional_catch_binding: bool,
}

pub(crate) mod optional_catch_binding;
