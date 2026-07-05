use swc_core::ecma::ast::{self as legacy};
use swc_experimental_allocator::vec::Vec as ArenaVec;
use swc_experimental_ecma_ast::{self as experimental};
use swc_experimental_ecma_semantic::resolver::Semantic;

use crate::exp_to_legacy::convert::AstConvert;

pub struct OwnedConvert<'a> {
    semantic: &'a Semantic,
}

impl<'a> OwnedConvert<'a> {
    pub fn new(semantic: &'a Semantic) -> Self {
        Self { semantic }
    }
}

impl AstConvert for OwnedConvert<'_> {
    fn semantic(&self) -> &Semantic {
        self.semantic
    }

    fn alloc_box<T>(&self, value: T) -> Box<T> {
        Box::new(value)
    }

    fn convert_vec<T, U, F: Fn(&mut Self, T) -> U>(
        &mut self,
        items: ArenaVec<'_, T>,
        transformer: F,
    ) -> Vec<U> {
        let mut ret = Vec::with_capacity(items.len());
        for item in items {
            ret.push(transformer(self, item));
        }
        ret
    }
}

impl OwnedConvert<'_> {
    pub fn convert_program(&mut self, root: experimental::Program) -> legacy::Program {
        self.convert_program_inner(root)
    }

    pub fn convert_module(&mut self, module: experimental::Module) -> legacy::Module {
        self.convert_module_inner(module)
    }

    pub fn convert_script(&mut self, script: experimental::Script) -> legacy::Script {
        self.convert_script_inner(script)
    }
}
