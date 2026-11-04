use std::{
    fs, io,
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use serde_json::{Map, Value};
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_minifier::{CompressOptions, MinifierOptions};

use super::{Case, MinifierCase, fixtures};

/// Fixture root of SWC's minifier tests, including its terser port.
fn minifier_fixtures() -> PathBuf {
    fixtures().join("swc/crates/swc_ecma_minifier/tests")
}

/// An `input.js` fixture of the SWC minifier tests.
#[derive(Debug)]
struct MinifierFixture {
    path: PathBuf,
    name: String,
    input: String,
    expected: Option<String>,
    options: MinifierOptions,
    source_type: SourceType,
}

impl MinifierFixture {
    /// Reads the fixtures in `directories`, which compare output with the
    /// `reference` file.
    fn read_all(directories: &[&str], reference: &str) -> io::Result<Vec<Self>> {
        let root = minifier_fixtures();
        let mut cases = Vec::new();
        for directory in directories {
            let suite_root = root.join(directory);
            // The walk skips dot-prefixed directories, which SWC uses to
            // disable fixtures.
            for entry in WalkDir::new(&suite_root) {
                let entry = entry.map_err(io::Error::other)?;
                if !entry.file_type().is_file() || entry.file_name() != "input.js" {
                    continue;
                }

                let path = entry.path();
                let case_dir = path.parent().unwrap();
                let config = read_config(&suite_root, case_dir)?;
                // Mangled names are specific to each minifier, so outputs of
                // mangled fixtures cannot be compared.
                let expected = if case_dir.join("mangle.json").try_exists()? {
                    None
                } else {
                    read_optional(&case_dir.join(reference))?
                };
                cases.push(Self {
                    name: case_dir
                        .strip_prefix(&root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                    input: fs::read_to_string(&path)?,
                    expected,
                    options: minifier_options(&config),
                    source_type: if config.get("module").and_then(Value::as_bool) == Some(true) {
                        SourceType::Module
                    } else {
                        // SWC parses these fixtures as a module only when they
                        // contain module syntax.
                        SourceType::Unambiguous
                    },
                    path,
                });
            }
        }

        cases.sort_unstable_by(|left, right| left.path.cmp(&right.path));
        Ok(cases)
    }
}

/// Reads the nearest `config.json` within the suite, as SWC does.
fn read_config(suite_root: &Path, case_dir: &Path) -> io::Result<Map<String, Value>> {
    for dir in case_dir.ancestors() {
        if let Some(config) = read_optional(&dir.join("config.json"))? {
            return serde_json::from_str(&config).map_err(io::Error::other);
        }
        if dir == suite_root {
            break;
        }
    }
    Ok(Map::new())
}

fn read_optional(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// Maps terser compress options to this minifier's options.
///
/// As in the terser and SWC test runners, an option that the config omits
/// takes the value of `defaults`, which is `false` when omitted. Terser
/// shortens unshadowed global constants unless `ie8` is enabled.
fn minifier_options(config: &Map<String, Value>) -> MinifierOptions {
    let defaults = config.get("defaults").and_then(Value::as_bool) == Some(true);
    let option = |name| match config.get(name) {
        Some(Value::Bool(value)) => *value,
        // Non-boolean values select a variant of an enabled option.
        Some(_) => true,
        None => defaults,
    };
    MinifierOptions {
        compress: Some(CompressOptions {
            booleans: option("booleans"),
            dead_code: option("dead_code"),
            evaluate: option("evaluate"),
            global_defs: config.get("ie8").and_then(Value::as_bool) != Some(true),
            typeofs: option("typeofs"),
            unused: option("unused"),
        }),
        mangle: false,
    }
}

/// Terser compress tests, as ported to SWC's minifier test suite. The
/// reference output is terser's.
pub struct TerserMinifierCase(MinifierFixture);

/// SWC minifier fixtures. The reference output is SWC's.
pub struct SwcMinifierCase(MinifierFixture);

macro_rules! impl_minifier_case {
    ($case:ident, $name:literal, $directories:expr, $reference:literal) => {
        impl Case for $case {
            const NAME: &'static str = $name;

            fn read() -> Vec<Self> {
                MinifierFixture::read_all(&$directories, $reference)
                    .expect("failed to read minifier fixtures; run ./scripts/clone_fixtures.sh")
                    .into_iter()
                    .map(Self)
                    .collect()
            }

            fn path(&self) -> &Path {
                &self.0.path
            }

            fn name(&self) -> &str {
                &self.0.name
            }

            fn code(&self) -> &str {
                &self.0.input
            }

            fn source_type(&self) -> SourceType {
                self.0.source_type
            }

            fn lang(&self) -> Lang {
                Lang::Js
            }

            fn expects_error(&self) -> bool {
                false
            }
        }

        impl MinifierCase for $case {
            fn minifier_options(&self) -> MinifierOptions {
                self.0.options.clone()
            }

            fn expected_output(&self) -> Option<&str> {
                self.0.expected.as_deref()
            }
        }
    };
}

impl_minifier_case!(
    TerserMinifierCase,
    "terser",
    ["terser/compress"],
    "output.terser.js"
);
impl_minifier_case!(
    SwcMinifierCase,
    "swc",
    ["fixture", "pass-1", "pass-default"],
    "output.js"
);
