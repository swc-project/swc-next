//! ES2018 transform options. RegExp features are configured separately.

pub(crate) mod object_rest_spread;

pub use object_rest_spread::Config as ObjectRestSpreadOptions;

/// Selects ES2018 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2018Options {
    /// Lower object rest/spread. `None` disables the pass.
    pub object_rest_spread: Option<ObjectRestSpreadOptions>,
}
