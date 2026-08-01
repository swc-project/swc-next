//! Scope, symbol, reference, module-record, and early-error analysis for the flat ECMAScript AST.
//!
//! This crate is a Rust port of the semantic-analysis design from
//! [yuku-toolchain/yuku](https://github.com/yuku-toolchain/yuku).

#![deny(clippy::all)]
#![deny(unused)]

use swc_next_ecma_ast::{Ast, Diagnostic, Program};

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

/// Semantic analysis output and its scope-dependent early errors.
pub struct SemanticReturn<'a> {
    pub semantic: Semantic<'a>,
    pub diagnostics: Vec<Diagnostic<'a>>,
}

/// Runs semantic analysis on a tree. Builds the complete `Semantic`
/// model and reports the scope-dependent early errors.
///
/// Diagnostics are returned separately from parser diagnostics. Their
/// messages use the tree's arena, so the result is valid as long as the
/// tree is alive.
pub fn analyze<'a>(root: Program, ast: &'a mut Ast<'a>) -> SemanticReturn<'a> {
    let mut visitor = Checker::new(ast.allocator());
    let sem = SemanticBuilder::build(root, ast, &mut visitor);
    visitor.check_unresolved_exports(ast, &sem);
    SemanticReturn {
        semantic: sem,
        diagnostics: visitor.into_diagnostics(),
    }
}
