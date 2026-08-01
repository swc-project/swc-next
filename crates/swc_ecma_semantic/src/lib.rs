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
mod module_record;
mod scope;

/// Complete semantic information for one parsed source file.
///
/// The model contains every scope, symbol, and reference, fully resolved and
/// cross-indexed. It is a snapshot of the AST at analysis time; run
/// [`analyze`] again after mutating bindings or scopes.
#[derive(Debug)]
pub struct Semantic<'a> {}

/// Runs semantic analysis on a tree. Builds the complete `Semantic`
/// model and reports the scope-dependent early errors.
///
/// Diagnostics are appended directly to the tree alongside parse
/// errors. All allocations use the tree's arena, so the returned
/// model is valid as long as the tree is alive.
pub fn analyze<'a>(root: Program, ast: &'a mut Ast<'a>) -> Semantic<'a> {
    let mut visitor = Checker::new(ast);
    let sem = SemanticBuilder::build(root, &mut visitor);
    visitor.check_unresolved_exports(&sem);
    sem
}
