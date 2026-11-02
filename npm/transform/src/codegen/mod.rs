use swc_next_bridge::{CodegenOptions, CodegenOutput, DeserializeError, bridge};

#[bridge]
/// # Safety
///
/// Sections must be produced by the matching JavaScript encoder and satisfy
/// the validity, alignment, and immutability contracts of
/// `swc_next_bridge::print_from_parts`.
pub unsafe fn print_sync(
    header: &[u8],
    spans: &[u8],
    kinds: &[u8],
    payloads: &[u8],
    extra: &[u8],
    strings: &[u8],
    options: Option<CodegenOptions>,
) -> Result<CodegenOutput, DeserializeError> {
    // SAFETY: The caller guarantees valid, immutable encoder sections.
    unsafe {
        swc_next_bridge::print_from_parts(header, spans, kinds, payloads, extra, strings, options)
    }
}
