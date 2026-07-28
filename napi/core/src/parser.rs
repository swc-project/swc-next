use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

#[napi(object)]
#[derive(Default)]
pub struct ParserOptions {
    pub source_type: String,
}

#[napi]
pub fn source_type_from_path(path: String) -> String {
    todo!()
}

#[napi]
pub fn lang_from_path(path: String) -> String {
    todo!()
}

#[napi]
pub fn parse_sync(source: String, options: Option<ParserOptions>) -> Buffer {
    todo!()
}
