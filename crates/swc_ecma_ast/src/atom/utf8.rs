use std::hash::{BuildHasher, Hash, Hasher};

use hashbrown::{DefaultHashBuilder, HashMap};
use oxc_index::IndexVec;
use swc_core::atoms::Atom;

#[derive(Default)]
pub struct Utf8Allocator {
    storage: IndexVec<Utf8Ref, Atom>,
    dedup: HashMap<Utf8Ref, ()>,
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

impl Utf8Allocator {
    pub fn add(&mut self, s: &str) -> Utf8Ref {
        let hash = make_hash(&self.hasher, s);
        let entry = self.dedup.raw_entry_mut().from_hash(hash, |symbol| {
            // SAFETY: This is safe because we only operate on symbols that
            //         we receive from our backend making them valid.
            s == unsafe { self.storage.get(*symbol).unwrap_unchecked() }
        });
        use hashbrown::hash_map::RawEntryMut;
        let (&mut symbol, &mut ()) = match entry {
            RawEntryMut::Occupied(occupied) => occupied.into_key_value(),
            RawEntryMut::Vacant(vacant) => {
                let symbol = self.storage.push(Atom::new(s));
                vacant.insert_with_hasher(hash, symbol, (), |symbol| {
                    // SAFETY: This is safe because we only operate on symbols that
                    //         we receive from our backend making them valid.
                    let string = unsafe { self.storage.get(*symbol).unwrap_unchecked() };
                    make_hash(&self.hasher, string)
                })
            }
        };
        symbol
    }

    pub fn resolve(&self, id: Utf8Ref) -> Option<&str> {
        self.storage.get(id).map(|atom| atom.as_str())
    }

    pub fn resolve_atom(&self, id: Utf8Ref) -> Option<Atom> {
        self.storage.get(id).cloned()
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
