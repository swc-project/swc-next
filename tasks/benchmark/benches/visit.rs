#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use criterion::{Bencher, Criterion, criterion_group, criterion_main};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, BindingIdentifier, IdentifierName, IdentifierReference, LabelIdentifier,
    PrivateIdentifier, Traverse, TraverseAction, TraverseCtx, TraverseWith, Visit, VisitMut,
    VisitMutWith, VisitWith,
};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

fn bench_visit(b: &mut Bencher, src: &'static str) {
    let allocator = Allocator::new();
    let mut ast = Ast::new(src.as_bytes(), &allocator);
    let program = Parser::init(&mut ast, Options::default(), NoTokenParserConfig)
        .parse()
        .program;

    struct Counter<'a> {
        ast: &'a Ast<'a>,
        count: usize,
    }

    impl<'a> Visit<'a> for Counter<'a> {
        fn ast(&self) -> &Ast<'a> {
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

    b.iter(|| {
        let mut counter = Counter {
            ast: &ast,
            count: 0,
        };
        program.visit_with(&mut counter);
        std::hint::black_box(counter.count);
    });
}

fn bench_visit_mut(b: &mut Bencher, src: &'static str) {
    struct Counter<'visit, 'ast> {
        ast: &'visit mut Ast<'ast>,
        count: usize,
    }

    impl<'ast> VisitMut<'ast> for Counter<'_, 'ast> {
        fn ast(&mut self) -> &mut Ast<'ast> {
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

    let allocator = Allocator::new();
    let mut ast = Ast::new(src.as_bytes(), &allocator);
    let program = Parser::init(&mut ast, Options::default(), NoTokenParserConfig)
        .parse()
        .program;

    b.iter(|| {
        let mut counter = Counter {
            ast: &mut ast,
            count: 0,
        };
        program.visit_mut_with(&mut counter);
        std::hint::black_box(counter.count);
    });
}

fn bench_traverse(b: &mut Bencher, src: &'static str) {
    struct Counter {
        count: usize,
    }

    impl<'ast> Traverse<'ast> for Counter {
        fn enter_identifier_reference(
            &mut self,
            _node: IdentifierReference,
            _ctx: &mut TraverseCtx<'_, 'ast>,
        ) -> TraverseAction {
            self.count += 1;
            TraverseAction::Proceed
        }

        fn enter_private_identifier(
            &mut self,
            _node: PrivateIdentifier,
            _ctx: &mut TraverseCtx<'_, 'ast>,
        ) -> TraverseAction {
            self.count += 1;
            TraverseAction::Proceed
        }

        fn enter_binding_identifier(
            &mut self,
            _node: BindingIdentifier,
            _ctx: &mut TraverseCtx<'_, 'ast>,
        ) -> TraverseAction {
            self.count += 1;
            TraverseAction::Proceed
        }

        fn enter_identifier_name(
            &mut self,
            _node: IdentifierName,
            _ctx: &mut TraverseCtx<'_, 'ast>,
        ) -> TraverseAction {
            self.count += 1;
            TraverseAction::Proceed
        }

        fn enter_label_identifier(
            &mut self,
            _node: LabelIdentifier,
            _ctx: &mut TraverseCtx<'_, 'ast>,
        ) -> TraverseAction {
            self.count += 1;
            TraverseAction::Proceed
        }
    }

    let allocator = Allocator::new();
    let mut ast = Ast::new(src.as_bytes(), &allocator);
    let program = Parser::init(&mut ast, Options::default(), NoTokenParserConfig)
        .parse()
        .program;

    b.iter(|| {
        let mut counter = Counter { count: 0 };
        let _ = program.traverse_with(&mut counter, &mut TraverseCtx::new(&mut ast, ()));
        std::hint::black_box(counter.count);
    });
}

fn bench_files(c: &mut Criterion) {
    let bench_cases = &[
        ("typescript", include_str!("../files/typescript.js")),
        ("cal.com.tsx", include_str!("../files/cal.com.tsx")),
    ];

    for (name, source) in bench_cases {
        c.bench_function(&format!("{name}/visit/new"), |b| bench_visit(b, source));
        c.bench_function(&format!("{name}/visit_mut/new"), |b| {
            bench_visit_mut(b, source)
        });
        c.bench_function(&format!("{name}/traverse/new"), |b| {
            bench_traverse(b, source)
        });
    }
}

criterion_group!(benches, bench_files);
criterion_main!(benches);
