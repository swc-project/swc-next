/// SWC's `swc_ecma_transforms_typescript::Config`, with TSX configuration nested
/// under [`jsx`](Self::jsx) as in SWC Next.
///
/// Flags default to `false`. Enum defaults follow SWC. Enabling TypeScript does
/// not enable JSX lowering; JSX import-retention configuration can be used when
/// JSX is preserved for another transform.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Config {
    /// Preserve value import/export syntax according to `verbatimModuleSyntax`.
    pub verbatim_module_syntax: bool,
    /// Preserve native class fields while lowering TypeScript class syntax.
    pub native_class_properties: bool,
    /// How to handle imports that are used only as types.
    pub import_not_used_as_values: ImportsNotUsedAsValues,
    /// Do not add `export {}` when stripping removes the last module declaration.
    pub no_empty_export: bool,
    /// How to handle TypeScript `import =` and `export =` declarations.
    pub import_export_assign_config: TsImportExportAssignConfig,
    /// Disable extra inlining of non-const enum member reads in the same module.
    /// Enum initializers still use TypeScript's constant evaluation rules.
    pub ts_enum_is_mutable: bool,
    /// Legacy SWC's internal marker for an AST parsed as Flow, selecting Flow
    /// cleanup after type erasure. This does not select or configure a parser.
    pub flow_syntax: bool,
    /// JSX factory dependencies, corresponding to SWC's `TsxConfig`.
    pub jsx: TsxConfig,
}

/// SWC's `TsxConfig`, used to retain imports referenced by JSX factories.
///
/// Comment directives can override these expressions. SWC defaults to
/// `React.createElement` and `React.Fragment` when they are unset. Strings replace
/// legacy SWC's byte-string type.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TsxConfig {
    /// Whether JSX uses the automatic runtime (which does not require classic factory imports).
    pub automatic: bool,
    pub pragma: Option<String>,
    pub pragma_frag: Option<String>,
}

/// SWC's policy for imports that are used only as types.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ImportsNotUsedAsValues {
    #[default]
    Remove,
    Preserve,
}

/// SWC's TypeScript import/export assignment policy.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TsImportExportAssignConfig {
    /// Rewrite `import = require()` to a variable and `export =` to
    /// `module.exports`. SWC's legacy default; module passes may instead handle
    /// these declarations after preservation.
    #[default]
    Classic,
    /// Leave the declarations for CommonJS/AMD/UMD lowering.
    Preserve,
    /// Use Node's `createRequire` for `import = require()` and reject `export =`.
    NodeNext,
    /// Reject both import and export assignments.
    EsNext,
}
