use std::collections::{HashMap, HashSet};

use swc_next_allocator::{Allocator, wtf8::Wtf8};

use super::{
    DeserializeError as Error, HEADER_SIZE, MAGIC, NONE, PAYLOAD_SIZE, SPAN_SIZE, VERSION,
    schema::{self, Field, FieldValidator},
};
use crate::ffi::ast::BorrowedAst;
use crate::{Ast, ExtraData, Lang, NodeId, NodeKind, NodePayload, SourceType, Span};
use std::{mem::align_of, ptr::NonNull, slice};
use swc_next_allocator::vec::{ArenaIndexVec, ArenaVec};

const EXTENSION_HEADER: usize = 64;
const EXTENSION_MAGIC: u32 = u32::from_le_bytes(*b"SWE2");
const DELTA_HEADER: usize = 96;
const DELTA_MAGIC: u32 = u32::from_le_bytes(*b"SWE3");
const DELTA_VERSION: u32 = 3;
const ROW_SIZE: usize = 32;

fn invalid(message: &'static str) -> Error {
    Error(message)
}
fn word(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let end = offset
        .checked_add(4)
        .ok_or(invalid("buffer offset overflow"))?;
    let bytes = bytes
        .get(offset..end)
        .ok_or(invalid("truncated AST buffer"))?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}
fn end(start: usize, count: usize, width: usize) -> Result<usize, Error> {
    start
        .checked_add(
            count
                .checked_mul(width)
                .ok_or(invalid("section length overflow"))?,
        )
        .ok_or(invalid("section offset overflow"))
}
fn flags(value: u32) -> Result<(), Error> {
    if value > 31 || (value >> 2) > 4 {
        return Err(invalid("invalid AST language flags"));
    }
    Ok(())
}

struct Parts<'a> {
    bytes: &'a [u8],
    nodes: usize,
    kinds: usize,
    payloads: usize,
    count: usize,
    extra: usize,
    extra_count: usize,
    strings: usize,
    string_len: usize,
    source_len: usize,
    root: usize,
    flags: u32,
    // Column tables use native indices. Delta rows use this sparse stable-ID
    // lookup, and the three node offsets point into the first 32-byte row.
    rows: Option<HashMap<usize, usize>>,
}

