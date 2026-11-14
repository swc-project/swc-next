use std::marker::PhantomData;

use crate::{Ast, AstNode, ExtraDataId, ExtraDataUnit, NodeId};

use super::sealed::SubRangeNode as SealedSubRangeNode;

/// Internal dispatch for node and optional-node entries in extra-data lists.
///
/// Implementations are restricted to nodes and optional nodes in this crate.
/// This trait is not re-exported from the AST crate.
pub trait SubRangeNode: Sized + SealedSubRangeNode {
    fn into_raw_node_id(self) -> u32;

    /// # Safety
    /// `raw` must have been stored for `Self` in the same AST.
    unsafe fn from_raw_node_id(raw: u32) -> Self;
}

impl<T: AstNode> SealedSubRangeNode for T {}

impl<T: AstNode> SubRangeNode for T {
    #[inline]
    fn into_raw_node_id(self) -> u32 {
        self.node_id().index() as u32
    }

    #[inline]
    unsafe fn from_raw_node_id(raw: u32) -> Self {
        unsafe { T::from_node_id_unchecked(NodeId::from_raw_unchecked(raw)) }
    }
}

impl<T: AstNode> SealedSubRangeNode for Option<T> {}

impl<T: AstNode> SubRangeNode for Option<T> {
    #[inline]
    fn into_raw_node_id(self) -> u32 {
        self.map_or(u32::MAX, |node| node.node_id().index() as u32)
    }

    #[inline]
    unsafe fn from_raw_node_id(raw: u32) -> Self {
        (raw != u32::MAX)
            .then(|| unsafe { T::from_node_id_unchecked(NodeId::from_raw_unchecked(raw)) })
    }
}

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
}

impl<T: SubRangeNode> SubRange<T> {
    #[inline]
    pub fn replace_slot<'a>(self, ast: &mut Ast<'a>, at: NodeExtraDataId<T>, new: T) {
        // SAFETY: A typed sub-range index identifies a slot assigned to `T`.
        unsafe {
            *ast.extra_data.get_unchecked_mut(at.inner) =
                ExtraDataUnit::from_raw_node_id(new.into_raw_node_id());
        };
    }

    #[inline]
    pub fn get_node<'a>(self, ast: &Ast<'a>, index: usize) -> Option<T> {
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
    pub fn get_node_in_sub_range<T: SubRangeNode>(&self, id: NodeExtraDataId<T>) -> T {
        // SAFETY: A typed sub-range index identifies a slot containing `T`.
        unsafe { T::from_raw_node_id(self.extra_data.get_unchecked(id.inner).optional_node_id) }
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;

    use crate::{AstCtx, AstNode, Span};

    #[test]
    fn node_lists() {
        let allocator = Allocator::new();
        let mut context = AstCtx::new("", &allocator);
        // SAFETY: Every handle below is created and used within this AST.
        let ast = unsafe { context.ast_mut() };
        let first = ast.stmt_empty_statement(Span::new(0, 0));
        let second = ast.stmt_empty_statement(Span::new(0, 0));

        let nodes = ast.add_sub_range([first]);
        assert_eq!(
            ast.get_node_in_sub_range(nodes.first().unwrap()).node_id(),
            first.node_id()
        );
        nodes.replace_slot(ast, nodes.first().unwrap(), second);
        assert_eq!(nodes.get_node(ast, 0).unwrap().node_id(), second.node_id());

        let nodes = ast.add_sub_range([Some(first), None]);
        assert_eq!(
            ast.get_node_in_sub_range(nodes.first().unwrap())
                .map(AstNode::node_id),
            Some(first.node_id())
        );
        assert!(ast.get_node_in_sub_range(nodes.last().unwrap()).is_none());
        nodes.replace_slot(ast, nodes.last().unwrap(), Some(second));
        assert_eq!(
            nodes.get_node(ast, 1).unwrap().map(AstNode::node_id),
            Some(second.node_id())
        );
        nodes.replace_slot(ast, nodes.first().unwrap(), None);
        assert!(nodes.get_node(ast, 0).unwrap().is_none());
    }
}
