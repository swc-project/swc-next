use std::{hash::Hash, marker::PhantomData, ptr::NonNull};

use bumpalo::Bump;
use oxc_index::IndexVec;
use rustc_hash::FxHasher;
use self_cell::self_cell;
use std::hash::Hasher;
use swc_core::atoms::wtf8::Wtf8;

use crate::{OptionalUtf8Ref, OptionalWtf8Ref, Utf8Ref, Wtf8Ref};

#[cfg(target_pointer_width = "64")]
const ROTATE: u32 = 26;
#[cfg(target_pointer_width = "32")]
const ROTATE: u32 = 15;

#[derive(Clone, Copy)]
pub struct Utf8Str<'s> {
    ptr: NonNull<u8>,
    len_and_hash: u64,
    _marker: PhantomData<&'s ()>,
}

#[derive(Clone, Copy)]
pub struct Wtf8Str<'s> {
    ptr: NonNull<u8>,
    len_and_hash: u64,
    _marker: PhantomData<&'s ()>,
}

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
    allocated_utf8: IndexVec<Utf8Ref, Utf8Str<'cell>>,
    allocated_wtf8: IndexVec<Wtf8Ref, Wtf8Str<'cell>>,
}

impl StringAllocator {
    pub fn new_with_source_len(source_len: usize) -> Self {
        Self::new(Bump::with_capacity(source_len / 2), |_| {
            StringAllocatorInner {
                allocated_utf8: IndexVec::new(),
                allocated_wtf8: IndexVec::new(),
            }
        })
    }

    #[inline]
    pub fn add_utf8(&mut self, s: &str) -> Utf8Ref {
        self.with_dependent_mut(|_allocator, cell| {
            let ptr = NonNull::from(s).cast::<u8>();

            // Produce a hash of the string
            let hash = {
                let mut hasher = FxHasher::default();
                s.hash(&mut hasher);
                hasher.finish()
            };

            // `FxHasher::finish` performs a rotation.
            // That's not what we want as we want the highest entropy bits in top 32 bits.
            // Undo that rotation here by rotating back in the opposite direction.
            // This code is the exact reverse of the code in `FxHasher::finish`,
            // so compiler should see that together they make a no-op, and remove both rotations.
            let hash = hash.rotate_right(ROTATE);

            // With FxHasher, highest entropy is in top 32 bits. Clear bottom 32 bits.
            let hash = hash & !(u32::MAX as u64);
            // We know `s.len()` is <= u32::MAX so don't bother masking it
            let len = s.len() as u64;

            let len_and_hash = len | hash;

            cell.allocated_utf8.push(Utf8Str {
                ptr,
                len_and_hash,
                _marker: PhantomData,
            })
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
        self.with_dependent_mut(|_allocator, cell| {
            let ptr = NonNull::from(s).cast::<u8>();

            // Produce a hash of the string
            let hash = {
                let mut hasher = FxHasher::default();
                s.hash(&mut hasher);
                hasher.finish()
            };

            // `FxHasher::finish` performs a rotation.
            // That's not what we want as we want the highest entropy bits in top 32 bits.
            // Undo that rotation here by rotating back in the opposite direction.
            // This code is the exact reverse of the code in `FxHasher::finish`,
            // so compiler should see that together they make a no-op, and remove both rotations.
            let hash = hash.rotate_right(ROTATE);

            // With FxHasher, highest entropy is in top 32 bits. Clear bottom 32 bits.
            let hash = hash & !(u32::MAX as u64);
            // We know `s.len()` is <= u32::MAX so don't bother masking it
            let len = s.len() as u64;

            let len_and_hash = len | hash;

            cell.allocated_wtf8.push(Wtf8Str {
                ptr,
                len_and_hash,
                _marker: PhantomData,
            })
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
    pub fn get_utf8(&self, id: Utf8Ref) -> Utf8Str<'_> {
        self.borrow_dependent().allocated_utf8[id]
    }

    #[inline]
    pub fn get_optional_utf8(&self, id: OptionalUtf8Ref) -> Option<Utf8Str<'_>> {
        let id = id.to_option()?;
        Some(self.get_utf8(id))
    }

    #[inline]
    pub fn get_wtf8(&self, id: Wtf8Ref) -> Wtf8Str<'_> {
        self.borrow_dependent().allocated_wtf8[id]
    }

    #[inline]
    pub fn get_optional_wtf8(&self, id: OptionalWtf8Ref) -> Option<Wtf8Str<'_>> {
        let id = id.to_option()?;
        Some(self.get_wtf8(id))
    }
}
