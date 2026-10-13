use std::hint::black_box;

use gungraun::client_requests::callgrind;
use gungraun::prelude::*;
use gungraun::{Callgrind, EntryPoint, EventKind, FlamegraphConfig};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};

#[library_benchmark]
#[bench::angular(include_str!("../../benchmark/files/angular-1.2.5.js"), Lang::Js)]
#[bench::astro_config(include_str!("../../benchmark/files/astro-config.ts"), Lang::Ts)]
#[bench::backbone(include_str!("../../benchmark/files/backbone-1.1.0.js"), Lang::Js)]
#[bench::cal_com_tsx(include_str!("../../benchmark/files/cal.com.tsx"), Lang::Tsx)]
#[bench::effect_schema_ast(include_str!("../../benchmark/files/effect-schema-ast.ts"), Lang::Ts)]
#[bench::hono_types(include_str!("../../benchmark/files/hono-types.ts"), Lang::Ts)]
#[bench::jquery(include_str!("../../benchmark/files/jquery-1.9.1.js"), Lang::Js)]
#[bench::jquery_mobile(include_str!("../../benchmark/files/jquery.mobile-1.4.2.js"), Lang::Js)]
#[bench::mootools(include_str!("../../benchmark/files/mootools-1.4.5.js"), Lang::Js)]
#[bench::three(include_str!("../../benchmark/files/three-0.138.3.js"), Lang::Js)]
#[bench::typescript(include_str!("../../benchmark/files/typescript.js"), Lang::Js)]
#[bench::underscore(include_str!("../../benchmark/files/underscore-1.5.2.js"), Lang::Js)]
#[bench::yui(include_str!("../../benchmark/files/yui-3.12.0.js"), Lang::Js)]
fn bench_parser(source: &str, lang: Lang) {
    callgrind::start_instrumentation();
    let (nodes, diagnostics) = {
        let allocator = Allocator::new();
        let options = Options {
            source_type: SourceType::Unambiguous,
            lang,
            preserve_parens: true,
            comments: CommentMode::None,
        };
        let result = Parser::init(
            &allocator,
            black_box(source),
            black_box(options),
            NoTokenParserConfig,
        )
        .parse();
        let nodes = result.ast.node_count();
        let diagnostics = result.diagnostics.len();
        black_box(result);
        (nodes, diagnostics)
    };
    callgrind::stop_instrumentation();
    assert!(nodes > 1);
    assert_eq!(diagnostics, 0);
}

library_benchmark_group!(name = parser, benchmarks = bench_parser);

main!(
    config = LibraryBenchmarkConfig::default().tool(
        Callgrind::with_args([
            "--instr-atstart=no",
            "--collect-atstart=yes",
            "--cache-sim=yes",
            "--branch-sim=yes",
            "--I1=32768,8,64",
            "--D1=32768,8,64",
            "--LL=8388608,16,64",
        ])
        .entry_point(EntryPoint::None)
        .format([EventKind::Ir])
        .flamegraph(FlamegraphConfig::default()),
    ),
    library_benchmark_groups = [parser]
);
