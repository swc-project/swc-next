//! Binary AST transfer format used by the JavaScript FFI.
//!
//! The AST already stores nodes as three structure-of-arrays sections, so the
//! wire format preserves that representation instead of packing every logical
//! field again. JavaScript can reconstruct fields from `kind`, `payload`, and
//! `extra_data` using the same generated AST schema.
//!
//! Layout, in order:
//! - 36-byte AST-only header;
//! - `node_count` raw UTF-8 byte spans (`start: u32`, `end: u32`);
//! - `node_count` one-byte node kinds, padded to four bytes;
//! - `node_count` raw 16-byte `NodePayload` values;
//! - `extra_count` raw 8-byte `ExtraData` values;
//! - the raw extra string pool, padded to four bytes.
//!
//! Comments and diagnostics are intentionally omitted for now. Their counts
//! and flags in the header are always zero.

use std::{mem::size_of, slice};

use crate::{Ast, ExtraData, NodeKind, NodePayload, Span};

const HEADER_SIZE: usize = 36;
const SPAN_SIZE: usize = 8;
const NODE_KIND_SIZE: usize = 1;
const NODE_PAYLOAD_SIZE: usize = 16;
const EXTRA_DATA_SIZE: usize = 8;
const FLAG_TS: u32 = 1 << 0;
const FLAG_SOURCE_TYPE_SHIFT: u32 = 1;

const _: () = {
    assert!(cfg!(target_endian = "little"));
    assert!(size_of::<Span>() == SPAN_SIZE);
    assert!(size_of::<NodeKind>() == NODE_KIND_SIZE);
    assert!(size_of::<NodePayload>() == NODE_PAYLOAD_SIZE);
    assert!(size_of::<ExtraData>() == EXTRA_DATA_SIZE);
};

/// Return the exact number of bytes required to serialize `tree`.
pub fn buffer_size(tree: &Ast<'_>) -> usize {
    HEADER_SIZE
        + tree.node_count() * SPAN_SIZE
        + align_to_four(tree.node_count() * NODE_KIND_SIZE)
        + tree.node_count() * NODE_PAYLOAD_SIZE
        + tree.extra_data_count() * EXTRA_DATA_SIZE
        + align_to_four(tree.strings.extra.len())
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

    let mut pos = 0;
    for value in [
        node_count,
        extra_count,
        string_pool_len,
        source_len,
        0, // comment_count
        0, // attached_comment_count
        0, // diag_count
        tree.root_program().node_id().index() as u32,
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
    buf[pos..pos + spans.len()].copy_from_slice(spans);
    pos += spans.len();

    // ==================== Kinds =====================
    let kinds = unsafe {
        slice::from_raw_parts(
            tree.nodes.kind.as_ptr().cast::<u8>(),
            tree.node_count() * NODE_KIND_SIZE,
        )
    };
    buf[pos..pos + kinds.len()].copy_from_slice(kinds);
    pos += kinds.len();
    pad_to_four(buf, &mut pos);

    // =================== Payloads ====================
    let payloads = unsafe {
        slice::from_raw_parts(
            tree.nodes.data.as_ptr().cast::<u8>(),
            tree.node_count() * NODE_PAYLOAD_SIZE,
        )
    };
    buf[pos..pos + payloads.len()].copy_from_slice(payloads);
    pos += payloads.len();

    // =================== Extra Data ====================
    let extra_data = unsafe {
        slice::from_raw_parts(
            tree.extra_data.as_slice().as_ptr().cast::<u8>(),
            tree.extra_data_count() * EXTRA_DATA_SIZE,
        )
    };
    buf[pos..pos + extra_data.len()].copy_from_slice(extra_data);
    pos += extra_data.len();

    // =================== String Pool ====================
    let string_pool = &tree.strings.extra;
    buf[pos..pos + string_pool.len()].copy_from_slice(string_pool);
    pos += string_pool.len();
    pad_to_four(buf, &mut pos);

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
    buf[*pos..*pos + size_of::<u32>()].copy_from_slice(&value.to_le_bytes());
    *pos += size_of::<u32>();
}
