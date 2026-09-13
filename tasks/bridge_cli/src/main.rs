mod bindings;
mod build;

use std::{path::PathBuf, process::ExitCode};

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
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Build {
            component,
            release,
            output_dir,
        } => build::build(component.as_deref(), release, output_dir.as_deref()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bridge_cli: {error}");
            ExitCode::FAILURE
        }
    }
}
