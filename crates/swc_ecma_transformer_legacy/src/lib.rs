//! Legacy SWC JavaScript transforms for the SWC Next AST.
//!
//! Options follow SWC Next's per-edition and language grouping, with the fields
//! and defaults of legacy SWC's transform configurations. [`Transformer::transform`]
//! accepts a mutable AST without a semantic model. Selected transforms currently
//! report errors until they are ported. The source inventory is in
//! `docs/transform-inventory.md`.
//!
//! Configuring one edition does not enable another edition or RegExp lowering.
//! Switches default to `false`; optional transforms default to `None` (disabled).
//! Use `Some(Default::default())` to enable a configurable transform with SWC's
//! defaults. Per-transform SWC flags remain local to their transform options.
//!
//! ```
//! use swc_next_ecma_transformer_legacy::{
//!     TransformerOptions,
//!     env::EnvOptions,
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
//!     env: EnvOptions {
//!         es2016: ES2016Options { exponentiation: true },
//!         es2017: ES2017Options {
//!             async_to_generator: Some(AsyncToGeneratorOptions::default()),
//!         },
//!         es2018: ES2018Options {
//!             object_rest_spread: Some(ObjectRestSpreadOptions::default()),
//!         },
//!         es2019: ES2019Options { optional_catch_binding: true },
//!         es2020: ES2020Options {
//!             export_namespace_from: true,
//!             nullish_coalescing: Some(NullishCoalescingOptions::default()),
//!             optional_chaining: Some(OptionalChainingOptions::default()),
//!         },
//!         es2021: ES2021Options { logical_assignments: true },
//!         es2022: ES2022Options {
//!             static_blocks: true,
//!             class_properties: Some(ClassPropertiesOptions::default()),
//!             private_in_object: true,
//!         },
//!         regexp: RegExpOptions { dot_all_regex: true, ..Default::default() },
//!     },
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

pub mod env;
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

mod transformer;
pub use transformer::{Transformer, TransformerReturn};

/// A boolean shorthand or explicit configuration, as in SWC Next's public API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolOr<T> {
    Bool(bool),
    Options(T),
}

/// Selects legacy SWC transforms using SWC Next's option grouping.
///
/// All transforms are disabled by default. Edition groups select individual
/// passes rather than target presets. Options describe the intended transforms;
/// this crate does not execute them yet.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TransformerOptions {
    /// Source filename for transforms such as JSX development metadata.
    pub filename: Option<String>,
    /// Resolved ECMAScript and RegExp transform selections.
    pub env: env::EnvOptions,
    /// SWC's React preset options, including JSX and React Refresh.
    pub jsx: Option<jsx::JsxOptions>,
    /// TypeScript stripping/lowering and JSX import-retention options.
    pub typescript: Option<typescript::TypeScriptOptions>,
}
