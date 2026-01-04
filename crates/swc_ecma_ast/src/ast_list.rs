use std::ops::{Index, IndexMut};

use oxc_index::IndexVec;

use crate::{AstNode, NodeId, NodeKind};

/// A generational arena of AST nodes.
#[derive(Default)]
pub(crate) struct NodeList {
    inner: IndexVec<NodeId, AstNode>,

    free_list: Vec<NodeId>,

    /// The number of elements in the arena, except the freed nodes.
    num_elems: u32,
}

impl NodeList {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: IndexVec::with_capacity(capacity),
            free_list: Default::default(),
            num_elems: Default::default(),
        }
    }

    #[inline]
    /// Add a new node to the arena.
    ///
    /// This function will reuse a freed node slot if available.
    pub fn add_node(&mut self, node: AstNode) -> NodeId {
        self.num_elems += 1;

        // We first check if there is a reusable slot in the free list.
        match self.free_list.pop() {
            Some(node_id) => {
                debug_assert!(self.inner[node_id].kind == NodeKind::__FREED);
                self.inner[node_id] = node;
                node_id
            }
            None => self.inner.push(node),
        }
    }

    #[inline]
    /// Free a node in the arena and update the free head.
    pub fn free_node(&mut self, node_id: NodeId) {
        debug_assert!(self.inner[node_id].kind != NodeKind::__FREED);

        // Mark the node as freed and save the next free node to the node's data.
        self.num_elems -= 1;
        self.inner[node_id].kind = NodeKind::__FREED;
        self.free_list.push(node_id);
    }

    #[inline]
    pub fn replace_node(&mut self, dest: NodeId, src: NodeId) {
        self.inner[dest] = self.inner[src].clone();
        self.free_node(src);
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &AstNode)> {
        self.inner.iter_enumerated()
    }

    #[inline]
    /// The number of elements in the arena, except the freed nodes.
    pub fn len(&self) -> u32 {
        self.num_elems
    }

    #[inline]
    /// The total length of the arena.
    pub fn capacity(&self) -> usize {
        self.inner.len()
    }
}

impl NodeList {
    /// Get a reference to a node in the arena without boundary check.
    ///
    /// # Safety:
    /// 1. The node_id must be valid.
    pub(crate) unsafe fn get_unchecked(&self, node_id: NodeId) -> &AstNode {
        debug_assert!(node_id.index() < self.inner.len());
        unsafe { self.inner.as_raw_slice().get_unchecked(node_id.index()) }
    }

    /// Get a mutable reference to a node in the arena without boundary check.
    ///
    /// # Safety:
    /// 1. The node_id must be valid.
    pub(crate) unsafe fn get_unchecked_mut(&mut self, node_id: NodeId) -> &mut AstNode {
        unsafe {
            self.inner
                .as_raw_slice_mut()
                .get_unchecked_mut(node_id.index())
        }
    }
}

impl Index<NodeId> for NodeList {
    type Output = AstNode;

    fn index(&self, k: NodeId) -> &AstNode {
        &self.inner[k]
    }
}

impl IndexMut<NodeId> for NodeList {
    fn index_mut(&mut self, k: NodeId) -> &mut AstNode {
        &mut self.inner[k]
    }
}
