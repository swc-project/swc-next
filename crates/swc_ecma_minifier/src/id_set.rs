//! Sets and maps of dense ids, such as node and symbol ids.
//!
//! Node and symbol ids index arrays, so a bit or a slot per id replaces
//! hashing. Both grow on insertion, which keeps nodes created during
//! compression valid keys.

use std::marker::PhantomData;

use swc_next_allocator::Idx;

pub(crate) struct IdSet<I> {
    words: Vec<u64>,
    _marker: PhantomData<I>,
}

impl<I> Default for IdSet<I> {
    fn default() -> Self {
        Self {
            words: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<I: Idx> IdSet<I> {
    /// Adds `id`, and returns whether it was absent.
    pub(crate) fn insert(&mut self, id: I) -> bool {
        let (word, bit) = position(id);
        if word >= self.words.len() {
            self.words.resize(word + 1, 0);
        }
        let absent = self.words[word] & bit == 0;
        self.words[word] |= bit;
        absent
    }

    /// Removes `id`, and returns whether it was present.
    pub(crate) fn remove(&mut self, id: I) -> bool {
        let (word, bit) = position(id);
        let Some(word) = self.words.get_mut(word) else {
            return false;
        };
        let present = *word & bit != 0;
        *word &= !bit;
        present
    }

    pub(crate) fn contains(&self, id: I) -> bool {
        let (word, bit) = position(id);
        self.words.get(word).is_some_and(|word| word & bit != 0)
    }

    pub(crate) fn extend(&mut self, other: &Self) {
        if self.words.len() < other.words.len() {
            self.words.resize(other.words.len(), 0);
        }
        for (word, other) in self.words.iter_mut().zip(&other.words) {
            *word |= other;
        }
    }

    /// Iterates the members in ascending id order.
    pub(crate) fn iter(&self) -> impl Iterator<Item = I> + '_ {
        self.words.iter().enumerate().flat_map(|(index, &word)| {
            let mut word = word;
            std::iter::from_fn(move || {
                if word == 0 {
                    return None;
                }
                let bit = word.trailing_zeros() as usize;
                word &= word - 1;
                Some(I::from_usize(index * 64 + bit))
            })
        })
    }
}

/// A map from dense ids to values, with a slot per id up to the largest key.
/// Use it for ids that are few, such as symbol ids, rather than node ids.
pub(crate) struct IdMap<I, V> {
    values: Vec<Option<V>>,
    _marker: PhantomData<I>,
}

impl<I, V> Default for IdMap<I, V> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<I: Idx, V> IdMap<I, V> {
    pub(crate) fn insert(&mut self, id: I, value: V) {
        let index = id.index();
        if index >= self.values.len() {
            self.values.resize_with(index + 1, || None);
        }
        self.values[index] = Some(value);
    }

    pub(crate) fn get(&self, id: I) -> Option<&V> {
        self.values.get(id.index())?.as_ref()
    }
}

impl<I: Idx, V> FromIterator<(I, V)> for IdMap<I, V> {
    fn from_iter<T: IntoIterator<Item = (I, V)>>(iter: T) -> Self {
        let mut map = Self::default();
        for (id, value) in iter {
            map.insert(id, value);
        }
        map
    }
}

fn position<I: Idx>(id: I) -> (usize, u64) {
    let index = id.index();
    (index / 64, 1 << (index % 64))
}

#[cfg(test)]
mod tests {
    use swc_next_ecma_ast::NodeId;

    use super::{IdMap, IdSet};

    #[test]
    fn membership() {
        let mut set = IdSet::default();
        assert!(!set.contains(NodeId::from_usize(200)));
        assert!(!set.remove(NodeId::from_usize(200)));

        assert!(set.insert(NodeId::from_usize(200)));
        assert!(!set.insert(NodeId::from_usize(200)));
        assert!(set.insert(NodeId::from_usize(3)));
        assert!(set.contains(NodeId::from_usize(200)));
        assert!(!set.contains(NodeId::from_usize(201)));

        let mut other = IdSet::default();
        other.insert(NodeId::from_usize(64));
        set.extend(&other);
        let members: Vec<_> = set.iter().map(|id| id.index()).collect();
        assert_eq!(members, [3, 64, 200]);

        assert!(set.remove(NodeId::from_usize(200)));
        assert!(!set.contains(NodeId::from_usize(200)));

        let mut map = IdMap::default();
        assert_eq!(map.get(NodeId::from_usize(5)), None);
        map.insert(NodeId::from_usize(5), 'a');
        map.insert(NodeId::from_usize(5), 'b');
        assert_eq!(map.get(NodeId::from_usize(5)), Some(&'b'));
        assert_eq!(map.get(NodeId::from_usize(4)), None);
    }
}
