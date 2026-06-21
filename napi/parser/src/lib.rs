use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::Encoder;
use swc_experimental_ecma_parser::{Parser, StringSource};

#[napi(object)]
pub struct ParseOptions {}

#[napi]
pub fn parse_sync<'a>(source: String, _options: ParseOptions) -> Buffer {
    let allocator = Allocator::new();
    let mut parser = Parser::new(
        &allocator,
        swc_experimental_ecma_parser::Syntax::Es(Default::default()),
        StringSource::new(&source),
        None,
    );
    let program = parser.parse_program().unwrap();

    let mut buf = Vec::with_capacity(1024 * 1024);
    let header = Encoder::header(&program);
    let mut encoder = Encoder::new(header, &mut buf).unwrap();
    encoder.encode(&program);
    Buffer::from(buf)
}
