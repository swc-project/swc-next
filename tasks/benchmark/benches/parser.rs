#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use criterion::{Bencher, Criterion, criterion_group, criterion_main};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::Ast;
use swc_experimental_ecma_parser::{Options, parse};

fn bench_new(b: &mut Bencher, src: &'static str) {
    b.iter(|| {
        let allocator = Allocator::new();
        let mut ast = Ast::new(src.as_bytes(), &allocator);
        parse(&mut ast, src.as_bytes(), Options::default()).unwrap();
    });
}

fn bench_files(c: &mut Criterion) {
    let bench_cases = &[
        ("typescript", include_str!("../files/typescript.js")),
        ("cal.com.tsx", include_str!("../files/cal.com.tsx")),
    ];

    for (name, source) in bench_cases {
        c.bench_function(&format!("{name}/parser/new"), |b| bench_new(b, source));
    }
}

criterion_group!(benches, bench_files);
criterion_main!(benches);
