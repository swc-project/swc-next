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
//! - `diag_count` packed variable-length diagnostic records: severity (`u8`;
//!   `0` error, `1` warning, `2` hint, `3` info), start, end, message, optional
//!   help, label count, then labels (start, end, message). Other integers are
//!   unaligned little-endian `u32`; strings are a byte length followed by
//!   unpadded UTF-8 bytes. Optional help uses a one-byte presence flag.
//!
//! Attached comments are intentionally omitted for now. Their count in the
//! header is always zero.
//!
//! All multi-byte integers are little-endian, independent of the host target.

use std::{
    fmt,
    mem::{offset_of, size_of},
    ptr, slice,
};

use swc_next_allocator::Allocator;

use crate::{
    Ast, Comment, CommentKind, CommentPosition, Diagnostic, ExtraData, Lang, NodeId, NodeKind,
    NodePayload, SourceType, Span,
};

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

/// Return the exact number of bytes required to serialize `tree` and `diagnostics`.
pub fn buffer_size(tree: &Ast<'_>, diagnostics: &[Diagnostic<'_>]) -> usize {
    HEADER_SIZE
        + tree.node_count() * SPAN_SIZE
        + align_to_four(tree.node_count() * NODE_KIND_SIZE)
        + tree.node_count() * NODE_PAYLOAD_SIZE
        + tree.extra_data_count() * EXTRA_DATA_SIZE
        + align_to_four(tree.strings.extra.len())
        + tree.comments().len() * COMMENT_SIZE
        + diagnostics
            .iter()
            .map(|diagnostic| {
                let mut size = 0;

                size += size_of::<u8>(); // severity
                size += SPAN_SIZE; // start, end
                size += size_of::<u32>() + diagnostic.message.len(); // message_len, message
                size += size_of::<u8>(); // has_help
                if let Some(help) = &diagnostic.help {
                    size += size_of::<u32>() + help.len(); // help_len, help
                }
                size += size_of::<u32>(); // label_count
                for label in &diagnostic.labels {
                    size += SPAN_SIZE; // start, end
                    size += size_of::<u32>() + label.message.len(); // message_len, message
                }

                size
            })
            .sum::<usize>()
}

