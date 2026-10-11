//! ECMAScript and TypeScript code generation.
//!
//! The printer is based on esbuild's MIT-licensed
//! [`js_printer.go`](https://github.com/evanw/esbuild/blob/main/internal/js_printer/js_printer.go)
//! implementation, with modifications for SWC Next.
//! MIT license: see `THIRD_PARTY_NOTICES` at the repository root.

mod operator;
mod printer;
mod sourcemap;
mod util;

pub use printer::print;
pub use printer::{NumberFormat, Options, PrintResult};
pub use sourcemap::SourceMap;