impl<'a> Parts<'a> {
    fn base(bytes: &'a [u8]) -> Result<Self, Error> {
        if bytes.len() < HEADER_SIZE
            || word(bytes, 0)? != MAGIC
            || word(bytes, 4)? != VERSION
            || word(bytes, 8)? != schema::SCHEMA_ID
        {
            return Err(invalid("AST format or schema mismatch"));
        }
        let value = Self {
            bytes,
            nodes: word(bytes, 28)? as usize,
            kinds: end(
                word(bytes, 28)? as usize,
                word(bytes, 20)? as usize,
                SPAN_SIZE,
            )?,
            payloads: end(
                end(
                    end(
                        word(bytes, 28)? as usize,
                        word(bytes, 20)? as usize,
                        SPAN_SIZE,
                    )?,
                    word(bytes, 20)? as usize,
                    1,
                )?,
                7,
                1,
            )? & !7,
            count: word(bytes, 20)? as usize,
            extra: word(bytes, 36)? as usize,
            extra_count: word(bytes, 32)? as usize,
            strings: word(bytes, 40)? as usize,
            string_len: word(bytes, 48)? as usize,
            source_len: word(bytes, 44)? as usize,
            root: word(bytes, 24)? as usize,
            flags: word(bytes, 12)?,
            rows: None,
        };
        flags(value.flags)?;
        let comments = end(end(value.strings, value.string_len, 1)?, 3, 1)? & !3;
        let diagnostics = end(comments, word(bytes, 56)? as usize, 8)?;
        if word(bytes, 16)? as usize != bytes.len()
            || value.nodes != HEADER_SIZE
            || value.extra != end(value.payloads, value.count, PAYLOAD_SIZE)?
            || value.strings != end(value.extra, value.extra_count, 8)?
            || value.source_len > value.string_len
            || value.count == 0
            || value.root >= value.count
            || word(bytes, 52)? as usize != comments
            || word(bytes, 60)? as usize != diagnostics
            || end(diagnostics, word(bytes, 68)? as usize, 1)? != bytes.len()
            || (word(bytes, 64)? as u64) * 18 > word(bytes, 68)? as u64
        {
            return Err(invalid("invalid AST section layout"));
        }
        Ok(value)
    }
    fn extension(bytes: &'a [u8], base: &Self) -> Result<Self, Error> {
        if word(bytes, 0)? == DELTA_MAGIC {
            return Self::delta(bytes, base);
        }
        if bytes.len() < EXTENSION_HEADER
            || word(bytes, 0)? != EXTENSION_MAGIC
            || word(bytes, 4)? != VERSION
            || word(bytes, 8)? != schema::SCHEMA_ID
        {
            return Err(invalid("extension format or schema mismatch"));
        }
        if word(bytes, 16)? as usize != base.count
            || word(bytes, 20)? as usize != base.extra_count
            || word(bytes, 24)? as usize != base.string_len
            || word(bytes, 28)? as usize != base.bytes.len()
        {
            return Err(invalid("extension does not match its base buffer"));
        }
        let value = Self {
            bytes,
            nodes: word(bytes, 52)? as usize,
            kinds: end(
                word(bytes, 52)? as usize,
                word(bytes, 32)? as usize,
                SPAN_SIZE,
            )?,
            payloads: end(
                end(
                    end(
                        word(bytes, 52)? as usize,
                        word(bytes, 32)? as usize,
                        SPAN_SIZE,
                    )?,
                    word(bytes, 32)? as usize,
                    1,
                )?,
                7,
                1,
            )? & !7,
            count: word(bytes, 32)? as usize,
            extra: word(bytes, 56)? as usize,
            extra_count: word(bytes, 36)? as usize,
            strings: word(bytes, 60)? as usize,
            string_len: word(bytes, 40)? as usize,
            source_len: 0,
            root: word(bytes, 44)? as usize,
            flags: word(bytes, 48)?,
            rows: None,
        };
        flags(value.flags)?;
        if word(bytes, 12)? as usize != bytes.len()
            || value.nodes != EXTENSION_HEADER
            || value.extra != end(value.payloads, value.count, PAYLOAD_SIZE)?
            || value.strings != end(value.extra, value.extra_count, 8)?
            || end(value.strings, value.string_len, 1)? != bytes.len()
        {
            return Err(invalid("invalid extension section layout"));
        }
        Ok(value)
    }
    fn delta(bytes: &'a [u8], base: &Self) -> Result<Self, Error> {
        if bytes.len() < DELTA_HEADER
            || word(bytes, 4)? != DELTA_VERSION
            || word(bytes, 8)? != schema::SCHEMA_ID
        {
            return Err(invalid("delta format or schema mismatch"));
        }
        if word(bytes, 16)? as usize != base.count
            || word(bytes, 20)? as usize != base.extra_count
            || word(bytes, 24)? as usize != base.string_len
            || word(bytes, 28)? as usize != base.bytes.len()
        {
            return Err(invalid("delta does not match its base buffer"));
        }
        let count = word(bytes, 32)? as usize;
        let row_count = word(bytes, 36)? as usize;
        let row_capacity = word(bytes, 60)? as usize;
        let extra_count = word(bytes, 40)? as usize;
        let extra_capacity = word(bytes, 68)? as usize;
        let string_len = word(bytes, 44)? as usize;
        let string_capacity = word(bytes, 76)? as usize;
        let nodes = word(bytes, 56)? as usize;
        let extra = word(bytes, 64)? as usize;
        let strings = word(bytes, 72)? as usize;
        let total_count = end(base.count, count, 1)?;
        if word(bytes, 12)? as usize != bytes.len()
            || nodes != DELTA_HEADER
            || row_count > row_capacity
            || extra_count > extra_capacity
            || string_len > string_capacity
            || count > row_count
            || row_count > total_count
            || total_count >= NONE as usize
            || extra != end(nodes, row_capacity, ROW_SIZE)?
            || strings != end(extra, extra_capacity, 8)?
            || end(strings, string_capacity, 1)? != bytes.len()
            || bytes[80..DELTA_HEADER].iter().any(|&byte| byte != 0)
        {
            return Err(invalid("invalid delta section layout"));
        }
        let value_flags = word(bytes, 52)?;
        flags(value_flags)?;
        let mut rows = HashMap::with_capacity(row_count);
        let mut new_count = 0;
        for index in 0..row_count {
            let offset = nodes + index * ROW_SIZE;
            let target = word(bytes, offset + 12)? as usize;
            if bytes[offset + 9] != 1
                || bytes[offset + 10] != 0
                || bytes[offset + 11] != 0
            {
                return Err(invalid("invalid delta row status"));
            }
            if target >= total_count || rows.insert(target, index).is_some() {
                return Err(invalid("invalid or duplicate delta target"));
            }
            new_count += usize::from(target >= base.count);
        }
        // Unique targets in the final ID range cover every new ID exactly when
        // their count matches the declared number of appended nodes.
        if new_count != count {
            return Err(invalid("missing new delta node"));
        }
        Ok(Self {
            bytes,
            nodes,
            kinds: nodes + 8,
            payloads: nodes + 16,
            count,
            extra,
            extra_count,
            strings,
            string_len,
            source_len: 0,
            root: word(bytes, 48)? as usize,
            flags: value_flags,
            rows: Some(rows),
        })
    }
    #[cfg(target_endian = "big")]
    fn span(&self, id: usize) -> Result<Span, Error> {
        let stride = if self.rows.is_some() { ROW_SIZE } else { SPAN_SIZE };
        let offset = self.nodes + id * stride;
        Ok(Span::new(
            word(self.bytes, offset)?,
            word(self.bytes, offset + 4)?,
        ))
    }
    fn kind(&self, id: usize) -> u8 {
        self.bytes[self.kinds + id * if self.rows.is_some() { ROW_SIZE } else { 1 }]
    }
    fn payload(&self, id: usize) -> &[u8] {
        let stride = if self.rows.is_some() { ROW_SIZE } else { PAYLOAD_SIZE };
        let offset = self.payloads + id * stride;
        &self.bytes[offset..offset + PAYLOAD_SIZE]
    }
}

