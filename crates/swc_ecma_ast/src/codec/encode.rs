use crate::{
    Program,
    codec::{AstEncode, Header, NONE, PackedNode},
};
use swc_experimental_allocator::atom::{Atom, Wtf8Atom};

pub struct Encoder {
    start_node_ptr: *mut PackedNode,
    node_ptr: *mut PackedNode,
    start_extra_ptr: *mut u32,
    extra_ptr: *mut u32,
    start_string_pool_ptr: *mut u8,
    string_pool_ptr: *mut u8,
}

impl Encoder {
    pub fn new(header: Header, buf: &mut [u8]) -> Option<Self> {
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
            start_node_ptr: node_ptr,
            node_ptr,
            start_extra_ptr: extra_ptr,
            extra_ptr,
            start_string_pool_ptr: string_pool_ptr,
            string_pool_ptr,
        })
    }

    pub fn encode(&mut self, ast: &Program) {
        ast.encode(self);
    }
}

impl Encoder {
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
        let start = unsafe {
            // SAFETY: Both pointers are derived from the same extra area, and
            // `extra_ptr` always points at the next extra slot.
            self.extra_ptr.offset_from(self.start_extra_ptr) as u32
        };
        let len = nodes.len() as u32;

        for node in nodes {
            let node_id = node.encode(self);
            self.extra(node_id);
        }

        (start, len)
    }

    fn extra(&mut self, value: u32) {
        unsafe {
            // SAFETY: `new` positions `extra_ptr` at the current slot in the
            // extra area reserved by the caller-provided header.
            self.extra_ptr.write(value);
            // SAFETY: The header protocol reserves enough extra slots for all
            // calls to this method. Advancing one serialized `u32` stays within
            // or one-past that extra area.
            self.extra_ptr = self.extra_ptr.add(1);
        }
    }
}

impl Encoder {
    pub(super) fn utf8(&mut self, atom: Atom) -> (u32, u32) {
        self.bytes(atom.as_str().as_bytes())
    }

    pub(super) fn opt_utf8(&mut self, atom: Option<Atom>) -> (u32, u32) {
        match atom {
            Some(atom) => self.utf8(atom),
            None => (NONE, 0),
        }
    }

    pub(super) fn wtf8(&mut self, atom: Wtf8Atom) -> (u32, u32) {
        self.bytes(atom.as_wtf8().as_bytes())
    }

    pub(super) fn opt_wtf8(&mut self, atom: Option<Wtf8Atom>) -> (u32, u32) {
        match atom {
            Some(atom) => self.wtf8(atom),
            None => (NONE, 0),
        }
    }

    fn bytes(&mut self, bytes: &[u8]) -> (u32, u32) {
        let start = unsafe {
            // SAFETY: Both pointers are derived from the same string pool, and
            // `string_pool_ptr` always points at the next byte to write.
            self.string_pool_ptr.offset_from(self.start_string_pool_ptr) as u32
        };
        let len = bytes.len() as u32;

        unsafe {
            // SAFETY: `new` positions `string_pool_ptr` at the current byte in
            // the string pool reserved by the caller-provided header. Source
            // and destination are disjoint because `bytes` comes from the AST
            // while the destination is the caller-provided output buffer.
            bytes
                .as_ptr()
                .copy_to_nonoverlapping(self.string_pool_ptr, bytes.len());
            // SAFETY: The header protocol reserves enough string-pool bytes for
            // all calls to this method. Advancing by the copied length stays
            // within or one-past that string pool.
            self.string_pool_ptr = self.string_pool_ptr.add(bytes.len());
        }

        (start, len)
    }
}
