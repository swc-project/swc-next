mod bindings;
mod build;

use std::{ffi::OsString, path::PathBuf};

use anyhow::Result;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Build WebAssembly packages and generate JavaScript bindings")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build a Wasm component and generate JavaScript bindings.
    Build {
        /// Component to build.
        component: String,
        /// Output directory, relative to the current directory.
        #[arg(long, short = 'o', value_name = "DIR")]
        output_dir: PathBuf,
        /// Additional arguments passed to cargo build.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        cargo_args: Vec<OsString>,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Build {
            component,
            output_dir,
            cargo_args,
        } => build::build(&component, &output_dir, &cargo_args),
    }
}