struct Reader<'a> {
    base: Parts<'a>,
    extension: Option<Parts<'a>>,
    source: &'a str,
    count: usize,
    extra_count: usize,
    string_len: usize,
    root: usize,
    flags: u32,
}

impl<'a> Reader<'a> {
    unsafe fn new(base: &'a [u8], extension: &'a [u8], trusted: bool) -> Result<Self, Error> {
        let base = Parts::base(base)?;
        let extension = if extension.is_empty() {
            None
        } else {
            Some(Parts::extension(extension, &base)?)
        };
        let count = end(base.count, extension.as_ref().map_or(0, |v| v.count), 1)?;
        let extra_count = end(
            base.extra_count,
            extension.as_ref().map_or(0, |v| v.extra_count),
            1,
        )?;
        let string_len = end(
            base.string_len,
            extension.as_ref().map_or(0, |v| v.string_len),
            1,
        )?;
        if count >= NONE as usize || extra_count >= NONE as usize || string_len >= NONE as usize {
            return Err(invalid("combined AST exceeds 32-bit limits"));
        }
        let root = extension.as_ref().map_or(base.root, |v| v.root);
        let flags = extension.as_ref().map_or(base.flags, |v| v.flags);
        if root >= count {
            return Err(invalid("AST root out of bounds"));
        }
        let source_bytes = &base.bytes[base.strings..base.strings + base.source_len];
        let source = if trusted {
            // SAFETY: The trusted-transfer caller guarantees valid original UTF-8.
            unsafe { std::str::from_utf8_unchecked(source_bytes) }
        } else {
            std::str::from_utf8(source_bytes).map_err(|_| invalid("invalid UTF-8 source"))?
        };
        Ok(Self {
            base,
            extension,
            source,
            count,
            extra_count,
            string_len,
            root,
            flags,
        })
    }
    fn node(&self, id: usize) -> Result<(&Parts<'a>, usize), Error> {
        if id >= self.count {
            return Err(invalid("node reference out of bounds"));
        }
        if let Some(extension) = &self.extension
            && let Some(rows) = &extension.rows
            && let Some(&index) = rows.get(&id)
        {
            return Ok((extension, index));
        }
        Ok(if id < self.base.count {
            (&self.base, id)
        } else {
            (self.extension.as_ref().unwrap(), id - self.base.count)
        })
    }
    fn kind(&self, id: usize) -> Result<NodeKind, Error> {
        let (part, index) = self.node(id)?;
        let kind = part.kind(index) as u32;
        if kind >= schema::NODE_KIND_COUNT as u32 {
            return Err(invalid("invalid node kind"));
        }
        // SAFETY: The generated kind table matches NodeKind's dense u8 discriminants.
        Ok(unsafe { std::mem::transmute::<u8, NodeKind>(kind as u8) })
    }
    fn extra(&self, id: usize) -> Result<&[u8], Error> {
        if id >= self.extra_count {
            return Err(invalid("extra-data reference out of bounds"));
        }
        let (part, index) = if id < self.base.extra_count {
            (&self.base, id)
        } else {
            (self.extension.as_ref().unwrap(), id - self.base.extra_count)
        };
        Ok(&part.bytes[part.extra + index * 8..part.extra + index * 8 + 8])
    }
    fn string(&self, start: usize, end: usize) -> Result<&[u8], Error> {
        if start > end || end > self.string_len {
            return Err(invalid("string reference out of bounds"));
        }
        if start == end {
            return Ok(&[]);
        }
        if start < self.base.source_len && end > self.base.source_len {
            return Err(invalid("string crosses source boundary"));
        }
        if start < self.base.string_len {
            if end > self.base.string_len {
                return Err(invalid("string crosses buffer boundary"));
            }
            return Ok(&self.base.bytes[self.base.strings + start..self.base.strings + end]);
        }
        let part = self
            .extension
            .as_ref()
            .ok_or(invalid("missing extension strings"))?;
        Ok(&part.bytes[part.strings + start - self.base.string_len
            ..part.strings + end - self.base.string_len])
    }
    fn validate(&self) -> Result<(), Error> {
        if self.kind(self.root)? != NodeKind::Program {
            return Err(invalid("AST root is not a Program"));
        }
        let mut edges = Vec::new();
        let mut ranges = Vec::with_capacity(self.count);
        let mut strings = HashSet::new();
        let budget = end(end(0, self.count, 16)?, self.extra_count, 1)?;
        for id in 0..self.count {
            let start = edges.len();
            let (part, index) = self.node(id)?;
            let mut validator = Validator {
                reader: self,
                payload: part.payload(index),
                extra: 0,
                edges: &mut edges,
                budget,
                strings: &mut strings,
            };
            schema::validate_fields(self.kind(id)?, &mut validator)?;
            ranges.push((start, edges.len()));
        }
        let mut colors = vec![0u8; self.count];
        let mut stack = Vec::new();
        for root in 0..self.count {
            if colors[root] != 0 {
                continue;
            }
            stack.push((root, false));
            while let Some((id, leave)) = stack.pop() {
                if leave {
                    colors[id] = 2;
                    continue;
                }
                if colors[id] == 1 {
                    return Err(invalid("cyclic AST references"));
                }
                if colors[id] == 2 {
                    continue;
                }
                colors[id] = 1;
                stack.push((id, true));
                let (start, end) = ranges[id];
                for child in edges[start..end].iter().rev() {
                    stack.push((*child as usize, false));
                }
            }
        }
        Ok(())
    }
}

