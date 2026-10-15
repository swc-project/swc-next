use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use cargo_metadata::{Metadata, MetadataCommand};
use serde::Deserialize;

pub(super) const TARGET: &str = "wasm32-unknown-unknown";

#[derive(Deserialize)]
struct Manifest {
    version: Option<String>,
    wasm: Option<WasmConfig>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmConfig {
    binary_name: String,
}

pub(super) struct Component {
    pub name: String,
    pub output_dir: PathBuf,
    pub cargo_package: String,
    pub input: PathBuf,
    pub binary_name: String,
    pub version: String,
}

/// Validated package metadata and resolved paths used by the build stage.
pub(super) struct Workspace {
    pub root: PathBuf,
    pub components: Vec<Component>,
}

pub(super) fn read(component: &str, output_dir: &Path) -> Result<Workspace> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = root.canonicalize()?;
    let metadata = MetadataCommand::new()
        .current_dir(&root)
        .no_deps()
        .verbose(true)
        .other_options(vec!["--locked".into()])
        .exec()?;
    let mut components = discover_components(&metadata)?;
    components.retain(|candidate| candidate.name == component);
    ensure!(!components.is_empty(), "no Wasm packages found");

    for component in &mut components {
        component.output_dir = std::path::absolute(output_dir)?;
    }
    Ok(Workspace { root, components })
}

fn discover_components(metadata: &Metadata) -> Result<Vec<Component>> {
    let mut components = Vec::new();
    for package in &metadata.packages {
        let directory = package.manifest_path.as_std_path().parent().unwrap();
        let manifest_path = directory.join("package.json");
        let source = match fs::read(manifest_path) {
            Ok(source) => source,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        let manifest: Manifest = serde_json::from_slice(&source)?;
        let Some(config) = manifest.wasm else {
            continue;
        };
        let version = manifest
            .version
            .filter(|version| !version.is_empty())
            .context("missing package version")?;
        let library = package
            .targets
            .iter()
            .find(|target| target.is_cdylib())
            .context("missing cdylib target")?;
        components.push(Component {
            name: directory
                .file_name()
                .and_then(|name| name.to_str())
                .context("Wasm package directory name must be UTF-8")?
                .to_owned(),
            output_dir: PathBuf::default(),
            cargo_package: package.name.to_string(),
            input: PathBuf::from(format!("{}.wasm", library.name)),
            binary_name: config.binary_name,
            version,
        });
    }
    components.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(components)
}
