use std::{
    borrow::Cow,
    fmt,
    hash::{Hash, Hasher},
};

use nohash_hasher::{BuildNoHashHasher, IsEnabled};
use rustc_hash::FxHasher;
use swc_next_allocator::{
    Allocator,
    hash_map::{HashMap as ArenaHashMap, RawEntryMut},
    vec::Vec as ArenaVec,
    wtf8::Wtf8,
};

const STORAGE_EXTRA: u32 = 1 << 31;
const GENERATED_NAME: u32 = 1 << 31;
const SUFFIX_SHIFT: u32 = 29;
const FLAT_OFFSET_MASK: u32 = (1 << 31) - 1;
const COMPACT_OFFSET_MASK: u32 = (1 << 29) - 1;
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

macro_rules! define_ast_string {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
        pub struct $name {
            pub(crate) start: u32,
            pub(crate) end: u32,
        }

        impl $name {
            pub const EMPTY: Self = Self { start: 0, end: 0 };

            #[inline]
            pub const fn len(self) -> u32 {
                let is_generated = self.end & GENERATED_NAME != 0;
                let mask = if is_generated {
                    COMPACT_OFFSET_MASK
                } else {
                    FLAT_OFFSET_MASK
                };
                let start = self.start & mask;
                let end = self.end & mask;
                debug_assert!(end >= start);
                end - start + is_generated as u32
            }

            #[inline]
            pub const fn is_empty(self) -> bool {
                self.end & GENERATED_NAME == 0
                    && (self.start & FLAT_OFFSET_MASK) == (self.end & FLAT_OFFSET_MASK)
            }
        }
    };
}

define_ast_string!(AstUtf8);
define_ast_string!(AstInternedUtf8);
define_ast_string!(AstWtf8);

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

/// String-pool state shared by a sequence of generated names with one base.
#[derive(Debug)]
pub struct GeneratedUtf8State {
    base: AstInternedUtf8,
    block: u64,
    prefix: Option<AstUtf8>,
    initialized: bool,
}

impl GeneratedUtf8State {
    #[inline]
    pub fn base(&self) -> AstInternedUtf8 {
        self.base
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

impl AstInternedUtf8 {
    #[inline]
    const fn from_utf8_range(value: AstUtf8) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }

    #[inline]
    const fn try_flat_extra(start: u32, end: u32) -> Option<Self> {
        if start <= end && end <= FLAT_OFFSET_MASK {
            Some(Self {
                start: STORAGE_EXTRA | start,
                end,
            })
        } else {
            None
        }
    }

    #[inline]
    const fn flat_extra(start: u32, end: u32) -> Self {
        match Self::try_flat_extra(start, end) {
            Some(id) => id,
            None => panic!("extra string range exceeds 31-bit representation"),
        }
    }

    #[inline]
    const fn try_generated(prefix: Self, suffix: u8) -> Option<Self> {
        if prefix.is_generated()
            || prefix.start_offset() > COMPACT_OFFSET_MASK
            || prefix.end_offset() > COMPACT_OFFSET_MASK
            || suffix >= 16
        {
            return None;
        }
        let storage = prefix.start & STORAGE_EXTRA;
        Some(Self {
            start: storage | (((suffix >> 2) as u32) << SUFFIX_SHIFT) | prefix.start_offset(),
            end: GENERATED_NAME | (((suffix & 0b11) as u32) << SUFFIX_SHIFT) | prefix.end_offset(),
        })
    }

    #[inline]
    const fn generated(prefix: Self, suffix: u8) -> Self {
        match Self::try_generated(prefix, suffix) {
            Some(id) => id,
            None => panic!("generated name prefix exceeds 29-bit representation"),
        }
    }

    #[inline]
    const fn is_extra(self) -> bool {
        self.start & STORAGE_EXTRA != 0
    }

    #[inline]
    const fn is_generated(self) -> bool {
        self.end & GENERATED_NAME != 0
    }

    #[inline]
    const fn offset_mask(self) -> u32 {
        if self.is_generated() {
            COMPACT_OFFSET_MASK
        } else {
            FLAT_OFFSET_MASK
        }
    }

    #[inline]
    const fn start_offset(self) -> u32 {
        self.start & self.offset_mask()
    }

    #[inline]
    const fn end_offset(self) -> u32 {
        self.end & self.offset_mask()
    }

    #[inline]
    const fn suffix(self) -> Option<u8> {
        if !self.is_generated() {
            return None;
        }
        Some(
            (((self.start >> SUFFIX_SHIFT) & 0b11) << 2 | ((self.end >> SUFFIX_SHIFT) & 0b11))
                as u8,
        )
    }

    #[inline]
    const fn generated_prefix(self) -> Self {
        debug_assert!(self.is_generated());
        Self {
            start: (self.start & STORAGE_EXTRA) | self.start_offset(),
            end: self.end_offset(),
        }
    }

    #[inline]
    const fn can_be_compact_prefix(self) -> bool {
        !self.is_generated()
            && self.start_offset() <= COMPACT_OFFSET_MASK
            && self.end_offset() <= COMPACT_OFFSET_MASK
    }
}

type InternBuildHasher = BuildNoHashHasher<InternEntry>;

#[derive(Clone, Copy, Debug)]
struct InternEntry {
    hash: u64,
    id: AstInternedUtf8,
}

impl InternEntry {
    #[inline]
    fn new(hash: u64, id: AstInternedUtf8, used_as_name: bool) -> Self {
        debug_assert!(!id.is_generated());
        Self {
            hash,
            id: AstInternedUtf8 {
                start: id.start,
                end: id.end | if used_as_name { GENERATED_NAME } else { 0 },
            },
        }
    }

