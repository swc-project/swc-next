oxc_index::define_index_type! {
    pub struct NodeId = u32;
}

pub trait AstNode {
    fn node_id(self) -> NodeId;
}
