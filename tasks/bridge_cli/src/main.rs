mod metadata;
mod render;

use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Generate JavaScript bindings from bridge metadata")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate JavaScript bindings for the selected packages.
    Generate {
        #[arg(long, value_name = "PATH")]
        metadata_dir: PathBuf,
        #[arg(long, value_name = "FILE")]
        runtime: String,
        #[arg(long, value_name = "PATH")]
        output: PathBuf,
        #[arg(long = "package", value_name = "NAME", required = true)]
        packages: Vec<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Generate {
            metadata_dir,
            runtime,
            output,
            packages,
        } => render::generate_binding(&metadata_dir, &packages, &runtime, &output),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bridge_cli: {error}");
            ExitCode::FAILURE
        }
    }
}
