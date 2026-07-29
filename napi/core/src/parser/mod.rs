use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, Lang as AstLang, SourceType as AstSourceType,
    ffi::ast::{buffer_size, serialize_into},
};
use swc_next_ecma_parser::parse;

use crate::parser::options::{Lang, ParserOptions, SourceType};

pub mod options;

#[napi]
pub fn parse_sync(source: String, options: Option<ParserOptions>) -> Buffer {
    let allocator = Allocator::default();
    let mut ast = Ast::new(source.as_bytes(), &allocator);
    let program = parse(
        &mut ast,
        source.as_bytes(),
        options.unwrap_or_default().into(),
    );

    let buffer_size = buffer_size(&ast);
    let mut buf = vec![0; buffer_size];
    serialize_into(program, &ast, &mut buf);
    Buffer::from(buf)
}

#[napi]
pub fn source_type_from_path(path: String) -> SourceType {
    AstSourceType::from_path(&path).into()
}

#[napi]
pub fn lang_from_path(path: String) -> Lang {
    AstLang::from_path(&path).into()
}
