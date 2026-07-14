use std::{
    alloc::{Layout, handle_alloc_error},
    cell::Cell,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use allocator_api2::alloc::AllocError;

mod allocator_api;
pub mod atom;
pub mod boxed;
pub mod hash_map;
pub mod hash_set;
pub mod raw_vec;
pub mod vec;
pub mod wtf8;

pub trait CloneIn<'a>: Sized {
    type Cloned;

    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned;
}

macro_rules! impl_clone_in_trivial {
    ($i:ty) => {
        impl<'a> CloneIn<'a> for $i {
            type Cloned = $i;

            fn clone_in(&self, _allocator: &'a Allocator) -> Self::Cloned {
                self.clone()
            }
        }
    };
}

impl_clone_in_trivial!(bool);
impl_clone_in_trivial!(f64);

impl<'a, 'src, C: 'a, T> CloneIn<'a> for boxed::Box<'src, T>
where
    T: CloneIn<'a, Cloned = C>,
{
    type Cloned = boxed::Box<'a, C>;

    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        allocator.boxed((**self).clone_in(allocator))
    }
}

impl<'a, 'src, C: 'a, T> CloneIn<'a> for vec::Vec<'src, T>
where
    T: CloneIn<'a, Cloned = C>,
{
    type Cloned = vec::Vec<'a, C>;

    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        let mut cloned = vec::Vec::with_capacity_in(self.len(), allocator);
        for item in self {
            cloned.push(item.clone_in(allocator));
        }
        cloned
    }
}

impl<'a, C, T> CloneIn<'a> for Option<T>
where
    T: CloneIn<'a, Cloned = C>,
{
    type Cloned = Option<C>;

    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        self.as_ref().map(|it| it.clone_in(allocator))
    }
}

impl<'a, C, T> CloneIn<'a> for Cell<T>
where
    T: Copy + CloneIn<'a, Cloned = C>,
{
    type Cloned = Cell<C>;

    fn clone_in(&self, allocator: &'a Allocator) -> Self::Cloned {
        Cell::new(self.get().clone_in(allocator))
    }
}

/// This trait works similarly to the standard library [`From`] trait.
pub trait FromIn<'a, T>: Sized {
    /// Converts to this type from the input type within the given `allocator`.
    fn from_in(value: T, allocator: &'a Allocator) -> Self;
}

/// This trait works similarly to the standard library [`Into`] trait.
pub trait IntoIn<'a, T>: Sized {
    /// Converts this type into the inferred target type within the given `allocator`.
    fn into_in(self, allocator: &'a Allocator) -> T;
}

impl<'a, T> FromIn<'a, T> for T {
    #[inline(always)]
    fn from_in(t: T, _: &'a Allocator) -> T {
        t
    }
}

impl<'a, T, U> IntoIn<'a, U> for T
where
    U: FromIn<'a, T>,
{
    #[inline]
    fn into_in(self, allocator: &'a Allocator) -> U {
        U::from_in(self, allocator)
    }
}

impl<'a> FromIn<'a, String> for &'a str {
    #[inline(always)]
    fn from_in(value: String, allocator: &'a Allocator) -> Self {
        allocator.alloc_str(value.as_str())
    }
}

impl<'a, T> FromIn<'a, T> for boxed::Box<'a, T> {
    #[inline(always)]
    fn from_in(value: T, allocator: &'a Allocator) -> Self {
        boxed::Box::new_in(value, allocator)
    }
}

impl<'a, T> FromIn<'a, Option<T>> for Option<boxed::Box<'a, T>> {
    #[inline(always)]
    fn from_in(value: Option<T>, allocator: &'a Allocator) -> Self {
        value.map(|it| boxed::Box::new_in(it, allocator))
    }
}

/// A bump allocator backed by a caller-owned byte slice.
///
/// The allocator never grows beyond the supplied buffer. Individual allocations are reclaimed
/// only when they are the most recent allocation; [`Allocator::reset`] reclaims everything at
/// once.
pub struct Allocator<'buffer> {
    base: NonNull<u8>,
    capacity: usize,
    end: Cell<usize>,
    _buffer: PhantomData<&'buffer mut [MaybeUninit<u8>]>,
}

impl<'buffer> Allocator<'buffer> {
    /// Creates an allocator over `buffer`.
    #[inline]
    pub fn new(buffer: &'buffer mut [MaybeUninit<u8>]) -> Self {
        Self {
            // A slice's pointer is non-null even when the slice is empty.
            base: unsafe { NonNull::new_unchecked(buffer.as_mut_ptr().cast()) },
            capacity: buffer.len(),
            end: Cell::new(0),
            _buffer: PhantomData,
        }
    }

