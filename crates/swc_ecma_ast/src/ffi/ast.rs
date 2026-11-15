//! Binary AST transfer format used by the JavaScript FFI.
//!
//! For internal SWC Next use only. These APIs and the AST/diagnostic formats do
//! not follow semantic versioning and may change in any release. Buffers are not
//! intended for persistent storage or exchange between versions.
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
//! - `extra_count` 4-byte `ExtraDataUnit` values;
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
use {
    std::ptr::NonNull,
    swc_next_allocator::vec::{ArenaIndexVec, ArenaVec},
};

use crate::{
    Ast, AstCtx, Comment, CommentKind, CommentPosition, Diagnostic, ExtraDataUnit, Lang, NodeId,
    NodeKind, NodePayload, SourceType, Span,
};

const HEADER_SIZE: usize = 40;
const SPAN_SIZE: usize = 8;
const NODE_KIND_SIZE: usize = 1;
const NODE_PAYLOAD_SIZE: usize = 16;
const EXTRA_DATA_SIZE: usize = 4;
const COMMENT_SIZE: usize = 16;
// SourceType uses the low two bits; Lang uses the next three.
const FLAG_LANG_SHIFT: u32 = 2;

// Skip ASCII runs in 64-byte chunks instead of decoding each byte as a character.
const ASCII_SCAN_CHUNK_SIZE: usize = 64;

const _: () = {
    assert!(size_of::<Span>() == SPAN_SIZE);
    assert!(size_of::<NodeKind>() == NODE_KIND_SIZE);
    assert!(size_of::<NodePayload>() == NODE_PAYLOAD_SIZE);
    assert!(size_of::<ExtraDataUnit>() == EXTRA_DATA_SIZE);
    assert!(align_of::<NodePayload>() <= 8);
    assert!(align_of::<ExtraDataUnit>() == 4);
    assert!(size_of::<CommentKind>() == 2);
    assert!(size_of::<CommentPosition>() == 2);
    assert!(size_of::<Comment>() == COMMENT_SIZE);
    assert!(offset_of!(Comment, span) == 0);
    assert!(offset_of!(Comment, attached_to) == 8);
    assert!(offset_of!(Comment, kind) == 12);
    assert!(offset_of!(Comment, position) == 14);
};

/// Return the exact number of bytes required to serialize `tree` and `diagnostics`.
pub fn buffer_size(tree: &AstCtx<'_>, diagnostics: &[Diagnostic]) -> usize {
    // SAFETY: Sizing only reads this AST's storage lengths and comments.
    let tree = unsafe { tree.ast() };
    HEADER_SIZE
        + tree.node_count() * SPAN_SIZE
        + align_to_eight(tree.node_count() * NODE_KIND_SIZE)
        + tree.node_count() * NODE_PAYLOAD_SIZE
        + tree.extra_data_count() * EXTRA_DATA_SIZE
        + align_to_four(tree.strings.extra_bytes().len())
        + tree.comments().len() * COMMENT_SIZE
        + diagnostics_buffer_size(diagnostics)
}

/// Return the size of packed diagnostic records in the JavaScript transfer format.
pub fn diagnostics_buffer_size(diagnostics: &[Diagnostic]) -> usize {
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
        .sum::<usize>()
}

