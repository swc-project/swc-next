use swc_next_bridge::bridge;
use swc_next_core::js::parser::{
    JsLang, JsParserOptions as ParserOptions, JsSourceType, parse_to_buffer,
};
use swc_next_ecma_ast::{Lang, SourceType};

#[bridge]
/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
/// `source_utf16_len` must match the UTF-16 code unit length of `source`.
pub unsafe fn parse_sync(
    source: &[u8],
    source_utf16_len: u32,
    options: Option<ParserOptions>,
) -> Vec<u8> {
    unsafe { parse_to_buffer(source, source_utf16_len, options) }
}

#[bridge(ts_return_type = "SourceType")]
pub fn source_type_from_path(path: String) -> JsSourceType {
    SourceType::from_path(&path).into()
}

#[bridge(ts_return_type = "Lang")]
pub fn lang_from_path(path: String) -> JsLang {
    Lang::from_path(&path).into()
}
