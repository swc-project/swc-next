//! TypeScript isolated declaration generation for the flat swc-next AST.
//!
//! [`transform()`] consumes a [`SemanticAst`](swc_next_ecma_semantic::SemanticAst)
//! binding and rewrites its AST in place. Re-run analysis after transforming the tree.

#![deny(clippy::all)]
#![deny(unused)]

mod surface;
mod transform;
mod type_usage;

pub use transform::transform;
pub(crate) use type_usage::{SymbolUsage, UsedSymbols};
