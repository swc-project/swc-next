use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
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
    /// Retain const enums and their member reads so runtime mutations remain
    /// observable, independently of `verbatimModuleSyntax`. Defaults to false.
    pub ts_enum_is_mutable: bool,
    /// Suppress only the synthetic `export {}` marker inserted after erasing
    /// all module syntax. Explicit exports remain unchanged. Defaults to false.
    pub no_empty_export: bool,
    /// Retain the loading effects of value imports whose bindings are used only
    /// as types or are unused. Explicit `import type` declarations are erased.
    pub import_not_used_as_values: ImportsNotUsedAsValues,
    /// Keep instance field initializers in native class fields. When false,
    /// classes with parameter properties initialize those properties before
    /// other instance fields. Defaults to true. Computed keys, auto-accessors,
    /// decorators, field `new.target`, and constructor-binding capture currently
    /// produce diagnostics when their initializers would need to move.
    #[serde(default = "default_native_class_properties")]
    pub native_class_properties: bool,
    /// Runtime policy for TypeScript import/export assignments. The default
    /// retains SWC Next's synchronous-import representation.
    pub import_export_assign_config: ImportExportAssignConfig,
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

/// Policy for value imports without runtime references.
#[derive(Debug, Default, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImportsNotUsedAsValues {
    #[default]
    Remove,
    Preserve,
}

fn default_native_class_properties() -> bool {
    true
}

impl Default for TypeScriptOptions {
    fn default() -> Self {
        Self {
            verbatim_module_syntax: false,
            preserve_const_enums: false,
            ts_enum_is_mutable: false,
            no_empty_export: false,
            import_not_used_as_values: ImportsNotUsedAsValues::Remove,
            native_class_properties: true,
            import_export_assign_config: ImportExportAssignConfig::ImportSync,
            jsx: JsxImportOptions::default(),
        }
    }
}

/// Emission policy for `import value = require("pkg")` and `export = value`.
#[derive(Debug, Default, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum ImportExportAssignConfig {
    /// Emit `import.sync`, retaining export assignments for later lowering.
    #[default]
    ImportSync,
    /// Emit `require` calls and `module.exports` assignments.
    Classic,
    /// Keep runtime TypeScript assignments for a later module transform.
    Preserve,
    /// Load through Node's `createRequire(import.meta.url)`; reject `export =`.
    NodeNext,
    /// Reject runtime external import assignments and export assignments.
    EsNext,
}
