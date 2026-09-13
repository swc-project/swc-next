mod build;
mod environment;
mod metadata;
mod render;
mod wasm;

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Build WebAssembly packages and generate JavaScript bindings")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build workspace Wasm packages and emit bindings into their npm src-js directories.
    Build {
        /// Build one component; defaults to every public npm package with Wasm configuration.
        component: Option<String>,
        /// Use the wasm-release Cargo profile.
        #[arg(long, short = 'r')]
        release: bool,
        /// Override the output directory for a single component, relative to the current directory.
        #[arg(long, short = 'o', requires = "component", value_name = "DIR")]
        output_dir: Option<PathBuf>,
    },
    /// Emit JavaScript bindings, environment entry points, runtime, and stripped Wasm.
    Generate {
        #[arg(value_name = "WASM")]
        input: PathBuf,
        #[arg(long, value_name = "DIR")]
        output_dir: PathBuf,
        /// Output file prefix; defaults to the input file stem.
        #[arg(long, value_name = "NAME")]
        out_name: Option<String>,
        /// Package version exported by the environment entry points.
        #[arg(long, value_name = "VERSION")]
        package_version: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Build {
            component,
            release,
            output_dir,
        } => build::build(component.as_deref(), release, output_dir.as_deref()),
        Command::Generate {
            input,
            output_dir,
            out_name,
            package_version,
        } => generate(&input, &output_dir, out_name.as_deref(), &package_version),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bridge_cli: {error}");
            ExitCode::FAILURE
        }
    }
}

fn generate(
    input: &Path,
    output_dir: &Path,
    out_name: Option<&str>,
    version: &str,
) -> Result<(), String> {
    let name = out_name
        .or_else(|| input.file_stem().and_then(|stem| stem.to_str()))
        .ok_or("input must have a UTF-8 file stem, or specify --out-name")?;
    if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\', '\0']) {
        return Err("output name must be a non-empty file name".to_owned());
    }

    let bytes =
        fs::read(input).map_err(|error| format!("failed to read {}: {error}", input.display()))?;
    let (descriptors, stripped) = wasm::extract_metadata(&bytes)?;
    let runtime_name = format!("{name}.runtime.mjs");
    let binding = render::render_binding(descriptors, &runtime_name)?;

    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;
    for (name, content) in [
        (format!("{name}.bridge.mjs"), binding.as_bytes()),
        (
            runtime_name,
            include_bytes!("../templates/runtime.mjs").as_slice(),
        ),
        (format!("{name}.wasm"), stripped.as_slice()),
    ] {
        let output = output_dir.join(name);
        fs::write(&output, content)
            .map_err(|error| format!("failed to write {}: {error}", output.display()))?;
    }
    environment::write_entries(output_dir, name, version)
}
