use crate::Ast;

pub trait NodeIdTrait: Copy {
    /// Get node id from node
    fn node_id(self) -> NodeId;

    /// # Safety
    /// 1. The caller should ensure that `node.kind` is corresponding to `Self`
    unsafe fn from_node_id_unchecked(id: NodeId, ast: &Ast<'_>) -> Self;
}

oxc_index::define_index_type! {
    pub struct NodeId = u32;
}

/// An 4 bytes optimized version of `Option<NodeId>` (8 bytes)
///
/// We regard it as `None` if the inner is `u32::MAX`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OptionalNodeId(u32);

impl From<NodeId> for OptionalNodeId {
    fn from(value: NodeId) -> Self {
        Self(value.0)
    }
}

impl From<Option<NodeId>> for OptionalNodeId {
    fn from(value: Option<NodeId>) -> Self {
        match value {
            Some(value) => Self(value.0),
            None => OptionalNodeId::none(),
        }
    }
}

impl OptionalNodeId {
    #[inline]
    pub fn none() -> Self {
        Self(u32::MAX)
    }

    #[inline]
    pub fn to_option(self) -> Option<NodeId> {
        if self.0 == u32::MAX {
            return None;
        }

        Some(NodeId(self.0))
    }

    #[inline]
    pub fn map<U, F>(self, f: F) -> Option<U>
    where
        F: FnOnce(NodeId) -> U,
    {
        if self.0 == u32::MAX {
            return None;
        }

        Some(f(NodeId(self.0)))
    }

    /// Get the raw u32 value for inline storage
    #[inline]
    pub fn into_raw(self) -> u32 {
        self.0
    }

    /// Create from raw u32 value (for inline storage deserialization)
    #[inline]
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }
}
