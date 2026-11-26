use std::collections::HashSet;

use swc_next_allocator::{
    Allocator, hash_map::HashMap as ArenaHashMap, hash_set::HashSet as ArenaHashSet, wtf8::Wtf8,
};

use super::BorrowedAst;
use super::{
    DeserializeError as Error, NONE, PAYLOAD_SIZE, POSITION_RUN_SIZE, SPAN_SIZE,
    schema::{self, Field, FieldValidator},
};
use crate::{Ast, ExtraDataUnit, Lang, NodeId, NodeKind, NodePayload, SourceType, Span};
use std::slice;
#[cfg(target_endian = "little")]
use std::{mem::align_of, ptr::NonNull};
#[cfg(target_endian = "little")]
use swc_next_allocator::vec::ArenaIndexVec;
use swc_next_allocator::vec::ArenaVec;

const ROW_SIZE: usize = 32;
const UTF8_STRINGS: u32 = 0x8000_0000;

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
    // Appended slots are interleaved; their IDs follow the base slots contiguously.
    interleaved: bool,
    utf8_strings: bool,
    positions: usize,
    position_count: usize,
    position_excess: usize,
}

impl<'a> Parts<'a> {
    fn base(bytes: &'a [u8]) -> Result<Self, Error> {
        let count = word(bytes, 0)? as usize;
        let nodes = 8;
        let kinds = end(nodes, count, SPAN_SIZE)?;
        let payloads = end(end(kinds, count, 1)?, 7, 1)? & !7;
        let extra_info = end(payloads, count, PAYLOAD_SIZE)?;
        let extra_count = word(bytes, extra_info)? as usize;
        let extra = end(extra_info, 4, 1)?;
        let strings_info = end(extra, extra_count, 4)?;
        let source_len = word(bytes, strings_info)? as usize;
        let string_len = word(bytes, strings_info + 4)? as usize;
        let strings = end(strings_info, 8, 1)?;
        let comments_info = end(end(strings, string_len, 1)?, 3, 1)? & !3;
        let comments = end(comments_info, 4, 1)?;
        let diagnostics_info = end(comments, word(bytes, comments_info)? as usize, 8)?;
        let diagnostic_count = word(bytes, diagnostics_info)? as usize;
        let diagnostic_len = word(bytes, diagnostics_info + 4)? as usize;
        let positions_info = end(end(diagnostics_info, 8, 1)?, diagnostic_len, 1)?;
        let positions_info = end(positions_info, 3, 1)? & !3;
        let position_count = word(bytes, positions_info)? as usize;
        let position_excess = word(bytes, positions_info + 4)? as usize;
        let positions = end(positions_info, 8, 1)?;
        let metadata = end(positions, position_count, POSITION_RUN_SIZE)?;
        let root = word(bytes, metadata)? as usize;
        let value_flags = word(bytes, metadata + 4)?;
        flags(value_flags)?;
        if count == 0
            || root >= count
            || source_len > string_len
            || position_excess > source_len
            || (position_count == 0) != (position_excess == 0)
            || (diagnostic_count as u64) * 18 > diagnostic_len as u64
            || end(metadata, 8, 1)? != bytes.len()
        {
            return Err(invalid("invalid AST array layout"));
        }
        Ok(Self {
            bytes,
            nodes,
            kinds,
            payloads,
            count,
            extra,
            extra_count,
            strings,
            string_len,
            source_len,
            root,
            flags: value_flags,
            interleaved: false,
            utf8_strings: true,
            positions,
            position_count,
            position_excess,
        })
    }
    fn extension(bytes: &'a [u8], base: &Self) -> Result<Self, Error> {
        let row_count = word(bytes, 0)? as usize;
        let capacity_flags = word(bytes, 4)?;
        let row_capacity = (capacity_flags & !UTF8_STRINGS) as usize;
        let nodes = 8;
        let extra_info = end(nodes, row_capacity, ROW_SIZE)?;
        let extra_count = word(bytes, extra_info)? as usize;
        let extra_capacity = word(bytes, extra_info + 4)? as usize;
        let extra = end(extra_info, 8, 1)?;
        let strings_info = end(extra, extra_capacity, 4)?;
        let string_len = word(bytes, strings_info)? as usize;
        let string_capacity = word(bytes, strings_info + 4)? as usize;
        let strings = end(strings_info, 8, 1)?;
        let metadata = end(strings, string_capacity, 1)?;
        let root = word(bytes, metadata)? as usize;
        let value_flags = word(bytes, metadata + 4)?;
        let count = word(bytes, metadata + 8)? as usize;
        if word(bytes, metadata + 12)? as usize != base.count
            || word(bytes, metadata + 16)? as usize != base.extra_count
            || word(bytes, metadata + 20)? as usize != base.string_len
            || word(bytes, metadata + 24)? as usize != base.bytes.len()
        {
            return Err(invalid("extension does not match its base buffer"));
        }
        let total_count = end(base.count, count, 1)?;
        if row_count > row_capacity
            || extra_count > extra_capacity
            || string_len > string_capacity
            || count != row_count
            || total_count >= NONE as usize
            || end(metadata, 28, 1)? != bytes.len()
        {
            return Err(invalid("invalid delta array layout"));
        }
        flags(value_flags)?;
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
            root,
            flags: value_flags,
            interleaved: true,
            utf8_strings: capacity_flags & UTF8_STRINGS != 0,
            positions: 0,
            position_count: 0,
            position_excess: 0,
        })
    }
    #[cfg(target_endian = "big")]
    fn span(&self, id: usize) -> Result<Span, Error> {
        let stride = if self.interleaved {
            ROW_SIZE
        } else {
            SPAN_SIZE
        };
        let offset = self.nodes + id * stride;
        Ok(Span::new(
            word(self.bytes, offset)?,
            word(self.bytes, offset + 4)?,
        ))
    }
    fn kind(&self, id: usize) -> u8 {
        self.bytes[self.kinds + id * if self.interleaved { ROW_SIZE } else { 1 }]
    }
    fn payload(&self, id: usize) -> &[u8] {
        let stride = if self.interleaved {
            ROW_SIZE
        } else {
            PAYLOAD_SIZE
        };
        let offset = self.payloads + id * stride;
        &self.bytes[offset..offset + PAYLOAD_SIZE]
    }
}

