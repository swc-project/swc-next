///! https://github.com/microsoft/typescript-go/blob/main/internal/testrunner/test_case_parser.go
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use serde_json::Value;

use jwalk::WalkDir;
use regex::Regex;
use swc_experimental_ecma_parser::{Syntax, TsSyntax};

use crate::cases::{Case, IsModule, fixtures};

/// Regex for parsing options in the format "// @Alpha: Value of any sort"
static OPTION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^//\s*@(\w+)\s*:\s*([^\r\n]*)").unwrap());

/// Regex for parsing @link option: "// @link: source -> target"
static LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^//\s*@link\s*:\s*([^\r\n]*)\s*->\s*([^\r\n]*)").unwrap());

/// A single test unit parsed from a multi-file test
#[derive(Debug)]
struct TestUnit {
    name: String,
    content: String,
}

/// Result of parsing a test file
#[derive(Debug)]
struct ParsedTestContent {
    test_units: Vec<TestUnit>,
    #[allow(dead_code)]
    symlinks: HashMap<String, String>,
    #[allow(dead_code)]
    current_directory: Option<String>,
    global_options: HashMap<String, String>,
}

/// A single TypeScript test case
pub struct TypeScriptCase {
    /// Path to the original test file
    path: PathBuf,
    /// Name of this unit (from @Filename or basename)
    #[allow(dead_code)]
    name: String,
    /// Source code content
    code: String,
    /// Global options parsed from metadata
    global_options: HashMap<String, String>,
}

