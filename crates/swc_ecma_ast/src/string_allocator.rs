use swc_core::atoms::wtf8::{Wtf8, Wtf8Buf};

use crate::{OptionalUtf8Ref, OptionalWtf8Ref, Utf8Ref, Wtf8Ref};

/// A string allocator that can be used to allocate strings for the AST.
/// All the strings are stored in a single buffer to avoid memory fragmentation.
pub struct StringAllocator {
    allocated_utf8: String,
    allocated_wtf8: Wtf8Buf,
}

impl StringAllocator {
    /// Create a new string allocator with the given source length.
    /// The source length is used to pre-allocate memory for the string allocator.
    /// We assume that half length of the source code is the sum of utf8 identifier lengths.
    pub fn new(source_len: usize) -> Self {
        Self {
            allocated_utf8: String::with_capacity(source_len / 2),
            allocated_wtf8: Wtf8Buf::new(),
        }
    }

    #[inline]
    pub fn add_utf8(&mut self, s: &str) -> Utf8Ref {
        let lo = self.allocated_utf8.len() as u32;
        self.allocated_utf8.push_str(s);
        let hi = self.allocated_utf8.len() as u32;
        Utf8Ref::new_ref(lo, hi)
    }

    #[inline]
    pub fn add_optional_utf8(&mut self, s: Option<&str>) -> OptionalUtf8Ref {
        match s {
            Some(s) => self.add_utf8(s).into(),
            None => OptionalUtf8Ref::new_none(),
        }
    }

    #[inline]
    pub fn add_wtf8(&mut self, s: &Wtf8) -> Wtf8Ref {
        let lo = self.allocated_wtf8.len() as u32;
        self.allocated_wtf8.push_wtf8(s);
        let hi = self.allocated_wtf8.len() as u32;
        Wtf8Ref::new_ref(lo, hi)
    }

    #[inline]
    pub fn add_optional_wtf8(&mut self, s: Option<&Wtf8>) -> OptionalWtf8Ref {
        match s {
            Some(s) => self.add_wtf8(s).into(),
            None => OptionalWtf8Ref::new_none(),
        }
    }

    #[inline]
    pub fn get_utf8(&self, id: Utf8Ref) -> &str {
        &self.allocated_utf8[id.lo() as usize..id.hi() as usize]
    }

    #[inline]
    pub fn get_optional_utf8(&self, id: OptionalUtf8Ref) -> Option<&str> {
        let id = id.to_option()?;
        Some(self.get_utf8(id))
    }

    #[inline]
    pub fn get_wtf8(&self, id: Wtf8Ref) -> &Wtf8 {
        self.allocated_wtf8
            .slice(id.lo() as usize, id.hi() as usize)
    }

    #[inline]
    pub fn get_optional_wtf8(&self, id: OptionalWtf8Ref) -> Option<&Wtf8> {
        let id = id.to_option()?;
        Some(self.get_wtf8(id))
    }

    pub(crate) fn get_checkpoint_data(&self) -> (usize, usize) {
        (self.allocated_utf8.len(), self.allocated_wtf8.len())
    }

    pub(crate) fn restore_checkpoint_data(&mut self, data: (usize, usize)) {
        self.allocated_utf8.truncate(data.0);
        self.allocated_wtf8.truncate(data.1);
    }
}
