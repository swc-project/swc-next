use serde::Deserialize;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JsxRuntime {
    Classic,
    #[default]
    Automatic,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct JsxOptions {
    #[serde(skip)]
    pub jsx_self: bool,
    #[serde(skip)]
    pub jsx_source: bool,
    pub runtime: JsxRuntime,
    pub development: bool,
    pub source_self: bool,
    pub import_source: Option<String>,
    pub pragma: Option<String>,
    pub pragma_frag: Option<String>,
    pub throw_if_namespace: bool,
    pub pure: Option<bool>,
    pub use_built_ins: Option<bool>,
    pub use_spread: Option<bool>,
    pub refresh: Option<ReactRefreshOptions>,
}

impl Default for JsxOptions {
    fn default() -> Self {
        Self {
            jsx_self: false,
            jsx_source: false,
            runtime: JsxRuntime::Automatic,
            development: false,
            source_self: false,
            import_source: None,
            pragma: None,
            pragma_frag: None,
            throw_if_namespace: true,
            pure: None,
            use_built_ins: None,
            use_spread: None,
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
