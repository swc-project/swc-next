//! Prettier conformance for the JavaScript, JSX, and TypeScript formatter.
//!
//! The runner reads Prettier's own `runFormatTest` specifications and committed
//! Jest snapshots. This keeps the oracle independent from both our formatter
//! and a locally installed Prettier executable.

use std::{
    any::Any,
    collections::{BTreeMap, BTreeSet},
    env,
    fmt::Write,
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[path = "../../../crates/swc_ecma_formatter/tests/support/prettier_options.rs"]
mod prettier_options;

use prettier_options::{apply_prettier_option, format_options};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde_json::Value;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, CallExpression, Expr, GetSpan, IdentifierName, Lang, NodeKind, ObjectExpression,
    PropertyKey, Severity, SourceType, StringLiteral, VariableDeclarator, Visit, VisitWith,
};
use swc_next_ecma_formatter::{
    FormatError, FormatOptions, PRETTIER_COMPATIBILITY_VERSION, format, format_ast,
};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParserOptions, Parser};

use crate::util::crate_root;

pub const PRETTIER_FIXTURE_REVISION: &str = "cdd17f2288b28b170a76416c72dac56e3ea5daff";

const FORMAT_TEST_SPEC_NAME: &str = "format.test.js";
const SNAPSHOT_DIR_NAME: &str = "__snapshots__";
const SNAPSHOT_FILE_NAME: &str = "format.test.js.snap";
const UNIVERSAL_IGNORE: &str = "/_errors_/";

type OptionSet = BTreeMap<String, Value>;

/// Paths that do not exercise supported whole-file JS/TS formatting.
///
/// Keep this list categorical. A normal formatting mismatch belongs in the
/// report and must not be hidden here.
const IGNORE: &[&str] = &[
    // Parser-only invalid/bogus AST fixtures.
    "typescript/conformance/classes/constructorDeclarations/constructorParameters/readonlyReadonly.ts",
    "typescript/conformance/parser/ecmascript5/Statements/parserES5ForOfStatement21.ts",
    "js/call/invalid",
    "typescript/trailing-comma/invalid.ts",
    "typescript/error-recovery/",
    // Syntax proposals not accepted by the parser.
    "js/optional-chaining-assignment/",
    "js/async-do-expressions/",
    "js/do",
    "jsx/do/",
    "jsx/fbt/",
    "js/export-default/export-default-from",
    "js/export-default/escaped",
    "js/module-blocks",
    "js/tuple",
    "js/record",
    "tuple-and-record.js",
    "jsx/tuple/",
    "js/comments-pipeline-own-line",
    "js/partial-application",
    "js/pipeline-operator",
    "js/arrows-bind",
    "js/bind-expressions",
    "js/objects/expression.js",
    "js/no-semi-babylon-extensions/no-semi.js",
    "js/destructuring-private-fields",
    "js/import-reflection",
    "js/throw_expressions",
    "js/deferred-import-evaluation",
    "js/source-phase-imports",
    "js/babel-plugins",
    // Embedded languages are separate formatters, not JS/TS layout cases.
    "js/comments-closure-typecast/styled-components.js",
    "js/multiparser",
    "typescript/multiparser",
    "typescript/angular-component-examples",
    "js/strings/template-literals.js",
    "js/template-literals/css-prop.js",
    "js/template-literals/styled-components-with-expressions.js",
    "js/template-literals/styled-jsx-with-expressions.js",
    "js/template-literals/styled-jsx.js",
    "js/last-argument-expansion/embed.js",
    "jsx/template/styled-components.js",
    "typescript/as/as-const-embedded.ts",
    "typescript/decorators-ts/angular.ts",
    // Invalid modifier combinations, rejected before formatting.
    "typescript/decorator-auto-accessors/decorator-auto-accessors-abstract-class.ts",
    "typescript/decorator-auto-accessors/decorator-auto-accessors-declare-class.ts",
    "typescript/decorator-auto-accessors/decorator-auto-accessors-mixed-modifiers.ts",
    // These require APIs beyond whole-file formatting.
    "prettier-ignore",
    "js/ignore",
    "range",
    "cursor",
    // Parser-mode differences that do not exercise formatter layout.
    "js/top-level-await",
    "jsx/top-level-await",
    "typescript/top-level-await",
    "js/ternaries/parenthesis/await-expression.js",
    "js/await/like-call.js",
    // Prettier validates this fixture as ES5; our parser uses ES2015+ identifiers.
    "js/quotes/objects.js",
];

#[derive(Clone, Copy)]
pub struct ConformanceConfig {
    pub language: &'static str,
    pub parser_language: Lang,
    pub fixture_roots: &'static [&'static str],
}

pub const JAVASCRIPT: ConformanceConfig = ConformanceConfig {
    language: "js",
    parser_language: Lang::Jsx,
    fixture_roots: &["js", "jsx"],
};

pub const TYPESCRIPT: ConformanceConfig = ConformanceConfig {
    language: "typescript",
    parser_language: Lang::Tsx,
    // Shared JSX fixtures exercise the TypeScript partition as well.
    fixture_roots: &["typescript", "jsx"],
};

pub struct ConformanceOutput {
    pub report: Option<String>,
    pub structural_errors: Vec<String>,
}

#[derive(Clone)]
struct SpecCall {
    parsers: Vec<String>,
    options: OptionSet,
    snapshot_options: Vec<(String, String)>,
    expected_parser_errors: ExpectedParserErrors,
}

#[derive(Clone, Default)]
struct ExpectedParserErrors {
    all: bool,
    javascript: ParserErrorExpectation,
    typescript: ParserErrorExpectation,
}

impl ExpectedParserErrors {
    fn fixture_parser(&self, language: Lang, file_name: &str) -> Option<&'static str> {
        let parser = fixture_parser_key(language);
        let expected = if language.is_typescript() {
            self.typescript.matches(file_name)
        } else {
            self.javascript.matches(file_name)
        };
        (self.all || expected).then_some(parser)
    }
}

