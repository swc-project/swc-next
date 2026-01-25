use oxc_index::IndexVec;
use swc_core::atoms::wtf8::{Wtf8, Wtf8Buf};

use crate::{OptionalUtf8Ref, OptionalWtf8Ref, Utf8Ref, Wtf8Ref};

/// A string allocator that can be used to allocate strings for the AST.
/// All the strings are stored in a single buffer to avoid memory fragmentation.
pub struct StringAllocator {
    allocated_utf8: String,
    allocated_wtf8: Wtf8Buf,
    utf8_ends: IndexVec<Utf8Ref, u32>,
    wtf8_ends: IndexVec<Wtf8Ref, u32>,
    empty_utf8_ref: Utf8Ref,
    empty_wtf8_ref: Wtf8Ref,
}

impl StringAllocator {
    /// Create a new string allocator with the given source length.
    /// The source length is used to pre-allocate memory for the string allocator.
    /// We assume that half length of the source code is the sum of utf8 identifier lengths.
    pub fn new(source_len: usize) -> Self {
        let mut utf8_ends = IndexVec::new();
        utf8_ends.push(0);
        let empty_utf8_ref = utf8_ends.push(0);

        let mut wtf8_ends = IndexVec::new();
        wtf8_ends.push(0);
        let empty_wtf8_ref = wtf8_ends.push(0);

        Self {
            allocated_utf8: String::with_capacity(source_len / 2),
            allocated_wtf8: Wtf8Buf::new(),
            utf8_ends,
            wtf8_ends,
            empty_utf8_ref,
            empty_wtf8_ref,
        }
    }

    #[inline]
    pub fn add_utf8(&mut self, s: &str) -> Utf8Ref {
        self.allocated_utf8.push_str(s);
        let hi = self.allocated_utf8.len() as u32;
        self.utf8_ends.push(hi)
    }

    #[inline]
    pub fn add_optional_utf8(&mut self, s: Option<&str>) -> OptionalUtf8Ref {
        match s {
            Some(s) => self.add_utf8(s).into(),
            None => OptionalUtf8Ref::none(),
        }
    }

    #[inline]
    pub fn add_wtf8(&mut self, s: &Wtf8) -> Wtf8Ref {
        self.allocated_wtf8.push_wtf8(s);
        let hi = self.allocated_wtf8.len() as u32;
        self.wtf8_ends.push(hi)
    }

    #[inline]
    pub fn add_optional_wtf8(&mut self, s: Option<&Wtf8>) -> OptionalWtf8Ref {
        match s {
            Some(s) => self.add_wtf8(s).into(),
            None => OptionalWtf8Ref::none(),
        }
    }

    #[inline]
    pub fn get_utf8(&self, id: Utf8Ref) -> &str {
        let hi = unsafe { *self.utf8_ends.as_raw_slice().get_unchecked(id.index()) };
        let lo = unsafe { *self.utf8_ends.as_raw_slice().get_unchecked(id.index() - 1) };
        &self.allocated_utf8[lo as usize..hi as usize]
    }

    #[inline]
    pub fn get_optional_utf8(&self, id: OptionalUtf8Ref) -> Option<&str> {
        let id = id.to_option()?;
        Some(self.get_utf8(id))
    }

    #[inline]
    pub fn get_wtf8(&self, id: Wtf8Ref) -> &Wtf8 {
        let hi = unsafe { *self.wtf8_ends.as_raw_slice().get_unchecked(id.index()) };
        let lo = unsafe { *self.wtf8_ends.as_raw_slice().get_unchecked(id.index() - 1) };
        self.allocated_wtf8.slice(lo as usize, hi as usize)
    }

    #[inline]
    pub fn get_optional_wtf8(&self, id: OptionalWtf8Ref) -> Option<&Wtf8> {
        let id = id.to_option()?;
        Some(self.get_wtf8(id))
    }

    #[inline]
    pub fn empty_utf8_ref(&self) -> Utf8Ref {
        self.empty_utf8_ref
    }

    #[inline]
    pub fn empty_wtf8_ref(&self) -> Wtf8Ref {
        self.empty_wtf8_ref
    }
}
