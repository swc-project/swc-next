use swc_next_bridge::bridge;
use swc_next_core::js::transformer as transform;

#[bridge]
/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
/// `source_utf16_len` must match the UTF-16 code unit length of `source`.
pub unsafe fn transform_sync(
    source: &[u8],
    source_utf16_len: u32,
    options: String,
) -> Result<Vec<u8>, String> {
    // SAFETY: The caller guarantees the source encoding and UTF-16 length.
    unsafe { transform::transform_source_to_buffer(source, source_utf16_len, &options) }
}

#[bridge]
/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
/// `source_utf16_len` must match the UTF-16 code unit length of `source`.
pub unsafe fn parse_for_transform_sync(
    source: &[u8],
    source_utf16_len: u32,
    options: String,
) -> Result<Vec<u8>, String> {
    // SAFETY: The caller guarantees the source encoding and UTF-16 length.
    unsafe { transform::parse_for_transform_to_buffer(source, source_utf16_len, &options) }
}

#[bridge]
/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
/// `source_utf16_len` must match the UTF-16 code unit length of `source`.
pub unsafe fn parse_transform_sync(
    source: &[u8],
    source_utf16_len: u32,
    options: String,
) -> Result<Vec<u8>, String> {
    // SAFETY: The caller guarantees the source encoding and UTF-16 length.
    unsafe { transform::parse_transform_to_buffer(source, source_utf16_len, &options) }
}

#[bridge]
/// # Safety
///
/// Sections must come from the matching JavaScript encoder and satisfy the
/// validity, alignment, and immutability contracts of
/// `swc_next_core::js::transformer::transform_from_parts`.
pub unsafe fn transform_ast_sync(
    header: &[u8],
    spans: &[u8],
    kinds: &[u8],
    payloads: &[u8],
    extra: &[u8],
    strings: &[u8],
    options: String,
) -> Result<Vec<u8>, String> {
    // SAFETY: The caller guarantees valid, immutable encoder sections.
    unsafe {
        transform::transform_from_parts([header, spans, kinds, payloads, extra, strings], &options)
    }
}

#[bridge]
/// # Safety
///
/// Sections must come from the matching JavaScript encoder and satisfy the
/// validity, alignment, and immutability contracts of
/// `swc_next_core::js::transformer::transform_print_from_parts`.
pub unsafe fn transform_print_sync(
    header: &[u8],
    spans: &[u8],
    kinds: &[u8],
    payloads: &[u8],
    extra: &[u8],
    strings: &[u8],
    options: String,
) -> Result<Vec<u8>, String> {
    // SAFETY: The caller guarantees valid, immutable encoder sections.
    unsafe {
        transform::transform_print_from_parts(
            [header, spans, kinds, payloads, extra, strings],
            &options,
        )
    }
}

#[bridge]
/// # Safety
///
/// Sections must come from the matching JavaScript encoder and satisfy the
/// validity, alignment, and immutability contracts of
/// `swc_next_core::js::transformer::print_transformed_from_parts`.
pub unsafe fn print_transformed_sync(
    header: &[u8],
    spans: &[u8],
    kinds: &[u8],
    payloads: &[u8],
    extra: &[u8],
    strings: &[u8],
    options: String,
) -> Result<Vec<u8>, String> {
    // SAFETY: The caller guarantees valid, immutable encoder sections.
    unsafe {
        transform::print_transformed_from_parts(
            [header, spans, kinds, payloads, extra, strings],
            &options,
        )
    }
}
