oxc_index::define_index_type! {
    #[repr(transparent)]
    pub struct NodeId = u32;
}

pub trait AstNode {
    fn node_id(self) -> NodeId;

    /// Creates a typed wrapper after the caller has verified the node kind.
    ///
    /// # Safety
    ///
    /// `node_id` must refer to the concrete node type represented by `Self`.
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self
    where
        Self: Sized,
    {
        const {
            assert!(std::mem::size_of::<Self>() == std::mem::size_of::<NodeId>());
        };
        unsafe { std::mem::transmute_copy(&node_id) }
    }
}
