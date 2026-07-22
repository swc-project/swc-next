//! Based on TypeScript Go's test-case parser:
//! <https://github.com/microsoft/typescript-go/blob/main/internal/testrunner/test_case_parser.go>

use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use jwalk::WalkDir;
use regex::Regex;
use serde_json::Value;
use swc_experimental_ecma_ast::{Lang, SourceType};

use crate::cases::{Case, fixtures, typescript::constants::NOT_SUPPORTED_ERROR_CODES};

mod constants;

/// Regex for parsing options in the format `// @Alpha: Value of any sort`.
static OPTION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^//\s*@([[:word:]]+)\s*:\s*([^\r\n]*)").unwrap());

/// Regex for parsing `@link` options in the format `// @link: source -> target`.
static LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^//\s*@link\s*:\s*([^\r\n]*)\s*->\s*([^\r\n]*)").unwrap());

/// Regex for a TypeScript diagnostic such as `error TS1234: ...`.
static TS_ERROR_CODE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"error[[:space:]]+TS(?<code>\d{4,5}):[[:space:]]+").unwrap());

#[derive(Debug)]
struct TestUnit {
    name: String,
    content: String,
}

#[derive(Debug)]
struct ParsedTestContent {
    test_units: Vec<TestUnit>,
    global_options: HashMap<String, String>,
}

pub struct TypeScriptCase {
    path: PathBuf,
    name: String,
    code: String,
    source_type: SourceType,
    lang: Lang,
    expects_error: bool,
}

impl Case for TypeScriptCase {
    const NAME: &'static str = "typescript";

