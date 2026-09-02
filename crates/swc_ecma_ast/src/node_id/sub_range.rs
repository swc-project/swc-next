use std::marker::PhantomData;

use crate::{
    Ast,
    node_id::{ExtraDataCompact, ExtraDataId},
};

/// A typed range of values in the AST extra-data arena.
///
/// It only contains the start and end of the range in [crate::ast_list::NodeList].
#[repr(C)]
#[derive(Debug, Hash)]
pub struct SubRange<T> {
    pub start: ExtraDataId,
    pub end: ExtraDataId,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T> Clone for SubRange<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SubRange<T> {}

impl<T> SubRange<T> {
    pub fn empty() -> Self {
        Self {
            start: ExtraDataId::from_raw(0),
            end: ExtraDataId::from_raw(0),
            _phantom: PhantomData,
        }
    }

    pub fn len(self) -> usize {
        self.end.index() - self.start.index()
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub(crate) fn raw(self) -> u64 {
        (self.start.index() as u64) | ((self.end.index() as u64) << 32)
    }

    #[inline]
    pub(crate) fn from_raw(raw: u64) -> Self {
        Self {
            start: ExtraDataId::from_raw(raw as u32),
            end: ExtraDataId::from_raw((raw >> 32) as u32),
            _phantom: PhantomData,
        }
    }

    /// Similar to `Vec::remove(0)`.
    pub fn remove_first(&mut self) -> NodeExtraDataId<T> {
        assert!(!self.is_empty());
        let start = self.start;
        self.start += 1;
        NodeExtraDataId {
            inner: start,
            _phantom: PhantomData,
        }
    }

    pub fn get(self, index: usize) -> Option<NodeExtraDataId<T>> {
        if index < self.len() {
            let id = self.start + index;
            return Some(NodeExtraDataId {
                inner: id,
                _phantom: PhantomData,
            });
        }
        None
    }

    pub fn first(self) -> Option<NodeExtraDataId<T>> {
        if self.is_empty() {
            return None;
        }

        Some(NodeExtraDataId {
            inner: self.start,
            _phantom: PhantomData,
        })
    }

    pub fn last(self) -> Option<NodeExtraDataId<T>> {
        if self.is_empty() {
            return None;
        }

        Some(NodeExtraDataId {
            inner: self.end - 1,
            _phantom: PhantomData,
        })
    }

    pub fn iter(self) -> SubRangeIterator<T> {
        SubRangeIterator {
            cur: self.start,
            end: self.end,
            _phantom: PhantomData,
        }
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        if at > self.len() {
            panic!(
                "Split point {} is larger than the length {}",
                at,
                self.len()
            );
        }

        let at = self.start + at;
        let original_end = self.end;
        self.end = at;
        Self {
            start: at,
            end: original_end,
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn replace_slot<'a>(self, ast: &mut Ast<'a>, at: NodeExtraDataId<T>, new: T)
    where
        T: ExtraDataCompact<'a>,
    {
        let data = new.to_extra_data(ast);
        ast.extra_data[at.inner.index()] = data;
    }

    #[inline]
    pub fn get_node<'a>(self, ast: &Ast<'a>, index: usize) -> Option<T>
    where
        T: ExtraDataCompact<'a>,
    {
        self.get(index).map(|id| ast.get_node_in_sub_range(id))
    }
}

pub struct SubRangeIterator<T> {
    cur: ExtraDataId,
    end: ExtraDataId,
    _phantom: PhantomData<T>,
}

impl<T> Iterator for SubRangeIterator<T> {
    type Item = NodeExtraDataId<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.end {
            return None;
        }

        let next = self.cur;
        debug_assert!(next < self.end);

        self.cur = ExtraDataId::from_usize_unchecked(self.cur.index().wrapping_add(1));
        debug_assert!(self.cur <= self.end);

        Some(NodeExtraDataId {
            inner: next,
            _phantom: PhantomData,
        })
    }
}

impl<T> DoubleEndedIterator for SubRangeIterator<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.cur == self.end {
            return None;
        }

        self.end = ExtraDataId::from_usize_unchecked(self.end.index().wrapping_sub(1));
        debug_assert!(self.cur <= self.end);

        let next = self.end;
        debug_assert!(next >= self.cur);

        Some(NodeExtraDataId {
            inner: next,
            _phantom: PhantomData,
        })
    }
}

/// A typed index for a value inside a [`SubRange`].
#[derive(Debug, Clone, Copy)]
pub struct NodeExtraDataId<T> {
    inner: ExtraDataId,
    _phantom: PhantomData<T>,
}

impl<'a> Ast<'a> {
    pub fn get_node_in_sub_range<T: ExtraDataCompact<'a>>(&self, id: NodeExtraDataId<T>) -> T {
        // Safety: `NodeExtraDataId<T>` should always be valid.
        unsafe {
            T::from_extra_data(
                *self.extra_data.as_slice().get_unchecked(id.inner.index()),
                self,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_representation_roundtrips() {
        let range = SubRange::<()> {
            start: ExtraDataId::from_raw(12),
            end: ExtraDataId::from_raw(34),
            _phantom: PhantomData,
        };

        let decoded = SubRange::<()>::from_raw(range.raw());
        assert_eq!(decoded.start, range.start);
        assert_eq!(decoded.end, range.end);
    }
}
