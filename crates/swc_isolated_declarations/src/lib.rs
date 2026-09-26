//! TypeScript isolated declaration generation for the flat swc-next AST.
//!
//! [`transform()`] first traces dependencies across declaration surfaces using
//! the input semantic model, then rewrites retained declarations in place. Dependency
//! analysis lives in `type_usage`; shared expression, key, and accessor decisions
//! live in `surface`; `transform` owns AST mutation. Start at [`transform()`],
//! then follow dependency collection before reading the emit modules by declaration
//! kind. Accessor output and property-key spelling have separate transform modules.

#![deny(clippy::all)]
#![deny(unused)]

mod surface;
mod transform;
mod type_usage;

pub use transform::transform;
pub(crate) use type_usage::{SymbolUsage, UsedSymbols};
