#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::hint::black_box;

use gungraun::prelude::*;
use gungraun::{Callgrind, EventKind};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, BindingIdentifier, Decorator, IdentifierName, IdentifierReference, LabelIdentifier, Lang,
    PrivateIdentifier, SourceType, Traverse, TraverseCtx, TraverseWith, TsTypeAnnotation, Visit,
    VisitMut, VisitMutWith, VisitWith,
};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_minifier::{Minifier, MinifierOptions};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, Semantic, SyntaxChecks, analyze};
use swc_next_ecma_transformer::{
    Transformer, TransformerOptions, es2022::ES2022Options, normalize::Normalizer,
    proposal::ProposalOptions, typescript::TypeScriptOptions,
};
use swc_next_type_strip::{Options as StripOptions, strip_types};

const MINIFIER_SEMANTIC: &str =
    include_str!("../../../crates/swc_ecma_minifier/tests/fixtures/semantic/combined/input.js");

const ANGULAR: &str = include_str!("../files/angular-1.2.5.js");
const BACKBONE: &str = include_str!("../files/backbone-1.1.0.js");
const JQUERY: &str = include_str!("../files/jquery-1.9.1.js");
const JQUERY_MOBILE: &str = include_str!("../files/jquery.mobile-1.4.2.js");
const MOOTOOLS: &str = include_str!("../files/mootools-1.4.5.js");
const UNDERSCORE: &str = include_str!("../files/underscore-1.5.2.js");
const THREE: &str = include_str!("../files/three-0.138.3.js");
const YUI: &str = include_str!("../files/yui-3.12.0.js");
const TYPESCRIPT: &str = include_str!("../files/typescript.js");
const TYPESCRIPT_CHECKER: &str = include_str!("../files/checker.ts");
const CAL_COM_TSX: &str = include_str!("../files/cal.com.tsx");
const ASTRO_CONFIG: &str = include_str!("../files/astro-config.ts");
const EFFECT_SCHEMA_AST: &str = include_str!("../files/effect-schema-ast.ts");
const HONO_TYPES: &str = include_str!("../files/hono-types.ts");

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
#[bench::hono_types(HONO_TYPES, Lang::Ts)]
fn bench_parser(source: &str, lang: Lang) {
    let source = black_box(source);
    let allocator = Allocator::new();
    black_box(
        Parser::init(
            &allocator,
            source,
            Options {
                lang,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse(),
    );
}

fn setup_program(source: &'static str, lang: Lang) -> Ast<'static> {
    let allocator = Box::leak(Box::new(Allocator::new()));
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
    result.ast
}

fn setup_minifier(source: &'static str) -> Ast<'static> {
    let allocator = Box::leak(Box::new(Allocator::new()));
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
    let ast = result.ast;
    let analysis = analyze(
        &ast,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
        },
    );
    assert!(analysis.diagnostics.is_empty());
    ast
}

#[library_benchmark]
#[bench::underscore(setup_minifier(UNDERSCORE))]
#[bench::angular(setup_minifier(ANGULAR))]
#[bench::typescript(setup_minifier(TYPESCRIPT))]
#[bench::semantic(setup_minifier(MINIFIER_SEMANTIC))]
fn bench_minifier(ast: Ast<'static>) -> Ast<'static> {
    let mut ast = black_box(ast);
    Minifier::minify(&mut ast, black_box(MinifierOptions::default()));
    // Return the AST so its drop is outside the measured function.
    black_box(ast)
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
#[bench::typescript_checker(setup_program(TYPESCRIPT_CHECKER, Lang::Ts))]
fn bench_analyze(ast: Ast<'static>) {
    black_box(analyze(&ast, AnalyzeOptions::default()));
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
#[bench::typescript_checker(setup_program(TYPESCRIPT_CHECKER, Lang::Ts))]
fn bench_syntax_check(ast: Ast<'static>) {
    black_box(analyze(
        &ast,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
        },
    ));
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_codegen(ast: Ast<'static>) {
    black_box(print(&ast, CodegenOptions::default()));
}

type TransformInput = (Ast<'static>, Semantic<'static>, TransformerOptions);

fn setup_transform(source: &'static str, lang: Lang) -> TransformInput {
    let mut ast = setup_program(source, lang);
    Normalizer::normalize_runtime(ast.root_program(), &mut ast);
    let options = TransformerOptions {
        typescript: Some(TypeScriptOptions::default()),
        es2022: ES2022Options {
            static_blocks: true,
        },
        proposal: ProposalOptions {
            prevent_extensions: true,
        },
        ..TransformerOptions::default()
    };
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    (ast, semantic, options)
}

#[library_benchmark]
#[bench::angular(setup_transform(ANGULAR, Lang::Js))]
#[bench::backbone(setup_transform(BACKBONE, Lang::Js))]
#[bench::jquery(setup_transform(JQUERY, Lang::Js))]
#[bench::jquery_mobile(setup_transform(JQUERY_MOBILE, Lang::Js))]
#[bench::mootools(setup_transform(MOOTOOLS, Lang::Js))]
#[bench::underscore(setup_transform(UNDERSCORE, Lang::Js))]
#[bench::three(setup_transform(THREE, Lang::Js))]
#[bench::yui(setup_transform(YUI, Lang::Js))]
#[bench::typescript(setup_transform(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_transform(CAL_COM_TSX, Lang::Tsx))]
#[bench::astro_config(setup_transform(ASTRO_CONFIG, Lang::Ts))]
#[bench::effect_schema_ast(setup_transform(EFFECT_SCHEMA_AST, Lang::Ts))]
#[bench::hono_types(setup_transform(HONO_TYPES, Lang::Ts))]
fn bench_transform(input: TransformInput) {
    let (mut ast, semantic, options) = black_box(input);
    let result = Transformer::transform(&mut ast, semantic, options);
    black_box((ast, result));
}

fn type_strip_options(lang: Lang) -> StripOptions {
    StripOptions {
        source_type: SourceType::Module,
        lang,
    }
}

#[library_benchmark(
    config = LibraryBenchmarkConfig::default().tool(
        Callgrind::with_args(["--dump-before=*benchmark_parse_boundary"]),
    )
)]
#[bench::astro_config(ASTRO_CONFIG, type_strip_options(Lang::Ts))]
#[bench::effect_schema_ast(EFFECT_SCHEMA_AST, type_strip_options(Lang::Ts))]
#[bench::hono_types(HONO_TYPES, type_strip_options(Lang::Ts))]
fn bench_type_strip(source: &str, options: StripOptions) {
    black_box(strip_types(black_box(source), black_box(options)));
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

    fn visit_mut_ts_type_annotation(
        &mut self,
        _node: TsTypeAnnotation,
    ) -> Option<TsTypeAnnotation> {
        self.count += 1;
        None
    }

    fn visit_mut_decorator(&mut self, _node: Decorator) -> Option<Decorator> {
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
    ) {
        self.count += 1;
    }

    fn enter_private_identifier(
        &mut self,
        _node: PrivateIdentifier,
        _ctx: &mut TraverseCtx<'_, 'ast>,
    ) {
        self.count += 1;
    }

    fn enter_binding_identifier(
        &mut self,
        _node: BindingIdentifier,
        _ctx: &mut TraverseCtx<'_, 'ast>,
    ) {
        self.count += 1;
    }

    fn enter_identifier_name(&mut self, _node: IdentifierName, _ctx: &mut TraverseCtx<'_, 'ast>) {
        self.count += 1;
    }

    fn enter_label_identifier(&mut self, _node: LabelIdentifier, _ctx: &mut TraverseCtx<'_, 'ast>) {
        self.count += 1;
    }
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_visit(ast: Ast<'static>) {
    let program = ast.root_program();
    let mut counter = VisitCounter {
        ast: &ast,
        count: 0,
    };
    program.visit_with(&mut counter);
    black_box(counter.count);
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_visit_mut(mut ast: Ast<'static>) {
    let program = ast.root_program();
    let mut counter = VisitMutCounter {
        ast: &mut ast,
        count: 0,
    };
    program.visit_mut_with(&mut counter);
    black_box(counter.count);
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_traverse(ast: Ast<'static>) {
    let program = ast.root_program();
    let mut counter = TraverseCounter { count: 0 };
    program.traverse_with(&mut counter, &mut TraverseCtx::new(&ast, ()));
    black_box(counter.count);
}

library_benchmark_group!(name = parser, benchmarks = bench_parser);

library_benchmark_group!(
    name = semantic,
    benchmarks = [bench_analyze, bench_syntax_check]
);

library_benchmark_group!(name = codegen, benchmarks = bench_codegen);

library_benchmark_group!(name = minifier, benchmarks = bench_minifier);

library_benchmark_group!(name = transform, benchmarks = bench_transform);

library_benchmark_group!(name = type_strip, benchmarks = bench_type_strip);

library_benchmark_group!(
    name = visit,
    benchmarks = [bench_visit, bench_visit_mut, bench_traverse]
);

main!(
    config =
        LibraryBenchmarkConfig::default().tool(
            Callgrind::with_args(["--branch-sim=no", "--cache-sim=no",]).format([EventKind::Ir]),
        ),
    library_benchmark_groups = [
        parser, semantic, codegen, minifier, transform, type_strip, visit
    ]
);
