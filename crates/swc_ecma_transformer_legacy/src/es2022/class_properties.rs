/// SWC's `class_properties::Config`. All flags default to `false`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ClassPropertiesOptions {
    /// Represent private members as ordinary properties instead of WeakMaps
    /// and WeakSets.
    pub private_as_properties: bool,
    /// Assign public fields rather than defining them.
    pub set_public_fields: bool,
    /// Assume a class's superclass does not change.
    pub constant_super: bool,
    /// Assume values cannot be the special `document.all` object.
    pub no_document_all: bool,
    /// Assume property getters have no side effects.
    ///
    /// The singular spelling follows SWC's class-properties configuration.
    pub pure_getter: bool,
}
