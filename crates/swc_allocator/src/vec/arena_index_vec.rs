//! An arena-backed vector with type-safe indices from [`oxc_index`].

use std::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    slice,
};

use crate::{Allocator, CloneIn, Idx, IndexSlice, vec::ArenaVec};

/// A growable arena-backed vector indexed by `I`.
///
/// Like [`oxc_index::IndexVec`], this dereferences to [`IndexSlice`] and
/// [`push`](Self::push) returns the inserted element's index. Storage comes from
/// [`Allocator`] and is reclaimed with the arena. Elements must not require
/// [`Drop`], as enforced by the underlying arena [`ArenaVec`].
///
/// Operations that grow the vector panic before its length exceeds `I::MAX + 1`.
/// Removing or reordering elements can invalidate previously returned indices.
///
/// ```
/// use swc_next_allocator::{Allocator, vec::ArenaIndexVec};
///
/// swc_next_allocator::define_index_type! {
///     struct NodeId = u32;
///     MAX_INDEX = u32::MAX as usize;
/// }
///
/// let allocator = Allocator::new();
/// let mut nodes = ArenaIndexVec::<NodeId, _>::new_in(&allocator);
/// let id = nodes.push(42);
/// assert_eq!(nodes[id], 42);
/// assert_eq!(nodes.iter_enumerated().next(), Some((id, &42)));
/// ```
#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArenaIndexVec<'alloc, I: Idx, T> {
    raw: ArenaVec<'alloc, T>,
    marker: PhantomData<fn(&I)>,
}

impl<'alloc, I: Idx, T> ArenaIndexVec<'alloc, I, T> {
    /// Creates an empty vector without allocating.
    #[inline]
    pub fn new_in(allocator: &'alloc Allocator) -> Self {
        Self {
            raw: ArenaVec::new_in(allocator),
            marker: PhantomData,
        }
    }

    /// Creates an empty vector with space for at least `capacity` elements.
    ///
    /// Panics if the requested capacity exceeds the index type's range.
    #[inline]
    pub fn with_capacity_in(capacity: usize, allocator: &'alloc Allocator) -> Self {
        Self::check_len(capacity);
        Self {
            raw: ArenaVec::with_capacity_in(capacity, allocator),
            marker: PhantomData,
        }
    }

    /// Collects values into a vector allocated in `allocator`.
    #[inline]
    pub fn from_iter_in(iter: impl IntoIterator<Item = T>, allocator: &'alloc Allocator) -> Self {
        let mut result = Self::new_in(allocator);
        result.extend(iter);
        result
    }

    /// Wraps an arena vector without allocating or copying its elements.
    ///
    /// Panics if any element's index cannot be represented by `I`.
    #[inline]
    pub fn from_vec(raw: ArenaVec<'alloc, T>) -> Self {
        Self::check_len(raw.len());
        Self {
            raw,
            marker: PhantomData,
        }
    }

