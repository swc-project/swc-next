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
