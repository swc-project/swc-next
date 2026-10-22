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
pub use codegen::{CodegenOptions, CodegenOutput, print_from_buffer};

#[cfg(feature = "transformer")]
mod transformer;
#[cfg(all(feature = "transformer", feature = "parser"))]
pub use transformer::parse_transform_to_buffer;
#[cfg(all(feature = "transformer", feature = "codegen"))]
pub use transformer::transform_print_from_buffer;
#[cfg(all(feature = "transformer", feature = "parser", feature = "codegen"))]
pub use transformer::transform_source_to_buffer;
#[cfg(feature = "transformer")]
pub use transformer::{TransformPassOptions, transform_from_buffer};

#[cfg(any(feature = "codegen", feature = "transformer"))]
pub use swc_next_ecma_ast::ffi::ast::DeserializeError;

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
