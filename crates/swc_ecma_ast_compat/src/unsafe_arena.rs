use std::{
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use bumpalo::Bump;
use swc_core::ecma::ast::{self as legacy};
use swc_experimental_ecma_ast::{self as experimental, Ast, ExtraDataCompact, TypedSubRange};
use swc_experimental_ecma_semantic::resolver::Semantic;

use crate::compat_impl::CompatImpl;

/// A compatibility AST value whose `Box` / `Vec` storage is backed by a bump
/// allocator and therefore must never be dropped normally.
#[must_use]
pub struct UnsafeConverted<'alloc, T> {
    inner: ManuallyDrop<T>,
    _marker: PhantomData<&'alloc Bump>,
}

impl<'alloc, T> UnsafeConverted<'alloc, T> {
    fn new(value: T) -> Self {
        Self {
            inner: ManuallyDrop::new(value),
            _marker: PhantomData,
        }
    }

    pub fn as_ref(&self) -> &T {
        self
    }

    pub fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<T> Deref for UnsafeConverted<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(&self.inner as *const ManuallyDrop<T> as *const T) }
    }
}

impl<T> DerefMut for UnsafeConverted<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(&mut self.inner as *mut ManuallyDrop<T> as *mut T) }
    }
}

/// Converts the experimental AST into legacy SWC AST nodes allocated inside a
/// bump arena.
///
/// The produced legacy AST intentionally erases the allocator lifetime from all
/// nested `Box` / `Vec` fields via `unsafe`. As a consequence, the returned
/// value is wrapped in [UnsafeConverted] so that it is never dropped normally.
/// Users must keep the backing bump allocator alive for the entire time the
/// converted AST is accessed and must treat the AST as a whole value.
pub struct UnsafeArenaAstCompat<'ast, 'alloc> {
    ast: &'ast Ast,
    semantic: &'ast Semantic,
    allocator: &'alloc Bump,
}

impl<'ast, 'alloc> UnsafeArenaAstCompat<'ast, 'alloc> {
    pub fn new(ast: &'ast Ast, semantic: &'ast Semantic, allocator: &'alloc Bump) -> Self {
        Self {
            ast,
            semantic,
            allocator,
        }
    }
}

impl<'alloc> CompatImpl for UnsafeArenaAstCompat<'_, 'alloc> {
    fn ast(&self) -> &Ast {
        self.ast
    }

    fn semantic(&self) -> &Semantic {
        self.semantic
    }

    fn alloc_box<T>(&self, value: T) -> Box<T> {
        let ptr = self.allocator.alloc(value) as *mut T;

        // Safety: `ptr` points to initialized memory owned by the bump
        // allocator. The resulting `Box<T>` is never allowed to run its normal
        // destructor because the root AST is wrapped by `UnsafeConverted`.
        unsafe { Box::from_raw(ptr) }
    }

    fn compat_type_sub_range<T: ExtraDataCompact, U, F: Fn(&mut Self, T) -> U>(
        &mut self,
        typed_range: TypedSubRange<T>,
        transformer: F,
    ) -> Vec<U> {
        let len = typed_range.len();
        let mut iter = typed_range.iter();
        let allocator = self.allocator;
        let slice = allocator.alloc_slice_fill_with(len, |_| {
            let item = iter
                .next()
                .expect("typed sub range length should match iterator length");
            transformer(self, self.ast.get_node_in_sub_range(item))
        });

        // Safety: `slice` lives inside the bump allocator and the returned
        // `Vec<U>` is never dropped normally for the same reason as `alloc_box`.
        unsafe { Vec::from_raw_parts(slice.as_mut_ptr(), len, len) }
    }
}

impl<'alloc> UnsafeArenaAstCompat<'_, 'alloc> {
    pub fn compat_program(
        &mut self,
        root: experimental::Program,
    ) -> UnsafeConverted<'alloc, legacy::Program> {
        UnsafeConverted::new(self.compat_program_inner(root))
    }

    pub fn compat_module(
        &mut self,
        module: experimental::Module,
    ) -> UnsafeConverted<'alloc, legacy::Module> {
        UnsafeConverted::new(self.compat_module_inner(module))
    }

    pub fn compat_script(
        &mut self,
        script: experimental::Script,
    ) -> UnsafeConverted<'alloc, legacy::Script> {
        UnsafeConverted::new(self.compat_script_inner(script))
    }
}
