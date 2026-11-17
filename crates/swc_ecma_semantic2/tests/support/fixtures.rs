use std::{collections::BTreeMap, fmt::Write, fs, path::Path};

use serde::{Deserialize, Serialize};
use swc_next_ecma_ast::{Lang, SourceType};

use super::options::TestOptions;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct FixtureOptions {
    pub(crate) names: Vec<String>,
    /// Additional public names to query on a named container, including absent names.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) exports: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_type: Option<String>,
    #[serde(default = "preserve_parens", skip_serializing_if = "is_true")]
    preserve_parens: bool,
}

fn preserve_parens() -> bool {
    true
}
fn is_true(value: &bool) -> bool {
    *value
}

impl FixtureOptions {
    pub(crate) fn options(&self, path: &Path) -> TestOptions {
        let source_type = match self.source_type.as_deref().unwrap_or("module") {
            "module" => SourceType::Module,
            "script" => SourceType::Script,
            "commonjs" => SourceType::CommonJs,
            "unambiguous" => SourceType::Unambiguous,
            value => panic!("unknown source type: {value}"),
        };
        let mut options = TestOptions::new(Lang::from_path(path.to_str().unwrap()), source_type);
        options.parser.preserve_parens = self.preserve_parens;
        options
    }
}

pub(crate) fn option_sets(mut directory: &Path, root: &Path) -> Vec<FixtureOptions> {
    loop {
        let options = directory.join("options.json");
        if options.exists() || directory == root {
            let source = fs::read_to_string(options).unwrap();
            return serde_json::from_str(&source).unwrap();
        }
        directory = directory.parent().unwrap();
    }
}

pub(crate) fn run<T: Serialize>(
    suite: &str,
    inspect: impl Fn(&str, TestOptions, &FixtureOptions) -> T,
) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(suite);
    insta::glob!(&root, "**/input.{js,jsx,ts,tsx,d.ts}", |path| {
        let directory = path.parent().unwrap();
        let input = fs::read_to_string(path).unwrap();
        let mut snapshot = format!("---------- Input ----------\n{input}");
        for spec in option_sets(directory, &root) {
            let options = spec.options(path);
            let output = inspect(&input, options, &spec);
            writeln!(
                snapshot,
                "---------- Options ----------\n{}",
                serde_json::to_string(&spec).unwrap()
            )
            .unwrap();
            writeln!(
                snapshot,
                "---------- Output ----------\n{}",
                serde_json::to_string_pretty(&output).unwrap()
            )
            .unwrap();
        }
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(directory);
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.remove_snapshot_suffix();
        settings.remove_input_file();
        settings.bind(|| insta::assert_snapshot!("output", snapshot));
    });
}
