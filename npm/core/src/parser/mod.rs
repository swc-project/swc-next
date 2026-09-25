use swc_next_bridge::{Lang, ParserOptions, SourceType, bridge, parse_to_buffer};
use swc_next_ecma_ast::{Lang as AstLang, SourceType as AstSourceType};

#[bridge]
/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
pub unsafe fn parse_sync(source: &[u8], options: Option<ParserOptions>) -> Vec<u8> {
    unsafe { parse_to_buffer(source, options) }
}

#[bridge]
pub fn source_type_from_path(path: String) -> SourceType {
    AstSourceType::from_path(&path).into()
}

#[bridge]
pub fn lang_from_path(path: String) -> Lang {
    AstLang::from_path(&path).into()
}
