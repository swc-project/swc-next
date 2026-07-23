//! Parse a JavaScript or TypeScript file with the next parser.
//!
//! Run with:
//! `cargo run -p swc_next_ecma_parser --example parser -- [filename] [--ast] [--comments]`

use std::{fs, path::Path};

use pico_args::Arguments;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, SourceType};
use swc_next_ecma_parser::{Options, Parser};

fn main() -> Result<(), String> {
    let mut args = Arguments::from_env();

    let name = args
        .free_from_str()
        .unwrap_or_else(|_| "test.js".to_string());

    // Read source file
    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let source = source_text.as_bytes();
    let source_type = SourceType::from_path(&name);
    let lang = Lang::from_path(&name);

    // Parse the source code
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let options = Options {
        source_type,
        lang,
        ..Options::default()
    };
    let diagnostics = {
        let mut parser = Parser::init(&mut ast, source, options);
        let _ = parser.parse();
        std::mem::take(&mut parser.diagnostics)
    };

    for diagnostic in &diagnostics {
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
        ast.node_count(),
        ast.extra_data_count(),
    );

    Ok(())
}
