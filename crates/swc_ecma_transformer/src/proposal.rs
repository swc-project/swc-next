#[derive(Debug, Default, Clone, Copy, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ProposalOptions {
    /// Preserve class static block behavior when a block makes the class
    /// object non-extensible with `Object.preventExtensions`, `Object.seal`,
    /// or `Object.freeze`.
    pub prevent_extensions: bool,
}
