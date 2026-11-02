mod options;
mod options_wire;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::ffi::ast::{buffer_size, serialize_into};
use swc_next_ecma_ast::{AstCtx, Diagnostic};
use swc_next_ecma_parser::{NoTokenParserConfig, Parser};
use swc_next_ecma_semantic::{SyntaxChecks, check_syntax};

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
    // SAFETY: The caller guarantees valid UTF-8 source.
    unsafe {
        parse_and_serialize(source, options, |ast, diagnostics| {
            let mut buffer = vec![0; buffer_size(ast, diagnostics)];
            serialize_into(ast, diagnostics, &mut buffer, source_utf16_len as usize);
            buffer
        })
    }
}

/// Parse into a self-contained FFI2 buffer with UTF-8 byte spans.
///
/// # Safety
/// The caller must ensure that `source` contains valid UTF-8.
pub unsafe fn parse_to_buffer2(source: &[u8], options: Option<ParserOptions>) -> Vec<u8> {
    // SAFETY: The caller guarantees valid UTF-8 source, and the context is only read to
    // serialize it, so none of its handles outlives the call.
    unsafe {
        parse_and_serialize(source, options, |ast, diagnostics| {
            swc_next_ecma_ast::ffi2::serialize(ast.ast(), diagnostics)
                .expect("parser AST must fit the FFI2 format and be acyclic")
        })
    }
}

unsafe fn parse_and_serialize(
    source: &[u8],
    options: Option<ParserOptions>,
    serialize: impl FnOnce(&AstCtx<'_>, &[Diagnostic]) -> Vec<u8>,
) -> Vec<u8> {
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    let allocator = Allocator::default();
    let options = options.unwrap_or_default().normalize();
    let result = Parser::init(&allocator, source, options.parser, NoTokenParserConfig).parse();

    let mut diagnostics = result.diagnostics;
    if options.semantic_errors {
        diagnostics.extend(check_syntax(&result.ast, SyntaxChecks::all()));
    }

    serialize(&result.ast, &diagnostics)
}
