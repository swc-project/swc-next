use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, de::DeserializeOwned};

pub(super) const TARGET: &str = "wasm32-unknown-unknown";

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<CargoPackage>,
    target_directory: PathBuf,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    manifest_path: PathBuf,
    targets: Vec<CargoTarget>,
}

#[derive(Deserialize)]
struct CargoTarget {
    name: String,
    kind: Vec<String>,
}

#[derive(Deserialize)]
struct Manifest {
    #[serde(default)]
    private: bool,
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
    pub profile: &'static str,
    pub components: Vec<Component>,
}

pub(super) fn read(
    component: Option<&str>,
    release: bool,
    output_dir: Option<&Path>,
) -> Result<Workspace> {
    let profile = if release { "wasm-release" } else { "debug" };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = root.canonicalize()?;
    let metadata = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps", "--locked"])
        .current_dir(&root)
        .stderr(Stdio::inherit())
        .output()?;
    ensure!(metadata.status.success(), "cargo metadata failed");
    let metadata: Metadata = serde_json::from_slice(&metadata.stdout)?;
    let mut components = discover_components(&root, &metadata)?;
    if let Some(name) = component {
        ensure!(
            components.iter().any(|component| component.name == name),
            "unknown component"
        );
        components.retain(|component| component.name == name);
    }
    ensure!(!components.is_empty(), "no Wasm packages found");

    for component in &mut components {
        if let Some(directory) = output_dir {
            component.output_dir = std::path::absolute(directory)?;
        }
        component.input = metadata
            .target_directory
            .join(TARGET)
            .join(profile)
            .join(&component.input);
    }
    Ok(Workspace {
        root,
        profile,
        components,
    })
}

fn discover_components(root: &Path, metadata: &Metadata) -> Result<Vec<Component>> {
    let npm = root.join("npm");
    let mut entries = fs::read_dir(&npm)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    let mut components = Vec::new();
    for entry in entries {
        let directory = entry.path();
        let manifest_path = directory.join("package.json");
        if !directory.is_dir() || !manifest_path.exists() {
            continue;
        }
        let manifest: Manifest = read_json(&manifest_path)?;
        if manifest.private {
            continue;
        }
        let Some(config) = manifest.wasm else {
            continue;
        };
        ensure!(
            !config.binary_name.is_empty()
                && config
                    .binary_name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')),
            "invalid Wasm binary name"
        );
        let version = manifest
            .version
            .filter(|version| !version.is_empty())
            .context("missing package version")?;
        let cargo_manifest = directory.join("Cargo.toml");
        let package = metadata
            .packages
            .iter()
            .find(|package| package.manifest_path == cargo_manifest)
            .context("missing Cargo package")?;
        let library = package
            .targets
            .iter()
            .find(|target| target.kind.iter().any(|kind| kind == "cdylib"))
            .context("missing cdylib target")?;
        components.push(Component {
            name: entry
                .file_name()
                .into_string()
                .ok()
                .context("npm directory name must be UTF-8")?,
            output_dir: directory.join("src-js"),
            cargo_package: package.name.clone(),
            input: PathBuf::from(format!("{}.wasm", library.name)),
            binary_name: config.binary_name,
            version,
        });
    }
    Ok(components)
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let source = fs::read(path)?;
    Ok(serde_json::from_slice(&source)?)
}
