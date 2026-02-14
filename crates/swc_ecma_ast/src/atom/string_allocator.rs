use std::cell::UnsafeCell;

use swc_core::atoms::{Atom, Wtf8Atom, wtf8::Wtf8};

use crate::{
    OptionalUtf8Ref, OptionalWtf8Ref, Utf8Ref, Wtf8Ref, utf8::Utf8Allocator, wtf8::Wtf8Allocator,
};

/// # Safety
/// [StringAllocatorInner] uses [string_interner::backend::BucketBackend], which has stable string references
pub struct StringAllocator(UnsafeCell<StringAllocatorInner>);

impl Default for StringAllocator {
    fn default() -> Self {
        Self(UnsafeCell::new(StringAllocatorInner::new()))
    }
}

impl StringAllocator {
    /// # Safety
    /// [StringAllocatorInner] uses [string_interner::backend::BucketBackend], which has stable string references
    #[allow(clippy::mut_from_ref)]
    fn inner(&self) -> &mut StringAllocatorInner {
        unsafe { &mut *self.0.get() }
    }

    pub fn add_utf8(&self, s: &str) -> Utf8Ref {
        self.inner().add_utf8(s)
    }

    pub fn add_optional_utf8(&self, s: Option<&str>) -> OptionalUtf8Ref {
        self.inner().add_optional_utf8(s)
    }

    pub fn add_wtf8(&self, s: &Wtf8) -> Wtf8Ref {
        self.inner().add_wtf8(s)
    }

    pub fn add_optional_wtf8(&self, s: Option<&Wtf8>) -> OptionalWtf8Ref {
        self.inner().add_optional_wtf8(s)
    }

    pub fn get_utf8(&self, id: Utf8Ref) -> &str {
        self.inner().get_utf8(id)
    }

    pub fn get_atom(&self, id: Utf8Ref) -> Atom {
        self.inner().get_atom(id)
    }

    pub fn get_optional_utf8(&self, id: OptionalUtf8Ref) -> Option<&str> {
        self.inner().get_optional_utf8(id)
    }

    pub fn get_wtf8(&self, id: Wtf8Ref) -> &Wtf8 {
        self.inner().get_wtf8(id)
    }

    pub fn get_wtf8_atom(&self, id: Wtf8Ref) -> Wtf8Atom {
        self.inner().get_wtf8_atom(id)
    }

    pub fn get_optional_wtf8(&self, id: OptionalWtf8Ref) -> Option<&Wtf8> {
        self.inner().get_optional_wtf8(id)
    }

    pub fn empty_utf8_ref(&self) -> Utf8Ref {
        self.inner().empty_utf8_ref()
    }
}

/// A string allocator that can be used to allocate strings for the AST.
/// All the strings are stored in a single buffer to avoid memory fragmentation.
struct StringAllocatorInner {
    allocated_utf8: Utf8Allocator,
    allocated_wtf8: Wtf8Allocator,
    empty_utf8: Utf8Ref,
}

impl StringAllocatorInner {
    pub fn new() -> Self {
        let mut allocated_utf8 = Utf8Allocator::new();
        let empty_utf8 = allocated_utf8.get_or_intern("");
        Self {
            allocated_utf8,
            allocated_wtf8: Wtf8Allocator::default(),
            empty_utf8,
        }
    }

    #[inline]
    pub fn add_utf8(&mut self, s: &str) -> Utf8Ref {
        self.allocated_utf8.get_or_intern(s)
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
        self.allocated_wtf8.add(s)
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
        self.allocated_utf8.resolve(id).unwrap()
    }

    #[inline]
    pub fn get_atom(&self, id: Utf8Ref) -> Atom {
        Atom::from(self.get_utf8(id))
    }

    #[inline]
    pub fn get_optional_utf8(&self, id: OptionalUtf8Ref) -> Option<&str> {
        let id = id.to_option()?;
        Some(self.get_utf8(id))
    }

    #[inline]
    pub fn get_wtf8(&self, id: Wtf8Ref) -> &Wtf8 {
        self.allocated_wtf8.resolve(id).unwrap()
    }

    #[inline]
    pub fn get_wtf8_atom(&self, id: Wtf8Ref) -> Wtf8Atom {
        Wtf8Atom::from(self.get_wtf8(id))
    }

    #[inline]
    pub fn get_optional_wtf8(&self, id: OptionalWtf8Ref) -> Option<&Wtf8> {
        let id = id.to_option()?;
        Some(self.get_wtf8(id))
    }

    #[inline]
    pub fn empty_utf8_ref(&self) -> Utf8Ref {
        self.empty_utf8
    }
}
