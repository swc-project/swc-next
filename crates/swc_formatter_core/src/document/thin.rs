//! Compact, immutable borrows used by the document instruction encoding.

use std::{fmt, marker::PhantomData, ptr::NonNull};

/// A slice borrow with a checked `u32` length and four-byte alignment.
#[repr(C, packed(4))]
pub(super) struct ThinSlice<'a, T> {
    ptr: NonNull<T>,
    len: u32,
    marker: PhantomData<&'a [T]>,
}

impl<'a, T> ThinSlice<'a, T> {
    /// Panics if the slice has more than `u32::MAX` elements.
    #[inline]
    pub(super) const fn new(slice: &'a [T]) -> Self {
        assert!(
            slice.len() <= u32::MAX as usize,
            "document slice exceeds u32::MAX"
        );
        Self {
            ptr: NonNull::from_ref(slice).cast(),
            len: slice.len() as u32,
            marker: PhantomData,
        }
    }

    #[inline]
    pub(super) fn get(self) -> &'a [T] {
        // SAFETY: `new` retains the original slice's non-null, aligned pointer
        // and checks that its length fits without truncation. The shared borrow
        // remains valid for `'a`, as recorded by `marker`; neither field changes.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len as usize) }
    }
}

impl<T> Clone for ThinSlice<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ThinSlice<'_, T> {}

impl<T: fmt::Debug> fmt::Debug for ThinSlice<'_, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(formatter)
    }
}

/// UTF-8 text whose bytes retain the original string borrow.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub(super) struct ThinStr<'a>(ThinSlice<'a, u8>);

impl<'a> ThinStr<'a> {
    #[inline]
    pub(super) const fn new(source: &'a str) -> Self {
        Self(ThinSlice::new(source.as_bytes()))
    }

    #[inline]
    pub(super) fn as_str(self) -> &'a str {
        // SAFETY: The private byte slice is constructed only from a valid
        // `&'a str` in `new`, and its bytes and checked length never change.
        unsafe { std::str::from_utf8_unchecked(self.0.get()) }
    }
}

impl fmt::Debug for ThinStr<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(formatter)
    }
}
