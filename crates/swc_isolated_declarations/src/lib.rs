//! TypeScript isolated declaration generation for the flat swc-next AST.
//!
//! [`transform()`] uses a caller-provided semantic model to rewrite an
//! [`AstCtx`](swc_next_ecma_ast::AstCtx) in place. The caller must ensure that
//! the model describes the current AST. Re-run analysis after transforming the tree.

#![deny(clippy::all)]
#![deny(unused)]

mod surface;
mod transform;
mod type_usage;

pub use transform::transform;
pub(crate) use type_usage::{SymbolUsage, UsedSymbols};