/// Serialize an AST and its diagnostics into `buf` and return the required number of bytes.
///
/// Internal transport; this API and format do not follow semver. Decode with the
/// matching SWC Next JavaScript decoder.
///
/// When `buf` is too small, including when it is empty, this returns the
/// required size without writing a partial payload.
/// `source_utf16_len` must be the UTF-16 code unit length of `tree.source()`.
pub fn serialize_into(
    tree: &AstCtx<'_>,
    diagnostics: &[Diagnostic],
    buf: &mut [u8],
    source_utf16_len: usize,
) -> usize {
    let required = buffer_size(tree, diagnostics);
    if buf.len() < required {
        return required;
    }
    // SAFETY: Serialization only reads handles and storage belonging to this AST.
    let tree = unsafe { tree.ast() };
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

    let flags = (tree.source_type() as u32) | ((tree.lang() as u32) << FLAG_LANG_SHIFT);
    let first_non_ascii_pos =
        u32::try_from(first_non_ascii_pos).expect("AST source length exceeds u32");
    let header = [
        node_count,
        extra_count,
        string_pool_len,
        source_len,
        comment_count,
        0, // attached_comment_count
        diag_count,
        tree.root_node_id().index() as u32,
        flags,
        first_non_ascii_pos,
    ];
    for (value, output) in header
        .into_iter()
        .zip(buf[..HEADER_SIZE].as_chunks_mut::<4>().0.iter_mut())
    {
        output.copy_from_slice(&value.to_le_bytes());
    }
    let mut pos = HEADER_SIZE;

    // ==================== Spans =====================
    let spans = unsafe {
        slice::from_raw_parts(
            tree.nodes.span.as_ptr().cast::<u8>(),
            tree.node_count() * SPAN_SIZE,
        )
    };
    let mut cache = PositionCache::new(&positions, source_len);
    if positions.is_empty() && cfg!(target_endian = "little") {
        write(buf, &mut pos, spans);
    } else {
        let span_buf = &mut buf[pos..pos + spans.len()];
        for (span, output) in
            unsafe { slice::from_raw_parts(tree.nodes.span.as_ptr(), tree.node_count()) }
                .iter()
                .zip(span_buf.as_chunks_mut::<SPAN_SIZE>().0.iter_mut())
        {
            let start = cache.convert(span.start);
            let end = cache.convert(span.end);
            output[..4].copy_from_slice(&start.to_le_bytes());
            output[4..].copy_from_slice(&end.to_le_bytes());
        }
        pos += spans.len();
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
            tree.extra_data.as_raw_slice().as_ptr().cast::<u8>(),
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
    let comments = tree.comments();
    let comments_len = comments.len() * COMMENT_SIZE;
    let comment_buf = &mut buf[pos..pos + comments_len];
    for (comment, output) in comments
        .iter()
        .zip(comment_buf.as_chunks_mut::<COMMENT_SIZE>().0.iter_mut())
    {
        let start = cache.convert(comment.span.start);
        let end = cache.convert(comment.span.end);
        output[..4].copy_from_slice(&start.to_le_bytes());
        output[4..8].copy_from_slice(&end.to_le_bytes());
        output[8..12].copy_from_slice(&comment.span.start.to_le_bytes());
        output[12..].copy_from_slice(&comment.span.end.to_le_bytes());
    }
    pos += comments_len;

    // =================== Diagnostics ====================
    write_diagnostics(diagnostics, buf, &mut pos, &mut cache);

    debug_assert_eq!(pos, required);
    required
}

/// Serialize packed diagnostics with UTF-16 spans into a caller-owned buffer.
/// This internal API and format do not follow semver; use a matching consumer.
///
/// Returns the required size without writing when the buffer is too small.
/// `source_utf16_len` must match the source's UTF-16 length.
pub fn serialize_diagnostics(
    source: &str,
    diagnostics: &[Diagnostic],
    buf: &mut [u8],
    source_utf16_len: usize,
) -> usize {
    let required = diagnostics_buffer_size(diagnostics);
    if buf.len() < required || diagnostics.is_empty() {
        return required;
    }
    let (positions, _) = build_pos_map(source, source_utf16_len);
    let mut cache = PositionCache::new(&positions, source.len() as u32);
    let mut pos = 0;
    write_diagnostics(diagnostics, buf, &mut pos, &mut cache);
    debug_assert_eq!(pos, required);
    required
}

fn write_diagnostics(
    diagnostics: &[Diagnostic],
    buf: &mut [u8],
    pos: &mut usize,
    cache: &mut PositionCache<'_>,
) {
    for diagnostic in diagnostics {
        write(buf, pos, &[diagnostic.severity as u8]);
        let start = cache.convert(diagnostic.span.start);
        let end = cache.convert(diagnostic.span.end);
        write_span(buf, pos, start, end);
        write_bytes_with_length(buf, pos, diagnostic.message.as_bytes());
        write(buf, pos, &[u8::from(diagnostic.help.is_some())]);
        if let Some(help) = &diagnostic.help {
            write_bytes_with_length(buf, pos, help.as_bytes());
        }
        let label_count =
            u32::try_from(diagnostic.labels.len()).expect("diagnostic label count exceeds u32");
        write(buf, pos, &label_count.to_le_bytes());
        for label in &diagnostic.labels {
            let start = cache.convert(label.span.start);
            let end = cache.convert(label.span.end);
            write_span(buf, pos, start, end);
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

/// A read-only AST view backed by immutable transfer sections.
///
/// Mutation is unavailable because its tables may alias immutable buffer bytes.
pub struct BorrowedAst<'a> {
    ast: AstCtx<'a>,
}

impl<'a> Deref for BorrowedAst<'a> {
    type Target = AstCtx<'a>;

    fn deref(&self) -> &Self::Target {
        &self.ast
    }
}

/// Decode an AST produced by the generated JavaScript encoder.
///
/// Internal transport; this API and format do not follow semver.
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
        // SAFETY: All handles used below belong to this AST and remain valid.
        let mut tree = unsafe { Ast::new(source, allocator) };
        tree.reserve(node_count, 0);

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
        deserialize_extra_data(&mut tree, &buf[extra..strings]);

        // =================== String Pool ====================
        tree.strings.initialize_extra(&buf[source_end..pool_end]);

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

        Ok(BorrowedAst { ast: tree.into() })
    }

    #[cfg(target_endian = "little")]
    {
        debug_assert!(buf.as_ptr().addr().is_multiple_of(8));
        let node_count = read_u32(buf, 0) as usize;
        let extra_count = read_u32(buf, 4) as usize;
        let pool_len = read_u32(buf, 8) as usize;
        let source_len = read_u32(buf, 12) as usize;
        let spans = HEADER_SIZE;
        let kinds = spans + node_count * SPAN_SIZE;
        let payloads = align_to_eight(kinds + node_count);
        let extra = payloads + node_count * NODE_PAYLOAD_SIZE;
        let strings = extra + extra_count * EXTRA_DATA_SIZE;
        let source_end = strings + source_len;

        // SAFETY: The caller guarantees the transfer layout and alignment.
        unsafe {
            deserialize_parts(
                &buf[..HEADER_SIZE],
                &buf[spans..kinds],
                &buf[kinds..kinds + node_count],
                &buf[payloads..extra],
                &buf[extra..strings],
                &buf[strings..source_end + pool_len],
                allocator,
            )
        }
    }
}

/// Borrow separate sections produced by the JavaScript encoder without joining them.
///
/// Internal transport; this API and format do not follow semver.
///
/// # Safety
///
/// The sections must obey the same version, encoding, node, and metadata contracts
/// as [`deserialize_from`]. `header` must contain 40 bytes; the other sections must
/// contain the initialized spans, kinds, payloads, extra data, and strings described
/// by that header, without padding. Spans and extra data must be aligned to four
/// bytes, and payloads to eight bytes. Every section must remain immutable for the
/// returned view's lifetime. Comment and diagnostic sections must be empty.
#[cfg(target_endian = "little")]
#[inline]
pub unsafe fn deserialize_parts<'a>(
    header: &[u8],
    spans: &'a [u8],
    kinds: &'a [u8],
    payloads: &'a [u8],
    extra: &'a [u8],
    strings: &'a [u8],
    allocator: &'a Allocator,
) -> Result<BorrowedAst<'a>, DeserializeError> {
    let node_count = read_u32(header, 0) as usize;
    let extra_count = read_u32(header, 4) as usize;
    let pool_len = read_u32(header, 8) as usize;
    let source_len = read_u32(header, 12) as usize;
    let root = read_u32(header, 28) as usize;
    let flags = read_u32(header, 32);
    debug_assert!(spans.as_ptr().addr().is_multiple_of(align_of::<Span>()));
    debug_assert!(payloads.as_ptr().addr().is_multiple_of(8));
    debug_assert!(extra.as_ptr().addr().is_multiple_of(4));

    // SAFETY: The caller guarantees valid encodings, initialized tables, and
    // alignment. BorrowedAst exposes no mutable access to these pointers.
    unsafe {
        let source = std::str::from_utf8_unchecked(&strings[..source_len]);
        let mut ast = Ast::new(source, allocator);
        ast.nodes.span = NonNull::new_unchecked(spans.as_ptr().cast_mut().cast());
        ast.nodes.kind = NonNull::new_unchecked(kinds.as_ptr().cast_mut().cast());
        ast.nodes.data = NonNull::new_unchecked(payloads.as_ptr().cast_mut().cast());
        ast.nodes.len = node_count as u32;
        ast.nodes.cap = node_count as u32;
        ast.extra_data = ArenaIndexVec::from_vec(ArenaVec::from_borrowed_slice_in(
            slice::from_raw_parts(extra.as_ptr().cast(), extra_count),
            allocator,
        ));
        ast.strings
            .borrow_extra(&strings[source_len..source_len + pool_len]);
        ast.root = Some(NodeId::from_usize(root));
        ast.source_type = std::mem::transmute::<u8, SourceType>((flags & 3) as u8);
        ast.lang = std::mem::transmute::<u8, Lang>(((flags >> FLAG_LANG_SHIFT) & 7) as u8);
        Ok(BorrowedAst { ast: ast.into() })
    }
}

