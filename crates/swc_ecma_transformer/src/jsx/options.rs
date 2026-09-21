use serde::Deserialize;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JsxRuntime {
    Classic,
    #[default]
    Automatic,
}

/// JSX configuration targeting Babel 8.
///
/// `useSpread` and `useBuiltIns` were removed in Babel 8 and are intentionally
/// rejected. Lowering object spread syntax belongs to a separate transform.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct JsxOptions {
    pub runtime: JsxRuntime,
    pub development: bool,
    /// Opt into development source/self metadata, disabled by default in Babel 8.
    pub source_self: bool,
    pub import_source: Option<String>,
    pub pragma: Option<String>,
    pub pragma_frag: Option<String>,
    pub throw_if_namespace: bool,
    pub pure: Option<bool>,
    pub refresh: Option<ReactRefreshOptions>,
}

impl Default for JsxOptions {
    fn default() -> Self {
        Self {
            runtime: JsxRuntime::Automatic,
            development: false,
            source_self: false,
            import_source: None,
            pragma: None,
            pragma_frag: None,
            throw_if_namespace: true,
            pure: None,
            refresh: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ReactRefreshOptions {
    pub refresh_reg: String,
}

impl Default for ReactRefreshOptions {
    fn default() -> Self {
        Self {
            refresh_reg: "$RefreshReg$".to_owned(),
        }
    }
}
