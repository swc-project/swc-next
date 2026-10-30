//! From https://github.com/oxc-project/oxc/blob/65f9233148aa8bb4ea433130c040675e14255c36/crates/oxc_allocator/src/hash_set.rs
//!
//! A hash set without `Drop` that stores data in arena allocator.
//!
//! By default uses [`FxHasher`] to hash keys. The hasher can be customized via the `S` type
//! parameter (e.g. [`IdentBuildHasher`] for `Ident` keys).
//!
//! See [`HashSet`] for more details.
//!
//! [`FxHasher`]: rustc_hash::FxHasher
//! [`IdentBuildHasher`]: crate::IdentBuildHasher

// All methods which just delegate to `hashbrown::HashSet` methods marked `#[inline(always)]`
#![expect(clippy::inline_always)]

use std::{
    fmt,
    hash::{BuildHasher, Hash},
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use rustc_hash::FxBuildHasher;

// Re-export additional types from `hashbrown`
pub use hashbrown::hash_set::{
    Difference, Drain, Entry, ExtractIf, Intersection, IntoIter, Iter, SymmetricDifference, Union,
};

use crate::{Allocator, hash_map::HashMap};

type InnerHashSet<'alloc, T, S> = hashbrown::HashSet<T, S, &'alloc Allocator>;

/// A hash set without `Drop` that stores data in arena allocator.
///
/// Uses [`FxHasher`] by default. The hasher can be customized via the `S` type parameter.
///
/// Just a thin wrapper around [`hashbrown::HashSet`], which disables the `Drop` implementation.
///
/// All APIs are the same, except create a [`HashSet`] with
/// either [`new_in`](HashSet::new_in) or [`with_capacity_in`](HashSet::with_capacity_in).
///
/// # Thread safety
///
/// This collection is neither `Send` nor `Sync`, even when its elements are.
/// Removing the previous `Sync` implementation is an intentional compatibility
/// correction: `Deref` exposes shared methods that allocate in the non-thread-safe
/// arena. `Deref` and `DerefMut` remain available on the same thread.
///
/// A shared reference cannot cross a thread boundary to access the allocator:
///
/// ```compile_fail,E0277
/// use std::ops::Deref;
/// use swc_next_allocator::{Allocator, hash_set::HashSet};
///
/// let allocator = Allocator::new();
/// let collection = HashSet::<u32>::new_in(&allocator);
/// let shared = &collection;
/// std::thread::scope(|scope| {
///     scope.spawn(move || { shared.deref().allocator().alloc(1_u64); });
/// });
/// ```
///
/// Explicitly cloning the underlying collection cannot bypass this restriction:
///
/// ```compile_fail,E0277
/// use std::ops::Deref;
/// use swc_next_allocator::{Allocator, hash_set::HashSet};
///
/// let allocator = Allocator::new();
/// let mut collection = HashSet::<u32>::new_in(&allocator);
/// collection.insert(1);
/// let shared = &collection;
/// std::thread::scope(|scope| {
///     scope.spawn(move || { let _ = shared.deref().clone(); });
/// });
/// ```
///
/// # No `Drop`s
///
/// Objects allocated into Oxc memory arenas are never [`Dropped`](Drop). Memory is released in bulk
/// when the allocator is dropped, without dropping the individual objects in the arena.
///
/// Therefore, it would produce a memory leak if you allocated [`Drop`] types into the arena
/// which own memory allocations outside the arena.
///
/// Static checks make this impossible to do. [`HashSet::new_in`] and all other methods which create
/// a [`HashSet`] will refuse to compile if the key is a [`Drop`] type.
///
/// [`FxHasher`]: rustc_hash::FxHasher
pub struct HashSet<'alloc, T, S = FxBuildHasher>(ManuallyDrop<InnerHashSet<'alloc, T, S>>);

impl<T: fmt::Debug, S> fmt::Debug for HashSet<'_, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.0.iter()).finish()
    }
}

// TODO: `IntoIter` and other consuming iterators provided by `hashbrown` are `Drop`.
// Wrap them in `ManuallyDrop` to prevent that.

impl<'alloc, T, S> HashSet<'alloc, T, S> {
    /// Const assertion that `T` is not `Drop`.
    /// Must be referenced in all methods which create a `HashSet`.
    const ASSERT_T_IS_NOT_DROP: () = {
        assert!(
            !std::mem::needs_drop::<T>(),
            "Cannot create a HashSet<T> where T is a Drop type"
        );
    };

    /// Creates an empty [`HashSet`] with the given hasher. It will be allocated with the given allocator.
    ///
    /// The hash set is initially created with a capacity of 0, so it will not allocate
    /// until it is first inserted into.
    #[inline(always)]
    pub fn with_hasher_in(hasher: S, allocator: &'alloc Allocator) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let inner = InnerHashSet::with_hasher_in(hasher, allocator);
        Self(ManuallyDrop::new(inner))
    }

