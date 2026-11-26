//! Self-contained, native-layout AST buffers for lazy JavaScript views.
//!
//! A buffer serializes the AST's arrays with their lengths and stores root/source
//! metadata as AST fields. There is no protocol header, magic, version or schema
//! fingerprint. Spans retain UTF-8 byte offsets and node IDs retain native indices.
//!
//! The node array starts with its count and alignment padding, then spans, kinds
//! and aligned payloads. Extra-data, strings, comments, diagnostics and source
//! position rows follow, each with its own count or byte length. Root ID and
//! language/source-type flags end the buffer.
//!
//! JavaScript edits existing spans, kinds, payloads and reference slots in place.
//! Incremental buffers contain only newly allocated nodes, extra data and strings,
//! with used counts and capacities followed by root/source metadata and
//! base counts. New node IDs follow the base nodes contiguously; each 32-byte slot
//! holds a span, kind, alignment padding and payload. Rust merges the parts into
//! contiguous AST storage. A row-capacity high bit marks UTF-8/WTF-8 strings,
//! whose handles are retained; legacy UTF-16 strings are transcoded and rebased.
//!
//! Position rows contain UTF-8 start, prior byte excess, character width and UTF-8
//! end. Adjacent non-ASCII characters of equal width share a row; JavaScript queries
//! the Rust-built index without scanning source bytes or rewriting native spans.
//!
//! Checked reads validate bounds, typed references and the program's tree structure. Trusted
//! reads require valid bytes from the matching parser/encoder. JavaScript may edit
//! between reads; keep both inputs unchanged while a borrowed owner is alive.
//!
mod decode;
mod schema;
pub use decode::{deserialize, deserialize_unchecked};

use std::{fmt, mem::size_of, ops::Deref, slice};

use crate::{
    Ast, AstCtx, AstNode, CloneIn, Diagnostic, ExportSpecifier, ExtraDataUnit, NodeId, NodeKind,
    NodePayload, Span,
};

pub const SPAN_SIZE: usize = 8;
pub const PAYLOAD_SIZE: usize = 16;
pub(super) const POSITION_RUN_SIZE: usize = 16;
const ASCII_SCAN_CHUNK_SIZE: usize = 64;
const NONE: u32 = u32::MAX;

const _: () = {
    assert!(size_of::<Span>() == SPAN_SIZE);
    assert!(size_of::<NodePayload>() == PAYLOAD_SIZE);
    assert!(size_of::<ExtraDataUnit>() == 4);
};

/// Serialization requires a valid root and offsets that fit the 32-bit format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerializeError {
    TooLarge,
    InvalidTree,
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooLarge => "AST buffer exceeds the 32-bit format limit",
            Self::InvalidTree => "AST root is missing or refers to an invalid node",
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

/// A read-only AST view backed by immutable transfer sections.
///
/// Mutation is unavailable because its tables may alias immutable buffer bytes.
pub struct BorrowedAst<'a> {
    ast: AstCtx<'a>,
}

impl<'a> BorrowedAst<'a> {
    pub(crate) fn from_ast(ast: Ast<'a>) -> Self {
        Self { ast: ast.into() }
    }
}

impl<'a> Deref for BorrowedAst<'a> {
    type Target = AstCtx<'a>;

    fn deref(&self) -> &Self::Target {
        &self.ast
    }
}

