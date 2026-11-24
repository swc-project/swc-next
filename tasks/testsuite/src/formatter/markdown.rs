//! Upstream Markdown suites include both physical files and generated inline examples.
//! Each Jest snapshot carries the options, input and expected output for a single case.

use std::{collections::BTreeMap, env, fmt::Write, fs};

use serde_json::Value;
use swc_next_formatter::markdown::{self, FormatOptions, ProseWrap, SyntaxOptions};
use swc_next_markdown_parser::parse;

mod prettier;

use super::{
    KNOWN_DIVERGENCES, OptionSet, PRETTIER_COMPATIBILITY_VERSION, PRETTIER_FIXTURE_REVISION,
    SNAPSHOT_DIR_NAME, SNAPSHOT_FILE_NAME, collect_test_dirs_recursive,
    options::{format_options, unsupported_option_names},
    print_text_diff, validate_fixture_version,
};
use crate::util::crate_root;
use prettier::Collector;

struct Snapshot<'a> {
    name: &'a str,
    options: OptionSet,
    input: &'a str,
    output: &'a str,
}

impl<'a> Snapshot<'a> {
    fn read(entry: &'a str) -> Option<Self> {
        let (name, body) = entry.split_once("`] = `\n")?;
        let name = name.strip_suffix(" format 1")?;
        let (header, body) = body.split_once(
            "\n=====================================input======================================\n",
        )?;
        let (input, body) = body.split_once(
            "\n=====================================output=====================================\n",
        )?;
        let (output, _) = body.rsplit_once(
            "\n================================================================================",
        )?;
        let mut options = OptionSet::new();
        for line in header.lines().skip(1) {
            let (key, value) = line.trim().split_once(": ")?;
            match key {
                "parsers" => continue,
                "printWidth" => {
                    let width = value.split_whitespace().next()?.parse::<u16>().ok()?;
                    options.insert(key.into(), width.into());
                }
                _ => {
                    options.insert(key.into(), serde_json::from_str(value).ok()?);
                }
            }
        }
        Some(Self {
            name,
            options,
            input,
            output,
        })
    }

    fn format_options(&self, source: &str) -> Result<FormatOptions, String> {
        let mut options = self.options.clone();
        let prose_wrap = match options.remove("proseWrap") {
            None => ProseWrap::Preserve,
            Some(value) if value == "preserve" => ProseWrap::Preserve,
            Some(value) if value == "always" => ProseWrap::Always,
            Some(value) if value == "never" => ProseWrap::Never,
            Some(value) => return Err(format!("unsupported Prettier option proseWrap={value}")),
        };
        let embed = match options.remove("embeddedLanguageFormatting") {
            None => true,
            Some(value) if value == "auto" => true,
            Some(value) if value == "off" => false,
            Some(value) => {
                return Err(format!(
                    "unsupported Prettier option embeddedLanguageFormatting={value}"
                ));
            }
        };
        let layout = format_options(&options, source).map_err(|error| {
            let reasons = unsupported_option_names(&options);
            if reasons.is_empty() {
                error
            } else {
                reasons.join(", ")
            }
        })?;
        let syntax = SyntaxOptions {
            prose_wrap,
            single_quote: options
                .get("singleQuote")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        };
        let mut document = FormatOptions {
            layout: layout.into(),
            syntax,
            ..Default::default()
        };
        document.embedded.enabled = embed;
        Ok(document)
    }
}

fn snapshot_key(name: &str) -> (&str, &str) {
    match name.split_once(" - {") {
        Some((path, _)) => (path, &name[path.len() + " - ".len()..]),
        None => (name, ""),
    }
}

// Jest escapes backticks, backslashes and template interpolation in the snapshot file.
fn unescape_template(raw: &str) -> String {
    let mut source = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\'
            && let Some(escaped) = chars.next_if(|c| matches!(c, '\\' | '`' | '$'))
        {
            source.push(escaped);
        } else {
            source.push(ch);
        }
    }
    source
}

#[derive(Default)]
struct Counts {
    tested: usize,
    passed: usize,
    matched: usize,
    unstable: usize,
    unsupported: usize,
    misaligned: usize,
}