#[derive(Clone, Default)]
struct ParserErrorExpectation {
    all: bool,
    files: BTreeSet<String>,
}

impl ParserErrorExpectation {
    fn matches(&self, file_name: &str) -> bool {
        self.all || self.files.contains(file_name)
    }
}

#[derive(Clone)]
struct FormatCase {
    parser_options: ParserOptions,
    file_name: String,
    relative_path: String,
    source: Arc<str>,
    snapshots: Arc<str>,
    expected: String,
    spec: SpecCall,
}

struct Discovery {
    cases: Vec<FormatCase>,
    ignored_files: usize,
    unsupported_specs: Vec<String>,
    structural_errors: Vec<String>,
}

struct CaseResult {
    path: String,
    options: String,
    status: CaseStatus,
    debug_diff: Option<(String, String)>,
    ast_debug_diff: Option<(String, String)>,
    ast_parity: Option<bool>,
    ast_error: Option<String>,
    idempotent: Option<bool>,
    second_pass_error: bool,
}

enum CaseStatus {
    Matched,
    Mismatched,
    FormatError(String),
    ParseError(String),
    ExpectedParserError(String),
    StructuralError(String),
    Panic(String),
}

#[derive(Default)]
struct FileFailures {
    mismatched: usize,
    format_errors: usize,
    panics: usize,
    ast_mismatched: usize,
    ast_errors: usize,
    not_idempotent: usize,
    total: usize,
}

struct FormattedSource {
    text: String,
    ast: Result<String, String>,
    output_invariants: Result<(), String>,
}

enum FormatSourceError {
    Options(String),
    Formatter(FormatError),
}

impl std::fmt::Display for FormatSourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Options(error) => formatter.write_str(error),
            Self::Formatter(error) => error.fmt(formatter),
        }
    }
}

/// Run one language partition of the pinned Prettier suite.
///
/// `PRETTIER_FILTER=<substring>` enables debug mode: only matching input paths
/// run, detailed diffs are printed, and no report is returned for snapshotting.
pub fn run_conformance(config: ConformanceConfig) -> ConformanceOutput {
    let suite_root = crate_root().join("fixtures/prettier");
    let format_root = suite_root.join("tests/format");
    let filter = env::var("PRETTIER_FILTER").ok();
    let mut discovery = discover_cases(config, &suite_root, &format_root, filter.as_deref());

    if !discovery.structural_errors.is_empty() {
        discovery.structural_errors.sort();
        return ConformanceOutput {
            report: None,
            structural_errors: discovery.structural_errors,
        };
    }

    let debug = filter.is_some();
    let cases = std::mem::take(&mut discovery.cases);
    let total_cases = cases.len();
    let completed_cases = AtomicUsize::new(0);
    println!("Discovered {total_cases} formatter comparisons");
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .stack_size(16 * 1024 * 1024)
        .build()
        .expect("formatter conformance thread pool should build");
    let mut results = thread_pool.install(|| {
        cases
            .into_par_iter()
            .map(|case| {
                let result = run_case_catching_panic(&case, debug);
                let completed = completed_cases.fetch_add(1, Ordering::Relaxed) + 1;
                if !debug && (completed.is_multiple_of(500) || completed == total_cases) {
                    println!("Completed {completed}/{total_cases} formatter comparisons");
                }
                result
            })
            .collect::<Vec<_>>()
    });
    results.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.options.cmp(&right.options))
    });

    let mut structural_errors = results
        .iter()
        .filter_map(|result| {
            let CaseStatus::StructuralError(error) = &result.status else {
                return None;
            };
            Some(format!("{}: {error}", result.path))
        })
        .collect::<Vec<_>>();
    if !structural_errors.is_empty() {
        if debug {
            print_debug_results(&results);
        }
        structural_errors.sort();
        return ConformanceOutput {
            report: None,
            structural_errors,
        };
    }

    if filter.is_some() {
        print_debug_results(&results);
        return ConformanceOutput {
            report: None,
            structural_errors: Vec::new(),
        };
    }

    let report = build_report(config, &results, &discovery);
    ConformanceOutput {
        report: Some(report),
        structural_errors: Vec::new(),
    }
}

fn discover_cases(
    config: ConformanceConfig,
    suite_root: &Path,
    format_root: &Path,
    filter: Option<&str>,
) -> Discovery {
    let mut discovery = Discovery {
        cases: Vec::new(),
        ignored_files: 0,
        unsupported_specs: Vec::new(),
        structural_errors: Vec::new(),
    };

    if !format_root.is_dir() {
        discovery.structural_errors.push(format!(
            "Prettier {PRETTIER_COMPATIBILITY_VERSION} fixtures are missing at {}; run ./scripts/clone_fixtures.sh",
            format_root.display()
        ));
        return discovery;
    }

    validate_fixture_version(suite_root, &mut discovery.structural_errors);
    for fixture_root in config.fixture_roots {
        let root = format_root.join(fixture_root);
        if !root.is_dir() {
            discovery.structural_errors.push(format!(
                "Prettier fixture root is missing: {}",
                root.display()
            ));
        }
    }
    if !discovery.structural_errors.is_empty() {
        return discovery;
    }

    let mut test_dirs = collect_test_dirs(config, format_root);
    test_dirs.sort();
    for dir in test_dirs {
        discover_directory(config, format_root, &dir, filter, &mut discovery);
    }

    discovery.unsupported_specs.sort();
    discovery
}

fn validate_fixture_version(suite_root: &Path, errors: &mut Vec<String>) {
    let package_path = suite_root.join("package.json");
    let package_source = match fs::read_to_string(&package_path) {
        Ok(source) => source,
        Err(error) => {
            errors.push(format!("read {}: {error}", package_path.display()));
            return;
        }
    };
    let package: Value = match serde_json::from_str(&package_source) {
        Ok(package) => package,
        Err(error) => {
            errors.push(format!("parse {}: {error}", package_path.display()));
            return;
        }
    };
    let actual = package.get("version").and_then(Value::as_str);
    if actual != Some(PRETTIER_COMPATIBILITY_VERSION) {
        errors.push(format!(
            "Prettier fixture version mismatch: expected {PRETTIER_COMPATIBILITY_VERSION}, found {} in {}",
            actual.unwrap_or("<missing>"),
            package_path.display()
        ));
    }
}

