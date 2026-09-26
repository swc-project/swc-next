use swc_next_ecma_codegen::Options as AstCodegenOptions;

use crate::bridge;

#[bridge(object)]
#[derive(Default)]
pub struct CodegenOptions {
    pub ascii_only: Option<bool>,
    /// Escape closing script tags; tagged template raw text is preserved. Defaults to false.
    pub inline_script: Option<bool>,
    pub minify: Option<bool>,
}

impl From<CodegenOptions> for AstCodegenOptions {
    fn from(options: CodegenOptions) -> Self {
        let defaults = Self::default();
        Self {
            ascii_only: options.ascii_only.unwrap_or(defaults.ascii_only),
            inline_script: options.inline_script.unwrap_or(defaults.inline_script),
            minify: options.minify.unwrap_or(defaults.minify),
            comments: false,
            ..defaults
        }
    }
}
