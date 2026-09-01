//! Binary AST transfer format used by the JavaScript FFI.
//!
//! The AST already stores nodes as three structure-of-arrays sections, so the
//! wire format preserves that representation instead of packing every logical
//! field again. JavaScript can reconstruct fields from `kind`, `payload`, and
//! `extra_data` using the same generated AST schema.
//!
//! Layout, in order:
//! - 36-byte header;
//! - `node_count` UTF-8 byte spans (`start: u32`, `end: u32`);
//! - `node_count` one-byte node kinds, padded to four bytes;
//! - `node_count` 16-byte `NodePayload` values;
//! - `extra_count` 8-byte `ExtraData` values;
//! - the raw extra string pool, padded to four bytes;
//! - `comment_count` 16-byte [`Comment`] records in source order.
//!   Each record contains the full start, full end, attachment offset,
//!   kind (`0` line, `1` block), and attachment position.
//!
//! Attached comments and diagnostics are intentionally omitted for now. Their
//! counts in the header are always zero.
//!
//! All multi-byte integers are little-endian, independent of the host target.

use std::{
    mem::{offset_of, size_of},
    slice,
};

use crate::{Ast, Comment, CommentKind, CommentPosition, ExtraData, NodeKind, NodePayload, Span};

const HEADER_SIZE: usize = 36;
const SPAN_SIZE: usize = 8;
const NODE_KIND_SIZE: usize = 1;
const NODE_PAYLOAD_SIZE: usize = 16;
const EXTRA_DATA_SIZE: usize = 8;
const COMMENT_SIZE: usize = 16;
const FLAG_TS: u32 = 1 << 0;
const FLAG_SOURCE_TYPE_SHIFT: u32 = 1;

const _: () = {
    assert!(size_of::<Span>() == SPAN_SIZE);
    assert!(size_of::<NodeKind>() == NODE_KIND_SIZE);
    assert!(size_of::<NodePayload>() == NODE_PAYLOAD_SIZE);
    assert!(size_of::<ExtraData>() == EXTRA_DATA_SIZE);
    assert!(size_of::<CommentKind>() == 2);
    assert!(size_of::<CommentPosition>() == 2);
    assert!(size_of::<Comment>() == COMMENT_SIZE);
    assert!(offset_of!(Comment, span) == 0);
    assert!(offset_of!(Comment, attached_to) == 8);
    assert!(offset_of!(Comment, kind) == 12);
    assert!(offset_of!(Comment, position) == 14);
};

/// Return the exact number of bytes required to serialize `tree`.
pub fn buffer_size(tree: &Ast<'_>) -> usize {
    HEADER_SIZE
        + tree.node_count() * SPAN_SIZE
        + align_to_four(tree.node_count() * NODE_KIND_SIZE)
        + tree.node_count() * NODE_PAYLOAD_SIZE
        + tree.extra_data_count() * EXTRA_DATA_SIZE
        + align_to_four(tree.strings.extra.len())
        + tree.comments().len() * COMMENT_SIZE
}

