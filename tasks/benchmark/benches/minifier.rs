#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use criterion::{Criterion, criterion_group, criterion_main};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, SourceType};
use swc_next_ecma_minifier::{Minifier, MinifierOptions};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze};

fn parse<'a>(allocator: &'a Allocator, source: &'a str) -> Ast<'a> {
    let result = Parser::init(
        allocator,
        source,
        Options {
            source_type: SourceType::Script,
            lang: Lang::Js,
            preserve_parens: false,
            comments: CommentMode::None,
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(result.diagnostics.is_empty());
    result.ast
}

fn bench_minifier(c: &mut Criterion) {
    let bench_cases = [
        ("underscore", include_str!("../files/underscore-1.5.2.js")),
        ("angular", include_str!("../files/angular-1.2.5.js")),
        ("typescript", include_str!("../files/typescript.js")),
        (
            "semantic",
            include_str!(
                "../../../crates/swc_ecma_minifier/tests/fixtures/semantic/combined/input.js"
            ),
        ),
    ];

    for (name, source) in bench_cases {
        let allocator = Allocator::new();
        let ast = parse(&allocator, source);
        let analysis = analyze(
            &ast,
            AnalyzeOptions {
                check_syntax: SyntaxChecks::all(),
            },
        );
        assert!(analysis.diagnostics.is_empty());

        c.bench_function(&format!("{name}/minifier/minify"), |b| {
            // Parse each input outside the measured section and drop it after.
            b.iter_custom(|iters| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iters {
                    let allocator = Allocator::new();
                    let mut ast = black_box(parse(&allocator, source));
                    let start = Instant::now();
                    Minifier::minify(&mut ast, black_box(MinifierOptions::default()));
                    elapsed += start.elapsed();
                    black_box(ast);
                }
                elapsed
            });
        });
    }
}

criterion_group!(benches, bench_minifier);
criterion_main!(benches);
