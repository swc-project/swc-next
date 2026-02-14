use string_interner::{StringInterner, Symbol, backend::BucketBackend};

use crate::define_optional_index_type;

pub(super) type Utf8Allocator = StringInterner<BucketBackend<Utf8Ref>>;

/// The reference to a utf8 string in the string allocator.
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub struct Utf8Ref(u32);

impl Symbol for Utf8Ref {
    fn try_from_usize(index: usize) -> Option<Self> {
        if index == u32::MAX as usize {
            return None;
        }
        Some(Self(index as u32))
    }

    fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl oxc_index::Idx for Utf8Ref {
    const MAX: usize = (u32::MAX - 1) as usize;

    unsafe fn from_usize_unchecked(idx: usize) -> Self {
        Self(idx as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

define_optional_index_type!(OptionalUtf8Ref, Utf8Ref);
