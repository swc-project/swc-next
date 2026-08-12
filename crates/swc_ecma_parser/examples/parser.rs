//! Parse a JavaScript or TypeScript file with the next parser.
//!
//! Run with:
//! `cargo run -p swc_next_ecma_parser --example parser -- [filename] [--preserve-parens]`

use std::{fs, path::Path};

use clap::Parser;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_parser::{Options, Parser as EcmaParser, TokenParserConfig};

#[derive(Parser)]
#[command(about = "Parse a JavaScript or TypeScript file with the next parser")]
struct Args {
    #[arg(default_value = "input.js")]
    filename: String,

    /// Represent parenthesized expressions and TypeScript types as explicit nodes.
    #[arg(long)]
    preserve_parens: bool,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    // Read source file
    let path = Path::new(&args.filename);
    let source_text =
        fs::read_to_string(path).map_err(|_| format!("Missing '{}'", path.display()))?;
    let source = source_text.as_str();
    let source_type = SourceType::from_path(&args.filename);
    let lang = Lang::from_path(&args.filename);

    // Parse the source code
    let allocator = Allocator::new();
    let options = Options {
        source_type,
        lang,
        preserve_parens: args.preserve_parens,
        ..Options::default()
    };
    let ret = EcmaParser::init(&allocator, source, options, TokenParserConfig).parse();

    for diagnostic in &ret.diagnostics {
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

    println!(
        "Parsed successfully as {} {:?}: {} nodes, {} extra-data entries.",
        source_type.as_str(),
        lang,
        ret.ast.node_count(),
        ret.ast.extra_data_count(),
    );

    println!("Tokens: {:?}", ret.tokens);

    Ok(())
}
