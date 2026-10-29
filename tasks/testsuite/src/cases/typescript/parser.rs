//! TypeScript parser cases with SWC Next diagnostic and virtual-file filters.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use serde::Deserialize;
use swc_next_ecma_ast::{Lang, SourceType, Span};

use crate::cases::{Case, fixtures, json_paths, read_json, swc_conformance};

use super::{
    TestUnit, constants::NOT_SUPPORTED_ERROR_CODES, lang_from_path, read_test_content,
    typescript_case_paths,
};

type UnitErrorCodes = HashMap<String, Vec<HashSet<String>>>;

/// Diagnostics from one configured TypeScript baseline, keyed by the raw
/// virtual file name rendered in that baseline.
struct ErrorCodeVariant {
    name: String,
    error_codes: UnitErrorCodes,
}

/// Option variants for each actual test file, relative to `tests/cases`.
type TestErrorCodes = HashMap<PathBuf, Vec<ErrorCodeVariant>>;
type TestErrorSpans = HashMap<PathBuf, HashMap<String, Vec<Span>>>;

pub struct TypeScriptParserCase {
    path: PathBuf,
    name: String,
    code: String,
    source_type: SourceType,
    lang: Lang,
    expects_error: bool,
    error_spans: Option<Vec<Span>>,
    asserts_semantic_success: bool,
}

impl Case for TypeScriptParserCase {
    const NAME: &'static str = "typescript";

