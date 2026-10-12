//! Shared TypeScript fixture decoding, virtual files, and compiler-option metadata.

use std::{
    collections::HashMap,
    fs::{read, read_to_string},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use jwalk::WalkDir;
use regex::Regex;
use serde_json::Value;
use swc_next_ecma_ast::Lang;

mod constants;
pub mod parser;
pub mod transform;

/// Regex for parsing options in the format `// @Alpha: Value of any sort`.
static OPTION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^//\s*@([[:word:]]+)\s*:\s*([^\r\n]*)").unwrap());

/// Regex for parsing `@link` options in the format `// @link: source -> target`.
static LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^//\s*@link\s*:\s*([^\r\n]*)\s*->\s*([^\r\n]*)").unwrap());

#[derive(Debug)]
pub(crate) struct TestUnit {
    pub(crate) name: String,
    pub(crate) content: String,
}

#[derive(Debug)]
pub(crate) struct ParsedTestContent {
    pub(crate) test_units: Vec<TestUnit>,
    global_options: HashMap<String, String>,
}

impl ParsedTestContent {
    pub(crate) fn global_option(&self, name: &str) -> Option<&str> {
        self.global_options.get(name).map(String::as_str)
    }
}

fn read_test_content(path: &Path) -> Result<ParsedTestContent, String> {
    let code = read_typescript_file(path).map_err(|error| {
        format!(
            "failed to read TypeScript input `{}`: {error}",
            path.display()
        )
    })?;
    let mut parsed = parse_test_files_and_symlinks(&code, &path.to_string_lossy());
    merge_tsconfig_options(path, &mut parsed);
    Ok(parsed)
}

fn typescript_case_paths(test_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    for suite in ["compiler", "conformance"] {
        let suite_root = test_dir.join(suite);
        for entry in WalkDir::new(&suite_root) {
            let entry = entry.map_err(|error| {
                format!(
                    "failed to enumerate TypeScript inputs in `{}`: {error}",
                    suite_root.display()
                )
            })?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if lang_from_path(&path.to_string_lossy()).is_some_and(Lang::is_typescript) {
                paths.push(path);
            }
        }
    }
    paths.sort_unstable();
    Ok(paths)
}

pub(crate) fn read_typescript_file(path: &Path) -> std::io::Result<String> {
    let bytes = read(path)?;
    if let Some(bytes) = bytes.strip_prefix(&[0xfe, 0xff]) {
        let (pairs, _) = bytes.as_chunks::<2>();
        let code_units = pairs
            .iter()
            .copied()
            .map(u16::from_be_bytes)
            .collect::<Vec<_>>();
        return Ok(String::from_utf16_lossy(&code_units));
    }
    if let Some(bytes) = bytes.strip_prefix(&[0xff, 0xfe]) {
        let (pairs, _) = bytes.as_chunks::<2>();
        let code_units = pairs
            .iter()
            .copied()
            .map(u16::from_le_bytes)
            .collect::<Vec<_>>();
        return Ok(String::from_utf16_lossy(&code_units));
    }
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

pub(crate) fn merge_tsconfig_options(path: &Path, parsed: &mut ParsedTestContent) {
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
        parsed
            .global_options
            .entry(key.to_lowercase())
            .or_insert(value);
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

pub(crate) fn parse_test_files_and_symlinks(code: &str, file_name: &str) -> ParsedTestContent {
    let mut test_units = Vec::new();
    let mut symlinks = HashMap::new();
    let mut global_options = HashMap::new();
    let mut current_file_content = None::<String>;
    let mut current_file_name = None;

    for line in split_content_by_newlines(code) {
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
                    // TypeScript drops a virtual-file marker that is replaced
                    // before any source line belongs to it.
                    if let Some(content) = current_file_content.take() {
                        test_units.push(TestUnit {
                            name: previous_name,
                            content,
                        });
                    }
                } else {
                    // TypeScript permits only trivia before the first virtual file.
                    // Do not attach that trivia to the first SourceFile.
                    current_file_content = None;
                }
                continue;
            }
            if !matches!(meta_name.as_str(), "ts-check" | "ts-nocheck") {
                global_options.insert(meta_name, meta_value);
                continue;
            }
        }

        let content = current_file_content.get_or_insert_default();
        if !content.is_empty() {
            content.push('\n');
        }
        content.push_str(line);
    }

    if let Some(name) = current_file_name {
        // The final marker always denotes a virtual file, including an empty
        // file at EOF. Only an intermediate marker superseded before any
        // source line is omitted above.
        test_units.push(TestUnit {
            name,
            content: current_file_content.unwrap_or_default(),
        });
    } else {
        let name = Path::new(file_name)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("test.ts")
            .to_string();
        test_units.push(TestUnit {
            name,
            content: current_file_content.unwrap_or_default(),
        });
    }

    ParsedTestContent {
        test_units,
        global_options,
    }
}

/// Match TypeScript's `/\r?\n/` split, including the final empty segment that
/// preserves a newline at EOF. A bare carriage return is source text.
fn split_content_by_newlines(mut code: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    while let Some(newline) = code.find('\n') {
        let line = &code[..newline];
        lines.push(line.strip_suffix('\r').unwrap_or(line));
        code = &code[newline + 1..];
    }
    lines.push(code);
    lines
}

pub(crate) fn lang_from_path(path: &str) -> Option<Lang> {
    if is_declaration_file_name(path) {
        Some(Lang::Dts)
    } else if path.ends_with(".tsx") {
        Some(Lang::Tsx)
    } else if path.ends_with(".ts") || path.ends_with(".mts") || path.ends_with(".cts") {
        Some(Lang::Ts)
    } else if path.ends_with(".js")
        || path.ends_with(".jsx")
        || path.ends_with(".mjs")
        || path.ends_with(".cjs")
    {
        Some(Lang::Jsx)
    } else {
        None
    }
}

fn is_declaration_file_name(path: &str) -> bool {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    name.ends_with(".d.ts")
        || name.ends_with(".d.mts")
        || name.ends_with(".d.cts")
        || (name.ends_with(".ts") && name.contains(".d."))
}
