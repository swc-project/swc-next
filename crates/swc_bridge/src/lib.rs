extern crate self as swc_next_bridge;

#[cfg(feature = "codegen")]
mod codegen;
#[cfg(feature = "parser")]
mod parser;
mod wire;

#[cfg(feature = "codegen")]
pub use codegen::{CodegenOptions, CodegenOutput, print_from_buffer};
#[cfg(feature = "parser")]
pub use parser::{CommentMode, Lang, ParserOptions, SourceType, parse_to_buffer};
pub use swc_next_bridge_macros::bridge;
#[cfg(feature = "codegen")]
pub use swc_next_ecma_ast::ffi::ast::DeserializeError;
#[cfg(not(target_family = "wasm"))]
#[doc(hidden)]
pub use wire::NativeAbi;
#[doc(hidden)]
pub use wire::private;
pub use wire::{BridgeDecode, BridgeEncode, BridgeWire};
