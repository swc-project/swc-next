use swc_next_allocator::{
    Allocator,
    hash_map::{EntryRef, HashMap as ArenaHashMap},
    vec::ArenaVec,
    wtf8::Wtf8,
};

macro_rules! define_ast_string {
    ($name:ident) => {
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
        pub struct $name {
            pub(crate) start: u32,
            pub(crate) end: u32,
        }

        impl $name {
            pub const EMPTY: Self = Self { start: 0, end: 0 };

            #[inline]
            pub const fn len(self) -> u32 {
                debug_assert!(self.end >= self.start);
                self.end - self.start
            }

            #[inline]
            pub const fn is_empty(self) -> bool {
                self.start == self.end
            }
        }
    };
}

define_ast_string!(AstUtf8);
define_ast_string!(AstInternedUtf8);
define_ast_string!(AstWtf8);

/// Read-only access to canonical ranges in an AST's arena-owned string table.
/// The view remains valid when the AST grows or is dropped before its arena.
#[derive(Clone, Copy, Debug)]
pub struct AstStringLookup<'a> {
    dedup: *mut (),
    marker: std::marker::PhantomData<&'a Allocator>,
}

impl AstStringLookup<'_> {
    /// Finds a previously interned value without adding it to the table.
    #[inline]
    pub fn get(self, text: &[u8]) -> Option<AstWtf8> {
        if text.is_empty() {
            return Some(AstWtf8::EMPTY);
        }
        // SAFETY: the table and its owned keys live in the arena. No reference
        // into the table escapes this call. The raw pointer prevents Send/Sync.
        unsafe {
            &*self
                .dedup
                .cast::<ArenaHashMap<'_, ArenaVec<'_, u8>, (u32, u32)>>()
        }
        .get(text)
        .map(|&(start, end)| AstWtf8 { start, end })
    }
}

impl AstUtf8 {
    #[inline]
    pub(crate) const fn from_raw(raw: u64) -> Self {
        Self {
            start: raw as u32,
            end: (raw >> 32) as u32,
        }
    }

    #[inline]
    pub(crate) const fn raw(self) -> u64 {
        self.start as u64 | ((self.end as u64) << 32)
    }
}

impl AstWtf8 {
    #[inline]
    pub(crate) const fn from_raw(raw: u64) -> Self {
        Self {
            start: raw as u32,
            end: (raw >> 32) as u32,
        }
    }

    #[inline]
    pub(crate) const fn raw(self) -> u64 {
        self.start as u64 | ((self.end as u64) << 32)
    }
}

