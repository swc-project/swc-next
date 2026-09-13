mod descriptor;
mod environment;
mod render;
mod wasm;

use std::{fs, path::Path};

use anyhow::Result;

pub(super) fn generate(input: &Path, output_dir: &Path, name: &str, version: &str) -> Result<()> {
    let bytes = fs::read(input)?;
    let (descriptors, stripped) = wasm::extract_metadata(&bytes)?;
    let runtime_name = format!("{name}.runtime.mjs");
    let binding = render::render_binding(descriptors, &runtime_name)?;

    fs::create_dir_all(output_dir)?;
    for (name, content) in [
        (format!("{name}.bridge.mjs"), binding.as_bytes()),
        (runtime_name, include_bytes!("runtime.mjs").as_slice()),
        (format!("{name}.wasm"), stripped.as_slice()),
    ] {
        let output = output_dir.join(name);
        fs::write(&output, content)?;
    }
    environment::write_entries(output_dir, name, version)
}