// Each entry records the byte offset where the UTF-8 to UTF-16 difference changes.
type PositionMap = Vec<(u32, u32)>;

fn build_pos_map(source: &str, source_utf16_len: usize) -> (PositionMap, usize) {
    if source.len() == source_utf16_len {
        return (Vec::new(), source.len());
    }

    let bytes = source.as_bytes();
    let mut positions = Vec::new();
    let mut byte = 0;
    let mut delta = 0;
    let mut first_non_ascii = source.len();
    while byte < bytes.len() {
        let chunk_end = (byte + ASCII_SCAN_CHUNK_SIZE).min(bytes.len());
        if bytes[byte..chunk_end].is_ascii() {
            byte = chunk_end;
            continue;
        }
        while byte < chunk_end {
            let len = match bytes[byte] {
                0..=0x7f => 1,
                0xc0..=0xdf => 2,
                0xe0..=0xef => 3,
                0xf0..=0xf4 => 4,
                _ => unreachable!("source is valid UTF-8"),
            };
            if len > 1 {
                first_non_ascii = first_non_ascii.min(byte);
                // Match the existing mapping even for offsets inside a UTF-8 scalar.
                for offset in 1..len {
                    if len == 4 && offset == 3 {
                        continue;
                    }
                    delta += 1;
                    positions.push(((byte + offset) as u32, delta));
                }
            }
            byte += len;
        }
    }
    (positions, first_non_ascii)
}

