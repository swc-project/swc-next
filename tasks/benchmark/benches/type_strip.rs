#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_type_strip::{Options, strip_types};

fn bench_type_strip(c: &mut Criterion) {
    let bench_cases = [
        ("astro-config", include_str!("../files/astro-config.ts")),
        (
            "effect-schema-ast",
            include_str!("../files/effect-schema-ast.ts"),
        ),
        ("hono-types", include_str!("../files/hono-types.ts")),
    ];

    for (name, source) in bench_cases {
        c.bench_function(&format!("{name}/type_strip/strip"), |b| {
            b.iter(|| {
                black_box(strip_types(
                    black_box(source),
                    black_box(Options {
                        source_type: SourceType::Module,
                        lang: Lang::Ts,
                    }),
                ));
            });
        });
    }
}

criterion_group!(benches, bench_type_strip);
criterion_main!(benches);
