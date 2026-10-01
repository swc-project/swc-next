//! Native AST deep clones, with parsing, validation, and destruction outside Callgrind.
//! Use the system allocator for all implementations so DHAT sees heap allocations.
//! swc-next CloneIn can only clone within the same Ast; Oxc also uses its source arena.
use std::hint::black_box;

use gungraun::prelude::*;
use gungraun::{Callgrind, Dhat, EventKind};
use oxc::ast::ast::Program as OxcProgram;
use oxc_allocator::{Allocator as OxcAllocator, CloneIn as OxcCloneIn};
use swc_core::{
    common::{FileName, SourceMap, sync::Lrc},
    ecma::{
        ast::{EsVersion, Program as SwcProgram},
        parser::{EsSyntax, Syntax, TsSyntax, parse_file_as_program},
    },
};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, CloneIn, Lang, Program, SourceType};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};

const UNDERSCORE: &str = include_str!("../files/underscore-1.5.2.js");
const ANGULAR: &str = include_str!("../files/angular-1.2.5.js");
const TYPESCRIPT: &str = include_str!("../files/typescript.js");
const TSX: &str = include_str!("../files/cal.com.tsx");

fn setup_next(source: &'static str, lang: Lang) -> Ast<'static> {
    let allocator = Box::leak(Box::new(Allocator::new()));
    let parsed = Parser::init(
        allocator,
        source,
        Options {
            lang,
            source_type: SourceType::Module,
            preserve_parens: false,
            comments: CommentMode::None,
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty());
    parsed.ast
}

fn setup_next_reserved(source: &'static str, lang: Lang) -> Ast<'static> {
    let mut ast = setup_next(source, lang);
    ast.reserve(ast.node_count(), ast.extra_data_count());
    ast
}

fn verify_next((mut ast, cloned): (Ast<'static>, Program)) {
    let original = print(ast.allocator(), &ast, CodegenOptions::default())
        .code
        .to_owned();
    ast.set_root_program(cloned);
    let copy = print(ast.allocator(), &ast, CodegenOptions::default());
    assert_eq!(original, copy.code);
}

#[library_benchmark(teardown = verify_next)]
#[bench::underscore(setup_next(UNDERSCORE, Lang::Js))]
#[bench::angular(setup_next(ANGULAR, Lang::Js))]
#[bench::typescript(setup_next(TYPESCRIPT, Lang::Js))]
#[bench::tsx(setup_next(TSX, Lang::Tsx))]
fn bench_clone_next(mut ast: Ast<'static>) -> (Ast<'static>, Program) {
    let root = black_box(ast.root_program());
    let cloned = black_box(root.clone_in(black_box(&mut ast)));
    (ast, cloned)
}

#[library_benchmark(teardown = verify_next)]
#[bench::underscore(setup_next_reserved(UNDERSCORE, Lang::Js))]
#[bench::angular(setup_next_reserved(ANGULAR, Lang::Js))]
#[bench::typescript(setup_next_reserved(TYPESCRIPT, Lang::Js))]
#[bench::tsx(setup_next_reserved(TSX, Lang::Tsx))]
fn bench_clone_next_reserved(mut ast: Ast<'static>) -> (Ast<'static>, Program) {
    let root = black_box(ast.root_program());
    let cloned = black_box(root.clone_in(black_box(&mut ast)));
    (ast, cloned)
}

fn setup_oxc(source: &'static str, lang: Lang) -> (&'static OxcAllocator, OxcProgram<'static>) {
    let allocator = Box::leak(Box::new(OxcAllocator::default()));
    let source_type = if lang == Lang::Tsx {
        oxc::span::SourceType::tsx()
    } else {
        oxc::span::SourceType::mjs()
    };
    let parsed = oxc::parser::Parser::new(allocator, source, source_type)
        .with_options(oxc::parser::ParseOptions {
            preserve_parens: false,
            ..Default::default()
        })
        .parse();
    assert!(parsed.diagnostics.is_empty());
    let mut program = parsed.program;
    // SWC stores comments separately and swc-next comments are disabled.
    program.comments.clear();
    (allocator, program)
}

fn setup_oxc_syntax_only(
    source: &'static str,
    lang: Lang,
) -> (&'static OxcAllocator, OxcProgram<'static>) {
    let (allocator, mut program) = setup_oxc(source, lang);
    // Oxc's native Program clone additionally copies the complete source text.
    program.source_text = "";
    (allocator, program)
}

fn verify_oxc((original, cloned): (OxcProgram<'static>, OxcProgram<'static>)) {
    assert_eq!(format!("{original:?}"), format!("{cloned:?}"));
}

#[library_benchmark(teardown = verify_oxc)]
#[bench::underscore(setup_oxc(UNDERSCORE, Lang::Js))]
#[bench::angular(setup_oxc(ANGULAR, Lang::Js))]
#[bench::typescript(setup_oxc(TYPESCRIPT, Lang::Js))]
#[bench::tsx(setup_oxc(TSX, Lang::Tsx))]
fn bench_clone_oxc(
    (allocator, program): (&'static OxcAllocator, OxcProgram<'static>),
) -> (OxcProgram<'static>, OxcProgram<'static>) {
    let cloned = black_box(black_box(&program).clone_in(black_box(allocator)));
    (program, cloned)
}

#[library_benchmark(teardown = verify_oxc)]
#[bench::underscore(setup_oxc_syntax_only(UNDERSCORE, Lang::Js))]
#[bench::angular(setup_oxc_syntax_only(ANGULAR, Lang::Js))]
#[bench::typescript(setup_oxc_syntax_only(TYPESCRIPT, Lang::Js))]
#[bench::tsx(setup_oxc_syntax_only(TSX, Lang::Tsx))]
fn bench_clone_oxc_syntax_only(
    (allocator, program): (&'static OxcAllocator, OxcProgram<'static>),
) -> (OxcProgram<'static>, OxcProgram<'static>) {
    let cloned = black_box(black_box(&program).clone_in(black_box(allocator)));
    (program, cloned)
}

fn setup_swc(source: &'static str, lang: Lang) -> SwcProgram {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(FileName::Custom("input.js".into()).into(), source);
    let syntax = if lang == Lang::Tsx {
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        })
    } else {
        Syntax::Es(EsSyntax::default())
    };
    let mut errors = Vec::new();
    let program =
        parse_file_as_program(&fm, syntax, EsVersion::latest(), None, &mut errors).unwrap();
    assert!(errors.is_empty());
    program
}

fn verify_swc((original, cloned): (SwcProgram, SwcProgram)) {
    assert_eq!(original, cloned);
}

#[library_benchmark(teardown = verify_swc)]
#[bench::underscore(setup_swc(UNDERSCORE, Lang::Js))]
#[bench::angular(setup_swc(ANGULAR, Lang::Js))]
#[bench::typescript(setup_swc(TYPESCRIPT, Lang::Js))]
#[bench::tsx(setup_swc(TSX, Lang::Tsx))]
fn bench_clone_swc(program: SwcProgram) -> (SwcProgram, SwcProgram) {
    let cloned = black_box(black_box(&program).clone());
    (program, cloned)
}

library_benchmark_group!(
    name = ast_clone,
    benchmarks = [
        bench_clone_next,
        bench_clone_oxc,
        bench_clone_swc,
        bench_clone_next_reserved,
        bench_clone_oxc_syntax_only,
    ]
);

main!(
    config = LibraryBenchmarkConfig::default()
        .tool(Callgrind::with_args(["--branch-sim=no", "--cache-sim=no"]).format([EventKind::Ir]))
        .tool(Dhat::default()),
    library_benchmark_groups = [ast_clone]
);
