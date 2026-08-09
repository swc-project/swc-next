#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::hint::black_box;

use gungraun::prelude::*;
use gungraun::{Callgrind, EventKind};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, BindingIdentifier, IdentifierName, IdentifierReference, LabelIdentifier, Lang,
    PrivateIdentifier, Program, Traverse, TraverseAction, TraverseCtx, TraverseWith, Visit,
    VisitMut, VisitMutWith, VisitWith,
};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

const ANGULAR: &str = include_str!("../files/angular-1.2.5.js");
const BACKBONE: &str = include_str!("../files/backbone-1.1.0.js");
const JQUERY: &str = include_str!("../files/jquery-1.9.1.js");
const JQUERY_MOBILE: &str = include_str!("../files/jquery.mobile-1.4.2.js");
const MOOTOOLS: &str = include_str!("../files/mootools-1.4.5.js");
const UNDERSCORE: &str = include_str!("../files/underscore-1.5.2.js");
const THREE: &str = include_str!("../files/three-0.138.3.js");
const YUI: &str = include_str!("../files/yui-3.12.0.js");
const TYPESCRIPT: &str = include_str!("../files/typescript.js");
const CAL_COM_TSX: &str = include_str!("../files/cal.com.tsx");

#[library_benchmark]
#[bench::angular(ANGULAR, Lang::Js)]
#[bench::backbone(BACKBONE, Lang::Js)]
#[bench::jquery(JQUERY, Lang::Js)]
#[bench::jquery_mobile(JQUERY_MOBILE, Lang::Js)]
#[bench::mootools(MOOTOOLS, Lang::Js)]
#[bench::underscore(UNDERSCORE, Lang::Js)]
#[bench::three(THREE, Lang::Js)]
#[bench::yui(YUI, Lang::Js)]
#[bench::typescript(TYPESCRIPT, Lang::Js)]
#[bench::cal_com_tsx(CAL_COM_TSX, Lang::Tsx)]
fn bench_parser(source: &str, lang: Lang) {
    let source = black_box(source);
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    black_box(
        Parser::init(
            &mut ast,
            Options {
                lang,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse(),
    );
}

struct ParsedProgram {
    ast: Ast<'static>,
    program: Program,
}

fn setup_program(source: &'static str, lang: Lang) -> ParsedProgram {
    let allocator = Box::leak(Box::new(Allocator::new()));
    let mut ast = Ast::new(source, allocator);
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
    ParsedProgram { ast, program }
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_semantic(parsed: ParsedProgram) {
    black_box(analyze(
        parsed.program,
        &parsed.ast,
        AnalyzeOptions::default(),
    ));
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_semantic_with_syntax_check(parsed: ParsedProgram) {
    black_box(analyze(
        parsed.program,
        &parsed.ast,
        AnalyzeOptions { check_syntax: true },
    ));
}

struct VisitCounter<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    count: usize,
}

impl<'ast> Visit<'ast> for VisitCounter<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_identifier_reference(&mut self, _node: IdentifierReference) {
        self.count += 1;
    }

    fn visit_private_identifier(&mut self, _node: PrivateIdentifier) {
        self.count += 1;
    }

    fn visit_binding_identifier(&mut self, _node: BindingIdentifier) {
        self.count += 1;
    }

    fn visit_identifier_name(&mut self, _node: IdentifierName) {
        self.count += 1;
    }

    fn visit_label_identifier(&mut self, _node: LabelIdentifier) {
        self.count += 1;
    }
}

struct VisitMutCounter<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    count: usize,
}

impl<'ast> VisitMut<'ast> for VisitMutCounter<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_identifier_reference(
        &mut self,
        _node: IdentifierReference,
    ) -> Option<IdentifierReference> {
        self.count += 1;
        None
    }

    fn visit_mut_private_identifier(
        &mut self,
        _node: PrivateIdentifier,
    ) -> Option<PrivateIdentifier> {
        self.count += 1;
        None
    }

    fn visit_mut_binding_identifier(
        &mut self,
        _node: BindingIdentifier,
    ) -> Option<BindingIdentifier> {
        self.count += 1;
        None
    }

    fn visit_mut_identifier_name(&mut self, _node: IdentifierName) -> Option<IdentifierName> {
        self.count += 1;
        None
    }

    fn visit_mut_label_identifier(&mut self, _node: LabelIdentifier) -> Option<LabelIdentifier> {
        self.count += 1;
        None
    }
}

struct TraverseCounter {
    count: usize,
}

impl<'ast> Traverse<'ast> for TraverseCounter {
    fn enter_identifier_reference(
        &mut self,
        _node: IdentifierReference,
        _ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction<IdentifierReference> {
        self.count += 1;
        TraverseAction::Proceed
    }

    fn enter_private_identifier(
        &mut self,
        _node: PrivateIdentifier,
        _ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction<PrivateIdentifier> {
        self.count += 1;
        TraverseAction::Proceed
    }

    fn enter_binding_identifier(
        &mut self,
        _node: BindingIdentifier,
        _ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction<BindingIdentifier> {
        self.count += 1;
        TraverseAction::Proceed
    }

    fn enter_identifier_name(
        &mut self,
        _node: IdentifierName,
        _ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction<IdentifierName> {
        self.count += 1;
        TraverseAction::Proceed
    }

    fn enter_label_identifier(
        &mut self,
        _node: LabelIdentifier,
        _ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction<LabelIdentifier> {
        self.count += 1;
        TraverseAction::Proceed
    }
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_visit(parsed: ParsedProgram) {
    let mut counter = VisitCounter {
        ast: &parsed.ast,
        count: 0,
    };
    parsed.program.visit_with(&mut counter);
    black_box(counter.count);
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_visit_mut(mut parsed: ParsedProgram) {
    let mut counter = VisitMutCounter {
        ast: &mut parsed.ast,
        count: 0,
    };
    parsed.program.visit_mut_with(&mut counter);
    black_box(counter.count);
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_traverse(mut parsed: ParsedProgram) {
    let mut counter = TraverseCounter { count: 0 };
    let _ = parsed
        .program
        .traverse_with(&mut counter, &mut TraverseCtx::new(&mut parsed.ast, ()));
    black_box(counter.count);
}

library_benchmark_group!(name = parser, benchmarks = bench_parser);

library_benchmark_group!(
    name = semantic,
    benchmarks = [bench_semantic, bench_semantic_with_syntax_check]
);

library_benchmark_group!(
    name = visit,
    benchmarks = [bench_visit, bench_visit_mut, bench_traverse]
);

main!(
    config =
        LibraryBenchmarkConfig::default().tool(
            Callgrind::with_args(["--branch-sim=no", "--cache-sim=no",]).format([EventKind::Ir]),
        ),
    library_benchmark_groups = [parser, semantic, visit]
);
