use std::{fmt::Write, fs};

use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Ast, EsVersion, Ident, Visit, VisitWith};
use swc_experimental_ecma_parser::{EsSyntax, Syntax, parse_file_as_program};
use swc_experimental_ecma_semantic::resolver::{Semantic, resolver};

struct ScopeDisplayVisitor<'visit, 'ast, 'out> {
    ast: &'visit Ast<'ast>,
    semantic: &'visit Semantic,
    output: &'out mut String,
}

impl<'ast> Visit<'ast> for ScopeDisplayVisitor<'_, 'ast, '_> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_ident(&mut self, node: Ident) {
        let sym = node.sym(self.ast);
        let _ = writeln!(
            self.output,
            "{} ({:?}) -> {:?}",
            sym.as_str(self.ast),
            sym.as_str(self.ast),
            self.semantic.node_scope(node, self.ast),
        );
    }
}

#[test]
fn snapshot() {
    insta::glob!("fixtures/**/*.{js,jsx,ts,tsx}", |path| {
        let source_text = fs::read_to_string(path).unwrap();

        let syntax = Syntax::Es(EsSyntax::default());

        let bump = Allocator::new();
        let mut ast = Ast::new(source_text.len(), &bump);
        let root =
            parse_file_as_program(&mut ast, &source_text, syntax, EsVersion::EsNext, None).unwrap();
        let semantic = resolver(root, &mut ast);

        let mut output = String::new();
        let mut visitor = ScopeDisplayVisitor {
            ast: &ast,
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
        root.visit_with(&mut visitor);

        let name = path.file_stem().unwrap().to_str().unwrap();

        insta::with_settings!({ snapshot_path => path.parent().unwrap(), prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
            insta::assert_snapshot!(name, output);
        });
    });
}

struct MetadataVisitor<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    ident_count: usize,
    block_count: usize,
}

impl<'ast> Visit<'ast> for MetadataVisitor<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_ident(&mut self, node: Ident) {
        assert!(node.symbol_id(self.ast).is_some());
        self.ident_count += 1;
    }

    fn visit_block_stmt(&mut self, node: swc_experimental_ecma_ast::BlockStmt) {
        assert!(node.scope_id(self.ast).is_some());
        self.block_count += 1;
        node.visit_children_with(self);
    }
}

#[test]
fn writes_symbol_and_scope_ids_to_ast() {
    let source_text = "{ let value = 1; value; }";
    let allocator = Allocator::new();
    let mut ast = Ast::new(source_text.len(), &allocator);
    let root = parse_file_as_program(
        &mut ast,
        source_text,
        Syntax::Es(EsSyntax::default()),
        EsVersion::EsNext,
        None,
    )
    .unwrap();

    resolver(root, &mut ast);

    let mut visitor = MetadataVisitor {
        ast: &ast,
        ident_count: 0,
        block_count: 0,
    };
    root.visit_with(&mut visitor);
    assert!(visitor.ident_count >= 2);
    assert_eq!(visitor.block_count, 1);
}
