//! TypeScript isolated declaration generation for the flat swc-next AST.
//!
//! [`transform()`] consumes a [`SemanticAst`](swc_next_ecma_semantic2::SemanticAst)
//! binding and rewrites its AST in place. Re-run analysis after transforming the tree.

#![deny(clippy::all)]
#![deny(unused)]

mod surface;
mod transform;
mod type_usage;

use swc_next_ecma_ast::{ReferenceId, SymbolId};
use swc_next_ecma_semantic2::{Semantic, SymbolFlags};

pub use transform::transform;
pub(crate) use type_usage::{SymbolUsage, UsedSymbols};

fn is_unresolved(semantic: &Semantic<'_>, reference: ReferenceId) -> bool {
    semantic.reference(reference).symbol.is_none()
        && semantic.reference_candidates(reference).is_empty()
}

fn value_binding(semantic: &Semantic<'_>, symbol: SymbolId) -> SymbolId {
    let flags = semantic.symbol(symbol).flags;
    if flags.contains(SymbolFlags::EXPORT_VALUE) {
        return semantic.ts_export_symbol(symbol).unwrap();
    }
    symbol
}
