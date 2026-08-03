#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

fn bench_semantic(c: &mut Criterion) {
    let bench_cases = [
        (
            "typescript",
            include_str!("../files/typescript.js"),
            Lang::Js,
        ),
        (
            "cal.com.tsx",
            include_str!("../files/cal.com.tsx"),
            Lang::Tsx,
        ),
    ];
    let mut group = c.benchmark_group("semantic");
    for (name, source, lang) in bench_cases {
        group.bench_function(format!("{name}/semantic/legacy"), |b| {
            let allocator = Allocator::new();
            b.iter_batched(
                || {
                    let mut ast = Ast::new(source, &allocator);
                    let result = Parser::init(
                        &mut ast,
                        Options {
                            lang,
                            ..Options::default()
                        },
                        NoTokenParserConfig,
                    )
                    .parse();
                    assert!(result.diagnostics.is_empty());
                    let program = result.program;
                    (program, ast)
                },
                |(program, ast)| {
                    black_box(analyze(program, &ast, AnalyzeOptions::default()));
                },
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group!(semantic, bench_semantic);
criterion_main!(semantic);