/// Serialize an AST into `buf` and return the required number of bytes.
///
/// When `buf` is too small, including when it is empty, this returns the
/// required size without writing a partial payload.
pub fn serialize_into(tree: &Ast<'_>, buf: &mut [u8]) -> usize {
    let required = buffer_size(tree);
    if buf.len() < required {
        return required;
    }

    // ====================== Header ======================
    let node_count = u32::try_from(tree.node_count()).expect("AST node count exceeds u32");
    let extra_count =
        u32::try_from(tree.extra_data_count()).expect("AST extra-data count exceeds u32");
    let string_pool_len =
        u32::try_from(tree.strings.extra.len()).expect("AST string pool exceeds u32");
    let source_len = u32::try_from(tree.source().len()).expect("AST source length exceeds u32");
    let comment_count = u32::try_from(tree.comments().len()).expect("comment count exceeds u32");

    let mut pos = 0;
    for value in [
        node_count,
        extra_count,
        string_pool_len,
        source_len,
        comment_count,
        0, // attached_comment_count
        0, // diag_count
        tree.root_node_id().index() as u32,
        (if tree.is_ts() { FLAG_TS } else { 0 })
            | ((tree.source_type() as u32) << FLAG_SOURCE_TYPE_SHIFT),
    ] {
        write_u32(buf, &mut pos, value);
    }

    // ==================== Spans =====================
    let spans = unsafe {
        slice::from_raw_parts(
            tree.nodes.span.as_ptr().cast::<u8>(),
            tree.node_count() * SPAN_SIZE,
        )
    };
    if cfg!(target_endian = "little") {
        write_slice(buf, &mut pos, spans);
    } else {
        for span in unsafe { slice::from_raw_parts(tree.nodes.span.as_ptr(), tree.node_count()) } {
            write_u32(buf, &mut pos, span.start);
            write_u32(buf, &mut pos, span.end);
        }
    }

    // ==================== Kinds =====================
    let kinds = unsafe {
        slice::from_raw_parts(
            tree.nodes.kind.as_ptr().cast::<u8>(),
            tree.node_count() * NODE_KIND_SIZE,
        )
    };
    write_slice(buf, &mut pos, kinds);
    pad_to_four(buf, &mut pos);

    // =================== Payloads ====================
    let payloads = unsafe {
        slice::from_raw_parts(
            tree.nodes.data.as_ptr().cast::<u8>(),
            tree.node_count() * NODE_PAYLOAD_SIZE,
        )
    };
    if cfg!(target_endian = "little") {
        write_slice(buf, &mut pos, payloads);
    } else {
        for payload in unsafe { slice::from_raw_parts(tree.nodes.data.as_ptr(), tree.node_count()) }
        {
            write_bytes(buf, &mut pos, &payload.0.to_le_bytes());
        }
    }

    // =================== Extra Data ====================
    let extra_data = unsafe {
        slice::from_raw_parts(
            tree.extra_data.as_slice().as_ptr().cast::<u8>(),
            tree.extra_data_count() * EXTRA_DATA_SIZE,
        )
    };
    if cfg!(target_endian = "little") {
        write_slice(buf, &mut pos, extra_data);
    } else {
        for data in tree.extra_data.as_slice() {
            write_bytes(buf, &mut pos, &data.raw().to_le_bytes());
        }
    }

    // =================== String Pool ====================
    let string_pool = &tree.strings.extra;
    buf[pos..pos + string_pool.len()].copy_from_slice(string_pool);
    pos += string_pool.len();
    pad_to_four(buf, &mut pos);

    // =================== Comments ====================
    let comments = unsafe {
        slice::from_raw_parts(
            tree.comments().as_ptr().cast::<u8>(),
            tree.comments().len() * COMMENT_SIZE,
        )
    };
    if cfg!(target_endian = "little") {
        write_slice(buf, &mut pos, comments);
    } else {
        for comment in tree.comments() {
            write_u32(buf, &mut pos, comment.span.start);
            write_u32(buf, &mut pos, comment.span.end);
            write_u32(buf, &mut pos, comment.attached_to);
            write_u16(buf, &mut pos, comment.kind as u16);
            write_u16(buf, &mut pos, comment.position as u16);
        }
    }

    debug_assert_eq!(pos, required);
    required
}

#[inline]
const fn align_to_four(len: usize) -> usize {
    (len + 3) & !3
}

#[inline]
fn pad_to_four(buf: &mut [u8], pos: &mut usize) {
    let aligned = align_to_four(*pos);
    buf[*pos..aligned].fill(0);
    *pos = aligned;
}

#[inline]
fn write_u32(buf: &mut [u8], pos: &mut usize, value: u32) {
    write_bytes(buf, pos, &value.to_le_bytes());
}

#[inline]
fn write_u16(buf: &mut [u8], pos: &mut usize, value: u16) {
    write_bytes(buf, pos, &value.to_le_bytes());
}

#[inline]
fn write_bytes<const N: usize>(buf: &mut [u8], pos: &mut usize, bytes: &[u8; N]) {
    write_slice(buf, pos, bytes);
}

#[inline]
fn write_slice(buf: &mut [u8], pos: &mut usize, bytes: &[u8]) {
    buf[*pos..*pos + bytes.len()].copy_from_slice(bytes);
    *pos += bytes.len();
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;

    use super::{buffer_size, serialize_into};
    use crate::{Ast, Span, TypedSubRange};

    #[test]
    fn canonical_little_endian_format() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let statement = ast.stmt_empty_statement(Span::new(0, 0));
        let body = ast.add_typed_sub_range([statement]);
        let program = ast.program(Span::new(0, 0), TypedSubRange::empty(), body, None);
        ast.set_root_program(program);

        let mut buffer = vec![0; buffer_size(&ast)];
        assert_eq!(serialize_into(&ast, &mut buffer), buffer.len());
        assert_eq!(buffer.len(), 104);

        assert_eq!(u32_at(&buffer, 0), 2); // node count
        assert_eq!(u32_at(&buffer, 4), 2); // extra-data count
        assert_eq!(u32_at(&buffer, 28), 1); // root node ID

        assert_eq!(u32_at(&buffer, 72), u32::MAX); // no hashbang
        assert_eq!(u32_at(&buffer, 76), 0); // inline directives start
        assert_eq!(u32_at(&buffer, 80), 0); // inline directives end
        assert_eq!(u32_at(&buffer, 84), 1); // out-of-line body field

        assert_eq!(u32_at(&buffer, 88), 0); // body element node ID
        assert_eq!(u32_at(&buffer, 92), 0); // node ID padding
        assert_eq!(u32_at(&buffer, 96), 0); // body range start
        assert_eq!(u32_at(&buffer, 100), 1); // body range end
    }

    fn u32_at(buffer: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap())
    }
}
