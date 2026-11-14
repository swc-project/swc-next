//! Internal native/Wasm parser bindings, wrapped by the public JavaScript API.
//!
//! These low-level APIs and binary protocols do not follow semantic versioning
//! and may change in any release. Use a matching JavaScript wrapper and decoder.

#[cfg(not(target_family = "wasm"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod parser;