/// Give each shorthand export name its own node before encoding a tree.
pub fn split_export_specifier_names(ast: &mut Ast<'_>) {
    let count = ast.node_count();
    let mut start = 0;
    // Shorthand exports can share a name handle. The transferred tree needs a
    // distinct node for each position so edits to either name are independent.
    if count == 0 {
        return;
    }
    loop {
        // SAFETY: The parser initialized all original kinds. Refresh the slice
        // whenever cloning a node may have reallocated the table.
        let kinds = unsafe { slice::from_raw_parts(ast.nodes.kind.as_ptr().cast::<u8>(), count) };
        let Some(offset) = memchr::memchr(NodeKind::ExportSpecifier as u8, &kinds[start..]) else {
            break;
        };
        let index = start + offset;
        // SAFETY: The kind table identifies this node as an export specifier.
        let specifier =
            unsafe { ExportSpecifier::from_node_id_unchecked(NodeId::from_usize(index)) };
        let local = specifier.local(ast);
        if local.node_id() == specifier.exported(ast).node_id() {
            let local = local.clone_in(ast);
            specifier.set_local(ast, local);
        }
        start = index + 1;
    }
}

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
/// `source_utf16_len` must be the UTF-16 code unit length of `tree.source()`.
/// UTF-8 spans and native IDs are preserved without traversing/rebasing the tree.
/// Graph validation is performed by the checked reader, not by the serializer.
pub fn serialize(
    tree: &Ast<'_>,
    diagnostics: &[Diagnostic],
    source_utf16_len: usize,
) -> Result<Vec<u8>, SerializeError> {
    let count = tree.node_count();
    let root = tree.root.ok_or(SerializeError::InvalidTree)?;
    if root.index() >= count {
        return Err(SerializeError::InvalidTree);
    }
    let spans = 8;
    let kinds = add(spans, multiply(count, SPAN_SIZE)?)?;
    let payloads = add(add(kinds, count)?, 7)? & !7;
    let extra_count_offset = add(payloads, multiply(count, PAYLOAD_SIZE)?)?;
    let extra = add(extra_count_offset, 4)?;
    let strings_info = add(extra, multiply(tree.extra_data.len(), 4)?)?;
    let strings = add(strings_info, 8)?;
    let string_len = add(tree.source().len(), tree.strings.extra_bytes().len())?;
    let comments_info = add(add(strings, string_len)?, 3)? & !3;
    let comments = add(comments_info, 4)?;
    let diagnostics_info = add(comments, multiply(tree.comments().len(), 8)?)?;
    let diagnostic_offset = add(diagnostics_info, 8)?;
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
    let diagnostic_end = add(diagnostic_offset, diagnostic_len)?;
    let positions_info = add(diagnostic_end, 3)? & !3;
    let positions = add(positions_info, 8)?;
    let mut position_rows = Vec::new();
    let (position_count, byte_excess) =
        append_position_runs(&mut position_rows, tree.source(), source_utf16_len)?;
    let metadata = add(positions, position_rows.len())?;
    let total = add(metadata, 8)?;
    let mut buffer = Vec::with_capacity(total);
    buffer.extend_from_slice(&checked(count)?.to_le_bytes());
    buffer.extend_from_slice(&0u32.to_le_bytes());
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
        buffer.extend_from_slice(&checked(tree.extra_data.len())?.to_le_bytes());
        buffer.extend_from_slice(slice::from_raw_parts(
            tree.extra_data.as_raw_slice().as_ptr().cast::<u8>(),
            tree.extra_data.len() * 4,
        ));
    }
    if cfg!(target_endian = "big") {
        for index in 0..count {
            let id = crate::NodeId::from_usize(index);
            let kind = tree.node_kind(id);
            let mut raw = unsafe { (*tree.nodes.data.as_ptr().add(index)).raw };
            crate::generated::ast_payload::swap_payload_endian(&mut raw, kind);
            buffer.extend_from_slice(&raw);
        }
        buffer.extend_from_slice(&checked(tree.extra_data.len())?.to_le_bytes());
        for value in tree.extra_data.as_raw_slice() {
            buffer.extend_from_slice(&value.raw().to_le_bytes());
        }
    }
    buffer.extend_from_slice(&checked(tree.source().len())?.to_le_bytes());
    buffer.extend_from_slice(&checked(string_len)?.to_le_bytes());
    buffer.extend_from_slice(tree.source().as_bytes());
    buffer.extend_from_slice(tree.strings.extra_bytes());
    buffer.resize(comments_info, 0);
    buffer.extend_from_slice(&checked(tree.comments().len())?.to_le_bytes());
    for comment in tree.comments() {
        buffer.extend_from_slice(&comment.span.start.to_le_bytes());
        buffer.extend_from_slice(&comment.span.end.to_le_bytes());
    }
    buffer.extend_from_slice(&checked(diagnostics.len())?.to_le_bytes());
    buffer.extend_from_slice(&checked(diagnostic_len)?.to_le_bytes());
    buffer.resize(positions, 0);
    put(&mut buffer, positions_info, position_count);
    put(&mut buffer, positions_info + 4, byte_excess);
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
    debug_assert_eq!(cursor, diagnostic_end);
    buffer.extend_from_slice(&position_rows);
    buffer.extend_from_slice(&checked(root.index())?.to_le_bytes());
    buffer.extend_from_slice(
        &((tree.source_type() as u32) | ((tree.lang() as u32) << 2)).to_le_bytes(),
    );
    debug_assert_eq!(buffer.len(), total);
    Ok(buffer)
}

