use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Diagnostic, Lang, Severity, SourceType};
use swc_next_ecma_codegen::{Options as PrintOptions, print};
use swc_next_ecma_minifier::{Minifier, MinifierOptions};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParseOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze};

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FixtureOptions {
    source_type: FixtureSourceType,
    minify: MinifierOptions,
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum FixtureSourceType {
    #[default]
    Module,
    Script,
}

impl From<FixtureSourceType> for SourceType {
    fn from(value: FixtureSourceType) -> Self {
        match value {
            FixtureSourceType::Module => Self::Module,
            FixtureSourceType::Script => Self::Script,
        }
    }
}

pub fn discover(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut cases = Vec::new();
    collect(root, &mut cases)?;
    cases.sort();
    if cases.is_empty() {
        return Err(format!("no minifier fixtures found in {}", root.display()));
    }
    Ok(cases)
}

fn collect(directory: &Path, cases: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .and_then(|entries| entries.collect::<io::Result<Vec<_>>>())
        .map_err(|error| format!("read {}: {error}", directory.display()))?;
    // Discover incomplete fixtures too, so missing inputs/outputs fail visibly.
    if entries.iter().any(|entry| {
        matches!(
            entry.file_name().to_str(),
            Some("input.js" | "output.js" | "options.json")
        )
    }) {
        cases.push(directory.to_path_buf());
    }
    for entry in entries {
        let file_type = entry
            .file_type()
            .map_err(|error| format!("stat {}: {error}", entry.path().display()))?;
        if file_type.is_dir() {
            collect(&entry.path(), cases)?;
        }
    }
    Ok(())
}

pub fn run_fixture(directory: &Path) -> Result<(), String> {
    let input = read(&directory.join("input.js"))?;
    let expected = read(&directory.join("output.js"))?;
    let options_path = directory.join("options.json");
    let options = match fs::read_to_string(&options_path) {
        Ok(json) => serde_json::from_str::<FixtureOptions>(&json)
            .map_err(|error| format!("{}: {error}", options_path.display()))?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => FixtureOptions::default(),
        Err(error) => return Err(format!("read {}: {error}", options_path.display())),
    };

    let actual = minify_source(&input, &options).map_err(|error| format!("input: {error}"))?;
    let repeated = minify_source(&actual, &options).map_err(|error| format!("output: {error}"))?;
    if repeated != actual {
        return Err(format!(
            "minification is not stable\nfirst:\n{actual}\nsecond:\n{repeated}"
        ));
    }
    if normalize_newlines(&actual) != normalize_newlines(&expected) {
        return Err(format!(
            "output differs from output.js\nexpected:\n{expected}\nactual:\n{actual}"
        ));
    }
    Ok(())
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn normalize_newlines(source: &str) -> String {
    source
        .replace("\r\n", "\n")
        .trim_end_matches('\n')
        .to_owned()
}

fn minify_source(source: &str, options: &FixtureOptions) -> Result<String, String> {
    let allocator = Allocator::new();
    let result = Parser::init(
        &allocator,
        source,
        ParseOptions {
            source_type: options.source_type.into(),
            lang: Lang::Js,
            preserve_parens: false,
            comments: CommentMode::None,
        },
        NoTokenParserConfig,
    )
    .parse();
    check_diagnostics("parser", &result.diagnostics)?;
    let mut ast = result.ast;
    let analysis = analyze(
        &ast,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
        },
    );
    check_diagnostics("semantic", &analysis.diagnostics)?;
    drop(analysis);

    Minifier::minify(&mut ast, options.minify.clone());
    Ok(print(
        &ast,
        PrintOptions {
            minify: true,
            comments: false,
            ..PrintOptions::default()
        },
    )
    .code)
}

fn check_diagnostics(stage: &str, diagnostics: &[Diagnostic<'_>]) -> Result<(), String> {
    if let Some(error) = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(format!(
            "{stage}: {} at {}..{}",
            error.message, error.span.start, error.span.end
        ));
    }
    Ok(())
}
