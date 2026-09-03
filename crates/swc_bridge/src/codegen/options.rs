use swc_next_ecma_codegen::Options as AstCodegenOptions;

use crate::bridge;

#[bridge(object)]
#[derive(Default)]
pub struct CodegenOptions {
    pub ascii_only: Option<bool>,
    pub minify: Option<bool>,
}

impl From<CodegenOptions> for AstCodegenOptions {
    fn from(options: CodegenOptions) -> Self {
        let defaults = Self::default();
        Self {
            ascii_only: options.ascii_only.unwrap_or(defaults.ascii_only),
            minify: options.minify.unwrap_or(defaults.minify),
            comments: false,
            ..defaults
        }
    }
}