/// Cache an interval over which the UTF-8 to UTF-16 difference is constant.
struct PositionCache<'a> {
    positions: &'a PositionMap,
    source_len: u32,
    next: usize,
    range_start: u32,
    range_end: u64,
    delta: u32,
}

impl<'a> PositionCache<'a> {
    fn new(positions: &'a PositionMap, source_len: u32) -> Self {
        Self {
            positions,
            source_len,
            next: 0,
            range_start: 0,
            range_end: positions
                .first()
                .map_or(u64::from(source_len) + 1, |entry| u64::from(entry.0)),
            delta: 0,
        }
    }

    #[inline]
    fn convert(&mut self, byte: u32) -> u32 {
        if byte > self.source_len {
            return byte;
        }
        if byte >= self.range_start && u64::from(byte) < self.range_end {
            return byte - self.delta;
        }
        self.convert_slow(byte)
    }

    #[cold]
    fn convert_slow(&mut self, byte: u32) -> u32 {
        let positions = self.positions;
        let next = if byte < self.range_start {
            let mut next = self.next;
            let limit = next.saturating_sub(8);
            while next > limit && positions[next - 1].0 > byte {
                next -= 1;
            }
            if next == limit && next > 0 && positions[next - 1].0 > byte {
                next = positions[..next].partition_point(|&(offset, _)| offset <= byte);
            }
            next
        } else {
            let mut next = self.next;
            let limit = (next + 8).min(positions.len());
            while next < limit && positions[next].0 <= byte {
                next += 1;
            }
            if next == limit {
                next += positions[next..].partition_point(|&(offset, _)| offset <= byte);
            }
            next
        };
        self.next = next;
        if next == 0 {
            self.range_start = 0;
            self.delta = 0;
        } else {
            (self.range_start, self.delta) = positions[next - 1];
        }
        self.range_end = positions
            .get(next)
            .map_or(u64::from(self.source_len) + 1, |entry| u64::from(entry.0));
        byte - self.delta
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
fn write_span(buf: &mut [u8], pos: &mut usize, start: u32, end: u32) {
    let output = &mut buf[*pos..*pos + SPAN_SIZE];
    output[..4].copy_from_slice(&start.to_le_bytes());
    output[4..].copy_from_slice(&end.to_le_bytes());
    *pos += SPAN_SIZE;
}

#[inline]
fn read_u32(buf: &[u8], pos: usize) -> u32 {
    u32::from_le_bytes(buf[pos..pos + size_of::<u32>()].try_into().unwrap())
}

#[inline]
fn write_bytes_with_length(buf: &mut [u8], pos: &mut usize, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).expect("byte sequence length exceeds u32");
    let size = size_of::<u32>() + bytes.len();
    let output = &mut buf[*pos..*pos + size];
    output[..4].copy_from_slice(&len.to_le_bytes());
    output[4..].copy_from_slice(bytes);
    *pos += size;
}

#[cfg(target_endian = "big")]
fn deserialize_extra_data(tree: &mut Ast<'_>, bytes: &[u8]) {
    assert!(bytes.len().is_multiple_of(EXTRA_DATA_SIZE));
    let extra_count = bytes.len() / EXTRA_DATA_SIZE;
    tree.extra_data.reserve(extra_count);

    // SAFETY: The allocation has room for `extra_count` entries. Copy through
    // its allocation pointer before creating a byte slice over initialized
    // storage. All bit patterns are valid for `ExtraDataUnit`.
    unsafe {
        let destination = tree.extra_data.as_mut_ptr().cast::<u8>();
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), destination, bytes.len());
        let initialized = slice::from_raw_parts_mut(destination, bytes.len());
        for word in initialized.as_chunks_mut::<4>().0 {
            word.reverse();
        }
        tree.extra_data.set_len(extra_count);
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use swc_next_allocator::Allocator;

