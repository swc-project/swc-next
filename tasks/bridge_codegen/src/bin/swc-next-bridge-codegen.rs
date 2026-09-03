use std::{env, path::PathBuf, process::ExitCode};

use bridge_codegen::generate_binding;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("swc-next-bridge-codegen: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args().skip(1);
    if arguments.next().as_deref() != Some("generate") {
        return Err(usage());
    }

    let mut metadata_directory = None;
    let mut output = None;
    let mut packages = Vec::new();
    let mut runtime = None;
    while let Some(argument) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{argument} requires a value"))?;
        match argument.as_str() {
            "--metadata-dir" => metadata_directory = Some(PathBuf::from(value)),
            "--output" => output = Some(PathBuf::from(value)),
            "--package" => packages.push(value),
            "--runtime" => runtime = Some(value),
            _ => return Err(format!("unknown option {argument}\n{}", usage())),
        }
    }

    let metadata_directory =
        metadata_directory.ok_or_else(|| "--metadata-dir is required".to_owned())?;
    let output = output.ok_or_else(|| "--output is required".to_owned())?;
    let runtime = runtime.ok_or_else(|| "--runtime is required".to_owned())?;
    generate_binding(&metadata_directory, &packages, &runtime, &output)
}

fn usage() -> String {
    "usage: swc-next-bridge-codegen generate --metadata-dir <path> --runtime <file> --output <path> --package <name>...".to_owned()
}
