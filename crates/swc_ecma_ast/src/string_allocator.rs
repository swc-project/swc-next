use crate::{OptionalUtf8Ref, OptionalWtf8Ref, Utf8Ref, Wtf8Ref};
use bumpalo::Bump;
use oxc_index::IndexVec;
use self_cell::self_cell;
use swc_core::atoms::wtf8::Wtf8;

self_cell! {
    /// A string allocator that can be used to allocate strings for the AST.
    /// All the strings are stored in a single buffer to avoid memory fragmentation.
    pub struct StringAllocator {
        owner: Bump,
        #[covariant]
        dependent: StringAllocatorInner,
    }
}

pub struct StringAllocatorInner<'cell> {
    empty_utf8_ref: Utf8Ref,
    allocated_utf8: IndexVec<Utf8Ref, &'cell str>,
    allocated_wtf8: IndexVec<Wtf8Ref, &'cell Wtf8>,
}

impl StringAllocator {
    pub fn new_with_source_len(source_len: usize) -> Self {
        Self::new(Bump::with_capacity(source_len / 2), |_| {
            let mut allocated_utf8 = IndexVec::new();
            let empty_utf8_ref = allocated_utf8.push("");
            StringAllocatorInner {
                empty_utf8_ref,
                allocated_utf8,
                allocated_wtf8: IndexVec::new(),
            }
        })
    }

    #[inline]
    pub fn add_utf8(&mut self, s: &str) -> Utf8Ref {
        self.with_dependent_mut(|allocator, cell| {
            let s = allocator.alloc_str(s);
            cell.allocated_utf8.push(s)
        })
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
        self.with_dependent_mut(|allocator, cell| {
            let s = allocator.alloc_slice_copy(s.as_bytes());
            cell.allocated_wtf8
                .push(unsafe { Wtf8::from_bytes_unchecked(s) })
        })
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
        self.borrow_dependent().allocated_utf8[id]
    }

    #[inline]
    pub fn get_optional_utf8(&self, id: OptionalUtf8Ref) -> Option<&str> {
        let id = id.to_option()?;
        Some(self.get_utf8(id))
    }

    #[inline]
    pub fn get_wtf8(&self, id: Wtf8Ref) -> &Wtf8 {
        self.borrow_dependent().allocated_wtf8[id]
    }

    #[inline]
    pub fn get_optional_wtf8(&self, id: OptionalWtf8Ref) -> Option<&Wtf8> {
        let id = id.to_option()?;
        Some(self.get_wtf8(id))
    }

    #[inline]
    pub fn empty_utf8_ref(&self) -> Utf8Ref {
        self.borrow_dependent().empty_utf8_ref
    }
}