    use super::{PositionCache, buffer_size, build_pos_map, deserialize_from, serialize_into};
    use crate::{Ast, AstCtx, Diagnostic, GetSpan, Label, Severity, Span, SubRange};

    #[test]
    fn canonical_little_endian_format() {
        let allocator = Allocator::new();
        // SAFETY: All handles used below belong to this AST and remain valid.
        let mut ast = unsafe { Ast::new("", &allocator) };
        let statement = ast.stmt_empty_statement(Span::new(0, 0));
        let body = ast.add_sub_range([statement]);
        let program = ast.program(Span::new(0, 0), SubRange::empty(), body, None);
        ast.set_root_program(program);

        let ast = AstCtx::from(ast);
        let mut buffer = vec![0; buffer_size(&ast, &[])];
        assert_eq!(serialize_into(&ast, &[], &mut buffer, 0), buffer.len());
        assert_eq!(buffer.len(), 108);

        assert_eq!(u32_at(&buffer, 0), 2); // node count
        assert_eq!(u32_at(&buffer, 4), 3); // extra-data unit count
        assert_eq!(u32_at(&buffer, 28), 1); // root node ID
        assert_eq!(u32_at(&buffer, 36), 0); // no non-ASCII source bytes

        assert_eq!(u32_at(&buffer, 80), 0); // inline directives start
        assert_eq!(u32_at(&buffer, 84), 0); // inline directives end
        assert_eq!(u32_at(&buffer, 88), u32::MAX); // no hashbang
        assert_eq!(u32_at(&buffer, 92), 1); // out-of-line body field

        assert_eq!(u32_at(&buffer, 96), 0); // body element node ID
        assert_eq!(u32_at(&buffer, 100), 0); // body range start
        assert_eq!(u32_at(&buffer, 104), 1); // body range end
    }

