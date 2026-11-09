use swc_next_bridge::bridge;
use swc_next_core::js::parser::JsParserConfig as ParserConfig;

#[cfg(all(feature = "test-native", not(target_family = "wasm")))]
pub mod test;

#[bridge]
pub fn format_doc_buffer_sync(
    source: String,
    parser: Option<ParserConfig>,
    options: &[u8],
) -> Result<Vec<u8>, String> {
    swc_next_bridge::format_doc(source, parser.map(Into::into), options)
}
