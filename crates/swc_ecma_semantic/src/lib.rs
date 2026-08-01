//! Scope, symbol, reference, module-record, and early-error analysis for the flat ECMAScript AST.
//!
//! This crate is a Rust port of the semantic-analysis design from
//! [yuku-toolchain/yuku](https://github.com/yuku-toolchain/yuku).

#![deny(clippy::all)]
#![deny(unused)]

use swc_next_ecma_ast::{Ast, Diagnostic, Program, TraverseCtx, TraverseWith};

use crate::{checker::Checker, context::SemanticContext};

mod binder;
mod checker;
mod context;
mod ecmascript;
pub mod scope;

pub use binder::{Reference, ReferenceFlags, ReferenceSpace, Semantic, Symbol, SymbolFlags};
pub use swc_next_ecma_ast::ReferenceId;

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
pub fn analyze<'a>(root: Program, ast: &mut Ast<'a>) -> SemanticReturn<'a> {
    let mut visitor = Checker::new(ast.allocator());
    let state = SemanticContext::init(ast);
    let mut context = TraverseCtx::new(ast, state);
    let _ = root.traverse_with(&mut visitor, &mut context);
    let sem = context.state.finalize(context.ast);

    visitor.check_unresolved_exports(ast, &sem);
    SemanticReturn {
        semantic: sem,
        diagnostics: visitor.into_diagnostics(),
    }
}
