use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Encoder, Header};
use swc_experimental_ecma_parser::{Parser, StringSource};

#[napi(object)]
pub struct ParseOptions {}

#[napi]
pub fn parse_sync<'a>(source: String, _options: Option<ParseOptions>) -> Buffer {
    let allocator = Allocator::new();
    let mut parser = Parser::new(
        &allocator,
        swc_experimental_ecma_parser::Syntax::Es(Default::default()),
        StringSource::new(&source),
        None,
    );
    let program = parser.parse_program().unwrap();

    let header = Header::measure(&program, &source);
    let mut buf = vec![0; header.size()];
    let mut encoder = Encoder::new(header, &source, &mut buf).unwrap();
    encoder.encode(&program);
    Buffer::from(buf)
}
