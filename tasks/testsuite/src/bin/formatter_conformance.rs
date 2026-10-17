use std::{fs, process::ExitCode};

use testsuite::{
    formatter_conformance::{ConformanceConfig, JAVASCRIPT, TYPESCRIPT, run_conformance},
    util::crate_root,
};

fn main() -> ExitCode {
    let mut failed = false;
    for config in [JAVASCRIPT, TYPESCRIPT] {
        failed |= update_snapshot(config);
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn update_snapshot(config: ConformanceConfig) -> bool {
    println!("Running Prettier conformance for {}...", config.language);
    let output = run_conformance(config);
    if !output.structural_errors.is_empty() {
        for error in output.structural_errors {
            eprintln!("{error}");
        }
        return true;
    }

    let Some(report) = output.report else {
        return false;
    };
    let snapshot_path = crate_root()
        .join("snapshots")
        .join(format!("formatter_prettier_{}.snap", config.language));
    if let Err(error) = fs::write(&snapshot_path, report) {
        eprintln!("write {}: {error}", snapshot_path.display());
        return true;
    }
    println!("Updated {}", snapshot_path.display());
    false
}
