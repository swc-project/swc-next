use rustc_hash::FxHashMap;
use swc_next_simd::prelude::*;

use crate::util::next_code_point;

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub struct SourceMap<'a> {
    pub(crate) version: u8,
    pub(crate) file: Option<&'a [u8]>,
    pub(crate) source_root: Option<&'a [u8]>,
    pub(crate) sources: Vec<Vec<u8>>,
    pub(crate) sources_content: Option<Vec<Option<&'a [u8]>>>,
    pub(crate) names: Vec<&'a [u8]>,
    pub(crate) mappings: Vec<u8>,
}

impl<'a> SourceMap<'a> {
    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn file(&self) -> Option<&'a [u8]> {
        self.file
    }

    pub fn source_root(&self) -> Option<&'a [u8]> {
        self.source_root
    }

    pub fn sources(&self) -> &[Vec<u8>] {
        &self.sources
    }

    pub fn sources_content(&self) -> Option<&[Option<&'a [u8]>]> {
        self.sources_content.as_deref()
    }

    pub fn names(&self) -> &[&'a [u8]] {
        &self.names
    }

    pub fn mappings(&self) -> &[u8] {
        &self.mappings
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct SourceMapState {
    // This isn't stored in the source map. It's only used to join source map
    // chunks together correctly.
    pub(crate) generated_line: i32,

    // These are stored in the source map in VLQ format.
    pub(crate) generated_column: i32,
    pub(crate) source_index: i32,
    pub(crate) original_line: i32,
    pub(crate) original_column: i32,
    pub(crate) original_name: i32,
    pub(crate) has_original_name: bool,
}

#[allow(dead_code)] // Used when independently-generated chunks are joined.
pub(crate) struct MappingsBuffer {
    pub(crate) data: Vec<u8>,
    pub(crate) first_name_offset: Option<u32>,
}

#[allow(dead_code)] // The single-source adapter only consumes part of a chunk.
pub(crate) struct Chunk<'a> {
    pub(crate) buffer: MappingsBuffer,
    pub(crate) quoted_names: Vec<&'a [u8]>,

    // This end state is used to rewrite the start of a following source map
    // chunk so the delta-encoded VLQ numbers are preserved.
    pub(crate) end_state: SourceMapState,

    // There normally isn't a source mapping at the end of the file, but this
    // is needed when another source map chunk is appended after this one.
    pub(crate) final_generated_column: i32,
    pub(crate) should_ignore: bool,
}

impl<'a> Chunk<'a> {
    pub(crate) fn finalize(self, source: &'a [u8], source_path: Vec<u8>) -> SourceMap<'a> {
        // Chunk joining and JSON assembly are intentionally outside sourcemap.rs:
        // they adapt esbuild's chunk-oriented API to this crate's single-source
        // public result type.
        let Chunk {
            buffer: MappingsBuffer { data: mappings, .. },
            quoted_names: names,
            ..
        } = self;
        SourceMap {
            version: 3,
            file: None,
            source_root: None,
            sources: vec![source_path],
            sources_content: Some(vec![Some(source)]),
            names,
            mappings,
        }
    }
}

pub(crate) struct LineOffsetTable {
    columns_for_non_ascii: Option<Vec<i32>>,
    byte_offset_to_first_non_ascii: i32,
    byte_offset_to_start_of_line: i32,
}

pub(crate) fn generate_line_offset_tables(contents: &[u8]) -> Vec<LineOffsetTable> {
    let mut columns_for_non_ascii = None::<Vec<i32>>;
    let mut byte_offset_to_first_non_ascii = 0i32;
    let mut line_byte_offset = 0usize;
    let mut column_byte_offset = 0usize;
    let mut column = 0i32;
    let mut line_offset_tables = Vec::new();

    let mut index = 0;
    while index < contents.len() {
        if column == 0 {
            line_byte_offset = index;
        }
        // Before the first non-ASCII byte, byte and UTF-16 columns coincide.
        if columns_for_non_ascii.is_none() {
            while let Some(chunk) = contents[index..].first_chunk::<16>() {
                let bytes = u8x16::from_array(*chunk);
                let special = bytes.simd_ge(u8x16::splat(0x80))
                    | bytes.simd_eq(u8x16::splat(b'\n'))
                    | bytes.simd_eq(u8x16::splat(b'\r'));
                let run = special.first_set().unwrap_or(16);
                index += run;
                column += run as i32;
                if run < 16 {
                    break;
                }
            }
            if index == contents.len() {
                break;
            }
        }
        let (code_point, width) = next_code_point(contents, index);

        if code_point > 0x7f && columns_for_non_ascii.is_none() {
            column_byte_offset = index - line_byte_offset;
            byte_offset_to_first_non_ascii = column_byte_offset as i32;
            columns_for_non_ascii = Some(Vec::new());
        }

        if let Some(columns) = &mut columns_for_non_ascii {
            let line_bytes_so_far = index - line_byte_offset;
            while column_byte_offset <= line_bytes_so_far {
                columns.push(column);
                column_byte_offset += 1;
            }
        }

        match code_point {
            0x0d | 0x0a | 0x2028 | 0x2029 => {
                if code_point == 0x0d && contents.get(index + 1) == Some(&b'\n') {
                    column += 1;
                    index += width;
                    continue;
                }

                line_offset_tables.push(LineOffsetTable {
                    byte_offset_to_start_of_line: line_byte_offset as i32,
                    byte_offset_to_first_non_ascii,
                    columns_for_non_ascii: columns_for_non_ascii.take(),
                });
                column_byte_offset = 0;
                byte_offset_to_first_non_ascii = 0;
                column = 0;
            }
            _ if code_point <= 0xffff => column += 1,
            _ => column += 2,
        }
        index += width;
    }

    if column == 0 {
        line_byte_offset = contents.len();
    }

    if let Some(columns) = &mut columns_for_non_ascii {
        let line_bytes_so_far = contents.len() - line_byte_offset;
        while column_byte_offset <= line_bytes_so_far {
            columns.push(column);
            column_byte_offset += 1;
        }
    }

    line_offset_tables.push(LineOffsetTable {
        byte_offset_to_start_of_line: line_byte_offset as i32,
        byte_offset_to_first_non_ascii,
        columns_for_non_ascii,
    });
    line_offset_tables
}

pub(crate) struct ChunkBuilder<'a> {
    source_map: Vec<u8>,
    quoted_names: Vec<&'a [u8]>,
    names_map: FxHashMap<&'a [u8], u32>,
    line_offset_tables: Vec<LineOffsetTable>,
    prev_original_name: &'a [u8],
    prev_state: SourceMapState,
    last_generated_update: usize,
    generated_column: i32,
    prev_generated_len: usize,
    prev_original_loc: Option<u32>,
    first_name_offset: Option<u32>,
    has_prev_state: bool,
    line_starts_with_mapping: bool,
    cover_lines_without_mappings: bool,
}

