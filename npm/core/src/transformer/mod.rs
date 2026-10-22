use swc_next_bridge::{
    CodegenOptions, DeserializeError, ParserOptions, TransformPassOptions, bridge,
};

#[bridge]
/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
/// `source_utf16_len` must match the UTF-16 code unit length of `source`.
pub unsafe fn parse_transform_sync(
    source: &[u8],
    source_utf16_len: u32,
    parser_options: Option<ParserOptions>,
    transform_options: Option<TransformPassOptions>,
) -> Vec<u8> {
    // SAFETY: The caller guarantees the source encoding and UTF-16 length.
    unsafe {
        swc_next_bridge::parse_transform_to_buffer(
            source,
            source_utf16_len,
            parser_options,
            transform_options,
        )
    }
}

#[bridge]
/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
/// `source_utf16_len` must match the UTF-16 code unit length of `source`.
pub unsafe fn transform_sync(
    source: &[u8],
    source_utf16_len: u32,
    parser_options: Option<ParserOptions>,
    transform_options: Option<TransformPassOptions>,
    codegen_options: Option<CodegenOptions>,
) -> Vec<u8> {
    // SAFETY: The caller guarantees the source encoding and UTF-16 length.
    unsafe {
        swc_next_bridge::transform_source_to_buffer(
            source,
            source_utf16_len,
            parser_options,
            transform_options,
            codegen_options,
        )
    }
}

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

#[bridge]
/// # Safety
///
/// `buffer` must be produced by `@swc-next/encoder` matching the version of this
/// binding and follow the [AST transfer protocol](https://github.com/swc-project/swc-next/blob/main/crates/swc_ecma_ast/src/ffi/ast.rs).
/// It must be aligned to eight bytes and remain immutable throughout the call.
pub unsafe fn transform_print_sync(
    buffer: &[u8],
    options: Option<TransformPassOptions>,
    codegen_options: Option<CodegenOptions>,
) -> Result<Vec<u8>, DeserializeError> {
    // SAFETY: The caller guarantees the AST transfer protocol and alignment.
    unsafe { swc_next_bridge::transform_print_from_buffer(buffer, options, codegen_options) }
}
