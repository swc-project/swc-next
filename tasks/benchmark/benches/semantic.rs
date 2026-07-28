#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::Ast;
use swc_next_ecma_parser::{Options, parse};
use swc_next_ecma_semantic::analyze;

fn bench_semantic(c: &mut Criterion) {
    let bench_cases = &[
        ("typescript", include_str!("../files/typescript.js")),
        ("cal.com.tsx", include_str!("../files/cal.com.tsx")),
    ];
    let mut group = c.benchmark_group("semantic");
    for (name, source) in bench_cases {
        group.bench_function(format!("{name}/semantic/legacy"), |b| {
            let allocator = Allocator::new();
            b.iter_batched(
                || {
                    let mut ast = Ast::new(source.as_bytes(), &allocator);
                    let program = parse(&mut ast, source.as_bytes(), Options::default());
                    (program, ast)
                },
                |(program, mut ast)| {
                    black_box(analyze(program, &mut ast));
                },
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group!(semantic, bench_semantic);
criterion_main!(semantic);
