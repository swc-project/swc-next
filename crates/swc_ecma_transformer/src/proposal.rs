#[derive(Debug, Default, Clone, Copy)]
pub struct ProposalOptions {
    /// Preserve class static block behavior when a block makes the class
    /// object non-extensible with `Object.preventExtensions`, `Object.seal`,
    /// or `Object.freeze`.
    pub prevent_extensions: bool,
}
