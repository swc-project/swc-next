//! ECMAScript and TypeScript code generation.
//!
//! The printer is a Rust port of esbuild's
//! [`js_printer.go`](https://github.com/evanw/esbuild/blob/main/internal/js_printer/js_printer.go)
//! design.

mod operator;
mod printer;
mod sourcemap;
mod util;

pub use printer::print;
pub use printer::{NumberFormat, Options, PrintResult};
pub use sourcemap::SourceMap;
