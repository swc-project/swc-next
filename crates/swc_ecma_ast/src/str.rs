use std::{fmt::Debug, hash::Hash, marker::PhantomData, ops::Deref, ptr::NonNull};

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

#[derive(Clone, Copy, Eq)]
pub struct Utf8Str<'s> {
    ptr: NonNull<u8>,
    len_and_hash: u64,
    _marker: PhantomData<&'s ()>,
}

impl<'s> Utf8Str<'s> {
    pub fn new(s: &str) -> Self {
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

        Self {
            ptr,
            len_and_hash,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len_and_hash as u32 as usize
    }

    pub fn as_str(self) -> &'s str {
        unsafe {
            let slice = std::slice::from_raw_parts(self.ptr.as_ptr(), self.len());
            std::str::from_utf8_unchecked(slice)
        }
    }
}

impl PartialEq for Utf8Str<'_> {
    #[inline]
    fn eq(&self, other: &Utf8Str<'_>) -> bool {
        // Skip full string comparison unless *both* length and hash match.
        // So we get faster `==` as well as faster hashing.
        self.len_and_hash == other.len_and_hash && self.as_str() == other.as_str()
    }
}

impl PartialEq<str> for Utf8Str<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl Deref for Utf8Str<'_> {
    type Target = str;

    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Debug for Utf8Str<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_str(), f)
    }
}

// Only use this `Hash` impl with `IdentHashMap`
impl Hash for Utf8Str<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        // Hash is stored in top 32 bits of `len_and_hash`.
        // `hashbrown` only uses top 7 bits and bottom N bits of hash,
        // where N is the exponent of number of buckets in the hash table.
        // So just rotate by 25 bits, so 32-bit hash occupies top 7 bits
        // and bottom 25 bits. This gives good entropy as long as HashMap
        // contains less than 58 million entries ((1 << 25) * 7 / 8).
        // The bits in the middle have low entropy, but that doesn't matter
        // because `hashbrown` will ignore them anyway.
        let hash = self.len_and_hash.rotate_left(32 - 7);
        hasher.write_u64(hash);
    }
}

#[derive(Clone, Copy)]
pub struct Wtf8Str<'s> {
    ptr: NonNull<u8>,
    len_and_hash: u64,
    _marker: PhantomData<&'s ()>,
}

impl<'s> Wtf8Str<'s> {
    pub fn new(s: &Wtf8) -> Self {
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

        Self {
            ptr,
            len_and_hash,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len_and_hash as u32 as usize
    }

    pub fn as_wtf8(self) -> &'s Wtf8 {
        unsafe {
            let slice = std::slice::from_raw_parts(self.ptr.as_ptr(), self.len());
            Wtf8::from_bytes_unchecked(slice)
        }
    }
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
    empty_utf8_ref: Utf8Ref,
    allocated_utf8: IndexVec<Utf8Ref, Utf8Str<'cell>>,
    allocated_wtf8: IndexVec<Wtf8Ref, Wtf8Str<'cell>>,
}

impl StringAllocator {
    pub fn new_with_source_len(source_len: usize) -> Self {
        Self::new(Bump::with_capacity(source_len / 2), |_| {
            let mut allocated_utf8 = IndexVec::new();
            let empty_utf8_ref = allocated_utf8.push(Utf8Str::new(""));
            StringAllocatorInner {
                empty_utf8_ref,
                allocated_utf8,
                allocated_wtf8: IndexVec::new(),
            }
        })
    }

    #[inline]
    pub fn add_utf8(&mut self, s: &str) -> Utf8Ref {
        self.with_dependent_mut(|_allocator, cell| cell.allocated_utf8.push(Utf8Str::new(s)))
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
        self.with_dependent_mut(|_allocator, cell| cell.allocated_wtf8.push(Wtf8Str::new(s)))
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

    #[inline]
    pub fn empty_utf8_ref(&self) -> Utf8Ref {
        self.borrow_dependent().empty_utf8_ref
    }
}
