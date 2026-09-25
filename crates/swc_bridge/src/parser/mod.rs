mod options;
mod options_wire;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::ffi::ast::{buffer_size, serialize_into};
use swc_next_ecma_parser::{NoTokenParserConfig, Parser};

pub use self::options::{CommentMode, Lang, ParserOptions, SourceType};

/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
/// `source_utf16_len` must match the UTF-16 code unit length of `source`.
pub unsafe fn parse_to_buffer(
    source: &[u8],
    source_utf16_len: u32,
    options: Option<ParserOptions>,
) -> Vec<u8> {
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    let allocator = Allocator::default();
    let result = Parser::init(
        &allocator,
        source,
        options.unwrap_or_default().into(),
        NoTokenParserConfig,
    )
    .parse();

    let buffer_size = buffer_size(&result.ast, &result.diagnostics);
    let mut buffer = vec![0; buffer_size];
    serialize_into(
        &result.ast,
        &result.diagnostics,
        &mut buffer,
        source_utf16_len as usize,
    );
    buffer
}
