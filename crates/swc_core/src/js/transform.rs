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
}

/// Implemented, individually controllable JavaScript transforms.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum JsFeature {
    /// Lower class static blocks (`class-static-block`).
    ClassStaticBlock,
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
