use super::sealed::AstNode as SealedAstNode;

oxc_index::define_index_type! {
    #[repr(transparent)]
    pub struct NodeId = u32;
}

/// A generated AST handle. Implementations are restricted to this crate.
pub trait AstNode: SealedAstNode {
    fn node_id(self) -> NodeId;

    /// Creates a typed wrapper after the caller has verified the node kind.
    ///
    /// # Safety
    ///
    /// `node_id` must refer to the concrete node type represented by `Self`.
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self
    where
        Self: Sized;
}