    #[inline]
    fn id(self) -> AstInternedUtf8 {
        AstInternedUtf8 {
            start: self.id.start,
            end: self.id.end & !GENERATED_NAME,
        }
    }

    #[inline]
    fn used_as_name(self) -> bool {
        self.id.end & GENERATED_NAME != 0
    }

    #[inline]
    fn reserve_name(&mut self) -> bool {
        let was_free = !self.used_as_name();
        self.id.end |= GENERATED_NAME;
        was_free
    }
}

impl Hash for InternEntry {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl PartialEq for InternEntry {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.id() == other.id()
    }
}

impl Eq for InternEntry {}

impl IsEnabled for InternEntry {}

#[derive(Clone, Copy)]
struct LogicalBytes<'a> {
    prefix: &'a [u8],
    suffix: Option<u8>,
}

impl LogicalBytes<'_> {
    #[inline]
    fn len(self) -> usize {
        self.prefix.len() + usize::from(self.suffix.is_some())
    }

    #[inline]
    fn eq(self, other: Self) -> bool {
        match (self.suffix, other.suffix) {
            (None, None) => self.prefix == other.prefix,
            (Some(left), Some(right)) => left == right && self.prefix == other.prefix,
            (Some(suffix), None) => other
                .prefix
                .split_last()
                .is_some_and(|(last, prefix)| *last == suffix && prefix == self.prefix),
            (None, Some(suffix)) => self
                .prefix
                .split_last()
                .is_some_and(|(last, prefix)| *last == suffix && prefix == other.prefix),
        }
    }

    #[inline]
    fn eq_slice(self, other: &[u8]) -> bool {
        match self.suffix {
            None => self.prefix == other,
            Some(suffix) => other
                .split_last()
                .is_some_and(|(last, prefix)| *last == suffix && prefix == self.prefix),
        }
    }

    #[inline]
    fn hash(self) -> u64 {
        debug_assert!(self.suffix.is_none());
        hash_bytes(self.prefix)
    }
}

#[inline]
fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = FxHasher::default();
    hasher.write(bytes);
    hasher.finish()
}

#[inline]
fn hash_with_scratch(buffer: &mut [u8], start: usize, end: usize, suffix: u8) -> u64 {
    debug_assert!(start <= end && end < buffer.len());
    let saved = buffer[end];
    buffer[end] = suffix;
    let hash = hash_bytes(&buffer[start..=end]);
    buffer[end] = saved;
    hash
}

/// A non-allocating logical view of a UTF-8 handle.
#[derive(Clone, Copy, Debug)]
pub struct Utf8Chunks<'a> {
    prefix: &'a str,
    suffix: Option<u8>,
}

impl<'a> Utf8Chunks<'a> {
    #[inline]
    pub fn prefix(self) -> &'a str {
        self.prefix
    }

    #[inline]
    pub fn suffix(self) -> Option<u8> {
        self.suffix
    }

    #[inline]
    pub fn bytes(self) -> impl Iterator<Item = u8> + 'a {
        self.prefix.bytes().chain(self.suffix)
    }

    #[inline]
    pub fn contains_byte(self, byte: u8) -> bool {
        self.prefix.as_bytes().contains(&byte) || self.suffix == Some(byte)
    }

    #[inline]
    pub fn first_byte(self) -> Option<u8> {
        self.prefix.as_bytes().first().copied().or(self.suffix)
    }

    pub fn to_cow(self) -> Cow<'a, str> {
        let Some(suffix) = self.suffix else {
            return Cow::Borrowed(self.prefix);
        };
        let mut output = String::with_capacity(self.prefix.len() + 1);
        output.push_str(self.prefix);
        output.push(char::from(suffix));
        Cow::Owned(output)
    }
}

impl fmt::Display for Utf8Chunks<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.prefix)?;
        if let Some(suffix) = self.suffix {
            formatter.write_str(unsafe {
                std::str::from_utf8_unchecked(std::slice::from_ref(&suffix))
            })?;
        }
        Ok(())
    }
}

pub(crate) struct StringPool<'a> {
    pub(crate) source: &'a str,
    pub(crate) extra: ArenaVec<'a, u8>,
    dedup: ArenaHashMap<'a, InternEntry, (), InternBuildHasher>,
    generated_names: ArenaHashMap<'a, AstInternedUtf8, u32>,
    // Compact UTF-8 is segmented, so WTF-8 needs one flat alias for the same content.
    generated_wtf8_aliases: ArenaHashMap<'a, AstInternedUtf8, AstWtf8>,
    compact_prefixes: ArenaHashMap<'a, AstInternedUtf8, AstInternedUtf8>,
}

impl<'a> StringPool<'a> {
    pub fn new_in(allocator: &'a Allocator, source: &'a str) -> Self {
        assert!(
            source.len() <= FLAT_OFFSET_MASK as usize,
            "source exceeds 2 GiB string-handle range"
        );
        let mut extra = ArenaVec::new_in(allocator);
        extra.push(0);
        Self {
            source,
            extra,
            dedup: ArenaHashMap::with_hasher_in(InternBuildHasher::default(), allocator),
            generated_names: ArenaHashMap::new_in(allocator),
            generated_wtf8_aliases: ArenaHashMap::new_in(allocator),
            compact_prefixes: ArenaHashMap::new_in(allocator),
        }
    }

