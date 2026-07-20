#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use criterion::{Bencher, Criterion, criterion_group, criterion_main};
use swc_experimental_allocator::Allocator;

fn bench_new(b: &mut Bencher, src: &'static str) {
    use swc_experimental_ecma_ast::{Ast, NodeKind};
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

        fn enter_node(&mut self, node_id: swc_experimental_ecma_ast::NodeId) {
            if matches!(
                self.ast.node_kind(node_id),
                NodeKind::IdentifierReference
                    | NodeKind::PrivateIdentifier
                    | NodeKind::BindingIdentifier
                    | NodeKind::IdentifierName
                    | NodeKind::LabelIdentifier
            ) {
                self.count += 1;
            }
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

fn bench_new_mut(b: &mut Bencher, src: &'static str) {
    use swc_experimental_ecma_ast::{
        Ast, BindingIdentifier, IdentifierName, IdentifierReference, LabelIdentifier,
        PrivateIdentifier,
    };
    use swc_experimental_ecma_parser::{Options, parse};
    use swc_experimental_ecma_visit::VisitMutWith;

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

    let allocator = Allocator::new();
    let mut ast = Ast::new(src.as_bytes(), &allocator);
    let program = parse(&mut ast, src.as_bytes(), Options::default()).unwrap();

    b.iter(|| {
        let mut counter = Counter {
            ast: &mut ast,
            count: 0,
        };
        program.visit_mut_with(&mut counter);
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
        c.bench_function(&format!("{name}/visit/new"), |b| bench_new(b, source));
        c.bench_function(&format!("{name}/visit_mut/new"), |b| {
            bench_new_mut(b, source)
        });
    }
}

criterion_group!(benches, bench_files);
criterion_main!(benches);
