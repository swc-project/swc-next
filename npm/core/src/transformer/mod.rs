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
/// Sections must be produced by the matching JavaScript encoder and satisfy
/// the validity, alignment, and immutability contracts of
/// `swc_next_bridge::transform_from_parts`.
pub unsafe fn transform_ast_sync(
    header: &[u8],
    spans: &[u8],
    kinds: &[u8],
    payloads: &[u8],
    extra: &[u8],
    strings: &[u8],
    options: Option<TransformPassOptions>,
) -> Result<Vec<u8>, DeserializeError> {
    // SAFETY: The caller guarantees valid, immutable encoder sections.
    unsafe {
        swc_next_bridge::transform_from_parts(
            header, spans, kinds, payloads, extra, strings, options,
        )
    }
}

#[bridge]
/// # Safety
///
/// Sections must be produced by the matching JavaScript encoder and satisfy
/// the validity, alignment, and immutability contracts of
/// `swc_next_bridge::transform_print_from_parts`.
#[allow(
    clippy::too_many_arguments,
    reason = "AST transfers contain six separate sections"
)]
pub unsafe fn transform_print_sync(
    header: &[u8],
    spans: &[u8],
    kinds: &[u8],
    payloads: &[u8],
    extra: &[u8],
    strings: &[u8],
    options: Option<TransformPassOptions>,
    codegen_options: Option<CodegenOptions>,
) -> Result<Vec<u8>, DeserializeError> {
    // SAFETY: The caller guarantees valid, immutable encoder sections.
    unsafe {
        swc_next_bridge::transform_print_from_parts(
            header,
            spans,
            kinds,
            payloads,
            extra,
            strings,
            options,
            codegen_options,
        )
    }
}