struct Validator<'a, 'b> {
    reader: &'b Reader<'a>,
    payload: &'b [u8],
    extra: usize,
    edges: &'b mut Vec<u32>,
    budget: usize,
    strings: &'b mut HashSet<(u32, u32, bool)>,
}

impl Validator<'_, '_> {
    fn bytes(&self, field: Field) -> Result<&[u8], Error> {
        match field {
            Field::Payload(offset) => self
                .payload
                .get(offset..)
                .ok_or(invalid("invalid payload field")),
            Field::Extra(offset) => self.reader.extra(end(self.extra, offset, 1)?),
        }
    }
    fn child(&mut self, id: u32, kinds: &[NodeKind], optional: bool) -> Result<(), Error> {
        if id == NONE && optional {
            return Ok(());
        }
        let kind = self.reader.kind(id as usize)?;
        if !kinds.contains(&kind) {
            return Err(invalid("node kind does not match its field"));
        }
        if self.edges.len() >= self.budget {
            return Err(invalid("AST contains excessive shared ranges"));
        }
        self.edges.push(id);
        Ok(())
    }
}

impl FieldValidator for Validator<'_, '_> {
    fn extra(&mut self, pointer: usize, count: usize) -> Result<(), Error> {
        self.extra = word(self.payload, pointer)? as usize;
        if end(self.extra, count, 1)? > self.reader.extra_count {
            return Err(invalid("node extra-data block out of bounds"));
        }
        Ok(())
    }
    fn node(&mut self, field: Field, kinds: &[NodeKind], optional: bool) -> Result<(), Error> {
        self.child(word(self.bytes(field)?, 0)?, kinds, optional)
    }
    fn list(&mut self, field: Field, kinds: &[NodeKind], optional: bool) -> Result<(), Error> {
        let bytes = self.bytes(field)?;
        let start = word(bytes, 0)? as usize;
        let end = word(bytes, 4)? as usize;
        if start > end || end > self.reader.extra_count {
            return Err(invalid("AST list out of bounds"));
        }
        for index in start..end {
            self.child(word(self.reader.extra(index)?, 0)?, kinds, optional)?;
        }
        Ok(())
    }
    fn boolean(&mut self, field: Field) -> Result<(), Error> {
        if self.bytes(field)?[0] > 1 {
            return Err(invalid("invalid boolean field"));
        }
        Ok(())
    }
    fn enumeration(&mut self, field: Field, count: usize, optional: bool) -> Result<(), Error> {
        let value = self.bytes(field)?[0] as usize;
        if value >= count && !(optional && value == 255) {
            return Err(invalid("invalid enum field"));
        }
        Ok(())
    }
    fn string(&mut self, field: Field, wtf8: bool, optional: bool) -> Result<(), Error> {
        let bytes = self.bytes(field)?;
        let start = word(bytes, 0)?;
        let end = word(bytes, 4)?;
        if optional && start == NONE && end == 0 {
            return Ok(());
        }
        let value = self.reader.string(start as usize, end as usize)?;
        if (end as usize) <= self.reader.base.source_len {
            self.reader
                .source
                .get(start as usize..end as usize)
                .ok_or(invalid("string splits a UTF-8 character"))?;
            // Reader::new validated the complete source. These spans only need
            // character-boundary checks, not hashing or repeated UTF-8 scans.
            return Ok(());
        }
        if !self.strings.insert((start, end, wtf8)) {
            return Ok(());
        }
        if wtf8 {
            Wtf8::from_bytes(value).map_err(|_| invalid("invalid WTF-8 string"))?;
        } else {
            std::str::from_utf8(value).map_err(|_| invalid("invalid UTF-8 string"))?;
        }
        Ok(())
    }
    fn index(&mut self, field: Field, nonzero: bool) -> Result<(), Error> {
        let value = word(self.bytes(field)?, 0)?;
        if (nonzero && value == 0) || (!nonzero && value == NONE) {
            return Err(invalid("invalid semantic index"));
        }
        Ok(())
    }
}

