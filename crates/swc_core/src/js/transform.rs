use std::collections::BTreeMap;

use crate::BoolOr;

pub use swc_next_ecma_transformer::jsx::JsxRuntime;

/// Currently implemented JavaScript and TypeScript transforms.
#[derive(Clone, Debug, Default)]
pub struct JsTransformOptions {
    /// Disable TypeScript lowering or enable it with defaults or explicit settings.
    pub typescript: Option<BoolOr<JsTypeScriptOptions>>,
    /// Disable JSX lowering or enable it with defaults or explicit settings.
    pub jsx: Option<BoolOr<JsxOptions>>,
    /// Explicitly enable or disable implemented lowering transforms.
    pub features: Option<BTreeMap<JsFeature, bool>>,
    // pub decorators: Option<JsDecorators>,
    // pub assumptions: Option<JsAssumptions>,
    // pub module: Option<JsModuleOptions>, // Defaults to the preserve format.
    // pub define: Option<BTreeMap<String, String>>, // Global expression replacements.
    // pub helpers: Option<JsHelpers>, // Defaults to inline helpers.
}

/// Implemented, individually controllable JavaScript transforms.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum JsFeature {
    /// Lower class static blocks (`class-static-block`).
    ClassStaticBlock,
    // Custom(String), // The draft also accepts other lowering feature names.
}

/// Public TypeScript lowering settings.
///
/// JSX import dependencies used internally by the transformer are derived from
/// the JSX settings rather than exposed in this section.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsTypeScriptOptions {
    /// Preserve value imports and exports, and disable const-enum inlining and
    /// removal.
    pub verbatim_module_syntax: Option<bool>,
    /// Preserve const-enum declarations without disabling inlining.
    pub preserve_const_enums: Option<bool>,
}

/// JSX lowering settings targeting Babel 8.
#[derive(Clone, Debug, Default)]
pub struct JsxOptions {
    pub runtime: Option<JsxRuntime>,
    pub development: Option<bool>,
    /// Include development source and self metadata.
    pub source_self: Option<bool>,
    /// Import source for the automatic runtime.
    pub import_source: Option<String>,
    /// Factory expression for the classic runtime.
    pub pragma: Option<String>,
    /// Fragment factory expression for the classic runtime.
    pub pragma_frag: Option<String>,
    pub throw_if_namespace: Option<bool>,
    /// Mark factory calls as pure. The default depends on whether a custom
    /// pragma is supplied.
    pub pure: Option<bool>,
    /// Add display names to `createReactClass` calls.
    pub display_name: Option<bool>,
    pub refresh: Option<BoolOr<JsReactRefreshOptions>>,
}

/// React Refresh instrumentation settings.
#[derive(Clone, Debug, Default)]
pub struct JsReactRefreshOptions {
    /// Registration expression. Defaults to `$RefreshReg$` when enabled.
    pub refresh_reg: Option<String>,
    /// Signature expression. Defaults to `$RefreshSig$` when enabled.
    pub refresh_sig: Option<String>,
    pub emit_full_signatures: Option<bool>,
}

// pub enum JsDecorators {
//     Disabled, // false
//     Options(JsDecoratorOptions),
// }
//
// pub struct JsDecoratorOptions {
//     pub version: JsDecoratorVersion,
//     pub metadata: Option<bool>, // Legacy decorators only; defaults to false.
// }
//
// pub enum JsDecoratorVersion {
//     V2023_11, // "2023-11"
//     Legacy, // "legacy"
// }
//
// All assumptions default to false.
// pub struct JsAssumptions {
//     pub array_like_is_iterable: Option<bool>,
//     pub constant_reexports: Option<bool>,
//     pub constant_super: Option<bool>,
//     pub enumerable_module_meta: Option<bool>,
//     pub ignore_function_length: Option<bool>,
//     pub ignore_function_name: Option<bool>,
//     pub ignore_to_primitive_hint: Option<bool>,
//     pub iterable_is_array: Option<bool>,
//     pub mutable_template_object: Option<bool>,
//     pub no_class_calls: Option<bool>,
//     pub no_document_all: Option<bool>,
//     pub no_incomplete_ns_import_detection: Option<bool>,
//     pub no_new_arrows: Option<bool>,
//     pub object_rest_no_symbols: Option<bool>,
//     pub private_fields_as_properties: Option<bool>,
//     pub pure_getters: Option<bool>,
//     pub set_class_methods: Option<bool>,
//     pub set_computed_properties: Option<bool>,
//     pub set_public_class_fields: Option<bool>,
//     pub set_spread_properties: Option<bool>,
//     pub skip_for_of_iterator_closing: Option<bool>,
//     pub super_is_callable_constructor: Option<bool>,
// }
//
// pub struct JsModuleOptions {
//     pub r#type: JsModuleType,
// }
//
// pub enum JsModuleType {
//     Preserve,
//     CommonJs,
// }
//
// pub enum JsHelpers {
//     Inline,
//     External, // Import from @swc/helpers.
// }