    #[inline]
    pub fn get_flat_utf8(&self, id: AstUtf8) -> &str {
        let id = AstInternedUtf8::from_utf8_range(id);
        debug_assert!(!id.is_generated(), "compact UTF-8 requires a logical view");
        // SAFETY: `AstUtf8` is guaranteed to reference valid UTF-8.
        unsafe { std::str::from_utf8_unchecked(self.get_flat_bytes(id)) }
    }

    #[inline]
    /// Materializes a logical UTF-8 handle when its bytes are segmented.
    ///
    /// Test-only convenience API. Production callers should compare or write
    /// handles through the string pool without materializing them.
    pub fn get_utf8(&self, id: AstUtf8) -> Cow<'_, str> {
        self.utf8_chunks(id).to_cow()
    }

    #[inline]
    pub fn utf8_chunks(&self, id: AstUtf8) -> Utf8Chunks<'_> {
        let bytes = self.logical_bytes(AstInternedUtf8::from_utf8_range(id));
        // SAFETY: `AstUtf8` is valid UTF-8 and the optional suffix is ASCII.
        let prefix = unsafe { std::str::from_utf8_unchecked(bytes.prefix) };
        Utf8Chunks {
            prefix,
            suffix: bytes.suffix,
        }
    }

    #[inline]
    pub fn utf8_eq(&self, left: AstUtf8, right: AstUtf8) -> bool {
        self.logical_bytes(AstInternedUtf8::from_utf8_range(left))
            .eq(self.logical_bytes(AstInternedUtf8::from_utf8_range(right)))
    }

    #[inline]
    pub fn utf8_eq_bytes(&self, id: AstUtf8, bytes: &[u8]) -> bool {
        self.logical_bytes(AstInternedUtf8::from_utf8_range(id))
            .eq_slice(bytes)
    }

    #[inline]
    pub fn utf8_eq_wtf8(&self, utf8: AstUtf8, wtf8: AstWtf8) -> bool {
        self.logical_bytes(AstInternedUtf8::from_utf8_range(utf8))
            .eq_slice(self.get_wtf8(wtf8).as_bytes())
    }

    #[inline]
    pub fn get_wtf8(&self, id: AstWtf8) -> &Wtf8 {
        // SAFETY: `AstWtf8` is guaranteed to reference valid WTF-8.
        unsafe { Wtf8::from_bytes_unchecked(self.get_wtf8_bytes(id)) }
    }

    #[inline]
    pub fn as_utf8(&self, id: AstWtf8) -> Option<AstUtf8> {
        self.get_wtf8(id).as_str().map(|_| AstUtf8 {
            start: id.start,
            end: id.end,
        })
    }

    #[inline]
    fn get_flat_bytes(&self, id: AstInternedUtf8) -> &[u8] {
        debug_assert!(!id.is_generated());
        self.get_storage_bytes(id)
    }

    #[inline]
    fn get_wtf8_bytes(&self, id: AstWtf8) -> &[u8] {
        wtf8_bytes_from(self.source.as_bytes(), &self.extra, id)
    }

    #[inline]
    fn logical_bytes(&self, id: AstInternedUtf8) -> LogicalBytes<'_> {
        LogicalBytes {
            prefix: self.get_storage_bytes(id),
            suffix: id.suffix().map(|suffix| HEX_DIGITS[usize::from(suffix)]),
        }
    }

    /// Hashes an interned handle as one contiguous byte slice. Compact generated
    /// names use the sentinel byte after their extra-storage prefix as scratch
    /// storage for the final hexadecimal digit.
    #[inline]
    fn hash_interned(&mut self, id: AstInternedUtf8) -> u64 {
        let Some(suffix) = id.suffix() else {
            return hash_bytes(self.get_storage_bytes(id));
        };

        let start = id.start_offset() as usize;
        let end = id.end_offset() as usize;
        let suffix = HEX_DIGITS[usize::from(suffix)];
        debug_assert!(id.is_extra(), "compact prefixes live in extra storage");
        hash_with_scratch(&mut self.extra, start, end, suffix)
    }

    #[inline]
    fn get_storage_bytes(&self, id: AstInternedUtf8) -> &[u8] {
        let start = id.start_offset() as usize;
        let end = id.end_offset() as usize;
        debug_assert!(end >= start);
        if id.is_extra() {
            &self.extra[start..end]
        } else {
            &self.source.as_bytes()[start..end]
        }
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
    pub fn add_utf8(&mut self, text: &str) -> AstInternedUtf8 {
        self.intern_utf8_bytes(text.as_bytes())
    }

    #[inline]
    pub fn add_wtf8(&mut self, text: &Wtf8) -> AstWtf8 {
        let parts = LogicalBytes {
            prefix: text.as_bytes(),
            suffix: None,
        };
        let hash = parts.hash();
        if let Some(entry) = self.find_intern_entry(parts, hash) {
            let id = entry.id();
            return AstWtf8 {
                start: id.start,
                end: id.end,
            };
        }
        if let Some(generated) = self.generated_utf8_for_flat_bytes(text.as_bytes()) {
            return self.add_generated_wtf8_alias(generated, text.as_bytes());
        }
        let interned = self.append_extra(text.as_bytes());
        let id = AstWtf8 {
            start: interned.start,
            end: interned.end,
        };
        self.insert_utf8(hash, interned, false);
        id
    }

    /// Reinterprets a flat UTF-8 handle as WTF-8 without copying. Compact
    /// handles are materialized without changing generated-name reservations.
    pub fn utf8_to_wtf8(&mut self, text: AstUtf8) -> AstWtf8 {
        let id = AstInternedUtf8::from_utf8_range(text);
        if !id.is_generated() {
            return AstWtf8 {
                start: id.start,
                end: id.end,
            };
        }
        let text = self.utf8_chunks(text).to_cow().into_owned();
        self.add_wtf8(Wtf8::from_str(&text))
    }

    /// Canonicalizes a UTF-8 range already stored in this pool.
    #[inline]
    pub fn intern_utf8(&mut self, text: AstUtf8) -> AstInternedUtf8 {
        self.intern_utf8_id(AstInternedUtf8::from_utf8_range(text), false)
    }

    /// Canonicalizes an identifier and records it in the program-wide UID
    /// collision domain during the same interner lookup.
    #[inline]
    pub fn intern_name(&mut self, text: AstUtf8) -> AstInternedUtf8 {
        let string = AstInternedUtf8::from_utf8_range(text);
        if string.is_generated() || !self.generated_names.is_empty() {
            return self.intern_utf8_id(string, true);
        }

        let source = self.source.as_bytes();
        let parts = logical_bytes_from(source, &self.extra, string);
        let hash = parts.hash();
        let (entry, ()) = self
            .dedup
            .raw_entry_mut()
            .from_hash(hash, |entry| {
                entry.hash == hash && logical_bytes_from(source, &self.extra, entry.id()).eq(parts)
            })
            .or_insert(InternEntry::new(hash, string, true), ());
        entry.reserve_name();
        entry.id()
    }

    /// Canonicalizes a WTF-8 range already stored in this pool.
    #[inline]
    pub fn intern_wtf8(&mut self, text: AstWtf8) -> AstWtf8 {
        let parts = LogicalBytes {
            prefix: self.get_wtf8_bytes(text),
            suffix: None,
        };
        let hash = parts.hash();
        if let Some(entry) = self.find_intern_entry(parts, hash) {
            let id = entry.id();
            return AstWtf8 {
                start: id.start,
                end: id.end,
            };
        }
        if let Some(generated) = self.generated_utf8_for_flat_bytes(parts.prefix) {
            if let Some(&alias) = self.generated_wtf8_aliases.get(&generated) {
                return alias;
            }
            self.generated_wtf8_aliases.insert(generated, text);
            return text;
        }
        let id = AstInternedUtf8 {
            start: text.start,
            end: text.end,
        };
        self.insert_utf8(hash, id, false);
        text
    }

    /// Creates string-pool state for a generated-name sequence.
    pub fn generated_utf8_state(&mut self, base: &str) -> GeneratedUtf8State {
        GeneratedUtf8State {
            base: self.add_utf8(base),
            block: 0,
            prefix: None,
            initialized: false,
        }
    }

    /// Reserves a generated name in the pool's program-wide UID collision
    /// domain. Returns the canonical handle and whether the name was free.
    pub fn reserve_generated_name_from_state(
        &mut self,
        state: &mut GeneratedUtf8State,
        suffix: u64,
    ) -> (AstInternedUtf8, bool) {
        let block = suffix >> 4;
        if !state.initialized || state.block != block {
            state.block = block;
            state.prefix = self.generated_name_prefix(state.base, block);
            state.initialized = true;
        }
        self.reserve_generated_name(state.base, state.prefix, suffix)
    }

    /// Builds or reuses the physical prefix for one block of 16 generated names.
    /// Returns `None` when the prefix cannot fit in the compact 29-bit range.
    pub fn generated_name_prefix(&mut self, base: AstInternedUtf8, block: u64) -> Option<AstUtf8> {
        if block == 0 && self.can_hash_compact_prefix(base) {
            return Some(base.into());
        }

        let mut bytes = Vec::with_capacity(self.logical_bytes(base).len() + 16);
        let logical_base = self.logical_bytes(base);
        bytes.extend_from_slice(logical_base.prefix);
        if let Some(suffix) = logical_base.suffix {
            bytes.push(suffix);
        }
        if block != 0 {
            append_hex(&mut bytes, block);
        }
        self.compact_prefix_for_bytes(&bytes).map(Into::into)
    }

    fn reserve_generated_name(
        &mut self,
        base: AstInternedUtf8,
        prefix: Option<AstUtf8>,
        suffix: u64,
    ) -> (AstInternedUtf8, bool) {
        let nibble = (suffix & 0xf) as u8;
        if let Some(prefix) = prefix {
            let id = AstInternedUtf8::generated(AstInternedUtf8::from_utf8_range(prefix), nibble);
            let hash = self.hash_interned(id);
            let source = self.source.as_bytes();
            let parts = logical_bytes_from(source, &self.extra, id);
            if let Some(entry) =
                Self::find_intern_entry_mut(&mut self.dedup, source, &self.extra, parts, hash)
            {
                let was_free = entry.reserve_name();
                return (entry.id(), was_free);
            }

            let prefix = AstInternedUtf8::from_utf8_range(prefix);
            let bit = 1_u32 << nibble;
            let used_bit = bit << 16;
            let flags = self.generated_names.entry(prefix).or_insert(0);
            let was_free = *flags & used_bit == 0;
            *flags |= bit | used_bit;
            return (AstInternedUtf8::generated(prefix, nibble), was_free);
        }

        let mut bytes = Vec::with_capacity(self.logical_bytes(base).len() + 16);
        let logical_base = self.logical_bytes(base);
        bytes.extend_from_slice(logical_base.prefix);
        if let Some(base_suffix) = logical_base.suffix {
            bytes.push(base_suffix);
        }
        append_hex(&mut bytes, suffix);
        let name = self.intern_utf8_bytes(&bytes);
        let was_free = self.reserve_name(name);
        (name, was_free)
    }

    /// Marks an interned string as occupied in the program-wide generated-name
    /// collision domain. Returns whether it was previously free.
    pub fn reserve_name(&mut self, name: AstInternedUtf8) -> bool {
        if let Some(nibble) = name.suffix() {
            let bit = 1_u32 << nibble;
            let used_bit = bit << 16;
            let flags = self
                .generated_names
                .entry(name.generated_prefix())
                .or_insert(0);
            let was_free = *flags & used_bit == 0;
            *flags |= bit | used_bit;
            return was_free;
        }

        let source = self.source.as_bytes();
        let parts = logical_bytes_from(source, &self.extra, name);
        let hash = parts.hash();
        Self::find_intern_entry_mut(&mut self.dedup, source, &self.extra, parts, hash)
            .expect("interned UTF-8 has a dedup entry")
            .reserve_name()
    }

    fn compact_prefix_for_bytes(&mut self, bytes: &[u8]) -> Option<AstInternedUtf8> {
        let parts = LogicalBytes {
            prefix: bytes,
            suffix: None,
        };
        let hash = parts.hash();
        if let Some(entry) = self.find_intern_entry(parts, hash) {
            let canonical = entry.id();
            if self.can_hash_compact_prefix(canonical) {
                return Some(canonical);
            }
            if let Some(&prefix) = self.compact_prefixes.get(&canonical) {
                return Some(prefix);
            }

            compact_extra_range(self.extra_content_len(), bytes.len())?;
            let id = self.append_extra(bytes);
            self.compact_prefixes.insert(canonical, id);
            return Some(id);
        }

        compact_extra_range(self.extra_content_len(), bytes.len())?;
        let id = self.append_extra(bytes);
        self.insert_utf8(hash, id, false);
        Some(id)
    }

    fn intern_utf8_bytes(&mut self, text: &[u8]) -> AstInternedUtf8 {
        let parts = LogicalBytes {
            prefix: text,
            suffix: None,
        };
        let hash = parts.hash();
        if let Some(entry) = self.find_intern_entry(parts, hash) {
            return entry.id();
        }
        if let Some(id) = self.generated_utf8_for_flat_bytes(text) {
            return id;
        }
        let id = self.append_extra(text);
        self.insert_utf8(hash, id, false)
    }

    fn next_extra_id(&self, len: usize) -> AstInternedUtf8 {
        if len == 0 {
            return AstInternedUtf8::EMPTY;
        }
        let start = self.extra_content_len();
        let end = start.checked_add(len).expect("string pool offset overflow");
        assert!(
            end <= FLAT_OFFSET_MASK as usize,
            "extra strings exceed 2 GiB string-handle range"
        );
        AstInternedUtf8::flat_extra(start as u32, end as u32)
    }

    fn append_extra(&mut self, text: &[u8]) -> AstInternedUtf8 {
        if text.is_empty() {
            return AstInternedUtf8::EMPTY;
        }
        let id = self.next_extra_id(text.len());
        self.extra.pop();
        self.extra.extend_from_slice(text);
        self.extra.push(0);
        id
    }

    fn add_generated_wtf8_alias(&mut self, generated: AstInternedUtf8, text: &[u8]) -> AstWtf8 {
        if let Some(&alias) = self.generated_wtf8_aliases.get(&generated) {
            return alias;
        }
        let id = self.append_extra(text);
        let alias = AstWtf8 {
            start: id.start,
            end: id.end,
        };
        self.generated_wtf8_aliases.insert(generated, alias);
        alias
    }

    #[inline]
    fn extra_content_len(&self) -> usize {
        debug_assert!(!self.extra.is_empty());
        self.extra.len() - 1
    }

    fn find_intern_entry(&self, parts: LogicalBytes<'_>, hash: u64) -> Option<&InternEntry> {
        let source = self.source.as_bytes();
        self.dedup
            .raw_entry()
            .from_hash(hash, |entry| {
                entry.hash == hash && logical_bytes_from(source, &self.extra, entry.id()).eq(parts)
            })
            .map(|(entry, ())| entry)
    }

    fn find_intern_entry_mut<'map>(
        dedup: &'map mut ArenaHashMap<'a, InternEntry, (), InternBuildHasher>,
        source: &[u8],
        extra: &[u8],
        parts: LogicalBytes<'_>,
        hash: u64,
    ) -> Option<&'map mut InternEntry> {
        match dedup.raw_entry_mut().from_hash(hash, |entry| {
            entry.hash == hash && logical_bytes_from(source, extra, entry.id()).eq(parts)
        }) {
            RawEntryMut::Occupied(entry) => Some(entry.into_key_value().0),
            RawEntryMut::Vacant(_) => None,
        }
    }

    fn generated_utf8_for_flat_bytes(&self, bytes: &[u8]) -> Option<AstInternedUtf8> {
        if self.generated_names.is_empty() {
            return None;
        }
        let (&suffix, prefix) = bytes.split_last()?;
        let nibble = hex_nibble(suffix)?;
        let parts = LogicalBytes {
            prefix,
            suffix: None,
        };
        let canonical = self.find_intern_entry(parts, parts.hash())?.id();
        let physical = if self.can_hash_compact_prefix(canonical) {
            canonical
        } else {
            *self.compact_prefixes.get(&canonical)?
        };
        if self.generated_names.get(&physical).copied().unwrap_or(0) & (1_u32 << nibble) == 0 {
            return None;
        }
        Some(AstInternedUtf8::generated(physical, nibble))
    }

    #[inline]
    fn can_hash_compact_prefix(&self, id: AstInternedUtf8) -> bool {
        id.can_be_compact_prefix() && id.is_extra()
    }

    fn intern_utf8_id(&mut self, string: AstInternedUtf8, used_as_name: bool) -> AstInternedUtf8 {
        let hash = self.hash_interned(string);
        let source = self.source.as_bytes();
        let parts = logical_bytes_from(source, &self.extra, string);
        if let Some(entry) =
            Self::find_intern_entry_mut(&mut self.dedup, source, &self.extra, parts, hash)
        {
            if used_as_name {
                entry.reserve_name();
            }
            return entry.id();
        }
        if string.is_generated() {
            let nibble = string.suffix().unwrap();
            let bit = 1_u32 << nibble;
            let flags = self
                .generated_names
                .entry(string.generated_prefix())
                .or_insert(0);
            *flags |= bit | if used_as_name { bit << 16 } else { 0 };
            return string;
        }
        if let Some(id) = self.generated_utf8_for_flat_bytes(parts.prefix) {
            if used_as_name {
                self.reserve_name(id);
            }
            return id;
        }
        self.insert_utf8(hash, string, used_as_name)
    }

    #[inline]
    fn insert_utf8(
        &mut self,
        hash: u64,
        string: AstInternedUtf8,
        used_as_name: bool,
    ) -> AstInternedUtf8 {
        self.dedup
            .insert(InternEntry::new(hash, string, used_as_name), ());
        string
    }
}

