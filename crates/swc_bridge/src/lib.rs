//! Native strings use Latin-1 for ASCII and UTF-16 transcoding otherwise.
//! N-API copies the output into the JavaScript heap before the Rust allocation
//! is released.
//! WebAssembly retains its existing string path.

extern crate self as swc_next_bridge;

mod wire;
pub use swc_next_bridge_macros::bridge;
#[doc(hidden)]
pub use wire::private;
pub use wire::{BridgeDecode, BridgeEncode, BridgeWire};

#[cfg(feature = "parser")]
mod parser;
#[cfg(feature = "parser")]
pub use parser::{CommentMode, Lang, ParserOptions, SourceType, parse_to_buffer};

#[cfg(feature = "codegen")]
mod codegen;
#[cfg(feature = "codegen")]
pub use {
    codegen::{CodegenOptions, CodegenOutput, print_from_buffer},
    swc_next_ecma_ast::ffi::ast::DeserializeError,
};

#[cfg(not(target_family = "wasm"))]
#[doc(hidden)]
pub use {napi, napi_derive, wire::NativeAbi};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[doc(hidden)]
pub mod wasm;

#[cfg(all(
    target_family = "wasm",
    not(all(target_arch = "wasm32", target_os = "unknown"))
))]
compile_error!("only wasm32-unknown-unknown is supported by the WebAssembly bridge");
