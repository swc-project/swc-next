#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Ast, Program};
use swc_experimental_ecma_parser::{Parser, StringSource};
use swc_experimental_ecma_semantic::resolver::resolver;

fn bench_semantic(c: &mut Criterion) {
    let bench_cases = &[("typescript", include_str!("../files/typescript.js"))];
    let mut group = c.benchmark_group("semantic");
    for (name, source) in bench_cases {
        group.bench_function(format!("{name}/semantic/legacy"), |b| {
            let allocator = Allocator::new();
            b.iter_batched(
                || {
                    let input = StringSource::new(source);
                    let mut ast = Ast::new(input.source_len(), &allocator);
                    let module = Parser::new(
                        &mut ast,
                        swc_experimental_ecma_parser::Syntax::Es(Default::default()),
                        input,
                        None,
                    )
                    .parse_module()
                    .unwrap();
                    (module, ast)
                },
                |(module, mut ast)| {
                    black_box(resolver(Program::Module(module), &mut ast));
                },
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group!(semantic, bench_semantic);
criterion_main!(semantic);
