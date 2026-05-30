#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use criterion::{Bencher, Criterion, criterion_group, criterion_main};
use swc_experimental_ecma_ast::{Ast, StringAllocator};
use swc_experimental_ecma_parser::StringSource;

fn bench_new(b: &mut Bencher, src: &'static str) {
    use swc_experimental_ecma_parser::Parser;
    b.iter(|| {
        let input = StringSource::new(src);
        let mut ast = Ast::new(input.source_len(), StringAllocator::default());
        let mut parser = Parser::new(
            &mut ast,
            swc_experimental_ecma_parser::Syntax::Es(Default::default()),
            input,
            None,
        );
        parser.parse_module().unwrap();
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
        c.bench_function(&format!("{name}/parser/new"), |b| bench_new(b, source));
    }
}

criterion_group!(benches, bench_files);
criterion_main!(benches);
