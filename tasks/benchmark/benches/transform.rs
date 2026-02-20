#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::rc::Rc;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use swc_core::common::comments::SingleThreadedComments;
use swc_experimental_ecma_ast::Ast;
use swc_experimental_ecma_parser::{Parser, StringSource};
use swc_experimental_ecma_transforms_base::remove_paren::remove_paren;

fn bench_transform(c: &mut Criterion) {
    let bench_cases = &[("typescript", include_str!("../files/typescript.js"))];
    let mut group = c.benchmark_group("semantic");
    for (name, source) in bench_cases {
        group.bench_function(format!("{name}/transform/remove_paren"), |b| {
            b.iter_batched(
                || {
                    let comments = SingleThreadedComments::default();
                    let input = StringSource::new(source);
                    let mut ast = Ast::new(input.source_len(), Rc::default());
                    let mut parser = Parser::new(
                        &mut ast,
                        swc_experimental_ecma_parser::Syntax::Es(Default::default()),
                        input,
                        Some(&comments),
                    );
                    let module = parser.parse_module().unwrap();
                    (module, ast, comments)
                },
                |(module, mut ast, comments)| {
                    remove_paren(module, &mut ast, Some(&comments));
                },
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group!(transform, bench_transform);
criterion_main!(transform);
