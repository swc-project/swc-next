mod descriptor;
mod environment;
mod render;
mod wasm;

use std::{fs, path::Path};

pub(super) fn generate(
    input: &Path,
    output_dir: &Path,
    name: &str,
    version: &str,
) -> Result<(), String> {
    let bytes =
        fs::read(input).map_err(|error| format!("failed to read {}: {error}", input.display()))?;
    let (descriptors, stripped) = wasm::extract_metadata(&bytes)?;
    let runtime_name = format!("{name}.runtime.mjs");
    let binding = render::render_binding(descriptors, &runtime_name)?;

    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;
    for (name, content) in [
        (format!("{name}.bridge.mjs"), binding.as_bytes()),
        (runtime_name, include_bytes!("runtime.mjs").as_slice()),
        (format!("{name}.wasm"), stripped.as_slice()),
    ] {
        let output = output_dir.join(name);
        fs::write(&output, content)
            .map_err(|error| format!("failed to write {}: {error}", output.display()))?;
    }
    environment::write_entries(output_dir, name, version)
}
