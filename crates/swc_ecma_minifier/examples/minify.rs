//! Minifies JavaScript scripts with the default options.
//!
//! ```sh
//! cargo run --release -p swc_next_ecma_minifier --example minify -- <out-dir> <input.js>...
//! ```
//!
//! Each output is written to `<out-dir>` with the input's file name. The
//! output is parsed again, and the run fails if it does not parse.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Diagnostic, Lang, Severity, SourceType};
use swc_next_ecma_codegen::{Options as PrintOptions, print};
use swc_next_ecma_minifier::{Minifier, MinifierOptions};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParseOptions, Parser};

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let (Some(out_dir), inputs) = (args.next().map(PathBuf::from), args.collect::<Vec<_>>()) else {
        eprintln!("usage: minify <out-dir> <input.js>...");
        return ExitCode::FAILURE;
    };
    if let Err(error) = run(&out_dir, inputs.into_iter().map(PathBuf::from)) {
        eprintln!("{error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run(out_dir: &Path, inputs: impl Iterator<Item = PathBuf>) -> Result<(), String> {
    fs::create_dir_all(out_dir).map_err(|error| format!("{}: {error}", out_dir.display()))?;
    for input in inputs {
        let source =
            fs::read_to_string(&input).map_err(|error| format!("{}: {error}", input.display()))?;
        let code = minify(&source).map_err(|error| format!("{}: {error}", input.display()))?;
        check(&code).map_err(|error| format!("{}: output: {error}", input.display()))?;
        let name = input
            .file_name()
            .ok_or_else(|| format!("{}: no file name", input.display()))?;
        let output = out_dir.join(name);
        fs::write(&output, code).map_err(|error| format!("{}: {error}", output.display()))?;
    }
    Ok(())
}

fn parse_options() -> ParseOptions {
    ParseOptions {
        source_type: SourceType::Script,
        lang: Lang::Js,
        preserve_parens: false,
        comments: CommentMode::None,
    }
}

fn minify(source: &str) -> Result<String, String> {
    let allocator = Allocator::new();
    let result = Parser::init(&allocator, source, parse_options(), NoTokenParserConfig).parse();
    check_diagnostics(&result.diagnostics)?;
    let mut ast = result.ast;
    Minifier::minify(&mut ast, MinifierOptions::default());
    let options = PrintOptions {
        minify: true,
        comments: false,
        ..PrintOptions::default()
    };
    Ok(print(&ast, options).code)
}

fn check(code: &str) -> Result<(), String> {
    let allocator = Allocator::new();
    let result = Parser::init(&allocator, code, parse_options(), NoTokenParserConfig).parse();
    check_diagnostics(&result.diagnostics)
}

fn check_diagnostics(diagnostics: &[Diagnostic<'_>]) -> Result<(), String> {
    match diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        Some(error) => Err(format!(
            "{} at {}..{}",
            error.message, error.span.start, error.span.end
        )),
        None => Ok(()),
    }
}
