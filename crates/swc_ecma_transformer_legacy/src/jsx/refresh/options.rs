/// SWC's React Refresh configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// Registration function, defaulting to `$RefreshReg$`.
    pub refresh_reg: String,
    /// Signature function, defaulting to `$RefreshSig$`.
    pub refresh_sig: String,
    /// Emit full hook signatures instead of hashes. Defaults to `false`.
    pub emit_full_signatures: bool,
}

impl Default for Options {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/options.rs#L29>.
    fn default() -> Self {
        Self {
            refresh_reg: "$RefreshReg$".to_owned(),
            refresh_sig: "$RefreshSig$".to_owned(),
            emit_full_signatures: false,
        }
    }
}
