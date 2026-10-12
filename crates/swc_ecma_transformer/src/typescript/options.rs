use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct TypeScriptOptions {
    /// TypeScript's `verbatimModuleSyntax`: preserves value imports/exports
    /// and disables const-enum inlining and removal. Defaults to `false`.
    /// Empty value import/export clauses retain their module-loading effects
    /// in either mode.
    pub verbatim_module_syntax: bool,
    /// TypeScript's `preserveConstEnums`: preserves declarations without
    /// disabling inlining. Defaults to `false`; only safe, unexported enums
    /// may be removed by this single-file transform.
    pub preserve_const_enums: bool,
    /// Factory dependencies for JSX preserved for a later transform.
    pub jsx: JsxImportOptions,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct JsxImportOptions {
    pub automatic: bool,
    pub pragma: Option<String>,
    pub pragma_frag: Option<String>,
}
