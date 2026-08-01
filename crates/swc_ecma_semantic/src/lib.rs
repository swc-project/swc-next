//! Scope, symbol, reference, module-record, and early-error analysis for the flat ECMAScript AST.
//!
//! This crate is a Rust port of the semantic-analysis design from
//! [yuku-toolchain/yuku](https://github.com/yuku-toolchain/yuku).

#![deny(clippy::all)]
#![deny(unused)]

use swc_next_ecma_ast::{Ast, Program};

use crate::{builder::SemanticBuilder, checker::Checker};

mod binder;
mod builder;
mod checker;
mod ecmascript;
pub mod module_record;
pub mod scope;

pub use binder::{
    Reference, ReferenceFlags, ReferenceId, ReferenceSpace, Semantic, Symbol, SymbolFlags,
};

/// Runs semantic analysis on a tree. Builds the complete `Semantic`
/// model and reports the scope-dependent early errors.
///
/// Diagnostics are appended directly to the tree alongside parse
/// errors. All allocations use the tree's arena, so the returned
/// model is valid as long as the tree is alive.
pub fn analyze<'a>(root: Program, ast: &mut Ast<'a>) -> Semantic<'a> {
    let mut visitor = Checker::new(ast);
    let sem = SemanticBuilder::build(root, &mut visitor);
    visitor.check_unresolved_exports(&sem);
    sem
}
