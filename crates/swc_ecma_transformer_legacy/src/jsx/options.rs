/// SWC's React JSX runtime selection. The default runtime is classic.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum JsxRuntime {
    Automatic,
    #[default]
    Classic,
    /// Keep JSX syntax, matching SWC's compiler-level `preserve` mode.
    Preserve,
}

/// SWC's `swc_ecma_transforms_react::Options`.
///
/// Optional values preserve SWC's distinction between unset and explicitly set
/// configuration. An unset runtime selects [`JsxRuntime::Classic`]; import source
/// defaults to `react`, pragmas to `React.createElement` / `React.Fragment`,
/// `throw_if_namespace` to `true`, and `development` to `false`.
///
/// The React preset includes display names and purity annotations. Development
/// mode also enables source/self metadata; these are not independent SWC options.
/// Text options use owned strings instead of legacy SWC's atom/byte-string types.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct JsxOptions {
    /// SWC compatibility flag, currently unused by the reference React lowering.
    pub next: Option<bool>,
    /// `None` uses the classic runtime.
    pub runtime: Option<JsxRuntime>,
    /// Module supplying the automatic JSX runtime.
    pub import_source: Option<String>,
    /// JSX factory expression for the classic runtime.
    pub pragma: Option<String>,
    /// Fragment factory expression for the classic runtime.
    pub pragma_frag: Option<String>,
    /// Report XML namespace syntax in JSX. Defaults to `true` when unset.
    pub throw_if_namespace: Option<bool>,
    /// Select development JSX output and source/self metadata.
    pub development: Option<bool>,
    /// Legacy SWC compatibility option; ignored by its current JSX lowering.
    pub use_builtins: Option<bool>,
    /// Legacy SWC compatibility option; ignored by its current JSX lowering.
    pub use_spread: Option<bool>,
    /// Configure React Refresh; `None` disables it. SWC applies it in development
    /// mode only.
    pub refresh: Option<ReactRefreshOptions>,
}

/// SWC's React Refresh configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactRefreshOptions {
    /// Registration function, defaulting to `$RefreshReg$`.
    pub refresh_reg: String,
    /// Signature function, defaulting to `$RefreshSig$`.
    pub refresh_sig: String,
    /// Emit full hook signatures instead of hashes. Defaults to `false`.
    pub emit_full_signatures: bool,
}

impl Default for ReactRefreshOptions {
    fn default() -> Self {
        Self {
            refresh_reg: "$RefreshReg$".to_owned(),
            refresh_sig: "$RefreshSig$".to_owned(),
            emit_full_signatures: false,
        }
    }
}
