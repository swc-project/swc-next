#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::time::{Duration, Instant};

use benchmark::allocator_buffer;
use criterion::{Criterion, criterion_group, criterion_main};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_parser::{Parser, StringSource};
use swc_experimental_ecma_transforms_base::remove_paren::remove_paren;

fn bench_transform(c: &mut Criterion) {
    let bench_cases = &[("typescript", include_str!("../files/typescript.js"))];
    let mut group = c.benchmark_group("semantic");
    for (name, source) in bench_cases {
        group.bench_function(format!("{name}/transform/remove_paren"), |b| {
            let mut buffer = allocator_buffer(source.len());
            let mut allocator = Allocator::new(&mut buffer);
            b.iter_custom(|iterations| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let input = StringSource::new(source);
                    let mut parser = Parser::new(
                        &allocator,
                        swc_experimental_ecma_parser::Syntax::Es(Default::default()),
                        input,
                        None,
                    );
                    let mut module = parser.parse_module().unwrap();
                    let start = Instant::now();
                    remove_paren(&mut module, &allocator, None);
                    elapsed += start.elapsed();
                    allocator.reset();
                }
                elapsed
            });
        });
    }
}

criterion_group!(transform, bench_transform);
criterion_main!(transform);
