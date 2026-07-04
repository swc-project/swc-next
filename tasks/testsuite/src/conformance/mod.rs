//! This module contains utilities for running conformance tests
//!  between the legacy `swc_core` and the new `swc_experimental_`.

pub mod codegen;
pub mod parser;
pub mod remove_paren;
pub mod semantic;

use swc_core::ecma::ast as legacy_ast;
use swc_experimental_ecma_ast as experimental_ast;
use swc_experimental_ecma_ast_compat::OwnedConvert;
use swc_experimental_ecma_semantic::resolver::resolver;

pub fn convert_experimental_program<'a>(
    program: experimental_ast::Program<'a>,
) -> legacy_ast::Program {
    let semantic = resolver(&program);
    OwnedConvert::new(&semantic).convert_program(program)
}
