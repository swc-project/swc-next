#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Ast, Comments};
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
                    let mut comments = Comments::new_in(&allocator);
                    let input = StringSource::new(source);
                    let mut ast = Ast::new(input.source_len(), &allocator);
                    let module = {
                        let mut parser = Parser::new(
                            &mut ast,
                            swc_experimental_ecma_parser::Syntax::Es(Default::default()),
                            input,
                            Some(&mut comments),
                        );
                        parser.parse_module().unwrap()
                    };
                    (module, ast, comments)
                },
                |(mut module, mut ast, mut comments)| {
                    remove_paren(&mut module, &mut ast, Some(&mut comments));
                },
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group!(transform, bench_transform);
criterion_main!(transform);
