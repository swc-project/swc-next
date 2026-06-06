#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_parser::{Parser, StringSource};
use swc_experimental_ecma_transforms_base::remove_paren::remove_paren;

fn bench_transform(c: &mut Criterion) {
    let bench_cases = &[("typescript", include_str!("../files/typescript.js"))];
    let mut group = c.benchmark_group("semantic");
    for (name, source) in bench_cases {
        group.bench_function(format!("{name}/transform/remove_paren"), |b| {
            let allocator = Allocator::new();
            b.iter_batched(
                || {
                    let input = StringSource::new(source);
                    let mut parser = Parser::new(
                        &allocator,
                        swc_experimental_ecma_parser::Syntax::Es(Default::default()),
                        input,
                        None,
                    );
                    parser.parse_module().unwrap()
                },
                |mut module| {
                    remove_paren(&mut module, &allocator, None);
                },
                BatchSize::PerIteration,
            );
        });
    }
}

criterion_group!(transform, bench_transform);
criterion_main!(transform);