    #[test]
    fn packed_extra_data() {
        let allocator = Allocator::new();
        // SAFETY: All handles below are created and used within this AST.
        let mut ast = unsafe { Ast::new("", &allocator) };
        let statement = ast.stmt_empty_statement(Span::new(0, 0));
        let body = ast.add_sub_range([statement]);
        let program = ast.program(Span::new(0, 0), SubRange::empty(), body, None);
        let following = ast.add_sub_range([Some(statement), None]);

        assert_eq!(ast.extra_data_count(), 5);
        assert_eq!(program.body(&ast).len(), 1);
        program.set_body(&mut ast, SubRange::empty());
        assert!(program.body(&ast).is_empty());
        assert!(
            following
                .get_node(&ast, 0)
                .unwrap()
                .unwrap()
                .is_empty_statement(&ast)
        );
        assert!(following.get_node(&ast, 1).unwrap().is_none());
    }

    #[test]
    fn deserialize_little_endian_format() {
        let allocator = Allocator::new();
        // SAFETY: All handles used below belong to this AST and remain valid.
        let mut ast = unsafe { Ast::new("", &allocator) };
        let statement = ast.stmt_empty_statement(Span::new(2, 3));
        let body = ast.add_sub_range([statement]);
        let program = ast.program(Span::new(1, 4), SubRange::empty(), body, None);
        ast.set_root_program(program);

        let ast = AstCtx::from(ast);
        let mut buffer = vec![0; buffer_size(&ast, &[])];
        serialize_into(&ast, &[], &mut buffer, 0);
        let buffer = AlignedBuffer::new(&buffer);

        let decoded_allocator = Allocator::new();
        // SAFETY: The serializer produced a valid, aligned transfer.
        let decoded = unsafe { deserialize_from(&buffer, &decoded_allocator) }.unwrap();
        // SAFETY: All decoded handles below are used only with the decoded AST.
        let decoded = unsafe { decoded.ast() };
        let decoded_program = decoded.root_program();
        let decoded_body = decoded_program.body(decoded);
        let decoded_statement = decoded_body.get_node(decoded, 0).unwrap();

        assert_eq!(decoded.node_count(), 2);
        assert_eq!(decoded.extra_data_count(), 3);
        assert_eq!(decoded_program.span(decoded), Span::new(1, 4));
        assert_eq!(decoded_statement.span(decoded), Span::new(2, 3));
        assert!(decoded_statement.is_empty_statement(decoded));

        // Every Program stores its body range in extra data. Exercise an empty
        // section directly through the production decoder.
        #[cfg(target_endian = "big")]
        {
            // SAFETY: All handles used below belong to this AST and remain valid.
            let mut empty = unsafe { Ast::new("", &allocator) };
            super::deserialize_extra_data(&mut empty, &[]);
            assert_eq!(empty.extra_data_count(), 0);
        }
    }

