use std::{fmt::Write, fs};

use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{EsVersion, Ident, Visit};
use swc_experimental_ecma_parser::{EsSyntax, Syntax, parse_file_as_program};
use swc_experimental_ecma_semantic::resolver::{Semantic, resolver};

struct ScopeDisplayVisitor<'a, 'b> {
    semantic: &'a Semantic,
    output: &'b mut String,
}

impl<'a> Visit<'a> for ScopeDisplayVisitor<'_, '_> {
    fn visit_ident(&mut self, node: &Ident) {
        let _ = writeln!(
            self.output,
            "{} ({:?}) -> {:?}",
            node.sym,
            node.sym,
            self.semantic.node_scope(node),
        );
    }
}

#[test]
fn snapshot() {
    insta::glob!("fixtures/**/*.{js,jsx,ts,tsx}", |path| {
        let source_text = fs::read_to_string(path).unwrap();

        let syntax = Syntax::Es(EsSyntax::default());

        let bump = Allocator::new();
        let root =
            parse_file_as_program(&bump, &source_text, syntax, EsVersion::EsNext, None).unwrap();
        let semantic = resolver(&root);

        let mut output = String::new();
        let mut visitor = ScopeDisplayVisitor {
            semantic: &semantic,
            output: &mut output,
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
        visitor.visit_program(&root);

        let name = path.file_stem().unwrap().to_str().unwrap();

        insta::with_settings!({ snapshot_path => path.parent().unwrap(), prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
            insta::assert_snapshot!(name, output);
        });
    });
}