/// The index is query metadata; validation never rewrites the native AST spans.
fn validate_position_runs(base: &Parts<'_>) -> Result<(), Error> {
    let bytes = base.bytes;
    let source_len = base.source_len;
    let table = base.positions;
    let count = base.position_count;
    let mut previous_end = 0usize;
    let mut byte_excess = 0usize;
    for index in 0..count {
        let offset = table + index * POSITION_RUN_SIZE;
        let start = word(bytes, offset)? as usize;
        let prior_excess = word(bytes, offset + 4)? as usize;
        let width = word(bytes, offset + 8)? as usize;
        let stop = word(bytes, offset + 12)? as usize;
        if !(2..=4).contains(&width)
            || start < previous_end
            || start >= stop
            || stop > source_len
            || prior_excess != byte_excess
            || prior_excess > start
            || !(stop - start).is_multiple_of(width)
        {
            return Err(invalid("invalid source position run"));
        }
        let units = if width == 4 { 2 } else { 1 };
        byte_excess += (stop - start) / width * (width - units);
        previous_end = stop;
    }
    if byte_excess != base.position_excess {
        return Err(invalid("invalid source byte excess"));
    }
    Ok(())
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
        if !trusted {
            validate_position_runs(&base)?;
        }
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
        Ok(&part.bytes[part.extra + index * 4..part.extra + part.extra_count * 4])
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
        // The current program must be a tree. Unreachable old slots remain in
        // incremental storage and may still reference nodes in that tree.
        for (pass, root) in std::iter::once(self.root).chain(0..self.count).enumerate() {
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
                    if pass == 0 {
                        return Err(invalid("AST must be a tree"));
                    }
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
    /// Validate a handle into the extension's UTF-16 pool. Every code-unit sequence
    /// transcodes to WTF-8, so only the boundary and the guarantee of a UTF-8 field
    /// are properties of a valid AST; the merge step rewrites the handle.
    fn extension_string(
        &self,
        start: u32,
        end: u32,
        units: &[u8],
        wtf8: bool,
    ) -> Result<(), Error> {
        let base = self.reader.base.string_len as u32;
        if !(start - base).is_multiple_of(2) || !(end - start).is_multiple_of(2) {
            return Err(invalid("string handle does not fall on a UTF-16 boundary"));
        }
        if !wtf8 && utf16_le_has_unpaired_surrogate(units) {
            return Err(invalid("string requires WTF-8 in a UTF-8 field"));
        }
        Ok(())
    }
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
        if start as usize >= self.reader.base.string_len
            && self
                .reader
                .extension
                .as_ref()
                .is_some_and(|part| !part.utf8_strings)
        {
            return self.extension_string(start, end, value, wtf8);
        }
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
/// Array bounds and contiguous appended slot counts are still checked.
///
/// # Safety
/// Both buffers must be produced by the matching parser/incremental encoder and
/// remain immutable for the lifetime of the returned borrowed AST. Edits must
/// preserve valid kinds, typed references, ranges, strings and graph edges; the
/// program must remain an acyclic tree.
/// The source must contain valid UTF-8.
pub unsafe fn deserialize_unchecked<'a>(
    base: &'a [u8],
    extension: &'a [u8],
    allocator: &'a Allocator,
) -> Result<BorrowedAst<'a>, Error> {
    let reader = unsafe { Reader::new(base, extension, true) }?;
    unsafe { read_tables(reader, allocator) }
}

