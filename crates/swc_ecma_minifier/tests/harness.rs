use std::{fs, path::Path};

use tempfile::TempDir;

mod support;

fn fixture(input: &str, output: &str) -> TempDir {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("input.js"), input).unwrap();
    fs::write(directory.path().join("output.js"), output).unwrap();
    directory
}

fn with_options(directory: &Path, options: &str) -> Result<(), String> {
    fs::write(directory.join("options.json"), options).unwrap();
    support::run_fixture(directory)
}

#[test]
fn fixture_discovery() {
    let directory = tempfile::tempdir().unwrap();
    assert!(support::discover(directory.path()).is_err());

    let directory = fixture("value;", "value;\n");
    assert_eq!(
        support::discover(directory.path()).unwrap(),
        [directory.path()]
    );

    fs::remove_file(directory.path().join("input.js")).unwrap();
    assert_eq!(
        support::discover(directory.path()).unwrap(),
        [directory.path()]
    );
    assert!(support::run_fixture(directory.path()).is_err());

    let directory = fixture("value;", "value;\n");
    fs::remove_file(directory.path().join("output.js")).unwrap();
    assert!(support::run_fixture(directory.path()).is_err());
}

#[test]
fn fixture_options() {
    let directory = fixture("value;", "value;\n");
    assert!(with_options(directory.path(), "{}").is_ok());

    assert!(with_options(directory.path(), "{broken").is_err());

    assert!(with_options(directory.path(), r#"{"unknown":true}"#).is_err());

    assert!(with_options(directory.path(), r#"{"source_type":"typo"}"#).is_err());

    assert!(with_options(directory.path(), r#"{"minify":{"unknown":true}}"#).is_err());

    assert!(
        with_options(
            directory.path(),
            r#"{"minify":{"compress":{"unknown":true}}}"#
        )
        .is_err()
    );
}

#[test]
fn fixture_validation() {
    let directory = fixture("let = ;", "");
    assert!(support::run_fixture(directory.path()).is_err());

    let directory = fixture("let value; let value;", "let value;let value;");
    assert!(support::run_fixture(directory.path()).is_err());

    let directory = fixture("value;", "different;");
    assert!(support::run_fixture(directory.path()).is_err());

    let directory = fixture("value;", "value;\r\n");
    assert!(support::run_fixture(directory.path()).is_ok());

    let directory = fixture("value;", " value;\n");
    assert!(support::run_fixture(directory.path()).is_err());
}
