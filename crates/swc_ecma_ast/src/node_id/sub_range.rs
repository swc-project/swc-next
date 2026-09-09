use std::marker::PhantomData;

use crate::{
    Ast,
    node_id::{ExtraDataId, ExtraDataValue},
};

/// A typed range of values in the AST extra-data arena.
///
/// It only contains the start and end of the range in [crate::ast_list::NodeList].
#[repr(C)]
#[derive(Debug, Hash)]
pub struct SubRange<T> {
    pub(crate) start: ExtraDataId,
    pub(crate) end: ExtraDataId,
    _phantom: PhantomData<T>,
}

impl<T> Clone for SubRange<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SubRange<T> {}

impl<T> SubRange<T> {
    pub(crate) fn new(start: ExtraDataId, end: ExtraDataId) -> Self {
        Self {
            start,
            end,
            _phantom: PhantomData,
        }
    }

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
        T: ExtraDataValue,
    {
        // SAFETY: A typed sub-range index identifies a slot assigned to `T`.
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(at.inner.index()) = new.into_extra_data();
        };
    }

    #[inline]
    pub fn get_node<'a>(self, ast: &Ast<'a>, index: usize) -> Option<T>
    where
        T: ExtraDataValue,
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
    pub fn get_node_in_sub_range<T: ExtraDataValue>(&self, id: NodeExtraDataId<T>) -> T {
        // SAFETY: A typed sub-range index identifies a slot containing `T`.
        unsafe { T::from_extra_data(self.extra_data.as_slice().get_unchecked(id.inner.index())) }
    }
}