impl From<AstInternedUtf8> for AstUtf8 {
    #[inline]
    fn from(value: AstInternedUtf8) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<AstUtf8> for AstWtf8 {
    #[inline]
    fn from(value: AstUtf8) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<AstInternedUtf8> for AstWtf8 {
    #[inline]
    fn from(value: AstInternedUtf8) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

pub(crate) struct StringPool<'a> {
    allocator: &'a Allocator,
    pub(crate) source: &'a str,
    extra: ArenaVec<'a, u8>,
    // Type-erased arena pointer keeps the pool covariant in `'a`. The pointee
    // is mutated only through `dedup`, and raw mutable pointers make the pool
    // non-`Sync`, so safe code cannot intern concurrently.
    dedup: *mut (),
    // Memoize immutable ranges, including aliases of the canonical range. A
    // second semantic pass must not hash their contents again.
    interned_ranges: *mut (),
}

impl<'a> StringPool<'a> {
    pub fn reserve_interned_ranges(&self, capacity: usize) {
        // SAFETY: same arena-owned, non-Sync table used by intern_range; no
        // references into it escape this call.
        let ranges = unsafe {
            &mut *self
                .interned_ranges
                .cast::<ArenaHashMap<'a, u64, (u32, u32)>>()
        };
        if capacity > ranges.capacity() {
            let additional = capacity.saturating_sub(ranges.len());
            ranges.reserve(additional);
        }
    }

    pub fn lookup(&self) -> AstStringLookup<'a> {
        AstStringLookup {
            dedup: self.dedup,
            marker: std::marker::PhantomData,
        }
    }
    pub fn new_in(allocator: &'a Allocator, source: &'a str) -> Self {
        assert!(source.len() <= u32::MAX as usize);
        let dedup: &'a mut ArenaHashMap<'a, ArenaVec<'a, u8>, (u32, u32)> =
            allocator.alloc(ArenaHashMap::new_in(allocator));
        let interned_ranges: &'a mut ArenaHashMap<'a, u64, (u32, u32)> =
            allocator.alloc(ArenaHashMap::new_in(allocator));
        Self {
            source,
            allocator,
            extra: ArenaVec::new_in(allocator),
            dedup: (dedup as *mut ArenaHashMap<'a, ArenaVec<'a, u8>, (u32, u32)>).cast(),
            interned_ranges: (interned_ranges as *mut ArenaHashMap<'a, u64, (u32, u32)>).cast(),
        }
    }

    #[inline]
    pub fn get_utf8(&self, id: AstUtf8) -> &'a str {
        // SAFETY: `AstUtf8` is guaranteed to reference valid UTF-8.
        unsafe { std::str::from_utf8_unchecked(self.get_bytes(id.start, id.end)) }
    }

    #[inline]
    pub fn get_wtf8(&self, id: AstWtf8) -> &'a Wtf8 {
        // SAFETY: `AstWtf8` is guaranteed to reference valid WTF-8.
        unsafe { Wtf8::from_bytes_unchecked(self.get_bytes(id.start, id.end)) }
    }

    #[inline]
    pub fn as_utf8(&self, id: AstWtf8) -> Option<AstUtf8> {
        self.get_wtf8(id).as_str().map(|_| AstUtf8 {
            start: id.start,
            end: id.end,
        })
    }

