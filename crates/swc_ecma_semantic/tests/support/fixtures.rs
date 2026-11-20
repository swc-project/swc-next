use std::{fmt::Write, fs, io::ErrorKind, path::Path};

use serde::{Deserialize, Serialize};
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_semantic::SyntaxChecks;

use super::{analysis::DiagnosticCounts, options::TestOptions};

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FixtureOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preserve_parens: Option<bool>,
}

impl FixtureOptions {
    fn options(&self, path: &Path) -> TestOptions {
        let lang = match self.lang.as_deref() {
            None => Lang::from_path(path.to_str().unwrap()),
            Some("js") => Lang::Js,
            Some("ts") => Lang::Ts,
            Some("jsx") => Lang::Jsx,
            Some("tsx") => Lang::Tsx,
            Some("dts") => Lang::Dts,
            Some(value) => panic!("unknown language: {value}"),
        };
        let source_type = match self.source_type.as_deref().unwrap_or("module") {
            "module" => SourceType::Module,
            "script" => SourceType::Script,
            "commonjs" => SourceType::CommonJs,
            "unambiguous" => SourceType::Unambiguous,
            value => panic!("unknown source type: {value}"),
        };
        let mut options = TestOptions::new(lang, source_type);
        options.parser.preserve_parens = self.preserve_parens.unwrap_or(true);
        options
    }
}

pub(crate) fn run(inspect: impl Fn(&str, TestOptions) -> (String, DiagnosticCounts)) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/semantic");
    insta::glob!(&root, "**/input.{js,jsx,ts,tsx,d.ts}", |path| {
        let directory = path.parent().unwrap();
        let input = fs::read_to_string(path).unwrap();
        let mut snapshot = format!("---------- Input ----------\n{input}");
        if !snapshot.ends_with('\n') {
            snapshot.push('\n');
        }
        let (specs, show_options): (Vec<FixtureOptions>, bool) =
            match fs::read_to_string(directory.join("options.json")) {
                Ok(source) => (serde_json::from_str(&source).unwrap(), true),
                Err(e) if e.kind() == ErrorKind::NotFound => {
                    (vec![FixtureOptions::default()], false)
                }
                Err(e) => panic!("{}: {e}", directory.display()),
            };
        assert!(!specs.is_empty());
        // Configurations with the same graph share one output block.
        let mut outputs = Vec::<(String, Vec<FixtureOptions>)>::new();
        for spec in specs {
            let options = spec.options(path);
            let (graph, diagnostics) = inspect(&input, options);
            let (unchecked, unchecked_diagnostics) =
                inspect(&input, options.syntax(SyntaxChecks::empty()));
            assert_eq!(
                unchecked,
                graph,
                "syntax checks changed semantic results for {}",
                path.display()
            );
            assert_eq!(unchecked_diagnostics.parser, diagnostics.parser);
            assert_eq!(unchecked_diagnostics.semantic, 0);
            let mut output = String::new();
            if diagnostics != DiagnosticCounts::default() {
                writeln!(
                    output,
                    "---------- Diagnostics ----------\n{}",
                    serde_json::to_string(&diagnostics).unwrap()
                )
                .unwrap();
            }
            writeln!(output, "---------- Output ----------\n{graph}").unwrap();
            if let Some((_, specs)) = outputs.iter_mut().find(|(previous, _)| previous == &output) {
                specs.push(spec);
            } else {
                outputs.push((output, vec![spec]));
            }
        }
        for (output, specs) in outputs {
            if show_options {
                let options = serde_json::to_string(&specs).unwrap();
                writeln!(snapshot, "---------- Options ----------\n{options}").unwrap();
            }
            snapshot.push_str(&output);
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
