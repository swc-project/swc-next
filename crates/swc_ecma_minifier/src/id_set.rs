//! Sets of dense ids, such as node and symbol ids.
//!
//! Node and symbol ids index arrays, so a bit per id replaces hashing. Sets
//! grow on insertion, which keeps nodes created during compression valid
//! members.

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

fn position<I: Idx>(id: I) -> (usize, u64) {
    let index = id.index();
    (index / 64, 1 << (index % 64))
}

#[cfg(test)]
mod tests {
    use swc_next_ecma_ast::NodeId;

    use super::IdSet;

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
    }
}
