mod options;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::ffi::ast::{buffer_size, serialize_into};
use swc_next_ecma_parser::{NoTokenParserConfig, Parser};

pub use self::options::{CommentMode, Lang, ParserOptions, SourceType};

pub fn parse_to_buffer(source: &str, options: Option<ParserOptions>) -> Vec<u8> {
    let allocator = Allocator::default();
    let result = Parser::init(
        &allocator,
        source,
        options.unwrap_or_default().into(),
        NoTokenParserConfig,
    )
    .parse();

    let buffer_size = buffer_size(&result.ast);
    let mut buffer = vec![0; buffer_size];
    serialize_into(&result.ast, &mut buffer);
    buffer
}

#[cfg(test)]
mod tests {
    use super::{CommentMode, ParserOptions, parse_to_buffer};

    #[test]
    fn serializes_flat_comments_in_source_order() {
        let source = "#!/usr/bin/env node\n// one\n/* two */";
        let default_buffer = parse_to_buffer(source, None);
        let flat_buffer = parse_to_buffer(
            source,
            Some(ParserOptions {
                comments: Some(CommentMode::Flat),
                ..ParserOptions::default()
            }),
        );
        assert_eq!(default_buffer, flat_buffer);

        let node_count = read_u32(&flat_buffer, 0) as usize;
        let extra_count = read_u32(&flat_buffer, 1) as usize;
        let string_pool_len = read_u32(&flat_buffer, 2) as usize;
        assert_eq!(read_u32(&flat_buffer, 4), 2);
        assert_eq!(read_u32(&flat_buffer, 5), 0);

        let comments_offset = 36
            + node_count * 8
            + align_to_four(node_count)
            + node_count * 16
            + extra_count * 8
            + align_to_four(string_pool_len);
        assert_eq!(flat_buffer.len(), comments_offset + 2 * 20);

        let line_start = source.find("// one").unwrap() as u32;
        assert_eq!(
            read_record(&flat_buffer, comments_offset),
            [
                0,
                line_start + 2,
                line_start + 6,
                line_start,
                line_start + 6,
            ]
        );

        let block_start = source.find("/* two */").unwrap() as u32;
        assert_eq!(
            read_record(&flat_buffer, comments_offset + 20),
            [
                1,
                block_start + 2,
                block_start + 7,
                block_start,
                block_start + 9,
            ]
        );
    }

    fn read_record(buffer: &[u8], offset: usize) -> [u32; 5] {
        std::array::from_fn(|index| read_u32(buffer, offset / 4 + index))
    }

    fn read_u32(buffer: &[u8], index: usize) -> u32 {
        u32::from_le_bytes(buffer[index * 4..index * 4 + 4].try_into().unwrap())
    }

    const fn align_to_four(value: usize) -> usize {
        (value + 3) & !3
    }
}
