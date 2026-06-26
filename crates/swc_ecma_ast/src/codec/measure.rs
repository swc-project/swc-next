use swc_experimental_allocator::atom::{Atom, Wtf8Atom};

use crate::{
    Program,
    codec::{AstMeasure, Header},
};

impl Header {
    pub fn measure(ast: &Program, source: &str) -> Self {
        let mut header = Header {
            extra_count: 0,
            node_count: 0,
            string_pool_size: source.len() as u32,
        };
        ast.measure(&mut header);
        header
    }
}

impl Header {
    pub(super) fn node(&mut self) {
        self.node_count += 1;
    }

    pub(super) fn utf8(&mut self, _atom: Atom) {}

    pub(super) fn wtf8(&mut self, _atom: Wtf8Atom) {}

    pub(super) fn nodes<M: AstMeasure>(&mut self, nodes: &[M]) {
        self.extra_count += nodes.len() as u32;
        for node in nodes {
            node.measure(self);
        }
    }

    pub(super) fn opt_utf8(&mut self, atom: Option<Atom>) {
        match atom {
            Some(atom) => self.utf8(atom),
            None => {}
        }
    }

    pub(super) fn opt_wtf8(&mut self, atom: Option<Wtf8Atom>) {
        match atom {
            Some(atom) => self.wtf8(atom),
            None => {}
        }
    }
}
