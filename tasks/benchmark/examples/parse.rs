use std::{env, fs::read_to_string};

use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_parser::{EsSyntax, Parser, StringSource, Syntax};

fn main() {
    let source = env::args()
        .nth(1)
        .map(|p| read_to_string(p).unwrap())
        .unwrap_or(include_str!("../files/typescript.js").to_owned());
    let syntax = Syntax::Es(EsSyntax::default());
    let allocator = Allocator::new();
    let input = StringSource::new(&source);

    let mut parser = Parser::new(&allocator, syntax, input, None);
    let _root = parser.parse_program().unwrap();
}
