use std::{
    env, fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

pub fn crate_root() -> PathBuf {
    workspace_root().unwrap().join("tasks/testsuite")
}

pub fn workspace_root() -> io::Result<PathBuf> {
    let path = env::current_dir()?;
    for parent in path.ancestors() {
        if parent.join("Cargo.lock").is_file() {
            return Ok(parent.to_path_buf());
        }
    }
    Err(ErrorKind::NotFound.into())
}

pub fn read_optional(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    name.into()
}
