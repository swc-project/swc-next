//! Parse files or directories with file I/O outside the Callgrind collection region.

use std::{
    env, fs,
    hint::black_box,
    path::{Path, PathBuf},
};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::Lang;
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

fn collect(path: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            if path.file_name().is_some_and(|name| name != ".git") {
                collect(&path, files)?;
            }
        }
    } else if path.extension().is_some_and(|ext| {
        matches!(
            ext.to_str(),
            Some("js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "mts" | "cts")
        )
    }) {
        files.push(path.to_owned());
    }
    Ok(())
}

#[unsafe(no_mangle)]
#[inline(never)]
fn profile_parse(source: &str, lang: Lang) -> (usize, usize, usize) {
    let allocator = Allocator::new();
    let result = Parser::init(
        &allocator,
        black_box(source),
        Options {
            lang,
            ..Options::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    black_box((
        result.ast.node_count() as usize,
        result.ast.extra_data_count() as usize,
        result.diagnostics.len(),
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for path in env::args_os().skip(1) {
        collect(Path::new(&path), &mut files)?;
    }
    files.sort();
    if files.is_empty() {
        return Err("provide JavaScript/TypeScript files or directories".into());
    }
    for path in files {
        let source = fs::read_to_string(&path)?;
        let lang = Lang::from_path(&path.to_string_lossy());
        let (nodes, extra, diagnostics) = profile_parse(&source, lang);
        println!(
            "{}\t{}\t{nodes}\t{extra}\t{diagnostics}",
            path.display(),
            source.len()
        );
    }
    Ok(())
}
