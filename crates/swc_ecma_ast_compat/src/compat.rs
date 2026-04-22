use swc_core::ecma::ast::{self as legacy};
use swc_experimental_ecma_ast::{self as experimental, Ast, ExtraDataCompact, TypedSubRange};
use swc_experimental_ecma_semantic::resolver::Semantic;

use crate::compat_impl::CompatImpl;

pub struct AstCompat<'a> {
    ast: &'a Ast,
    semantic: &'a Semantic,
}

impl<'a> AstCompat<'a> {
    pub fn new(ast: &'a Ast, semantic: &'a Semantic) -> Self {
        Self { ast, semantic }
    }
}

impl CompatImpl for AstCompat<'_> {
    fn ast(&self) -> &Ast {
        self.ast
    }

    fn semantic(&self) -> &Semantic {
        self.semantic
    }

    fn alloc_box<T>(&self, value: T) -> Box<T> {
        Box::new(value)
    }

    fn compat_type_sub_range<T: ExtraDataCompact, U, F: Fn(&mut Self, T) -> U>(
        &mut self,
        typed_range: TypedSubRange<T>,
        transformer: F,
    ) -> Vec<U> {
        let mut ret = Vec::with_capacity(typed_range.len());
        for item in typed_range.iter() {
            ret.push(transformer(self, self.ast.get_node_in_sub_range(item)));
        }
        ret
    }
}

impl AstCompat<'_> {
    pub fn compat_program(&mut self, root: experimental::Program) -> legacy::Program {
        self.compat_program_inner(root)
    }

    pub fn compat_module(&mut self, module: experimental::Module) -> legacy::Module {
        self.compat_module_inner(module)
    }

    pub fn compat_script(&mut self, script: experimental::Script) -> legacy::Script {
        self.compat_script_inner(script)
    }
}
