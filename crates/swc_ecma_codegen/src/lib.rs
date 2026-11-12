//! ECMAScript and TypeScript code generation.
//!
//! The generator is based on esbuild's
//! [`js_printer.go`](https://github.com/evanw/esbuild/blob/main/internal/js_printer/js_printer.go)
//! implementation, with modifications for SWC Next.

mod operator;
mod printer;
mod sourcemap;
mod util;

pub use printer::generate;
pub use printer::{GenerateResult, NumberFormat, Options};
pub use sourcemap::SourceMap;
