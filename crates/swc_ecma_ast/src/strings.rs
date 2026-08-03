use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct AstUtf8 {
    pub start: u32,
    pub end: u32,
}

impl AstUtf8 {
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

#[derive(Clone, Debug, Default)]
pub struct StringPool<'a> {
    pub source: &'a [u8],
    pub extra: Vec<u8>,
    dedup: HashMap<Vec<u8>, AstUtf8>,
}

impl<'a> StringPool<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        assert!(source.len() <= u32::MAX as usize);
        Self {
            source,
            extra: Vec::new(),
            dedup: HashMap::new(),
        }
    }

    #[inline]
    pub fn get(&self, id: AstUtf8) -> &[u8] {
        debug_assert!(id.end >= id.start);
        if id.start == id.end {
            return b"";
        }
        let source_len = self.source.len() as u32;
        if id.start < source_len {
            debug_assert!(id.end <= source_len);
            &self.source[id.start as usize..id.end as usize]
        } else {
            let start = (id.start - source_len) as usize;
            let end = (id.end - source_len) as usize;
            &self.extra[start..end]
        }
    }

    #[inline]
    pub fn source_slice(&self, start: u32, end: u32) -> AstUtf8 {
        debug_assert!(start <= end);
        debug_assert!(end <= self.source.len() as u32);
        AstUtf8 { start, end }
    }

    pub fn ensure_unused_capacity(&mut self, bytes: u32, entries: u32) {
        self.extra.reserve(bytes as usize);
        self.dedup.reserve(entries as usize);
    }

    pub fn add_string(&mut self, text: &[u8]) -> AstUtf8 {
        if text.is_empty() {
            return AstUtf8::EMPTY;
        }
        if let Some(&id) = self.dedup.get(text) {
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
        let id = AstUtf8 {
            start: start as u32,
            end: end as u32,
        };
        self.dedup.insert(text.to_vec(), id);
        id
    }
}
