use swc_core::common::BytePos;
use swc_experimental_allocator::Allocator;

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
    use swc_experimental_ecma_ast::{
        Ast, BindingIdentifier, IdentifierName, IdentifierReference, LabelIdentifier,
        PrivateIdentifier,
    };
    use swc_experimental_ecma_parser::{Options, parse};
    use swc_experimental_ecma_visit::VisitWith;

    let allocator = Allocator::new();
    let mut ast = Ast::new(src.as_bytes(), &allocator);
    let program = parse(&mut ast, src.as_bytes(), Options::default()).unwrap();

    struct Counter<'visit, 'ast> {
        ast: &'visit Ast<'ast>,
        count: usize,
    }

    impl<'ast> swc_experimental_ecma_visit::Visit<'ast> for Counter<'_, 'ast> {
        fn ast(&self) -> &swc_experimental_ecma_ast::Ast<'ast> {
            self.ast
        }

        fn visit_identifier_reference(&mut self, _node: IdentifierReference) {
            self.count += 1;
        }

        fn visit_private_identifier(&mut self, _node: PrivateIdentifier) {
            self.count += 1;
        }

        fn visit_binding_identifier(&mut self, _node: BindingIdentifier) {
            self.count += 1;
        }

        fn visit_identifier_name(&mut self, _node: IdentifierName) {
            self.count += 1;
        }

        fn visit_label_identifier(&mut self, _node: LabelIdentifier) {
            self.count += 1;
        }
    }

    let mut counter = Counter {
        ast: &ast,
        count: 0,
    };
    program.visit_with(&mut counter);
    counter.count
}

fn test_new_mut(src: &str) -> usize {
    use swc_experimental_ecma_ast::{
        Ast, BindingIdentifier, IdentifierName, IdentifierReference, LabelIdentifier,
        PrivateIdentifier,
    };
    use swc_experimental_ecma_parser::{Options, parse};
    use swc_experimental_ecma_visit::VisitMutWith;

    let allocator = Allocator::new();
    let mut ast = Ast::new(src.as_bytes(), &allocator);
    let root = parse(&mut ast, src.as_bytes(), Options::default()).unwrap();

    struct Counter<'visit, 'ast> {
        ast: &'visit mut Ast<'ast>,
        count: usize,
    }

    impl<'ast> swc_experimental_ecma_visit::VisitMut<'ast> for Counter<'_, 'ast> {
        fn ast(&mut self) -> &mut swc_experimental_ecma_ast::Ast<'ast> {
            self.ast
        }

        fn visit_mut_identifier_reference(&mut self, _node: IdentifierReference) {
            self.count += 1;
        }

        fn visit_mut_private_identifier(&mut self, _node: PrivateIdentifier) {
            self.count += 1;
        }

        fn visit_mut_binding_identifier(&mut self, _node: BindingIdentifier) {
            self.count += 1;
        }

        fn visit_mut_identifier_name(&mut self, _node: IdentifierName) {
            self.count += 1;
        }

        fn visit_mut_label_identifier(&mut self, _node: LabelIdentifier) {
            self.count += 1;
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
