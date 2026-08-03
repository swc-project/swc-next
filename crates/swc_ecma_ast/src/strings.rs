use std::collections::HashMap;

use swc_next_allocator::wtf8::Wtf8;

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
    };
}

define_ast_string!(AstUtf8);
define_ast_string!(AstWtf8);

impl From<AstUtf8> for AstWtf8 {
    #[inline]
    fn from(value: AstUtf8) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StringPool<'a> {
    pub source: &'a str,
    pub extra: Vec<u8>,
    dedup: HashMap<Vec<u8>, (u32, u32)>,
}

impl<'a> StringPool<'a> {
    pub fn new(source: &'a str) -> Self {
        assert!(source.len() <= u32::MAX as usize);
        Self {
            source,
            extra: Vec::new(),
            dedup: HashMap::new(),
        }
    }

    #[inline]
    pub fn get_utf8(&self, id: AstUtf8) -> &str {
        // SAFETY: `AstUtf8` is guaranteed to reference valid UTF-8.
        unsafe { std::str::from_utf8_unchecked(self.get_bytes(id.start, id.end)) }
    }

    #[inline]
    pub fn get_wtf8(&self, id: AstWtf8) -> &Wtf8 {
        // SAFETY: `AstWtf8` is guaranteed to reference valid WTF-8.
        unsafe { Wtf8::from_bytes_unchecked(self.get_bytes(id.start, id.end)) }
    }

    #[inline]
    fn get_bytes(&self, start: u32, end: u32) -> &[u8] {
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
            &self.extra[start..end]
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
    pub fn add_utf8(&mut self, text: &str) -> AstUtf8 {
        let (start, end) = self.add_bytes(text.as_bytes());
        AstUtf8 { start, end }
    }

    #[inline]
    pub fn add_wtf8(&mut self, text: &Wtf8) -> AstWtf8 {
        let (start, end) = self.add_bytes(text.as_bytes());
        AstWtf8 { start, end }
    }

    fn add_bytes(&mut self, text: &[u8]) -> (u32, u32) {
        if text.is_empty() {
            return (0, 0);
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

        let id = (start as u32, end as u32);
        self.dedup.insert(text.to_vec(), id);
        id
    }
}
