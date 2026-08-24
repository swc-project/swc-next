use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use swc_next_ecma_ast::{Lang as AstLang, SourceType as AstSourceType};
use swc_next_napi::{
    FragmentContext, Lang, ParserOptions, SourceType, parse_expression_fragment_to_buffer,
    parse_statement_fragment_to_buffer, parse_to_buffer,
};

#[napi]
pub fn parse_sync(source: String, options: Option<ParserOptions>) -> Buffer {
    Buffer::from(parse_to_buffer(&source, options))
}

#[napi]
pub fn parse_expression_fragment_sync(
    source: String,
    fragment_context: FragmentContext,
    options: Option<ParserOptions>,
) -> napi::Result<Buffer> {
    parse_expression_fragment_to_buffer(&source, fragment_context, options).map(Buffer::from)
}

#[napi]
pub fn parse_statement_fragment_sync(
    source: String,
    fragment_context: FragmentContext,
    options: Option<ParserOptions>,
) -> napi::Result<Buffer> {
    parse_statement_fragment_to_buffer(&source, fragment_context, options).map(Buffer::from)
}

#[napi]
pub fn source_type_from_path(path: String) -> SourceType {
    AstSourceType::from_path(&path).into()
}

#[napi]
pub fn lang_from_path(path: String) -> Lang {
    AstLang::from_path(&path).into()
}