fn collect_test_dirs(config: ConformanceConfig, format_root: &Path) -> Vec<PathBuf> {
    let mut directories = Vec::new();
    for fixture_root in config.fixture_roots {
        let root = format_root.join(fixture_root);
        collect_test_dirs_recursive(&root, &mut directories);
    }
    directories
}

fn collect_test_dirs_recursive(path: &Path, directories: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    let mut children = entries.filter_map(Result::ok).collect::<Vec<_>>();
    children.sort_by_key(fs::DirEntry::file_name);

    if path.join(FORMAT_TEST_SPEC_NAME).is_file()
        && path
            .join(SNAPSHOT_DIR_NAME)
            .join(SNAPSHOT_FILE_NAME)
            .is_file()
    {
        directories.push(path.to_path_buf());
    }

    for child in children {
        if child.file_type().is_ok_and(|file_type| file_type.is_dir())
            && child.file_name() != SNAPSHOT_DIR_NAME
        {
            collect_test_dirs_recursive(&child.path(), directories);
        }
    }
}

fn discover_directory(
    config: ConformanceConfig,
    format_root: &Path,
    dir: &Path,
    filter: Option<&str>,
    discovery: &mut Discovery,
) {
    let spec_path = dir.join(FORMAT_TEST_SPEC_NAME);
    let spec_source = match fs::read_to_string(&spec_path) {
        Ok(source) => source,
        Err(error) => {
            discovery
                .structural_errors
                .push(format!("read {}: {error}", spec_path.display()));
            return;
        }
    };
    let spec_calls = match parse_spec(&spec_source) {
        Ok(calls) => calls,
        Err(error) => {
            discovery.structural_errors.push(format!(
                "parse {}: {error}",
                report_path(&spec_path, format_root)
            ));
            return;
        }
    };
    if spec_calls.is_empty() {
        discovery.structural_errors.push(format!(
            "no runFormatTest call found in {}",
            report_path(&spec_path, format_root)
        ));
        return;
    }

    let snapshot_path = dir.join(SNAPSHOT_DIR_NAME).join(SNAPSHOT_FILE_NAME);
    let snapshots: Arc<str> = match fs::read_to_string(&snapshot_path) {
        Ok(snapshots) => snapshots.into(),
        Err(error) => {
            discovery
                .structural_errors
                .push(format!("read {}: {error}", snapshot_path.display()));
            return;
        }
    };

    let mut supported_specs = Vec::new();
    for spec in spec_calls {
        let unsupported = unsupported_option_names(&spec.options);
        if unsupported.is_empty() {
            supported_specs.push(spec);
        } else {
            discovery.unsupported_specs.push(format!(
                "{} - {}",
                report_path(&spec_path, format_root),
                unsupported.join(", ")
            ));
        }
    }
    if supported_specs.is_empty() {
        return;
    }

    let mut inputs = match fs::read_dir(dir) {
        Ok(entries) => entries.filter_map(Result::ok).collect::<Vec<_>>(),
        Err(error) => {
            discovery
                .structural_errors
                .push(format!("read {}: {error}", dir.display()));
            return;
        }
    };
    inputs.sort_by_key(fs::DirEntry::file_name);

    for input in inputs {
        if !input.file_type().is_ok_and(|file_type| file_type.is_file())
            || input.file_name() == FORMAT_TEST_SPEC_NAME
        {
            continue;
        }
        let path = input.path();
        let relative_path = report_path(&path, format_root);
        if relative_path.contains(UNIVERSAL_IGNORE)
            || IGNORE.iter().any(|ignored| relative_path.contains(ignored))
        {
            discovery.ignored_files += 1;
            continue;
        }
        if filter.is_some_and(|filter| !relative_path.contains(filter)) {
            continue;
        }

        let source: Arc<str> = match fs::read_to_string(&path) {
            Ok(source) => source.into(),
            Err(error) => {
                discovery
                    .structural_errors
                    .push(format!("read {}: {error}", report_path(&path, format_root)));
                continue;
            }
        };
        let file_name = input.file_name().to_string_lossy().into_owned();

        for spec in &supported_specs {
            let Some(expected) =
                find_output_from_snapshots(&snapshots, &file_name, &spec.snapshot_options)
            else {
                discovery.structural_errors.push(format!(
                    "snapshot section missing for {} with {}",
                    relative_path,
                    display_options(&spec.snapshot_options)
                ));
                continue;
            };
            discovery.cases.push(FormatCase {
                parser_options: parser_options(config, &path, &spec.parsers),
                file_name: file_name.clone(),
                relative_path: relative_path.clone(),
                source: source.clone(),
                snapshots: snapshots.clone(),
                expected,
                spec: spec.clone(),
            });
        }
    }
}

