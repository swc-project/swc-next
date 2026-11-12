//! Self-contained, native-layout AST buffers for lazy JavaScript views.
//!
//! Version 2 preserves the Rust AST's structure-of-arrays tables and node IDs.
//! The 72-byte little-endian header contains magic (`SWA2`), version, schema ID,
//! language/source-type flags, total bytes, node count, root ID, span offset,
//! extra count/offset, string offset, source/string lengths, comment offset/count,
//! and diagnostic offset/count/byte length (eighteen `u32` words).
//!
//! Tables follow in order: eight-byte UTF-8 spans, one-byte kinds, padding to
//! eight bytes, sixteen-byte payloads, eight-byte extra entries, source followed
//! by the extra WTF-8 pool, padding to four bytes, eight-byte comment spans,
//! and packed diagnostics. Kinds follow spans; payloads follow aligned kinds.
//! References retain native IDs, including shared nodes and unused arena slots.
//! JavaScript derives occurrence/parent information only when it is needed.
//!
//! Deltas have a 96-byte `SWE3` version-4 header (twenty-four `u32` words):
//! magic/version/schema/total, base node/extra/string/byte counts, new node count,
//! row count, extra count, string pool length, root ID, flags, row offset/capacity,
//! extra offset/capacity, string offset/capacity, and four reserved zero words.
//! Capacity-sized row, extra and string arenas follow contiguously. A 32-byte
//! row contains an eight-byte string span, kind byte, committed status byte (1),
//! two reserved zero bytes, target node ID, and sixteen-byte native payload.
//! Targets are unique stable IDs: existing nodes are replaced in their original
//! slots, and every newly allocated ID has a row. Extra and string references
//! address the logical concatenation of the base and populated delta tables.
//! Spare capacity is excluded from those logical tables.
//!
//! A string pool transfers UTF-16 code units, the form a JS string already is, so the
//! writer copies a string instead of encoding it. The reader transcodes every referenced
//! span to WTF-8 while merging it into the pool an AST reads, then rewrites the spans of
//! the extension's own rows, because an encoded span is not as long as its code units.
//! A span into the base pool keeps its handle: that pool is UTF-8, and base rows cannot
//! reference an extension pool.
//!
//! Legacy extensions with a 64-byte `SWE2` header remain supported. That header
//! contains version/schema/total, base node/extra/string/byte counts, extension
//! node/extra/string counts, new root, flags and span/extra/string offsets.
//! Their appended tables retain the base's column layout and concatenated IDs.
//!
//! [`deserialize`] validates arbitrary buffers before exposing a read-only AST.
//! [`deserialize_unchecked`] follows the existing FFI's trusted-transfer contract.
//! Aligned little-endian base tables can be borrowed directly. Transfers needing
//! merging or endian/alignment conversion are copied into the allocator.
//! Neither entry point permits mutation of borrowed storage. The protocol is
//! versioned and is not a stable on-disk format.

mod decode;
mod schema;
pub use decode::{deserialize, deserialize_unchecked};

use std::{fmt, mem::size_of, slice};

use crate::{Ast, Diagnostic, ExtraData, NodePayload, Span};

pub const HEADER_SIZE: usize = 72;
pub const SPAN_SIZE: usize = 8;
pub const PAYLOAD_SIZE: usize = 16;
pub const VERSION: u32 = 2;
pub const MAGIC: u32 = u32::from_le_bytes(*b"SWA2");
/// Version of a legacy extension part; its string pool transfers UTF-16 code units.
pub const EXTENSION_VERSION: u32 = 4;
/// Version of a delta part; its string pool also transfers UTF-16 code units.
pub const DELTA_VERSION: u32 = 4;
const NONE: u32 = u32::MAX;

const _: () = {
    assert!(size_of::<Span>() == SPAN_SIZE);
    assert!(size_of::<NodePayload>() == PAYLOAD_SIZE);
    assert!(size_of::<ExtraData>() == 8);
};

/// A transfer cannot exceed 32-bit offsets or contain a structural cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerializeError {
    TooLarge,
    InvalidTree,
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooLarge => "AST buffer exceeds the 32-bit format limit",
            Self::InvalidTree => "AST root contains a cycle or an invalid node reference",
        })
    }
}