/// Validate a transfer and expose read-only native tables.
/// Aligned little-endian base-only transfers borrow their tables and strings.
pub fn deserialize<'a>(
    base: &'a [u8],
    extension: &'a [u8],
    allocator: &'a Allocator,
) -> Result<BorrowedAst<'a>, Error> {
    // SAFETY: trusted=false checks UTF-8 before constructing a source str.
    let reader = unsafe { Reader::new(base, extension, false) }?;
    reader.validate()?;
    // SAFETY: The complete tables and references were validated above.
    unsafe { read_tables(reader, allocator) }
}

/// Borrow trusted parser/encoder transfers without repeating structural validation.
/// Headers, section bounds and committed delta targets are still checked.
///
/// # Safety
/// Both buffers must be produced by the matching parser/incremental encoder and
/// remain immutable. All kinds, typed references, ranges, strings and graph edges
/// must be valid and acyclic. The source must contain valid UTF-8.
pub unsafe fn deserialize_unchecked<'a>(
    base: &'a [u8],
    extension: &'a [u8],
    allocator: &'a Allocator,
) -> Result<BorrowedAst<'a>, Error> {
    let reader = unsafe { Reader::new(base, extension, true) }?;
    unsafe { read_tables(reader, allocator) }
}

unsafe fn read_tables<'a>(
    reader: Reader<'a>,
    allocator: &'a Allocator,
) -> Result<BorrowedAst<'a>, Error> {
    let base = &reader.base;
    #[cfg(target_endian = "little")]
    if reader.extension.is_none()
        && base.bytes[base.nodes..]
            .as_ptr()
            .addr()
            .is_multiple_of(align_of::<Span>())
        && base.bytes[base.payloads..]
            .as_ptr()
            .addr()
            .is_multiple_of(align_of::<NodePayload>())
        && base.bytes[base.extra..]
            .as_ptr()
            .addr()
            .is_multiple_of(align_of::<ExtraData>())
    {
        let mut ast = Ast::new(reader.source, allocator);
        // SAFETY: The caller supplies validated/contractually valid immutable
        // tables. The read-only owner prevents any mutable access to borrowed data.
        unsafe {
            ast.nodes.span =
                NonNull::new_unchecked(base.bytes.as_ptr().add(base.nodes).cast_mut().cast());
            ast.nodes.kind =
                NonNull::new_unchecked(base.bytes.as_ptr().add(base.kinds).cast_mut().cast());
            ast.nodes.data =
                NonNull::new_unchecked(base.bytes.as_ptr().add(base.payloads).cast_mut().cast());
            ast.nodes.len = reader.count as u32;
            ast.nodes.cap = reader.count as u32;
            ast.extra_data = ArenaIndexVec::from_vec(ArenaVec::from_borrowed_slice_in(
                slice::from_raw_parts(
                    base.bytes.as_ptr().add(base.extra).cast(),
                    reader.extra_count,
                ),
                allocator,
            ));
            ast.strings.borrow_extra(
                &base.bytes[base.strings + base.source_len..base.strings + base.string_len],
            );
            ast.source_type = std::mem::transmute::<u8, SourceType>((reader.flags & 3) as u8);
            ast.lang = std::mem::transmute::<u8, Lang>((reader.flags >> 2) as u8);
        }
        ast.root = Some(NodeId::from_usize(reader.root));
        return Ok(BorrowedAst::from_ast(ast));
    }
    let mut ast = Ast::new(reader.source, allocator);
    ast.reserve(reader.count, reader.extra_count);
    #[cfg(target_endian = "little")]
    {
        let mut node_start = 0;
        let mut extra_start = 0;
        for part in std::iter::once(&reader.base).chain(reader.extension.iter()) {
            // Byte copies support unaligned input while arena tables stay aligned.
            unsafe {
                if let Some(rows) = &part.rows {
                    for row in part.bytes[part.nodes..part.nodes + rows.len() * ROW_SIZE]
                        .chunks_exact(ROW_SIZE)
                    {
                        let target = word(row, 12)? as usize;
                        std::ptr::copy_nonoverlapping(
                            row.as_ptr(),
                            ast.nodes.span.as_ptr().add(target).cast::<u8>(),
                            SPAN_SIZE,
                        );
                        std::ptr::copy_nonoverlapping(
                            row.as_ptr().add(8),
                            ast.nodes.kind.as_ptr().add(target).cast::<u8>(),
                            1,
                        );
                        std::ptr::copy_nonoverlapping(
                            row.as_ptr().add(16),
                            ast.nodes.data.as_ptr().add(target).cast::<u8>(),
                            PAYLOAD_SIZE,
                        );
                    }
                } else {
                    std::ptr::copy_nonoverlapping(
                        part.bytes.as_ptr().add(part.nodes),
                        ast.nodes.span.as_ptr().add(node_start).cast::<u8>(),
                        part.count * SPAN_SIZE,
                    );
                    std::ptr::copy_nonoverlapping(
                        part.bytes.as_ptr().add(part.kinds),
                        ast.nodes.kind.as_ptr().add(node_start).cast::<u8>(),
                        part.count,
                    );
                    std::ptr::copy_nonoverlapping(
                        part.bytes.as_ptr().add(part.payloads),
                        ast.nodes.data.as_ptr().add(node_start).cast::<u8>(),
                        part.count * PAYLOAD_SIZE,
                    );
                }
                std::ptr::copy_nonoverlapping(
                    part.bytes.as_ptr().add(part.extra),
                    ast.extra_data.as_mut_ptr().add(extra_start).cast::<u8>(),
                    part.extra_count * 8,
                );
            }
            node_start += part.count;
            extra_start += part.extra_count;
        }
        // The base copy and complete, unique delta targets initialize every slot.
        ast.nodes.len = reader.count as u32;
        unsafe { ast.extra_data.set_len(reader.extra_count) };
    }
    #[cfg(target_endian = "big")]
    for id in 0..reader.count {
        let (part, index) = reader.node(id)?;
        let kind = reader.kind(id)?;
        let mut raw: [u8; 16] = part.payload(index).try_into().unwrap();
        crate::generated::ast_payload::swap_payload_endian(&mut raw, kind);
        ast.add_node(part.span(index)?, kind, NodePayload { raw });
    }
    #[cfg(target_endian = "big")]
    for id in 0..reader.extra_count {
        ast.extra_data.push(ExtraData::from_raw(u64::from_le_bytes(
            reader.extra(id)?.try_into().unwrap(),
        )));
    }
    ast.strings
        .extend_extra(&base.bytes[base.strings + base.source_len..base.strings + base.string_len]);
    if let Some(extension) = &reader.extension {
        ast.strings.extend_extra(
            &extension.bytes[extension.strings..extension.strings + extension.string_len],
        );
    }
    ast.root = Some(NodeId::from_usize(reader.root));
    ast.set_source_metadata(
        unsafe { std::mem::transmute::<u8, SourceType>((reader.flags & 3) as u8) },
        unsafe { std::mem::transmute::<u8, Lang>((reader.flags >> 2) as u8) },
    );
    Ok(BorrowedAst::from_ast(ast))
}
