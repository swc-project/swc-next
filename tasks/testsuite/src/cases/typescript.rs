//! Based on TypeScript Go's test-case parser:
//! <https://github.com/microsoft/typescript-go/blob/main/internal/testrunner/test_case_parser.go>

use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use jwalk::WalkDir;
use rayon::prelude::*;
use regex::Regex;
use serde_json::Value;
use swc_next_ecma_ast::{Lang, SourceType};

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
    asserts_semantic_success: bool,
}

impl Case for TypeScriptCase {
    const NAME: &'static str = "typescript";

    fn read() -> Vec<Self> {
        let test_dir = fixtures().join("typescript/tests/cases");
        let error_codes = read_error_codes();

        WalkDir::new(test_dir)
            .into_iter()
            .filter_map(|file| {
                let file = file.ok()?;
                if !file.file_type().is_file() {
                    return None;
                }

                let path = file.path();
                let path_str = path.to_string_lossy();
                let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                (matches!(
                    ext,
                    "ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs"
                ) && !path_str.ends_with(".d.ts")
                    && !path_str.contains("_FIXTURE")
                    && !path_str.contains("transpile")
                    && !IGNORED_TESTS
                        .iter()
                        .any(|ignored| path_str.replace('\\', "/").contains(ignored)))
                .then_some(path)
            })
            .collect::<Vec<_>>()
            .into_par_iter()
            .flat_map(|path| Self::read_file(path, &error_codes))
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

    fn asserts_semantic_success(&self) -> bool {
        self.asserts_semantic_success
    }
}

impl TypeScriptCase {
    fn read_file(path: PathBuf, error_codes: &HashMap<String, HashSet<String>>) -> Vec<Self> {
        let Ok(code) = read_to_string(&path) else {
            return Vec::new();
        };
        let path_str = path.to_string_lossy();
        let mut parsed = parse_test_files_and_symlinks(&code, &path_str);
        merge_tsconfig_options(&path, &mut parsed.global_options);
        let expects_error = should_fail(&path, &parsed.global_options, error_codes);
        let check_js = boolean_option(&parsed.global_options, "checkjs");

        parsed
            .test_units
            .into_iter()
            .filter_map(|unit| {
                let lang = lang_from_path(&unit.name)?;
                let asserts_semantic_success =
                    asserts_semantic_success(lang, &unit.content, check_js);
                Some(Self {
                    path: path.clone(),
                    source_type: source_type(&unit.name, &unit.content),
                    lang,
                    name: unit.name,
                    code: unit.content,
                    expects_error,
                    asserts_semantic_success,
                })
            })
            .collect()
    }
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

fn read_error_codes() -> HashMap<String, HashSet<String>> {
    let baselines = fixtures().join("typescript/tests/baselines/reference");

    WalkDir::new(baselines)
        .into_iter()
        .filter_map(|file| {
            let file = file.ok()?;
            if !file.file_type().is_file() {
                return None;
            }

            let file_name = file.file_name().to_string_lossy();
            let baseline_name = file_name.strip_suffix(".errors.txt")?;
            // Option variants have names such as `foo(target=es5).errors.txt`.
            // Variants are flattened in this harness, so merge their diagnostics.
            let test_name = baseline_name
                .split_once('(')
                .map_or(baseline_name, |(name, _)| name)
                .to_string();
            Some((test_name, file.path()))
        })
        .collect::<Vec<_>>()
        .into_par_iter()
        .filter_map(|(test_name, path)| {
            let baseline = read_to_string(path).ok()?;
            let codes = TS_ERROR_CODE_REGEX
                .captures_iter(&baseline)
                .filter_map(|captures| captures.name("code").map(|code| code.as_str().to_string()))
                .collect::<HashSet<_>>();
            Some((test_name, codes))
        })
        .fold(
            HashMap::<String, HashSet<String>>::new,
            |mut error_codes, (test_name, new_codes)| {
                let codes = error_codes.entry(test_name).or_default();
                codes.extend(new_codes);
                error_codes
            },
        )
        .reduce(HashMap::new, |mut error_codes, new_error_codes| {
            for (test_name, new_codes) in new_error_codes {
                error_codes.entry(test_name).or_default().extend(new_codes);
            }
            error_codes
        })
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
                continue;
            }
            if !matches!(meta_name.as_str(), "ts-check" | "ts-nocheck") {
                global_options.insert(meta_name, meta_value);
                continue;
            }
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
        SourceType::CommonJs
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