    fn read() -> Vec<Self> {
        let mut cases = Vec::new();
        let test_dir = fixtures().join("typescript/tests/cases");
        let error_codes = read_error_codes();

        for file in WalkDir::new(test_dir) {
            let Ok(file) = file else { continue };
            if !file.file_type().is_file() {
                continue;
            }

            let path = file.path();
            let path_str = path.to_string_lossy();
            let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if !matches!(
                ext,
                "ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs"
            ) || path_str.ends_with(".d.ts")
                || path_str.contains("_FIXTURE")
                || path_str.contains("transpile")
                || IGNORED_TESTS
                    .iter()
                    .any(|ignored| path_str.replace('\\', "/").contains(ignored))
            {
                continue;
            }

            let Ok(code) = read_to_string(&path) else {
                continue;
            };
            let mut parsed = parse_test_files_and_symlinks(&code, &path_str);
            merge_tsconfig_options(&path, &mut parsed.global_options);
            let expects_error = should_fail(&path, &parsed.global_options, &error_codes);

            for unit in parsed.test_units {
                let Some(lang) = lang_from_path(&unit.name) else {
                    continue;
                };
                cases.push(Self {
                    path: path.clone(),
                    source_type: source_type(&unit.name, &unit.content),
                    lang,
                    name: unit.name,
                    code: unit.content,
                    expects_error,
                });
            }
        }

        cases
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
}

fn read_error_codes() -> HashMap<String, HashSet<String>> {
    let mut error_codes = HashMap::<String, HashSet<String>>::new();
    let baselines = fixtures().join("typescript/tests/baselines/reference");

    for file in WalkDir::new(baselines) {
        let Ok(file) = file else { continue };
        if !file.file_type().is_file() {
            continue;
        }

        let file_name = file.file_name().to_string_lossy();
        let Some(baseline_name) = file_name.strip_suffix(".errors.txt") else {
            continue;
        };
        // Option variants have names such as `foo(target=es5).errors.txt`.
        // Variants are flattened in this harness, so merge their diagnostics.
        let test_name = baseline_name
            .split_once('(')
            .map_or(baseline_name, |(name, _)| name);
        let Ok(baseline) = read_to_string(file.path()) else {
            continue;
        };
        let codes = error_codes.entry(test_name.to_string()).or_default();
        for captures in TS_ERROR_CODE_REGEX.captures_iter(&baseline) {
            if let Some(code) = captures.name("code") {
                codes.insert(code.as_str().to_string());
            }
        }
    }

    error_codes
}

fn should_fail(
    path: &Path,
    global_options: &HashMap<String, String>,
    error_codes: &HashMap<String, HashSet<String>>,
) -> bool {
    if global_options.contains_key("negative") {
        return true;
    }

    let normalized_path = path.to_string_lossy().replace('\\', "/");
    if normalized_path.contains("/negative/") {
        return true;
    }

    let Some(test_name) = path.file_stem().and_then(|name| name.to_str()) else {
        return false;
    };
    error_codes.get(test_name).is_some_and(|codes| {
        codes
            .iter()
            .any(|code| !NOT_SUPPORTED_ERROR_CODES.contains(&code.as_str()))
    })
}

fn merge_tsconfig_options(path: &Path, global_options: &mut HashMap<String, String>) {
    let Some(parent_dir) = path.parent() else {
        return;
    };
    let Ok(tsconfig_content) = read_to_string(parent_dir.join("tsconfig.json")) else {
        return;
    };
    let Ok(tsconfig) = serde_json::from_str::<Value>(&tsconfig_content) else {
        return;
    };
    let Some(options) = tsconfig.get("compilerOptions").and_then(Value::as_object) else {
        return;
    };

    for (key, value) in options {
        let value = match value {
            Value::Bool(value) => value.to_string(),
            Value::String(value) => value.clone(),
            Value::Number(value) => value.to_string(),
            _ => continue,
        };
        global_options.entry(key.to_lowercase()).or_insert(value);
    }
}

fn parse_symlink_from_test(line: &str, symlinks: &mut HashMap<String, String>) -> bool {
    let Some(captures) = LINK_REGEX.captures(line) else {
        return false;
    };
    let source = captures.get(1).map_or("", |value| value.as_str().trim());
    let target = captures.get(2).map_or("", |value| value.as_str().trim());
    symlinks.insert(target.to_string(), source.to_string());
    true
}

fn parse_test_files_and_symlinks(code: &str, file_name: &str) -> ParsedTestContent {
    let mut test_units = Vec::new();
    let mut symlinks = HashMap::new();
    let mut global_options = HashMap::new();
    let mut current_file_content = String::new();
    let mut current_file_name = None;

    for line in code.lines() {
        if parse_symlink_from_test(line, &mut symlinks) {
            continue;
        }

        if let Some(captures) = OPTION_REGEX.captures(line) {
            let meta_name = captures
                .get(1)
                .map_or_else(String::new, |value| value.as_str().to_lowercase());
            let meta_value = captures
                .get(2)
                .map_or("", |value| value.as_str().trim())
                .to_string();

            if meta_name == "filename" {
                if let Some(previous_name) = current_file_name.replace(meta_value) {
                    test_units.push(TestUnit {
                        name: previous_name,
                        content: std::mem::take(&mut current_file_content),
                    });
                }
            } else {
                global_options.insert(meta_name, meta_value);
            }
            continue;
        }

        if !current_file_content.is_empty() {
            current_file_content.push('\n');
        }
        current_file_content.push_str(line);
    }

    let name = current_file_name.unwrap_or_else(|| {
        Path::new(file_name)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("test.ts")
            .to_string()
    });
    test_units.push(TestUnit {
        name,
        content: current_file_content,
    });

    ParsedTestContent {
        test_units,
        global_options,
    }
}

fn lang_from_path(path: &str) -> Option<Lang> {
    if path.ends_with(".d.ts") || path.ends_with(".d.mts") || path.ends_with(".d.cts") {
        Some(Lang::Dts)
    } else if path.ends_with(".tsx") {
        Some(Lang::Tsx)
    } else if path.ends_with(".ts") || path.ends_with(".mts") || path.ends_with(".cts") {
        Some(Lang::Ts)
    } else if path.ends_with(".jsx") {
        Some(Lang::Jsx)
    } else if path.ends_with(".js") || path.ends_with(".mjs") || path.ends_with(".cjs") {
        Some(Lang::Js)
    } else {
        None
    }
}

fn source_type(path: &str, code: &str) -> SourceType {
    if path.ends_with(".mts") || path.ends_with(".mjs") {
        SourceType::Module
    } else if path.ends_with(".cts") || path.ends_with(".cjs") {
        SourceType::Script
    } else if code.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("import ")
            || line.starts_with("import{")
            || line.starts_with("export ")
            || line.starts_with("export{")
    }) {
        SourceType::Module
    } else {
        SourceType::Script
    }
}

const IGNORED_TESTS: &[&str] = &[
    "fixtures/typescript/tests/cases/compiler/binderBinaryExpressionStress.ts",
    "fixtures/typescript/tests/cases/compiler/binderBinaryExpressionStressJs.ts",
];
