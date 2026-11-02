use swc_next_bridge::{Lang, ParserOptions, SourceType, bridge, parse_to_buffer};
use swc_next_ecma_ast::{Lang as AstLang, SourceType as AstSourceType};

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

#[bridge]
pub fn source_type_from_path(path: String) -> SourceType {
    AstSourceType::from_path(&path).into()
}

#[bridge]
pub fn lang_from_path(path: String) -> Lang {
    AstLang::from_path(&path).into()
}
