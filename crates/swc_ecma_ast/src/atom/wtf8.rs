use std::ptr::NonNull;

use oxc_index::IndexVec;
use swc_core::atoms::wtf8::{Wtf8, Wtf8Buf};

use crate::define_optional_index_type;

#[derive(Default)]
pub(super) struct Wtf8Allocator {
    spans: IndexVec<Wtf8Ref, NonNull<Wtf8>>,
    head: Wtf8Buf,
    full: Vec<Wtf8Buf>,
}

impl Wtf8Allocator {
    pub fn add(&mut self, s: &Wtf8) -> Wtf8Ref {
        const GAP: char = '\u{FFFD}';

        let cap = self.head.capacity();
        if cap < self.head.len() + s.len() + GAP.len_utf8() {
            let new_cap = (usize::max(cap, s.len()) + 1).next_power_of_two();
            let new_head = Wtf8Buf::with_capacity(new_cap);
            let old_head = core::mem::replace(&mut self.head, new_head);
            self.full.push(old_head);
        }
        // Append a separator to avoid two codepoints being merged
        self.head.push_char(GAP);

        let len = self.head.len();
        self.head.push_wtf8(s);
        self.spans.push(NonNull::from(
            // SAFETY: We convert from bytes to wtf8 from which we know through the
            //         input string that they must represent valid wtf8.
            unsafe { Wtf8::from_bytes_unchecked(&self.head.as_bytes()[len..]) },
        ))
    }

    #[inline]
    pub fn resolve(&self, s: Wtf8Ref) -> Option<&Wtf8> {
        self.spans.get(s).map(|s|
        // SAFETY: This is safe since we only ever operate on interned `Wtf8`
        //         that are never moved around in memory to avoid danling
        //         references.
            unsafe { s.as_ref() })
    }
}

/// The reference to a wtf8 string in the string allocator.
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub struct Wtf8Ref(u32);

impl oxc_index::Idx for Wtf8Ref {
    const MAX: usize = (u32::MAX - 1) as usize;

    unsafe fn from_usize_unchecked(idx: usize) -> Self {
        Self(idx as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

define_optional_index_type!(OptionalWtf8Ref, Wtf8Ref);

#[cfg(test)]
mod tests {
    use super::*;
    use swc_core::atoms::wtf8::CodePoint;

    #[test]
    fn test_allocator_basic() {
        let mut allocator = Wtf8Allocator::default();
        let s = Wtf8::from_str("hello");
        let r = allocator.add(s);
        assert_eq!(allocator.resolve(r), Some(s));
    }

    #[test]
    fn test_lone_surrogates() {
        let mut allocator = Wtf8Allocator::default();

        // High surrogate (D800)
        let mut buf1 = Wtf8Buf::new();
        buf1.push(unsafe { CodePoint::from_u32_unchecked(0xD800) });
        let s1 = buf1.slice(0, buf1.len());
        let r1 = allocator.add(s1);

        // Low surrogate (DC00)
        let mut buf2 = Wtf8Buf::new();
        buf2.push(unsafe { CodePoint::from_u32_unchecked(0xDC00) });
        let s2 = buf2.slice(0, buf2.len());
        let r2 = allocator.add(s2);

        // Verify they can be resolved correctly
        assert_eq!(
            allocator.resolve(r1).unwrap().as_bytes(),
            buf1.slice(0, buf1.len()).as_bytes()
        );
        assert_eq!(
            allocator.resolve(r2).unwrap().as_bytes(),
            buf2.slice(0, buf2.len()).as_bytes()
        );

        // Detailed check of resolved content
        let resolved1 = allocator.resolve(r1).unwrap();
        assert_eq!(resolved1.len(), 3);
        assert_eq!(resolved1.as_bytes(), &[0xED, 0xA0, 0x80]);

        let resolved2 = allocator.resolve(r2).unwrap();
        assert_eq!(resolved2.len(), 3);
        assert_eq!(resolved2.as_bytes(), &[0xED, 0xB0, 0x80]);
    }

    #[test]
    fn test_reallocation_and_persistence() {
        let mut allocator = Wtf8Allocator::default();
        let mut refs = Vec::new();
        let mut expected = Vec::new();

        // Push many strings to trigger multiple reallocations in Wtf8Allocator
        for i in 0..1000 {
            let mut buf = Wtf8Buf::new();
            buf.push_str(&format!("string-content-prefix-{}-", i));
            // Add a lone surrogate occasionally
            if i % 7 == 0 {
                buf.push(unsafe { CodePoint::from_u32_unchecked(0xD800 + (i % 0x400) as u32) });
            }
            buf.push_str("-suffix");

            let s = buf.slice(0, buf.len()).to_owned();
            let r = allocator.add(&s);
            refs.push(r);
            expected.push(s);
        }

        // Verify all references remain valid and point to correct content after all additions
        for (r, exp) in refs.into_iter().zip(expected.iter()) {
            let resolved = allocator.resolve(r).expect("Should find string");
            assert_eq!(
                resolved.as_bytes(),
                exp.as_bytes(),
                "Content mismatch for resolved string"
            );
        }
    }

    #[test]
    fn test_complex_lone_surrogates() {
        let mut allocator = Wtf8Allocator::default();

        // A string with mixed content: UTF-8, lone high, lone low, and valid surrogate pair (as UTF-8)
        let mut buf = Wtf8Buf::new();
        buf.push_str("A");
        buf.push(unsafe { CodePoint::from_u32_unchecked(0xD800) }); // Lone high
        buf.push_char('B');
        buf.push(unsafe { CodePoint::from_u32_unchecked(0xDC00) }); // Lone low
        buf.push_str("🙂"); // Valid emoji (surrogate pair in UTF-16, 4-byte UTF-8)

        let s = buf.slice(0, buf.len());
        let r = allocator.add(s);

        let resolved = allocator.resolve(r).unwrap();
        assert_eq!(resolved.as_bytes(), s.as_bytes());

        // Verify it spans 1 (A) + 3 (D800) + 1 (B) + 3 (DC00) + 4 (Emoji) = 12 bytes
        assert_eq!(resolved.len(), 12);
    }

    #[test]
    fn test_adjacent_surrogates_not_merged() {
        let mut allocator = Wtf8Allocator::default();

        // String 1 ends with lone high surrogate
        let mut buf1 = Wtf8Buf::new();
        buf1.push(unsafe { CodePoint::from_u32_unchecked(0xD800) });
        let r1 = allocator.add(buf1.slice(0, buf1.len()));

        // String 2 starts with lone low surrogate
        let mut buf2 = Wtf8Buf::new();
        buf2.push(unsafe { CodePoint::from_u32_unchecked(0xDC00) });
        let r2 = allocator.add(buf2.slice(0, buf2.len()));

        // Even if they are added consecutively, they are separated by GAP in the allocator
        // so they can't be accidentally seen as a pair if someone scans the allocator's internal buffer.
        // And resolving them should give back the original lone surrogates.
        assert_eq!(
            allocator.resolve(r1).unwrap().as_bytes(),
            &[0xED, 0xA0, 0x80]
        );
        assert_eq!(
            allocator.resolve(r2).unwrap().as_bytes(),
            &[0xED, 0xB0, 0x80]
        );
    }

    #[test]
    fn test_empty_string() {
        let mut allocator = Wtf8Allocator::default();
        let s = Wtf8::from_str("");
        let r = allocator.add(s);
        assert_eq!(allocator.resolve(r), Some(s));
        assert_eq!(allocator.resolve(r).unwrap().len(), 0);
    }
}
