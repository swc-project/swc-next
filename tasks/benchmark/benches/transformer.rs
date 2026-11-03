#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use criterion::{Criterion, criterion_group, criterion_main};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, Semantic, analyze};
use swc_next_ecma_transformer::{
    Transformer, TransformerOptions, es2022::ES2022Options, normalize::Normalizer,
    proposal::ProposalOptions, typescript::TypeScriptOptions,
};

fn options() -> TransformerOptions {
    TransformerOptions {
        typescript: Some(TypeScriptOptions::default()),
        es2022: ES2022Options {
            static_blocks: true,
        },
        proposal: ProposalOptions {
            prevent_extensions: true,
        },
        ..TransformerOptions::default()
    }
}

/// Parse, normalize, and analyze the input like the Gungraun transform benchmark.
fn setup<'a>(allocator: &'a Allocator, source: &'a str, lang: Lang) -> (Ast<'a>, Semantic<'a>) {
    let result = Parser::init(
        allocator,
        source,
        Options {
            lang,
            ..Options::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(result.diagnostics.is_empty());
    let mut ast = result.ast;
    Normalizer::normalize_runtime(ast.root_program(), &mut ast);
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    (ast, semantic)
}

fn bench_transformer(c: &mut Criterion) {
    let bench_cases = [
        (
            "angular",
            include_str!("../files/angular-1.2.5.js"),
            Lang::Js,
        ),
        (
            "backbone",
            include_str!("../files/backbone-1.1.0.js"),
            Lang::Js,
        ),
        ("jquery", include_str!("../files/jquery-1.9.1.js"), Lang::Js),
        (
            "jquery-mobile",
            include_str!("../files/jquery.mobile-1.4.2.js"),
            Lang::Js,
        ),
        (
            "mootools",
            include_str!("../files/mootools-1.4.5.js"),
            Lang::Js,
        ),
        (
            "underscore",
            include_str!("../files/underscore-1.5.2.js"),
            Lang::Js,
        ),
        ("three", include_str!("../files/three-0.138.3.js"), Lang::Js),
        ("yui", include_str!("../files/yui-3.12.0.js"), Lang::Js),
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
        (
            "astro-config",
            include_str!("../files/astro-config.ts"),
            Lang::Ts,
        ),
        (
            "effect-schema-ast",
            include_str!("../files/effect-schema-ast.ts"),
            Lang::Ts,
        ),
        (
            "hono-types",
            include_str!("../files/hono-types.ts"),
            Lang::Ts,
        ),
    ];

    for (name, source, lang) in bench_cases {
        c.bench_function(&format!("{name}/transformer/transform"), |b| {
            // Prepare each input outside the measured section and drop it after.
            b.iter_custom(|iters| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iters {
                    let allocator = Allocator::new();
                    let (mut ast, semantic) = black_box(setup(&allocator, source, lang));
                    let options = black_box(options());
                    let start = Instant::now();
                    let result = Transformer::transform(&mut ast, semantic, options);
                    elapsed += start.elapsed();
                    black_box((ast, result));
                }
                elapsed
            });
        });
    }
}

criterion_group!(benches, bench_transformer);
criterion_main!(benches);
