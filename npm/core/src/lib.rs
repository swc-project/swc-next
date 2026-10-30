#[cfg(not(target_family = "wasm"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod codegen;
pub mod parser;

#[cfg(feature = "test-codec")]
pub mod test;
