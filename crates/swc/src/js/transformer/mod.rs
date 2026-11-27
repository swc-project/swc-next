use std::collections::BTreeMap;

#[cfg(feature = "bridge")]
use swc_next_bridge::bridge;
#[cfg(all(feature = "bridge", not(target_family = "wasm")))]
use swc_next_bridge::napi;
#[cfg(any(feature = "js_transformer_v1", feature = "js_transformer_v2"))]
use swc_next_ecma_ast::{Diagnostic, Severity};

use crate::BoolOr;

#[cfg(all(feature = "bridge", feature = "js_transformer_v1"))]
pub mod bridge;
#[cfg(feature = "js_transformer_v1")]
mod v1;
#[cfg(feature = "js_transformer_v2")]
mod v2;

#[cfg(feature = "js_transformer_v1")]
pub use v1::transform_js as transform_js_v1;
#[cfg(feature = "js_transformer_v2")]
#[doc(hidden)]
pub use v2::transform_js as transform_js_v2;

/// JavaScript and TypeScript transform options.
#[cfg_attr(feature = "bridge", bridge(object, js_name = "TransformerConfig"))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Debug, Default)]
pub struct JsTransformerConfig {
    /// Source filename used by transforms, including JSX development metadata.
    /// Overrides the API's shared filename when set.
    pub filename: Option<String>,
    /// Target environment for ECMAScript lowering.
    #[cfg_attr(
        feature = "bridge",
        bridge(ts_type = "string | Array<string> | Record<string, string>")
    )]
    pub target: Option<JsTarget>,
    /// TypeScript transform. `false` keeps TypeScript syntax in the output.
    ///
    /// @default enabled when `lang` is `"ts"`, `"tsx"`, or `"dts"`
    #[cfg_attr(feature = "bridge", bridge(ts_type = "boolean | TypeScriptConfig"))]
    pub typescript: Option<BoolOr<JsTypeScriptConfig>>,
    /// JSX transform. `false` keeps JSX syntax in the output.
    ///
    /// @default enabled with the automatic runtime when `lang` is `"jsx"` or `"tsx"`
    #[cfg_attr(feature = "bridge", bridge(ts_type = "boolean | JsxConfig"))]
    pub jsx: Option<BoolOr<JsxConfig>>,
    /// Decorator transform. `false` keeps decorator syntax in the output.
    ///
    /// @default false
    #[cfg_attr(feature = "bridge", bridge(ts_type = "boolean | DecoratorConfig"))]
    pub decorators: Option<BoolOr<JsDecoratorConfig>>,
    /// Module output format. Defaults to preserving ECMAScript modules.
    #[cfg_attr(feature = "bridge", bridge(ts_type = "ModuleConfig"))]
    pub module: Option<JsModuleConfig>,
    /// Per-feature overrides. `true` enables a transform; `false` disables it.
    #[cfg_attr(
        feature = "bridge",
        bridge(
            ts_type = "Partial<Record<\"class-static-block\" | \"optional-catch-binding\", boolean>>"
        )
    )]
    pub features: Option<BTreeMap<JsFeature, bool>>,
    /// Babel-compatible assumptions.
    #[cfg_attr(feature = "bridge", bridge(ts_type = "Assumptions"))]
    pub assumptions: Option<JsAssumptions>,
    // pub define: Option<BTreeMap<String, String>>, // Global expression replacements.
    // pub helpers: Option<JsHelpers>, // Defaults to inline helpers.
}

/// Target environment for ECMAScript lowering.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JsTarget {
    /// ES edition (`"es2020"`, `"esnext"`) or a single browserslist query.
    Query(String),
    /// Browserslist query list.
    Queries(Vec<String>),
    /// Minimum engine versions (e.g. `{ "chrome": "120", "node": "18" }`).
    Engines(BTreeMap<String, String>),
}

/// Decorator transform options.
#[cfg_attr(feature = "bridge", bridge(object, js_name = "DecoratorConfig"))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JsDecoratorConfig {
    /// Decorator proposal version.
    ///
    /// @default "2023-11"
    #[cfg_attr(feature = "bridge", bridge(ts_type = "\"2023-11\" | \"legacy\""))]
    pub version: Option<JsDecoratorVersion>,
    /// Emit `Reflect.metadata` design-type metadata. Legacy decorators only.
    ///
    /// @default false
    pub metadata: Option<bool>,
}

/// Decorator proposal version.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum JsDecoratorVersion {
    #[default]
    #[cfg_attr(feature = "serde", serde(rename = "2023-11"))]
    V2023_11,
    #[cfg_attr(feature = "serde", serde(rename = "legacy"))]
    Legacy,
}

/// Module output configuration.
#[cfg_attr(feature = "bridge", bridge(object, js_name = "ModuleConfig"))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JsModuleConfig {
    /// Output module format.
    ///
    /// @default "preserve"
    #[cfg_attr(
        feature = "bridge",
        bridge(ts_type = "\"preserve\" | \"commonjs\" | \"amd\" | \"umd\" | \"systemjs\"")
    )]
    pub r#type: Option<JsModuleType>,
}

/// Output module format.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum JsModuleType {
    #[default]
    Preserve,
    CommonJs,
    Amd,
    Umd,
    SystemJs,
}

/// Names of individually controllable lowering transforms.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum JsFeature {
    /// Lower class static blocks (`class-static-block`).
    ClassStaticBlock,
    /// Add a binding to catch clauses (`optional-catch-binding`).
    OptionalCatchBinding,
}

