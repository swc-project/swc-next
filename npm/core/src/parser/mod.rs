use swc_next::js::parser::{JsLang, JsSourceType};
#[cfg(target_family = "wasm")]
use swc_next::js::parser::{JsParserConfig as ParserConfig, bridge::parse_to_buffer};
use swc_next_bridge::bridge;
#[cfg(not(target_family = "wasm"))]
use swc_next_bridge::{napi, napi_derive::napi};
use swc_next_ecma_ast::{Lang, SourceType};

#[cfg(target_family = "wasm")]
#[bridge(ts_args_type = "source: Uint8Array, sourceUtf16Len: number, options: number")]
/// Internal binding API and binary protocol, not covered by semver. Use a matching SWC Next wrapper.
///
/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
/// `source_utf16_len` must match the UTF-16 code unit length of `source`.
pub unsafe fn parse_sync(
    source: &[u8],
    source_utf16_len: u32,
    options: Option<ParserConfig>,
) -> Vec<u8> {
    unsafe { parse_to_buffer(source, source_utf16_len, options) }
}

#[cfg(not(target_family = "wasm"))]
#[napi(
    js_name = "parseSync",
    ts_args_type = "source: Uint8Array, sourceUtf16Len: number, options: number",
    ts_return_type = "Uint8Array"
)]
/// Internal binding API and binary protocol, not covered by semver. Use a matching SWC Next wrapper.
///
/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
/// `source_utf16_len` must match the UTF-16 code unit length of `source`.
pub unsafe fn parse_sync(
    env: napi::Env,
    source: swc_next_bridge::NativeBytes<'_>,
    source_utf16_len: u32,
    options: u32,
) -> napi::Result<napi::bindgen_prelude::Buffer> {
    unsafe {
        swc_next::js::parser::bridge::parse_to_node_buffer(&env, &source, source_utf16_len, options)
    }
}

#[bridge(ts_return_type = "SourceType")]
pub fn source_type_from_path(path: String) -> JsSourceType {
    SourceType::from_path(&path).into()
}

#[bridge(ts_return_type = "Lang")]
pub fn lang_from_path(path: String) -> JsLang {
    Lang::from_path(&path).into()
}

#[cfg(target_family = "wasm")]
#[bridge(ts_args_type = "source: Uint8Array, sourceUtf16Len: number, options: number")]
/// Internal binding API and binary protocol, not covered by semver. Use a matching SWC Next wrapper.
///
/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8 and that
/// `source_utf16_len` is its UTF-16 code unit length.
pub unsafe fn parse_sync2(
    source: &[u8],
    source_utf16_len: u32,
    options: Option<ParserConfig>,
) -> Vec<u8> {
    unsafe { swc_next::js::parser::bridge::parse_to_buffer2(source, source_utf16_len, options) }
}

#[cfg(not(target_family = "wasm"))]
#[napi(
    js_name = "parseSync2",
    ts_args_type = "source: Uint8Array, sourceUtf16Len: number, options: number",
    ts_return_type = "Uint8Array"
)]
/// Internal binding API and binary protocol, not covered by semver. Use a matching SWC Next wrapper.
///
/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8 and that
/// `source_utf16_len` is its UTF-16 code unit length.
pub unsafe fn parse_sync2(
    source: swc_next_bridge::NativeBytes<'_>,
    source_utf16_len: u32,
    options: u32,
) -> napi::Result<napi::bindgen_prelude::Buffer> {
    unsafe {
        swc_next::js::parser::bridge::parse_to_node_buffer2(&source, source_utf16_len, options)
    }
}