#[inline]
fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[inline]
fn compact_extra_range(start: usize, len: usize) -> Option<(u32, u32)> {
    let end = start.checked_add(len)?;
    if end > COMPACT_OFFSET_MASK as usize {
        return None;
    }
    Some((start as u32, end as u32))
}

#[inline]
fn logical_bytes_from<'a>(
    source: &'a [u8],
    extra: &'a [u8],
    id: AstInternedUtf8,
) -> LogicalBytes<'a> {
    let start = id.start_offset() as usize;
    let end = id.end_offset() as usize;
    let prefix = if id.is_extra() {
        &extra[start..end]
    } else {
        &source[start..end]
    };
    LogicalBytes {
        prefix,
        suffix: id.suffix().map(|suffix| HEX_DIGITS[usize::from(suffix)]),
    }
}

#[inline]
fn wtf8_bytes_from<'a>(source: &'a [u8], extra: &'a [u8], id: AstWtf8) -> &'a [u8] {
    let start = (id.start & FLAT_OFFSET_MASK) as usize;
    let end = id.end as usize;
    debug_assert!(end >= start);
    if id.start & STORAGE_EXTRA != 0 {
        &extra[start..end]
    } else {
        &source[start..end]
    }
}

fn append_hex(output: &mut Vec<u8>, mut value: u64) {
    let mut digits = [0_u8; 16];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = HEX_DIGITS[(value & 0xf) as usize];
        value >>= 4;
        if value == 0 {
            break;
        }
    }
    output.extend_from_slice(&digits[start..]);
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, mem::size_of};

    use swc_next_allocator::Allocator;

    use super::{
        AstInternedUtf8, AstUtf8, AstWtf8, COMPACT_OFFSET_MASK, FLAT_OFFSET_MASK, InternEntry,
        StringPool, compact_extra_range,
    };

    fn string_pool<'a>(allocator: &'a Allocator, source: &'a str) -> StringPool<'a> {
        StringPool::new_in(allocator, source)
    }

    #[test]
    fn string_handles_remain_eight_bytes() {
        assert_eq!(size_of::<AstUtf8>(), 8);
        assert_eq!(size_of::<AstInternedUtf8>(), 8);
        assert_eq!(size_of::<AstWtf8>(), 8);
        assert_eq!(size_of::<InternEntry>(), 16);
    }

    #[test]
    fn codec_round_trips_flat_and_generated_boundaries() {
        let source = AstInternedUtf8 {
            start: 0,
            end: FLAT_OFFSET_MASK,
        };
        assert!(!source.is_extra());
        assert_eq!(source.start_offset(), 0);
        assert_eq!(source.end_offset(), FLAT_OFFSET_MASK);

        let extra = AstInternedUtf8::try_flat_extra(0, FLAT_OFFSET_MASK).unwrap();
        assert!(extra.is_extra());
        assert_eq!(extra.start_offset(), 0);
        assert_eq!(extra.end_offset(), FLAT_OFFSET_MASK);

        let source_prefix = AstInternedUtf8 {
            start: 0,
            end: COMPACT_OFFSET_MASK,
        };
        let extra_prefix = AstInternedUtf8::try_flat_extra(0, COMPACT_OFFSET_MASK).unwrap();
        for suffix in 0..16 {
            let source_name = AstInternedUtf8::try_generated(source_prefix, suffix).unwrap();
            assert!(!source_name.is_extra());
            assert_eq!(source_name.start_offset(), 0);
            assert_eq!(source_name.end_offset(), COMPACT_OFFSET_MASK);
            assert_eq!(source_name.suffix(), Some(suffix));

            let extra_name = AstInternedUtf8::try_generated(extra_prefix, suffix).unwrap();
            assert!(extra_name.is_extra());
            assert_eq!(extra_name.start_offset(), 0);
            assert_eq!(extra_name.end_offset(), COMPACT_OFFSET_MASK);
            assert_eq!(extra_name.suffix(), Some(suffix));
        }

        assert!(AstInternedUtf8::try_flat_extra(1, 0).is_none());
        assert!(AstInternedUtf8::try_flat_extra(0, FLAT_OFFSET_MASK + 1).is_none());
        assert!(
            AstInternedUtf8::try_generated(
                AstInternedUtf8 {
                    start: COMPACT_OFFSET_MASK + 1,
                    end: COMPACT_OFFSET_MASK + 1,
                },
                0,
            )
            .is_none()
        );
        assert!(AstInternedUtf8::try_generated(source_prefix, 16).is_none());
    }

    #[test]
    fn compact_extra_boundary_selects_flat_fallback_without_allocating() {
        assert_eq!(
            compact_extra_range(COMPACT_OFFSET_MASK as usize - 1, 1),
            Some((COMPACT_OFFSET_MASK - 1, COMPACT_OFFSET_MASK))
        );
        assert_eq!(compact_extra_range(COMPACT_OFFSET_MASK as usize, 1), None);
        assert_eq!(compact_extra_range(usize::MAX, 1), None);
    }

    #[test]
    fn interns_existing_ranges_without_copying_to_extra() {
        let allocator = Allocator::new();
        let mut strings = string_pool(&allocator, "name name");
        let first = strings.source_slice(0, 4);
        let second = strings.source_slice(5, 9);

        let canonical = strings.intern_utf8(first);
        assert!(!canonical.is_extra());
        assert_eq!(canonical.start_offset(), 0);
        assert_eq!(canonical.end_offset(), 4);
        assert_eq!(strings.extra.as_slice(), b"\0");
        assert_eq!(strings.intern_utf8(second), canonical);
        assert_eq!(strings.extra.as_slice(), b"\0");
        assert_eq!(strings.dedup.len(), 1);
    }

    #[test]
    fn added_strings_reuse_the_extra_range() {
        let allocator = Allocator::new();
        let mut strings = string_pool(&allocator, "");
        let first = strings.add_utf8("generated");
        let second = strings.add_utf8("generated");

        assert_eq!(first, second);
        assert_eq!(strings.extra.as_slice(), b"generated\0");
        assert_eq!(strings.dedup.len(), 1);
    }

    #[test]
    fn utf8_and_wtf8_share_one_dedup_entry() {
        let allocator = Allocator::new();
        let mut strings = string_pool(&allocator, "");
        let utf8 = strings.add_utf8("shared");
        // SAFETY: ASCII is valid WTF-8.
        let text = unsafe { swc_next_allocator::wtf8::Wtf8::from_bytes_unchecked(b"shared") };
        let wtf8 = strings.add_wtf8(text);

        assert_eq!((utf8.start, utf8.end), (wtf8.start, wtf8.end));
        assert_eq!(strings.extra.as_slice(), b"shared\0");
        assert_eq!(strings.dedup.len(), 1);
    }

    #[test]
    fn flat_extra_uses_independent_offsets() {
        let allocator = Allocator::new();
        let mut strings = string_pool(&allocator, "source");
        let added = strings.add_utf8("extra");
        let id = added;

        assert!(id.is_extra());
        assert_eq!(id.start_offset(), 0);
        assert_eq!(id.end_offset(), 5);
        assert_eq!(strings.get_flat_utf8(added.into()), "extra");
    }

    #[test]
    fn generated_names_share_one_prefix_per_block() {
        let allocator = Allocator::new();
        let mut strings = string_pool(&allocator, "");
        let base = strings.add_utf8("_temp");
        let prefix = strings.generated_name_prefix(base, 1).unwrap();
        let extra_after_prefix = strings.extra.len();
        let ten = strings.reserve_generated_name(base, Some(prefix), 0x10).0;
        let one_f = strings.reserve_generated_name(base, Some(prefix), 0x1f).0;

        assert!(strings.utf8_eq_bytes(ten.into(), b"_temp10"));
        assert!(strings.utf8_eq_bytes(one_f.into(), b"_temp1f"));
        assert_eq!(strings.extra.len(), extra_after_prefix);
        assert_eq!(ten.len(), 7);
    }

    #[test]
    fn generated_name_hash_restores_the_following_byte() {
        let allocator = Allocator::new();
        let mut strings = string_pool(&allocator, "");
        let base = strings.add_utf8("_temp");
        let following = strings.add_utf8("next");
        let prefix = strings.generated_name_prefix(base, 0).unwrap();

        let generated = strings.reserve_generated_name(base, Some(prefix), 0).0;

        assert!(strings.utf8_eq_bytes(generated.into(), b"_temp0"));
        assert!(strings.utf8_eq_bytes(following.into(), b"next"));
        assert_eq!(strings.extra.as_slice(), b"_tempnext\0");
    }

    #[test]
    fn generated_name_hash_does_not_mutate_source() {
        let allocator = Allocator::new();
        let mut strings = string_pool(&allocator, "_tempnext");
        let base = strings.intern_utf8(strings.source_slice(0, 5));
        let following = strings.source_slice(5, 9);
        let prefix = strings.generated_name_prefix(base, 0).unwrap();

        let generated = strings.reserve_generated_name(base, Some(prefix), 0).0;

        assert!(strings.utf8_eq_bytes(generated.into(), b"_temp0"));
        assert_eq!(strings.get_flat_utf8(following), "next");
        assert_eq!(strings.source, "_tempnext");
        assert_eq!(strings.extra.as_slice(), b"_temp\0");
    }

    #[test]
    fn generated_name_prefix_at_source_end_is_copied_once() {
        let allocator = Allocator::new();
        let mut strings = string_pool(&allocator, "_temp");
        let base = strings.intern_utf8(strings.source_slice(0, 5));

        let first = strings.generated_name_prefix(base, 0).unwrap();
        let second = strings.generated_name_prefix(base, 0).unwrap();
        let generated = strings.reserve_generated_name(base, Some(first), 0).0;

        assert_eq!(first, second);
        assert!(AstInternedUtf8::from_utf8_range(first).is_extra());
        assert!(strings.utf8_eq_bytes(generated.into(), b"_temp0"));
        assert_eq!(strings.source, "_temp");
        assert_eq!(strings.extra.as_slice(), b"_temp\0");
    }

    #[test]
    fn generated_names_do_not_add_full_dedup_entries() {
        let allocator = Allocator::new();
        let mut strings = string_pool(&allocator, "");
        let mut state = strings.generated_utf8_state("_temp");
        strings.reserve_name(state.base());
        for suffix in 2..=1_001 {
            let (_, was_free) = strings.reserve_generated_name_from_state(&mut state, suffix);
            assert!(was_free);
        }

        // One entry for `_temp`, then one shared prefix for each later block
        // of 16 generated names. There is no entry per complete generated name.
        assert_eq!(strings.dedup.len(), 63);
        assert_eq!(strings.generated_names.len(), 63);
        assert!(strings.compact_prefixes.is_empty());
    }

    #[test]
    fn wtf8_operations_preserve_compact_generated_name_reservation() {
        let allocator = Allocator::new();
        let mut strings = string_pool(&allocator, "");
        let mut state = strings.generated_utf8_state("_temp");
        let (generated, was_free) = strings.reserve_generated_name_from_state(&mut state, 0);
        assert!(was_free);

        let converted = strings.utf8_to_wtf8(generated.into());
        assert_eq!(strings.get_wtf8(converted).as_bytes(), b"_temp0");

        let added = strings.add_wtf8(swc_next_allocator::wtf8::Wtf8::from_str("_temp0"));
        assert_eq!(strings.get_wtf8(added).as_bytes(), b"_temp0");
        assert_eq!(added, converted);
        assert_eq!(strings.intern_wtf8(added), added);

        let (reserved_again, was_free) = strings.reserve_generated_name_from_state(&mut state, 0);
        assert_eq!(reserved_again, generated);
        assert!(!was_free);
        assert_eq!(strings.dedup.len(), 1);
        assert_eq!(strings.generated_wtf8_aliases.len(), 1);
    }

    #[test]
    fn gets_or_writes_logical_utf8() {
        let allocator = Allocator::new();
        let mut strings = string_pool(&allocator, "source");
        let source = strings.source_slice(0, 6);
        assert_eq!(strings.get_utf8(source), Cow::Borrowed("source"));

        let base = strings.add_utf8("_temp");
        let prefix = strings.generated_name_prefix(base, 1).unwrap();
        let generated = strings.reserve_generated_name(base, Some(prefix), 0x10).0;
        assert!(matches!(
            strings.get_utf8(generated.into()),
            Cow::Owned(value) if value == "_temp10"
        ));

        let chunks = strings.utf8_chunks(generated.into());
        assert_eq!(chunks.to_string(), "_temp10");
        assert_eq!(chunks.prefix(), "_temp1");
        assert_eq!(chunks.suffix(), Some(b'0'));
    }

    #[test]
    fn generated_name_collides_with_existing_flat_name() {
        let allocator = Allocator::new();
        let mut strings = string_pool(&allocator, "_temp10");
        let existing = strings.intern_utf8(strings.source_slice(0, 7));
        let base = strings.add_utf8("_temp");
        let prefix = strings.generated_name_prefix(base, 1).unwrap();
        let generated = strings.reserve_generated_name(base, Some(prefix), 0x10).0;

        assert_eq!(generated, existing);
    }

    #[test]
    fn flat_name_reuses_an_existing_compact_generated_name() {
        let allocator = Allocator::new();
        let mut strings = string_pool(&allocator, "_temp10");
        let base = strings.add_utf8("_temp");
        let prefix = strings.generated_name_prefix(base, 1).unwrap();
        let generated = strings.reserve_generated_name(base, Some(prefix), 0x10).0;
        let flat = strings.intern_name(strings.source_slice(0, 7));

        assert_eq!(flat, generated);
        assert!(!strings.reserve_name(generated));
        assert_eq!(strings.dedup.len(), 2);
    }

    #[test]
    fn precomputed_hash_collisions_compare_string_bytes() {
        let allocator = Allocator::new();
        let mut strings = string_pool(&allocator, "ab");
        let first = AstInternedUtf8::from_utf8_range(strings.source_slice(0, 1));
        let second = AstInternedUtf8::from_utf8_range(strings.source_slice(1, 2));
        strings.dedup.insert(InternEntry::new(1, first, false), ());
        strings.dedup.insert(InternEntry::new(1, second, false), ());

        assert_eq!(strings.dedup.len(), 2);
        assert_eq!(
            strings
                .find_intern_entry(strings.logical_bytes(second), 1)
                .unwrap()
                .id(),
            second
        );
    }
}