pub(crate) fn make_chunk_builder<'a>(line_offset_tables: Vec<LineOffsetTable>) -> ChunkBuilder<'a> {
    ChunkBuilder {
        source_map: Vec::new(),
        quoted_names: Vec::new(),
        names_map: FxHashMap::default(),
        line_offset_tables,
        prev_original_name: &[],
        prev_state: SourceMapState::default(),
        last_generated_update: 0,
        generated_column: 0,
        prev_generated_len: 0,
        prev_original_loc: None,
        first_name_offset: None,
        has_prev_state: false,

        // There is no nested input source map in this reduced implementation,
        // so repeat prior mappings to cover otherwise-unmapped generated lines.
        line_starts_with_mapping: false,
        cover_lines_without_mappings: true,
    }
}

impl<'a> ChunkBuilder<'a> {
    pub(crate) fn add_source_mapping(
        &mut self,
        original_loc: u32,
        original_name: &'a [u8],
        output: &[u8],
    ) {
        if self.prev_original_loc == Some(original_loc)
            && (self.prev_generated_len == output.len() || self.prev_original_name == original_name)
        {
            return;
        }

        self.prev_original_loc = Some(original_loc);
        self.prev_generated_len = output.len();
        self.prev_original_name = original_name;

        let original_line = self
            .line_offset_tables
            .partition_point(|line| line.byte_offset_to_start_of_line <= original_loc as i32)
            .saturating_sub(1);
        let line = &self.line_offset_tables[original_line];
        let mut original_column = original_loc as i32 - line.byte_offset_to_start_of_line;
        if let Some(columns) = &line.columns_for_non_ascii
            && original_column >= line.byte_offset_to_first_non_ascii
        {
            original_column =
                columns[(original_column - line.byte_offset_to_first_non_ascii) as usize];
        }

        self.update_generated_line_and_column(output);

        if self.cover_lines_without_mappings
            && !self.line_starts_with_mapping
            && self.generated_column > 0
            && self.has_prev_state
        {
            self.append_mapping_without_remapping(SourceMapState {
                generated_line: self.prev_state.generated_line,
                generated_column: 0,
                source_index: self.prev_state.source_index,
                original_line: self.prev_state.original_line,
                original_column: self.prev_state.original_column,
                ..SourceMapState::default()
            });
        }

        self.append_mapping(
            original_name,
            SourceMapState {
                generated_line: self.prev_state.generated_line,
                generated_column: self.generated_column,
                original_line: original_line as i32,
                original_column,
                ..SourceMapState::default()
            },
        );
        self.line_starts_with_mapping = true;
    }

