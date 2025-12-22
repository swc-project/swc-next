use std::{marker::PhantomData, ops::Deref};

use crate::{Ast, AstNode, NodeIdTrait, node_id::ExtraDataId};

pub const EMPTY_SUB_RANGE: SubRange = SubRange {
    start: ExtraDataId::from_raw(0),
    end: ExtraDataId::from_raw(0),
};

/// [SubRange] is a cheap way to represent a undetermined range of nodes in an arena, just like a [std::vec::Vec].
///
/// It only contains the start and end of the range in [crate::ast_list::NodeList].
#[derive(Debug, Clone, Copy, Hash)]
pub struct SubRange {
    pub start: ExtraDataId,
    pub end: ExtraDataId,
}

impl SubRange {
    /// # Safety:
    /// 1. The caller should make sure that the nodes in the range are all of type `T`.
    pub(crate) unsafe fn cast_to_typed<T>(self) -> TypedSubRange<T> {
        TypedSubRange {
            inner: self,
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// An 8 bytes optimized version of `Option<SubRange>` (12 bytes).
///
/// We regard it as `None` if `OptionalSubRange::end` is `u32::MAX`.
#[derive(Debug, Clone, Copy, Hash)]
pub struct OptionalSubRange {
    pub start: ExtraDataId,
    pub end: ExtraDataId,
}

impl OptionalSubRange {
    /// # Safety:
    /// 1. The caller should make sure that the nodes in the range are all of type `T`.
    pub(crate) unsafe fn cast_to_typed<T>(self) -> OptionalTypedSubRange<T> {
        OptionalTypedSubRange {
            inner: self,
            _phantom: PhantomData,
        }
    }

    pub const fn new_none() -> Self {
        Self {
            start: ExtraDataId::from_raw(0),
            end: ExtraDataId::from_raw(u32::MAX),
        }
    }

    pub const fn is_none(&self) -> bool {
        self.end.raw() == u32::MAX
    }

    pub const fn unwrap(self) -> SubRange {
        assert!(!self.is_none());
        SubRange {
            start: self.start,
            end: self.end,
        }
    }
}

impl From<SubRange> for OptionalSubRange {
    fn from(value: SubRange) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<Option<SubRange>> for OptionalSubRange {
    fn from(value: Option<SubRange>) -> Self {
        match value {
            Some(value) => value.into(),
            None => OptionalSubRange::new_none(),
        }
    }
}

/// A typed sub range of nodes in an arena.
///
/// See [SubRange]
#[derive(Debug, Clone, Copy)]
pub struct TypedSubRange<T> {
    pub(crate) inner: SubRange,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T> TypedSubRange<T> {
    pub fn empty() -> Self {
        Self {
            inner: EMPTY_SUB_RANGE,
            _phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.end.index() - self.start.index()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Similar to `Vec::remove(0)`
    pub fn remove_first(&mut self) -> NodeExtraDataId<T> {
        assert!(!self.is_empty());
        let start = self.start;
        self.inner.start += 1;
        NodeExtraDataId {
            inner: start,
            _phantom: PhantomData,
        }
    }

    pub fn get(&self, index: usize) -> NodeExtraDataId<T> {
        assert!(index < self.len());
        let id = self.start + index;
        NodeExtraDataId {
            inner: id,
            _phantom: PhantomData,
        }
    }

    pub fn first(&self) -> Option<NodeExtraDataId<T>> {
        if self.is_empty() {
            return None;
        }

        Some(NodeExtraDataId {
            inner: self.start,
            _phantom: PhantomData,
        })
    }

    pub fn last(&self) -> Option<NodeExtraDataId<T>> {
        if self.is_empty() {
            return None;
        }

        Some(NodeExtraDataId {
            inner: self.end - 1,
            _phantom: PhantomData,
        })
    }

    pub fn iter(&self) -> TypedSubRangeIterator<T> {
        TypedSubRangeIterator {
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
        let original_end = self.inner.end;

        self.inner.end = at;
        TypedSubRange {
            inner: SubRange {
                start: at,
                end: original_end,
            },
            _phantom: PhantomData,
        }
    }
}

impl<T> From<TypedSubRange<T>> for SubRange {
    fn from(value: TypedSubRange<T>) -> Self {
        value.inner
    }
}
impl<T> Deref for TypedSubRange<T> {
    type Target = SubRange;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// An 8 bytes optimized version of `Option<TypedSubRange<T>>` (12 bytes).
///
/// We regard it as `None` if `OptionalTypedSubRange::end` is `u32::MAX`.
#[derive(Debug, Clone, Copy, Hash)]
pub struct OptionalTypedSubRange<T> {
    pub(crate) inner: OptionalSubRange,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T> OptionalTypedSubRange<T> {
    pub const fn to_option(&self) -> Option<TypedSubRange<T>> {
        if self.inner.is_none() {
            return None;
        }
        Some(TypedSubRange {
            inner: self.inner.unwrap(),
            _phantom: PhantomData,
        })
    }
}

pub struct TypedSubRangeIterator<T> {
    cur: ExtraDataId,
    end: ExtraDataId,
    _phantom: PhantomData<T>,
}

impl<T: NodeIdTrait> Iterator for TypedSubRangeIterator<T> {
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

impl<T: NodeIdTrait> DoubleEndedIterator for TypedSubRangeIterator<T> {
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

impl<T: NodeIdTrait> Iterator for TypedSubRangeIterator<Option<T>> {
    type Item = NodeExtraDataId<Option<T>>;

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

impl<T: NodeIdTrait> DoubleEndedIterator for TypedSubRangeIterator<Option<T>> {
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

/// This is a wrapper around `ExtraDataId` that is used to represent a node in a sub-range.
/// It's type safe and can be used to access nodes in the AST context.
#[derive(Debug, Clone, Copy)]
pub struct NodeExtraDataId<T> {
    inner: ExtraDataId,
    _phantom: PhantomData<T>,
}

impl Ast {
    pub fn get_node_in_sub_range<T: NodeIdTrait>(&self, id: NodeExtraDataId<T>) -> T {
        let node_id = unsafe { self.extra_data[id.inner].node };

        // Safety: `NodeExtraDataId<T>` should always be valid
        unsafe { T::from_node_id_unchecked(node_id, self) }
    }

    pub fn get_opt_node_in_sub_range<T: NodeIdTrait>(
        &self,
        id: NodeExtraDataId<Option<T>>,
    ) -> Option<T> {
        let opt_node_id = unsafe { self.extra_data[id.inner].optional_node };
        opt_node_id.map(|node_id| unsafe { T::from_node_id_unchecked(node_id, self) })
    }

    pub fn get_raw_node_in_sub_range<T>(&self, id: NodeExtraDataId<T>) -> &AstNode {
        let node_id = unsafe { self.extra_data[id.inner].node };
        &self.nodes[node_id]
    }
}
