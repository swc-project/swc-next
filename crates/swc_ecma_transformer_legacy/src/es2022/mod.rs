//! ES2022 transform options. RegExp features are configured separately.

pub(crate) mod class_fields_use_set;
pub(crate) mod class_properties;
pub use class_fields_use_set::ClassFieldsUseSetOptions;

pub use class_properties::Config as ClassPropertiesOptions;

/// Selects ES2022 transforms. All transforms are disabled by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ES2022Options {
    /// Convert class-field initialization from define to assignment semantics.
    pub class_fields_use_set: Option<ClassFieldsUseSetOptions>,
    /// Lower class static blocks before class-property lowering.
    pub static_blocks: bool,
    /// Lower public/private class fields and private methods/accessors.
    /// `None` disables the pass.
    pub class_properties: Option<ClassPropertiesOptions>,
    /// Lower private-brand checks (`#field in object`).
    pub private_in_object: bool,
}

pub(crate) mod class_static_block;

pub(crate) mod private_property_in_object;

pub(crate) mod optional_chaining_impl;