    pub(crate) fn generate_chunk(mut self, output: &[u8]) -> Chunk<'a> {
        self.update_generated_line_and_column(output);
        let should_ignore = self.source_map.iter().all(|&byte| byte == b';');
        Chunk {
            buffer: MappingsBuffer {
                data: self.source_map,
                first_name_offset: self.first_name_offset,
            },
            quoted_names: self.quoted_names,
            end_state: self.prev_state,
            final_generated_column: self.generated_column,
            should_ignore,
        }
    }

    fn update_generated_line_and_column(&mut self, output: &[u8]) {
        let mut index = self.last_generated_update;
        while index < output.len() {
            while let Some(chunk) = output[index..].first_chunk::<16>() {
                let bytes = u8x16::from_array(*chunk);
                let special = bytes.simd_ge(u8x16::splat(0x80))
                    | bytes.simd_eq(u8x16::splat(b'\n'))
                    | bytes.simd_eq(u8x16::splat(b'\r'));
                let run = special.first_set().unwrap_or(16);
                index += run;
                self.generated_column += run as i32;
                if run < 16 {
                    break;
                }
            }
            if index == output.len() {
                break;
            }
            let (code_point, width) = next_code_point(output, index);
            match code_point {
                0x0d | 0x0a | 0x2028 | 0x2029 => {
                    if code_point == 0x0d && output.get(index + 1) == Some(&b'\n') {
                        index += width;
                        continue;
                    }

                    if self.cover_lines_without_mappings
                        && !self.line_starts_with_mapping
                        && self.has_prev_state
                    {
                        self.append_mapping_without_remapping(SourceMapState {
                            generated_line: self.prev_state.generated_line,
                            generated_column: 0,
                            source_index: self.prev_state.source_index,
                            original_line: self.prev_state.original_line,
                            original_column: self.prev_state.original_column,
                            ..SourceMapState::default()
                        });
                    }

                    self.prev_state.generated_line += 1;
                    self.prev_state.generated_column = 0;
                    self.generated_column = 0;
                    self.source_map.push(b';');
                    self.line_starts_with_mapping = false;
                }
                _ if code_point <= 0xffff => self.generated_column += 1,
                _ => self.generated_column += 2,
            }
            index += width;
        }
        self.last_generated_update = output.len();
    }

    fn append_mapping(&mut self, original_name: &'a [u8], mut current_state: SourceMapState) {
        if !original_name.is_empty() {
            let name_index = if let Some(&index) = self.names_map.get(original_name) {
                index
            } else {
                let index = self.quoted_names.len() as u32;
                self.quoted_names.push(original_name);
                self.names_map.insert(original_name, index);
                index
            };
            current_state.original_name = name_index as i32;
            current_state.has_original_name = true;
        }
        self.append_mapping_without_remapping(current_state);
    }

    fn append_mapping_without_remapping(&mut self, current_state: SourceMapState) {
        let last_byte = self.source_map.last().copied().unwrap_or_default();
        let name_offset = append_mapping_to_buffer(
            &mut self.source_map,
            last_byte,
            self.prev_state,
            current_state,
            false,
        );
        let previous_original_name = self.prev_state.original_name;
        self.prev_state = current_state;
        if !current_state.has_original_name {
            self.prev_state.original_name = previous_original_name;
        } else if self.first_name_offset.is_none() {
            self.first_name_offset = name_offset;
        }
        self.has_prev_state = true;
    }
}

fn append_mapping_to_buffer(
    buffer: &mut Vec<u8>,
    last_byte: u8,
    previous_state: SourceMapState,
    current_state: SourceMapState,
    omit_source: bool,
) -> Option<u32> {
    if last_byte != 0 && last_byte != b';' && last_byte != b'"' {
        buffer.push(b',');
    }

    encode_vlq(
        buffer,
        current_state.generated_column - previous_state.generated_column,
    );
    if !omit_source {
        encode_vlq(
            buffer,
            current_state.source_index - previous_state.source_index,
        );
        encode_vlq(
            buffer,
            current_state.original_line - previous_state.original_line,
        );
        encode_vlq(
            buffer,
            current_state.original_column - previous_state.original_column,
        );
    }

    if current_state.has_original_name {
        let name_offset = buffer.len() as u32;
        encode_vlq(
            buffer,
            current_state.original_name - previous_state.original_name,
        );
        Some(name_offset)
    } else {
        None
    }
}

fn encode_vlq(encoded: &mut Vec<u8>, value: i32) {
    let mut vlq = if value < 0 {
        ((-(value as i64)) << 1 | 1) as u64
    } else {
        (value as u64) << 1
    };

    if vlq >> 5 == 0 {
        encoded.push(BASE64[(vlq & 31) as usize]);
        return;
    }

    loop {
        let mut digit = (vlq & 31) as usize;
        vlq >>= 5;
        if vlq != 0 {
            digit |= 32;
        }
        encoded.push(BASE64[digit]);
        if vlq == 0 {
            break;
        }
    }
}
