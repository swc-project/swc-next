//! ES2022 transform options. RegExp features are configured separately.

mod class_properties;

pub use class_properties::ClassPropertiesOptions;

/// Selects ES2022 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2022Options {
    /// Lower class static blocks before class-property lowering.
    pub static_blocks: bool,
    /// Lower public/private class fields and private methods/accessors.
    /// `None` disables the pass.
    pub class_properties: Option<ClassPropertiesOptions>,
    /// Lower private-brand checks (`#field in object`).
    pub private_in_object: bool,
}
