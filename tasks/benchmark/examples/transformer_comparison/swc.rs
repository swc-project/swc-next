use std::hint::black_box;

use swc_core::{
    common::{
        FileName, GLOBALS, Globals, Mark, SourceMap,
        comments::SingleThreadedComments,
        errors::{HANDLER, Handler},
        sync::Lrc,
    },
    ecma::{
        ast::{EsVersion, Program},
        parser::{Parser, StringInput, Syntax, TsSyntax, lexer::Lexer},
        transforms::{
            base::resolver,
            react::{Options, RefreshOptions, Runtime, react},
            typescript::{Config as TypeScriptOptions, typescript},
        },
    },
};

pub fn run(source: &str) {
    GLOBALS.set(&Globals::new(), || {
        let source_map = Lrc::new(SourceMap::default());
        let comments = SingleThreadedComments::default();
        let handler = Handler::with_emitter_writer(Box::new(std::io::sink()), None);
        HANDLER.set(&handler, || {
            let mut program = run_parse(source, &source_map, &comments);
            let unresolved_mark = Mark::new();
            let top_level_mark = Mark::new();
            run_resolver(&mut program, unresolved_mark, top_level_mark);
            run_transform(
                &mut program,
                source_map,
                comments,
                unresolved_mark,
                top_level_mark,
            );
            assert!(!handler.has_errors());
            black_box(program);
        });
    });
}

#[inline(never)]
fn run_parse(
    source: &str,
    source_map: &Lrc<SourceMap>,
    comments: &SingleThreadedComments,
) -> Program {
    let file = source_map.new_source_file(
        FileName::Custom("App.tsx".to_owned()).into(),
        source.to_owned(),
    );
    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..TsSyntax::default()
        }),
        EsVersion::EsNext,
        StringInput::from(&*file),
        Some(comments),
    );
    let mut parser = Parser::new_from(lexer);
    let program = Program::Module(parser.parse_module().expect("failed to parse App.tsx"));
    assert!(parser.take_errors().is_empty());
    program
}

#[inline(never)]
fn run_resolver(program: &mut Program, unresolved_mark: Mark, top_level_mark: Mark) {
    program.mutate(resolver(unresolved_mark, top_level_mark, true));
}

#[inline(never)]
fn run_transform(
    program: &mut Program,
    source_map: Lrc<SourceMap>,
    comments: SingleThreadedComments,
    unresolved_mark: Mark,
    top_level_mark: Mark,
) {
    let jsx_options = Options {
        runtime: Some(Runtime::Automatic),
        development: Some(true),
        throw_if_namespace: Some(true),
        refresh: Some(RefreshOptions::default()),
        ..Options::default()
    };
    let typescript_options = TypeScriptOptions {
        verbatim_module_syntax: false,
        ts_enum_is_mutable: true,
        native_class_properties: true,
        ..TypeScriptOptions::default()
    };
    program.mutate(react(
        source_map,
        Some(comments),
        jsx_options,
        top_level_mark,
        unresolved_mark,
    ));
    program.mutate(typescript(
        typescript_options,
        unresolved_mark,
        top_level_mark,
    ));
}
