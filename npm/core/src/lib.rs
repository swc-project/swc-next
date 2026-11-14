//! Internal native/Wasm binding entry points, wrapped by the public JavaScript API.
//!
//! These low-level APIs and binary protocols do not follow semantic versioning
//! and may change in any release. Use matching JavaScript wrappers and codecs.

#[cfg(not(target_family = "wasm"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod generator;
pub mod parser;
pub mod transformer;
