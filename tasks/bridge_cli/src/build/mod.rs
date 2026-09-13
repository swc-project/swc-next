mod metadata;

use std::{
    ffi::OsString,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, ensure};
use cargo_metadata::Message;

use crate::bindings;

pub(super) fn build(component: &str, output_dir: &Path, cargo_args: &[OsString]) -> Result<()> {
    let metadata = metadata::read(component, output_dir)?;
    for component in &metadata.components {
        // Keep each component's feature set independent, especially parser and codegen.
        let output = Command::new("cargo")
            .current_dir(&metadata.root)
            .args(["build", "--locked", "--target", metadata::TARGET])
            .args(["--package", &component.cargo_package])
            .args(cargo_args)
            .arg("--message-format=json-render-diagnostics")
            .stderr(Stdio::inherit())
            .output()?;
        ensure!(output.status.success(), "cargo build failed");

        // Cargo reports the actual path, including custom profiles and target directories.
        let mut input = None;
        for message in Message::parse_stream(output.stdout.as_slice()) {
            if let Message::CompilerArtifact(artifact) = message? {
                input = artifact
                    .filenames
                    .into_iter()
                    .find(|path| path.as_std_path().file_name() == component.input.file_name())
                    .or(input);
            }
        }
        let input = input.context("cargo build did not produce the component's Wasm artifact")?;
        bindings::generate(
            input.as_std_path(),
            &component.output_dir,
            &component.binary_name,
            &component.version,
        )?;
    }
    Ok(())
}
