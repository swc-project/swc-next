//! Generate JavaScript / TypeScript from a JavaScript / TypeScript file.
//!
//! Run with:
//! `cargo run -p swc_next_ecma_codegen --example codegen -- [filename] [--minify] [--ascii-only] [--preserve-parens]`

use std::{fs, path::Path};

use clap::Parser;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, SourceType};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser as EcmaParser};

#[derive(Parser)]
#[command(about = "Generate JavaScript from a JavaScript or TypeScript file")]
struct Args {
    #[arg(default_value = "input.js")]
    filename: String,

    /// Minify the generated JavaScript.
    #[arg(long)]
    minify: bool,

    /// Escape non-ASCII characters in identifiers and literals.
    #[arg(long)]
    ascii_only: bool,

    /// Preserve parentheses from the input where possible.
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
    let mut ast = Ast::new(source, &allocator);
    let options = ParserOptions {
        source_type,
        lang,
        preserve_parens: args.preserve_parens,
        ..ParserOptions::default()
    };
    let ret = EcmaParser::init(&mut ast, options, NoTokenParserConfig).parse();
    assert!(ret.diagnostics.is_empty());

    // Generate JavaScript from the parsed program
    let options = CodegenOptions {
        minify: args.minify,
        ascii_only: args.ascii_only,
        ..CodegenOptions::default()
    };
    let result = print(&allocator, &ast, ret.program, options);
    print!("{}", result.code);

    Ok(())
}
