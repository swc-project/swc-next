use std::{marker::PhantomData, str};

use swc_experimental_allocator::{
    Allocator, hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec, wtf8::Wtf8,
};

use crate::Ast;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct StringId {
    start: u32,
    end: u32,
}

impl StringId {
    pub(crate) const EMPTY: Self = Self { start: 0, end: 0 };
}

/// An interned UTF-8 string stored by an [`Ast`].
///
/// The two `u32`s are byte offsets into the AST's shared string buffer.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Atom<'a> {
    start: u32,
    end: u32,
    marker: PhantomData<&'a ()>,
}

impl<'a> Atom<'a> {
    pub const EMPTY: Self = Self::from_id(StringId::EMPTY);

    #[inline]
    const fn from_id(id: StringId) -> Self {
        Self {
            start: id.start,
            end: id.end,
            marker: PhantomData,
        }
    }

    #[inline]
    const fn id(self) -> StringId {
        StringId {
            start: self.start,
            end: self.end,
        }
    }

    #[inline]
    pub const fn empty() -> Self {
        Self::EMPTY
    }

    #[inline]
    pub fn new_in(value: &str, ast: &mut Ast<'a>) -> Self {
        ast.add_atom(value)
    }

    #[inline]
    pub const fn len(self) -> u32 {
        self.end - self.start
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    #[inline]
    pub fn as_str<'ast>(self, ast: &'ast Ast<'_>) -> &'ast str {
        ast.atom(self)
    }

    #[inline]
    pub fn as_bytes<'ast>(self, ast: &'ast Ast<'_>) -> &'ast [u8] {
        self.as_str(ast).as_bytes()
    }
}

/// An interned WTF-8 string stored by an [`Ast`].
///
/// UTF-8 atoms and WTF-8 atoms use the same byte buffer and interning table.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Wtf8Atom<'a> {
    start: u32,
    end: u32,
    marker: PhantomData<&'a ()>,
}

impl<'a> Wtf8Atom<'a> {
    pub const EMPTY: Self = Self::from_id(StringId::EMPTY);

    #[inline]
    const fn from_id(id: StringId) -> Self {
        Self {
            start: id.start,
            end: id.end,
            marker: PhantomData,
        }
    }

    #[inline]
    const fn id(self) -> StringId {
        StringId {
            start: self.start,
            end: self.end,
        }
    }

    #[inline]
    pub const fn empty() -> Self {
        Self::EMPTY
    }

    #[inline]
    pub fn new_in(value: &Wtf8, ast: &mut Ast<'a>) -> Self {
        ast.add_wtf8_atom(value)
    }

    #[inline]
    pub const fn len(self) -> u32 {
        self.end - self.start
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    #[inline]
    pub fn as_wtf8<'ast>(self, ast: &'ast Ast<'_>) -> &'ast Wtf8 {
        ast.wtf8_atom(self)
    }
}

#[derive(Debug)]
pub(crate) struct StringPool<'a> {
    bytes: ArenaVec<'a, u8>,
    dedup: ArenaHashMap<'a, ArenaVec<'a, u8>, StringId>,
}

impl<'a> StringPool<'a> {
    pub(crate) fn with_capacity_in(bytes: usize, entries: usize, allocator: &'a Allocator) -> Self {
        Self {
            bytes: ArenaVec::with_capacity_in(bytes, allocator),
            dedup: ArenaHashMap::with_capacity_in(entries, allocator),
        }
    }

    #[inline]
    fn get(&self, id: StringId) -> &[u8] {
        debug_assert!(id.end >= id.start);
        &self.bytes[id.start as usize..id.end as usize]
    }

    fn intern(&mut self, text: &[u8]) -> StringId {
        if text.is_empty() {
            return StringId::EMPTY;
        }
        if let Some(id) = self.dedup.get(text) {
            return *id;
        }

        let start = self.bytes.len() as u32;
        self.bytes.extend_from_slice_copy(text);
        let end = self.bytes.len() as u32;
        let id = StringId { start, end };
        let mut key = ArenaVec::with_capacity_in(text.len(), self.bytes.bump());
        key.extend_from_slice_copy(text);
        self.dedup.insert(key, id);
        id
    }

    #[inline]
    pub(crate) fn add_atom(&mut self, text: &str) -> Atom<'a> {
        Atom::from_id(self.intern(text.as_bytes()))
    }

    #[inline]
    pub(crate) fn add_wtf8_atom(&mut self, text: &Wtf8) -> Wtf8Atom<'a> {
        Wtf8Atom::from_id(self.intern(text.as_bytes()))
    }

    #[inline]
    pub(crate) fn atom(&self, atom: Atom<'_>) -> &str {
        // Atom can only be constructed from UTF-8 input.
        unsafe { str::from_utf8_unchecked(self.get(atom.id())) }
    }

    #[inline]
    pub(crate) fn wtf8_atom(&self, atom: Wtf8Atom<'_>) -> &Wtf8 {
        // Wtf8Atom can only be constructed from a validated Wtf8 value.
        unsafe { Wtf8::from_bytes_unchecked(self.get(atom.id())) }
    }
}

impl<'from, 'to> swc_experimental_allocator::CloneIn<'to> for Atom<'from> {
    type Cloned = Atom<'to>;

    #[inline]
    fn clone_in(&self, _allocator: &'to swc_experimental_allocator::Allocator) -> Self::Cloned {
        Atom::from_id(self.id())
    }
}

impl<'from, 'to> swc_experimental_allocator::CloneIn<'to> for Wtf8Atom<'from> {
    type Cloned = Wtf8Atom<'to>;

    #[inline]
    fn clone_in(&self, _allocator: &'to swc_experimental_allocator::Allocator) -> Self::Cloned {
        Wtf8Atom::from_id(self.id())
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn atoms_are_two_u32s() {
        assert_eq!(size_of::<Atom<'_>>(), 8);
        assert_eq!(size_of::<Wtf8Atom<'_>>(), 8);
        assert_eq!(size_of::<StringId>(), 8);
    }

    #[test]
    fn utf8_and_wtf8_share_interned_storage() {
        let allocator = Allocator::default();
        let mut pool = StringPool::with_capacity_in(0, 0, &allocator);
        let atom = pool.add_atom("shared");
        let wtf8 = pool.add_wtf8_atom(Wtf8::from_str("shared"));

        assert_eq!(atom.id(), wtf8.id());
        assert_eq!(pool.atom(atom), "shared");
        assert_eq!(pool.wtf8_atom(wtf8).as_str(), Some("shared"));
        assert_eq!(pool.dedup.len(), 1);
        assert_eq!(pool.bytes, b"shared");
    }
}
