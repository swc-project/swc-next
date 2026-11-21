/// SWC's `optional_chaining::Config`. All flags default to `false`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OptionalChainingOptions {
    /// Assume values cannot be the special `document.all` object.
    pub no_document_all: bool,
    /// Assume property getters have no side effects.
    ///
    /// The singular spelling follows SWC's optional-chaining configuration.
    pub pure_getter: bool,
}
