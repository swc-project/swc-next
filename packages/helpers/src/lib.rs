//! Runtime helper names exported by the `@swc-next/helpers` npm package.

/// Package entry point exposing helpers by their function names.
pub const MODULE: &str = "@swc-next/helpers";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Helper {
    ToPropertyKey,
    SetFunctionName,
}

impl Helper {
    pub const ALL: [Self; 2] = [Self::ToPropertyKey, Self::SetFunctionName];

    pub const fn name(self) -> &'static str {
        match self {
            Self::ToPropertyKey => "_to_property_key",
            Self::SetFunctionName => "_set_function_name",
        }
    }
}