/// Collect or rewrite the string handles of an extension row.
enum StringVisit<'a, 'm> {
    Collect {
        seen: &'m mut ArenaHashSet<'a, (u32, u32)>,
        ranges: &'m mut ArenaVec<'a, (u32, u32)>,
    },
    Remap {
        map: &'m ArenaHashMap<'a, (u32, u32), (u32, u32)>,
    },
}

/// Visits the string fields of one extension row, either collecting the spans they
/// reference or rewriting them. Handles and the extra-data index are read from the
/// transferred bytes, which are little endian and identical to the copy the AST reads, so
/// the rewrite only writes.
struct ExtensionStrings<'a, 'm> {
    /// Row payload as transferred.
    payload: [u8; PAYLOAD_SIZE],
    extra_base: usize,
    /// Original little-endian extra slots can be addressed from either node part.
    base_slots: *const u8,
    appended_slots: *const u8,
    base_extra_count: usize,
    /// Copies to rewrite: the row payload and the extra-data table, absent while
    /// collecting. Payload words are written in native order like every other payload
    /// field, and a string handle occupies two consecutive extra-data units. Handles are always read
    /// from the transfer, so rewriting a slot twice, which shared extra blocks allow,
    /// writes the same value twice.
    payload_out: Option<&'m mut [u8]>,
    extra_out: Option<*mut u8>,
    base_string_len: u32,
    visit: StringVisit<'a, 'm>,
}

impl ExtensionStrings<'_, '_> {
    fn read_slot(&self, index: usize) -> (u32, u32) {
        // SAFETY: `index` addresses a slot of the part's extra data, which validation
        // placed within bounds, and the words are read unaligned from the transfer.
        unsafe {
            let slot = if index < self.base_extra_count {
                self.base_slots.add(index * 4)
            } else {
                self.appended_slots.add((index - self.base_extra_count) * 4)
            };
            (
                std::ptr::read_unaligned(slot.cast::<u32>()).to_le(),
                std::ptr::read_unaligned(slot.add(4).cast::<u32>()).to_le(),
            )
        }
    }

    fn write_slot(&self, index: usize, start: u32, end: u32) {
        let extra = self
            .extra_out
            .expect("only the rewrite pass has arena tables");
        // SAFETY: Validation places both string-handle units within the owned tables.
        unsafe {
            let slot = extra.add(index * 4).cast::<u32>();
            std::ptr::write_unaligned(slot, start);
            std::ptr::write_unaligned(slot.add(1), end);
        }
    }

    /// Collect a referenced span, or resolve the transcoded span that replaces it. Spans
    /// into the base pool keep their handle because that pool transfers as WTF-8.
    fn relocate(
        &mut self,
        optional: bool,
        start: u32,
        end: u32,
    ) -> Result<Option<(u32, u32)>, Error> {
        if (optional && start == NONE && end == 0) || start < self.base_string_len {
            return Ok(None);
        }
        match &mut self.visit {
            StringVisit::Collect { seen, ranges } => {
                if seen.insert((start, end)) {
                    ranges.push((start, end));
                }
                Ok(None)
            }
            StringVisit::Remap { map } => map
                .get(&(start, end))
                .copied()
                .ok_or(invalid("extension string handle was not transcoded"))
                .map(Some),
        }
    }
}

