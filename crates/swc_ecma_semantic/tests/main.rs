use std::{fmt::Write, fs, rc::Rc};

use swc_experimental_ecma_ast::{Ast, Visit};
use swc_experimental_ecma_parser::{EsSyntax, Parser, StringSource, Syntax};
use swc_experimental_ecma_semantic::resolver::{Semantic, resolver};

struct ScopeDisplayVisitor<'a> {
    ast: &'a Ast,
    semantic: &'a Semantic,
    output: String,
}

impl Visit for ScopeDisplayVisitor<'_> {
    fn ast(&self) -> &Ast {
        self.ast
    }

    fn visit_ident(&mut self, node: swc_experimental_ecma_ast::Ident) {
        let _ = writeln!(
            self.output,
            "{} ({:?}) -> {:?}",
            self.ast.get_utf8(node.sym(self.ast)),
            node.sym(self.ast),
            self.semantic.node_scope(node),
        );
    }
}

#[test]
fn main() {
    insta::glob!("fixtures/**/*.{js,jsx,ts,tsx}", |path| {
        let source_text = fs::read_to_string(path).unwrap();

        let syntax = Syntax::Es(EsSyntax::default());
        let input = StringSource::new(&source_text);

        let mut ast = Ast::new(input.source_len(), Rc::default());
        let mut parser = Parser::new(&mut ast, syntax, input, None);
        let root = parser.parse_program().unwrap();
        let semantic = resolver(root, &ast);

        let mut visitor = ScopeDisplayVisitor {
            ast: &ast,
            semantic: &semantic,
            output: String::new(),
        };

        let _ = writeln!(
            visitor.output,
            "Top level: {:?}",
            semantic.top_level_scope_id()
        );
        let _ = writeln!(
            visitor.output,
            "Unresolved: {:?}",
            semantic.unresolved_scope_id()
        );
        visitor.visit_program(root);

        let name = path.file_stem().unwrap().to_str().unwrap();

        insta::with_settings!({ snapshot_path => path.parent().unwrap(), prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
            insta::assert_snapshot!(name, visitor.output);
        });
    });
}