    #[test]
    fn position_map() {
        let (ascii, first) = build_pos_map("abc", 3);
        assert!(ascii.is_empty());
        assert_eq!(first, 3);

        let source = "aé中😀z";
        let (map, first) = build_pos_map(source, source.encode_utf16().count());
        assert_eq!(first, 1);
        let mut cache = PositionCache::new(&map, source.len() as u32);
        assert_eq!(
            (0..=source.len() as u32)
                .map(|byte| cache.convert(byte))
                .collect::<Vec<_>>(),
            vec![0, 1, 1, 2, 2, 2, 3, 3, 3, 4, 5, 6]
        );
        assert_eq!(cache.convert(12), 12);
    }

    #[test]
    fn position_cache() {
        let source = format!("a{}z", "é".repeat(20));
        let (map, _) = build_pos_map(&source, source.encode_utf16().count());
        let mut cache = PositionCache::new(&map, source.len() as u32);

        assert_eq!(cache.convert(0), 0);
        assert_eq!(cache.convert(1), 1);
        assert_eq!(cache.convert(2), 1);
        assert_eq!(cache.convert(3), 2);
        assert_eq!(cache.convert(40), 20);
        assert_eq!(cache.convert(41), 21);
        assert_eq!(cache.convert(42), 22);
        assert_eq!(cache.convert(43), 43);
        assert_eq!(cache.convert(3), 2);
        assert_eq!(cache.convert(1), 1);
        assert_eq!(cache.convert(41), 21);
    }

    #[test]
    fn serialized_unicode_spans() {
        let allocator = Allocator::new();
        // SAFETY: All handles used below belong to this AST and remain valid.
        let mut ast = unsafe { Ast::new("éa😀b", &allocator) };
        let detached = ast.stmt_empty_statement(Span::new(2, 3));
        let later = ast.stmt_empty_statement(Span::new(7, 8));
        let earlier = ast.stmt_empty_statement(Span::new(2, 3));
        let body = ast.add_sub_range([later, earlier]);
        let program = ast.program(Span::new(0, 8), SubRange::empty(), body, None);
        ast.set_root_program(program);

        let ast = AstCtx::from(ast);
        let mut buffer = vec![0; buffer_size(&ast, &[])];
        serialize_into(&ast, &[], &mut buffer, 5);
        let span = |id: crate::NodeId| {
            let offset = super::HEADER_SIZE + id.index() * super::SPAN_SIZE;
            Span::new(u32_at(&buffer, offset), u32_at(&buffer, offset + 4))
        };
        assert_eq!(span(detached.node_id()), Span::new(1, 2));
        assert_eq!(span(later.node_id()), Span::new(4, 5));
        assert_eq!(span(earlier.node_id()), Span::new(1, 2));
        assert_eq!(span(program.node_id()), Span::new(0, 5));
    }

    #[test]
    fn packed_diagnostic_format() {
        let allocator = Allocator::new();
        // SAFETY: All handles used below belong to this AST and remain valid.
        let mut ast = unsafe { Ast::new("éx", &allocator) };
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
        let ast = AstCtx::from(ast);
        let diagnostics_offset = buffer_size(&ast, &[]);
        let required = buffer_size(&ast, &diagnostics);
        assert_eq!(required, diagnostics_offset + 40);

        let mut packed = vec![0; super::diagnostics_buffer_size(&diagnostics)];
        assert_eq!(
            super::serialize_diagnostics("éx", &diagnostics, &mut packed, 2),
            40
        );

        let mut too_small = vec![0xaa; required - 1];
        assert_eq!(
            serialize_into(&ast, &diagnostics, &mut too_small, 2),
            required
        );
        assert!(too_small.iter().all(|&byte| byte == 0xaa));

        let mut buffer = vec![0; required];
        assert_eq!(serialize_into(&ast, &diagnostics, &mut buffer, 2), required);
        assert_eq!(&buffer[diagnostics_offset..], &packed);
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
