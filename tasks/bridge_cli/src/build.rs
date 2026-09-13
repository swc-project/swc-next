use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde::{Deserialize, de::DeserializeOwned};

const TARGET: &str = "wasm32-unknown-unknown";

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
    target: String,
}

struct Component {
    name: String,
    directory: PathBuf,
    cargo_package: String,
    artifact: String,
    binary_name: String,
    version: String,
}

pub(super) fn build(
    component: Option<&str>,
    release: bool,
    output_dir: Option<&Path>,
) -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = root
        .canonicalize()
        .map_err(|error| format!("failed to resolve workspace root: {error}"))?;
    let metadata = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps", "--locked"])
        .current_dir(&root)
        .output()
        .map_err(|error| format!("failed to run cargo metadata: {error}"))?;
    if !metadata.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&metadata.stderr)
        ));
    }
    let metadata: Metadata = serde_json::from_slice(&metadata.stdout)
        .map_err(|error| format!("invalid cargo metadata: {error}"))?;
    let mut components = discover_components(&root, &metadata)?;
    if let Some(name) = component {
        if !components.iter().any(|component| component.name == name) {
            let names: Vec<_> = components
                .iter()
                .map(|component| component.name.as_str())
                .collect();
            return Err(format!(
                "unknown component {name}; expected {}",
                names.join(", ")
            ));
        }
        components.retain(|component| component.name == name);
    }
    if components.is_empty() {
        return Err("no Wasm packages found".to_owned());
    }

    for component in components {
        // Keep each component's feature set independent, especially parser and codegen.
        let mut cargo = Command::new("cargo");
        cargo
            .current_dir(&root)
            .args(["build", "--locked", "--target", TARGET]);
        cargo.args(["--package", &component.cargo_package]);
        if release {
            cargo.args(["--profile", "wasm-release"]);
        }
        run(&mut cargo)?;

        let destination = match output_dir {
            Some(directory) => std::path::absolute(directory)
                .map_err(|error| format!("failed to resolve output directory: {error}"))?,
            None => component.directory.join("src-js"),
        };
        let source = metadata
            .target_directory
            .join(TARGET)
            .join(if release { "wasm-release" } else { "debug" })
            .join(&component.artifact);
        crate::generate(
            &source,
            &destination,
            Some(&component.binary_name),
            &component.version,
        )?;
        if component.name == "core" {
            run(Command::new("node")
                .arg(root.join("scripts/browser-package.mjs"))
                .arg(&destination)
                .current_dir(&root))?;
        }
    }
    Ok(())
}

fn discover_components(root: &Path, metadata: &Metadata) -> Result<Vec<Component>, String> {
    let npm = root.join("npm");
    let mut entries = fs::read_dir(&npm)
        .map_err(|error| format!("failed to read {}: {error}", npm.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read npm directory entry: {error}"))?;
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
        if config.target != TARGET {
            return Err(format!(
                "{} has unsupported Wasm target {}",
                manifest_path.display(),
                config.target
            ));
        }
        if config.binary_name.is_empty()
            || !config
                .binary_name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(format!(
                "{} has an invalid Wasm binary name",
                manifest_path.display()
            ));
        }
        let version = manifest
            .version
            .filter(|version| !version.is_empty())
            .ok_or_else(|| format!("{} is missing a version", manifest_path.display()))?;
        let cargo_manifest = directory.join("Cargo.toml");
        let package = metadata
            .packages
            .iter()
            .find(|package| package.manifest_path == cargo_manifest)
            .ok_or_else(|| format!("missing Cargo package for {}", manifest_path.display()))?;
        let library = package
            .targets
            .iter()
            .find(|target| target.kind.iter().any(|kind| kind == "cdylib"))
            .ok_or_else(|| {
                format!(
                    "{} does not define a cdylib target",
                    cargo_manifest.display()
                )
            })?;
        components.push(Component {
            name: entry
                .file_name()
                .into_string()
                .map_err(|_| "npm directory name must be UTF-8")?,
            directory,
            cargo_package: package.name.clone(),
            artifact: format!("{}.wasm", library.name),
            binary_name: config.binary_name,
            version,
        });
    }
    Ok(components)
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let source =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_slice(&source)
        .map_err(|error| format!("invalid JSON in {}: {error}", path.display()))
}

fn run(command: &mut Command) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|error| format!("failed to run {command:?}: {error}"))?;
    if !status.success() {
        return Err(format!("{command:?} failed with {status}"));
    }
    Ok(())
}
