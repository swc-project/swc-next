use swc_next_bridge::bridge;
use swc_next_core::js::transformer::transform_source_to_buffer;

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
    unsafe { transform_source_to_buffer(source, source_utf16_len, &options) }
}
