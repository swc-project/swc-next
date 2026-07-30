use std::{env, fs::read_to_string};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::Ast;
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

fn main() {
    let source = env::args()
        .nth(1)
        .map(|p| read_to_string(p).unwrap())
        .unwrap_or(include_str!("../files/typescript.js").to_owned());
    let allocator = Allocator::new();
    let mut ast = Ast::new(source.as_bytes(), &allocator);
    let _return = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
}