/// Serialize an AST and its diagnostics into `buf` and return the required number of bytes.
///
/// When `buf` is too small, including when it is empty, this returns the
/// required size without writing a partial payload.
pub fn serialize_into(tree: &Ast<'_>, diagnostics: &[Diagnostic<'_>], buf: &mut [u8]) -> usize {
    let required = buffer_size(tree, diagnostics);
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
    let diag_count = u32::try_from(diagnostics.len()).expect("diagnostic count exceeds u32");

    let mut pos = 0;
    for value in [
        node_count,
        extra_count,
        string_pool_len,
        source_len,
        comment_count,
        0, // attached_comment_count
        diag_count,
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

    // =================== Diagnostics ====================
    for diagnostic in diagnostics {
        write_u8(buf, &mut pos, diagnostic.severity as u8);
        write_u32(buf, &mut pos, diagnostic.span.start);
        write_u32(buf, &mut pos, diagnostic.span.end);
        write_bytes_with_length(buf, &mut pos, diagnostic.message.as_bytes());
        write_u8(buf, &mut pos, u8::from(diagnostic.help.is_some()));
        if let Some(help) = &diagnostic.help {
            write_bytes_with_length(buf, &mut pos, help.as_bytes());
        }
        let label_count =
            u32::try_from(diagnostic.labels.len()).expect("diagnostic label count exceeds u32");
        write_u32(buf, &mut pos, label_count);
        for label in &diagnostic.labels {
            write_u32(buf, &mut pos, label.span.start);
            write_u32(buf, &mut pos, label.span.end);
            write_bytes_with_length(buf, &mut pos, label.message.as_bytes());
        }
    }

    debug_assert_eq!(pos, required);
    required
}

/// Error returned when a JavaScript-encoded AST buffer is malformed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeserializeError(&'static str);

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for DeserializeError {}

/// Decode a source-less AST produced by the generated JavaScript encoder.
pub fn deserialize_from<'a>(
    buf: &[u8],
    allocator: &'a Allocator,
) -> Result<Ast<'a>, DeserializeError> {
    // ====================== Header ======================
    if buf.len() < HEADER_SIZE {
        return Err(DeserializeError("AST buffer is shorter than its header"));
    }

    let node_count = read_u32(buf, 0) as usize;
    let extra_count = read_u32(buf, 4) as usize;
    let string_pool_len = read_u32(buf, 8) as usize;
    let source_len = read_u32(buf, 12);
    let comment_count = read_u32(buf, 16) as usize;
    let attached_comment_count = read_u32(buf, 20);
    let diagnostic_count = read_u32(buf, 24);
    let program_index = read_u32(buf, 28) as usize;
    let flags = read_u32(buf, 32);

    if source_len != 0 {
        return Err(DeserializeError(
            "JavaScript-encoded ASTs cannot reference source text",
        ));
    }
    if comment_count != 0 || attached_comment_count != 0 || diagnostic_count != 0 {
        return Err(DeserializeError("unsupported AST buffer sections"));
    }

    // ================== Section Offsets ==================
    let spans = HEADER_SIZE;
    let kinds = checked_section_end(spans, node_count, SPAN_SIZE)?;
    let payloads = align_to_four(checked_section_end(kinds, node_count, NODE_KIND_SIZE)?);
    let extra = checked_section_end(payloads, node_count, NODE_PAYLOAD_SIZE)?;
    let strings = checked_section_end(extra, extra_count, EXTRA_DATA_SIZE)?;
    let comments = checked_add(strings, align_to_four(string_pool_len))?;
    let end = checked_section_end(comments, comment_count, COMMENT_SIZE)?;
    if end != buf.len() {
        return Err(DeserializeError(
            "AST buffer length does not match its header",
        ));
    }
    if node_count == 0 || program_index >= node_count {
        return Err(DeserializeError("AST buffer has an invalid program index"));
    }

    // ============== Spans / Kinds / Payloads ==============
    let kind_bytes = &buf[kinds..kinds + node_count];
    if kind_bytes.iter().any(|&kind| kind >= NodeKind::COUNT) {
        return Err(DeserializeError("AST buffer contains an unknown node kind"));
    }
    if kind_bytes[program_index] != NodeKind::Program as u8 {
        return Err(DeserializeError("AST root is not a Program"));
    }

    // TODO: Use `sourceMap.source` when optional source-map generation is enabled.
    let mut tree = Ast::new("", allocator);
    tree.reserve(node_count, extra_count);

    // SAFETY: `reserve` allocated each destination for `node_count` elements.
    // The section bounds were checked above and all node-kind bytes are valid
    // discriminants. Little-endian targets can copy the canonical wire bytes;
    // big-endian targets reconstruct the numeric payload representation.
    unsafe {
        if cfg!(target_endian = "little") {
            ptr::copy_nonoverlapping(
                buf.as_ptr().add(spans),
                tree.nodes.span.as_ptr().cast::<u8>(),
                node_count * SPAN_SIZE,
            );
            ptr::copy_nonoverlapping(
                buf.as_ptr().add(payloads),
                tree.nodes.data.as_ptr().cast::<u8>(),
                node_count * NODE_PAYLOAD_SIZE,
            );
        } else {
            for index in 0..node_count {
                let span = spans + index * SPAN_SIZE;
                tree.nodes.span.as_ptr().add(index).write(Span::new(
                    read_u32(buf, span),
                    read_u32(buf, span + size_of::<u32>()),
                ));
                tree.nodes
                    .data
                    .as_ptr()
                    .add(index)
                    .write(NodePayload(read_u128(
                        buf,
                        payloads + index * NODE_PAYLOAD_SIZE,
                    )));
            }
        }
        ptr::copy_nonoverlapping(
            kind_bytes.as_ptr(),
            tree.nodes.kind.as_ptr().cast::<u8>(),
            node_count * NODE_KIND_SIZE,
        );
        tree.nodes.len = node_count as u32;
    }
    tree.root = Some(NodeId::from_usize(program_index));

    // =================== Extra Data ====================
    // SAFETY: `reserve` allocated space for `extra_count` entries, the source
    // section has the checked byte length, and all bit patterns are valid for
    // `ExtraData`.
    unsafe {
        if cfg!(target_endian = "little") {
            ptr::copy_nonoverlapping(
                buf.as_ptr().add(extra),
                tree.extra_data.as_mut_ptr().cast::<u8>(),
                extra_count * EXTRA_DATA_SIZE,
            );
        } else {
            for index in 0..extra_count {
                tree.extra_data
                    .as_mut_ptr()
                    .add(index)
                    .write(ExtraData::from_raw(read_u64(
                        buf,
                        extra + index * EXTRA_DATA_SIZE,
                    )));
            }
        }
        tree.extra_data.set_len(extra_count);
    }

    // =================== String Pool ====================
    tree.strings
        .extra
        .extend_from_slice_copy(&buf[strings..strings + string_pool_len]);

    // ================= Source Metadata =================
    let source_type = match (flags >> FLAG_SOURCE_TYPE_SHIFT) & 3 {
        0 => SourceType::Module,
        1 => SourceType::Script,
        2 => SourceType::CommonJs,
        _ => SourceType::Unambiguous,
    };
    tree.set_source_metadata(
        source_type,
        if flags & FLAG_TS == 0 {
            Lang::Js
        } else {
            Lang::Ts
        },
    );

    Ok(tree)
}

#[inline]
fn checked_section_end(start: usize, count: usize, size: usize) -> Result<usize, DeserializeError> {
    checked_add(
        start,
        count
            .checked_mul(size)
            .ok_or(DeserializeError("AST buffer section length overflow"))?,
    )
}