impl FieldValidator for ExtensionStrings<'_, '_> {
    fn extra(&mut self, pointer: usize, _count: usize) -> Result<(), Error> {
        self.extra_base = word(&self.payload, pointer)? as usize;
        Ok(())
    }
    fn node(&mut self, _field: Field, _kinds: &[NodeKind], _optional: bool) -> Result<(), Error> {
        Ok(())
    }
    fn list(&mut self, _field: Field, _kinds: &[NodeKind], _optional: bool) -> Result<(), Error> {
        Ok(())
    }
    fn boolean(&mut self, _field: Field) -> Result<(), Error> {
        Ok(())
    }
    fn enumeration(&mut self, _field: Field, _count: usize, _optional: bool) -> Result<(), Error> {
        Ok(())
    }
    fn string(&mut self, field: Field, _wtf8: bool, optional: bool) -> Result<(), Error> {
        match field {
            Field::Payload(offset) => {
                let start = word(&self.payload, offset)?;
                let end = word(&self.payload, offset + 4)?;
                if let Some((start, end)) = self.relocate(optional, start, end)? {
                    let payload = self
                        .payload_out
                        .as_mut()
                        .expect("only the rewrite pass has arena tables");
                    payload[offset..offset + 4].copy_from_slice(&start.to_ne_bytes());
                    payload[offset + 4..offset + 8].copy_from_slice(&end.to_ne_bytes());
                }
            }
            Field::Extra(offset) => {
                let index = self.extra_base + offset;
                let (start, end) = self.read_slot(index);
                if let Some((start, end)) = self.relocate(optional, start, end)? {
                    self.write_slot(index, start, end);
                }
            }
        }
        Ok(())
    }
}

/// Merge UTF-8 directly; legacy UTF-16 pools need one transcode and handle rewrite.
unsafe fn merge_extension_strings(
    reader: &Reader<'_>,
    ast: &mut Ast<'_>,
    allocator: &Allocator,
) -> Result<(), Error> {
    let extension = reader.extension.as_ref().expect("an appended part exists");
    if extension.string_len == 0 {
        return Ok(());
    }
    if extension.utf8_strings {
        append_string_bytes(
            ast,
            &extension.bytes[extension.strings..extension.strings + extension.string_len],
        )?;
        return Ok(());
    }
    let base_string_len = reader.base.string_len;
    let base_slots = unsafe { reader.base.bytes.as_ptr().add(reader.base.extra) };
    let appended_slots = unsafe { extension.bytes.as_ptr().add(extension.extra) };
    let mut seen = ArenaHashSet::new_in(allocator);
    let mut ranges = ArenaVec::new_in(allocator);
    for target in 0..reader.count {
        let (part, index) = reader.node(target)?;
        let mut visit = ExtensionStrings {
            payload: part.payload(index).try_into().expect("payload width"),
            extra_base: 0,
            base_slots,
            appended_slots,
            base_extra_count: reader.base.extra_count,
            payload_out: None,
            extra_out: None,
            base_string_len: base_string_len as u32,
            visit: StringVisit::Collect {
                seen: &mut seen,
                ranges: &mut ranges,
            },
        };
        schema::validate_fields(reader.kind(target)?, &mut visit)?;
    }
    ranges.sort_unstable();
    let mut map = ArenaHashMap::with_capacity_in(ranges.len(), allocator);
    let mut transcoded = ArenaVec::new_in(allocator);
    for &(start, end) in ranges.iter() {
        let units = &extension.bytes[extension.strings + start as usize - base_string_len
            ..extension.strings + end as usize - base_string_len];
        transcoded.clear();
        extend_wtf8_from_utf16_le(units, &mut transcoded);
        map.insert((start, end), append_string_bytes(ast, &transcoded)?);
    }
    let extra = ast.extra_data.as_mut_ptr().cast::<u8>();
    for target in 0..reader.count {
        let (part, index) = reader.node(target)?;
        let mut payload_out = unsafe {
            slice::from_raw_parts_mut(
                ast.nodes.data.as_ptr().add(target).cast::<u8>(),
                PAYLOAD_SIZE,
            )
        };
        let mut visit = ExtensionStrings {
            payload: part.payload(index).try_into().expect("payload width"),
            extra_base: 0,
            base_slots,
            appended_slots,
            base_extra_count: reader.base.extra_count,
            payload_out: Some(&mut payload_out),
            extra_out: Some(extra),
            base_string_len: base_string_len as u32,
            visit: StringVisit::Remap { map: &map },
        };
        schema::validate_fields(reader.kind(target)?, &mut visit)?;
    }
    Ok(())
}