fn run_case(case: &FormatCase, debug: bool) -> CaseResult {
    let options = if debug {
        display_options(&case.spec.snapshot_options)
    } else {
        String::new()
    };
    let expected_parser = case
        .spec
        .expected_parser_errors
        .fixture_parser(case.parser_options.lang, &case.file_name);
    let formatted = match format_source(case.parser_options, &case.source, &case.spec.options) {
        Ok(formatted) => formatted,
        Err(FormatSourceError::Options(error)) => {
            // Discovery rejects unsupported option sets before creating cases. Reaching this
            // branch means the discovery/runtime contract changed, not that formatting failed.
            return CaseResult {
                path: case.relative_path.clone(),
                options,
                status: CaseStatus::StructuralError(format!(
                    "format options were not rejected during discovery: {error}"
                )),
                debug_diff: None,
                ast_debug_diff: None,
                ast_parity: None,
                ast_error: None,
                idempotent: None,
                second_pass_error: false,
            };
        }
        Err(FormatSourceError::Formatter(error)) => {
            let status = classify_format_error(error, expected_parser);
            if let Some(parser) = expected_parser
                && matches!(&status, CaseStatus::ExpectedParserError(_))
                && !has_parser_error_snapshot(&case.snapshots, &case.file_name, parser)
            {
                return CaseResult {
                    path: case.relative_path.clone(),
                    options,
                    status: CaseStatus::StructuralError(format!(
                        "expected {parser} parser-error snapshot missing"
                    )),
                    debug_diff: None,
                    ast_debug_diff: None,
                    ast_parity: None,
                    ast_error: None,
                    idempotent: None,
                    second_pass_error: false,
                };
            }
            return CaseResult {
                path: case.relative_path.clone(),
                options,
                status,
                debug_diff: None,
                ast_debug_diff: None,
                ast_parity: None,
                ast_error: None,
                idempotent: None,
                second_pass_error: false,
            };
        }
    };

    let FormattedSource {
        text: actual,
        ast,
        output_invariants,
    } = formatted;
    let (ast_parity, ast_error, ast_debug_diff) = match ast {
        Ok(ast) => {
            let parity = ast == actual;
            let debug_diff = (debug && !parity).then(|| (actual.clone(), ast));
            (Some(parity), None, debug_diff)
        }
        Err(error) => (None, Some(error), None),
    };
    let second = format_text(case.parser_options, &actual, &case.spec.options);
    let idempotent = second.as_deref() == Ok(actual.as_str());
    let second_pass_error = second.is_err();
    let actual = replace_escape_and_eol(
        &actual,
        case.expected.contains("LF>") || case.expected.contains("<CR"),
    );
    let matched = actual == case.expected;
    let invariant_failed = output_invariants.is_err();
    let status = if let Err(error) = output_invariants {
        CaseStatus::StructuralError(error)
    } else if matched {
        CaseStatus::Matched
    } else {
        CaseStatus::Mismatched
    };
    let debug_diff =
        (debug && (!matched || invariant_failed)).then(|| (case.expected.clone(), actual));

    CaseResult {
        path: case.relative_path.clone(),
        options,
        status,
        debug_diff,
        ast_debug_diff,
        ast_parity,
        ast_error,
        idempotent: Some(idempotent),
        second_pass_error,
    }
}

fn classify_format_error(error: FormatError, expected_parser: Option<&'static str>) -> CaseStatus {
    let message = error.to_string();
    match error {
        // The fixture metadata explicitly marks parser failures that are expected for a
        // particular parser. Other parser failures remain comparison failures so they cannot
        // inflate the pass rate by being counted as formatter errors or silently passed.
        FormatError::Parse(_) => match expected_parser {
            Some(_) => CaseStatus::ExpectedParserError(message),
            None => CaseStatus::ParseError(message),
        },
        FormatError::Unsupported { .. } => CaseStatus::FormatError(message),
    }
}

fn run_case_catching_panic(case: &FormatCase, debug: bool) -> CaseResult {
    match catch_unwind(AssertUnwindSafe(|| run_case(case, debug))) {
        Ok(result) => result,
        Err(payload) => CaseResult {
            path: case.relative_path.clone(),
            options: if debug {
                display_options(&case.spec.snapshot_options)
            } else {
                String::new()
            },
            status: CaseStatus::Panic(panic_message(payload.as_ref())),
            debug_diff: None,
            ast_debug_diff: None,
            ast_parity: None,
            ast_error: None,
            idempotent: None,
            second_pass_error: false,
        },
    }
}

fn panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    "non-string panic payload".to_string()
}

fn format_source(
    parser_options: ParserOptions,
    source: &str,
    spec: &OptionSet,
) -> Result<FormattedSource, FormatSourceError> {
    let options = format_options(spec, source).map_err(FormatSourceError::Options)?;
    let text = format(source, parser_options, options).map_err(FormatSourceError::Formatter)?;

    let allocator = Allocator::new();
    let mut ast_parser_options = parser_options;
    ast_parser_options.comments = CommentMode::Flat;
    let parsed = Parser::init(&allocator, source, ast_parser_options, NoTokenParserConfig).parse();
    let ast = if parsed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        Err(format!(
            "cannot format source with {} parse error(s)",
            parsed.diagnostics.len()
        ))
    } else {
        format_ast(&parsed.ast, options).map_err(|error| error.to_string())
    };

    // Entry-point parity compares two printers of the same input AST. Independently
    // require the output to parse and retain the input's comment count. These checks
    // do not establish full semantic equivalence.
    let formatted_allocator = Allocator::new();
    let formatted = Parser::init(
        &formatted_allocator,
        &text,
        ast_parser_options,
        NoTokenParserConfig,
    )
    .parse();
    let output_has_parse_errors = formatted
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error);
    let input_comments = parsed.ast.comments().len();
    let output_comments = formatted.ast.comments().len();
    let output_invariants = if output_has_parse_errors {
        Err("formatted output does not parse".to_string())
    } else if input_comments != output_comments {
        Err(format!(
            "formatting changed comment count: {input_comments} -> {output_comments}"
        ))
    } else {
        Ok(())
    };
    Ok(FormattedSource {
        text,
        ast,
        output_invariants,
    })
}

fn format_text(
    parser_options: ParserOptions,
    source: &str,
    spec: &OptionSet,
) -> Result<String, String> {
    let options = format_options(spec, source)?;
    format(source, parser_options, options).map_err(|error| error.to_string())
}

fn parser_options(config: ConformanceConfig, path: &Path, parsers: &[String]) -> ParserOptions {
    let path = path.to_string_lossy();
    ParserOptions {
        source_type: SourceType::from_path(&path),
        lang: prettier_language(config, &path, parsers),
        ..ParserOptions::default()
    }
}

