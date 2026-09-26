use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_codegen::{Options as PrintOptions, print};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};
use swc_next_isolated_declarations::transform;

struct Fixture {
    name: String,
    category: String,
    path: PathBuf,
    lang: Lang,
    source: String,
    expected: String,
    diagnostics: Vec<ExpectedDiagnostic>,
}

struct ExpectedDiagnostic {
    code: String,
    message: String,
}

#[test]
fn covers_deduplicated_declaration_emit_fixtures() {
    let fixtures = read_fixtures();
    assert_eq!(fixtures.len(), 55);

    let mut names = HashSet::new();
    let mut sources = HashSet::new();
    let mut categories = HashSet::new();
    let mut failures = Vec::new();
    let filter = std::env::var("DTS_FIXTURE").ok();
    let exact_filter = filter
        .as_ref()
        .is_some_and(|filter| fixtures.iter().any(|fixture| &fixture.name == filter));

    for fixture in &fixtures {
        assert!(names.insert(&fixture.name), "duplicate fixture name");
        assert!(sources.insert(&fixture.source), "duplicate fixture source");
        categories.insert(fixture.category.as_str());

        if filter.as_ref().is_none_or(|filter| {
            if exact_filter {
                &fixture.name == filter
            } else {
                fixture.name.contains(filter)
            }
        }) && let Err(error) = run_fixture(fixture)
        {
            failures.push((fixture, error));
        }
    }

    assert_eq!(
        categories,
        HashSet::from(["bindings", "classes", "expressions", "functions",])
    );
    if failures.len() == 1 {
        let (fixture, error) = &failures[0];
        panic!("{} ({})\n{}", fixture.name, fixture.path.display(), error);
    }
    if !failures.is_empty() {
        let summary = failures
            .iter()
            .map(|(fixture, error)| {
                let reason = error.lines().next().unwrap_or("mismatch");
                format!("{}: {reason}", fixture.name)
            })
            .collect::<Vec<_>>()
            .join("\n");
        panic!(
            "{} fixtures failed; set DTS_FIXTURE=<name> for details\n{}",
            failures.len(),
            summary
        );
    }
}

fn read_fixtures() -> Vec<Fixture> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut fixtures = Vec::new();
    for category in sorted_directory_entries(&root) {
        if !category.is_dir() {
            continue;
        }
        let category_name = category.file_name().unwrap().to_string_lossy().into_owned();
        for input in sorted_directory_entries(&category) {
            let file_name = input.file_name().unwrap().to_string_lossy();
            let (name, lang) = if let Some(name) = file_name.strip_suffix(".input.ts") {
                (name, Lang::Ts)
            } else if let Some(name) = file_name.strip_suffix(".input.tsx") {
                (name, Lang::Tsx)
            } else {
                continue;
            };
            let base = category.join(name);
            let source = fs::read_to_string(&input).unwrap();
            fixtures.push(Fixture {
                name: format!("{category_name}/{name}"),
                category: category_name.clone(),
                path: input,
                lang,
                source,
                expected: fs::read_to_string(base.with_extension("output.d.ts")).unwrap(),
                diagnostics: read_diagnostics(&base.with_extension("errors.txt")),
            });
        }
    }
    fixtures
}

fn sorted_directory_entries(path: &Path) -> Vec<PathBuf> {
    let mut entries = fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn read_diagnostics(path: &Path) -> Vec<ExpectedDiagnostic> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (code, message) = line.split_once(": ").unwrap();
            ExpectedDiagnostic {
                code: code.to_owned(),
                message: message.to_owned(),
            }
        })
        .collect()
}

fn run_fixture(fixture: &Fixture) -> Result<(), String> {
    let allocator = Allocator::default();
    let parse = Parser::init(
        &allocator,
        &fixture.source,
        Options {
            lang: fixture.lang,
            source_type: SourceType::Unambiguous,
            comments: CommentMode::Flat,
            ..Options::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    if !parse.diagnostics.is_empty() {
        return Err(format!("parse diagnostics: {:#?}", parse.diagnostics));
    }

    let mut ast = parse.ast;
    let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
    let diagnostics = transform(&mut ast, &semantic)
        .into_iter()
        .map(|diagnostic| diagnostic.message)
        .collect::<Vec<_>>();
    let actual = print(&allocator, &ast, PrintOptions::default()).code;
    let actual = normalize_declaration(actual, !diagnostics.is_empty())?;
    let expected = normalize_declaration(&fixture.expected, !fixture.diagnostics.is_empty())?;
    let expected_diagnostics = fixture
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();

    let mut differences = Vec::new();
    if actual != expected {
        differences.push(format!(
            "declaration mismatch\n--- expected\n{expected}\n--- actual\n{actual}"
        ));
    }
    if diagnostics != expected_diagnostics {
        let expected = fixture
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
            .collect::<Vec<_>>()
            .join("\n");
        differences.push(format!(
            "diagnostic mismatch\n--- expected\n{expected}\n--- actual\n{}",
            diagnostics.join("\n")
        ));
    }
    if differences.is_empty() {
        Ok(())
    } else {
        Err(differences.join("\n"))
    }
}

fn normalize_declaration(source: &str, allow_recovery: bool) -> Result<String, String> {
    let allocator = Allocator::default();
    let parse = Parser::init(
        &allocator,
        source,
        Options {
            lang: Lang::Ts,
            source_type: SourceType::Unambiguous,
            comments: CommentMode::None,
            ..Options::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    if parse.diagnostics.is_empty() {
        return Ok(print(&allocator, &parse.ast, PrintOptions::default())
            .code
            .to_owned());
    }

    if !allow_recovery {
        return Err(format!(
            "invalid declaration output: {:#?}",
            parse.diagnostics
        ));
    }

    let bytes = source.as_bytes();
    let mut normalized = String::with_capacity(source.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            byte if byte.is_ascii_whitespace() => index += 1,
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index += 2;
                while index + 1 < bytes.len() && (bytes[index] != b'*' || bytes[index + 1] != b'/')
                {
                    index += 1;
                }
                index = (index + 2).min(bytes.len());
            }
            quote @ (b'\'' | b'"' | b'`') => {
                normalized.push(quote as char);
                index += 1;
                while index < bytes.len() {
                    let byte = bytes[index];
                    normalized.push(byte as char);
                    index += 1;
                    if byte == b'\\' && index < bytes.len() {
                        normalized.push(bytes[index] as char);
                        index += 1;
                    } else if byte == quote {
                        break;
                    }
                }
            }
            byte => {
                normalized.push(byte as char);
                index += 1;
            }
        }
    }
    Ok(normalized)
}
