use std::{env, fs::read_to_string, rc::Rc};

use swc_experimental_ecma_ast::{Ast, Visit};
use swc_experimental_ecma_parser::{EsSyntax, Parser, StringSource, Syntax};
use swc_experimental_ecma_semantic::resolver::{Semantic, resolver};

fn main() {
    let source = env::args()
        .nth(1)
        .map(|p| read_to_string(p).unwrap())
        .unwrap_or(include_str!("../files/typescript.js").to_owned());
    let syntax = Syntax::Es(EsSyntax::default());
    let input = StringSource::new(&source);

    let mut ast = Ast::new(input.source_len(), Rc::default());
    let mut parser = Parser::new(&mut ast, syntax, input, None);
    let root = parser.parse_program().unwrap();
    let semantic = resolver(root, &ast);

    println!("Top level: {:?}", semantic.top_level_scope_id());
    println!("Unresolved: {:?}", semantic.unresolved_scope_id());
    ScopeDisplayVisitor {
        ast: &ast,
        semantic: &semantic,
    }
    .visit_program(root);
}

struct ScopeDisplayVisitor<'a> {
    ast: &'a Ast,
    semantic: &'a Semantic,
}

impl Visit for ScopeDisplayVisitor<'_> {
    fn ast(&self) -> &Ast {
        self.ast
    }

    fn visit_ident(&mut self, node: swc_experimental_ecma_ast::Ident) {
        println!(
            "{} ({:?}) -> {:?}",
            self.ast.get_utf8(node.sym(self.ast)),
            node.sym(self.ast),
            self.semantic.node_scope(node),
        );
    }
}