fn prettier_language(config: ConformanceConfig, path: &str, parsers: &[String]) -> Lang {
    let lang = Lang::from_path(path);
    if lang.is_typescript() {
        return lang;
    }
    // The TypeScript partition also exercises shared JSX fixtures, including
    // specs without an explicit TypeScript parser entry.
    if config.parser_language.is_typescript() {
        return config.parser_language;
    }
    // Prefer the partition's parser regardless of upstream parser ordering.
    // A JS fixture with only typed parsers still needs TypeScript syntax support.
    let mut language = config.parser_language;
    for parser in parsers {
        match parser.as_str() {
            "typescript" | "babel-ts" | "oxc-ts" => language = Lang::Tsx,
            "babel" | "acorn" | "espree" | "meriyah" | "oxc" => return Lang::Jsx,
            _ => {}
        }
    }
    language
}

// Prettier fixture metadata and parser-error snapshot names use these upstream keys.
fn fixture_parser_key(language: Lang) -> &'static str {
    if language.is_typescript() {
        "oxc-ts"
    } else {
        "oxc"
    }
}

fn unsupported_option_names(options: &OptionSet) -> Vec<String> {
    let mut format_options = FormatOptions::prettier();
    options
        .iter()
        .filter_map(|(name, value)| {
            apply_prettier_option(&mut format_options, name, value, "").err()
        })
        .collect()
}

fn is_comparison_result(result: &CaseResult) -> bool {
    !matches!(
        result.status,
        CaseStatus::ExpectedParserError(_) | CaseStatus::StructuralError(_)
    )
}

