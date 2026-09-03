mod options;
mod options_wire;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::ffi::ast::{DeserializeError, deserialize_from};
use swc_next_ecma_codegen::print;

use crate::{BridgeEncode, bridge};

pub use self::options::CodegenOptions;

#[bridge(object)]
pub struct CodegenOutput {
    pub code: String,
}

#[bridge]
impl BridgeEncode for CodegenOutput {
    type Wire = String;

    fn encode(self) -> Self::Wire {
        self.code
    }
}

pub fn print_from_buffer(
    buffer: &[u8],
    options: Option<CodegenOptions>,
) -> Result<CodegenOutput, DeserializeError> {
    let allocator = Allocator::default();
    let tree = deserialize_from(buffer, &allocator)?;
    let result = print(&allocator, &tree, options.unwrap_or_default().into());
    Ok(CodegenOutput {
        code: result.code.into(),
    })
}
