use std::hint::black_box;

use gungraun::prelude::*;
use gungraun::{Callgrind, CallgrindMetrics, Dhat, DhatMetric, EventKind, FlamegraphConfig};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, Program};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

const TYPESCRIPT: &str = include_str!("../files/typescript.js");
const CAL_COM_TSX: &str = include_str!("../files/cal.com.tsx");

struct ParsedProgram {
    ast: Ast<'static>,
    program: Program,
}

fn setup_program(source: &'static str, lang: Lang) -> ParsedProgram {
    let allocator = Box::leak(Box::new(Allocator::new()));
    let mut ast = Ast::new(source.as_bytes(), allocator);
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
    ParsedProgram {
        ast,
        program: result.program,
    }
}

#[library_benchmark]
#[bench::typescript_js(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_semantic(parsed: ParsedProgram) {
    black_box(analyze(
        parsed.program,
        &parsed.ast,
        AnalyzeOptions::default(),
    ));
}

#[library_benchmark]
#[bench::typescript_js(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_semantic_with_syntax(parsed: ParsedProgram) {
    black_box(analyze(
        parsed.program,
        &parsed.ast,
        AnalyzeOptions { check_syntax: true },
    ));
}

library_benchmark_group!(
    name = semantic,
    benchmarks = [bench_semantic, bench_semantic_with_syntax]
);

fn benchmark_config() -> LibraryBenchmarkConfig {
    let mut config = LibraryBenchmarkConfig::default();
    config.tool(
        Callgrind::with_args(["--branch-sim=no", "--cache-sim=yes"])
            .format([CallgrindMetrics::All])
            .flamegraph(FlamegraphConfig::default().event_kinds([
                EventKind::Ir,
                EventKind::D1mr,
                EventKind::D1mw,
                EventKind::DLmr,
                EventKind::DLmw,
                EventKind::EstimatedCycles,
            ])),
    );
    config.tool(Dhat::default().format([
        DhatMetric::TotalBytes,
        DhatMetric::TotalBlocks,
        DhatMetric::AtTGmaxBytes,
        DhatMetric::AtTGmaxBlocks,
    ]));
    config
}

main!(
    config = benchmark_config(),
    library_benchmark_groups = [semantic]
);
