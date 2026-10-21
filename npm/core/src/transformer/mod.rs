use swc_next_bridge::{DeserializeError, TransformPassOptions, bridge};

#[bridge]
/// # Safety
///
/// `buffer` must be produced by `@swc-next/encoder` matching the version of this
/// binding and follow the [AST transfer protocol](https://github.com/swc-project/swc-next/blob/main/crates/swc_ecma_ast/src/ffi/ast.rs).
/// It must be aligned to eight bytes and remain immutable throughout the call.
pub unsafe fn transform_ast_sync(
    buffer: &[u8],
    options: Option<TransformPassOptions>,
) -> Result<Vec<u8>, DeserializeError> {
    // SAFETY: The caller guarantees the AST transfer protocol and alignment.
    unsafe { swc_next_bridge::transform_from_buffer(buffer, options) }
}