unsafe fn read_tables<'a>(
    reader: Reader<'a>,
    allocator: &'a Allocator,
) -> Result<BorrowedAst<'a>, Error> {
    let base = &reader.base;
    let base_strings = &base.bytes[base.strings + base.source_len..base.strings + base.string_len];
    // Appended rows or extra slots require owned tables, but a string-only UTF-8
    // extension can keep borrowing the base tables and append to the string pool.
    #[cfg(target_endian = "little")]
    let extension_borrowable = reader.extension.as_ref().is_none_or(|extension| {
        extension.count == 0
            && extension.extra_count == 0
            && (extension.string_len == 0 || extension.utf8_strings)
    });
    #[cfg(target_endian = "little")]
    if extension_borrowable
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
            .is_multiple_of(align_of::<ExtraDataUnit>())
    {
        // SAFETY: The AST stays private until the writes below restore its handle
        // invariants, and it is returned as a read-only owner.
        let mut ast = unsafe { Ast::new(reader.source, allocator) };
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
            let extension_strings = reader.extension.as_ref().map_or(&[][..], |extension| {
                &extension.bytes[extension.strings..extension.strings + extension.string_len]
            });
            let strings = if extension_strings.is_empty() {
                base_strings
            } else {
                let mut strings = ArenaVec::with_capacity_in(
                    base_strings.len() + extension_strings.len(),
                    allocator,
                );
                strings.extend_from_slice(base_strings);
                strings.extend_from_slice(extension_strings);
                strings.into_bump_slice()
            };
            ast.strings.borrow_extra(strings);
            ast.source_type = std::mem::transmute::<u8, SourceType>((reader.flags & 3) as u8);
            ast.lang = std::mem::transmute::<u8, Lang>((reader.flags >> 2) as u8);
        }
        ast.root = Some(NodeId::from_usize(reader.root));
        return Ok(BorrowedAst::from_ast(ast));
    }
    // SAFETY: The AST stays private until the writes below restore its handle
    // invariants, and it is returned as a read-only owner.
    let mut ast = unsafe { Ast::new(reader.source, allocator) };
    ast.reserve(reader.count, reader.extra_count);
    #[cfg(target_endian = "little")]
    {
        let mut node_start = 0;
        let mut extra_start = 0;
        for part in std::iter::once(&reader.base).chain(reader.extension.iter()) {
            // Byte copies support unaligned input while arena tables stay aligned.
            unsafe {
                if part.interleaved {
                    for (index, row) in part.bytes[part.nodes..part.nodes + part.count * ROW_SIZE]
                        .as_chunks::<ROW_SIZE>()
                        .0
                        .iter()
                        .enumerate()
                    {
                        let target = node_start + index;
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
                    part.extra_count * 4,
                );
            }
            node_start += part.count;
            extra_start += part.extra_count;
        }
        // Original columns followed by dense appended slots initialize every node.
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
        ast.extra_data
            .push(ExtraDataUnit::from_raw_node_id(u32::from_le_bytes(
                reader.extra(id)?[..4].try_into().unwrap(),
            )));
    }
    append_string_bytes(&mut ast, base_strings)?;
    if reader.extension.is_some() {
        // SAFETY: The extension part is validated or trusted, and the tables above are
        // arena copies this function owns.
        unsafe { merge_extension_strings(&reader, &mut ast, allocator)? };
    }
    ast.root = Some(NodeId::from_usize(reader.root));
    ast.set_source_metadata(
        unsafe { std::mem::transmute::<u8, SourceType>((reader.flags & 3) as u8) },
        unsafe { std::mem::transmute::<u8, Lang>((reader.flags >> 2) as u8) },
    );
    Ok(BorrowedAst::from_ast(ast))
}

fn append_string_bytes(ast: &mut Ast<'_>, bytes: &[u8]) -> Result<(u32, u32), Error> {
    let start = end(ast.source().len(), ast.strings.extra_bytes().len(), 1)?;
    let end = u32::try_from(end(start, bytes.len(), 1)?)
        .map_err(|_| invalid("AST string storage exceeds u32"))?;
    ast.strings.extend_extra(bytes);
    Ok((start as u32, end))
}

fn utf16_le(units: &[u8]) -> impl Iterator<Item = u16> + '_ {
    units
        .as_chunks::<2>()
        .0
        .iter()
        .map(|unit| u16::from_le_bytes(*unit))
}

fn utf16_le_has_unpaired_surrogate(units: &[u8]) -> bool {
    char::decode_utf16(utf16_le(units)).any(|decoded| decoded.is_err())
}

fn extend_wtf8_from_utf16_le(units: &[u8], out: &mut ArenaVec<'_, u8>) {
    for decoded in char::decode_utf16(utf16_le(units)) {
        match decoded {
            Ok(character) => {
                let mut buffer = [0; 4];
                out.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
            }
            Err(error) => {
                let half = error.unpaired_surrogate();
                out.push((0xE0 | (half >> 12)) as u8);
                out.push((0x80 | ((half >> 6) & 0x3F)) as u8);
                out.push((0x80 | (half & 0x3F)) as u8);
            }
        }
    }
}
