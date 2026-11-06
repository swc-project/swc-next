use crate::{Ast, AstNode, DUMMY_SP, NodeId};

/// Node mapping returned by [`Ast::import_synthesized`]. Only use handles from
/// the source AST supplied to that import. The returned handles belong to the
/// destination AST; source and destination can use different allocators.
#[derive(Clone, Copy)]
pub struct ImportedAst {
    start: usize,
    len: usize,
}

impl ImportedAst {
    pub fn node_id(self, node: NodeId) -> NodeId {
        assert!(node.index() < self.len);
        NodeId::from_usize_unchecked(self.start + node.index())
    }

    pub fn node<N: AstNode>(self, node: N) -> N {
        // SAFETY: Import preserves each node's kind, including union handles.
        unsafe { N::from_node_id_unchecked(self.node_id(node.node_id())) }
    }
}

impl Ast<'_> {
    /// Append an independent AST as synthesized nodes, copying its strings and
    /// lists and remapping all node handles. Keeps this AST's root, source
    /// metadata and comments; imported spans are [`DUMMY_SP`] and comments are
    /// omitted. The returned nodes are detached and have no semantic indexes.
    pub fn import_synthesized(&mut self, source: &Ast<'_>) -> ImportedAst {
        let mapping = ImportedAst {
            start: self.node_count(),
            len: source.node_count(),
        };
        assert!(
            mapping
                .start
                .checked_add(mapping.len)
                .is_some_and(|end| end < u32::MAX as usize)
        );
        // Reserve every identity before importing edges, including forward edges.
        for _ in 0..mapping.len {
            self.empty_statement(DUMMY_SP);
        }
        self.import_nodes(source, mapping);
        mapping
    }
}