    /// Creates an empty [`HashSet`] with the specified capacity and hasher.
    /// It will be allocated with the given allocator.
    ///
    /// The hash set will be able to hold at least capacity elements without reallocating.
    /// If capacity is 0, the hash set will not allocate.
    #[inline(always)]
    pub fn with_capacity_and_hasher_in(
        capacity: usize,
        hasher: S,
        allocator: &'alloc Allocator,
    ) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let inner = InnerHashSet::with_capacity_and_hasher_in(capacity, hasher, allocator);
        Self(ManuallyDrop::new(inner))
    }

    /// Calling this method produces a compile-time panic.
    ///
    /// This legacy guard shadows `hashbrown::HashSet::allocator`.
    /// Explicit `Deref` access remains available on the same thread.
    /// Thread safety is enforced by the collection's auto traits.
    #[allow(unfulfilled_lint_expectations)]
    #[expect(clippy::unused_self)]
    pub fn allocator(&self) -> &'alloc Allocator {
        const { panic!("This method cannot be called") };
        unreachable!();
    }
}

/// Methods that use the default [`FxBuildHasher`].
impl<'alloc, T> HashSet<'alloc, T> {
    /// Creates an empty [`HashSet`]. It will be allocated with the given allocator.
    ///
    /// The hash set is initially created with a capacity of 0, so it will not allocate
    /// until it is first inserted into.
    #[inline(always)]
    pub fn new_in(allocator: &'alloc Allocator) -> Self {
        Self::with_hasher_in(FxBuildHasher, allocator)
    }

    /// Creates an empty [`HashSet`] with the specified capacity. It will be allocated with the given allocator.
    ///
    /// The hash set will be able to hold at least capacity elements without reallocating.
    /// If capacity is 0, the hash set will not allocate.
    #[inline(always)]
    pub fn with_capacity_in(capacity: usize, allocator: &'alloc Allocator) -> Self {
        Self::with_capacity_and_hasher_in(capacity, FxBuildHasher, allocator)
    }

    /// Create a new [`HashSet`] whose elements are taken from an iterator and allocated in the given `allocator`.
    ///
    /// This is behaviorially identical to [`FromIterator::from_iter`].
    #[inline]
    pub fn from_iter_in<I: IntoIterator<Item = T>>(iter: I, allocator: &'alloc Allocator) -> Self
    where
        T: Eq + Hash,
    {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let iter = iter.into_iter();

        // Use the iterator's lower size bound.
        // This follows `hashbrown::HashSet`'s `from_iter` implementation.
        //
        // This is a trade-off:
        // * Negative: If lower bound is too low, the `HashSet` may have to grow and reallocate during `for_each` loop.
        // * Positive: Avoids potential large over-allocation for iterators where upper bound may be a large over-estimate
        //   e.g. filter iterators.
        let capacity = iter.size_hint().0;
        let set = InnerHashSet::with_capacity_and_hasher_in(capacity, FxBuildHasher, allocator);
        // Wrap in `ManuallyDrop` *before* calling `for_each`, so compiler doesn't insert unnecessary code
        // to drop the `FxHashSet` in case of a panic in iterator's `next` method
        let mut set = ManuallyDrop::new(set);

        iter.for_each(|v| {
            set.insert(v);
        });

        Self(set)
    }
}

// Provide access to all `hashbrown::HashSet`'s methods via deref
impl<'alloc, T, S> Deref for HashSet<'alloc, T, S> {
    type Target = InnerHashSet<'alloc, T, S>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'alloc, T, S> DerefMut for HashSet<'alloc, T, S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut InnerHashSet<'alloc, T, S> {
        &mut self.0
    }
}

impl<'alloc, T, S> IntoIterator for HashSet<'alloc, T, S> {
    type IntoIter = IntoIter<T, &'alloc Allocator>;
    type Item = T;

    /// Creates a consuming iterator, that is, one that moves each value out of the set
    /// in arbitrary order.
    ///
    /// The set cannot be used after calling this.
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        let inner = ManuallyDrop::into_inner(self.0);
        // TODO: `hashbrown::hash_set::IntoIter` is `Drop`.
        // Wrap it in `ManuallyDrop` to prevent that.
        inner.into_iter()
    }
}

impl<'alloc, 'i, T, S> IntoIterator for &'i HashSet<'alloc, T, S> {
    type IntoIter = <&'i InnerHashSet<'alloc, T, S> as IntoIterator>::IntoIter;
    type Item = &'i T;

    /// Creates an iterator over the values of a `HashSet` in arbitrary order.
    ///
    /// The iterator element type is `&'a T`.
    ///
    /// Return the same [`Iter`] struct as by the `iter` method on [`HashSet`].
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T, S> PartialEq for HashSet<'_, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T, S> Eq for HashSet<'_, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
}

// Note: `Index` and `Extend` are implemented via `Deref`

/// Convert `HashMap<T, ()>` to `HashSet<T>`.
///
/// This conversion is zero cost, as `HashSet<T>` is just a wrapper around `HashMap<T, ()>`.
impl<'alloc, T, S> From<HashMap<'alloc, T, (), S>> for HashSet<'alloc, T, S> {
    #[inline(always)]
    fn from(map: HashMap<'alloc, T, (), S>) -> Self {
        let inner_map = ManuallyDrop::into_inner(map.0);
        let inner_set = hashbrown::HashSet::from(inner_map);
        Self(ManuallyDrop::new(inner_set))
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::HashSet;
    use crate::Allocator;

    #[test]
    fn shared_access() {
        let allocator = Allocator::new();
        let mut set = HashSet::new_in(&allocator);
        set.insert(1_u32);

        let shared = &set;
        assert!(shared.contains(&1));
        assert_eq!(shared.iter().next(), Some(&1));

        let cloned = shared.deref().clone();
        assert!(cloned.contains(&1));
        assert_eq!(*shared.deref().allocator().alloc(3_u64), 3);
    }
}
