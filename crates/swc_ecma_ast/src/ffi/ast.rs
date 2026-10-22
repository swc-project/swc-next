//! Binary AST transfer format used by the JavaScript FFI.
//!
//! The AST already stores nodes as three structure-of-arrays sections, so the
//! wire format preserves that representation instead of packing every logical
//! field again. JavaScript can reconstruct fields from `kind`, `payload`, and
//! `extra_data` using the same generated AST schema.
//!
//! Layout, in order:
//! - 40-byte header, including language, source type, and the first non-ASCII
//!   source byte position;
//! - `node_count` UTF-16 spans (`start: u32`, `end: u32`);
//! - `node_count` one-byte node kinds, padded to eight bytes;
//! - `node_count` 16-byte `NodePayload` values;
//! - `extra_count` 8-byte `ExtraData` values;
//! - the raw extra string pool, padded to four bytes;
//! - `comment_count` 16-byte records in source order. Each record contains
//!   UTF-16 start/end and UTF-8 byte start/end for extracting comment text.
//! - `diag_count` packed variable-length diagnostic records: severity (`u8`;
//!   `0` error, `1` warning, `2` hint, `3` info), UTF-16 start/end, message,
//!   optional help, label count, then labels (UTF-16 start/end, message).
//!   Other integers are unaligned little-endian `u32`; strings are a byte
//!   length followed by unpadded UTF-8 bytes. Optional help uses a one-byte
//!   presence flag.
//!
//! Attached comments are intentionally omitted for now. Their count in the
//! header is always zero.
//!
//! JavaScript encoders and native consumers must use the same SWC Next version.
//!
//! All multi-byte values use little-endian byte order, including on big-endian
//! Rust and JavaScript targets.

use std::{
    fmt,
    mem::{align_of, offset_of, size_of},
    ops::Deref,
    slice,
};

use swc_next_allocator::Allocator;
#[cfg(target_endian = "little")]
use {std::ptr::NonNull, swc_next_allocator::vec::Vec as ArenaVec};

use crate::{
    Ast, Comment, CommentKind, CommentPosition, Diagnostic, ExtraData, Lang, NodeId, NodeKind,
    NodePayload, SourceType, Span,
};

const HEADER_SIZE: usize = 40;
const SPAN_SIZE: usize = 8;
const NODE_KIND_SIZE: usize = 1;
const NODE_PAYLOAD_SIZE: usize = 16;
const EXTRA_DATA_SIZE: usize = 8;
const COMMENT_SIZE: usize = 16;
// SourceType uses the low two bits; Lang uses the next three.
const FLAG_LANG_SHIFT: u32 = 2;

// Skip ASCII runs in 64-byte chunks instead of decoding each byte as a character.
const ASCII_SCAN_CHUNK_SIZE: usize = 64;
// Group source bytes into 256-byte blocks for the sparse position map.
const POSITION_BLOCK_SIZE: usize = 256;
// Use a dense map when there is more than one non-ASCII character per 128 bytes.
const DENSE_POSITION_THRESHOLD: usize = 128;
// Use a dense map when non-ASCII characters touch at least one quarter of the blocks.
const DENSE_BLOCK_COVERAGE_THRESHOLD: usize = 4;

const _: () = {
    assert!(size_of::<Span>() == SPAN_SIZE);
    assert!(size_of::<NodeKind>() == NODE_KIND_SIZE);
    assert!(size_of::<NodePayload>() == NODE_PAYLOAD_SIZE);
    assert!(size_of::<ExtraData>() == EXTRA_DATA_SIZE);
    assert!(align_of::<NodePayload>() <= 8);
    assert!(align_of::<ExtraData>() <= 8);
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
        + align_to_eight(tree.node_count() * NODE_KIND_SIZE)
        + tree.node_count() * NODE_PAYLOAD_SIZE
        + tree.extra_data_count() * EXTRA_DATA_SIZE
        + align_to_four(tree.strings.extra_bytes().len())
        + tree.comments().len() * COMMENT_SIZE
        + diagnostics_buffer_size(diagnostics)
}

