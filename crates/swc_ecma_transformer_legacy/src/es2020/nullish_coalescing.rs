/// SWC's `nullish_coalescing::Config`. All flags default to `false`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NullishCoalescingOptions {
    /// Assume values cannot be the special `document.all` object, allowing
    /// `value != null` instead of separate null and undefined comparisons.
    pub no_document_all: bool,
}