impl TypeScriptCase {
    pub fn read() -> Vec<Self> {
        let mut cases = Vec::new();
        let test_dir = fixtures().join("typescript").join("tests").join("cases");

        for file in WalkDir::new(&test_dir) {
            let Ok(file) = file else { continue };
            if !file.file_type().is_file() {
                continue;
            }

            let path = file.path();
            let path_str = path.to_string_lossy();

            // Only process TypeScript/JavaScript files
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(
                ext,
                "ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs"
            ) || path_str.ends_with(".d.ts")
            {
                continue;
            }

            // Skip fixture files
            if path_str.contains("_FIXTURE") {
                continue;
            }

            // Skip transpile cases
            if path_str.contains("transpile") {
                continue;
            }

            let Ok(code) = read_to_string(&path) else {
                continue;
            };

            let mut parsed = parse_test_files_and_symlinks(&code, &path_str);

            // Try to read tsconfig.json from the same directory
            if let Some(parent_dir) = path.parent() {
                let tsconfig_path = parent_dir.join("tsconfig.json");
                if tsconfig_path.exists() {
                    if let Ok(tsconfig_content) = read_to_string(&tsconfig_path) {
                        if let Ok(tsconfig) = serde_json::from_str::<Value>(&tsconfig_content) {
                            // Extract compilerOptions and merge into global_options
                            if let Some(compiler_options) = tsconfig.get("compilerOptions") {
                                if let Some(options_obj) = compiler_options.as_object() {
                                    for (key, value) in options_obj {
                                        let key_lower = key.to_lowercase();
                                        let value_str = match value {
                                            Value::Bool(b) => b.to_string(),
                                            Value::String(s) => s.clone(),
                                            Value::Number(n) => n.to_string(),
                                            _ => continue,
                                        };
                                        // Only add if not already present (file metadata takes precedence)
                                        parsed.global_options.entry(key_lower).or_insert(value_str);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Create a case for each test unit
            for unit in parsed.test_units {
                cases.push(TypeScriptCase {
                    path: path.clone(),
                    name: unit.name,
                    code: unit.content,
                    global_options: parsed.global_options.clone(),
                });
            }
        }

        cases
    }
}

/// Parse symlink directive from a line.
/// Returns true if a symlink was parsed.
fn parse_symlink_from_test(line: &str, symlinks: &mut HashMap<String, String>) -> bool {
    if let Some(caps) = LINK_REGEX.captures(line) {
        let source = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
        let target = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
        symlinks.insert(target.to_string(), source.to_string());
        return true;
    }
    false
}

/// Parse test files from a multi-file test content.
/// This is a port of Go's ParseTestFilesAndSymlinks function.
fn parse_test_files_and_symlinks(code: &str, file_name: &str) -> ParsedTestContent {
    let mut test_units: Vec<TestUnit> = Vec::new();
    let mut symlinks: HashMap<String, String> = HashMap::new();
    let mut current_directory: Option<String> = None;
    let mut global_options: HashMap<String, String> = HashMap::new();

    let lines: Vec<&str> = code.lines().collect();

    // State for parsing
    let mut current_file_content = String::new();
    let mut current_file_name: Option<String> = None;

    // Handle `// @xxxx``
    for line in lines {
        // Check for symlink directive
        if parse_symlink_from_test(line, &mut symlinks) {
            continue;
        }

        // Check for option directive using regex
        if let Some(caps) = OPTION_REGEX.captures(line) {
            let meta_name = caps
                .get(1)
                .map(|m| m.as_str().to_lowercase())
                .unwrap_or_default();
            let meta_value = caps
                .get(2)
                .map(|m| m.as_str().trim())
                .unwrap_or("")
                .to_string();

            // Handle currentdirectory option
            if meta_name == "currentdirectory" {
                current_directory = Some(meta_value.clone());
            }

            // Handle filename directive - start a new file
            if meta_name == "filename" {
                // Save previous file if exists
                if let Some(ref prev_name) = current_file_name {
                    test_units.push(TestUnit {
                        name: prev_name.clone(),
                        content: current_file_content.clone(),
                    });
                }

                // Start new file
                current_file_content = String::new();
                current_file_name = Some(meta_value);
            } else {
                // Global option (not filename)
                global_options.insert(meta_name, meta_value);
            }
            continue;
        }

        // Regular content line
        if !current_file_content.is_empty() {
            current_file_content.push('\n');
        }
        current_file_content.push_str(line);
    }

    // Normalize file name for single-file case
    if test_units.is_empty() && current_file_name.is_none() {
        current_file_name = Some(
            Path::new(file_name)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("test.ts")
                .to_string(),
        );
    }

    // Push final file
    if let Some(name) = current_file_name {
        test_units.push(TestUnit {
            name,
            content: current_file_content,
        });
    }

    ParsedTestContent {
        test_units,
        symlinks,
        current_directory,
        global_options,
    }
}

impl Case for TypeScriptCase {
    fn path(&self) -> &Path {
        &self.path
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn should_fail(&self) -> bool {
        // Check for @negative option or explicit error expectations
        if self.global_options.contains_key("negative") {
            return true;
        }

        // Check if path contains "negative" folder
        let path_str = self.path.to_string_lossy();
        if path_str.contains("/negative/") || path_str.contains("\\negative\\") {
            return true;
        }

        false
    }

    fn is_module(&self) -> IsModule {
        match self.ext().as_str() {
            "mts" | "mjs" => IsModule::Module,
            "cts" | "cjs" => IsModule::Script,
            _ => IsModule::Unknown,
        }
    }

    fn syntax(&self) -> Syntax {
        let ext = self.ext();
        let mut ts = TsSyntax::default();

        if ext == "tsx" {
            ts.tsx = true;
        }

        // Check for decorator support
        if self.global_options.contains_key("experimentaldecorators")
            || self.code.contains("@decorator")
        {
            ts.decorators = true;
        }

        Syntax::Typescript(ts)
    }

    fn should_ignore(&self) -> bool {
        IGNORED_TESTS.iter().any(|ignore| {
            self.path
                .to_string_lossy()
                .replace('\\', "/")
                .contains(ignore)
        })
    }
}

const IGNORED_TESTS: &[&str] = &[
    // Should be fixed: stackoverflow
    "fixtures/typescript/tests/cases/compiler/binderBinaryExpressionStress.ts",
    "fixtures/typescript/tests/cases/compiler/binderBinaryExpressionStressJs.ts",
];
