/// Currently implemented JavaScript code generation settings.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsCodegenOptions {
    pub ascii_only: Option<bool>,
    /// Escape closing script tags. Tagged template raw text is preserved.
    pub inline_script: Option<bool>,
    /// Print compact code independently of AST compression and mangling.
    pub minify: Option<bool>,
}
