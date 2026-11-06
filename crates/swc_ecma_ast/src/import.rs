use crate::{Ast, AstNode, DUMMY_SP, NodeId, NodePayload};

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
        let extra_data_len = self.extra_data_count();
        let mut import = ImportGuard {
            ast: self,
            start: mapping.start,
            extra_data_len,
            complete: false,
        };
        // Reserve each node with its final kind so remapped forward edges
        // always have the right type. No payload is read before it is copied.
        for index in 0..mapping.len {
            let kind = source.node_kind(NodeId::from_usize_unchecked(index));
            import
                .ast
                .add_node(DUMMY_SP, kind, NodePayload { raw: [0; 16] });
        }
        import.ast.import_nodes(source, mapping);
        import.complete = true;
        mapping
    }
}

struct ImportGuard<'s, 'a> {
    ast: &'s mut Ast<'a>,
    start: usize,
    extra_data_len: usize,
    complete: bool,
}

impl Drop for ImportGuard<'_, '_> {
    fn drop(&mut self) {
        if !self.complete {
            // SAFETY: Import only appends detached nodes and fields. No
            // retained node references these and no new handles have escaped.
            unsafe { self.ast.truncate(self.start, self.extra_data_len) };
        }
    }
}
