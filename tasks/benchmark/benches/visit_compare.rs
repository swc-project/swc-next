#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use criterion::{Bencher, Criterion, criterion_group, criterion_main};
use swc_core::common::BytePos;

fn bench_legacy(b: &mut Bencher, src: &'static str) {
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

    b.iter(|| {
        let mut counter = Counter { count: 0 };
        module.visit_with(&mut counter);
        std::hint::black_box(counter.count);
    });
}

fn bench_new(b: &mut Bencher, src: &'static str) {
    use swc_experimental_ecma_ast::Ast;
    use swc_experimental_ecma_parser::{Lexer, Parser, StringSource};
    use swc_experimental_ecma_visit::VisitWith;

    let input = StringSource::new(src);
    let lexer = Lexer::new(
        swc_experimental_ecma_parser::Syntax::Es(Default::default()),
        Default::default(),
        input,
        None,
    );
    let parser = Parser::new_from(lexer);
    let ret = parser.parse_module().unwrap();

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

    b.iter(|| {
        let mut counter = Counter {
            ast: &ret.ast,
            count: 0,
        };
        ret.root.visit_with(&mut counter);
        std::hint::black_box(counter.count);
    });
}

fn bench_legacy_mut(b: &mut Bencher, src: &'static str) {
    use swc_core::ecma::parser::{Parser, StringInput, Syntax, lexer::Lexer};
    use swc_core::ecma::visit::VisitMutWith;

    struct Counter {
        count: usize,
    }

    impl swc_core::ecma::visit::VisitMut for Counter {
        fn visit_mut_ident(&mut self, _node: &mut swc_core::ecma::ast::Ident) {
            self.count += 1;
        }
    }

    let input = StringInput::new(src, BytePos(0), BytePos(src.len() as u32));
    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        input,
        None,
    );
    let mut parser = Parser::new_from(lexer);
    let mut module = parser.parse_module().unwrap();

    b.iter(|| {
        let mut counter = Counter { count: 0 };
        module.visit_mut_with(&mut counter);
        std::hint::black_box(counter.count);
    });
}

fn bench_new_mut(b: &mut Bencher, src: &'static str) {
    use swc_experimental_ecma_ast::Ast;
    use swc_experimental_ecma_parser::{Lexer, Parser, StringSource};
    use swc_experimental_ecma_visit::VisitMutWith;

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

    let input = StringSource::new(src);
    let lexer = Lexer::new(
        swc_experimental_ecma_parser::Syntax::Es(Default::default()),
        Default::default(),
        input,
        None,
    );
    let parser = Parser::new_from(lexer);
    let mut ret = parser.parse_module().unwrap();

    b.iter(|| {
        let mut counter = Counter {
            ast: &mut ret.ast,
            count: 0,
        };
        ret.root.visit_mut_with(&mut counter);
        std::hint::black_box(counter.count);
    });
}

fn bench_files(c: &mut Criterion) {
    let bench_cases = &[
        ("colors", include_str!("../files/colors.js")),
        ("angular", include_str!("../files/angular-1.2.5.js")),
        ("backbone", include_str!("../files/backbone-1.1.0.js")),
        ("jquery", include_str!("../files/jquery-1.9.1.js")),
        (
            "jquery.mobile",
            include_str!("../files/jquery.mobile-1.4.2.js"),
        ),
        ("mootools", include_str!("../files/mootools-1.4.5.js")),
        ("underscore", include_str!("../files/underscore-1.5.2.js")),
        ("three", include_str!("../files/three-0.138.3.js")),
        ("yui", include_str!("../files/yui-3.12.0.js")),
        ("typescript", include_str!("../files/typescript.js")),
    ];

    for (name, source) in bench_cases {
        c.bench_function(&format!("{name}/visit/legacy"), |b| bench_legacy(b, source));
        c.bench_function(&format!("{name}/visit/new"), |b| bench_new(b, source));

        c.bench_function(&format!("{name}/visit_mut/legacy"), |b| {
            bench_legacy_mut(b, source)
        });
        c.bench_function(&format!("{name}/visit_mut/new"), |b| {
            bench_new_mut(b, source)
        });
    }
}

criterion_group!(benches, bench_files);
criterion_main!(benches);
