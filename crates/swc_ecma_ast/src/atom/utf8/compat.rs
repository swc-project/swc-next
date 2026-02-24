use oxc_index::IndexVec;
use swc_core::atoms::Atom;

#[derive(Default)]
pub struct Utf8Allocator(IndexVec<Utf8Ref, Atom>);

impl Utf8Allocator {
    pub fn add(&mut self, s: &str) -> Utf8Ref {
        let atom = Atom::new(s);
        let idx = self.0.push(atom);
        idx
    }

    pub fn resolve(&self, id: Utf8Ref) -> Option<&str> {
        self.0.get(id).map(|atom| atom.as_str())
    }

    pub fn resolve_atom(&self, id: Utf8Ref) -> Option<Atom> {
        self.0.get(id).cloned()
    }
}

/// The reference to a wtf8 string in the string allocator.
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub struct Utf8Ref(u32);

impl oxc_index::Idx for Utf8Ref {
    const MAX: usize = (u32::MAX - 1) as usize;

    unsafe fn from_usize_unchecked(idx: usize) -> Self {
        Self(idx as u32)
    }

    fn index(self) -> usize {
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
