#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use swc_experimental_ecma_ast::{Ast, StringAllocator};
use swc_experimental_ecma_parser::{Parser, StringSource};
use swc_experimental_ecma_semantic::resolver::resolver;

fn bench_semantic(c: &mut Criterion) {
    let bench_cases = &[("typescript", include_str!("../files/typescript.js"))];
    let mut group = c.benchmark_group("semantic");
    for (name, source) in bench_cases {
        group.bench_function(format!("{name}/semantic/legacy"), |b| {
            let input = StringSource::new(source);
            let mut ast = Ast::new(input.source_len(), StringAllocator::default());
            let mut parser = Parser::new(
                &mut ast,
                swc_experimental_ecma_parser::Syntax::Es(Default::default()),
                input,
                None,
            );
            let ret = parser.parse_module().unwrap();
            b.iter(|| {
                black_box(resolver(ret, &ast));
            });
        });
    }
}

criterion_group!(semantic, bench_semantic);
criterion_main!(semantic);
