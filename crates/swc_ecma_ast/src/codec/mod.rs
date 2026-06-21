use std::mem;

use crate::Encoder;

pub mod ast_encode;
pub mod encode;

const HEADER_SIZE: usize = mem::size_of::<Header>();
const NODE_SIZE: usize = mem::size_of::<PackedNode>();
const EXTRA_SIZE: usize = mem::size_of::<u32>();
const NONE: u32 = u32::MAX;

pub(crate) trait AstEncode {
    fn encode(&self, encoder: &mut Encoder) -> u32;
}

#[repr(C)]
pub struct Header {
    node_count: u32,
    extra_count: u32,
    string_pool_size: u32,
}

impl Header {
    fn size(&self) -> usize {
        HEADER_SIZE
            + (self.node_count as usize) * NODE_SIZE
            + (self.extra_count as usize) * EXTRA_SIZE
            + (self.string_pool_size as usize)
    }
}

#[repr(C)]
#[derive(Default)]
struct PackedNode {
    kind: NodeKind,
    _pad0: u8,
    flags: u16,
    span_lo: u32,
    span_hi: u32,
    field0: u32,
    field1: u32,
    field2: u32,
    field3: u32,
}

#[repr(u8)]
#[derive(Default)]
enum NodeKind {
    #[default]
    Unknown = 0,
    Module,
    Script,
    Ident,
}
