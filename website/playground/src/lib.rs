use swc_next::js::{
    parser::{JsParserConfig as ParserConfig, bridge::parse_to_buffer},
    transformer::bridge::transform_to_buffer,
};
#[cfg(target_family = "wasm")]
use swc_next_bridge::bridge;

/// # Safety
/// `source` must be valid UTF-8 and `source_utf16_len` must match its UTF-16 length.
#[cfg_attr(
    target_family = "wasm",
    bridge(ts_args_type = "source: Uint8Array, sourceUtf16Len: number, options: number")
)]
pub unsafe fn parse_sync(
    source: &[u8],
    source_utf16_len: u32,
    options: Option<ParserConfig>,
) -> Vec<u8> {
    unsafe { parse_to_buffer(source, source_utf16_len, options) }
}

/// # Safety
/// `source` must be valid UTF-8 and `source_utf16_len` must match its UTF-16 length.
#[cfg_attr(
    target_family = "wasm",
    bridge(ts_args_type = "source: Uint8Array, sourceUtf16Len: number, options: Uint8Array")
)]
pub unsafe fn transform_sync(
    source: &[u8],
    source_utf16_len: u32,
    options: &[u8],
) -> Result<Vec<u8>, String> {
    unsafe { transform_to_buffer(source, source_utf16_len, options) }
}
