use crate::{
    Program, Span,
    codec::{AstDecode, EXTRA_SIZE, HEADER_SIZE, NODE_SIZE, NONE},
};
use swc_experimental_allocator::{
    Allocator,
    atom::{Atom, Wtf8Atom},
    boxed::Box,
    vec::Vec,
    wtf8::Wtf8,
};

pub fn decode<'a>(allocator: &'a Allocator, buf: &[u8]) -> Program<'a> {
    let decoder = Decoder::new(allocator, buf);
    AstDecode::decode(&decoder, decoder.root_id())
}

impl<'a, D> AstDecode<'a> for Box<'a, D>
where
    D: AstDecode<'a> + 'a,
{
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        decoder.allocator.boxed(D::decode(decoder, id))
    }
}

impl<'a, D> AstDecode<'a> for Option<D>
where
    D: AstDecode<'a>,
{
    fn decode(decoder: &Decoder<'a, '_>, id: u32) -> Self {
        if id == NONE {
            None
        } else {
            Some(D::decode(decoder, id))
        }
    }
}

pub(crate) struct Decoder<'a, 'buf> {
    allocator: &'a Allocator,
    buf: &'buf [u8],
    node_count: u32,
    extra_count: u32,
    string_pool_size: u32,
    nodes_start: usize,
    extra_start: usize,
    string_pool_start: usize,
}

impl<'a, 'buf> Decoder<'a, 'buf> {
    fn new(allocator: &'a Allocator, buf: &'buf [u8]) -> Self {
        assert!(
            buf.len() >= HEADER_SIZE,
            "buffer is smaller than the protocol header"
        );

        let node_count = read_u32(buf, 0);
        let extra_count = read_u32(buf, 4);
        let string_pool_size = read_u32(buf, 8);
        let nodes_start = HEADER_SIZE;
        let extra_start = nodes_start + node_count as usize * NODE_SIZE;
        let string_pool_start = extra_start + extra_count as usize * EXTRA_SIZE;
        let size = string_pool_start + string_pool_size as usize;

        assert!(
            buf.len() >= size,
            "buffer is smaller than the encoded AST size"
        );

        Self {
            allocator,
            buf,
            node_count,
            extra_count,
            string_pool_size,
            nodes_start,
            extra_start,
            string_pool_start,
        }
    }

    pub(crate) fn root_id(&self) -> u32 {
        self.node_count
    }

    pub(crate) fn node(&self, id: u32) -> PackedNode {
        assert!(id != 0 && id != NONE, "invalid node id {id}");
        assert!(id <= self.node_count, "node id {id} is out of bounds");

        let offset = self.nodes_start + (id as usize - 1) * NODE_SIZE;
        PackedNode {
            kind: self.buf[offset],
            flags: read_u16(self.buf, offset + 2),
            span_lo: read_u32(self.buf, offset + 4),
            span_hi: read_u32(self.buf, offset + 8),
            field0: read_u32(self.buf, offset + 12),
            field1: read_u32(self.buf, offset + 16),
            field2: read_u32(self.buf, offset + 20),
            field3: read_u32(self.buf, offset + 24),
            field4: read_u32(self.buf, offset + 28),
        }
    }

    fn extra(&self, index: u32) -> u32 {
        assert!(
            index < self.extra_count,
            "extra index {index} is out of bounds"
        );
        read_u32(self.buf, self.extra_start + index as usize * EXTRA_SIZE)
    }

    pub(crate) fn nodes<T>(&self, start: u32, len: u32) -> Vec<'a, T>
    where
        T: AstDecode<'a> + 'a,
    {
        let mut out = Vec::with_capacity_in(len as usize, self.allocator);
        for i in 0..len {
            out.push(T::decode(self, self.extra(start + i)));
        }
        out
    }

    pub(crate) fn span(&self, node: PackedNode) -> Span {
        Span {
            start: node.span_lo,
            end: node.span_hi,
        }
    }

    pub(crate) fn atom(&self, start: u32, end: u32) -> Atom<'a> {
        Atom::new_in(
            std::str::from_utf8(self.string(start, end)).expect("invalid UTF-8 atom"),
            self.allocator,
        )
    }

    pub(crate) fn opt_atom(&self, start: u32, end: u32) -> Option<Atom<'a>> {
        if start == NONE {
            None
        } else {
            Some(self.atom(start, end))
        }
    }

    pub(crate) fn wtf8_atom(&self, start: u32, end: u32) -> Wtf8Atom<'a> {
        Wtf8Atom::new_in(
            Wtf8::from_bytes(self.string(start, end)).expect("invalid WTF-8 atom"),
            self.allocator,
        )
    }

    pub(crate) fn opt_wtf8_atom(&self, start: u32, end: u32) -> Option<Wtf8Atom<'a>> {
        if start == NONE {
            None
        } else {
            Some(self.wtf8_atom(start, end))
        }
    }

    fn string(&self, start: u32, end: u32) -> &[u8] {
        assert!(start <= end, "invalid string range {start}..{end}");
        assert!(
            end <= self.string_pool_size,
            "string range {start}..{end} is out of bounds"
        );

        &self.buf[self.string_pool_start + start as usize..self.string_pool_start + end as usize]
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PackedNode {
    pub(crate) kind: u8,
    pub(crate) flags: u16,
    span_lo: u32,
    span_hi: u32,
    pub(crate) field0: u32,
    pub(crate) field1: u32,
    pub(crate) field2: u32,
    pub(crate) field3: u32,
    pub(crate) field4: u32,
}

fn read_u16(buf: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([buf[offset], buf[offset + 1]])
}

fn read_u32(buf: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ])
}