/// Return the exact byte length of the packed diagnostic records, without a header.
pub fn diagnostics_buffer_size(diagnostics: &[Diagnostic<'_>]) -> usize {
    diagnostics
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
        .sum()
}

/// Serialize packed diagnostics with UTF-16 spans, without serializing an AST.
///
/// Returns the required byte length without writing when `buf` is too small.
/// `source_utf16_len` must be the UTF-16 code unit length of `source`.
pub fn serialize_diagnostics_into(
    diagnostics: &[Diagnostic<'_>],
    source: &str,
    buf: &mut [u8],
    source_utf16_len: usize,
) -> usize {
    let required = diagnostics_buffer_size(diagnostics);
    if buf.len() < required || diagnostics.is_empty() {
        return required;
    }
    let (positions, _) = build_pos_map(source, source_utf16_len);
    let mut pos = 0;
    write_diagnostics(diagnostics, positions.as_ref(), buf, &mut pos);
    debug_assert_eq!(pos, required);
    required
}

/// Serialize an AST and its diagnostics into `buf` and return the required number of bytes.
///
/// When `buf` is too small, including when it is empty, this returns the
/// required size without writing a partial payload.
/// `source_utf16_len` must be the UTF-16 code unit length of `tree.source()`.
pub fn serialize_into(
    tree: &Ast<'_>,
    diagnostics: &[Diagnostic<'_>],
    buf: &mut [u8],
    source_utf16_len: usize,
) -> usize {
    let required = buffer_size(tree, diagnostics);
    if buf.len() < required {
        return required;
    }
    let (positions, first_non_ascii_pos) = build_pos_map(tree.source(), source_utf16_len);

    // ====================== Header ======================
    let node_count = u32::try_from(tree.node_count()).expect("AST node count exceeds u32");
    let extra_count =
        u32::try_from(tree.extra_data_count()).expect("AST extra-data count exceeds u32");
    let string_pool_len =
        u32::try_from(tree.strings.extra_bytes().len()).expect("AST string pool exceeds u32");
    let source_len = u32::try_from(tree.source().len()).expect("AST source length exceeds u32");
    let comment_count = u32::try_from(tree.comments().len()).expect("comment count exceeds u32");
    let diag_count = u32::try_from(diagnostics.len()).expect("diagnostic count exceeds u32");

    let mut pos = 0;
    write(buf, &mut pos, &node_count.to_le_bytes());
    write(buf, &mut pos, &extra_count.to_le_bytes());
    write(buf, &mut pos, &string_pool_len.to_le_bytes());
    write(buf, &mut pos, &source_len.to_le_bytes());
    write(buf, &mut pos, &comment_count.to_le_bytes());
    write(buf, &mut pos, &0u32.to_le_bytes()); // attached_comment_count
    write(buf, &mut pos, &diag_count.to_le_bytes());
    write(
        buf,
        &mut pos,
        &(tree.root_node_id().index() as u32).to_le_bytes(),
    );
    let flags = (tree.source_type() as u32) | ((tree.lang() as u32) << FLAG_LANG_SHIFT);
    write(buf, &mut pos, &flags.to_le_bytes());
    let first_non_ascii_pos =
        u32::try_from(first_non_ascii_pos).expect("AST source length exceeds u32");
    write(buf, &mut pos, &first_non_ascii_pos.to_le_bytes());

    // ==================== Spans =====================
    let spans = unsafe {
        slice::from_raw_parts(
            tree.nodes.span.as_ptr().cast::<u8>(),
            tree.node_count() * SPAN_SIZE,
        )
    };
    if positions.is_none() && cfg!(target_endian = "little") {
        write(buf, &mut pos, spans);
    } else {
        for span in unsafe { slice::from_raw_parts(tree.nodes.span.as_ptr(), tree.node_count()) } {
            let start = utf16_pos(span.start, positions.as_ref());
            write(buf, &mut pos, &start.to_le_bytes());
            let end = utf16_pos(span.end, positions.as_ref());
            write(buf, &mut pos, &end.to_le_bytes());
        }
    }

    // ==================== Kinds =====================
    let kinds = unsafe {
        slice::from_raw_parts(
            tree.nodes.kind.as_ptr().cast::<u8>(),
            tree.node_count() * NODE_KIND_SIZE,
        )
    };
    write(buf, &mut pos, kinds);
    let payloads_start = align_to_eight(pos);
    buf[pos..payloads_start].fill(0);
    pos = payloads_start;

    // =================== Payloads ====================
    let payloads = unsafe {
        slice::from_raw_parts(
            tree.nodes.data.as_ptr().cast::<u8>(),
            tree.node_count() * NODE_PAYLOAD_SIZE,
        )
    };
    if cfg!(target_endian = "little") {
        write(buf, &mut pos, payloads);
    } else {
        for index in 0..tree.node_count() {
            let start = index * NODE_PAYLOAD_SIZE;
            let mut payload: [u8; NODE_PAYLOAD_SIZE] = payloads[start..start + NODE_PAYLOAD_SIZE]
                .try_into()
                .unwrap();
            // SAFETY: The AST node table contains `node_count` initialized kinds.
            let kind = unsafe { *tree.nodes.kind.as_ptr().add(index) };
            crate::generated::ast_payload::swap_payload_endian(&mut payload, kind);
            write(buf, &mut pos, &payload);
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
        write(buf, &mut pos, extra_data);
    } else {
        for data in tree.extra_data.as_slice() {
            write(buf, &mut pos, &data.raw().to_le_bytes());
        }
    }

    // =================== String Pool ====================
    let string_pool = tree.strings.extra_bytes();
    buf[pos..pos + string_pool.len()].copy_from_slice(string_pool);
    pos += string_pool.len();
    pad_to_four(buf, &mut pos);

    // =================== Comments ====================
    for comment in tree.comments() {
        let start = utf16_pos(comment.span.start, positions.as_ref());
        let end = utf16_pos(comment.span.end, positions.as_ref());
        write(buf, &mut pos, &start.to_le_bytes());
        write(buf, &mut pos, &end.to_le_bytes());
        write(buf, &mut pos, &comment.span.start.to_le_bytes());
        write(buf, &mut pos, &comment.span.end.to_le_bytes());
    }

    // =================== Diagnostics ====================
    write_diagnostics(diagnostics, positions.as_ref(), buf, &mut pos);

    debug_assert_eq!(pos, required);
    required
}

fn write_diagnostics(
    diagnostics: &[Diagnostic<'_>],
    positions: Option<&PositionMap>,
    buf: &mut [u8],
    pos: &mut usize,
) {
    for diagnostic in diagnostics {
        write(buf, pos, &[diagnostic.severity as u8]);
        let start = utf16_pos(diagnostic.span.start, positions);
        let end = utf16_pos(diagnostic.span.end, positions);
        write(buf, pos, &start.to_le_bytes());
        write(buf, pos, &end.to_le_bytes());
        write_bytes_with_length(buf, pos, diagnostic.message.as_bytes());
        write(buf, pos, &[u8::from(diagnostic.help.is_some())]);
        if let Some(help) = &diagnostic.help {
            write_bytes_with_length(buf, pos, help.as_bytes());
        }
        let label_count =
            u32::try_from(diagnostic.labels.len()).expect("diagnostic label count exceeds u32");
        write(buf, pos, &label_count.to_le_bytes());
        for label in &diagnostic.labels {
            let start = utf16_pos(label.span.start, positions);
            let end = utf16_pos(label.span.end, positions);
            write(buf, pos, &start.to_le_bytes());
            write(buf, pos, &end.to_le_bytes());
            write_bytes_with_length(buf, pos, label.message.as_bytes());
        }
    }
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

/// A read-only AST view backed by a transfer buffer.
///
/// Mutation is unavailable because its tables may alias immutable buffer bytes.
///
/// ```compile_fail
/// use swc_next_ecma_ast::ffi::ast::BorrowedAst;
///
/// fn reserve(view: &mut BorrowedAst<'_>) {
///     view.reserve(1, 1);
/// }
/// ```
pub struct BorrowedAst<'a> {
    ast: Ast<'a>,
}

impl<'a> Deref for BorrowedAst<'a> {
    type Target = Ast<'a>;

    fn deref(&self) -> &Self::Target {
        &self.ast
    }
}

impl<'a> BorrowedAst<'a> {
    /// Copy this transfer view into mutable, arena-owned storage.
    ///
    /// The transfer's string source and extra pool are concatenated into the
    /// owned AST's extra pool, preserving every string offset. The resulting
    /// [`Ast::source`] is empty: it never contained the original source text.
    /// Node spans remain UTF-16 offsets and can be serialized with a source
    /// length of zero. Mutation cannot alter the original transfer buffer.
    pub fn into_owned(self) -> Ast<'a> {
        let mut ast = Ast::new("", self.allocator());
        let node_count = self.node_count();
        ast.reserve(node_count, self.extra_data_count());
        // SAFETY: Both node tables have capacity for node_count entries, and
        // the source contains that many initialized entries in native order.
        // Their allocations are disjoint; every copied field is Copy.
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.nodes.span.as_ptr(),
                ast.nodes.span.as_ptr(),
                node_count,
            );
            std::ptr::copy_nonoverlapping(
                self.nodes.kind.as_ptr(),
                ast.nodes.kind.as_ptr(),
                node_count,
            );
            std::ptr::copy_nonoverlapping(
                self.nodes.data.as_ptr(),
                ast.nodes.data.as_ptr(),
                node_count,
            );
            ast.nodes.len = node_count as u32;
        }
        ast.extra_data.extend_from_slice_copy(&self.extra_data);
        ast.strings
            .initialize_extra(self.source().as_bytes(), self.strings.extra_bytes());
        ast.root = self.root;
        ast.set_source_metadata(self.source_type(), self.lang());
        ast
    }
}

/// Decode an AST produced by the generated JavaScript encoder.
///
/// The source section holds concatenated UTF-8 field values, not original source
/// text. Transfers containing lone surrogates use the extra WTF-8 pool instead.
/// Node spans remain independent UTF-16 positions in the original source.
///
/// Little-endian targets borrow every table and string section. Other targets
/// copy tables into `allocator` to convert their byte order.
///
/// # Safety
///
/// `buf` must be produced by `@swc-next/encoder` matching the version of this
/// crate and follow the [AST transfer protocol](crate::ffi::ast). It must contain
/// valid node kinds, typed payloads, indices, ranges, string encodings, and source
/// metadata. Comment and diagnostic sections must be empty. On little-endian
/// targets, its starting address must be aligned to eight bytes. The buffer must
/// remain immutable for the returned view's lifetime.
pub unsafe fn deserialize_from<'a>(
    buf: &'a [u8],
    allocator: &'a Allocator,
) -> Result<BorrowedAst<'a>, DeserializeError> {
    #[cfg(target_endian = "big")]
    {
        use std::ptr;

        #[inline]
        fn checked_section_end(
            start: usize,
            count: usize,
            size: usize,
        ) -> Result<usize, DeserializeError> {
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

        // ====================== Header ======================
        if buf.len() < HEADER_SIZE {
            return Err(DeserializeError("AST buffer is shorter than its header"));
        }

        let node_count = read_u32(buf, 0) as usize;
        let extra_count = read_u32(buf, 4) as usize;
        let string_pool_len = read_u32(buf, 8) as usize;
        let source_len = read_u32(buf, 12) as usize;
        let comment_count = read_u32(buf, 16) as usize;
        let attached_comment_count = read_u32(buf, 20);
        let diagnostic_count = read_u32(buf, 24);
        let program_index = read_u32(buf, 28) as usize;
        let flags = read_u32(buf, 32);

        if comment_count != 0 || attached_comment_count != 0 || diagnostic_count != 0 {
            return Err(DeserializeError("unsupported AST buffer sections"));
        }

        // ================== Section Offsets ==================
        let spans = HEADER_SIZE;
        let kinds = checked_section_end(spans, node_count, SPAN_SIZE)?;
        let payloads = align_to_eight(checked_section_end(kinds, node_count, NODE_KIND_SIZE)?);
        let extra = checked_section_end(payloads, node_count, NODE_PAYLOAD_SIZE)?;
        let strings = checked_section_end(extra, extra_count, EXTRA_DATA_SIZE)?;
        let source_end = checked_add(strings, source_len)?;
        let pool_end = checked_add(source_end, string_pool_len)?;
        let comments = align_to_four(pool_end);
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
        let source = std::str::from_utf8(&buf[strings..source_end])
            .map_err(|_| DeserializeError("AST string source contains invalid UTF-8"))?;
        let mut tree = Ast::new(source, allocator);
        tree.reserve(node_count, extra_count);

        // SAFETY: `reserve` allocated each destination for `node_count` elements.
        // The section bounds were checked above and all node-kind bytes are valid
        // discriminants. Numeric fields are converted from the canonical
        // little-endian wire bytes to native byte order.
        unsafe {
            for index in 0..node_count {
                let span = spans + index * SPAN_SIZE;
                tree.nodes.span.as_ptr().add(index).write(Span::new(
                    read_u32(buf, span),
                    read_u32(buf, span + size_of::<u32>()),
                ));
            }
            ptr::copy_nonoverlapping(
                buf.as_ptr().add(payloads),
                tree.nodes.data.as_ptr().cast::<u8>(),
                node_count * NODE_PAYLOAD_SIZE,
            );
            for (index, &kind_byte) in kind_bytes.iter().enumerate() {
                let payload = &mut *tree.nodes.data.as_ptr().add(index);
                let bytes = &mut *ptr::from_mut(payload).cast::<[u8; NODE_PAYLOAD_SIZE]>();
                let kind = std::mem::transmute::<u8, NodeKind>(kind_byte);
                crate::generated::ast_payload::swap_payload_endian(bytes, kind);
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
            ptr::copy_nonoverlapping(
                buf.as_ptr().add(extra),
                tree.extra_data.as_mut_ptr().cast::<u8>(),
                extra_count * EXTRA_DATA_SIZE,
            );
            let bytes = slice::from_raw_parts_mut(
                tree.extra_data.as_mut_ptr().cast::<u8>(),
                extra_count * EXTRA_DATA_SIZE,
            );
            for word in bytes.as_chunks_mut::<4>().0 {
                word.reverse();
            }
            tree.extra_data.set_len(extra_count);
        }

        // =================== String Pool ====================
        tree.strings
            .initialize_extra(&[], &buf[source_end..pool_end]);

        // ================= Source Metadata =================
        let source_type = match flags & 3 {
            0 => SourceType::Module,
            1 => SourceType::Script,
            2 => SourceType::CommonJs,
            _ => SourceType::Unambiguous,
        };
        let lang = match (flags >> FLAG_LANG_SHIFT) & 7 {
            0 => Lang::Js,
            1 => Lang::Ts,
            2 => Lang::Jsx,
            3 => Lang::Tsx,
            4 => Lang::Dts,
            _ => return Err(DeserializeError("AST buffer contains an unknown language")),
        };
        tree.set_source_metadata(source_type, lang);

        Ok(BorrowedAst { ast: tree })
    }

    #[cfg(target_endian = "little")]
    {
        debug_assert!(buf.as_ptr().addr().is_multiple_of(8));
        let node_count = read_u32(buf, 0) as usize;
        let extra_count = read_u32(buf, 4) as usize;
        let pool_len = read_u32(buf, 8) as usize;
        let source_len = read_u32(buf, 12) as usize;
        let root = read_u32(buf, 28) as usize;
        let flags = read_u32(buf, 32);

        let spans = HEADER_SIZE;
        let kinds = spans + node_count * SPAN_SIZE;
        let payloads = align_to_eight(kinds + node_count);
        let extra = payloads + node_count * NODE_PAYLOAD_SIZE;
        let strings = extra + extra_count * EXTRA_DATA_SIZE;
        let source_end = strings + source_len;

        // SAFETY: The caller guarantees valid encodings, table layouts, and
        // alignment. BorrowedAst exposes no mutable access to these pointers.
        unsafe {
            let source = std::str::from_utf8_unchecked(&buf[strings..source_end]);
            let mut ast = Ast::new(source, allocator);
            ast.nodes.span = NonNull::new_unchecked(buf.as_ptr().add(spans).cast_mut().cast());
            ast.nodes.kind = NonNull::new_unchecked(buf.as_ptr().add(kinds).cast_mut().cast());
            ast.nodes.data = NonNull::new_unchecked(buf.as_ptr().add(payloads).cast_mut().cast());
            ast.nodes.len = node_count as u32;
            ast.nodes.cap = node_count as u32;
            ast.extra_data = ArenaVec::from_borrowed_slice_in(
                slice::from_raw_parts(buf.as_ptr().add(extra).cast(), extra_count),
                allocator,
            );
            ast.strings
                .borrow_extra(&buf[source_end..source_end + pool_len]);
            ast.root = Some(NodeId::from_usize(root));
            ast.source_type = std::mem::transmute::<u8, SourceType>((flags & 3) as u8);
            ast.lang = std::mem::transmute::<u8, Lang>(((flags >> FLAG_LANG_SHIFT) & 7) as u8);
            Ok(BorrowedAst { ast })
        }
    }
}

struct NonAsciiChar {
    start: usize,
    end: usize,
    utf16_start: u32,
    utf16_end: u32,
}

struct PositionBlock {
    delta: u32,
    positions: Option<Box<[u32]>>,
}

enum PositionMap {
    Dense(Vec<u32>),
    Sparse {
        source_len: usize,
        blocks: Vec<PositionBlock>,
    },
}

fn build_pos_map(source: &str, source_utf16_len: usize) -> (Option<PositionMap>, usize) {
    if source.len() == source_utf16_len {
        return (None, source.len());
    }

    let bytes = source.as_bytes();
    let mut non_ascii = Vec::new();
    let mut byte = 0;
    let mut utf16 = 0u32;
    let dense_threshold = source.len() / DENSE_POSITION_THRESHOLD;
    let dense_block_threshold = source
        .len()
        .div_ceil(POSITION_BLOCK_SIZE)
        .div_ceil(DENSE_BLOCK_COVERAGE_THRESHOLD);
    let mut covered_blocks = 0;
    let mut last_covered_block = None;
    while byte < bytes.len() {
        let chunk_end = (byte + ASCII_SCAN_CHUNK_SIZE).min(bytes.len());
        if bytes[byte..chunk_end].is_ascii() {
            utf16 += (chunk_end - byte) as u32;
            byte = chunk_end;
            continue;
        }

        while byte < chunk_end {
            let char_len = match bytes[byte] {
                0..=0x7f => 1,
                0xc0..=0xdf => 2,
                0xe0..=0xef => 3,
                0xf0..=0xf4 => 4,
                _ => unreachable!("source is valid UTF-8"),
            };
            let utf16_len = if char_len == 4 { 2 } else { 1 };
            if char_len != 1 {
                for block in
                    byte / POSITION_BLOCK_SIZE..=(byte + char_len - 1) / POSITION_BLOCK_SIZE
                {
                    if last_covered_block != Some(block) {
                        covered_blocks += 1;
                        last_covered_block = Some(block);
                    }
                }
                non_ascii.push(NonAsciiChar {
                    start: byte,
                    end: byte + char_len,
                    utf16_start: utf16,
                    utf16_end: utf16 + utf16_len,
                });
                // A dense source or widely distributed Unicode is cheaper to map directly.
                if non_ascii.len() > dense_threshold || covered_blocks >= dense_block_threshold {
                    let first_non_ascii_pos = non_ascii[0].start;
                    return (
                        Some(PositionMap::Dense(build_dense_pos_map(source))),
                        first_non_ascii_pos,
                    );
                }
            }
            byte += char_len;
            utf16 += utf16_len;
        }
    }

    let first_non_ascii_pos = non_ascii[0].start;
    let mut blocks = Vec::with_capacity(source.len() / POSITION_BLOCK_SIZE + 1);
    let mut event_index = 0;
    let mut delta = 0;
    for block_start in (0..=source.len()).step_by(POSITION_BLOCK_SIZE) {
        while event_index < non_ascii.len() && non_ascii[event_index].end <= block_start {
            let event = &non_ascii[event_index];
            delta = event.end as u32 - event.utf16_end;
            event_index += 1;
        }

        let block_end = (block_start + POSITION_BLOCK_SIZE).min(source.len() + 1);
        let positions = if event_index < non_ascii.len() && non_ascii[event_index].start < block_end
        {
            let mut positions = Vec::with_capacity(block_end - block_start);
            let mut local_index = event_index;
            let mut local_delta = delta;
            for byte in block_start..block_end {
                while local_index < non_ascii.len() && non_ascii[local_index].end <= byte {
                    let event = &non_ascii[local_index];
                    local_delta = event.end as u32 - event.utf16_end;
                    local_index += 1;
                }
                let position =
                    if local_index < non_ascii.len() && non_ascii[local_index].start <= byte {
                        let event = &non_ascii[local_index];
                        // The final byte of a four-byte scalar maps to its second UTF-16 unit.
                        event.utf16_start
                            + u32::from(event.end - event.start == 4 && byte + 1 == event.end)
                    } else {
                        byte as u32 - local_delta
                    };
                positions.push(position);
            }
            Some(positions.into_boxed_slice())
        } else {
            None
        };
        blocks.push(PositionBlock { delta, positions });
    }

    (
        Some(PositionMap::Sparse {
            source_len: source.len(),
            blocks,
        }),
        first_non_ascii_pos,
    )
}

fn build_dense_pos_map(source: &str) -> Vec<u32> {
    let mut map = vec![0; source.len() + 1];
    let mut utf16 = 0u32;
    for (byte, ch) in source.char_indices() {
        let char_len = ch.len_utf8();
        map[byte..byte + char_len].fill(utf16);
        if char_len == 4 {
            map[byte + 3] = utf16 + 1;
        }
        utf16 += ch.len_utf16() as u32;
    }
    map[source.len()] = utf16;
    map
}

#[inline]
fn utf16_pos(byte: u32, map: Option<&PositionMap>) -> u32 {
    let Some(map) = map else { return byte };
    let byte_index = byte as usize;
    match map {
        PositionMap::Dense(positions) => positions.get(byte_index).copied().unwrap_or(byte),
        PositionMap::Sparse { source_len, blocks } => {
            if byte_index > *source_len {
                return byte;
            }
            let block = &blocks[byte_index / POSITION_BLOCK_SIZE];
            block
                .positions
                .as_ref()
                .map_or(byte - block.delta, |positions| {
                    positions[byte_index % POSITION_BLOCK_SIZE]
                })
        }
    }
}

#[inline]
const fn align_to_eight(len: usize) -> usize {
    (len + 7) & !7
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
fn write(buf: &mut [u8], pos: &mut usize, bytes: &[u8]) {
    buf[*pos..*pos + bytes.len()].copy_from_slice(bytes);
    *pos += bytes.len();
}

#[inline]
fn read_u32(buf: &[u8], pos: usize) -> u32 {
    u32::from_le_bytes(buf[pos..pos + size_of::<u32>()].try_into().unwrap())
}

#[inline]
fn write_bytes_with_length(buf: &mut [u8], pos: &mut usize, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).expect("byte sequence length exceeds u32");
    write(buf, pos, &len.to_le_bytes());
    write(buf, pos, bytes);
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use swc_next_allocator::{Allocator, wtf8::Wtf8};

    use super::{
        PositionMap, buffer_size, build_pos_map, deserialize_from, diagnostics_buffer_size,
        serialize_diagnostics_into, serialize_into, utf16_pos,
    };
    use crate::{
        Ast, Diagnostic, Expr, GetSpan, Label, Lang, Severity, SourceType, Span, SubRange,
    };

    #[test]
    fn canonical_little_endian_format() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let statement = ast.stmt_empty_statement(Span::new(0, 0));
        let body = ast.add_sub_range([statement]);
        let program = ast.program(Span::new(0, 0), SubRange::empty(), body, None);
        ast.set_root_program(program);

        let mut buffer = vec![0; buffer_size(&ast, &[])];
        assert_eq!(serialize_into(&ast, &[], &mut buffer, 0), buffer.len());
        assert_eq!(buffer.len(), 112);

        assert_eq!(u32_at(&buffer, 0), 2); // node count
        assert_eq!(u32_at(&buffer, 4), 2); // extra-data count
        assert_eq!(u32_at(&buffer, 28), 1); // root node ID
        assert_eq!(u32_at(&buffer, 36), 0); // no non-ASCII source bytes

        assert_eq!(u32_at(&buffer, 80), 0); // inline directives start
        assert_eq!(u32_at(&buffer, 84), 0); // inline directives end
        assert_eq!(u32_at(&buffer, 88), u32::MAX); // no hashbang
        assert_eq!(u32_at(&buffer, 92), 1); // out-of-line body field

        assert_eq!(u32_at(&buffer, 96), 0); // body element node ID
        assert_eq!(u32_at(&buffer, 100), 0); // node ID padding
        assert_eq!(u32_at(&buffer, 104), 0); // body range start
        assert_eq!(u32_at(&buffer, 108), 1); // body range end
    }

    #[test]
    fn deserialize_little_endian_format() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let statement = ast.stmt_empty_statement(Span::new(2, 3));
        let body = ast.add_sub_range([statement]);
        let program = ast.program(Span::new(1, 4), SubRange::empty(), body, None);
        ast.set_root_program(program);

        let mut buffer = vec![0; buffer_size(&ast, &[])];
        serialize_into(&ast, &[], &mut buffer, 0);
        let buffer = AlignedBuffer::new(&buffer);

        let decoded_allocator = Allocator::new();
        // SAFETY: The serializer produced a valid, aligned transfer.
        let decoded = unsafe { deserialize_from(&buffer, &decoded_allocator) }.unwrap();
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
    fn owned_transfer() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let source = ast.add_utf8("é😀");
        // SAFETY: ED A0 80 encodes a lone high surrogate, valid in WTF-8.
        let surrogate = unsafe { Wtf8::from_bytes_unchecked(b"\xed\xa0\x80") };
        let value = ast.add_wtf8(surrogate);
        let literal = ast.string_literal(Span::new(3, 6), value, source);
        let statement =
            ast.stmt_expression_statement(Span::new(3, 7), Expr::StringLiteral(literal));
        let body = ast.add_sub_range([statement]);
        let program = ast.program(Span::new(0, 7), SubRange::empty(), body, None);
        ast.set_root_program(program);
        ast.set_source_metadata(SourceType::Script, Lang::Tsx);

        let mut bytes = vec![0; buffer_size(&ast, &[])];
        serialize_into(&ast, &[], &mut bytes, 0);
        // JavaScript may place the UTF-8 prefix in its source section, followed
        // by the WTF-8 extra pool; string offsets use their concatenation.
        bytes[8..12].copy_from_slice(&(surrogate.len() as u32).to_le_bytes());
        bytes[12..16].copy_from_slice(&source.len().to_le_bytes());
        let buffer = AlignedBuffer::new(&bytes);
        let decoded_allocator = Allocator::new();
        // SAFETY: The serializer and adjusted section sizes form an aligned
        // transfer with valid UTF-8 source and WTF-8 pool strings.
        let decoded = unsafe { deserialize_from(&buffer, &decoded_allocator) }.unwrap();
        let mut owned = decoded.into_owned();

        assert_eq!(owned.source(), "");
        assert_eq!(owned.get_utf8(source), "é😀");
        assert_eq!(owned.get_wtf8(value).as_bytes(), surrogate.as_bytes());
        assert_eq!(literal.span(&owned), Span::new(3, 6));
        assert_eq!(owned.lang(), Lang::Tsx);
        assert_eq!(owned.source_type(), SourceType::Script);

        let replacement = owned.add_utf8("replacement");
        literal.set_value(&mut owned, replacement.into());
        program.set_body(&mut owned, SubRange::empty());
        *owned.nodes.span_mut(literal.node_id()) = Span::new(8, 9);
        assert_eq!(&*buffer, bytes);
        assert_eq!(
            owned.get_wtf8(literal.value(&owned)).as_bytes(),
            b"replacement"
        );
        assert!(program.body(&owned).is_empty());

        let mut output = vec![0; buffer_size(&owned, &[])];
        serialize_into(&owned, &[], &mut output, 0);
        assert_eq!(u32_at(&output, 12), 0);
        assert_eq!(u32_at(&output, 40), 8);
        assert_eq!(u32_at(&output, 44), 9);
    }

    #[test]
    fn position_map_across_blocks() {
        let (ascii, first_non_ascii) = build_pos_map("abc", 3);
        assert!(ascii.is_none());
        assert_eq!(first_non_ascii, 3);

        let source = format!(
            "{}😀é{}🦀{}",
            "a".repeat(255),
            "b".repeat(255),
            "c".repeat(4096)
        );
        let (map, first_non_ascii) = build_pos_map(&source, source.encode_utf16().count());
        assert!(matches!(map.as_ref(), Some(PositionMap::Sparse { .. })));
        let map = map.as_ref();
        assert_eq!(first_non_ascii, 255);
        assert_eq!(utf16_pos(254, map), 254);
        assert_eq!(utf16_pos(255, map), 255);
        assert_eq!(utf16_pos(256, map), 255);
        assert_eq!(utf16_pos(258, map), 256);
        assert_eq!(utf16_pos(259, map), 257);
        assert_eq!(utf16_pos(260, map), 257);
        assert_eq!(utf16_pos(261, map), 258);
        assert_eq!(utf16_pos(516, map), 513);
        assert_eq!(utf16_pos(519, map), 514);
        assert_eq!(utf16_pos(520, map), 515);
        assert_eq!(utf16_pos(521, map), 516);
        assert_eq!(
            utf16_pos((source.len() + 1) as u32, map),
            (source.len() + 1) as u32
        );

        let dense_source = "é".repeat(256);
        let (dense, first_non_ascii) =
            build_pos_map(&dense_source, dense_source.encode_utf16().count());
        assert!(matches!(dense, Some(PositionMap::Dense(_))));
        assert_eq!(first_non_ascii, 0);
        assert_eq!(utf16_pos(1, dense.as_ref()), 0);
        assert_eq!(utf16_pos(255, dense.as_ref()), 127);
        assert_eq!(utf16_pos(512, dense.as_ref()), 256);

        let distributed_source = format!("{}é", "a".repeat(254)).repeat(128);
        let (distributed, first_non_ascii) = build_pos_map(
            &distributed_source,
            distributed_source.encode_utf16().count(),
        );
        assert!(matches!(distributed, Some(PositionMap::Dense(_))));
        assert_eq!(first_non_ascii, 254);
        assert_eq!(utf16_pos(255, distributed.as_ref()), 254);
        assert_eq!(utf16_pos(256, distributed.as_ref()), 255);
        assert_eq!(utf16_pos(512, distributed.as_ref()), 510);
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
        assert_eq!(
            serialize_into(&ast, &diagnostics, &mut too_small, 2),
            required
        );
        assert!(too_small.iter().all(|&byte| byte == 0xaa));

        let mut buffer = vec![0; required];
        assert_eq!(serialize_into(&ast, &diagnostics, &mut buffer, 2), required);
        assert_eq!(u32_at(&buffer, 24), 1); // diagnostic count
        assert_eq!(u32_at(&buffer, 36), 0); // first non-ASCII byte
        assert_eq!(u32_at(&buffer, 44), 2); // program end in UTF-16 units

        let diagnostic = diagnostics_offset;
        assert_eq!(buffer[diagnostic], Severity::Warning as u8);
        assert_eq!(u32_at(&buffer, diagnostic + 1), 0); // start
        assert_eq!(u32_at(&buffer, diagnostic + 5), 1); // end
        assert_eq!(u32_at(&buffer, diagnostic + 9), 2); // message length
        assert_eq!(&buffer[diagnostic + 13..diagnostic + 15], "é".as_bytes());
        assert_eq!(buffer[diagnostic + 15], 1); // has help
        assert_eq!(u32_at(&buffer, diagnostic + 16), 1); // help length
        assert_eq!(buffer[diagnostic + 20], b'h');
        assert_eq!(u32_at(&buffer, diagnostic + 21), 1); // label count
        assert_eq!(u32_at(&buffer, diagnostic + 25), 1); // label start
        assert_eq!(u32_at(&buffer, diagnostic + 29), 2); // label end
        assert_eq!(u32_at(&buffer, diagnostic + 33), 3); // label message length
        assert_eq!(&buffer[diagnostic + 37..diagnostic + 40], b"abc");

        let diagnostic_size = diagnostics_buffer_size(&diagnostics);
        let mut records = vec![0; diagnostic_size];
        assert_eq!(
            serialize_diagnostics_into(&diagnostics, ast.source(), &mut records, 2),
            diagnostic_size
        );
        assert_eq!(records, buffer[diagnostics_offset..]);

        let mut too_small = vec![0xaa; diagnostic_size - 1];
        assert_eq!(
            serialize_diagnostics_into(&diagnostics, ast.source(), &mut too_small, 2),
            diagnostic_size
        );
        assert!(too_small.iter().all(|&byte| byte == 0xaa));
    }

    struct AlignedBuffer {
        words: Vec<u64>,
        len: usize,
    }

    impl AlignedBuffer {
        fn new(bytes: &[u8]) -> Self {
            let mut buffer = Self {
                words: vec![0; bytes.len().div_ceil(8)],
                len: bytes.len(),
            };
            // SAFETY: Every bit pattern is valid for u64; the slice stays in words.
            unsafe {
                std::slice::from_raw_parts_mut(buffer.words.as_mut_ptr().cast::<u8>(), buffer.len)
                    .copy_from_slice(bytes);
            }
            buffer
        }
    }

    impl std::ops::Deref for AlignedBuffer {
        type Target = [u8];

        fn deref(&self) -> &Self::Target {
            // SAFETY: The first len bytes were initialized by new.
            unsafe { std::slice::from_raw_parts(self.words.as_ptr().cast::<u8>(), self.len) }
        }
    }

    fn u32_at(buffer: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap())
    }
}