impl std::error::Error for SerializeError {}

/// A malformed or incompatible pair of AST buffers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeserializeError(pub(super) &'static str);

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for DeserializeError {}

fn checked(value: usize) -> Result<u32, SerializeError> {
    u32::try_from(value).map_err(|_| SerializeError::TooLarge)
}

fn add(left: usize, right: usize) -> Result<usize, SerializeError> {
    let value = left.checked_add(right).ok_or(SerializeError::TooLarge)?;
    checked(value)?;
    Ok(value)
}

fn multiply(left: usize, right: usize) -> Result<usize, SerializeError> {
    let value = left.checked_mul(right).ok_or(SerializeError::TooLarge)?;
    checked(value)?;
    Ok(value)
}

fn put(buffer: &mut [u8], offset: usize, value: u32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// Copy native AST tables, source, comments and diagnostics into one buffer.
///
/// UTF-8 spans and native IDs are preserved without traversing/rebasing the tree.
/// Graph validation is performed by the checked reader, not by the serializer.
pub fn serialize(tree: &Ast<'_>, diagnostics: &[Diagnostic]) -> Result<Vec<u8>, SerializeError> {
    let count = tree.node_count();
    let root = tree.root.ok_or(SerializeError::InvalidTree)?;
    if root.index() >= count {
        return Err(SerializeError::InvalidTree);
    }
    let spans = HEADER_SIZE;
    let kinds = add(spans, multiply(count, SPAN_SIZE)?)?;
    let payloads = add(add(kinds, count)?, 7)? & !7;
    let extra = add(payloads, multiply(count, PAYLOAD_SIZE)?)?;
    let strings = add(extra, multiply(tree.extra_data.len(), 8)?)?;
    let string_len = add(tree.source().len(), tree.strings.extra_bytes().len())?;
    let comments = add(add(strings, string_len)?, 3)? & !3;
    let diagnostic_offset = add(comments, multiply(tree.comments().len(), 8)?)?;
    let mut diagnostic_len = 0;
    for diagnostic in diagnostics {
        diagnostic_len = add(diagnostic_len, add(18, diagnostic.message.len())?)?;
        if let Some(help) = &diagnostic.help {
            diagnostic_len = add(diagnostic_len, add(4, help.len())?)?;
        }
        for label in &diagnostic.labels {
            diagnostic_len = add(diagnostic_len, add(12, label.message.len())?)?;
        }
    }
    let total = add(diagnostic_offset, diagnostic_len)?;
    let header = [
        MAGIC,
        VERSION,
        schema::SCHEMA_ID,
        (tree.source_type() as u32) | ((tree.lang() as u32) << 2),
        checked(total)?,
        checked(count)?,
        checked(root.index())?,
        checked(spans)?,
        checked(tree.extra_data.len())?,
        checked(extra)?,
        checked(strings)?,
        checked(tree.source().len())?,
        checked(string_len)?,
        checked(comments)?,
        checked(tree.comments().len())?,
        checked(diagnostic_offset)?,
        checked(diagnostics.len())?,
        checked(diagnostic_len)?,
    ];
    let mut buffer = Vec::with_capacity(total);
    for word in header {
        buffer.extend_from_slice(&word.to_le_bytes());
    }
    #[cfg(target_endian = "little")]
    // SAFETY: All native table entries, including payload padding, are initialized.
    unsafe {
        buffer.extend_from_slice(slice::from_raw_parts(
            tree.nodes.span.as_ptr().cast::<u8>(),
            count * SPAN_SIZE,
        ));
    }
    #[cfg(target_endian = "big")]
    for index in 0..count {
        let span = tree.span(crate::NodeId::from_usize(index));
        buffer.extend_from_slice(&span.start.to_le_bytes());
        buffer.extend_from_slice(&span.end.to_le_bytes());
    }
    // SAFETY: NodeKind is a byte-sized enum with initialized entries.
    unsafe {
        buffer.extend_from_slice(slice::from_raw_parts(
            tree.nodes.kind.as_ptr().cast::<u8>(),
            count,
        ));
    }
    buffer.resize(payloads, 0);
    #[cfg(target_endian = "little")]
    // SAFETY: Payload and extra-data tables are initialized and little-endian.
    unsafe {
        buffer.extend_from_slice(slice::from_raw_parts(
            tree.nodes.data.as_ptr().cast::<u8>(),
            count * PAYLOAD_SIZE,
        ));
        buffer.extend_from_slice(slice::from_raw_parts(
            tree.extra_data.as_raw_slice().as_ptr().cast::<u8>(),
            tree.extra_data.len() * 8,
        ));
    }
    #[cfg(target_endian = "big")]
    {
        for index in 0..count {
            let id = crate::NodeId::from_usize(index);
            let kind = tree.node_kind(id);
            let mut raw = unsafe { (*tree.nodes.data.as_ptr().add(index)).raw };
            crate::generated::ast_payload::swap_payload_endian(&mut raw, kind);
            buffer.extend_from_slice(&raw);
        }
        for value in tree.extra_data.as_raw_slice() {
            buffer.extend_from_slice(&value.raw().to_le_bytes());
        }
    }
    buffer.extend_from_slice(tree.source().as_bytes());
    buffer.extend_from_slice(tree.strings.extra_bytes());
    buffer.resize(comments, 0);
    for comment in tree.comments() {
        buffer.extend_from_slice(&comment.span.start.to_le_bytes());
        buffer.extend_from_slice(&comment.span.end.to_le_bytes());
    }
    buffer.resize(total, 0);
    let mut cursor = diagnostic_offset;
    for diagnostic in diagnostics {
        buffer[cursor] = diagnostic.severity as u8;
        cursor += 1;
        write_word(&mut buffer, &mut cursor, diagnostic.span.start);
        write_word(&mut buffer, &mut cursor, diagnostic.span.end);
        write_string(&mut buffer, &mut cursor, &diagnostic.message);
        buffer[cursor] = u8::from(diagnostic.help.is_some());
        cursor += 1;
        if let Some(help) = &diagnostic.help {
            write_string(&mut buffer, &mut cursor, help);
        }
        write_word(&mut buffer, &mut cursor, checked(diagnostic.labels.len())?);
        for label in &diagnostic.labels {
            write_word(&mut buffer, &mut cursor, label.span.start);
            write_word(&mut buffer, &mut cursor, label.span.end);
            write_string(&mut buffer, &mut cursor, &label.message);
        }
    }
    debug_assert_eq!(cursor, total);
    Ok(buffer)
}

fn write_word(buffer: &mut [u8], cursor: &mut usize, value: u32) {
    put(buffer, *cursor, value);
    *cursor += 4;
}

fn write_string(buffer: &mut [u8], cursor: &mut usize, value: &str) {
    // The complete buffer's checked size already bounds each string length.
    write_word(buffer, cursor, value.len() as u32);
    buffer[*cursor..*cursor + value.len()].copy_from_slice(value.as_bytes());
    *cursor += value.len();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Label, Span, SubRange};
    use swc_next_allocator::Allocator;

    fn word(buffer: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap())
    }
    fn payloads(buffer: &[u8]) -> usize {
        (HEADER_SIZE + word(buffer, 20) as usize * 9 + 7) & !7
    }
    fn program<'a>(allocator: &'a Allocator, source: &'a str) -> Ast<'a> {
        // SAFETY: The caller keeps the AST, its source, and its allocator alive together.
        let mut ast = unsafe { Ast::new(source, allocator) };
        let statement = ast.stmt_empty_statement(Span::new(0, source.len() as u32));
        let body = ast.add_sub_range([statement]);
        let root = ast.program(
            Span::new(0, source.len() as u32),
            SubRange::empty(),
            body,
            None,
        );
        ast.set_root_program(root);
        ast
    }
    fn delta(buffer: &[u8], new_nodes: u32, targets: &[u32]) -> Vec<u8> {
        let base_nodes = word(buffer, 20);
        let row_capacity = targets.len() + 1;
        let extra = 96 + row_capacity * 32;
        let strings = extra + 4 * 8;
        let total = strings + 8;
        let header = [
            u32::from_le_bytes(*b"SWE3"),
            DELTA_VERSION,
            schema::SCHEMA_ID,
            total as u32,
            base_nodes,
            word(buffer, 32),
            word(buffer, 48),
            buffer.len() as u32,
            new_nodes,
            targets.len() as u32,
            0,
            0,
            word(buffer, 24),
            word(buffer, 12),
            96,
            row_capacity as u32,
            extra as u32,
            4,
            strings as u32,
            8,
            0,
            0,
            0,
            0,
        ];
        let mut extension = vec![0; total];
        for (index, value) in header.into_iter().enumerate() {
            put(&mut extension, index * 4, value);
        }
        for (index, &target) in targets.iter().enumerate() {
            let source = if target < base_nodes {
                target as usize
            } else {
                0
            };
            let row = 96 + index * 32;
            let span = HEADER_SIZE + source * SPAN_SIZE;
            let payload = payloads(buffer) + source * PAYLOAD_SIZE;
            extension[row..row + SPAN_SIZE].copy_from_slice(&buffer[span..span + SPAN_SIZE]);
            extension[row + 8] = buffer[HEADER_SIZE + base_nodes as usize * SPAN_SIZE + source];
            extension[row + 9] = 1;
            put(&mut extension, row + 12, target);
            extension[row + 16..row + 32].copy_from_slice(&buffer[payload..payload + PAYLOAD_SIZE]);
        }
        extension
    }
    fn reject_delta_layout(buffer: &[u8], extension: &[u8]) {
        let allocator = Allocator::new();
        assert!(deserialize(buffer, extension, &allocator).is_err());
        // Malformed transfer metadata is rejected before exposing native tables,
        // including by the trusted reader.
        assert!(unsafe { deserialize_unchecked(buffer, extension, &allocator) }.is_err());
    }

    #[test]
    fn self_contained_utf8_spans() {
        let allocator = Allocator::new();
        let ast = program(&allocator, "中😀");
        let mut diagnostic = Diagnostic::error("message", Span::new(3, 7));
        diagnostic.labels.push(Label {
            span: Span::new(0, 3),
            message: "label".into(),
        });
        let buffer = serialize(&ast, &[diagnostic]).unwrap();
        assert_eq!(&buffer[..4], b"SWA2");
        assert_eq!(word(&buffer, 16) as usize, buffer.len());
        assert_eq!(word(&buffer, HEADER_SIZE + 4), 7);
        assert_eq!(
            word(
                &buffer,
                HEADER_SIZE + word(&buffer, 24) as usize * SPAN_SIZE + 4
            ),
            7
        );
        let source = word(&buffer, 40) as usize;
        assert_eq!(
            &buffer[source..source + word(&buffer, 44) as usize],
            "中😀".as_bytes()
        );
        let diagnostics = word(&buffer, 60) as usize;
        assert_eq!(word(&buffer, diagnostics + 1), 3);
        assert_eq!(word(&buffer, diagnostics + 5), 7);
    }

    #[test]
    fn shared_nodes_and_unused_slots() {
        let allocator = Allocator::new();
        let mut ast = unsafe { Ast::new("", &allocator) };
        let _unused = ast.stmt_empty_statement(Span::new(0, 0));
        let statement = ast.stmt_empty_statement(Span::new(0, 0));
        let body = ast.add_sub_range([statement, statement]);
        let root = ast.program(Span::new(0, 0), SubRange::empty(), body, None);
        ast.set_root_program(root);
        let buffer = serialize(&ast, &[]).unwrap();
        let decoded = deserialize(&buffer, &[], &allocator).unwrap();
        // SAFETY: All decoded handles below are used only with the decoded AST.
        let decoded = unsafe { decoded.ast() };
        assert_eq!(decoded.node_count(), ast.node_count());
        assert_eq!(decoded.root_program().node_id(), root.node_id());
        let body = decoded.root_program().body(decoded);
        assert_eq!(
            body.get_node(decoded, 0).unwrap().node_id(),
            statement.node_id()
        );
        assert_eq!(
            body.get_node(decoded, 1).unwrap().node_id(),
            statement.node_id()
        );
    }

    #[test]
    fn structural_cycles() {
        let allocator = Allocator::new();
        let mut ast = unsafe { Ast::new("", &allocator) };
        let statement = ast.stmt_block_statement(Span::new(0, 0), SubRange::empty());
        let body = ast.add_sub_range([statement]);
        statement
            .as_block_statement(&ast)
            .unwrap()
            .set_body(&mut ast, body);
        let root = ast.program(Span::new(0, 0), SubRange::empty(), body, None);
        ast.set_root_program(root);
        let buffer = serialize(&ast, &[]).unwrap();
        assert!(deserialize(&buffer, &[], &allocator).is_err());
    }

    #[test]
    fn decoded_buffers() {
        let allocator = Allocator::new();
        let original = program(&allocator, "中😀");
        let buffer = serialize(&original, &[]).unwrap();
        let decoded = deserialize(&buffer, &[], &allocator).unwrap();
        // SAFETY: All decoded handles below are used only with the decoded AST.
        let decoded = unsafe { decoded.ast() };
        assert_eq!(decoded.source(), "中😀");
        assert_eq!(decoded.span(decoded.root_program().node_id()).end, 7);
        assert_eq!(serialize(decoded, &[]).unwrap(), buffer);
        #[cfg(target_endian = "little")]
        assert_eq!(
            decoded.nodes.data.as_ptr().cast::<u8>().cast_const(),
            unsafe { buffer.as_ptr().add(payloads(&buffer)) }
        );
        // Trusted parser transfers share the same borrowed table path.
        let trusted = unsafe { deserialize_unchecked(&buffer, &[], &allocator) }.unwrap();
        assert_eq!(
            trusted.root_program().node_id(),
            decoded.root_program().node_id()
        );

        let mut padded = vec![0];
        padded.extend_from_slice(&buffer);
        let unaligned = deserialize(&padded[1..], &[], &allocator).unwrap();
        // SAFETY: All decoded handles below are used only with the decoded AST.
        let unaligned = unsafe { unaligned.ast() };
        assert_eq!(unaligned.source(), "中😀");
        assert_eq!(serialize(unaligned, &[]).unwrap(), buffer);
    }

    #[test]
    fn extension_references() {
        let allocator = Allocator::new();
        let original = program(&allocator, "");
        let buffer = serialize(&original, &[]).unwrap();
        let base_nodes = word(&buffer, 20);
        let base_extra = word(&buffer, 32);
        let root = word(&buffer, 24) as usize;
        let mut extension = vec![0; 112];
        let header = [
            u32::from_le_bytes(*b"SWE2"),
            EXTENSION_VERSION,
            schema::SCHEMA_ID,
            112,
            base_nodes,
            base_extra,
            word(&buffer, 48),
            buffer.len() as u32,
            1,
            2,
            0,
            base_nodes,
            1,
            64,
            96,
            112,
        ];
        for (index, value) in header.into_iter().enumerate() {
            put(&mut extension, index * 4, value);
        }
        extension[64..72].copy_from_slice(
            &buffer[HEADER_SIZE + root * SPAN_SIZE..HEADER_SIZE + (root + 1) * SPAN_SIZE],
        );
        extension[72] = buffer[HEADER_SIZE + base_nodes as usize * SPAN_SIZE + root];
        extension[80..96].copy_from_slice(
            &buffer[payloads(&buffer) + root * PAYLOAD_SIZE
                ..payloads(&buffer) + (root + 1) * PAYLOAD_SIZE],
        );
        put(&mut extension, 92, base_extra);
        put(&mut extension, 96, base_extra + 1);
        put(&mut extension, 100, base_extra + 2);
        put(&mut extension, 104, 0);
        let decoded = deserialize(&buffer, &extension, &allocator).unwrap();
        // SAFETY: All decoded handles below are used only with the decoded AST.
        let decoded = unsafe { decoded.ast() };
        assert_eq!(decoded.source_type(), crate::SourceType::Script);
        assert_eq!(decoded.node_count(), base_nodes as usize + 1);
        assert!(
            decoded
                .root_program()
                .body(decoded)
                .get_node(decoded, 0)
                .unwrap()
                .is_empty_statement(decoded)
        );
        put(&mut extension, 16, base_nodes + 1);
        assert!(deserialize(&buffer, &extension, &allocator).is_err());
    }

    #[test]
    fn delta_merging() {
        let allocator = Allocator::new();
        let original = program(&allocator, ";");
        let buffer = serialize(&original, &[]).unwrap();
        let base_nodes = word(&buffer, 20);
        let base_extra = word(&buffer, 32);
        let root = word(&buffer, 24);
        let mut extension = delta(&buffer, 1, &[base_nodes, 0, root]);
        let rows = word(&extension, 56) as usize;
        let extra = word(&extension, 64) as usize;
        extension[rows + 32 + 8] = crate::NodeKind::DebuggerStatement as u8;
        put(&mut extension, rows + 32, 1);
        put(&mut extension, rows + 64 + 28, base_extra + 2);
        put(&mut extension, 40, 3);
        put(&mut extension, extra, 0);
        put(&mut extension, extra + 8, base_nodes);
        put(&mut extension, extra + 16, base_extra);
        put(&mut extension, extra + 20, base_extra + 2);
        let decoded = deserialize(&buffer, &extension, &allocator).unwrap();
        // SAFETY: All decoded handles below are used only with the decoded AST.
        let decoded = unsafe { decoded.ast() };
        assert_eq!(decoded.node_count(), base_nodes as usize + 1);
        assert_eq!(decoded.extra_data.len(), base_extra as usize + 3);
        assert_eq!(decoded.root_program().node_id().index(), root as usize);
        let body = decoded.root_program().body(decoded);
        let changed = body.get_node(decoded, 0).unwrap();
        assert_eq!(changed.node_id().index(), 0);
        assert!(changed.is_debugger_statement(decoded));
        assert_eq!(decoded.span(changed.node_id()), Span::new(1, 1));
        let appended = body.get_node(decoded, 1).unwrap();
        assert_eq!(appended.node_id().index(), base_nodes as usize);
        assert!(appended.is_empty_statement(decoded));
        let trusted = unsafe { deserialize_unchecked(&buffer, &extension, &allocator) }.unwrap();
        // SAFETY: Both decoded ASTs are only serialized for comparison.
        let trusted = unsafe { trusted.ast() };
        assert_eq!(
            serialize(trusted, &[]).unwrap(),
            serialize(decoded, &[]).unwrap()
        );
    }

    #[test]
    fn delta_rows() {
        let allocator = Allocator::new();
        let original = program(&allocator, "");
        let buffer = serialize(&original, &[]).unwrap();
        let base_nodes = word(&buffer, 20);

        let extension = delta(&buffer, 0, &[0, 0]);
        reject_delta_layout(&buffer, &extension);

        let extension = delta(&buffer, 0, &[base_nodes]);
        reject_delta_layout(&buffer, &extension);

        let extension = delta(&buffer, 1, &[0]);
        reject_delta_layout(&buffer, &extension);

        let mut extension = delta(&buffer, 0, &[0]);
        extension[96 + 9] = 0;
        reject_delta_layout(&buffer, &extension);

        let mut extension = delta(&buffer, 0, &[0]);
        extension[96 + 10] = 1;
        reject_delta_layout(&buffer, &extension);
    }

    #[test]
    fn delta_layout() {
        let allocator = Allocator::new();
        let original = program(&allocator, "");
        let buffer = serialize(&original, &[]).unwrap();

        let mut extension = delta(&buffer, 0, &[0]);
        put(&mut extension, 36, 3);
        reject_delta_layout(&buffer, &extension);

        let mut extension = delta(&buffer, 0, &[0]);
        put(&mut extension, 40, 5);
        reject_delta_layout(&buffer, &extension);

        let mut extension = delta(&buffer, 0, &[0]);
        put(&mut extension, 44, 9);
        reject_delta_layout(&buffer, &extension);

        let mut extension = delta(&buffer, 0, &[0]);
        put(&mut extension, 64, 96);
        reject_delta_layout(&buffer, &extension);

        let mut extension = delta(&buffer, 0, &[0]);
        put(&mut extension, 80, 1);
        reject_delta_layout(&buffer, &extension);

        let mut extension = delta(&buffer, 0, &[0]);
        put(&mut extension, 16, 0);
        reject_delta_layout(&buffer, &extension);
    }

    #[test]
    fn delta_references() {
        let allocator = Allocator::new();
        let original = program(&allocator, "");
        let buffer = serialize(&original, &[]).unwrap();

        let mut extension = delta(&buffer, 0, &[0]);
        extension[96 + 8] = crate::NodeKind::IdentifierReference as u8;
        assert!(deserialize(&buffer, &extension, &allocator).is_err());

        let mut extension = delta(&buffer, 0, &[0]);
        extension[96 + 8] = crate::NodeKind::BlockStatement as u8;
        put(&mut extension, 96 + 20, 1);
        assert!(deserialize(&buffer, &extension, &allocator).is_err());
    }

    #[test]
    fn delta_strings() {
        let allocator = Allocator::new();
        let original = program(&allocator, ";");
        let buffer = serialize(&original, &[]).unwrap();
        let base_nodes = word(&buffer, 20);
        let base_strings = word(&buffer, 48);
        let mut extension = delta(&buffer, 1, &[0, base_nodes]);
        let strings = word(&extension, 72) as usize;
        extension[96 + 8] = crate::NodeKind::ExpressionStatement as u8;
        put(&mut extension, 96 + 16, base_nodes);
        extension[128 + 8] = crate::NodeKind::IdentifierReference as u8;
        put(&mut extension, 128 + 16, base_strings);
        put(&mut extension, 128 + 20, base_strings + 2);
        put(&mut extension, 44, 2);
        // The pool transfers UTF-16 code units, and a handle spans them.
        extension[strings..strings + 2].copy_from_slice(&0xe9u16.to_le_bytes());
        let decoded = deserialize(&buffer, &extension, &allocator).unwrap();
        // SAFETY: All decoded handles below are used only with the decoded AST.
        let decoded = unsafe { decoded.ast() };
        // The pool the AST reads holds transcoded WTF-8, and the row's handle points at it.
        assert_eq!(decoded.strings.extra_bytes(), "é".as_bytes());
        let body = decoded.root_program().body(decoded);
        let statement = body
            .get_node(decoded, 0)
            .unwrap()
            .as_expression_statement(decoded)
            .unwrap();
        let expression = statement.expression(decoded);
        assert_eq!(expression.node_id().index(), base_nodes as usize);
        let identifier = expression.as_identifier_reference(decoded).unwrap();
        assert_eq!(decoded.get_utf8(identifier.name(decoded)), "é");

        // An unpaired surrogate cannot be the value of a UTF-8 field.
        extension[strings..strings + 2].copy_from_slice(&0xd800u16.to_le_bytes());
        assert!(deserialize(&buffer, &extension, &allocator).is_err());

        // A handle must fall on a code-unit boundary.
        extension[strings..strings + 2].copy_from_slice(&0xe9u16.to_le_bytes());
        put(&mut extension, 128 + 20, base_strings + 1);
        assert!(deserialize(&buffer, &extension, &allocator).is_err());
    }

    #[test]
    fn invalid_fields() {
        let allocator = Allocator::new();
        let mut ast = unsafe { Ast::new("é", &allocator) };
        let name = ast.source_slice(0, 2);
        let value = ast.expr_identifier_reference(Span::new(0, 2), name);
        let statement = ast.stmt_expression_statement(Span::new(0, 2), value);
        let body = ast.add_sub_range([statement]);
        let root = ast.program(Span::new(0, 2), SubRange::empty(), body, None);
        ast.set_root_program(root);
        let mut buffer = serialize(&ast, &[]).unwrap();
        let start = payloads(&buffer);
        put(&mut buffer, start, 1);
        assert!(deserialize(&buffer, &[], &allocator).is_err());

        let mut ast = unsafe { Ast::new("{}", &allocator) };
        let empty = ast.stmt_empty_statement(Span::new(1, 1));
        let body = ast.add_sub_range([empty]);
        let block = ast.stmt_block_statement(Span::new(0, 2), body);
        let body = ast.add_sub_range([block]);
        let root = ast.program(Span::new(0, 2), SubRange::empty(), body, None);
        ast.set_root_program(root);
        let mut buffer = serialize(&ast, &[]).unwrap();
        let list = word(&buffer, payloads(&buffer) + PAYLOAD_SIZE) as usize;
        let extra = word(&buffer, 36) as usize;
        put(&mut buffer, extra + list * 8, 1);
        assert!(deserialize(&buffer, &[], &allocator).is_err());
    }
}
