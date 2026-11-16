//! Native and Wasm printing for language-independent formatting documents.
//!
//! Experimental and unstable. APIs, document formats, and formatting output may
//! change without notice.

use swc_next_bridge::bridge;
use swc_next_core::formatter::PrintConfig;

#[bridge(ts_args_type = "buffer: Uint8Array, options: Uint8Array")]
/// Internal binding API and binary protocol, not covered by semver. Use a matching SWC Next wrapper.
pub fn print_doc_buffer_sync(
    buffer: &[u8],
    options: Option<PrintConfig>,
) -> Result<String, String> {
    swc_next_core::formatter::bridge::print_doc(buffer, options)
}

#[bridge(ts_args_type = "buffer: Uint8Array, options: Uint8Array")]
/// Internal binding API and binary protocol, not covered by semver. Use a matching SWC Next wrapper.
pub fn print_doc_buffer_with_cursor_sync(
    buffer: &[u8],
    options: Option<PrintConfig>,
) -> Result<Vec<u8>, String> {
    swc_next_core::formatter::bridge::print_doc_with_cursor(buffer, options)
}
