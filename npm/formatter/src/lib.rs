use swc_next_bridge::{ParserOptions, bridge};

#[cfg(all(feature = "test-native", not(target_family = "wasm")))]
pub mod test;

#[bridge]
pub fn format_doc_buffer_sync(
    source: String,
    parser: Option<ParserOptions>,
    options: &[u8],
) -> Result<Vec<u8>, String> {
    swc_next_bridge::format_doc(source, parser, options)
}
