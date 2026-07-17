//! This module contains utilities for running conformance tests
//!  between the legacy `swc_core` and the new `swc_experimental_`.

pub mod codegen;
pub mod parser;
pub mod remove_paren;
pub mod semantic;

use swc_core::ecma::ast as legacy_ast;
use swc_experimental_ecma_ast as experimental_ast;
use swc_experimental_ecma_ast_compat::AstCompat;
use swc_experimental_ecma_semantic::resolver::resolver;

pub fn convert_experimental_program(
    program: experimental_ast::Program,
    ast: &mut experimental_ast::Ast<'_>,
) -> legacy_ast::Program {
    let semantic = resolver(program, ast);
    AstCompat::new(ast, &semantic).compat_program(program)
}
