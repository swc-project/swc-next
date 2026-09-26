use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct TypeScriptOptions {
    /// TypeScript's `verbatimModuleSyntax`: preserves value imports/exports.
    /// Defaults to `false`.
    /// Empty value import/export clauses retain their module-loading effects
    /// in either mode.
    pub verbatim_module_syntax: bool,
}