/// TypeScript transform options.
#[cfg_attr(feature = "bridge", bridge(object, js_name = "TypeScriptConfig"))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Debug, Default)]
pub struct JsTypeScriptConfig {
    /// TypeScript's `verbatimModuleSyntax`: preserve value imports and exports
    /// and disable const enum inlining and removal.
    ///
    /// @default false
    pub verbatim_module_syntax: Option<bool>,
    /// TypeScript's `preserveConstEnums`: preserve declarations without disabling inlining.
    ///
    /// @default false
    #[cfg(feature = "js_transformer_v2")]
    #[cfg_attr(feature = "bridge", bridge(skip))]
    pub preserve_const_enums: Option<bool>,
    /// JSX runtime used to determine which factory imports to retain when JSX
    /// syntax is preserved. Does not enable JSX lowering. When JSX lowering is
    /// enabled, its runtime takes precedence.
    ///
    /// @default "classic"
    #[cfg_attr(feature = "bridge", bridge(ts_type = "\"classic\" | \"automatic\""))]
    pub jsx_runtime: Option<JsxRuntime>,
    /// Classic JSX factory used to preserve its value import when JSX syntax
    /// is retained.
    ///
    /// @default "React.createElement"
    pub jsx_pragma: Option<String>,
    /// Classic JSX fragment factory used to preserve its value import when JSX
    /// syntax is retained.
    ///
    /// @default "React.Fragment"
    pub jsx_pragma_frag: Option<String>,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum JsxRuntime {
    Classic,
    #[default]
    Automatic,
}

/// JSX options targeting Babel 8. `useSpread` and `useBuiltIns` are not supported.
#[cfg_attr(feature = "bridge", bridge(object, js_name = "JsxConfig"))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Debug, Default)]
pub struct JsxConfig {
    /// @default "automatic"
    #[cfg_attr(feature = "bridge", bridge(ts_type = "\"classic\" | \"automatic\""))]
    pub runtime: Option<JsxRuntime>,
    /// @default false
    pub development: Option<bool>,
    /// Add `__source` and `__self` in development.
    ///
    /// @default false
    pub source_self: Option<bool>,
    /// Automatic runtime import source.
    ///
    /// @default "react"
    pub import_source: Option<String>,
    /// Classic runtime factory.
    ///
    /// @default "React.createElement"
    pub pragma: Option<String>,
    /// Classic runtime fragment factory.
    ///
    /// @default "React.Fragment"
    pub pragma_frag: Option<String>,
    /// @default true
    pub throw_if_namespace: Option<bool>,
    /// Mark factory calls as pure.
    pub pure: Option<bool>,
    /// Add `displayName` to `createReactClass` calls.
    ///
    /// @default false
    pub display_name: Option<bool>,
    /// React Refresh. `true` uses default options.
    ///
    /// @default false
    #[cfg_attr(feature = "bridge", bridge(ts_type = "boolean | ReactRefreshConfig"))]
    pub refresh: Option<BoolOr<JsReactRefreshConfig>>,
}

/// React Refresh options.
#[cfg_attr(feature = "bridge", bridge(object, js_name = "ReactRefreshConfig"))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Debug, Default)]
pub struct JsReactRefreshConfig {
    /// @default "$RefreshReg$"
    pub refresh_reg: Option<String>,
    /// @default "$RefreshSig$"
    pub refresh_sig: Option<String>,
    /// @default false
    pub emit_full_signatures: Option<bool>,
}

/// Babel assumptions. All default to false.
#[cfg_attr(feature = "bridge", bridge(object, js_name = "Assumptions"))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JsAssumptions {
    // pub array_like_is_iterable: Option<bool>,
    // pub constant_reexports: Option<bool>,
    pub constant_super: Option<bool>,
    // pub enumerable_module_meta: Option<bool>,
    pub ignore_function_length: Option<bool>,
    // pub ignore_function_name: Option<bool>,
    // pub ignore_to_primitive_hint: Option<bool>,
    // pub iterable_is_array: Option<bool>,
    // pub mutable_template_object: Option<bool>,
    // pub no_class_calls: Option<bool>,
    pub no_document_all: Option<bool>,
    // pub no_incomplete_ns_import_detection: Option<bool>,
    // pub no_new_arrows: Option<bool>,
    pub object_rest_no_symbols: Option<bool>,
    pub private_fields_as_properties: Option<bool>,
    pub pure_getters: Option<bool>,
    // pub set_class_methods: Option<bool>,
    // pub set_computed_properties: Option<bool>,
    pub set_public_class_fields: Option<bool>,
    pub set_spread_properties: Option<bool>,
    // pub skip_for_of_iterator_closing: Option<bool>,
    // pub super_is_callable_constructor: Option<bool>,
}

#[cfg(any(feature = "js_transformer_v1", feature = "js_transformer_v2"))]
pub(super) fn enabled_options<T: Default, U: From<T>>(options: Option<BoolOr<T>>) -> Option<U> {
    match options {
        None | Some(BoolOr::Bool(false)) => None,
        Some(BoolOr::Bool(true)) => Some(T::default().into()),
        Some(BoolOr::Options(options)) => Some(options.into()),
    }
}

#[cfg(any(feature = "js_transformer_v1", feature = "js_transformer_v2"))]
pub(super) fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
}
