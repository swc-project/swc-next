use std::hash::{BuildHasher, Hash, Hasher};

use hashbrown::{DefaultHashBuilder, HashTable};
use oxc_index::IndexVec;
use swc_core::atoms::{Wtf8Atom, wtf8::Wtf8};

#[derive(Default)]
pub struct Wtf8Allocator {
    storage: IndexVec<Wtf8Ref, Wtf8Atom>,
    dedup: HashTable<Wtf8Ref>,
    hasher: DefaultHashBuilder,
}

/// Creates the `u64` hash value for the given value using the given hash builder.
fn make_hash<T>(builder: &impl BuildHasher, value: &T) -> u64
where
    T: ?Sized + Hash,
{
    let state = &mut builder.build_hasher();
    value.hash(state);
    state.finish()
}

impl Wtf8Allocator {
    pub fn add(&mut self, s: &Wtf8) -> Wtf8Ref {
        let hash = make_hash(&self.hasher, s);

        if let Some(symbol) = self.dedup.find(hash, |symbol| {
            // SAFETY: This is safe because we only operate on symbols that
            //         we receive from our backend making them valid.
            s == unsafe { self.storage.get(*symbol).unwrap_unchecked() }.as_wtf8()
        }) {
            return *symbol;
        }

        let symbol = self.storage.push(Wtf8Atom::new(s));
        self.dedup.insert_unique(hash, symbol, |sym| {
            let string = unsafe { self.storage.get(*sym).unwrap_unchecked() };
            make_hash(&self.hasher, string.as_wtf8())
        });
        symbol
    }

    pub fn resolve(&self, id: Wtf8Ref) -> Option<&Wtf8> {
        self.storage.get(id).map(|atom| atom.as_wtf8())
    }

    pub fn resolve_atom(&self, id: Wtf8Ref) -> Option<Wtf8Atom> {
        self.storage.get(id).cloned()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OptionalWtf8Ref(u32);

impl From<Wtf8Ref> for OptionalWtf8Ref {
    fn from(value: Wtf8Ref) -> Self {
        Self(value.0)
    }
}

impl OptionalWtf8Ref {
    pub fn none() -> Self {
        Self(u32::MAX)
    }

    pub fn to_option(self) -> Option<Wtf8Ref> {
        if self.0 == u32::MAX {
            return None;
        }
        Some(Wtf8Ref(self.0))
    }
}
