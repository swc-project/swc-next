//! Transform a JavaScript or TypeScript file and print the result.
//!
//! Run with:
//! `cargo run -p swc_next_ecma_transformer --example transformer -- [filename]`
//! Class static block transforms are enabled; codegen uses default options.
//! TypeScript is lowered automatically; JSX is preserved.

use std::{fs, path::Path};

use clap::Parser;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Diagnostic, Lang, Severity, SourceType};
use swc_next_ecma_codegen::print;
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser as EcmaParser};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze};
use swc_next_ecma_transformer::{
    Transformer, TransformerOptions, es2022::ES2022Options, normalize::Normalizer,
    proposal::ProposalOptions, typescript::TypeScriptOptions,
};

#[derive(Parser)]
#[command(about = "Transform a JavaScript or TypeScript file and print the result")]
struct Args {
    #[arg(default_value = "input.js")]
    filename: String,
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let path = Path::new(&args.filename);
    let source = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read '{}': {error}", path.display()))?;
    let lang = Lang::from_path(&args.filename);
    let allocator = Allocator::new();
    let parsed = EcmaParser::init(
        &allocator,
        &source,
        ParserOptions {
            source_type: SourceType::from_path(&args.filename),
            lang,
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    report_diagnostics(path, &parsed.diagnostics)?;

    let mut ast = parsed.ast;
    let options = TransformerOptions {
        es2022: ES2022Options {
            static_blocks: true,
        },
        proposal: ProposalOptions {
            prevent_extensions: true,
        },
        typescript: lang.is_typescript().then_some(TypeScriptOptions {
            verbatim_module_syntax: true,
            preserve_const_enums: true,
            ..TypeScriptOptions::default()
        }),
        ..TransformerOptions::default()
    };
    if options.typescript.is_some() {
        Normalizer::normalize_runtime(ast.root_program(), &mut ast);
    } else {
        Normalizer::normalize(ast.root_program(), &mut ast);
    }
    let analyzed = analyze(
        &ast,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
            build_module_record: false,
        },
    );
    report_diagnostics(path, &analyzed.diagnostics)?;
    Transformer::transform(&mut ast, analyzed.semantic, options);

    let result = print(&ast, Default::default());
    print!("{}", result.code);
    Ok(())
}

fn report_diagnostics(path: &Path, diagnostics: &[Diagnostic<'_>]) -> Result<(), String> {
    for diagnostic in diagnostics {
        eprintln!(
            "{}:{}..{}: {}: {}",
            path.display(),
            diagnostic.span.start,
            diagnostic.span.end,
            diagnostic.severity.as_str(),
            diagnostic.message,
        );
        if let Some(help) = &diagnostic.help {
            eprintln!("  help: {help}");
        }
        for label in &diagnostic.labels {
            eprintln!(
                "  {}..{}: {}",
                label.span.start, label.span.end, label.message
            );
        }
    }
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(format!(
            "Cannot transform '{}' due to errors",
            path.display()
        ));
    }
    Ok(())
}
