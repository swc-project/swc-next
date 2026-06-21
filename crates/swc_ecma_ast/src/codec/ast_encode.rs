use crate::{
    Encoder, GetSpan, Ident, Module, ModuleItem, Program, Script, Stmt,
    codec::{AstEncode, NodeKind, PackedNode},
};

impl AstEncode for Program<'_> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        match self {
            Program::Module(it) => it.encode(encoder),
            Program::Script(it) => it.encode(encoder),
        }
    }
}

impl AstEncode for Module<'_> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (field0, field1) = encoder.nodes(&self.body);
        let (field2, field3) = encoder.opt_utf8(self.shebang);
        encoder.packed_node(PackedNode {
            kind: NodeKind::Module,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}

impl AstEncode for Script<'_> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (field0, field1) = encoder.nodes(&self.body);
        let (field2, field3) = encoder.opt_utf8(self.shebang);
        encoder.packed_node(PackedNode {
            kind: NodeKind::Script,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            field2,
            field3,
            ..Default::default()
        })
    }
}

impl AstEncode for ModuleItem<'_> {
    fn encode(&self, _encoder: &mut Encoder) -> u32 {
        todo!()
    }
}

impl AstEncode for Stmt<'_> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        todo!()
    }
}

impl AstEncode for Ident<'_> {
    fn encode(&self, encoder: &mut Encoder) -> u32 {
        let (field0, field1) = encoder.utf8(self.sym);
        // TODO: optional -> flags
        encoder.packed_node(PackedNode {
            kind: NodeKind::Ident,
            span_lo: self.span_lo(),
            span_hi: self.span_hi(),
            field0,
            field1,
            ..Default::default()
        })
    }
}
