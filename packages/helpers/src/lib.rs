//! Canonical helper sources, shared with the `@swc-next/helpers` npm package.

/// Package entry point exposing every helper by its function name.
pub const MODULE: &str = "@swc-next/helpers";

/// A runtime helper module. Inline consumers parse [`Self::code`] and resolve
/// its relative imports with [`Self::from_import`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Helper {
    TypeOf,
    ToPrimitive,
    ToPropertyKey,
    SetFunctionName,
    SetClassName,
}

impl Helper {
    pub const ALL: [Self; 5] = [
        Self::TypeOf,
        Self::ToPrimitive,
        Self::ToPropertyKey,
        Self::SetFunctionName,
        Self::SetClassName,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::TypeOf => "_type_of",
            Self::ToPrimitive => "_to_primitive",
            Self::ToPropertyKey => "_to_property_key",
            Self::SetFunctionName => "_set_function_name",
            Self::SetClassName => "_set_class_name",
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::TypeOf => include_str!("../esm/_type_of.js"),
            Self::ToPrimitive => include_str!("../esm/_to_primitive.js"),
            Self::ToPropertyKey => include_str!("../esm/_to_property_key.js"),
            Self::SetFunctionName => include_str!("../esm/_set_function_name.js"),
            Self::SetClassName => include_str!("../esm/_set_class_name.js"),
        }
    }

    pub fn from_import(source: &str) -> Option<Self> {
        let name = source.strip_prefix("./")?.strip_suffix(".js")?;
        Self::ALL.into_iter().find(|helper| helper.name() == name)
    }
}
