//! Internal native/Wasm transport for SWC Next's JavaScript bindings.
//!
//! This crate's APIs and ABI do not follow semantic versioning and may change in
//! any release. Use matching versions of the bridge macros, Rust components,
//! generated JavaScript bindings, and runtime.
//!
//! Native strings use Latin-1 for ASCII and UTF-16 transcoding otherwise.
//! N-API copies the output into the JavaScript heap before the Rust allocation
//! is released.
//! WebAssembly retains its existing string path.

extern crate self as swc_next_bridge;

mod wire;
pub use swc_next_bridge_macros::bridge;
#[doc(hidden)]
pub use wire::private;
pub use wire::{BridgeDecode, BridgeDecodeOptional, BridgeEncode, BridgeWire};

#[cfg(not(target_family = "wasm"))]
#[doc(hidden)]
pub use {napi, napi_derive, wire::NativeAbi};

#[cfg(not(target_family = "wasm"))]
pub use wire::create_buffer;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[doc(hidden)]
pub mod wasm;

#[cfg(all(
    target_family = "wasm",
    not(all(target_arch = "wasm32", target_os = "unknown"))
))]
compile_error!("only wasm32-unknown-unknown is supported by the WebAssembly bridge");
