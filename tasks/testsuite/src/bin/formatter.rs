use std::{fs, process::ExitCode};

use clap::Parser;
use testsuite::{
    formatter::{
        ConformanceConfig, JAVASCRIPT, TYPESCRIPT, run_conformance, run_markdown_conformance,
    },
    util::crate_root,
};

#[derive(Parser)]
#[command(about = "Compare SWC formatter output with Prettier snapshots")]
struct Args {
    /// Run only the Markdown conformance suite.
    #[arg(long, alias = "markdown-prettier-fences")]
    markdown: bool,
}

fn main() -> ExitCode {
    let mut failed = false;
    if !Args::parse().markdown {
        for config in [JAVASCRIPT, TYPESCRIPT] {
            failed |= update_snapshot(config);
        }
    }
    failed |= update_markdown_snapshot();
    ExitCode::from(u8::from(failed))
}

fn update_markdown_snapshot() -> bool {
    match run_markdown_conformance() {
        Ok(Some(report)) => write_report("markdown", report),
        Ok(None) => false,
        Err(error) => {
            eprintln!("{error}");
            true
        }
    }
}

fn update_snapshot(config: ConformanceConfig) -> bool {
    println!("Running Prettier conformance for {}...", config.language);
    let output = run_conformance(config);
    let mut failed = !output.structural_errors.is_empty();
    for error in output.structural_errors {
        eprintln!("{error}");
    }

    if let Some(report) = output.report {
        failed |= write_report(config.language, report);
    }
    failed
}

fn write_report(language: &str, report: String) -> bool {
    let snapshot_path = crate_root()
        .join("snapshots")
        .join(format!("formatter_prettier_{language}.snap"));
    if let Err(error) = fs::write(&snapshot_path, report) {
        eprintln!("write {}: {error}", snapshot_path.display());
        return true;
    }
    println!("Updated {}", snapshot_path.display());
    false
}
