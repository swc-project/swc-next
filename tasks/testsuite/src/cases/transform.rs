use std::path::Path;

use swc_next_ecma_codegen::Options as CodegenOptions;
use swc_next_ecma_transformer::TransformerOptions;

use super::Case;

/// An upstream input/output pair and the options needed to transform it.
pub trait TransformCase: Case {
    fn package_name(&self) -> &str;
    fn output_path(&self) -> &Path;
    fn output(&self) -> &str;
    fn transformer_options(&self) -> TransformerOptions;
    fn codegen_options(&self) -> CodegenOptions {
        CodegenOptions::default()
    }
}
