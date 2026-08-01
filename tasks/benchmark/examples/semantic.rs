use std::{env, fs::read_to_string};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, IdentifierReference, Visit, VisitWith};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{Semantic, analyze};

fn main() {
    let source = env::args()
        .nth(1)
        .map(|p| read_to_string(p).unwrap())
        .unwrap_or(include_str!("../files/typescript.js").to_owned());
    let allocator = Allocator::new();
    let mut ast = Ast::new(source.as_bytes(), &allocator);
    let root = Parser::init(&mut ast, Options::default(), NoTokenParserConfig)
        .parse()
        .program;
    let semantic = analyze(root, &mut ast).semantic;
    root.visit_with(&mut ScopeDisplayVisitor {
        ast: &ast,
        semantic: &semantic,
    });
}

struct ScopeDisplayVisitor<'a> {
    ast: &'a Ast<'a>,
    semantic: &'a Semantic<'a>,
}

impl<'a> Visit<'a> for ScopeDisplayVisitor<'a> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }

    fn visit_identifier_reference(&mut self, node: IdentifierReference) {
        println!(
            "{} ({:?}) -> {:?}",
            String::from_utf8_lossy(self.ast.string(node.name(self.ast))),
            node.name(self.ast),
            self.semantic.scope_of(node.node_id()),
        );
    }
}
