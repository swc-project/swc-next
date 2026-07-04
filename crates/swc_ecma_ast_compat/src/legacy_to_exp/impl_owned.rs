use swc_core::ecma::ast::{self as legacy};
use swc_experimental_allocator::vec::Vec as ArenaVec;
use swc_experimental_ecma_ast::{self as experimental};
use swc_experimental_ecma_semantic::resolver::Semantic;

use crate::legacy_to_exp::convert::AstConvert;

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

    fn compat_vec<T, U, F: Fn(&mut Self, T) -> U>(
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
