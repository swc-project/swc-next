#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::Program;
use swc_experimental_ecma_parser::{Parser, StringSource};
use swc_experimental_ecma_semantic::resolver::resolver;

fn bench_semantic(c: &mut Criterion) {
    let bench_cases = &[("typescript", include_str!("../files/typescript.js"))];
    let mut group = c.benchmark_group("semantic");
    for (name, source) in bench_cases {
        group.bench_function(format!("{name}/semantic/legacy"), |b| {
            let mut allocator = Allocator::new();
            b.iter(|| {
                let semantic = {
                    let input = StringSource::new(source);
                    let mut parser = Parser::new(
                        &allocator,
                        swc_experimental_ecma_parser::Syntax::Es(Default::default()),
                        input,
                        None,
                    );
                    let ret = parser.parse_module().unwrap();
                    let ret = Program::Module(allocator.boxed(ret));
                    resolver(&ret)
                };

                black_box(semantic);
                allocator.reset();
            });
        });
    }
}

criterion_group!(semantic, bench_semantic);
criterion_main!(semantic);
