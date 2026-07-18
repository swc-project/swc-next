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
pub(crate) struct AtomId(u64);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Wtf8AtomId(u64);

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
        atoms.push(atom);
        AtomId(u64::try_from(index).expect("atom table exceeds u64") + 1)
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
        atoms.push(atom);
        Wtf8AtomId(u64::try_from(index).expect("WTF-8 atom table exceeds u64") + 1)
    }

    #[inline]
    pub(crate) fn wtf8_atom(&self, id: Wtf8AtomId) -> Wtf8Atom<'a> {
        debug_assert_ne!(id.0, 0);
        let index = usize::try_from(id.0 - 1).expect("WTF-8 atom id exceeds usize");
        // SAFETY: Wtf8AtomIds are only created immediately after appending the
        // corresponding table entry.
        unsafe { *(&*self.wtf8_atoms.get()).as_slice().get_unchecked(index) }
    }

    #[inline]
    pub(crate) const fn none_atom() -> AtomId {
        AtomId(0)
    }

    #[inline]
    pub(crate) const fn none_wtf8_atom() -> Wtf8AtomId {
        Wtf8AtomId(0)
    }

    #[inline]
    pub(crate) const fn is_none_atom(id: AtomId) -> bool {
        id.0 == 0
    }

    #[inline]
    pub(crate) const fn is_none_wtf8_atom(id: Wtf8AtomId) -> bool {
        id.0 == 0
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;
    use crate::Ast;

    #[test]
    fn public_atoms_are_borrowed_slices_and_stored_ids_are_compact() {
        assert_eq!(size_of::<Atom<'_>>(), 16);
        assert_eq!(size_of::<Wtf8Atom<'_>>(), 16);
        assert_eq!(size_of::<AtomId>(), 8);
        assert_eq!(size_of::<Wtf8AtomId>(), 8);
    }

    #[test]
    fn atom_store_preserves_content_equality_without_interning() {
        let allocator = Allocator::default();
        let store = AtomStore::new_in(&allocator);
        let first = store.store_atom(Atom::from("same"));
        let second = store.store_atom(Atom::from("same"));

        assert_ne!(first, second);
        assert_eq!(store.atom(first), store.atom(second));
    }

    #[test]
    fn atom_store_keeps_the_original_borrowed_slice() {
        let allocator = Allocator::default();
        let store = AtomStore::new_in(&allocator);
        let source = String::from("borrowed");
        let atom = Atom::from(source.as_str());
        let id = store.store_atom(atom);
        let stored = store.atom(id);

        assert_eq!(stored.as_ptr(), source.as_ptr());
    }

    #[test]
    fn optional_atom_distinguishes_none_from_an_empty_atom() {
        use crate::ExtraDataCompact;

        let allocator = Allocator::default();
        let ast = Ast::new(0, &allocator);
        let none = None::<Atom<'_>>.to_extra_data(&ast);
        let empty = Some(Atom::empty()).to_extra_data(&ast);

        assert_eq!(
            unsafe { Option::<Atom<'_>>::from_extra_data(none, &ast) },
            None
        );
        assert_eq!(
            unsafe { Option::<Atom<'_>>::from_extra_data(empty, &ast) },
            Some(Atom::empty())
        );
    }
}
