//! Restore TypeScript virtual files and options from normalized fixture JSON.

use std::{
    collections::HashMap,
    fs::read,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use swc_next_ecma_ast::Lang;

use super::{fixture_json, fixtures, json_paths, read_json};

mod constants;
pub mod parser;
pub mod transform;

#[derive(Debug)]
pub(crate) struct TestUnit {
    pub(crate) name: String,
    pub(crate) content: String,
}

pub(crate) struct ParsedTestContent {
    pub(crate) test_units: Vec<TestUnit>,
    current_directory: String,
    global_options: HashMap<String, String>,
}

impl ParsedTestContent {
    pub(crate) fn global_option(&self, name: &str) -> Option<&str> {
        self.global_options.get(name).map(String::as_str)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Input {
    path: PathBuf,
    files: Vec<File>,
    tsconfig: Option<File>,
    current_directory: String,
    options: HashMap<String, String>,
}

#[derive(Deserialize)]
struct File {
    name: String,
    lines: Vec<usize>,
}

fn read_test_content(path: &Path) -> Result<ParsedTestContent, String> {
    let root = fixtures().join("typescript/tsc/testdata/tests/cases");
    let relative = path.strip_prefix(root).unwrap();
    let json = fixture_json()
        .join("typescript")
        .join(relative.with_added_extension("json"));
    let input: Input = read_json(&json);
    let source =
        read_typescript_file(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let source = source.replace("\r\n", "\n");
    let lines: Vec<_> = source.split('\n').collect();
    let mut test_units = Vec::new();
    let mut files = input.files.into_iter().chain(input.tsconfig).peekable();
    while let Some(file) = files.next() {
        // A later marker replaces an empty unit with the same name.
        if file.lines.is_empty() && files.peek().is_some_and(|next| next.name == file.name) {
            continue;
        }
        let content: Vec<_> = file.lines.iter().map(|&line| lines[line]).collect();
        test_units.push(TestUnit {
            name: file.name,
            content: content.join("\n"),
        });
    }
    Ok(ParsedTestContent {
        test_units,
        current_directory: input.current_directory,
        global_options: input.options,
    })
}

fn typescript_case_paths(test_dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for suite in ["compiler", "conformance"] {
        for path in json_paths(&format!("typescript/{suite}")) {
            let input: Input = read_json(&path);
            paths.push(test_dir.join(input.path));
        }
    }
    paths.sort_unstable();
    paths
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
