use swc_core::{
    common::{
        FileName, GLOBALS, Globals, Mark, SourceMap, comments::SingleThreadedComments, sync::Lrc,
    },
    ecma::{
        ast::{EsVersion, Program},
        codegen::{Config, Emitter, text_writer::JsWriter},
        minifier::{
            optimize,
            option::{
                CompressOptions, ExtraOptions, MangleOptions, MinifyOptions, TopLevelOptions,
            },
        },
        parser::{EsSyntax, Parser, StringInput, Syntax, lexer::Lexer},
        transforms::base::{fixer::fixer, resolver},
        visit::VisitMutWith,
    },
};

pub(super) fn minify(source: &str, filename: &str) -> String {
    minify_with_options(
        source,
        filename,
        CompressOptions::default(),
        MangleOptions::default(),
    )
}

pub(super) fn minify_encoder(source: &str, filename: &str) -> String {
    // Keep helpers as calls: inlining ensureNodes can allocate an IIFE per node.
    minify_with_options(
        source,
        filename,
        CompressOptions {
            module: true,
            top_level: Some(TopLevelOptions { functions: false }),
            inline: 0,
            reduce_vars: false,
            ..Default::default()
        },
        MangleOptions {
            top_level: Some(false),
            ..Default::default()
        },
    )
}

fn minify_with_options(
    source: &str,
    filename: &str,
    compress: CompressOptions,
    mangle: MangleOptions,
) -> String {
    GLOBALS.set(&Globals::new(), || {
        let source_map = Lrc::new(SourceMap::default());
        let source_file =
            source_map.new_source_file(FileName::Custom(filename.into()).into(), source.to_owned());
        let comments = SingleThreadedComments::default();
        let lexer = Lexer::new(
            Syntax::Es(EsSyntax::default()),
            EsVersion::EsNext,
            StringInput::from(&*source_file),
            Some(&comments),
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser
            .parse_module()
            .unwrap_or_else(|error| panic!("failed to parse generated JavaScript: {error:?}"));
        let recoverable_errors = parser.take_errors();
        assert!(
            recoverable_errors.is_empty(),
            "generated JavaScript has parse errors: {recoverable_errors:?}"
        );

        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();
        let mut program = Program::Module(module);
        program.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));
        let mut program = optimize(
            program,
            source_map.clone(),
            Some(&comments),
            None,
            &MinifyOptions {
                compress: Some(compress),
                mangle: Some(mangle),
                ..Default::default()
            },
            &ExtraOptions {
                unresolved_mark,
                top_level_mark,
                mangle_name_cache: None,
            },
        );
        program.visit_mut_with(&mut fixer(None));

        let mut output = Vec::new();
        Emitter {
            cfg: Config::default()
                .with_minify(true)
                .with_omit_last_semi(true),
            cm: source_map.clone(),
            comments: None,
            wr: JsWriter::new(source_map, "\n", &mut output, None),
        }
        .emit_program(&program)
        .expect("failed to print minified JavaScript");
        String::from_utf8(output).expect("SWC emitted invalid UTF-8")
    })
}