/// Format Markdown recursively with SWC and its embedded languages with Prettier.
pub fn run_markdown_conformance() -> Result<Option<String>, String> {
    let crate_root = crate_root();
    let suite = crate_root.join("fixtures/prettier");
    let root = suite.join("tests/format/markdown");
    let mut errors = Vec::new();
    validate_fixture_version(&suite, &mut errors);
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    let mut dirs = Vec::new();
    collect_test_dirs_recursive(&root, &mut dirs);
    if dirs.is_empty() {
        return Err(format!("no Markdown snapshots under {}", root.display()));
    }

    let mut collector = Collector::start(&crate_root)?;
    let filter = env::var("PRETTIER_FILTER").ok().filter(|s| !s.is_empty());
    let mut total = Counts::default();
    let mut rows = String::new();
    let mut failures = String::new();
    let mut mismatches = Vec::new();
    let mut unsupported = Vec::new();
    let mut alignment_failures = String::new();
    for dir in dirs {
        let group = dir
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let path = dir.join(SNAPSHOT_DIR_NAME).join(SNAPSHOT_FILE_NAME);
        let contents = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut counts = Counts::default();
        for entry in contents.split("\nexports[`").skip(1) {
            let case = Snapshot::read(entry)
                .ok_or_else(|| format!("invalid snapshot in {}", path.display()))?;
            let id = format!("markdown/{group}/{}", case.name);
            if filter.as_ref().is_some_and(|filter| !id.contains(filter)) {
                continue;
            }
            let (source, filepath) = if case.name.starts_with("snippet: ") {
                (unescape_template(case.input), dir.join("snippet.md"))
            } else {
                let (name, _) = snapshot_key(case.name);
                let filepath = dir.join(name);
                let source = fs::read_to_string(&filepath).map_err(|e| format!("{id}: {e}"))?;
                (source, filepath)
            };
            let layout = match case.format_options(&source) {
                Ok(options) => options,
                Err(reason) => {
                    unsupported.push(format!("- {id}: {reason}"));
                    counts.unsupported += 1;
                    continue;
                }
            };
            let actual = collector
                .format(
                    &source,
                    &case.options,
                    &filepath,
                    layout.syntax,
                    |dispatcher| markdown::format_with_dispatcher(&source, layout, dispatcher),
                )
                .map_err(|e| format!("{id}: {e}"))?;
            let ast = parse(&source);
            let from_ast = collector
                .format(
                    &source,
                    &case.options,
                    &filepath,
                    layout.syntax,
                    |dispatcher| markdown::format_ast_with_dispatcher(&ast, layout, dispatcher),
                )
                .map_err(|e| format!("{id}, AST: {e}"))?;
            if actual.text != from_ast.text || actual.alignment != from_ast.alignment {
                return Err(format!("{id}: source and AST entry points differ"));
            }
            let expected = unescape_template(case.output);
            let replay = collector
                .format(
                    &actual.text,
                    &case.options,
                    &filepath,
                    layout.syntax,
                    |dispatcher| markdown::format_with_dispatcher(&actual.text, layout, dispatcher),
                )
                .map_err(|e| format!("{id}, second pass: {e}"))?;
            counts.tested += 1;
            if actual.alignment.is_some() || replay.alignment.is_some() {
                counts.misaligned += 1;
                for (pass, difference) in [
                    ("first pass", actual.alignment),
                    ("second pass", replay.alignment),
                ] {
                    if let Some(difference) = difference {
                        let line = format!("- {id} ({pass}): {difference}");
                        writeln!(alignment_failures, "{line}").unwrap();
                        if filter.is_some() {
                            println!("{line}");
                        }
                    }
                }
                continue;
            }
            let actual = actual.text;
            let replay = replay.text;
            counts.passed += usize::from(actual == expected && actual == replay);
            counts.matched += usize::from(actual == expected);
            counts.unstable += usize::from(actual != replay);
            if actual != expected || actual != replay {
                let (path, options) = snapshot_key(&id);
                let reason = if actual == replay {
                    KNOWN_DIVERGENCES.reason_for_fixture(path, options, Some(&actual))
                } else {
                    None
                };
                if reason.is_none() {
                    let replay_note = if actual == replay {
                        ""
                    } else {
                        " (second pass differs)"
                    };
                    writeln!(failures, "- {id}{replay_note}").unwrap();
                }
                if filter.is_some() {
                    println!("{id}");
                    if let Some(reason) = reason {
                        println!("Known divergence: {reason}");
                    }
                    print_text_diff(&expected, &actual);
                    if actual != replay {
                        println!("--- Second pass\n{replay}");
                    }
                }
                if actual != expected {
                    mismatches.push((id, reason));
                }
            }
        }
        total.tested += counts.tested;
        total.passed += counts.passed;
        total.matched += counts.matched;
        total.unstable += counts.unstable;
        total.unsupported += counts.unsupported;
        total.misaligned += counts.misaligned;
        write!(
            rows,
            "{group}: {}/{} matched, {} not idempotent, {} unsupported",
            counts.matched,
            counts.tested - counts.misaligned,
            counts.unstable,
            counts.unsupported
        )
        .unwrap();
        if counts.misaligned != 0 {
            write!(rows, ", {} fence alignment differences", counts.misaligned).unwrap();
        }
        rows.push('\n');
    }
    collector.finish()?;
    if filter.is_some() {
        return Ok(None);
    }
    let stale = KNOWN_DIVERGENCES.stale_fixtures(
        &["markdown"],
        mismatches.iter().map(|(id, _)| snapshot_key(id)),
    );
    if !stale.is_empty() {
        return Err(stale.join("\n"));
    }
    let mut divergences = BTreeMap::<_, Vec<_>>::new();
    for (id, reason) in &mismatches {
        if let Some(reason) = reason {
            divergences.entry(*reason).or_default().push(id);
        }
    }
    let known = divergences.values().map(Vec::len).sum::<usize>();
    let Counts {
        tested,
        passed,
        matched,
        unstable,
        unsupported: skipped,
        misaligned,
    } = total;
    if tested == 0 {
        return Err("no supported Markdown comparisons".into());
    }
    let mut report = format!(
        "formatter * prettier {PRETTIER_COMPATIBILITY_VERSION} * markdown\n========================================\nFixture revision: {PRETTIER_FIXTURE_REVISION}\nScope: physical Markdown files and snapshot snippets\n"
    );
    let compared = tested - misaligned;
    let rate = 100.0 * passed as f64 / tested as f64;
    writeln!(
        report,
        "Embedded languages: SWC for nested Markdown, Prettier plugins for other languages"
    )
    .unwrap();
    writeln!(report, "Passed: {passed}/{tested} ({rate:.2}%)").unwrap();
    writeln!(report, "Snapshot matched: {matched}/{compared}").unwrap();
    writeln!(report, "Source and AST output parity: {tested}/{tested}").unwrap();
    writeln!(report, "Mismatched: {}/{compared}", compared - matched).unwrap();
    writeln!(report, "Known mismatches: {known}/{compared}").unwrap();
    writeln!(
        report,
        "Unclassified mismatches: {}/{compared}",
        mismatches.len() - known
    )
    .unwrap();
    writeln!(report, "Not idempotent: {unstable}/{compared}").unwrap();
    writeln!(
        report,
        "Cases compared after fence alignment: {compared}/{tested}"
    )
    .unwrap();
    writeln!(report, "Fence alignment differences: {misaligned}/{tested}").unwrap();
    write!(report, "Unsupported cases: {skipped}\n\n# Suites\n\n{rows}").unwrap();
    if !divergences.is_empty() {
        report.push_str("\n# Known divergences\n");
        for (reason, ids) in divergences {
            writeln!(report, "\n## {reason}\n").unwrap();
            for id in ids {
                writeln!(report, "- {id}").unwrap();
            }
        }
    }
    if !failures.is_empty() {
        write!(report, "\n# Failed\n\n{failures}").unwrap();
    }
    if !alignment_failures.is_empty() {
        write!(
            report,
            "\n# Fence alignment differences\n\n{alignment_failures}"
        )
        .unwrap();
    }
    if !unsupported.is_empty() {
        writeln!(
            report,
            "\n# Unsupported cases\n\n{}",
            unsupported.join("\n")
        )
        .unwrap();
    }
    Ok(Some(report))
}
