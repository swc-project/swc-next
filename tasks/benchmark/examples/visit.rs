use swc_core::common::BytePos;
use swc_experimental_ecma_ast::StringAllocator;

fn test_legacy(src: &str) -> usize {
    use swc_core::ecma::parser::{Parser, StringInput, Syntax, lexer::Lexer};
    use swc_core::ecma::visit::VisitWith;

    let input = StringInput::new(src, BytePos(0), BytePos(src.len() as u32));
    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        input,
        None,
    );
    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module().unwrap();

    struct Counter {
        count: usize,
    }

    impl swc_core::ecma::visit::Visit for Counter {
        fn visit_ident(&mut self, _node: &swc_core::ecma::ast::Ident) {
            self.count += 1;
        }
    }

    let mut counter = Counter { count: 0 };
    module.visit_with(&mut counter);
    counter.count
}

fn test_new(src: &str) -> usize {
    use swc_experimental_ecma_ast::Ast;
    use swc_experimental_ecma_parser::{Parser, StringSource};
    use swc_experimental_ecma_visit::VisitWith;

    let input = StringSource::new(src);
    let mut ast = Ast::new(input.source_len(), StringAllocator::default());
    let mut parser = Parser::new(
        &mut ast,
        swc_experimental_ecma_parser::Syntax::Es(Default::default()),
        input,
        None,
    );
    let module = parser.parse_module().unwrap();

    struct Counter<'a> {
        ast: &'a Ast,
        count: usize,
    }

    impl swc_experimental_ecma_visit::Visit for Counter<'_> {
        fn ast(&self) -> &swc_experimental_ecma_ast::Ast {
            self.ast
        }

        fn visit_ident(&mut self, _node: swc_experimental_ecma_ast::Ident) {
            self.count += 1;
        }
    }

    let mut counter = Counter {
        ast: &mut ast,
        count: 0,
    };
    module.visit_with(&mut counter);
    counter.count
}

fn test_new_mut(src: &str) -> usize {
    use swc_experimental_ecma_ast::Ast;
    use swc_experimental_ecma_parser::{Parser, StringSource};
    use swc_experimental_ecma_visit::VisitMutWith;

    let input = StringSource::new(src);
    let mut ast = Ast::new(input.source_len(), StringAllocator::default());
    let mut parser = Parser::new(
        &mut ast,
        swc_experimental_ecma_parser::Syntax::Es(Default::default()),
        input,
        None,
    );
    let root = parser.parse_module().unwrap();

    struct Counter<'a> {
        ast: &'a mut Ast,
        count: usize,
    }

    impl swc_experimental_ecma_visit::VisitMut for Counter<'_> {
        fn ast(&mut self) -> &mut swc_experimental_ecma_ast::Ast {
            self.ast
        }

        fn visit_mut_ident(
            &mut self,
            node: swc_experimental_ecma_ast::Ident,
        ) -> swc_experimental_ecma_ast::Ident {
            self.count += 1;
            node
        }
    }

    let mut counter = Counter {
        ast: &mut ast,
        count: 0,
    };
    root.visit_mut_with(&mut counter);
    counter.count
}

fn main() {
    let source = include_str!("../files/typescript.js");
    let legacy = test_legacy(source);
    let new = test_new(source);
    let new_mut = test_new_mut(source);

    assert_eq!(legacy, new);
    assert_eq!(legacy, new_mut);
}
