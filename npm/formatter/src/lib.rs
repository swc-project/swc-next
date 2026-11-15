//! Native and Wasm printing for language-independent formatting documents.
//!
//! Experimental and unstable. APIs, document formats, and formatting output may
//! change without notice.

use swc_next_bridge::bridge;

#[bridge]
/// Internal binding API and binary protocol, not covered by semver. Use a matching SWC Next wrapper.
pub fn print_doc_buffer_sync(buffer: &[u8], options: &[u8]) -> Result<String, String> {
    swc_next_core::formatter::bridge::print_doc(buffer, options)
}

#[bridge]
/// Internal binding API and binary protocol, not covered by semver. Use a matching SWC Next wrapper.
pub fn print_doc_buffer_with_cursor_sync(buffer: &[u8], options: &[u8]) -> Result<Vec<u8>, String> {
    swc_next_core::formatter::bridge::print_doc_with_cursor(buffer, options)
}