fn build_report(
    config: ConformanceConfig,
    results: &[CaseResult],
    discovery: &Discovery,
) -> String {
    let parse_errors = results
        .iter()
        .filter(|result| {
            is_comparison_result(result) && matches!(result.status, CaseStatus::ParseError(_))
        })
        .count();
    let expected_parser_errors = results
        .iter()
        .filter(|result| matches!(result.status, CaseStatus::ExpectedParserError(_)))
        .count();
    let total = results
        .iter()
        .filter(|result| is_comparison_result(result))
        .count();
    let fixture_files = results
        .iter()
        .filter(|result| is_comparison_result(result))
        .map(|result| result.path.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let matched = results
        .iter()
        .filter(|result| {
            is_comparison_result(result) && matches!(result.status, CaseStatus::Matched)
        })
        .count();
    let passed = results
        .iter()
        .filter(|result| {
            is_comparison_result(result)
                && matches!(result.status, CaseStatus::Matched)
                && result.ast_parity == Some(true)
                && result.ast_error.is_none()
                && result.idempotent == Some(true)
        })
        .count();
    let mismatched = results
        .iter()
        .filter(|result| {
            is_comparison_result(result) && matches!(result.status, CaseStatus::Mismatched)
        })
        .count();
    let format_errors = results
        .iter()
        .filter(|result| {
            is_comparison_result(result) && matches!(result.status, CaseStatus::FormatError(_))
        })
        .count();
    let panics = results
        .iter()
        .filter(|result| {
            is_comparison_result(result) && matches!(result.status, CaseStatus::Panic(_))
        })
        .count();
    let not_idempotent = results
        .iter()
        .filter(|result| is_comparison_result(result) && result.idempotent == Some(false))
        .count();
    let ast_compared = results
        .iter()
        .filter(|result| is_comparison_result(result) && result.ast_parity.is_some())
        .count();
    let ast_matched = results
        .iter()
        .filter(|result| is_comparison_result(result) && result.ast_parity == Some(true))
        .count();
    let ast_mismatched = results
        .iter()
        .filter(|result| is_comparison_result(result) && result.ast_parity == Some(false))
        .count();
    let ast_errors = results
        .iter()
        .filter(|result| is_comparison_result(result) && result.ast_error.is_some())
        .count();
    let rate = if total == 0 {
        0.0
    } else {
        (passed as f64 / total as f64) * 100.0
    };

    let mut report = format!(
        "formatter * prettier {PRETTIER_COMPATIBILITY_VERSION} * {}\n========================================\n",
        config.language
    );
    writeln!(report, "Fixture revision: {PRETTIER_FIXTURE_REVISION}").unwrap();
    writeln!(report, "Fixture files: {fixture_files}").unwrap();
    writeln!(
        report,
        "Scope: supported physical fixtures (inline snapshot snippets excluded)"
    )
    .unwrap();
    writeln!(report, "Passed: {passed}/{total} ({rate:.2}%)").unwrap();
    writeln!(report, "Snapshot matched: {matched}/{total}").unwrap();
    writeln!(report, "AST parity: {ast_matched}/{ast_compared}").unwrap();
    writeln!(report, "AST mismatched: {ast_mismatched}/{ast_compared}").unwrap();
    writeln!(report, "AST format errors: {ast_errors}/{total}").unwrap();
    writeln!(report, "Mismatched: {mismatched}/{total}").unwrap();
    writeln!(report, "Format errors: {format_errors}/{total}").unwrap();
    writeln!(report, "Panicked: {panics}/{total}").unwrap();
    writeln!(report, "Not idempotent: {not_idempotent}/{total}").unwrap();
    writeln!(report, "Ignored files: {}", discovery.ignored_files).unwrap();
    writeln!(report, "Parse errors: {parse_errors}/{total}").unwrap();
    writeln!(report, "Expected parser errors: {expected_parser_errors}").unwrap();
    writeln!(
        report,
        "Unsupported option sets: {}",
        discovery.unsupported_specs.len()
    )
    .unwrap();

    let mut parse_failures = BTreeMap::<&str, usize>::new();
    for result in results {
        if is_comparison_result(result) && matches!(result.status, CaseStatus::ParseError(_)) {
            *parse_failures.entry(&result.path).or_default() += 1;
        }
    }
    if !parse_failures.is_empty() {
        report.push_str("\n# Parse errors\n\n");
        for (path, count) in &parse_failures {
            writeln!(report, "- {path}: {count} parse error(s)").unwrap();
        }
    }

    let mut expected_parser_failures = BTreeMap::<&str, usize>::new();
    for result in results {
        if !matches!(result.status, CaseStatus::ExpectedParserError(_)) {
            continue;
        }
        *expected_parser_failures.entry(&result.path).or_default() += 1;
    }
    if !expected_parser_failures.is_empty() {
        report.push_str("\n# Expected parser errors\n\n");
        for (path, count) in &expected_parser_failures {
            writeln!(report, "- {path}: {count} expected parser error(s)").unwrap();
        }
    }

    let mut failures = BTreeMap::<&str, FileFailures>::new();
    for result in results {
        if !is_comparison_result(result) {
            continue;
        }
        if matches!(result.status, CaseStatus::ParseError(_)) {
            continue;
        }
        if matches!(result.status, CaseStatus::Matched)
            && result.ast_parity == Some(true)
            && result.ast_error.is_none()
            && result.idempotent == Some(true)
        {
            continue;
        }
        let failure = failures.entry(&result.path).or_default();
        failure.total += 1;
        match result.status {
            CaseStatus::Matched => {}
            CaseStatus::Mismatched => failure.mismatched += 1,
            CaseStatus::FormatError(_) => failure.format_errors += 1,
            CaseStatus::ParseError(_)
            | CaseStatus::ExpectedParserError(_)
            | CaseStatus::StructuralError(_) => unreachable!(),
            CaseStatus::Panic(_) => failure.panics += 1,
        }
        if result.ast_parity == Some(false) {
            failure.ast_mismatched += 1;
        }
        if result.ast_error.is_some() {
            failure.ast_errors += 1;
        }
        if result.idempotent == Some(false) {
            failure.not_idempotent += 1;
        }
    }

    if !failures.is_empty() {
        report.push_str("\n# Failed\n\n");
        for (path, failure) in &failures {
            writeln!(
                report,
                "- {path}: {}/{} mismatched, {}/{} format errors, {}/{} panicked, {}/{} not idempotent",
                failure.mismatched,
                failure.total,
                failure.format_errors,
                failure.total,
                failure.panics,
                failure.total,
                failure.not_idempotent,
                failure.total
            )
            .unwrap();
        }
    }

    let has_ast_failures = failures
        .values()
        .any(|failure| failure.ast_mismatched > 0 || failure.ast_errors > 0);
    if has_ast_failures {
        report.push_str("\n# AST parity failures\n\n");
        for (path, failure) in failures
            .iter()
            .filter(|(_, failure)| failure.ast_mismatched > 0 || failure.ast_errors > 0)
        {
            writeln!(
                report,
                "- {path}: {}/{} mismatched, {}/{} format errors",
                failure.ast_mismatched,
                failure.ast_mismatched + failure.ast_errors,
                failure.ast_errors,
                failure.ast_mismatched + failure.ast_errors
            )
            .unwrap();
        }
    }

    if !discovery.unsupported_specs.is_empty() {
        report.push_str("\n# Unsupported option sets\n\n");
        for spec in &discovery.unsupported_specs {
            writeln!(report, "- {spec}").unwrap();
        }
    }

    report
}

fn print_debug_results(results: &[CaseResult]) {
    for result in results {
        println!("Test: {}", result.path);
        println!("Options: {}", result.options);
        match &result.status {
            CaseStatus::Matched => println!("Snapshot matched"),
            CaseStatus::Mismatched => {
                println!("Snapshot mismatch");
                let (expected, actual) = result
                    .debug_diff
                    .as_ref()
                    .expect("debug mismatch should retain its diff");
                print_text_diff(expected, actual);
            }
            CaseStatus::FormatError(error) => println!("Format error: {error}"),
            CaseStatus::ParseError(error) => println!("Parse error: {error}"),
            CaseStatus::ExpectedParserError(error) => {
                println!("Expected parser error: {error}")
            }
            CaseStatus::StructuralError(error) => {
                println!("Structural error: {error}");
                if let Some((expected, actual)) = &result.debug_diff {
                    print_text_diff(expected, actual);
                }
            }
            CaseStatus::Panic(message) => println!("Panic: {message}"),
        }
        match result.ast_parity {
            Some(true) => println!("AST entry-point parity matched"),
            Some(false) => {
                println!("AST entry-point parity mismatch");
                let (text, ast) = result
                    .ast_debug_diff
                    .as_ref()
                    .expect("debug AST mismatch should retain its diff");
                print_ast_diff(text, ast);
            }
            None => {
                if let Some(error) = &result.ast_error {
                    println!("AST entry-point error: {error}");
                }
            }
        }
        if result.idempotent == Some(false) {
            if result.second_pass_error {
                println!("Not idempotent: second pass failed to format");
            } else {
                println!("Not idempotent: second pass differs");
            }
        }
        println!();
    }
}

fn print_text_diff(expected: &str, actual: &str) {
    println!("--- Prettier");
    println!("{expected}");
    println!("+++ SWC Next");
    println!("{actual}");
}

fn print_ast_diff(text: &str, ast: &str) {
    println!("--- format");
    println!("{text}");
    println!("+++ format_ast");
    println!("{ast}");
}

fn report_path(path: &Path, format_root: &Path) -> String {
    path.strip_prefix(format_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn display_options(options: &[(String, String)]) -> String {
    options
        .iter()
        .filter(|(key, _)| key != "parsers")
        .map(|(key, value)| format!("{key}: {value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn find_output_from_snapshots(
    snapshot: &str,
    file_name: &str,
    snapshot_options: &[(String, String)],
) -> Option<String> {
    let export_prefix = format!("exports[`{file_name} ");
    let mut search_from = 0;
    while let Some(relative_start) = snapshot[search_from..].find(&export_prefix) {
        let block_start = search_from + relative_start;
        let after_start = &snapshot[block_start + export_prefix.len()..];
        let block_length = after_start.find("\nexports[`").unwrap_or(after_start.len());
        let block_end = block_start + export_prefix.len() + block_length;
        let block = &snapshot[block_start..block_end];
        if let Some(output) = find_output_in_snapshot_block(block, snapshot_options) {
            return Some(output);
        }
        search_from = block_end;
    }
    None
}

fn has_parser_error_snapshot(snapshot: &str, file_name: &str, parser: &str) -> bool {
    let prefix = format!("exports[`{file_name} ");
    let parser_marker = format!(" format[{parser}] ");
    snapshot
        .lines()
        .any(|line| line.starts_with(&prefix) && line.contains(&parser_marker))
}

fn find_output_in_snapshot_block(
    snapshot_block: &str,
    snapshot_options: &[(String, String)],
) -> Option<String> {
    let options_pattern = format!(
        "====================================options=====================================\n{}\n",
        snapshot_options
            .iter()
            .map(|(key, value)| format!("{key}: {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let input_marker =
        "\n=====================================input======================================\n";
    let mut search_from = 0;
    let expected = loop {
        let position = snapshot_block[search_from..].find(&options_pattern)?;
        let after_options = search_from + position + options_pattern.len();
        let input_position = snapshot_block[after_options..].find(input_marker)?;
        let between = &snapshot_block[after_options..after_options + input_position];
        if !between.contains('\n') {
            break &snapshot_block[after_options + input_position..];
        }
        search_from = after_options;
    };

    let output_marker =
        "=====================================output=====================================\n";
    let output_started = expected.find(output_marker)? + output_marker.len();
    let output_end_marker =
        "\n================================================================================";
    let output_ended = expected[output_started..].find(output_end_marker)? + output_started;
    Some(expected[output_started..output_ended].to_string())
}

fn replace_escape_and_eol(input: &str, visualize_eol: bool) -> String {
    let input = input
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");
    if !visualize_eol {
        return input;
    }

    let mut chars = input.chars();
    let mut result = String::new();
    while let Some(character) = chars.next() {
        match character {
            '\n' => result.push_str("<LF>\n"),
            '\r' if chars.clone().next() == Some('\n') => {
                chars.next();
                result.push_str("<CRLF>\n");
            }
            '\r' => result.push_str("<CR>\n"),
            _ => result.push(character),
        }
    }
    result
}

fn parse_spec(source: &str) -> Result<Vec<SpecCall>, String> {
    let allocator = Allocator::new();
    let parsed = Parser::init(
        &allocator,
        source,
        ParserOptions {
            source_type: SourceType::Module,
            lang: Lang::Jsx,
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    if let Some(diagnostic) = parsed.diagnostics.first() {
        return Err(format!(
            "{} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        ));
    }

    let mut parser = SpecParser {
        ast: &parsed.ast,
        parsers: Vec::new(),
        calls: Vec::new(),
        errors: Vec::new(),
    };
    parsed.ast.root_program().visit_with(&mut parser);
    if !parser.errors.is_empty() {
        return Err(parser.errors.join("; "));
    }
    Ok(parser.calls)
}

struct SpecParser<'ast, 'source> {
    ast: &'ast Ast<'source>,
    parsers: Vec<String>,
    calls: Vec<SpecCall>,
    errors: Vec<String>,
}

impl<'source> Visit<'source> for SpecParser<'_, 'source> {
    fn ast(&self) -> &Ast<'source> {
        self.ast
    }

    fn visit_variable_declarator(&mut self, declaration: VariableDeclarator) {
        let Some(identifier) = declaration.id(self.ast).as_binding_identifier(self.ast) else {
            declaration.visit_children_with(self);
            return;
        };
        let name = self.ast.get_utf8(identifier.name(self.ast));
        if matches!(name, "parser" | "parsers")
            && let Some(array) = declaration
                .init(self.ast)
                .and_then(|expression| expression.as_array_expression(self.ast))
        {
            self.parsers = string_elements(self.ast, array);
        }
        declaration.visit_children_with(self);
    }

    fn visit_call_expression(&mut self, call: CallExpression) {
        let callee = call.callee(self.ast);
        let is_run_format_test =
            callee
                .as_identifier_reference(self.ast)
                .is_some_and(|identifier| {
                    self.ast.get_utf8(identifier.name(self.ast)) == "runFormatTest"
                });
        if !is_run_format_test {
            call.visit_children_with(self);
            return;
        }

        let arguments = call.arguments(self.ast);
        let Some(parser_argument) = arguments
            .get_node(self.ast, 1)
            .and_then(|argument| argument.as_expr(self.ast))
        else {
            return;
        };
        let parsers = if let Some(array) = parser_argument.as_array_expression(self.ast) {
            string_elements(self.ast, array)
        } else if parser_argument.is_identifier_reference(self.ast) {
            self.parsers.clone()
        } else {
            Vec::new()
        };
        if parsers.is_empty() {
            return;
        }

        let mut options = OptionSet::new();
        let mut snapshot_options = Vec::new();
        let mut expected_parser_errors = ExpectedParserErrors::default();
        if let Some(object) = arguments
            .get_node(self.ast, 2)
            .and_then(|argument| argument.as_expr(self.ast))
            .and_then(|expression| expression.as_object_expression(self.ast))
            && let Err(error) = collect_options(
                self.ast,
                object,
                &mut options,
                &mut snapshot_options,
                &mut expected_parser_errors,
            )
        {
            self.errors.push(error);
            return;
        }
        snapshot_options.push((
            "parsers".to_string(),
            format!(
                "[{}]",
                parsers
                    .iter()
                    .map(|parser| format!("\"{parser}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
        snapshot_options.sort_by(|left, right| left.0.cmp(&right.0));
        self.calls.push(SpecCall {
            parsers,
            options,
            snapshot_options,
            expected_parser_errors,
        });
    }
}

fn string_elements(ast: &Ast<'_>, array: swc_next_ecma_ast::ArrayExpression) -> Vec<String> {
    array
        .elements(ast)
        .iter()
        .filter_map(|element| ast.get_node_in_sub_range(element))
        .filter_map(|argument| argument.as_expr(ast))
        .filter_map(|expression| expression.as_string_literal(ast))
        .map(|literal| string_literal(ast, literal))
        .collect()
}

fn collect_options(
    ast: &Ast<'_>,
    object: ObjectExpression,
    options: &mut OptionSet,
    snapshot_options: &mut Vec<(String, String)>,
    expected_parser_errors: &mut ExpectedParserErrors,
) -> Result<(), String> {
    for property in object.properties(ast).iter() {
        let property = ast.get_node_in_sub_range(property);
        let Some(property) = property.as_object_property(ast) else {
            return Err("spread option is not supported".to_string());
        };
        let Some(name) = property_name(ast, property.key(ast)) else {
            return Err("computed option name is not supported".to_string());
        };
        if name == "errors" {
            collect_expected_parser_errors(ast, property.value(ast), expected_parser_errors)?;
            continue;
        }
        let value = property.value(ast);
        let literal = literal_json(ast, value)
            .ok_or_else(|| format!("non-literal option {name} is not supported"))?;
        options.insert(name.clone(), literal);
        let span = value.span(ast);
        snapshot_options.push((
            name,
            ast.source()[span.start as usize..span.end as usize].to_string(),
        ));
    }
    Ok(())
}

fn collect_expected_parser_errors(
    ast: &Ast<'_>,
    value: Expr,
    expected_parser_errors: &mut ExpectedParserErrors,
) -> Result<(), String> {
    if let Some(literal) = value.as_boolean_literal(ast) {
        expected_parser_errors.all = literal.value(ast);
        return Ok(());
    }

    let Some(object) = value.as_object_expression(ast) else {
        // Prettier only treats `true` and parser-keyed objects as expected
        // errors. Other values (including a top-level filename array) do not
        // match shouldThrowOnFormat in the pinned test runner.
        return Ok(());
    };

    for property in object.properties(ast).iter() {
        let property = ast.get_node_in_sub_range(property);
        let Some(property) = property.as_object_property(ast) else {
            // Unknown parser metadata is intentionally ignored. It cannot
            // affect the language-specific expectations selected by this runner.
            continue;
        };
        let Some(name) = property_name(ast, property.key(ast)) else {
            continue;
        };
        let expectation = match name.as_str() {
            "oxc" => &mut expected_parser_errors.javascript,
            "oxc-ts" => &mut expected_parser_errors.typescript,
            _ => continue,
        };
        collect_parser_error_expectation(ast, property.value(ast), expectation)?;
    }
    Ok(())
}

fn collect_parser_error_expectation(
    ast: &Ast<'_>,
    value: Expr,
    expectation: &mut ParserErrorExpectation,
) -> Result<(), String> {
    if let Some(literal) = value.as_boolean_literal(ast) {
        expectation.all = literal.value(ast);
        expectation.files.clear();
        return Ok(());
    }
    let Some(array) = value.as_array_expression(ast) else {
        // Some Prettier specs compute filename arrays in JavaScript. The
        // static runner only models literal arrays; ignored fixture roots
        // retain their existing discovery behavior for dynamic metadata.
        return Ok(());
    };
    expectation.all = false;
    if let Some(files) = string_array_elements(ast, array) {
        expectation.files.extend(files);
    }
    Ok(())
}

fn string_array_elements(
    ast: &Ast<'_>,
    array: swc_next_ecma_ast::ArrayExpression,
) -> Option<Vec<String>> {
    array
        .elements(ast)
        .iter()
        .map(|element| {
            let element = ast.get_node_in_sub_range(element)?;
            let expression = element.as_expr(ast)?;
            let literal = expression.as_string_literal(ast)?;
            Some(string_literal(ast, literal))
        })
        .collect()
}

fn property_name(ast: &Ast<'_>, key: PropertyKey) -> Option<String> {
    if let Some(identifier) = key.as_identifier_name(ast) {
        return Some(identifier_name(ast, identifier));
    }
    key.as_expr(ast)
        .and_then(|expression| expression.as_string_literal(ast))
        .map(|literal| string_literal(ast, literal))
}

fn literal_json(ast: &Ast<'_>, expression: Expr) -> Option<Value> {
    match ast.node_kind(expression.node_id()) {
        NodeKind::BooleanLiteral => expression
            .as_boolean_literal(ast)
            .map(|literal| Value::Bool(literal.value(ast))),
        NodeKind::NumericLiteral => {
            let value = expression.as_numeric_literal(ast)?.value(ast);
            if value.fract() == 0.0 && value >= 0.0 && value <= u64::MAX as f64 {
                Some(Value::Number((value as u64).into()))
            } else {
                serde_json::Number::from_f64(value).map(Value::Number)
            }
        }
        NodeKind::StringLiteral => expression
            .as_string_literal(ast)
            .map(|literal| Value::String(string_literal(ast, literal))),
        _ => None,
    }
}

fn identifier_name(ast: &Ast<'_>, identifier: IdentifierName) -> String {
    ast.get_utf8(identifier.name(ast)).to_string()
}

fn string_literal(ast: &Ast<'_>, literal: StringLiteral) -> String {
    ast.get_wtf8(literal.value(ast))
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::{JAVASCRIPT, TYPESCRIPT, prettier_language};
    use swc_next_ecma_ast::Lang;

    #[test]
    fn parser_partition_selection() {
        let mixed = vec!["babel".into(), "flow".into(), "typescript".into()];
        assert_eq!(
            prettier_language(JAVASCRIPT, "input.jsx", &mixed),
            Lang::Jsx
        );
        assert_eq!(
            prettier_language(TYPESCRIPT, "input.jsx", &mixed),
            Lang::Tsx
        );

        let reversed = vec!["typescript".into(), "babel".into()];
        assert_eq!(
            prettier_language(JAVASCRIPT, "input.jsx", &reversed),
            Lang::Jsx
        );
        assert_eq!(
            prettier_language(TYPESCRIPT, "input.jsx", &reversed),
            Lang::Tsx
        );

        let typed = vec!["typescript".into()];
        assert_eq!(prettier_language(JAVASCRIPT, "input.js", &typed), Lang::Tsx);

        let javascript = vec!["babel".into()];
        assert_eq!(
            prettier_language(TYPESCRIPT, "input.jsx", &javascript),
            Lang::Tsx
        );
        assert_eq!(
            prettier_language(TYPESCRIPT, "input.ts", &javascript),
            Lang::Ts
        );
        assert_eq!(
            prettier_language(TYPESCRIPT, "input.d.ts", &javascript),
            Lang::Dts
        );
    }
}