    /// Returns the underlying arena vector without allocating or copying.
    #[inline]
    pub fn into_vec(self) -> ArenaVec<'alloc, T> {
        self.raw
    }

    /// Returns the allocator backing this vector.
    #[inline]
    pub fn bump(&self) -> &'alloc Allocator {
        self.raw.bump()
    }

    /// Returns the number of elements in the vector.
    #[inline]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Returns whether the vector is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Iterates over all initialized indices.
    #[inline]
    pub fn indices(&self) -> std::iter::Map<std::ops::Range<u32>, fn(u32) -> I> {
        // Match the storage's counter width so index conversions do not
        // obscure the loop's bounds when its indices are used for access.
        (0..self.raw.len_u32()).map(|index| I::from_usize(index as usize))
    }

    /// Sets the initialized length without changing the allocation.
    ///
    /// # Safety
    /// `new_len` must not exceed capacity, and every element in `0..new_len`
    /// must be initialized.
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        Self::check_len(new_len);
        unsafe { self.raw.set_len(new_len) };
    }

    #[inline]
    fn check_len(len: usize) {
        assert!(
            len == 0 || len - 1 <= I::MAX,
            "length exceeds index type range"
        );
    }

    #[inline]
    fn check_additional(&self, additional: usize) {
        // ArenaVec already rejects lengths above u32::MAX, including for ZSTs.
        // Only narrower index types need an additional growth check.
        if I::MAX < u32::MAX as usize - 1 {
            Self::check_len(
                self.len()
                    .checked_add(additional)
                    .expect("capacity overflow"),
            );
        }
    }

    /// Returns the next index that [`push`](Self::push) will assign.
    ///
    /// Panics when no more indices are representable by `I`.
    #[inline]
    pub fn next_idx(&self) -> I {
        assert!(self.len() <= I::MAX, "length exceeds index type range");
        // SAFETY: The index was checked above.
        unsafe { I::from_usize_unchecked(self.len()) }
    }

    /// Appends a value and returns its index.
    ///
    /// Panics before modifying the vector if its next index exceeds `I::MAX`.
    #[inline]
    pub fn push(&mut self, value: T) -> I {
        let index = self.len();
        self.check_additional(1);
        self.raw.push(value);
        // SAFETY: Narrow indices were checked before the push. Otherwise,
        // ArenaVec's u32 length limit guarantees index < u32::MAX, so index <= I::MAX.
        unsafe { I::from_usize_unchecked(index) }
    }

    /// Removes and returns the last value, if any.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.raw.pop()
    }

    /// Returns the number of elements that fit in the current allocation.
    ///
    /// This can exceed the index type's range due to allocation rounding.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }

    /// Reserves space for at least `additional` more elements.
    ///
    /// Panics if the resulting length would exceed the index type's range.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.check_additional(additional);
        self.raw.reserve(additional);
    }

    /// Keeps only the first `len` elements, leaving capacity unchanged.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.raw.truncate(len);
    }

    /// Removes all elements, leaving capacity unchanged.
    #[inline]
    pub fn clear(&mut self) {
        self.raw.clear();
    }

    /// Resizes the vector, cloning `value` to fill new elements.
    #[inline]
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        Self::check_len(new_len);
        self.raw.resize(new_len, value);
    }

    /// Appends a slice by copying its elements.
    #[inline]
    pub fn extend_from_slice_copy(&mut self, other: &[T])
    where
        T: Copy,
    {
        self.check_additional(other.len());
        self.raw.extend_from_slice_copy(other);
    }

    /// Moves all elements from `other` to the end of this vector.
    #[inline]
    pub fn append(&mut self, other: &mut Self) {
        self.check_additional(other.len());
        self.raw.append(&mut other.raw);
    }

    /// Removes and returns an element, shifting later elements to the left.
    #[inline]
    pub fn remove(&mut self, index: I) -> T {
        self.raw.remove(index.index())
    }

    /// Removes and returns an element, replacing it with the last element.
    #[inline]
    pub fn swap_remove(&mut self, index: I) -> T {
        self.raw.swap_remove(index.index())
    }

    /// Returns an element without checking its index.
    ///
    /// # Safety
    /// `index` must identify an initialized element of this vector.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: I) -> &T {
        unsafe { self.raw.get_unchecked(index.index()) }
    }

    /// Returns a mutable element without checking its index.
    ///
    /// # Safety
    /// `index` must identify an initialized element of this vector.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: I) -> &mut T {
        unsafe { self.raw.get_unchecked_mut(index.index()) }
    }

    /// Returns a slice with type-safe indices.
    #[inline]
    pub fn as_slice(&self) -> &IndexSlice<I, [T]> {
        IndexSlice::new(self.raw.as_slice())
    }

    /// Returns a mutable slice with type-safe indices.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut IndexSlice<I, [T]> {
        IndexSlice::new_mut(self.raw.as_mut_slice())
    }

    /// Returns the elements as a standard slice.
    #[inline]
    pub fn as_raw_slice(&self) -> &[T] {
        self.raw.as_slice()
    }

    /// Returns the elements as a mutable standard slice.
    #[inline]
    pub fn as_raw_slice_mut(&mut self) -> &mut [T] {
        self.raw.as_mut_slice()
    }

    /// Returns a mutable pointer to the allocation, including spare capacity.
    ///
    /// This does not create a reference to the elements. The caller must
    /// initialize spare elements before including them in [`set_len`](Self::set_len).
    /// Operations that reallocate the vector invalidate this pointer.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.raw.as_mut_ptr()
    }

    /// Consumes the vector and returns a slice valid for the arena's lifetime.
    #[inline]
    pub fn into_bump_slice(self) -> &'alloc IndexSlice<I, [T]> {
        IndexSlice::new(self.raw.into_bump_slice())
    }

    /// Consumes the vector and returns a mutable slice for the arena's lifetime.
    #[inline]
    pub fn into_bump_slice_mut(self) -> &'alloc mut IndexSlice<I, [T]> {
        IndexSlice::new_mut(self.raw.into_bump_slice_mut())
    }
}