/// Encode the sparse position index in one source pass.
/// The source is valid UTF-8, so inspecting leading bytes is enough to find widths.
/// A source whose byte length equals its UTF-16 length is ASCII and needs no rows.
fn append_position_runs(
    buffer: &mut Vec<u8>,
    source: &str,
    source_utf16_len: usize,
) -> Result<(u32, u32), SerializeError> {
    if source.len() == source_utf16_len {
        return Ok((0, 0));
    }
    let bytes = source.as_bytes();
    let table_start = buffer.len();
    let mut cursor = 0;
    let mut byte_excess = 0u32;
    while cursor < bytes.len() {
        let chunk_end = (cursor + ASCII_SCAN_CHUNK_SIZE).min(bytes.len());
        if bytes[cursor..chunk_end].is_ascii() {
            cursor = chunk_end;
            continue;
        }
        while cursor < chunk_end {
            let first = bytes[cursor];
            if first < 0x80 {
                cursor += 1;
                continue;
            }
            let width = if first < 0xe0 {
                2
            } else if first < 0xf0 {
                3
            } else {
                4
            };
            let start = cursor;
            let prior_excess = byte_excess;
            let excess_per_char = if width == 4 { 2 } else { width - 1 };
            loop {
                cursor += width;
                byte_excess += excess_per_char as u32;
                let Some(&next) = bytes.get(cursor) else {
                    break;
                };
                let next_width = if next < 0x80 {
                    1
                } else if next < 0xe0 {
                    2
                } else if next < 0xf0 {
                    3
                } else {
                    4
                };
                if next_width != width {
                    break;
                }
            }
            add(buffer.len(), POSITION_RUN_SIZE)?;
            for word in [start as u32, prior_excess, width as u32, cursor as u32] {
                buffer.extend_from_slice(&word.to_le_bytes());
            }
        }
    }
    Ok((
        checked((buffer.len() - table_start) / POSITION_RUN_SIZE)?,
        byte_excess,
    ))
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
    use crate::{AstUtf8, Span, SubRange};
    use swc_next_allocator::Allocator;

    fn word(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }
    fn nodes(bytes: &[u8]) -> u32 {
        word(bytes, 0)
    }
    fn root(bytes: &[u8]) -> u32 {
        word(bytes, bytes.len() - 8)
    }
    fn payloads(bytes: &[u8]) -> usize {
        (8 + nodes(bytes) as usize * 9 + 7) & !7
    }
    fn extra_info(bytes: &[u8]) -> usize {
        payloads(bytes) + nodes(bytes) as usize * 16
    }
    fn strings_info(bytes: &[u8]) -> usize {
        extra_info(bytes) + 4 + word(bytes, extra_info(bytes)) as usize * 4
    }
    fn positions_info(bytes: &[u8]) -> usize {
        let strings = strings_info(bytes);
        let comments = (strings + 8 + word(bytes, strings + 4) as usize + 3) & !3;
        let diagnostics = comments + 4 + word(bytes, comments) as usize * 8;
        (diagnostics + 8 + word(bytes, diagnostics + 4) as usize + 3) & !3
    }
    fn program<'a>(allocator: &'a Allocator, source: &'a str) -> Ast<'a> {
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
    fn serialized(allocator: &Allocator, source: &str) -> Vec<u8> {
        serialize(
            &program(allocator, source),
            &[],
            source.encode_utf16().count(),
        )
        .unwrap()
    }
    fn delta(base: &[u8], new_nodes: u32) -> Vec<u8> {
        let capacity = new_nodes as usize + 1;
        let extra = 8 + capacity * 32 + 8;
        let strings = extra + 16 + 8;
        let metadata = strings + 8;
        let mut bytes = vec![0; metadata + 28];
        put(&mut bytes, 0, new_nodes);
        put(&mut bytes, 4, capacity as u32);
        put(&mut bytes, extra - 4, 4);
        put(&mut bytes, strings - 4, 8);
        put(&mut bytes, metadata, root(base));
        put(&mut bytes, metadata + 4, word(base, base.len() - 4));
        put(&mut bytes, metadata + 8, new_nodes);
        put(&mut bytes, metadata + 12, nodes(base));
        put(&mut bytes, metadata + 16, word(base, extra_info(base)));
        put(
            &mut bytes,
            metadata + 20,
            word(base, strings_info(base) + 4),
        );
        put(&mut bytes, metadata + 24, base.len() as u32);
        for index in 0..new_nodes as usize {
            let row = 8 + index * 32;
            bytes[row..row + 8].copy_from_slice(&base[8..16]);
            bytes[row + 8] = base[8 + nodes(base) as usize * 8];
            let payload = payloads(base);
            bytes[row + 16..row + 32].copy_from_slice(&base[payload..payload + 16]);
        }
        bytes
    }
    fn decoded<'a>(
        base: &'a [u8],
        delta: &'a [u8],
        allocator: &'a Allocator,
    ) -> Result<BorrowedAst<'a>, DeserializeError> {
        deserialize(base, delta, allocator)
    }
    fn identifier_delta(allocator: &Allocator, value: &[u8], utf8: bool) -> (Vec<u8>, Vec<u8>) {
        let mut base = serialized(allocator, ";");
        let new_id = nodes(&base);
        let pool = word(&base, strings_info(&base) + 4);
        let kind = 8 + nodes(&base) as usize * 8;
        base[kind] = crate::NodeKind::IdentifierReference as u8;
        let payload = payloads(&base);
        put(&mut base, payload, pool);
        put(&mut base, payload + 4, pool + value.len() as u32);
        let extra = extra_info(&base) + 4;
        put(&mut base, extra, new_id);
        let mut extension = delta(&base, 1);
        let capacity = word(&extension, 4);
        let strings_info = 8 + capacity as usize * 32 + 8 + 16;
        let strings = strings_info + 8;
        extension[16] = crate::NodeKind::ExpressionStatement as u8;
        put(&mut extension, 24, 0);
        put(&mut extension, strings_info, value.len() as u32);
        extension[strings..strings + value.len()].copy_from_slice(value);
        if utf8 {
            put(&mut extension, 4, capacity | 0x8000_0000);
        }
        (base, extension)
    }
    fn identifier_name(owner: &BorrowedAst<'_>) -> String {
        let ast = unsafe { owner.ast() };
        let statement = ast
            .root_program()
            .body(ast)
            .get_node(ast, 0)
            .unwrap()
            .as_expression_statement(ast)
            .unwrap();
        let identifier = statement
            .expression(ast)
            .as_identifier_reference(ast)
            .unwrap();
        ast.get_utf8(identifier.name(ast)).to_owned()
    }
    fn reject_delta(base: &[u8], delta: &[u8]) {
        let allocator = Allocator::new();
        assert!(decoded(base, delta, &allocator).is_err());
        assert!(unsafe { deserialize_unchecked(base, delta, &allocator) }.is_err());
    }
    fn position_rows(bytes: &[u8]) -> Vec<[u32; 4]> {
        let info = positions_info(bytes);
        (0..word(bytes, info) as usize)
            .map(|index| {
                std::array::from_fn(|field| word(bytes, info + 8 + index * 16 + field * 4))
            })
            .collect()
    }

    #[test]
    fn source_positions() {
        let allocator = Allocator::new();
        let ascii = serialized(&allocator, "abc");
        assert!(position_rows(&ascii).is_empty());
        assert_eq!(word(&ascii, positions_info(&ascii) + 4), 0);

        let mixed = serialized(&allocator, "éé中中😀😀xé中😀");
        assert_eq!(
            position_rows(&mixed),
            vec![
                [0, 0, 2, 4],
                [4, 2, 3, 10],
                [10, 6, 4, 18],
                [19, 10, 2, 21],
                [21, 11, 3, 24],
                [24, 13, 4, 28]
            ]
        );
        assert_eq!(word(&mixed, positions_info(&mixed) + 4), 15);

        let spaced = serialized(&allocator, "éxé");
        assert_eq!(position_rows(&spaced), vec![[0, 0, 2, 2], [3, 1, 2, 5]]);
    }
    #[test]
    fn source_position_validation() {
        let allocator = Allocator::new();
        let mut count = serialized(&allocator, "é");
        let info = positions_info(&count);
        put(&mut count, info, 2);
        assert!(decoded(&count, &[], &allocator).is_err());

        let mut width = serialized(&allocator, "é");
        let info = positions_info(&width);
        put(&mut width, info + 16, 1);
        assert!(decoded(&width, &[], &allocator).is_err());

        let mut excess = serialized(&allocator, "é");
        let info = positions_info(&excess);
        put(&mut excess, info + 12, 1);
        assert!(decoded(&excess, &[], &allocator).is_err());

        let mut stop = serialized(&allocator, "é");
        let info = positions_info(&stop);
        put(&mut stop, info + 20, 3);
        assert!(decoded(&stop, &[], &allocator).is_err());
    }
    #[test]
    fn utf8_spans() {
        let allocator = Allocator::new();
        let ast = program(&allocator, "中😀");
        let diagnostic = Diagnostic::error("message", Span::new(3, 7));
        let bytes = serialize(&ast, &[diagnostic], "中😀".encode_utf16().count()).unwrap();
        assert_eq!(word(&bytes, 12), 7);
        assert_eq!(word(&bytes, 8 + root(&bytes) as usize * 8 + 4), 7);
        let source = strings_info(&bytes) + 8;
        assert_eq!(&bytes[source..source + 7], "中😀".as_bytes());
    }
    #[test]
    fn structural_trees() {
        let allocator = Allocator::new();
        let mut ast = unsafe { Ast::new("", &allocator) };
        let _unused = ast.stmt_empty_statement(Span::new(0, 0));
        let statement = ast.stmt_empty_statement(Span::new(0, 0));
        let body = ast.add_sub_range([statement, statement]);
        let root = ast.program(Span::new(0, 0), SubRange::empty(), body, None);
        ast.set_root_program(root);
        let bytes = serialize(&ast, &[], 0).unwrap();
        assert!(decoded(&bytes, &[], &allocator).is_err());

        let body = ast.add_sub_range([statement]);
        root.set_body(&mut ast, body);
        let _orphan = ast.stmt_block_statement(Span::new(0, 0), body);
        let bytes = serialize(&ast, &[], 0).unwrap();
        let owner = decoded(&bytes, &[], &allocator).unwrap();
        let decoded = unsafe { owner.ast() };
        assert_eq!(decoded.node_count(), ast.node_count());
        let body = decoded.root_program().body(decoded);
        assert_eq!(
            body.get_node(decoded, 0).unwrap().node_id(),
            statement.node_id()
        );
        assert_eq!(body.len(), 1);
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
        let bytes = serialize(&ast, &[], 0).unwrap();
        assert!(decoded(&bytes, &[], &allocator).is_err());
    }
    #[test]
    fn decoded_buffers() {
        let allocator = Allocator::new();
        let bytes = serialized(&allocator, "中😀");
        let owner = decoded(&bytes, &[], &allocator).unwrap();
        let ast = unsafe { owner.ast() };
        assert_eq!(ast.source(), "中😀");
        assert_eq!(
            serialize(ast, &[], ast.source().encode_utf16().count()).unwrap(),
            bytes
        );
        #[cfg(target_endian = "little")]
        assert_eq!(ast.nodes.data.as_ptr().cast::<u8>().cast_const(), unsafe {
            bytes.as_ptr().add(payloads(&bytes))
        });
        let trusted = unsafe { deserialize_unchecked(&bytes, &[], &allocator) }.unwrap();
        assert_eq!(
            trusted.root_program().node_id(),
            ast.root_program().node_id()
        );

        let mut padded = vec![0];
        padded.extend_from_slice(&bytes);
        let owner = decoded(&padded[1..], &[], &allocator).unwrap();
        assert_eq!(unsafe { owner.ast() }.source(), "中😀");
    }
    #[test]
    fn metadata_edits() {
        let allocator = Allocator::new();
        let base = serialized(&allocator, "");
        let mut extension = delta(&base, 0);
        let metadata = extension.len() - 28;
        put(&mut extension, metadata + 4, 1);
        let owner = decoded(&base, &extension, &allocator).unwrap();
        assert_eq!(
            unsafe { owner.ast() }.source_type(),
            crate::SourceType::Script
        );
        put(&mut extension, metadata + 12, nodes(&base) + 1);
        reject_delta(&base, &extension);
    }
    #[test]
    fn delta_merging() {
        let allocator = Allocator::new();
        let mut base = serialized(&allocator, ";");
        let new_id = nodes(&base);
        let extra_base = word(&base, extra_info(&base));
        let kind = 8 + nodes(&base) as usize * 8;
        base[kind] = crate::NodeKind::DebuggerStatement as u8;
        put(&mut base, 8, 1);
        put(&mut base, 12, 1);
        let root_payload = payloads(&base) + root(&base) as usize * 16;
        put(&mut base, root_payload + 12, extra_base + 2);
        let mut extension = delta(&base, 1);
        extension[16] = crate::NodeKind::EmptyStatement as u8;
        let extra_info = 8 + word(&extension, 4) as usize * 32;
        let extra = extra_info + 8;
        put(&mut extension, extra_info, 4);
        put(&mut extension, extra, 0);
        put(&mut extension, extra + 4, new_id);
        put(&mut extension, extra + 8, extra_base);
        put(&mut extension, extra + 12, extra_base + 2);
        let owner = decoded(&base, &extension, &allocator).unwrap();
        let ast = unsafe { owner.ast() };
        let body = ast.root_program().body(ast);
        let changed = body.get_node(ast, 0).unwrap();
        assert_eq!(changed.node_id().index(), 0);
        assert!(changed.is_debugger_statement(ast));
        assert_eq!(ast.span(changed.node_id()), Span::new(1, 1));
        assert_eq!(
            body.get_node(ast, 1).unwrap().node_id().index(),
            new_id as usize
        );
        let trusted = unsafe { deserialize_unchecked(&base, &extension, &allocator) }.unwrap();
        assert_eq!(
            serialize(
                unsafe { trusted.ast() },
                &[],
                trusted.source().encode_utf16().count(),
            )
            .unwrap(),
            serialize(ast, &[], ast.source().encode_utf16().count()).unwrap()
        );
    }
    #[test]
    fn delta_rows() {
        let allocator = Allocator::new();
        let base = serialized(&allocator, "");
        let mut rows = delta(&base, 1);
        let metadata = rows.len() - 28;
        put(&mut rows, metadata + 8, 0);
        reject_delta(&base, &rows);

        let mut root = delta(&base, 1);
        let metadata = root.len() - 28;
        put(&mut root, metadata, nodes(&base) + 1);
        reject_delta(&base, &root);
    }
    #[test]
    fn delta_layout() {
        let allocator = Allocator::new();
        let base = serialized(&allocator, "");
        let mut rows = delta(&base, 0);
        put(&mut rows, 0, 3);
        reject_delta(&base, &rows);

        let mut extra = delta(&base, 0);
        let offset = 8 + word(&extra, 4) as usize * 32;
        put(&mut extra, offset, 5);
        reject_delta(&base, &extra);

        let mut strings = delta(&base, 0);
        let offset = 8 + word(&strings, 4) as usize * 32 + 8 + 16;
        put(&mut strings, offset, 9);
        reject_delta(&base, &strings);

        let mut truncated = delta(&base, 0);
        truncated.pop();
        reject_delta(&base, &truncated);
    }
    #[test]
    fn delta_references() {
        let allocator = Allocator::new();
        let mut typed = serialized(&allocator, "");
        let kind = 8 + nodes(&typed) as usize * 8;
        typed[kind] = crate::NodeKind::IdentifierReference as u8;
        assert!(decoded(&typed, &delta(&typed, 0), &allocator).is_err());

        let mut cyclic = serialized(&allocator, "");
        let kind = 8 + nodes(&cyclic) as usize * 8;
        cyclic[kind] = crate::NodeKind::BlockStatement as u8;
        let payload = payloads(&cyclic);
        put(&mut cyclic, payload, 0);
        put(&mut cyclic, payload + 4, 1);
        assert!(decoded(&cyclic, &delta(&cyclic, 0), &allocator).is_err());
    }
    #[test]
    fn delta_strings() {
        let allocator = Allocator::new();
        let mut base = serialized(&allocator, ";");
        let new_id = nodes(&base);
        let pool = word(&base, strings_info(&base) + 4);
        let kind = 8 + nodes(&base) as usize * 8;
        base[kind] = crate::NodeKind::IdentifierReference as u8;
        let payload = payloads(&base);
        put(&mut base, payload, pool);
        put(&mut base, payload + 4, pool + 2);
        let extra = extra_info(&base) + 4;
        put(&mut base, extra, new_id);
        let mut extension = delta(&base, 1);
        let strings_info = 8 + word(&extension, 4) as usize * 32 + 8 + 16;
        let strings = strings_info + 8;
        extension[16] = crate::NodeKind::ExpressionStatement as u8;
        put(&mut extension, 24, 0);
        put(&mut extension, strings_info, 2);
        extension[strings..strings + 2].copy_from_slice(&0x96eau16.to_le_bytes());
        let owner = decoded(&base, &extension, &allocator).unwrap();
        let ast = unsafe { owner.ast() };
        let statement = ast
            .root_program()
            .body(ast)
            .get_node(ast, 0)
            .unwrap()
            .as_expression_statement(ast)
            .unwrap();
        let identifier = statement
            .expression(ast)
            .as_identifier_reference(ast)
            .unwrap();
        assert_eq!(ast.get_utf8(identifier.name(ast)), "雪");
        let trusted = unsafe { deserialize_unchecked(&base, &extension, &allocator) }.unwrap();
        assert_eq!(
            serialize(
                unsafe { trusted.ast() },
                &[],
                trusted.source().encode_utf16().count(),
            )
            .unwrap(),
            serialize(ast, &[], ast.source().encode_utf16().count()).unwrap()
        );

        extension[strings..strings + 2].copy_from_slice(&0xd800u16.to_le_bytes());
        assert!(decoded(&base, &extension, &allocator).is_err());

        extension[strings..strings + 2].copy_from_slice(&0x96eau16.to_le_bytes());
        put(&mut base, payload + 4, pool + 1);
        assert!(decoded(&base, &extension, &allocator).is_err());
    }
    #[test]
    fn delta_utf8_strings() {
        let allocator = Allocator::new();
        let (ascii_base, ascii_delta) = identifier_delta(&allocator, b"name", true);
        let ascii = decoded(&ascii_base, &ascii_delta, &allocator).unwrap();
        assert_eq!(identifier_name(&ascii), "name");

        let (mut unicode_base, unicode_delta) = identifier_delta(&allocator, "雪".as_bytes(), true);
        let unicode = decoded(&unicode_base, &unicode_delta, &allocator).unwrap();
        assert_eq!(identifier_name(&unicode), "雪");
        let trusted =
            unsafe { deserialize_unchecked(&unicode_base, &unicode_delta, &allocator) }.unwrap();
        assert_eq!(
            serialize(
                unsafe { trusted.ast() },
                &[],
                trusted.source().encode_utf16().count(),
            )
            .unwrap(),
            serialize(
                unsafe { unicode.ast() },
                &[],
                unicode.source().encode_utf16().count(),
            )
            .unwrap()
        );

        let pool = word(&unicode_base, strings_info(&unicode_base) + 4);
        let payload = payloads(&unicode_base);
        put(&mut unicode_base, payload + 4, pool + 2);
        assert!(decoded(&unicode_base, &unicode_delta, &allocator).is_err());

        let (invalid_base, invalid_delta) = identifier_delta(&allocator, &[0xed, 0xa0, 0x80], true);
        assert!(decoded(&invalid_base, &invalid_delta, &allocator).is_err());
    }
    #[test]
    fn string_only_extension() {
        let allocator = Allocator::new();
        let mut ast = unsafe { Ast::new("x", &allocator) };
        let name = unsafe { AstUtf8::from_source_range("x", 0, 1) };
        let value = ast.expr_identifier_reference(Span::new(0, 1), name);
        let statement = ast.stmt_expression_statement(Span::new(0, 1), value);
        let body = ast.add_sub_range([statement]);
        let root = ast.program(Span::new(0, 1), SubRange::empty(), body, None);
        ast.set_root_program(root);
        let mut base = serialize(&ast, &[], 1).unwrap();
        let payload = payloads(&base);
        put(&mut base, payload, 1);
        put(&mut base, payload + 4, 2);

        let mut extension = delta(&base, 0);
        let strings_info = 8 + word(&extension, 4) as usize * 32 + 8 + 16;
        let strings = strings_info + 8;
        put(&mut extension, strings_info, 1);
        let capacity = word(&extension, 4);
        put(&mut extension, 4, capacity | 0x8000_0000);
        extension[strings] = b'a';

        let owner = decoded(&base, &extension, &allocator).unwrap();
        let ast = unsafe { owner.ast() };
        let statement = ast
            .root_program()
            .body(ast)
            .get_node(ast, 0)
            .unwrap()
            .as_expression_statement(ast)
            .unwrap();
        let identifier = statement
            .expression(ast)
            .as_identifier_reference(ast)
            .unwrap();
        assert_eq!(ast.get_utf8(identifier.name(ast)), "a");
        assert_eq!(ast.strings.extra_bytes(), b"a");
        #[cfg(target_endian = "little")]
        assert_eq!(ast.nodes.data.as_ptr().cast::<u8>().cast_const(), unsafe {
            base.as_ptr().add(payloads(&base))
        });

        let trusted = unsafe { deserialize_unchecked(&base, &extension, &allocator) }.unwrap();
        assert_eq!(
            trusted.root_program().node_id(),
            ast.root_program().node_id()
        );
    }
    #[test]
    fn invalid_fields() {
        let allocator = Allocator::new();
        let mut ast = unsafe { Ast::new("é", &allocator) };
        let name = unsafe { AstUtf8::from_source_range("é", 0, 2) };
        let value = ast.expr_identifier_reference(Span::new(0, 2), name);
        let statement = ast.stmt_expression_statement(Span::new(0, 2), value);
        let body = ast.add_sub_range([statement]);
        let root = ast.program(Span::new(0, 2), SubRange::empty(), body, None);
        ast.set_root_program(root);
        let mut bytes = serialize(&ast, &[], 1).unwrap();
        let payload = payloads(&bytes);
        put(&mut bytes, payload, 1);
        assert!(decoded(&bytes, &[], &allocator).is_err());
    }
}
