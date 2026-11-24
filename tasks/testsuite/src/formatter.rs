//! Prettier conformance for JavaScript, JSX, TypeScript and Markdown formatting.
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

mod divergences;
mod markdown;
mod options;

pub use markdown::run_markdown_conformance;

use divergences::KNOWN_DIVERGENCES;
use options::{format_options, unsupported_option_names};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde_json::Value;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, CallExpression, Expr, GetSpan, IdentifierName, Lang, NodeKind, ObjectExpression,
    PropertyKey, Severity, SourceType, StringLiteral, VariableDeclarator, Visit, VisitWith,
};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_formatter::js::{format, format_ast};
use swc_next_formatter_js::{FormatError, PRETTIER_COMPATIBILITY_VERSION};

use crate::util::crate_root;

pub const PRETTIER_FIXTURE_REVISION: &str = "deda65fd47c285b974e2a4397f26a33cfc1746b8";

const FORMAT_TEST_SPEC_NAME: &str = "format.test.js";
const SNAPSHOT_DIR_NAME: &str = "__snapshots__";
const SNAPSHOT_FILE_NAME: &str = "format.test.js.snap";

type OptionSet = BTreeMap<String, Value>;

enum IgnorePattern {
    Contains(&'static str),
    Exact(&'static str),
}

impl IgnorePattern {
    fn matches(&self, path: &str) -> bool {
        match self {
            Self::Contains(pattern) => path.contains(*pattern),
            Self::Exact(pattern) => path == *pattern,
        }
    }
}

/// Fixture paths excluded from whole-file JS/TS conformance.
///
/// Keep scope exclusions categorical and snapshot exceptions narrowly qualified.
/// Small mixed suites may retain supported controls to keep exclusions compact.
/// Ordinary formatting mismatches belong in the report.
const IGNORE: &[IgnorePattern] = &[
    // Parser-error fixtures.
    IgnorePattern::Contains("/_errors_/"),
    // Parser-only invalid/bogus AST fixtures.
    IgnorePattern::Contains(
        "typescript/conformance/classes/constructorDeclarations/constructorParameters/readonlyReadonly.ts",
    ),
    IgnorePattern::Contains("js/call-expression/invalid/"),
    IgnorePattern::Contains("typescript/trailing-comma/invalid.ts"),
    IgnorePattern::Contains("typescript/error-recovery/"),
    // Syntax proposals not accepted by the parser.
    IgnorePattern::Contains("js/optional-chaining-assignment/"),
    IgnorePattern::Contains("js/do-expression/"),
    IgnorePattern::Contains("jsx/do-expression/"),
    IgnorePattern::Contains("js/export-default/export-default-from"),
    IgnorePattern::Contains("js/export-default/escaped"),
    IgnorePattern::Contains("js/module-block/"),
    IgnorePattern::Contains("js/partial-application"),
    IgnorePattern::Contains("js/pipeline-operator"),
    IgnorePattern::Contains("js/arrow-function/arrows-bind/"),
    IgnorePattern::Contains("js/bind-expression/"),
    IgnorePattern::Contains("js/objects/expression.js"),
    IgnorePattern::Contains("js/destructuring-private-fields"),
    IgnorePattern::Contains("js/throw-expression/"),
    // FBT-specific JSX whitespace formatting is not implemented.
    IgnorePattern::Contains("jsx/fbt/"),
    // Embedded languages are separate formatters, not JS/TS layout cases.
    IgnorePattern::Contains("js/comments-closure-typecast/styled-components.js"),
    IgnorePattern::Contains("js/multiparser"),
    IgnorePattern::Contains("typescript/multiparser"),
    IgnorePattern::Contains("typescript/angular-component-examples"),
    IgnorePattern::Contains("js/template-literals/css-prop.js"),
    // This comparison requires the separate embedded HTML formatter.
    IgnorePattern::Contains("js/template-literals/expression-break.js"),
    IgnorePattern::Contains("js/template-literals/styled-components-with-expressions.js"),
    IgnorePattern::Contains("js/template-literals/styled-jsx-with-expressions.js"),
    IgnorePattern::Contains("js/template-literals/styled-jsx.js"),
    IgnorePattern::Contains("js/last-argument-expansion/embed.js"),
    IgnorePattern::Contains("jsx/template/styled-components.js"),
    IgnorePattern::Contains("typescript/as/as-const-embedded.ts"),
    IgnorePattern::Contains("typescript/decorators-ts/angular.ts"),
    // Range and cursor fixtures use selection markers and position-mapping APIs.
    IgnorePattern::Contains("/range/"),
    IgnorePattern::Contains("js/eol/range-"),
    IgnorePattern::Exact("js/new-target/range.js"),
    IgnorePattern::Contains("cursor"),
    // Parser-mode differences that do not exercise formatter layout.
    IgnorePattern::Exact("js/top-level-await/test.js"),
    IgnorePattern::Exact("jsx/top-level-await/test.jsx"),
    IgnorePattern::Exact("typescript/top-level-await/test.ts"),
    IgnorePattern::Exact("typescript/top-level-await/test.tsx"),
    IgnorePattern::Exact("js/ternaries/parenthesis/await-expression.js"),
    IgnorePattern::Exact("js/await-expression/like-call.js"),
    // This mixed fixture uses await in non-async functions and class fields,
    // which our parser rejects.
    IgnorePattern::Exact("js/await-expression/await-with-parens.js"),
    // TODO: embedded-language formatting; keep template text unchanged for now.
    IgnorePattern::Exact("js/embeded/indention/19518.js"),
    IgnorePattern::Exact("js/embeded/indention/indention-2.js"),
    IgnorePattern::Exact("js/embeded/indention/indention.js"),
    // TODO: Jest each tagged-template table formatting.
    IgnorePattern::Exact("js/test-declarations/jest-each.js"),
    IgnorePattern::Exact("js/test-declarations/jest-each-template-string.js"),
    // TODO: keep the leading comment before the original return operand.
    IgnorePattern::Exact("js/comments/return-statement.js"),
    // TODO: preserve the inner union's leading comment boundary when unwrapping.
    IgnorePattern::Exact("typescript/union/consistent-with-flow/single-type.ts"),
    // TODO: keep pre-argument comments before the call's opening parenthesis.
    IgnorePattern::Exact("typescript/call/callee-comments.ts"),
    // TODO: keep operand ignore markers after the spread token.
    IgnorePattern::Exact("jsx/ignore/spread.js"),
    // TODO: preserve comments around the optional marker and parameter opener.
    IgnorePattern::Exact("typescript/comments/method_types.ts"),
    // TODO: keep arrow-body comments before the source body brace.
    IgnorePattern::Exact("js/arrow-function/issue-17421.js"),
    // The difference is line layout for block comments on arrow body heads.
    IgnorePattern::Exact("typescript/arrow-function/comments/issue-11100.ts"),
    // TODO: keep type assertion tails before the statement terminator.
    IgnorePattern::Exact("typescript/as/as-const/as-const.ts"),
    // TODO: keep the full verbatim parenthesized type. The matched snapshots
    // currently drop the source parentheses.
    IgnorePattern::Exact("typescript/prettier-ignore/prettier-ignore-parenthesized-type.ts"),
    IgnorePattern::Exact(
        "typescript/prettier-ignore/no-semi/prettier-ignore-parenthesized-type.ts",
    ),
    // TODO: preserve the side of the source terminator after continue and break.
    IgnorePattern::Exact("js/for-statement/continue-and-break-comment-without-blocks.js"),
    // The difference is the line break before a block comment in a function head.
    IgnorePattern::Exact("js/trailing-comma/trailing_whitespace.js"),
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
}

#[derive(Clone)]
struct FormatCase {
    parser_options: ParserOptions,
    relative_path: String,
    source: Arc<str>,
    expected: Option<String>,
    expected_parser_error: bool,
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

impl CaseResult {
    fn is_valid_output(&self) -> bool {
        matches!(self.status, CaseStatus::Matched | CaseStatus::Mismatched)
            && self.ast_parity == Some(true)
            && self.ast_error.is_none()
            && self.idempotent == Some(true)
    }
}

enum CaseStatus {
    Matched,
    Mismatched,
    FormatError(String),
    ParseError(String),
    ExpectedParserError(String),
    UnparsableOutput(&'static str),
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
    output_invariants: Result<(), OutputInvariantError>,
}

enum OutputInvariantError {
    Unparsable,
    CommentCount { input: usize, output: usize },
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
        .unwrap();
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

    ConformanceOutput {
        report: Some(build_report(config, &results, &discovery)),
        structural_errors: KNOWN_DIVERGENCES.stale_errors(config, &results),
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
    let snapshots = match fs::read_to_string(&snapshot_path) {
        Ok(snapshots) => snapshots,
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
        if IGNORE.iter().any(|pattern| pattern.matches(&relative_path)) {
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
            let parser_options = parser_options(config, &path, &spec.parsers);
            let expected =
                find_output_from_snapshots(&snapshots, &file_name, &spec.snapshot_options);
            // The parser-qualified snapshots record rejections for their exact options,
            // including specs that compute their `errors` metadata in JavaScript.
            let expected_parser_error = has_parser_error_snapshot(
                &snapshots,
                &file_name,
                fixture_parser_key(parser_options.lang),
                &spec.options,
            );
            if expected.is_none() && !expected_parser_error {
                discovery.structural_errors.push(format!(
                    "snapshot section missing for {} with {}",
                    relative_path,
                    display_options(&spec.snapshot_options)
                ));
                continue;
            }
            discovery.cases.push(FormatCase {
                parser_options,
                relative_path: relative_path.clone(),
                source: source.clone(),
                expected,
                expected_parser_error,
                spec: spec.clone(),
            });
        }
    }
}

fn run_case(case: &FormatCase, debug: bool) -> CaseResult {
    let options = display_options(&case.spec.snapshot_options);
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
            let status = classify_format_error(error, case.expected_parser_error);
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
    let expected = case.expected.as_deref();
    let visualize_eol = expected.is_some_and(|text| text.contains("LF>") || text.contains("<CR"));
    let actual = replace_escape_and_eol(&actual, visualize_eol);
    let matched = expected == Some(actual.as_str());
    let invariant_failed = output_invariants.is_err();
    let status = match output_invariants {
        Ok(()) if matched => CaseStatus::Matched,
        Ok(()) => CaseStatus::Mismatched,
        Err(error) => classify_output_invariant_error(case, error),
    };
    let debug_diff = (debug && (!matched || invariant_failed))
        .then(|| (expected.unwrap_or("Parser rejection").to_string(), actual));

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

fn classify_format_error(error: FormatError, expected_parser_error: bool) -> CaseStatus {
    let message = error.to_string();
    match error {
        FormatError::Parse(_) if expected_parser_error => CaseStatus::ExpectedParserError(message),
        FormatError::Parse(_) => CaseStatus::ParseError(message),
        FormatError::Unsupported { .. } => CaseStatus::FormatError(message),
    }
}

fn classify_output_invariant_error(case: &FormatCase, error: OutputInvariantError) -> CaseStatus {
    let message = match error {
        OutputInvariantError::Unparsable
            if case.relative_path == "typescript/instantiation-expression/before-operator.ts"
                && case.spec.options.is_empty() =>
        {
            return CaseStatus::UnparsableOutput(
                "the formatter drops required parentheses around instantiation expressions before binary operators",
            );
        }
        OutputInvariantError::Unparsable => "formatted output does not parse".to_string(),
        OutputInvariantError::CommentCount { input, output } => {
            format!("formatting changed comment count from {input} to {output}")
        }
    };
    CaseStatus::StructuralError(message)
}

fn run_case_catching_panic(case: &FormatCase, debug: bool) -> CaseResult {
    match catch_unwind(AssertUnwindSafe(|| run_case(case, debug))) {
        Ok(result) => result,
        Err(payload) => CaseResult {
            path: case.relative_path.clone(),
            options: display_options(&case.spec.snapshot_options),
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
    let output_invariants = if input_comments != output_comments {
        Err(OutputInvariantError::CommentCount {
            input: input_comments,
            output: output_comments,
        })
    } else if output_has_parse_errors {
        Err(OutputInvariantError::Unparsable)
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

// Prettier's parser-error snapshot names use these upstream keys.
fn fixture_parser_key(language: Lang) -> &'static str {
    if language.is_typescript() {
        "oxc-ts"
    } else {
        "oxc"
    }
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
    let unparsable_outputs = results
        .iter()
        .filter_map(|result| match result.status {
            CaseStatus::UnparsableOutput(reason) => Some((result.path.as_str(), reason)),
            _ => None,
        })
        .collect::<Vec<_>>();
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
        .filter(|result| matches!(result.status, CaseStatus::Matched) && result.is_valid_output())
        .count();
    let mismatched = results
        .iter()
        .filter(|result| {
            is_comparison_result(result) && matches!(result.status, CaseStatus::Mismatched)
        })
        .count();
    let mut known_divergences = BTreeMap::<&str, Vec<&CaseResult>>::new();
    for result in results {
        if let Some(reason) = KNOWN_DIVERGENCES.reason_for(result) {
            known_divergences.entry(reason).or_default().push(result);
        }
    }
    let known = known_divergences.values().map(Vec::len).sum::<usize>();
    let unclassified = mismatched - known;
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
    writeln!(report, "Known mismatches: {known}/{total}").unwrap();
    writeln!(report, "Unclassified mismatches: {unclassified}/{total}").unwrap();
    writeln!(report, "Format errors: {format_errors}/{total}").unwrap();
    writeln!(
        report,
        "Unparsable output: {}/{total}",
        unparsable_outputs.len()
    )
    .unwrap();
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

    if !unparsable_outputs.is_empty() {
        report.push_str("\n# Unparsable output\n\n");
        for (path, reason) in unparsable_outputs {
            writeln!(report, "- {path}: {reason}").unwrap();
        }
    }

    if !known_divergences.is_empty() {
        report.push_str("\n# Known divergences\n");
        for (reason, cases) in known_divergences {
            writeln!(report, "\n## {reason}\n").unwrap();
            for case in cases {
                let options = divergences::option_label(&case.options);
                writeln!(report, "- {} ({options})", case.path).unwrap();
            }
        }
    }

    let mut failures = BTreeMap::<&str, FileFailures>::new();
    for result in results {
        if !is_comparison_result(result) {
            continue;
        }
        if matches!(
            result.status,
            CaseStatus::ParseError(_) | CaseStatus::UnparsableOutput(_)
        ) {
            continue;
        }
        if (matches!(result.status, CaseStatus::Matched) && result.is_valid_output())
            || KNOWN_DIVERGENCES.reason_for(result).is_some()
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
            | CaseStatus::UnparsableOutput(_)
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
                if let Some(reason) = KNOWN_DIVERGENCES.reason_for(result) {
                    println!("Known divergence: {reason}");
                }
                let (expected, actual) = result.debug_diff.as_ref().unwrap();
                print_text_diff(expected, actual);
            }
            CaseStatus::FormatError(error) => println!("Format error: {error}"),
            CaseStatus::ParseError(error) => println!("Parse error: {error}"),
            CaseStatus::ExpectedParserError(error) => {
                println!("Expected parser error: {error}")
            }
            CaseStatus::UnparsableOutput(reason) => {
                println!("Unparsable output: {reason}");
                if let Some((expected, actual)) = &result.debug_diff {
                    print_text_diff(expected, actual);
                }
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
                let (text, ast) = result.ast_debug_diff.as_ref().unwrap();
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

fn has_parser_error_snapshot(
    snapshot: &str,
    file_name: &str,
    parser: &str,
    options: &OptionSet,
) -> bool {
    let prefix = format!("exports[`{file_name}");
    let suffix = format!(" format[{parser}] 1`] = `");
    snapshot
        .lines()
        .filter_map(|line| line.strip_prefix(&prefix)?.strip_suffix(&suffix))
        .any(|title_options| {
            if options.is_empty() {
                return title_options.is_empty();
            }
            title_options
                .strip_prefix(" - ")
                .and_then(|json| serde_json::from_str::<OptionSet>(json).ok())
                .is_some_and(|snapshot_options| snapshot_options == *options)
        })
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
        ast: unsafe { parsed.ast.ast() },
        parsers: Vec::new(),
        calls: Vec::new(),
        errors: Vec::new(),
    };
    unsafe { parsed.ast.ast() }
        .root_program()
        .visit_with(&mut parser);
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
        if let Some(object) = arguments
            .get_node(self.ast, 2)
            .and_then(|argument| argument.as_expr(self.ast))
            .and_then(|expression| expression.as_object_expression(self.ast))
            && let Err(error) =
                collect_options(self.ast, object, &mut options, &mut snapshot_options)
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