impl<I: Idx, T> Deref for ArenaIndexVec<'_, I, T> {
    type Target = IndexSlice<I, [T]>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<I: Idx, T> DerefMut for ArenaIndexVec<'_, I, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<I: Idx, T: fmt::Debug> fmt::Debug for ArenaIndexVec<'_, I, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

impl<I: Idx, T> Extend<T> for ArenaIndexVec<'_, I, T> {
    #[inline]
    fn extend<J: IntoIterator<Item = T>>(&mut self, iter: J) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        for value in iter {
            self.push(value);
        }
    }
}

impl<'alloc, I: Idx, T> IntoIterator for ArenaIndexVec<'alloc, I, T> {
    type Item = T;
    type IntoIter = crate::vec::IntoIter<'alloc, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.raw.into_iter()
    }
}

impl<'a, I: Idx, T> IntoIterator for &'a ArenaIndexVec<'_, I, T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter()
    }
}

impl<'a, I: Idx, T> IntoIterator for &'a mut ArenaIndexVec<'_, I, T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter_mut()
    }
}

impl<'alloc, I: Idx, T, C: 'alloc> CloneIn<'alloc> for ArenaIndexVec<'_, I, T>
where
    T: CloneIn<'alloc, Cloned = C>,
{
    type Cloned = ArenaIndexVec<'alloc, I, C>;

    #[inline]
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        ArenaIndexVec::from_vec(self.raw.clone_in(allocator))
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;

    crate::define_index_type! {
        struct TestId = u32;
        MAX_INDEX = u32::MAX as usize;
    }

    crate::define_index_type! {
        struct SmallId = u8;
        MAX_INDEX = 3;
        DISABLE_MAX_INDEX_CHECK = true;
    }

    crate::define_nonmax_u32_index_type! {
        struct NonMaxId;
    }

    #[test]
    fn indexing_and_growth() {
        let allocator = Allocator::new();
        let mut data = ArenaIndexVec::<TestId, _>::with_capacity_in(1, &allocator);
        for value in 0..100 {
            let id = data.push(value);
            assert_eq!(id.index(), value as usize);
            assert_eq!(data[id], value);
        }
        assert_eq!(data.len(), 100);
        assert!(data.capacity() >= data.len());

        let id = TestId::from(1usize);
        data[id] = 200;
        assert_eq!(data.get(id), Some(&200));
        assert_eq!(data[id..TestId::from(3usize)].raw, [200, 2]);
        assert_eq!(data.iter_enumerated().nth(1), Some((id, &200)));
        assert_eq!(data.indices().next_back(), Some(data.last_idx()));
        assert!(data.get(data.next_idx()).is_none());
    }

    #[test]
    fn mutation_and_iteration() {
        let allocator = Allocator::new();
        let mut data = ArenaIndexVec::<TestId, _>::from_iter_in([1, 2], &allocator);
        data.extend([3, 4]);
        for value in &mut data {
            *value += 1;
        }
        assert_eq!(
            (&data).into_iter().copied().collect::<std::vec::Vec<_>>(),
            [2, 3, 4, 5]
        );
        assert_eq!(data.pop(), Some(5));
        assert_eq!(data.swap_remove(TestId::from(0usize)), 2);
        assert_eq!(data.remove(TestId::from(0usize)), 4);

        data.resize(3, 9);
        assert_eq!(data.as_raw_slice(), [3, 9, 9]);
        data.truncate(1);
        let mut other = ArenaIndexVec::from_iter_in([7, 8], &allocator);
        data.append(&mut other);
        assert!(other.is_empty());
        assert_eq!(data.into_iter().collect::<std::vec::Vec<_>>(), [3, 7, 8]);
    }

    #[test]
    fn drain_bounds() {
        use std::ops::Bound::{Excluded, Unbounded};

        let allocator = Allocator::new();
        let mut data = ArenaVec::from_iter_in([1, 2, 3], &allocator);
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                data.drain((Excluded(usize::MAX), Unbounded)).count()
            }))
            .is_err()
        );
        assert_eq!(data.as_slice(), [1, 2, 3]);

        assert!(catch_unwind(AssertUnwindSafe(|| data.drain(..=usize::MAX).count())).is_err());
        assert_eq!(data.as_slice(), [1, 2, 3]);

        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                data.splice((Excluded(usize::MAX), Unbounded), [9]).count()
            }))
            .is_err()
        );
        assert_eq!(data.as_slice(), [1, 2, 3]);

        assert!(
            catch_unwind(AssertUnwindSafe(|| data.splice(..=usize::MAX, [9]).count())).is_err()
        );
        assert_eq!(data.as_slice(), [1, 2, 3]);

        let mut data = ArenaVec::from_iter_in([1, 2, 3, 4], &allocator);
        assert_eq!(data.drain(1..3).collect::<std::vec::Vec<_>>(), [2, 3]);
        assert_eq!(data.as_slice(), [1, 4]);

        let mut data = ArenaVec::from_iter_in([1, 2, 3], &allocator);
        assert_eq!(data.drain(..).collect::<std::vec::Vec<_>>(), [1, 2, 3]);
        assert!(data.is_empty());

        let mut data = ArenaVec::from_iter_in([1, 2, 3], &allocator);
        assert_eq!(
            data.drain(..=data.len() - 1).collect::<std::vec::Vec<_>>(),
            [1, 2, 3]
        );
        assert!(data.is_empty());

        let mut data = ArenaVec::from_iter_in([1, 2, 3], &allocator);
        assert_eq!(
            data.drain((Excluded(0), Unbounded))
                .collect::<std::vec::Vec<_>>(),
            [2, 3]
        );
        assert_eq!(data.as_slice(), [1]);

        let mut data = ArenaVec::from_iter_in([1, 2, 3], &allocator);
        assert_eq!(
            data.splice(1..=1, [4, 5]).collect::<std::vec::Vec<_>>(),
            [2]
        );
        assert_eq!(data.as_slice(), [1, 4, 5, 3]);
    }

    fn unit_vec(allocator: &Allocator, len: u32) -> ArenaVec<'_, ()> {
        let mut data = ArenaVec::new_in(allocator);
        // SAFETY: Unit values need no storage or initialization, and len fits u32.
        unsafe { data.set_len(len as usize) };
        data
    }

    #[test]
    fn zst_length_limits() {
        let allocator = Allocator::new();
        let max_len = u32::MAX as usize;
        let mut data = unit_vec(&allocator, u32::MAX);
        assert!(catch_unwind(AssertUnwindSafe(|| data.insert(0, ()))).is_err());
        assert_eq!(data.len(), max_len);

        let mut data = unit_vec(&allocator, u32::MAX);
        assert!(catch_unwind(AssertUnwindSafe(|| data.extend_from_slice(&[()]))).is_err());
        assert_eq!(data.len(), max_len);

        let mut data = unit_vec(&allocator, u32::MAX - 1);
        assert!(catch_unwind(AssertUnwindSafe(|| data.extend_from_slice(&[(), ()]))).is_err());
        assert_eq!(data.len(), max_len - 1);

        struct CountClone;
        thread_local! {
            static CLONES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
        }
        impl Clone for CountClone {
            fn clone(&self) -> Self {
                CLONES.set(CLONES.get() + 1);
                Self
            }
        }
        let mut data = ArenaVec::<CountClone>::new_in(&allocator);
        // SAFETY: CountClone is an inhabited ZST, and this length fits u32.
        unsafe { data.set_len(max_len - 1) };
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                data.extend_from_slice(&[CountClone, CountClone]);
            }))
            .is_err()
        );
        assert_eq!(data.len(), max_len - 1);
        assert_eq!(CLONES.get(), 0);

        let mut data = unit_vec(&allocator, u32::MAX - 1);
        data.insert(0, ());
        assert_eq!(data.len(), max_len);

        let mut data = unit_vec(&allocator, u32::MAX - 1);
        data.extend_from_slice(&[()]);
        assert_eq!(data.len(), max_len);
        data.extend_from_slice(&[]);
        assert_eq!(data.len(), max_len);

        let mut data = ArenaVec::from_iter_in(["a", "c"], &allocator);
        data.insert(1, "b");
        assert_eq!(data.as_slice(), ["a", "b", "c"]);
        data.extend_from_slice(&["d", "e"]);
        assert_eq!(data.as_slice(), ["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn arena_conversions() {
        let allocator = Allocator::new();
        let mut data =
            ArenaIndexVec::<TestId, _>::from_vec(ArenaVec::from_iter_in([1, 2], &allocator));
        let ptr = data.as_raw_slice().as_ptr();
        data.as_raw_slice_mut()[0] = 3;
        let raw = data.into_vec();
        assert_eq!(raw.as_ptr(), ptr);
        let slice = ArenaIndexVec::<TestId, _>::from_vec(raw).into_bump_slice_mut();
        slice[TestId::from(1usize)] = 4;
        assert_eq!(slice.raw, [3, 4]);

        let data = ArenaIndexVec::<TestId, _>::from_iter_in([true, false], &allocator);
        assert_eq!(data.clone().as_raw_slice(), [true, false]);
        let other_allocator = Allocator::new();
        let cloned = data.clone_in(&other_allocator);
        drop(allocator);
        assert_eq!(cloned.into_bump_slice().raw, [true, false]);
    }

    #[test]
    fn raw_storage() {
        let allocator = Allocator::new();
        let mut data = ArenaIndexVec::<TestId, u32>::with_capacity_in(2, &allocator);
        assert!(std::ptr::eq(data.bump(), &allocator));
        let ptr = data.as_mut_ptr();

        // SAFETY: The allocation has room for two u32 values, both initialized
        // before exposing them through the vector's length.
        unsafe {
            ptr.write(10);
            ptr.add(1).write(20);
            data.set_len(2);
        }
        assert_eq!(data.as_raw_slice(), [10, 20]);

        // SAFETY: Both indices identify initialized elements.
        unsafe {
            assert_eq!(*data.get_unchecked(TestId::from(0usize)), 10);
            *data.get_unchecked_mut(TestId::from(1usize)) = 30;
        }
        assert_eq!(data.as_raw_slice(), [10, 30]);

        data.extend_from_slice_copy(&[40, 50]);
        assert_eq!(data.as_raw_slice(), [10, 30, 40, 50]);
    }

    #[test]
    fn index_limits() {
        let allocator = Allocator::new();
        let mut data = ArenaIndexVec::<SmallId, _>::from_iter_in([0, 1, 2, 3], &allocator);
        assert_eq!(data[SmallId::from(3usize)], 3);
        assert!(catch_unwind(AssertUnwindSafe(|| data.push(4))).is_err());
        assert_eq!(data.as_raw_slice(), [0, 1, 2, 3]);
        assert!(catch_unwind(AssertUnwindSafe(|| data.reserve(1))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| data.resize(5, 0))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| data.extend_from_slice_copy(&[4]))).is_err());
        assert_eq!(data.as_raw_slice(), [0, 1, 2, 3]);

        let mut other = ArenaIndexVec::from_iter_in([4], &allocator);
        assert!(catch_unwind(AssertUnwindSafe(|| data.append(&mut other))).is_err());
        assert_eq!(other.as_raw_slice(), [4]);
        data.clear();
        assert_eq!(data.push(5).index(), 0);

        let mut zst = ArenaIndexVec::<SmallId, ()>::new_in(&allocator);
        zst.extend([(); 4]);
        assert!(catch_unwind(AssertUnwindSafe(|| zst.push(()))).is_err());
        assert_eq!(zst.len(), 4);

        let mut full = ArenaIndexVec::<NonMaxId, ()>::new_in(&allocator);
        // SAFETY: Unit values need no storage or initialization. ArenaVec
        // supports u32::MAX initialized elements, all representable by NonMaxId.
        unsafe { full.set_len(u32::MAX as usize) };
        assert_eq!(full.indices().len(), u32::MAX as usize);
        assert_eq!(
            full.indices().next_back().unwrap().index(),
            u32::MAX as usize - 1
        );
        assert!(catch_unwind(AssertUnwindSafe(|| full.next_idx())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| full.push(()))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| full.reserve(1))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| full.extend_from_slice_copy(&[()]))).is_err());
        let mut other = ArenaIndexVec::from_iter_in([()], &allocator);
        assert!(catch_unwind(AssertUnwindSafe(|| full.append(&mut other))).is_err());
        assert_eq!(full.len(), u32::MAX as usize);
        assert_eq!(other.len(), 1);

        let mut full = ArenaIndexVec::<TestId, ()>::new_in(&allocator);
        // SAFETY: As above, all unit values are initialized and fit the index type.
        unsafe { full.set_len(u32::MAX as usize) };
        assert_eq!(full.next_idx().index(), u32::MAX as usize);
        assert!(catch_unwind(AssertUnwindSafe(|| full.push(()))).is_err());
        assert_eq!(full.len(), u32::MAX as usize);
    }

    #[test]
    fn construction_limits() {
        let allocator = Allocator::new();
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                ArenaIndexVec::<SmallId, u8>::with_capacity_in(5, &allocator)
            }))
            .is_err()
        );

        let raw = ArenaVec::from_iter_in([(); 5], &allocator);
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                ArenaIndexVec::<SmallId, _>::from_vec(raw)
            }))
            .is_err()
        );

        let mut data = ArenaIndexVec::<TestId, ()>::new_in(&allocator);
        data.push(());
        assert!(catch_unwind(AssertUnwindSafe(|| data.reserve(usize::MAX))).is_err());
    }

    #[test]
    fn zero_sized_elements() {
        let allocator = Allocator::new();
        let data = ArenaIndexVec::<TestId, _>::from_iter_in([(); 3], &allocator);
        let mut iter = data.into_iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(()));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next_back(), Some(()));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.count(), 1);

        let data = ArenaIndexVec::<TestId, _>::from_iter_in([(); 3], &allocator);
        assert_eq!(data.into_iter().collect::<std::vec::Vec<_>>(), [(); 3]);

        let data = ArenaIndexVec::<TestId, _>::from_iter_in([(); 3], &allocator);
        assert_eq!(
            data.into_iter().rev().collect::<std::vec::Vec<_>>(),
            [(); 3]
        );
    }

    #[test]
    fn remaining_slices() {
        #[derive(Debug)]
        #[repr(align(8))]
        struct Z;

        fn check_remaining<T>(iter: &mut crate::vec::IntoIter<'_, T>, len: usize) {
            assert_eq!(iter.len(), len);
            let slice = iter.as_slice();
            assert_eq!(slice.len(), len);
            assert_eq!(slice.as_ptr().align_offset(std::mem::align_of::<T>()), 0);
            let slice = iter.as_mut_slice();
            assert_eq!(slice.len(), len);
            assert_eq!(slice.as_ptr().align_offset(std::mem::align_of::<T>()), 0);
        }

        let allocator = Allocator::new();
        let data = ArenaIndexVec::<TestId, _>::from_iter_in([Z, Z, Z], &allocator);
        let mut iter = data.into_iter();
        check_remaining(&mut iter, 3);
        assert!(iter.next().is_some());
        check_remaining(&mut iter, 2);
        assert_eq!(format!("{iter:?}"), "IntoIter([Z, Z])");
        assert!(iter.next_back().is_some());
        check_remaining(&mut iter, 1);
        assert!(iter.next().is_some());
        check_remaining(&mut iter, 0);
        assert!(iter.next().is_none());
        assert!(iter.next_back().is_none());

        let data = ArenaIndexVec::<TestId, _>::from_iter_in([(); 3], &allocator);
        let mut iter = data.into_iter();
        check_remaining(&mut iter, 3);
        assert_eq!(iter.next(), Some(()));
        check_remaining(&mut iter, 2);
        assert_eq!(iter.next_back(), Some(()));
        check_remaining(&mut iter, 1);
        assert_eq!(iter.next(), Some(()));
        check_remaining(&mut iter, 0);

        let data = ArenaIndexVec::<TestId, _>::from_iter_in([10, 20, 30], &allocator);
        let mut iter = data.into_iter();
        check_remaining(&mut iter, 3);
        assert_eq!(iter.next(), Some(10));
        assert_eq!(iter.as_slice(), [20, 30]);
        iter.as_mut_slice()[0] = 40;
        assert_eq!(iter.next_back(), Some(30));
        check_remaining(&mut iter, 1);
        assert_eq!(iter.next(), Some(40));
        check_remaining(&mut iter, 0);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }
}
