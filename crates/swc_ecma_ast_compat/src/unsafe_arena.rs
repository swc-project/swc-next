use std::mem::ManuallyDrop;

use bumpalo::Bump;
use swc_core::ecma::ast::{self as legacy};
use swc_experimental_allocator::vec::Vec as ArenaVec;
use swc_experimental_ecma_ast::{self as experimental};
use swc_experimental_ecma_semantic::resolver::Semantic;

use crate::compat_impl::CompatImpl;

/// Owns a bump allocator together with a legacy AST whose `Box` / `Vec`
/// storage has been unsafely rebound to allocations inside that arena.
///
/// The wrapper never drops the inner AST value. Instead, it only drops the bump
/// allocator, which releases all AST storage at once. Immutable access is
/// exposed through callback helpers to keep the safe API scoped to the owner.
#[must_use]
pub struct UnsafeConverted<T> {
    allocator: Bump,
    inner: ManuallyDrop<T>,
}

impl<T> UnsafeConverted<T> {
    fn new(allocator: Bump, value: T) -> Self {
        Self {
            allocator,
            inner: ManuallyDrop::new(value),
        }
    }

    #[inline]
    fn inner_ref(&self) -> &T {
        // Safety: `inner` was initialized in `new` and is never dropped while
        // `self` is alive. Casting away `ManuallyDrop` is therefore valid for
        // shared access.
        unsafe { &*(&self.inner as *const ManuallyDrop<T> as *const T) }
    }

    #[inline]
    unsafe fn inner_mut_unchecked(&mut self) -> &mut T {
        // Safety: callers of this method uphold the extra invariants required
        // by the bump-backed fake `Box` / `Vec` fields stored inside `T`.
        unsafe { &mut *(&mut self.inner as *mut ManuallyDrop<T> as *mut T) }
    }

    /// Executes a callback with shared access to the converted AST.
    pub fn with_ref<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(self.inner_ref())
    }

    /// Returns the bump allocator that owns all storage for the converted AST.
    pub fn allocator(&self) -> &Bump {
        &self.allocator
    }

    /// Executes a callback with mutable access to the converted AST.
    ///
    /// # Safety
    ///
    /// The converted AST contains `Box` / `Vec` values whose storage actually
    /// lives inside `self.allocator`. Callers must ensure the callback only
    /// performs in-place mutations that do not move out of, replace, or
    /// otherwise drop any bump-backed field. In particular, operations that
    /// would normally free existing `Box` / `Vec` allocations are unsound here.
    pub unsafe fn with_mut<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R {
        // Safety: the caller upholds the mutation invariants documented above.
        f(unsafe { self.inner_mut_unchecked() })
    }
}

/// Converts the experimental AST into legacy SWC AST nodes allocated inside a
/// bump arena owned by the returned wrapper.
pub struct UnsafeArenaAstCompat<'ast> {
    semantic: &'ast Semantic,
}

struct UnsafeArenaCompatSession<'ast, 'alloc> {
    semantic: &'ast Semantic,
    allocator: &'alloc Bump,
}

impl<'ast> UnsafeArenaAstCompat<'ast> {
    pub fn new(semantic: &'ast Semantic) -> Self {
        Self { semantic }
    }

    pub fn compat_program(&self, root: experimental::Program) -> UnsafeConverted<legacy::Program> {
        let allocator = Bump::new();
        let value = UnsafeArenaCompatSession {
            semantic: self.semantic,
            allocator: &allocator,
        }
        .compat_program_inner(root);
        UnsafeConverted::new(allocator, value)
    }

    pub fn compat_module(&self, module: experimental::Module) -> UnsafeConverted<legacy::Module> {
        let allocator = Bump::new();
        let value = UnsafeArenaCompatSession {
            semantic: self.semantic,
            allocator: &allocator,
        }
        .compat_module_inner(module);
        UnsafeConverted::new(allocator, value)
    }

    pub fn compat_script(&self, script: experimental::Script) -> UnsafeConverted<legacy::Script> {
        let allocator = Bump::new();
        let value = UnsafeArenaCompatSession {
            semantic: self.semantic,
            allocator: &allocator,
        }
        .compat_script_inner(script);
        UnsafeConverted::new(allocator, value)
    }
}

impl CompatImpl for UnsafeArenaCompatSession<'_, '_> {
    fn semantic(&self) -> &Semantic {
        self.semantic
    }

    fn alloc_box<T>(&self, value: T) -> Box<T> {
        let ptr = self.allocator.alloc(value) as *mut T;

        // Safety: `ptr` points to initialized memory inside the bump arena.
        // The produced `Box<T>` is never allowed to run its normal destructor;
        // it only exists to satisfy the legacy AST layout while the owning
        // `UnsafeConverted` keeps the backing arena alive.
        unsafe { Box::from_raw(ptr) }
    }

    fn compat_vec<T, U, F: Fn(&mut Self, T) -> U>(
        &mut self,
        items: ArenaVec<'_, T>,
        transformer: F,
    ) -> Vec<U> {
        let len = items.len();
        let mut iter = items.into_iter();
        let allocator = self.allocator;
        let slice = allocator.alloc_slice_fill_with(len, |_| {
            let item = iter
                .next()
                .expect("vec length should match iterator length");
            transformer(self, item)
        });

        // Safety: `slice` is fully initialized and lives inside the same bump
        // arena as the rest of the converted AST. As with `alloc_box`, the
        // resulting `Vec<U>` must never be normally dropped.
        unsafe { Vec::from_raw_parts(slice.as_mut_ptr(), len, len) }
    }
}
