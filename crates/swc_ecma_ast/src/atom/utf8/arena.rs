use string_interner::{StringInterner, Symbol, backend::BucketBackend};
use swc_core::atoms::Atom;

pub struct Utf8Allocator(StringInterner<BucketBackend<Utf8Ref>>);

impl Default for Utf8Allocator {
    fn default() -> Self {
        Self(StringInterner::new())
    }
}

impl Utf8Allocator {
    pub fn add(&mut self, s: &str) -> Utf8Ref {
        self.0.get_or_intern(s)
    }

    pub fn resolve(&self, id: Utf8Ref) -> Option<&str> {
        self.0.resolve(id)
    }

    pub fn resolve_atom(&self, id: Utf8Ref) -> Option<Atom> {
        self.resolve(id).map(|s| Atom::from(s))
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OptionalUtf8Ref(u32);

impl From<Utf8Ref> for OptionalUtf8Ref {
    fn from(value: Utf8Ref) -> Self {
        Self(value.0)
    }
}

impl OptionalUtf8Ref {
    pub fn none() -> Self {
        Self(u32::MAX)
    }

    pub fn to_option(self) -> Option<Utf8Ref> {
        if self.0 == u32::MAX {
            return None;
        }
        Some(Utf8Ref(self.0))
    }
}
