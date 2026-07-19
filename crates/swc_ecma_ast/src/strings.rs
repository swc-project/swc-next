use std::cell::UnsafeCell;

use swc_experimental_allocator::{
    Allocator,
    atom::{Atom, Wtf8Atom},
    vec::Vec as ArenaVec,
};

/// Compact handles stored in node payloads and `ExtraData`.
///
/// Zero is reserved for `None`; table indices are encoded as `index + 1`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct AtomId(u32);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Wtf8AtomId(u32);

impl AtomId {
    #[inline]
    pub(crate) const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    #[inline]
    pub(crate) const fn raw(self) -> u32 {
        self.0
    }
}

impl Wtf8AtomId {
    #[inline]
    pub(crate) const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    #[inline]
    pub(crate) const fn raw(self) -> u32 {
        self.0
    }
}

/// Append-only side tables for borrowed atoms.
///
/// Mutation through `UnsafeCell` is safe because `Ast` is single-threaded and
/// the tables only append. Existing `Atom` values point to source or arena
/// storage, not into these vectors.
pub(crate) struct AtomStore<'a> {
    atoms: UnsafeCell<ArenaVec<'a, Atom<'a>>>,
    wtf8_atoms: UnsafeCell<ArenaVec<'a, Wtf8Atom<'a>>>,
}

impl<'a> AtomStore<'a> {
    pub(crate) fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            atoms: UnsafeCell::new(ArenaVec::new_in(allocator)),
            wtf8_atoms: UnsafeCell::new(ArenaVec::new_in(allocator)),
        }
    }

    #[inline]
    pub(crate) fn store_atom(&self, atom: Atom<'a>) -> AtomId {
        // SAFETY: The store is append-only and Ast is !Sync.
        let atoms = unsafe { &mut *self.atoms.get() };
        let index = atoms.len();
        let id = index as u32 + 1;
        atoms.push(atom);
        AtomId(id)
    }

    #[inline]
    pub(crate) fn atom(&self, id: AtomId) -> Atom<'a> {
        debug_assert_ne!(id.0, 0);
        let index = usize::try_from(id.0 - 1).expect("atom id exceeds usize");
        // SAFETY: AtomIds are only created immediately after appending the
        // corresponding table entry.
        unsafe { *(&*self.atoms.get()).as_slice().get_unchecked(index) }
    }

    #[inline]
    pub(crate) fn store_wtf8_atom(&self, atom: Wtf8Atom<'a>) -> Wtf8AtomId {
        // SAFETY: The store is append-only and Ast is !Sync.
        let atoms = unsafe { &mut *self.wtf8_atoms.get() };
        let index = atoms.len();
        let id = index as u32 + 1;
        atoms.push(atom);
        Wtf8AtomId(id)
    }

    #[inline]
    pub(crate) fn wtf8_atom(&self, id: Wtf8AtomId) -> Wtf8Atom<'a> {
        debug_assert_ne!(id.0, 0);
        let index = usize::try_from(id.0 - 1).expect("WTF-8 atom id exceeds usize");
        // SAFETY: Wtf8AtomIds are only created immediately after appending the
        // corresponding table entry.
        unsafe { *(&*self.wtf8_atoms.get()).as_slice().get_unchecked(index) }
    }
}
