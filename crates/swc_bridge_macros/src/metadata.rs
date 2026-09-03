use std::{
    collections::HashSet,
    env,
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
    sync::{LazyLock, Mutex},
};

use serde::Serialize;

const METADATA_DIRECTORY_ENV: &str = "SWC_BRIDGE_METADATA_DIR";

static PREPARED_FILES: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum BridgeDescriptor {
    Function(BridgeFunction),
    StringEnum(StringEnum),
    WireMapping(WireMapping),
}

#[derive(Debug, Serialize)]
pub(super) struct BridgeFunction {
    pub rust_name: String,
    pub js_name: String,
    pub wasm_export_name: String,
    pub parameters: Vec<BridgeParameter>,
    pub result: String,
    pub fallible: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct BridgeParameter {
    pub rust_name: String,
    pub js_name: String,
    pub semantic_type: String,
}

#[derive(Debug, Serialize)]
pub(super) struct StringEnum {
    pub rust_name: String,
    pub js_name: String,
    pub variants: Vec<StringEnumVariant>,
}

#[derive(Debug, Serialize)]
pub(super) struct StringEnumVariant {
    pub rust_name: String,
    pub js_value: String,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum WireDirection {
    Decode,
    Encode,
}

#[derive(Debug, Serialize)]
pub(super) struct WireMapping {
    pub direction: WireDirection,
    pub semantic_type: String,
    pub wire_type: String,
}

pub(super) fn emit_descriptor(descriptor: &BridgeDescriptor) -> Result<(), String> {
    let Some(path) = metadata_path() else {
        return Ok(());
    };
    let mut prepared_files = PREPARED_FILES
        .lock()
        .map_err(|_| "bridge metadata lock is poisoned".to_owned())?;
    if prepared_files.insert(path.clone()) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create bridge metadata directory: {error}"))?;
        }
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!("failed to replace bridge metadata file: {error}"));
            }
        }
    }

    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .map_err(|error| format!("failed to open bridge metadata file: {error}"))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, descriptor)
        .map_err(|error| format!("failed to serialize bridge metadata: {error}"))?;
    writer
        .write_all(b"\n")
        .and_then(|()| writer.flush())
        .map_err(|error| format!("failed to write bridge metadata: {error}"))
}

fn metadata_path() -> Option<PathBuf> {
    let directory = env::var_os(METADATA_DIRECTORY_ENV)?;
    let package_name = env::var_os("CARGO_PKG_NAME")?;
    Some(PathBuf::from(directory).join(package_name))
}
