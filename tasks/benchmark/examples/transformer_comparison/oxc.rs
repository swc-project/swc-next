use std::{hint::black_box, path::Path};

use oxc::{
    ast::ast::Program,
    parser::{ParseOptions, Parser},
    semantic::{Scoping, SemanticBuilder},
    span::SourceType,
    transformer::{
        JsxOptions, JsxRuntime, ReactRefreshOptions, TransformOptions, Transformer,
        TypeScriptOptions,
    },
};
use oxc_allocator::Allocator;

pub fn run(source: &str) {
    let allocator = Allocator::default();
    let mut program = run_parse(source, &allocator);
    let scoping = run_semantic(&program);
    run_transform(&allocator, &mut program, scoping);
    black_box(program);
}

#[inline(never)]
fn run_parse<'a>(source: &'a str, allocator: &'a Allocator) -> Program<'a> {
    let result = Parser::new(allocator, source, SourceType::tsx())
        .with_options(ParseOptions {
            preserve_parens: false,
            ..ParseOptions::default()
        })
        .parse();
    assert!(result.diagnostics.is_empty());
    result.program
}

#[inline(never)]
fn run_semantic(program: &Program<'_>) -> Scoping {
    let result = SemanticBuilder::new()
        .with_build_nodes(false)
        .with_excess_capacity(2.0)
        .with_enum_eval(true)
        .build(program);
    assert!(result.diagnostics.is_empty());
    result.semantic.into_scoping()
}

#[inline(never)]
fn run_transform<'a>(allocator: &'a Allocator, program: &mut Program<'a>, scoping: Scoping) {
    let options = TransformOptions {
        typescript: TypeScriptOptions {
            only_remove_type_imports: false,
            optimize_const_enums: true,
            optimize_enums: false,
            ..TypeScriptOptions::default()
        },
        jsx: JsxOptions {
            runtime: JsxRuntime::Automatic,
            development: true,
            display_name_plugin: true,
            jsx_self_plugin: true,
            jsx_source_plugin: true,
            refresh: Some(ReactRefreshOptions::default()),
            pure: true,
            ..JsxOptions::default()
        },
        ..TransformOptions::default()
    };
    let result = Transformer::new(allocator, Path::new("App.tsx"), &options)
        .build_with_scoping(scoping, program);
    assert!(result.diagnostics.is_empty());
    black_box(result);
}
