//! Native and Wasm JavaScript and TypeScript formatting document production.
//!
//! Experimental and unstable. APIs, document formats, and formatting output may
//! change without notice.

use swc_next_bridge::bridge;
use swc_next_core::js::parser::JsParserConfig as ParserConfig;

#[cfg(all(feature = "test-native", not(target_family = "wasm")))]
pub mod test;

#[bridge(ts_args_type = "source: string, parser: number, options: Uint8Array")]
/// Internal binding API and binary protocol, not covered by semver. Use a matching SWC Next wrapper.
pub fn format_doc_buffer_sync(
    source: String,
    parser: Option<ParserConfig>,
    options: &[u8],
) -> Result<Vec<u8>, String> {
    swc_next_core::js::formatter::bridge::format_doc(source, parser.map(Into::into), options)
}
