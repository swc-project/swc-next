use std::num::NonZeroUsize;

use clap::{Parser, ValueEnum};

#[derive(Parser)]
pub struct Args {
    /// JavaScript or TypeScript input file.
    pub file: String,

    #[arg(value_enum)]
    pub operation: Operation,

    /// Number of operations to run after reading the input.
    #[arg(default_value = "10")]
    pub iterations: NonZeroUsize,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Operation {
    Parse,
    IsolatedDeclaration,
    TypeStrip,
}
