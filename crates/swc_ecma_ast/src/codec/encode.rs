use crate::{
    Program, Span,
    codec::{AstEncode, Header, NONE, PackedNode, utf8_to_utf16::PositionMap},
};
use swc_experimental_allocator::atom::{Atom, Wtf8Atom};

pub struct Encoder<'a> {
    source: &'a str,
    pos_map: PositionMap,
    start_node_ptr: *mut PackedNode,
    node_ptr: *mut PackedNode,
    start_extra_ptr: *mut u32,
    extra_ptr: *mut u32,
    start_string_pool_ptr: *mut u8,
    string_pool_ptr: *mut u8,
    scratch: Vec<u32>,
}

impl<'a> Encoder<'a> {
    pub fn new(header: Header, source: &'a str, buf: &mut [u8]) -> Option<Self> {
        let size = header.size();
        if buf.len() < size {
            return None;
        }

        let header_ptr = buf.as_mut_ptr() as *mut Header;
        let node_ptr = unsafe {
            // SAFETY: `size <= buf.len()` was checked above, and the node area
            // starts immediately after the header.
            header_ptr.add(1)
        } as *mut PackedNode;
        let extra_ptr = unsafe {
            // SAFETY: `node_ptr` points at the start of the node area, and the
            // header reserves `node_count` packed nodes before the extra area.
            node_ptr.add(header.node_count as usize)
        } as *mut u32;
        let string_pool_ptr = unsafe {
            // SAFETY: `extra_ptr` points at the start of the extra area, and
            // the header reserves `extra_count` u32 entries before strings.
            extra_ptr.add(header.extra_count as usize)
        } as *mut u8;

        unsafe {
            // SAFETY: `header_ptr` points to the beginning of `buf`, and the
            // buffer is at least `HEADER_SIZE` bytes long because `size`
            // includes the header.
            header_ptr.write(header);
        }

        Some(Self {
            source,
            pos_map: PositionMap::new(source),
            start_node_ptr: node_ptr,
            node_ptr,
            start_extra_ptr: extra_ptr,
            extra_ptr,
            start_string_pool_ptr: string_pool_ptr,
            string_pool_ptr,
            scratch: Vec::new(),
        })
    }

    pub fn encode(&mut self, ast: &Program) {
        ast.encode(self);
    }
}

impl Encoder<'_> {
    pub(super) fn packed_node(&mut self, node: PackedNode) -> u32 {
        unsafe {
            // SAFETY: `new` positions `node_ptr` at the current slot in the
            // node area reserved by the caller-provided header.
            self.node_ptr.write(node);
            // SAFETY: The header protocol reserves enough node slots for all
            // calls to this method. Advancing one serialized node stays within
            // or one-past that node area.
            self.node_ptr = self.node_ptr.add(1);
            // SAFETY: Both pointers are derived from the same node area. The
            // result follows the original encoder protocol: node ids are
            // one-based because the cursor is advanced before measuring.
            self.node_ptr.offset_from(self.start_node_ptr) as u32
        }
    }

    pub(super) fn nodes<E: AstEncode>(&mut self, nodes: &[E]) -> (u32, u32) {
        let scratch_start = self.scratch.len();

        for node in nodes {
            let node_id = node.encode(self);
            self.scratch.push(node_id);
        }

        let start = unsafe {
            // SAFETY: Both pointers are derived from the same extra area, and
            // `extra_ptr` always points at the next extra slot.
            self.extra_ptr.offset_from(self.start_extra_ptr) as u32
        };
        let len = nodes.len() as u32;

        for &node_id in &self.scratch[scratch_start..] {
            unsafe {
                // SAFETY: `new` positions `extra_ptr` at the current slot in the
                // extra area reserved by the caller-provided header.
                self.extra_ptr.write(node_id);
                // SAFETY: The header protocol reserves enough extra slots for all
                // calls to this method. Advancing one serialized `u32` stays within
                // or one-past that extra area.
                self.extra_ptr = self.extra_ptr.add(1);
            }
        }

        self.scratch.truncate(scratch_start);
        (start, len)
    }
}

impl Encoder<'_> {
    pub(super) fn span(&self, span: Span) -> (u32, u32) {
        (
            self.pos_map.convert(span.start.saturating_sub(1)),
            self.pos_map.convert(span.end.saturating_sub(1)),
        )
    }

    pub(super) fn opt_span(&self, span: Option<Span>) -> (u32, u32) {
        match span {
            Some(span) => self.span(span),
            None => (NONE, 0),
        }
    }

    pub(super) fn utf8(&mut self, atom: Atom) -> (u32, u32) {
        let s = atom.as_str();
        self.bytes(s.as_bytes(), s.encode_utf16())
    }

    pub(super) fn opt_utf8(&mut self, atom: Option<Atom>) -> (u32, u32) {
        match atom {
            Some(atom) => self.utf8(atom),
            None => (NONE, 0),
        }
    }

    pub(super) fn wtf8(&mut self, atom: Wtf8Atom) -> (u32, u32) {
        let wtf8 = atom.as_wtf8();
        self.bytes(wtf8.as_bytes(), wtf8.to_ill_formed_utf16())
    }

    pub(super) fn opt_wtf8(&mut self, atom: Option<Wtf8Atom>) -> (u32, u32) {
        match atom {
            Some(atom) => self.wtf8(atom),
            None => (NONE, 0),
        }
    }

    fn bytes<I>(&mut self, bytes: &[u8], utf16: I) -> (u32, u32)
    where
        I: IntoIterator<Item = u16>,
    {
        let h_start = self.source.as_ptr() as usize;
        let h_end = h_start + self.source.len();
        let n_start = bytes.as_ptr() as usize;
        let n_end = n_start + bytes.len();

        if h_start <= n_start && n_end <= h_end {
            let offset = (n_start - h_start) as u32;
            return (
                self.pos_map.convert(offset),
                self.pos_map.convert(offset + bytes.len() as u32),
            );
        }

        let pool_start = unsafe {
            // SAFETY: Both pointers are derived from the same string pool, and
            // `string_pool_ptr` always points at the next byte to write.
            self.string_pool_ptr.offset_from(self.start_string_pool_ptr) as u32
        } / 2;
        let start = self.pos_map.source_len() + pool_start;
        let mut len = 0;

        for code_unit in utf16 {
            let bytes = code_unit.to_le_bytes();
            unsafe {
                // SAFETY: `new` positions `string_pool_ptr` at the current byte in
                // the string pool reserved by the caller-provided header.
                bytes
                    .as_ptr()
                    .copy_to_nonoverlapping(self.string_pool_ptr, bytes.len());
                // SAFETY: The header protocol reserves enough string-pool bytes for
                // all calls to this method. Advancing by the copied length stays
                // within or one-past that string pool.
                self.string_pool_ptr = self.string_pool_ptr.add(bytes.len());
            }
            len += 1;
        }

        (start, start + len)
    }
}