#[inline]
fn checked_add(left: usize, right: usize) -> Result<usize, DeserializeError> {
    left.checked_add(right)
        .ok_or(DeserializeError("AST buffer section offset overflow"))
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
fn write_u8(buf: &mut [u8], pos: &mut usize, value: u8) {
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

#[inline]
fn read_u32(buf: &[u8], pos: usize) -> u32 {
    u32::from_le_bytes(buf[pos..pos + size_of::<u32>()].try_into().unwrap())
}

#[inline]
fn read_u64(buf: &[u8], pos: usize) -> u64 {
    u64::from_le_bytes(buf[pos..pos + size_of::<u64>()].try_into().unwrap())
}

#[inline]
fn read_u128(buf: &[u8], pos: usize) -> u128 {
    u128::from_le_bytes(buf[pos..pos + size_of::<u128>()].try_into().unwrap())
}

#[inline]
fn write_bytes_with_length(buf: &mut [u8], pos: &mut usize, bytes: &[u8]) {
    write_u32(
        buf,
        pos,
        u32::try_from(bytes.len()).expect("byte sequence length exceeds u32"),
    );
    write_slice(buf, pos, bytes);
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use swc_next_allocator::Allocator;

    use super::{buffer_size, deserialize_from, serialize_into};
    use crate::{Ast, Diagnostic, GetSpan, Label, Severity, Span, SubRange};

    #[test]
    fn canonical_little_endian_format() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let statement = ast.stmt_empty_statement(Span::new(0, 0));
        let body = ast.add_sub_range([statement]);
        let program = ast.program(Span::new(0, 0), SubRange::empty(), body, None);
        ast.set_root_program(program);

        let mut buffer = vec![0; buffer_size(&ast, &[])];
        assert_eq!(serialize_into(&ast, &[], &mut buffer), buffer.len());
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

    #[test]
    fn deserialize_canonical_little_endian_format() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let statement = ast.stmt_empty_statement(Span::new(2, 3));
        let body = ast.add_sub_range([statement]);
        let program = ast.program(Span::new(1, 4), SubRange::empty(), body, None);
        ast.set_root_program(program);

        let mut buffer = vec![0; buffer_size(&ast, &[])];
        serialize_into(&ast, &[], &mut buffer);

        let decoded_allocator = Allocator::new();
        let decoded = deserialize_from(&buffer, &decoded_allocator).unwrap();
        let decoded_program = decoded.root_program();
        let decoded_body = decoded_program.body(&decoded);
        let decoded_statement = decoded_body.get_node(&decoded, 0).unwrap();

        assert_eq!(decoded.node_count(), 2);
        assert_eq!(decoded.extra_data_count(), 2);
        assert_eq!(decoded_program.span(&decoded), Span::new(1, 4));
        assert_eq!(decoded_statement.span(&decoded), Span::new(2, 3));
        assert!(decoded_statement.is_empty_statement(&decoded));
    }

    #[test]
    fn packed_diagnostic_format() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("éx", &allocator);
        let program = ast.program(Span::new(0, 3), SubRange::empty(), SubRange::empty(), None);
        ast.set_root_program(program);

        let diagnostics = [Diagnostic {
            severity: Severity::Warning,
            message: Cow::Borrowed("é"),
            span: Span::new(0, 2),
            help: Some(Cow::Borrowed("h")),
            labels: vec![Label {
                span: Span::new(2, 3),
                message: Cow::Borrowed("abc"),
            }],
        }];
        let diagnostics_offset = buffer_size(&ast, &[]);
        let required = buffer_size(&ast, &diagnostics);
        assert_eq!(required, diagnostics_offset + 40);

        let mut too_small = vec![0xaa; required - 1];
        assert_eq!(serialize_into(&ast, &diagnostics, &mut too_small), required);
        assert!(too_small.iter().all(|&byte| byte == 0xaa));

        let mut buffer = vec![0; required];
        assert_eq!(serialize_into(&ast, &diagnostics, &mut buffer), required);
        assert_eq!(u32_at(&buffer, 24), 1); // diagnostic count

        let diagnostic = diagnostics_offset;
        assert_eq!(buffer[diagnostic], Severity::Warning as u8);
        assert_eq!(u32_at(&buffer, diagnostic + 1), 0); // start
        assert_eq!(u32_at(&buffer, diagnostic + 5), 2); // end
        assert_eq!(u32_at(&buffer, diagnostic + 9), 2); // message length
        assert_eq!(&buffer[diagnostic + 13..diagnostic + 15], "é".as_bytes());
        assert_eq!(buffer[diagnostic + 15], 1); // has help
        assert_eq!(u32_at(&buffer, diagnostic + 16), 1); // help length
        assert_eq!(buffer[diagnostic + 20], b'h');
        assert_eq!(u32_at(&buffer, diagnostic + 21), 1); // label count
        assert_eq!(u32_at(&buffer, diagnostic + 25), 2); // label start
        assert_eq!(u32_at(&buffer, diagnostic + 29), 3); // label end
        assert_eq!(u32_at(&buffer, diagnostic + 33), 3); // label message length
        assert_eq!(&buffer[diagnostic + 37..diagnostic + 40], b"abc");
    }

    fn u32_at(buffer: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap())
    }
}
