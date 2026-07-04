//! Experimental ECMAScript code generation support.
//!
//! This crate is currently a thin adapter over `swc_core` code generation for
//! the experimental AST and semantic model. It exists to support internal test
//! coverage while the surrounding experimental pipeline is being built out.
//!
//! The public API is intentionally small and should be treated as unstable. At
//! the moment, this crate is not intended for external production use.

use swc_experimental_ecma_ast::Program;
use swc_experimental_ecma_semantic::resolver::Semantic;

pub struct Codegen {
    #[allow(unused)]
    options: CodegenOptions,
}

#[derive(Debug, Clone, Default)]
pub struct CodegenOptions {}

impl Codegen {
    pub fn new(options: CodegenOptions) -> Self {
        Self { options }
    }

    pub fn build(self, program: Program, semantic: &Semantic) -> String {
        let convert = swc_experimental_ecma_ast_compat::ArenaConvert::new(semantic);
        convert
            .convert_program(program)
            .with_ref(swc_core::ecma::codegen::to_code)
    }
}
