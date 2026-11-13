#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use criterion::{Criterion, criterion_group, criterion_main};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::Lang;
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
            // Give each iteration its own allocator, and parse and drop outside the measured
            // section, so memory does not accumulate across iterations.
            b.iter_custom(|iters| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iters {
                    let allocator = Allocator::new();
                    let result = Parser::init(
                        &allocator,
                        source,
                        Options {
                            lang,
                            ..Options::default()
                        },
                        NoTokenParserConfig,
                    )
                    .parse();
                    assert!(result.diagnostics.is_empty());
                    let ast = black_box(result.ast);
                    let start = Instant::now();
                    let semantic = analyze(&ast, AnalyzeOptions::default());
                    elapsed += start.elapsed();
                    black_box(semantic);
                }
                elapsed
            });
        });
    }
}

criterion_group!(semantic, bench_semantic);
criterion_main!(semantic);
