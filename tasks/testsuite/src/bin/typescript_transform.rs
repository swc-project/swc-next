use std::{
    fmt::Write as _,
    fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;
use rayon::prelude::*;
use testsuite::{
    cases::typescript_transform::{
        TypeScriptTransformInput, TypeScriptTransformMode, TypeScriptTransformUnit,
    },
    runner::typescript_transform::{
        OtherTransformReason, TypeScriptTransformCheckResult, TypeScriptTransformRunner,
    },
    util::crate_root,
};

const SNAPSHOT_FILE: &str = "typescript_transform.snap";
const ARTIFACT_DIR: &str = "outputs/typescript_transform";
const TYPESCRIPT_CASES_DIR: &str = "fixtures/typescript/tests/cases";

#[derive(Parser)]
#[command(about = "Check TypeScript-only transform invariants across the upstream corpus")]
struct AppArgs {
    /// Print every full-corpus failure instead of the first 100.
    #[arg(long)]
    verbose: bool,
}

fn main() -> Result<ExitCode, Box<dyn std::error::Error>> {
    rayon::ThreadPoolBuilder::new()
        .stack_size(32 * 1024 * 1024)
        .build_global()?;

    let args = AppArgs::parse();
    let inputs = TypeScriptTransformInput::read_all()?;
    let checks = inputs
        .iter()
        .flat_map(|input| {
            input
                .units
                .iter()
                .enumerate()
                .flat_map(move |(unit_index, unit)| {
                    let has_multiple_modes = input.modes.len() > 1;
                    input.modes.iter().map(move |&mode| {
                        (
                            mode,
                            has_multiple_modes,
                            &input.relative_path,
                            unit_index,
                            unit,
                        )
                    })
                })
        })
        .collect::<Vec<_>>();
    let results = checks
        .par_iter()
        .map(|(mode, _, _, _, unit)| TypeScriptTransformRunner::check(*mode, unit))
        .collect::<Vec<_>>();

    let mut report = SnapshotReport::new(inputs.len());
    let mut artifacts = TransformArtifacts::new()?;
    for ((mode, has_multiple_modes, relative_path, unit_index, unit), result) in
        checks.iter().zip(results)
    {
        report.record(*mode, relative_path, unit, &result);
        artifacts.record(
            *mode,
            *has_multiple_modes,
            relative_path,
            *unit_index,
            unit,
            &result,
        )?;
    }
    report.messages.sort_unstable();

    let message_limit = if args.verbose { usize::MAX } else { 100 };
    for message in report.messages.iter().take(message_limit) {
        eprintln!("{message}");
    }
    if report.messages.len() > message_limit {
        eprintln!(
            "... {} more failures omitted",
            report.messages.len() - message_limit
        );
    }

    let snapshot_path = crate_root().join("snapshots").join(SNAPSHOT_FILE);
    fs::write(&snapshot_path, report.render())?;
    println!("Updated {}", snapshot_path.display());
    let artifact_path = artifacts.write()?;
    println!(
        "Wrote {} physical fixture outputs to {}",
        inputs.len(),
        artifact_path.display()
    );

    let summary = &report.total;
    println!(
        "TypeScript transform invariants: {} transformed, {} parse skipped, {} semantic skipped, {} decorators-transform skipped, {} module-transform skipped, {} failed, {} panicked, {} total",
        summary.transformed,
        summary.input_parse_skipped,
        summary.input_semantic_skipped,
        summary.decorator_transform_skipped,
        summary.module_transform_skipped,
        summary.failed,
        summary.panicked,
        summary.total(),
    );

    Ok(if summary.failed == 0 && summary.panicked == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn check_label(
    mode: TypeScriptTransformMode,
    relative_path: &std::path::Path,
    unit: &TypeScriptTransformUnit,
) -> String {
    format!(
        "{} :: {} [{}]",
        relative_path.display(),
        unit.name,
        mode.name()
    )
}

#[derive(Default)]
struct Summary {
    transformed: usize,
    input_parse_skipped: usize,
    input_semantic_skipped: usize,
    decorator_transform_skipped: usize,
    module_transform_skipped: usize,
    failed: usize,
    panicked: usize,
}

impl Summary {
    fn record(&mut self, result: &TypeScriptTransformCheckResult) {
        match result {
            TypeScriptTransformCheckResult::Transformed(_) => self.transformed += 1,
            TypeScriptTransformCheckResult::InputParseSkipped => self.input_parse_skipped += 1,
            TypeScriptTransformCheckResult::InputSemanticSkipped => {
                self.input_semantic_skipped += 1;
            }
            TypeScriptTransformCheckResult::OtherTransformSkipped(reason) => match reason {
                OtherTransformReason::Decorators => self.decorator_transform_skipped += 1,
                OtherTransformReason::Module => self.module_transform_skipped += 1,
            },
            TypeScriptTransformCheckResult::Failed(_) => self.failed += 1,
            TypeScriptTransformCheckResult::Panic(_) => self.panicked += 1,
        }
    }

    fn total(&self) -> usize {
        self.transformed
            + self.input_parse_skipped
            + self.input_semantic_skipped
            + self.decorator_transform_skipped
            + self.module_transform_skipped
            + self.failed
            + self.panicked
    }

    fn write_to(&self, output: &mut String) {
        let total = self.total();
        self.write_count(output, "Transformed", self.transformed, total);
        self.write_count(
            output,
            "Input parse skipped",
            self.input_parse_skipped,
            total,
        );
        self.write_count(
            output,
            "Input semantic skipped",
            self.input_semantic_skipped,
            total,
        );
        self.write_count(
            output,
            "Decorators transform skipped",
            self.decorator_transform_skipped,
            total,
        );
        self.write_count(
            output,
            "Module transform skipped",
            self.module_transform_skipped,
            total,
        );
        self.write_count(output, "Failed", self.failed, total);
        self.write_count(output, "Panicked", self.panicked, total);
    }

    fn write_count(&self, output: &mut String, label: &str, count: usize, total: usize) {
        let rate = if total == 0 {
            0.0
        } else {
            count as f64 / total as f64 * 100.0
        };
        writeln!(output, "{label}: {count}/{total} ({rate:.2}%)").unwrap();
    }
}

struct SnapshotReport {
    physical_fixtures: usize,
    total: Summary,
    messages: Vec<String>,
}

impl SnapshotReport {
    fn new(physical_fixtures: usize) -> Self {
        Self {
            physical_fixtures,
            total: Summary::default(),
            messages: Vec::new(),
        }
    }

    fn record(
        &mut self,
        mode: TypeScriptTransformMode,
        relative_path: &std::path::Path,
        unit: &TypeScriptTransformUnit,
        result: &TypeScriptTransformCheckResult,
    ) {
        self.total.record(result);

        let message = match result {
            TypeScriptTransformCheckResult::Failed(error) => Some(format!(
                "Failed: {} - {error}",
                check_label(mode, relative_path, unit)
            )),
            TypeScriptTransformCheckResult::Panic(message) => Some(format!(
                "Panic: {} - {message}",
                check_label(mode, relative_path, unit)
            )),
            _ => None,
        };
        if let Some(message) = message {
            self.messages.push(message);
        }
    }

    fn render(&self) -> String {
        let mut output = String::from("typescript transform\n====================\n\n");
        writeln!(output, "Physical fixtures: {}", self.physical_fixtures).unwrap();
        output.push_str("\nChecks\n------\n");
        self.total.write_to(&mut output);
        if !self.messages.is_empty() {
            output.push_str("\nMessages\n========\n");
            for message in &self.messages {
                writeln!(output, "{message}").unwrap();
            }
        }
        output
    }
}

struct TransformArtifacts {
    root: PathBuf,
    current_path: Option<PathBuf>,
    current_unit: Option<usize>,
    output: String,
}

impl TransformArtifacts {
    fn new() -> io::Result<Self> {
        let root = crate_root().join(ARTIFACT_DIR);
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        Ok(Self {
            root,
            current_path: None,
            current_unit: None,
            output: String::new(),
        })
    }

    fn record(
        &mut self,
        mode: TypeScriptTransformMode,
        has_multiple_modes: bool,
        relative_path: &std::path::Path,
        unit_index: usize,
        unit: &TypeScriptTransformUnit,
        result: &TypeScriptTransformCheckResult,
    ) -> io::Result<()> {
        if self.current_path.as_deref() != Some(relative_path) {
            self.flush_current()?;
            self.current_path = Some(relative_path.to_path_buf());
            writeln!(
                self.output,
                "TypeScript transform output\n===========================\n\nInput: {}",
                relative_path.display()
            )
            .unwrap();
        }
        if self.current_unit != Some(unit_index) {
            self.current_unit = Some(unit_index);
            let unit = format!("{unit_index:03} {}", unit.name);
            writeln!(
                self.output,
                "\n{unit}\n{}",
                "=".repeat(unit.chars().count())
            )
            .unwrap();
        }

        if has_multiple_modes {
            let mode = mode.name();
            writeln!(
                self.output,
                "\n{mode}\n{}",
                "-".repeat(mode.chars().count())
            )
            .unwrap();
        } else {
            self.output.push('\n');
        }
        match result {
            TypeScriptTransformCheckResult::Transformed(transformed) if transformed.is_empty() => {
                self.output.push_str("<empty output>\n");
            }
            TypeScriptTransformCheckResult::Transformed(transformed) => {
                self.output.push_str(transformed);
                if !transformed.ends_with('\n') {
                    self.output.push('\n');
                }
            }
            TypeScriptTransformCheckResult::InputParseSkipped => {
                self.output.push_str("<input parse skipped>\n");
            }
            TypeScriptTransformCheckResult::InputSemanticSkipped => {
                self.output.push_str("<input semantic skipped>\n");
            }
            TypeScriptTransformCheckResult::OtherTransformSkipped(reason) => {
                writeln!(self.output, "<requires {reason}>").unwrap();
            }
            TypeScriptTransformCheckResult::Failed(error) => {
                writeln!(self.output, "<failed: {error}>").unwrap();
            }
            TypeScriptTransformCheckResult::Panic(message) => {
                writeln!(self.output, "<panic: {message}>").unwrap();
            }
        }
        Ok(())
    }

    fn write(mut self) -> io::Result<PathBuf> {
        self.flush_current()?;
        Ok(self.root)
    }

    fn flush_current(&mut self) -> io::Result<()> {
        let Some(relative_path) = self.current_path.take() else {
            return Ok(());
        };
        write_artifact(&self.root, &relative_path, &self.output)?;
        self.current_unit = None;
        self.output.clear();
        Ok(())
    }
}

fn write_artifact(root: &Path, relative_path: &Path, output: &str) -> io::Result<()> {
    let source_link = root.join(relative_path);
    let path = source_link.with_added_extension("snap");
    let parent = path
        .parent()
        .expect("a TypeScript transform output has a parent directory");
    fs::create_dir_all(parent)?;
    fs::write(path, output)?;

    let source_target = source_link_target(relative_path);
    symlink_file(&source_target, &source_link)
}

fn source_link_target(relative_path: &Path) -> PathBuf {
    let link_path = Path::new(ARTIFACT_DIR).join(relative_path);
    let link_parent = link_path
        .parent()
        .expect("a TypeScript source link has a parent directory");
    let mut target = PathBuf::new();
    for _ in link_parent.components() {
        target.push("..");
    }
    target.push(TYPESCRIPT_CASES_DIR);
    target.push(relative_path);
    target
}

#[cfg(unix)]
fn symlink_file(original: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

#[cfg(windows)]
fn symlink_file(original: &Path, link: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(original, link)
}
