//! Native and Wasm JavaScript and TypeScript formatting document production.
//!
//! Experimental and unstable. APIs, document formats, and formatting output may
//! change without notice.

use swc_next_bridge::bridge;
use swc_next_core::js::{
    formatter::{JsFormatterConfig, bridge::FormatDocOutput},
    parser::JsParserConfig as ParserConfig,
};

#[bridge(
    ts_args_type = "source: string, parser: number, options: Uint8Array",
    ts_return_type = "Uint8Array"
)]
/// Internal binding API and binary protocol, not covered by semver. Use a matching SWC Next wrapper.
pub fn format_doc_buffer_sync(
    source: String,
    parser: Option<ParserConfig>,
    options: Option<JsFormatterConfig>,
) -> Result<FormatDocOutput, String> {
    swc_next_core::js::formatter::bridge::format_doc(source, parser.map(Into::into), options)
}