    #[inline]
    fn get_bytes(&self, start: u32, end: u32) -> &'a [u8] {
        debug_assert!(end >= start);
        if start == end {
            return b"";
        }
        let source_len = self.source.len() as u32;
        if start < source_len {
            debug_assert!(end <= source_len);
            &self.source.as_bytes()[start as usize..end as usize]
        } else {
            let start = (start - source_len) as usize;
            let end = (end - source_len) as usize;
            assert!(start <= end && end <= self.extra.len());
            // SAFETY: The checked range contains initialized bytes. `extra` is
            // private and only appended to, so existing bytes are never changed.
            // ArenaVec's RawVec allocates and copies when growing, without
            // freeing the old buffer; dropping it does not free the buffer either.
            // Thus this allocation and its bytes remain valid for the allocator's
            // lifetime, even after StringPool is dropped or grows. Use the raw
            // buffer pointer so this reference is not derived from a temporary
            // slice borrow of the vector.
            unsafe { std::slice::from_raw_parts(self.extra.as_ptr().add(start), end - start) }
        }
    }

    #[cfg(target_endian = "big")]
    pub(crate) fn initialize_extra(&mut self, bytes: &[u8]) {
        assert!(self.extra.is_empty());
        self.extra.extend_from_slice_copy(bytes);
    }

    /// # Safety
    ///
    /// This pool must only be accessed immutably after initialization.
    #[cfg(target_endian = "little")]
    pub(crate) unsafe fn borrow_extra(&mut self, bytes: &'a [u8]) {
        assert!(self.extra.is_empty());
        // SAFETY: The caller prevents mutation of the borrowed storage.
        self.extra = unsafe { ArenaVec::from_borrowed_slice_in(bytes, self.allocator) };
    }

    pub(crate) fn extra_bytes(&self) -> &[u8] {
        &self.extra
    }

    /// Returns a slice of the source string corresponding to the given start and end offsets.
    /// Actually this function should be unsafe because it does not check if the offsets are valid.
    /// So external crates should be better not use this function directly.
    #[inline]
    pub fn source_slice(&self, start: u32, end: u32) -> AstUtf8 {
        debug_assert!(self.source.get(start as usize..end as usize).is_some());
        AstUtf8 { start, end }
    }

    /// Returns a slice of the source string corresponding to the given start and end offsets.
    /// Actually this function should be unsafe because it does not check if the offsets are valid.
    /// So external crates should be better not use this function directly.
    #[inline]
    pub fn source_slice_wtf8(&self, start: u32, end: u32) -> AstWtf8 {
        debug_assert!(self.source.get(start as usize..end as usize).is_some());
        AstWtf8 { start, end }
    }

    #[inline]
    pub fn add_utf8(&mut self, text: &str) -> AstUtf8 {
        let (start, end) = self.add_bytes(text.as_bytes());
        AstUtf8 { start, end }
    }

    #[inline]
    pub fn add_wtf8(&mut self, text: &Wtf8) -> AstWtf8 {
        let (start, end) = self.add_bytes(text.as_bytes());
        AstWtf8 { start, end }
    }

    /// Canonicalizes a UTF-8 range already stored in this pool.
    #[inline]
    pub fn intern_utf8(&self, text: AstUtf8) -> AstInternedUtf8 {
        let (start, end) = self.intern_range(text.start, text.end);
        AstInternedUtf8 { start, end }
    }

    /// Canonicalizes a WTF-8 range already stored in this pool.
    #[inline]
    pub fn intern_wtf8(&self, text: AstWtf8) -> AstWtf8 {
        let (start, end) = self.intern_range(text.start, text.end);
        AstWtf8 { start, end }
    }

    fn intern_range(&self, start: u32, end: u32) -> (u32, u32) {
        if start == end {
            return (0, 0);
        }
        // SAFETY: this arena allocation outlives the pool; the pool is not
        // Sync and neither content lookup nor insertion accesses this map.
        let ranges = unsafe {
            &mut *self
                .interned_ranges
                .cast::<ArenaHashMap<'a, u64, (u32, u32)>>()
        };
        *ranges
            .entry(AstUtf8 { start, end }.raw())
            .or_insert_with(|| {
                let text = self.get_bytes(start, end);
                // SAFETY: this is the separate arena-owned content table. No
                // references to its entries escape, and allocating its owned
                // key does not access either interning table.
                let dedup = unsafe {
                    &mut *self
                        .dedup
                        .cast::<ArenaHashMap<'a, ArenaVec<'a, u8>, (u32, u32)>>()
                };
                match dedup.entry_ref(text) {
                    EntryRef::Occupied(entry) => *entry.get(),
                    EntryRef::Vacant(entry) => {
                        // Retain the original range. Only the map key needs
                        // stable owned bytes; reuse the lookup hash on insertion.
                        let mut key = ArenaVec::with_capacity_in(text.len(), self.allocator);
                        key.extend_from_slice(text);
                        // SAFETY: key is an exact copy of the queried bytes,
                        // so checking equivalence again would repeat the work.
                        *unsafe { entry.insert_with_key_unchecked(key, (start, end)) }
                    }
                }
            })
    }

    fn add_bytes(&mut self, text: &[u8]) -> (u32, u32) {
        if text.is_empty() {
            return (0, 0);
        }
        if let Some(id) = self.dedup_get(text) {
            return id;
        }
        let start = self
            .source
            .len()
            .checked_add(self.extra.len())
            .expect("string pool offset overflow");
        let end = start
            .checked_add(text.len())
            .expect("string pool offset overflow");
        assert!(end <= u32::MAX as usize, "string pool exceeds u32 range");
        self.extra.extend_from_slice(text);

        let id = (start as u32, end as u32);
        let mut key = ArenaVec::with_capacity_in(text.len(), self.allocator);
        key.extend_from_slice(text);
        self.dedup_insert(key, id);
        id
    }

    #[inline]
    fn dedup_get(&self, text: &[u8]) -> Option<(u32, u32)> {
        // SAFETY: `dedup` is initialized from an arena allocation of exactly
        // this type and remains valid for `'a`.
        unsafe {
            (&*self
                .dedup
                .cast::<ArenaHashMap<'a, ArenaVec<'a, u8>, (u32, u32)>>())
                .get(text)
                .copied()
        }
    }

    #[inline]
    fn dedup_insert(&self, key: ArenaVec<'a, u8>, value: (u32, u32)) {
        // SAFETY: `dedup` is initialized from an arena allocation of exactly
        // this type and remains valid for `'a`. `StringPool` is not `Sync`, and
        // this method keeps the mutable access scoped to one insertion.
        unsafe {
            (&mut *self
                .dedup
                .cast::<ArenaHashMap<'a, ArenaVec<'a, u8>, (u32, u32)>>())
                .insert(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ast;

    #[test]
    fn string_lifetimes() {
        let allocator = Allocator::new();
        let (source, first, large, last, surrogate) = {
            let mut ast = Ast::new("source", &allocator);
            let source = ast.get_utf8(ast.source_slice(0, 6));
            let first = ast.add_utf8("first");
            let first = ast.get_utf8(first);
            let large = ast.add_utf8(&"x".repeat(4096));
            let large = ast.get_utf8(large);
            let copy = ast.add_utf8(first);
            assert_eq!(ast.get_utf8(copy), first);
            let prefix = ast.add_utf8(&first[..2]);
            assert_eq!(ast.get_utf8(prefix), "fi");
            assert_eq!(first, "first");
            assert_eq!(large.len(), 4096);

            // SAFETY: ED A0 80 encodes a lone high surrogate, valid in WTF-8.
            let surrogate = unsafe { Wtf8::from_bytes_unchecked(b"\xed\xa0\x80") };
            let surrogate = ast.add_wtf8(surrogate);
            let surrogate = ast.get_wtf8(surrogate);
            let larger = ast.add_utf8(&"y".repeat(16384));
            assert_eq!(ast.get_utf8(larger).len(), 16384);

            let last = ast.add_utf8("last");
            let last = ast.get_utf8(last);
            (source, first, large, last, surrogate)
        };
        assert_eq!(source, "source");
        assert_eq!(first, "first");
        assert_eq!(large.as_bytes()[4095], b'x');
        assert_eq!(last, "last");
        assert_eq!(surrogate.as_bytes(), b"\xed\xa0\x80");
    }

    #[test]
    fn interns_existing_ranges_without_moving_the_first_value() {
        let allocator = Allocator::new();
        let strings = StringPool::new_in(&allocator, "name name");
        let first = strings.source_slice(0, 4);
        let second = strings.source_slice(5, 9);

        let canonical = strings.intern_utf8(first);
        assert_eq!(canonical.start, 0);
        assert_eq!(canonical.end, 4);
        assert_eq!(strings.extra.len(), 0);
        assert_eq!(strings.intern_utf8(second), canonical);
        assert_eq!(strings.extra.len(), 0);
        assert_eq!(strings.intern_utf8(first), canonical);
        assert_eq!(strings.intern_utf8(second), canonical);
        assert_eq!(strings.intern_wtf8(second.into()), canonical.into());
    }

    #[test]
    fn canonical_lookup_lifetime() {
        let allocator = Allocator::new();
        let (lookup, canonical) = {
            let mut ast = Ast::new("name name", &allocator);
            let lookup = ast.string_lookup();
            assert_eq!(lookup.get(b"name"), None);
            let canonical = ast.intern_utf8(ast.source_slice(5, 9));
            assert_eq!(lookup.get(b"name"), Some(canonical.into()));
            ast.add_utf8(&"x".repeat(4096));
            assert_eq!(ast.intern_utf8(ast.source_slice(0, 4)), canonical);
            (lookup, canonical)
        };
        assert_eq!(lookup.get(b"name"), Some(canonical.into()));
        assert_eq!(lookup.get(b"missing"), None);
        assert_eq!(lookup.get(b""), Some(AstWtf8::EMPTY));
    }
}
