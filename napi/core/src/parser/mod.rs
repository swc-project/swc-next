use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast,
    ffi::ast::{buffer_size, serialize_into},
};
use swc_next_ecma_parser::parse;

use crate::parser::options::ParserOptions;

pub mod options;

#[napi]
pub fn parse_sync(source: String, options: Option<ParserOptions>) -> Buffer {
    let allocator = Allocator::default();
    let mut ast = Ast::new(source.as_bytes(), &allocator);
    parse(
        &mut ast,
        source.as_bytes(),
        options.unwrap_or_default().into(),
    );

    let buffer_size = buffer_size(&ast);
    let mut buf = Vec::with_capacity(buffer_size);
    serialize_into(&ast, &mut buf);
    Buffer::from(buf.as_ref())
}
