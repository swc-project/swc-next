//! Legacy SWC JavaScript transforms for the SWC Next AST.
//!
//! Options follow SWC Next's per-edition and language grouping, with the fields
//! and defaults of legacy SWC's transform configurations. Transform execution is
//! not implemented yet. The source inventory is in `docs/transform-inventory.md`.
//!
//! Configuring one edition does not enable another edition or RegExp lowering.
//! Switches default to `false`; optional transforms default to `None` (disabled).
//! Use `Some(Default::default())` to enable a configurable transform with SWC's
//! defaults. Per-transform SWC flags remain local to their transform options.
//!
//! ```
//! use swc_next_ecma_transformer_legacy::{
//!     TransformerOptions,
//!     es2016::ES2016Options,
//!     es2017::{AsyncToGeneratorOptions, ES2017Options},
//!     es2018::{ES2018Options, ObjectRestSpreadOptions},
//!     es2019::ES2019Options,
//!     es2020::{ES2020Options, NullishCoalescingOptions, OptionalChainingOptions},
//!     es2021::ES2021Options,
//!     es2022::{ClassPropertiesOptions, ES2022Options},
//!     jsx::{JsxOptions, JsxRuntime, ReactRefreshOptions},
//!     regexp::RegExpOptions,
//!     typescript::{TsImportExportAssignConfig, TypeScriptOptions},
//! };
//!
//! let options = TransformerOptions {
//!     filename: Some("input.tsx".into()),
//!     es2016: ES2016Options { exponentiation: true },
//!     es2017: ES2017Options {
//!         async_to_generator: Some(AsyncToGeneratorOptions::default()),
//!     },
//!     es2018: ES2018Options {
//!         object_rest_spread: Some(ObjectRestSpreadOptions::default()),
//!     },
//!     es2019: ES2019Options { optional_catch_binding: true },
//!     es2020: ES2020Options {
//!         export_namespace_from: true,
//!         nullish_coalescing: Some(NullishCoalescingOptions::default()),
//!         optional_chaining: Some(OptionalChainingOptions::default()),
//!     },
//!     es2021: ES2021Options { logical_assignments: true },
//!     es2022: ES2022Options {
//!         static_blocks: true,
//!         class_properties: Some(ClassPropertiesOptions::default()),
//!         private_in_object: true,
//!     },
//!     regexp: RegExpOptions { dot_all_regex: true, ..Default::default() },
//!     jsx: Some(JsxOptions {
//!         runtime: Some(JsxRuntime::Automatic),
//!         development: Some(true),
//!         refresh: Some(ReactRefreshOptions::default()),
//!         ..Default::default()
//!     }),
//!     typescript: Some(TypeScriptOptions {
//!         import_export_assign_config: TsImportExportAssignConfig::EsNext,
//!         ..Default::default()
//!     }),
//! };
//! ```

pub mod es2016;
pub mod es2017;
pub mod es2018;
pub mod es2019;
pub mod es2020;
pub mod es2021;
pub mod es2022;
pub mod jsx;
pub mod regexp;
pub mod typescript;

/// Selects legacy SWC transforms using SWC Next's option grouping.
///
/// All transforms are disabled by default. Edition groups select individual
/// passes rather than target presets. Options describe the intended transforms;
/// this crate does not execute them yet.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TransformerOptions {
    /// Source filename for transforms such as JSX development metadata.
    pub filename: Option<String>,
    pub es2016: es2016::ES2016Options,
    pub es2017: es2017::ES2017Options,
    pub es2018: es2018::ES2018Options,
    pub es2019: es2019::ES2019Options,
    pub es2020: es2020::ES2020Options,
    pub es2021: es2021::ES2021Options,
    pub es2022: es2022::ES2022Options,
    /// RegExp feature switches, independent of the edition groups.
    pub regexp: regexp::RegExpOptions,
    /// SWC's React preset options, including JSX and React Refresh.
    pub jsx: Option<jsx::JsxOptions>,
    /// TypeScript stripping/lowering and JSX import-retention options.
    pub typescript: Option<typescript::TypeScriptOptions>,
}
