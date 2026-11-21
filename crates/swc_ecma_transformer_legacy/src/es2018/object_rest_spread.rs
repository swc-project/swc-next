/// SWC's `object_rest_spread::Config`. All flags default to `false`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ObjectRestSpreadOptions {
    /// Exclude symbol keys when copying object-rest properties.
    pub no_symbol: bool,
    /// Assign spread properties rather than defining them.
    pub set_property: bool,
    /// Assume property getters have no side effects.
    pub pure_getters: bool,
}