    /// Reclaims all allocations made from this allocator.
    #[inline]
    pub fn reset(&mut self) {
        self.end.set(0);
    }

    /// Returns the number of bytes consumed from the backing buffer, including alignment padding.
    #[inline]
    pub fn used_bytes(&self) -> usize {
        self.end.get()
    }

    /// Returns the total size of the backing buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub fn alloc<T>(&self, value: T) -> &mut T {
        let layout = Layout::new::<T>();
        let ptr = self
            .try_alloc_layout(layout)
            .unwrap_or_else(|_| handle_alloc_error(layout))
            .cast::<T>();
        unsafe {
            ptr.as_ptr().write(value);
            &mut *ptr.as_ptr()
        }
    }

    #[inline]
    pub fn alloc_str(&self, s: &str) -> &str {
        let bytes = self.alloc_slice_copy(s.as_bytes());
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    #[inline]
    pub fn alloc_wtf8(&self, s: &wtf8::Wtf8) -> &wtf8::Wtf8 {
        let bytes = self.alloc_slice_copy(s.as_bytes());
        unsafe { wtf8::Wtf8::from_bytes_unchecked(bytes) }
    }

    pub fn boxed<T>(&self, value: T) -> boxed::Box<'_, T> {
        boxed::Box::new_in(value, self)
    }

    pub fn vec<T>(&self) -> vec::Vec<'_, T> {
        vec::Vec::new_in(self)
    }

    #[inline]
    fn alloc_slice_copy<T: Copy>(&self, source: &[T]) -> &mut [T] {
        let layout = Layout::array::<T>(source.len()).unwrap_or_else(|_| {
            handle_alloc_error(Layout::new::<T>());
        });
        let allocation = self
            .try_alloc_layout(layout)
            .unwrap_or_else(|_| handle_alloc_error(layout));
        let target = allocation.cast::<T>();
        unsafe {
            ptr::copy_nonoverlapping(source.as_ptr(), target.as_ptr(), source.len());
            std::slice::from_raw_parts_mut(target.as_ptr(), source.len())
        }
    }

    #[inline]
    pub(crate) fn try_alloc_layout(&self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        if layout.size() == 0 {
            return Ok(unsafe { NonNull::new_unchecked(layout.align() as *mut u8) });
        }

        let base = self.base.as_ptr() as usize;
        let current = base.checked_add(self.end.get()).ok_or(AllocError)?;
        let aligned = current
            .checked_add(layout.align() - 1)
            .map(|address| address & !(layout.align() - 1))
            .ok_or(AllocError)?;
        let start = aligned.checked_sub(base).ok_or(AllocError)?;
        let new_end = start.checked_add(layout.size()).ok_or(AllocError)?;
        if new_end > self.capacity {
            return Err(AllocError);
        }

        self.end.set(new_end);
        Ok(unsafe { NonNull::new_unchecked(self.base.as_ptr().add(start)) })
    }

    #[inline]
    pub(crate) unsafe fn deallocate_layout(&self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() == 0 {
            return;
        }

        let start = ptr.as_ptr() as usize - self.base.as_ptr() as usize;
        if start + layout.size() == self.end.get() {
            self.end.set(start);
        }
    }

    pub(crate) unsafe fn shrink_layout(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if old_layout.size() != 0 {
            let start = ptr.as_ptr() as usize - self.base.as_ptr() as usize;
            if ptr.as_ptr() as usize % new_layout.align() == 0 {
                if start + old_layout.size() == self.end.get() {
                    self.end.set(start + new_layout.size());
                }
                return Ok(NonNull::slice_from_raw_parts(ptr, new_layout.size()));
            }
        }

        let new_ptr = self.try_alloc_layout(new_layout)?;
        if new_layout.size() != 0 {
            unsafe { ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), new_layout.size()) };
        }
        Ok(NonNull::slice_from_raw_parts(new_ptr, new_layout.size()))
    }

    pub(crate) unsafe fn grow_layout(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
        zeroed: bool,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let old_size = old_layout.size();
        let new_size = new_layout.size();

        if old_size != 0 {
            let start = ptr.as_ptr() as usize - self.base.as_ptr() as usize;
            if start + old_size == self.end.get()
                && ptr.as_ptr() as usize % new_layout.align() == 0
                && start
                    .checked_add(new_size)
                    .is_some_and(|end| end <= self.capacity)
            {
                self.end.set(start + new_size);
                if zeroed {
                    unsafe {
                        ptr.as_ptr()
                            .add(old_size)
                            .write_bytes(0, new_size - old_size)
                    };
                }
                return Ok(NonNull::slice_from_raw_parts(ptr, new_size));
            }
        }

        let new_ptr = self.try_alloc_layout(new_layout)?;
        if old_size != 0 {
            unsafe { ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), old_size) };
        }
        if zeroed {
            unsafe {
                new_ptr
                    .as_ptr()
                    .add(old_size)
                    .write_bytes(0, new_size - old_size)
            };
        }
        Ok(NonNull::slice_from_raw_parts(new_ptr, new_size))
    }

    pub(crate) unsafe fn try_grow_in_place(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<(), AllocError> {
        let start = ptr.as_ptr() as usize - self.base.as_ptr() as usize;
        let new_end = start.checked_add(new_layout.size()).ok_or(AllocError)?;
        if start + old_layout.size() != self.end.get()
            || ptr.as_ptr() as usize % new_layout.align() != 0
            || new_end > self.capacity
        {
            return Err(AllocError);
        }
        self.end.set(new_end);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{alloc::Layout, mem::MaybeUninit};

    use allocator_api2::alloc::Allocator as _;

    use super::Allocator;

    #[test]
    fn allocates_from_the_supplied_buffer_with_the_requested_alignment() {
        let mut buffer = [MaybeUninit::uninit(); 64];
        let buffer_start = buffer.as_ptr() as usize;
        let buffer_end = buffer_start + buffer.len();
        let allocator = Allocator::new(&mut buffer);

        let byte = allocator.alloc(1_u8) as *mut u8 as usize;
        let word = allocator.alloc(2_u64) as *mut u64 as usize;

        assert!((buffer_start..buffer_end).contains(&byte));
        assert!((buffer_start..buffer_end).contains(&word));
        assert_eq!(word % align_of::<u64>(), 0);
    }

    #[test]
    fn reports_allocation_failure_when_the_buffer_is_exhausted() {
        let mut buffer = [MaybeUninit::uninit(); 4];
        let allocator = Allocator::new(&mut buffer);

        assert!(
            allocator
                .allocate(Layout::from_size_align(4, 1).unwrap())
                .is_ok()
        );
        assert!(allocator.allocate(Layout::new::<u8>()).is_err());
    }

    #[test]
    fn grows_the_latest_allocation_in_place() {
        let mut buffer = [MaybeUninit::uninit(); 16];
        let allocator = Allocator::new(&mut buffer);
        let old_layout = Layout::from_size_align(4, 1).unwrap();
        let new_layout = Layout::from_size_align(8, 1).unwrap();
        let allocation = allocator.allocate(old_layout).unwrap();
        let old_ptr = allocation.cast::<u8>();

        let grown = unsafe { allocator.grow(old_ptr, old_layout, new_layout) }.unwrap();

        assert_eq!(grown.cast::<u8>(), old_ptr);
        assert_eq!(allocator.used_bytes(), 8);
    }

    #[test]
    fn moves_a_non_latest_allocation_and_zeroes_the_grown_region() {
        let mut buffer = [MaybeUninit::uninit(); 32];
        let allocator = Allocator::new(&mut buffer);
        let old_layout = Layout::from_size_align(4, 1).unwrap();
        let new_layout = Layout::from_size_align(8, 1).unwrap();
        let allocation = allocator.allocate(old_layout).unwrap();
        let old_ptr = allocation.cast::<u8>();
        unsafe {
            old_ptr
                .as_ptr()
                .copy_from_nonoverlapping([1, 2, 3, 4].as_ptr(), 4)
        };
        let _blocker = allocator.alloc(0_u8);

        let grown = unsafe { allocator.grow_zeroed(old_ptr, old_layout, new_layout) }.unwrap();
        let grown = unsafe { grown.as_ptr().as_ref().unwrap() };

        assert_ne!(grown.as_ptr(), old_ptr.as_ptr());
        assert_eq!(grown, &[1, 2, 3, 4, 0, 0, 0, 0]);
    }

    #[test]
    fn reset_reuses_the_buffer() {
        let mut buffer = [MaybeUninit::uninit(); 8];
        let mut allocator = Allocator::new(&mut buffer);
        let first = allocator.alloc(1_u64) as *mut u64;

        allocator.reset();
        let second = allocator.alloc(2_u64) as *mut u64;

        assert_eq!(first, second);
    }
}
