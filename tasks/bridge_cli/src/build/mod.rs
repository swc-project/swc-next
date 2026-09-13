mod metadata;

use std::{path::Path, process::Command};

use anyhow::{Result, ensure};

use crate::bindings;

pub(super) fn build(
    component: Option<&str>,
    release: bool,
    output_dir: Option<&Path>,
) -> Result<()> {
    let metadata = metadata::read(component, release, output_dir)?;
    for component in &metadata.components {
        // Keep each component's feature set independent, especially parser and codegen.
        let mut cargo = Command::new("cargo");
        cargo
            .current_dir(&metadata.root)
            .args(["build", "--locked", "--target", metadata::TARGET])
            .args(["--package", &component.cargo_package]);
        if metadata.profile != "debug" {
            cargo.args(["--profile", metadata.profile]);
        }
        let status = cargo.status()?;
        ensure!(status.success(), "cargo build failed");

        bindings::generate(
            &component.input,
            &component.output_dir,
            &component.binary_name,
            &component.version,
        )?;
    }
    Ok(())
}
