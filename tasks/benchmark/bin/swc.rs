use std::{hint::black_box, io};

use benchmark::{Args, Operation};
use clap::Parser as _;
use swc_core::{
    common::{BytePos, FileName, GLOBALS, Globals, Mark, SourceMap, errors::Handler, sync::Lrc},
    ecma::{
        ast::{EsVersion, Program},
        codegen::to_code_default,
        parser::{EsSyntax, Parser, StringInput, Syntax, TsSyntax, lexer::Lexer},
        transforms::base::resolver,
        visit::VisitMutWith,
    },
    typescript::fast_dts::{FastDts, FastDtsOptions},
};

fn main() {
    let args = Args::parse();
    let source = args.file.source();
    let extension = std::path::Path::new(args.file.name())
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let typescript = matches!(extension, "ts" | "tsx" | "mts" | "cts");
    let ts_syntax = TsSyntax {
        tsx: extension == "tsx",
        ..Default::default()
    };
    let syntax = if typescript {
        Syntax::Typescript(ts_syntax)
    } else {
        Syntax::Es(EsSyntax {
            jsx: extension == "jsx",
            ..Default::default()
        })
    };

    if args.operation == Operation::Parse {
        let lexer = Lexer::new(
            syntax,
            EsVersion::latest(),
            StringInput::new(source, BytePos(1), BytePos(1 + source.len() as u32)),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        black_box(parser.parse_module().expect("parse failed"));
        assert!(parser.take_errors().is_empty(), "parse diagnostics");
        return;
    }

    GLOBALS.set(&Globals::default(), || {
        let cm: Lrc<SourceMap> = Default::default();
        if args.operation == Operation::TypeStrip {
            assert!(typescript, "type-strip requires TypeScript input");
            let handler = Handler::with_emitter_writer(Box::new(io::sink()), Some(cm.clone()));
            black_box(
                swc_ts_fast_strip::operate(
                    &cm,
                    &handler,
                    source.to_owned(),
                    swc_ts_fast_strip::Options {
                        module: Some(true),
                        parser: ts_syntax,
                        ..Default::default()
                    },
                )
                .expect("type stripping failed"),
            );
            assert!(!handler.has_errors(), "type stripping diagnostics");
            return;
        }
        let filename = FileName::Real(args.file.name().into());
        let file = cm.new_source_file(filename.clone().into(), source.to_owned());
        let lexer = Lexer::new(syntax, EsVersion::latest(), StringInput::from(&*file), None);
        let mut parser = Parser::new_from(lexer);
        let mut program = Program::Module(parser.parse_module().expect("parse failed"));
        assert!(parser.take_errors().is_empty(), "parse diagnostics");
        let unresolved = Mark::new();
        program.visit_mut_with(&mut resolver(unresolved, Mark::new(), typescript));
        let diagnostics = FastDts::new(filename.into(), unresolved, FastDtsOptions::default())
            .transform(&mut program);
        assert!(diagnostics.is_empty(), "isolated declaration diagnostics");
        black_box(to_code_default(cm, None, &program));
    });
}
