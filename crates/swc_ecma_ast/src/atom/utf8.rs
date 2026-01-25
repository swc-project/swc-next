use std::ops::{Index, IndexMut};

use fixedbitset::FixedBitSet;
use oxc_index::IndexVec;
use string_interner::Symbol;

use crate::define_optional_index_type;

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub struct Utf8Ref(u32);

impl Symbol for Utf8Ref {
    fn try_from_usize(index: usize) -> Option<Self> {
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

#[derive(Debug, Clone)]
pub struct Utf8Map<T>(IndexVec<Utf8Ref, Option<T>>);

impl<T> Default for Utf8Map<T> {
    fn default() -> Self {
        Self(IndexVec::new())
    }
}

impl<T> Utf8Map<T> {
    pub fn new() -> Self {
        Self(IndexVec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(IndexVec::with_capacity(capacity))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, index: Utf8Ref) -> Option<&T> {
        match self.0.get(index) {
            Some(Some(value)) => Some(value),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, index: Utf8Ref) -> Option<&mut T> {
        match self.0.get_mut(index) {
            Some(Some(value)) => Some(value),
            _ => None,
        }
    }

    pub fn insert(&mut self, key: Utf8Ref, value: T) {
        if self.0.len() <= key.0 as usize {
            self.0.resize_with(key.0 as usize + 1, Default::default);
        }
        self.0[key] = Some(value);
    }
}

impl<T> Index<Utf8Ref> for Utf8Map<T> {
    type Output = T;

    fn index(&self, index: Utf8Ref) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> IndexMut<Utf8Ref> for Utf8Map<T> {
    fn index_mut(&mut self, index: Utf8Ref) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Utf8HashSet(FixedBitSet);

impl Default for Utf8HashSet {
    fn default() -> Self {
        Self(FixedBitSet::new())
    }
}

impl Utf8HashSet {
    pub fn new() -> Self {
        Self(FixedBitSet::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(FixedBitSet::with_capacity(capacity))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn insert(&mut self, index: Utf8Ref) {
        self.0.grow_and_insert(index.0 as usize);
    }

    pub fn contains(&self, index: Utf8Ref) -> bool {
        self.0.contains(index.0 as usize)
    }

    pub fn iter(&self) -> impl Iterator<Item = Utf8Ref> {
        self.0.as_slice().iter().map(|i| Utf8Ref(*i as u32))
    }
}
