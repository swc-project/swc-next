#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use criterion::{Bencher, Criterion, criterion_group, criterion_main};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::Lang;
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

fn bench_new(b: &mut Bencher, src: &'static str, lang: Lang) {
    b.iter(|| {
        let allocator = Allocator::new();
        Parser::init(
            &allocator,
            src,
            Options {
                lang,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
    });
}

fn bench_files(c: &mut Criterion) {
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
            "jquery.mobile",
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
    ];

    for (name, source, lang) in bench_cases {
        c.bench_function(&format!("{name}/parser/new"), |b| {
            bench_new(b, source, lang)
        });
    }
}

criterion_group!(benches, bench_files);
criterion_main!(benches);
