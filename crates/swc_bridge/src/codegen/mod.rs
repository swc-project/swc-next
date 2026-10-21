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

/// Print a trusted AST transfer buffer.
///
/// # Safety
///
/// `buffer` must be produced by `@swc-next/encoder` matching the version of this
/// crate and follow the [AST transfer protocol](https://github.com/swc-project/swc-next/blob/main/crates/swc_ecma_ast/src/ffi/ast.rs).
/// It must satisfy the alignment requirements of [`deserialize_from`] and remain
/// immutable throughout this call.
pub unsafe fn print_from_buffer(
    buffer: &[u8],
    options: Option<CodegenOptions>,
) -> Result<CodegenOutput, DeserializeError> {
    let allocator = Allocator::default();
    // SAFETY: The caller guarantees a valid, immutable AST transfer buffer.
    let tree = unsafe { deserialize_from(buffer, &allocator) }?;
    let result = print(&tree, options.unwrap_or_default().into());
    Ok(CodegenOutput { code: result.code })
}
