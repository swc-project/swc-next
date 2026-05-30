use std::{env, fs::read_to_string};

use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Ident, Visit};
use swc_experimental_ecma_parser::{EsSyntax, Parser, StringSource, Syntax};
use swc_experimental_ecma_semantic::resolver::{Semantic, resolver};

fn main() {
    let source = env::args()
        .nth(1)
        .map(|p| read_to_string(p).unwrap())
        .unwrap_or(include_str!("../files/typescript.js").to_owned());
    let syntax = Syntax::Es(EsSyntax::default());
    let allocator = Allocator::new();
    let input = StringSource::new(&source);

    let mut parser = Parser::new(&allocator, syntax, input, None);
    let root = parser.parse_program().unwrap();
    let semantic = resolver(&root);

    println!("Top level: {:?}", semantic.top_level_scope_id());
    println!("Unresolved: {:?}", semantic.unresolved_scope_id());
    ScopeDisplayVisitor {
        semantic: &semantic,
    }
    .visit_program(&root);
}

struct ScopeDisplayVisitor<'a> {
    semantic: &'a Semantic,
}

impl<'a> Visit<'a> for ScopeDisplayVisitor<'_> {
    fn visit_ident(&mut self, node: &Ident) {
        println!(
            "{} ({:?}) -> {:?}",
            node.sym,
            node.sym,
            self.semantic.node_scope(node),
        );
    }
}
