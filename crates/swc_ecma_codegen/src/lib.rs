//! ECMAScript and TypeScript code generation.
//!
//! The printer is based on esbuild's
//! [`js_printer.go`](https://github.com/evanw/esbuild/blob/main/internal/js_printer/js_printer.go)
//! implementation, with modifications for SWC Next.

mod operator;
mod printer;
mod sourcemap;
mod ts_disambiguation;
mod util;

pub use printer::print;
pub use printer::{NumberFormat, Options, PrintResult};
pub use sourcemap::SourceMap;