    fn read() -> Vec<Self> {
        let test_dir = fixtures().join("typescript/tsc/testdata/tests/cases");
        let error_codes = read_error_codes();
        let error_spans = read_error_spans();
        let case_paths = compiler_case_paths(&test_dir);
        validate_non_program_test_paths(&case_paths, &test_dir);

        case_paths
            .into_par_iter()
            .flat_map(|path| Self::read_file(path, &test_dir, &error_codes, &error_spans))
            .collect()
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn source_type(&self) -> SourceType {
        self.source_type
    }

    fn lang(&self) -> Lang {
        self.lang
    }

    fn expects_error(&self) -> bool {
        self.expects_error
    }

    fn expected_error_spans(&self) -> Option<&[Span]> {
        self.error_spans.as_deref()
    }

    fn asserts_semantic_success(&self) -> bool {
        self.asserts_semantic_success
    }
}

impl TypeScriptParserCase {
    fn read_file(
        path: PathBuf,
        test_dir: &Path,
        error_codes: &TestErrorCodes,
        error_spans: &TestErrorSpans,
    ) -> Vec<Self> {
        let Ok(mut parsed) = read_test_content(&path) else {
            return Vec::new();
        };
        // TypeScript normalizes relative virtual file names in its baselines.
        for unit in &mut parsed.test_units {
            unit.name = normalize_unit_name(&unit.name);
        }
        let actual_path = path
            .strip_prefix(test_dir)
            .expect("TypeScript case is inside the cases directory")
            .to_path_buf();
        validate_baseline_units(&actual_path, &parsed.test_units, error_codes);
        validate_non_program_units(&actual_path, &parsed.test_units, error_codes);
        let check_js = boolean_option(&parsed.global_options, "checkjs");
        let allow_js = boolean_option(&parsed.global_options, "allowjs")
            .or(check_js)
            .unwrap_or(false);

        let mut unit_occurrences = HashMap::<String, usize>::new();
        parsed
            .test_units
            .into_iter()
            .filter_map(|unit| {
                let occurrence = unit_occurrences.entry(unit.name.clone()).or_default();
                let unit_occurrence = *occurrence;
                *occurrence += 1;
                let lang = lang_from_path(&unit.name)?;
                if !lang.is_typescript() && !allow_js {
                    return None;
                }
                if is_non_program_unit(&actual_path, &unit.name, unit_occurrence) {
                    return None;
                }
                let expects_error = should_fail(
                    &actual_path,
                    &unit.name,
                    unit_occurrence,
                    &parsed.global_options,
                    error_codes,
                );
                let spans = if expects_error {
                    let name = absolute_unit_name(&parsed.current_directory, &unit.name);
                    error_spans
                        .get(&actual_path)
                        .and_then(|files| files.get(&name))
                        .cloned()
                } else {
                    None
                };
                if let Some(spans) = &spans {
                    assert!(
                        spans.iter().all(|span| span.start <= span.end
                            && span.end as usize <= unit.content.len()),
                        "invalid diagnostic spans: {} - {}",
                        path.display(),
                        unit.name
                    );
                }
                let asserts_semantic_success =
                    asserts_semantic_success(lang, &unit.content, check_js);
                Some(Self {
                    path: path.clone(),
                    source_type: SourceType::from_path(&unit.name),
                    lang,
                    name: unit.name,
                    code: unit.content,
                    expects_error,
                    error_spans: spans,
                    asserts_semantic_success,
                })
            })
            .collect()
    }
}

fn compiler_case_paths(test_dir: &Path) -> Vec<PathBuf> {
    typescript_case_paths(test_dir)
        .into_iter()
        .filter(|path| {
            let actual_path = path.strip_prefix(test_dir).unwrap();
            !IGNORED_TESTS
                .iter()
                .any(|ignored| actual_path == Path::new(ignored))
        })
        .collect()
}

fn validate_non_program_test_paths(case_paths: &[PathBuf], test_dir: &Path) {
    let actual_paths = case_paths
        .iter()
        .map(|path| {
            path.strip_prefix(test_dir)
                .expect("TypeScript compiler case is inside the cases directory")
        })
        .collect::<HashSet<_>>();
    let mut identities = HashSet::new();
    for unit in NON_PROGRAM_UNITS {
        assert!(
            identities.insert((unit.test, unit.name, unit.occurrence)),
            "duplicate non-Program TypeScript unit: {} - {} #{}",
            unit.test,
            unit.name,
            unit.occurrence
        );
        assert!(
            actual_paths.contains(Path::new(unit.test)),
            "non-Program TypeScript unit refers to missing test `{}`",
            unit.test
        );
    }
}

fn validate_non_program_units(
    actual_path: &Path,
    test_units: &[TestUnit],
    error_codes: &TestErrorCodes,
) {
    let mut unit_counts = HashMap::<&str, usize>::new();
    for unit in test_units {
        *unit_counts.entry(&unit.name).or_default() += 1;
    }
    for unit in NON_PROGRAM_UNITS
        .iter()
        .filter(|unit| actual_path == Path::new(unit.test))
    {
        assert!(
            unit_counts
                .get(unit.name)
                .is_some_and(|count| unit.occurrence < *count),
            "non-Program TypeScript unit no longer exists: {} - {} #{}",
            unit.test,
            unit.name,
            unit.occurrence
        );
        if let Some(variants) = error_codes.get(actual_path) {
            for variant in variants {
                let codes = &variant.error_codes[unit.name][unit.occurrence];
                assert!(
                    codes.is_empty(),
                    "non-Program TypeScript unit has diagnostics in baseline `{}`: {} - {} #{}",
                    variant.name,
                    unit.test,
                    unit.name,
                    unit.occurrence
                );
            }
        }
    }
}

fn is_non_program_unit(actual_path: &Path, name: &str, occurrence: usize) -> bool {
    NON_PROGRAM_UNITS.iter().any(|unit| {
        actual_path == Path::new(unit.test) && name == unit.name && occurrence == unit.occurrence
    })
}

fn boolean_option(options: &HashMap<String, String>, name: &str) -> Option<bool> {
    match options.get(name)?.trim().to_ascii_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Whether TypeScript's lack of diagnostics is an oracle for scope-dependent
/// diagnostics in this virtual file. TypeScript files are checked by default.
/// JavaScript files require global `checkJs` or a leading `// @ts-check`; the
/// last leading `// @ts-check` / `// @ts-nocheck` directive overrides the
/// global setting.
fn asserts_semantic_success(lang: Lang, code: &str, check_js: Option<bool>) -> bool {
    if let Some(check) = file_check_js_directive(code) {
        return check;
    }
    lang.is_typescript() || check_js.unwrap_or(false)
}

fn file_check_js_directive(code: &str) -> Option<bool> {
    let bytes = code.as_bytes();
    let mut position = usize::from(bytes.starts_with(&[0xef, 0xbb, 0xbf])) * 3;
    let mut directive = None;

    loop {
        while bytes.get(position).is_some_and(u8::is_ascii_whitespace) {
            position += 1;
        }

        if bytes.get(position..position + 2) == Some(b"//") {
            let end = bytes[position..]
                .iter()
                .position(|&byte| matches!(byte, b'\n' | b'\r'))
                .map_or(bytes.len(), |offset| position + offset);
            directive = parse_check_js_directive(&code[position..end]).or(directive);
            position = end;
            continue;
        }

        if bytes.get(position..position + 2) == Some(b"/*") {
            let comment = &bytes[position + 2..];
            let Some(offset) = comment.windows(2).position(|window| window == b"*/") else {
                break;
            };
            position += offset + 4;
            continue;
        }

        break;
    }

    directive
}

fn parse_check_js_directive(comment: &str) -> Option<bool> {
    let content = comment.strip_prefix("//")?;
    let content = content.strip_prefix('/').unwrap_or(content).trim_start();
    let pragma = content.strip_prefix('@')?;
    let name_end = pragma
        .find(|character: char| character.is_ascii_whitespace() || character == ':')
        .unwrap_or(pragma.len());
    match &pragma[..name_end] {
        name if name.eq_ignore_ascii_case("ts-check") => Some(true),
        name if name.eq_ignore_ascii_case("ts-nocheck") => Some(false),
        _ => None,
    }
}

#[derive(Deserialize)]
struct Baseline {
    path: PathBuf,
    variants: Vec<Variant>,
}

#[derive(Deserialize)]
struct Variant {
    name: String,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Deserialize)]
struct Diagnostic {
    file: Option<String>,
    codes: Vec<i32>,
}

fn read_error_codes() -> TestErrorCodes {
    let mut error_codes = TestErrorCodes::new();
    for suite in ["compiler", "conformance"] {
        for path in json_paths(&format!("typescript/{suite}")) {
            let input: Baseline = read_json(&path);
            let mut variants = Vec::new();
            for variant in input.variants {
                if variant.diagnostics.is_empty() {
                    continue;
                }
                let mut codes = UnitErrorCodes::new();
                for diagnostic in variant.diagnostics {
                    if let Some(file) = diagnostic.file {
                        let name = normalize_unit_name(&file);
                        let codes = codes.entry(name).or_default();
                        codes.push(diagnostic.codes.iter().map(ToString::to_string).collect());
                    }
                }
                variants.push(ErrorCodeVariant {
                    name: variant.name,
                    error_codes: codes,
                });
            }
            error_codes.insert(input.path, variants);
        }
    }
    error_codes
}

#[derive(Deserialize)]
struct ReferenceCase {
    snapshot: PathBuf,
}

#[derive(Deserialize)]
struct ReferenceSnapshot {
    path: PathBuf,
    diagnostics: Vec<ReferenceDiagnostic>,
}

#[derive(Deserialize)]
struct ReferenceDiagnostic {
    file: String,
    code: i32,
    start: u32,
    end: u32,
}

fn read_error_spans() -> TestErrorSpans {
    let root = swc_conformance();
    let cases: Vec<ReferenceCase> = read_json(&root.join("fixtures-json/runners/tsrs.json"));
    let root = root.join("tests/parser/typescript");
    let mut output = TestErrorSpans::new();
    for case in cases {
        let snapshot: ReferenceSnapshot = read_json(&root.join(case.snapshot));
        for diagnostic in snapshot.diagnostics {
            if NOT_SUPPORTED_ERROR_CODES.contains(&diagnostic.code.to_string().as_str()) {
                continue;
            }
            let files = output.entry(snapshot.path.clone()).or_default();
            let spans = files.entry(diagnostic.file).or_default();
            let span = Span::new(diagnostic.start, diagnostic.end);
            if !spans.contains(&span) {
                spans.push(span);
            }
        }
    }
    output
}

fn absolute_unit_name(directory: &str, name: &str) -> String {
    let path = if name.starts_with('/') {
        name.to_owned()
    } else {
        format!("{directory}/{name}")
    };
    let path = path.replace('\\', "/");
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    format!("/{}", parts.join("/"))
}

fn normalize_unit_name(name: &str) -> String {
    Path::new(name.trim_start_matches("./"))
        .components()
        .collect::<PathBuf>()
        .to_string_lossy()
        .replace('\\', "/")
}

fn validate_baseline_units(
    actual_path: &Path,
    test_units: &[TestUnit],
    error_codes: &TestErrorCodes,
) {
    let Some(variants) = error_codes.get(actual_path) else {
        return;
    };
    let mut unit_counts = HashMap::<&str, usize>::new();
    for unit in test_units {
        // Content-mapped assets can be omitted from upstream error baselines.
        // Validate the JavaScript and TypeScript units that this runner parses.
        if lang_from_path(&unit.name).is_none() {
            continue;
        }
        *unit_counts.entry(unit.name.as_str()).or_default() += 1;
    }
    for variant in variants {
        let baseline_counts = variant
            .error_codes
            .iter()
            .filter(|(name, _)| lang_from_path(name).is_some())
            .map(|(name, occurrences)| (name.as_str(), occurrences.len()))
            .collect::<HashMap<_, _>>();
        assert_eq!(
            baseline_counts,
            unit_counts,
            "TypeScript baseline `{}` virtual files do not match `{}`",
            variant.name,
            actual_path.display()
        );
    }
}

fn should_fail(
    actual_path: &Path,
    unit_name: &str,
    unit_occurrence: usize,
    global_options: &HashMap<String, String>,
    error_codes: &TestErrorCodes,
) -> bool {
    if global_options.contains_key("negative") {
        return true;
    }

    let normalized_path = actual_path.to_string_lossy().replace('\\', "/");
    if normalized_path.contains("/negative/") {
        return true;
    }

    error_codes.get(actual_path).is_some_and(|variants| {
        variants.iter().any(|variant| {
            variant
                .error_codes
                .get(unit_name)
                .and_then(|occurrences| occurrences.get(unit_occurrence))
                .is_some_and(|codes| {
                    codes
                        .iter()
                        .any(|code| !NOT_SUPPORTED_ERROR_CODES.contains(&code.as_str()))
                })
        })
    })
}

const IGNORED_TESTS: &[&str] = &[
    "compiler/binderBinaryExpressionStress.ts",
    "compiler/binderBinaryExpressionStressJs.ts",
];

struct NonProgramUnit {
    test: &'static str,
    name: &'static str,
    occurrence: usize,
}

/// TypeScript error baselines render every input unit, including package
/// redirects and resolution alternatives that are not SourceFiles in the
/// Program. Their zero-error sections are not positive parser or semantic
/// oracles. Keep the exceptional identities exact and fail if fixtures drift.
const NON_PROGRAM_UNITS: &[NonProgramUnit] = &[
    NonProgramUnit {
        test: "compiler/duplicatePackage.ts",
        name: "/node_modules/b/node_modules/x/index.d.ts",
        occurrence: 0,
    },
    NonProgramUnit {
        test: "compiler/duplicatePackage_withErrors.ts",
        name: "/node_modules/b/node_modules/x/index.d.ts",
        occurrence: 0,
    },
    NonProgramUnit {
        test: "conformance/moduleResolution/resolutionModeTripleSlash1.ts",
        name: "/node_modules/@types/foo/index.d.cts",
        occurrence: 0,
    },
    NonProgramUnit {
        test: "conformance/moduleResolution/resolutionModeTripleSlash3.ts",
        name: "/node_modules/@types/foo/index.d.cts",
        occurrence: 0,
    },
    NonProgramUnit {
        test: "conformance/moduleResolution/resolutionModeTripleSlash4.ts",
        name: "/node_modules/@types/foo/index.d.cts",
        occurrence: 0,
    },
    NonProgramUnit {
        test: "conformance/moduleResolution/resolutionModeTripleSlash5.ts",
        name: "/node_modules/@types/foo/index.d.cts",
        occurrence: 0,
    },
];
