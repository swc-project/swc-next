use swc_next_allocator::Allocator;
use swc_next_ecma_ast::ffi::ast::{buffer_size, serialize_into};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{SyntaxChecks, check_syntax};

pub use swc_next_ecma_ast::{Lang, SourceType};
pub use swc_next_ecma_parser::CommentMode;

/// Complete parser settings shared by standalone parsing and JavaScript pipelines.
///
/// The JavaScript facade selects the relevant settings for each pipeline.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsParserOptions {
    /// Determines how the source is parsed and evaluated.
    pub source_type: Option<SourceType>,
    /// Determines which syntax features are enabled.
    pub lang: Option<Lang>,
    /// Preserve explicit parenthesized AST nodes.
    pub preserve_parens: Option<bool>,
    /// Determines whether comments are collected.
    pub comments: Option<CommentMode>,
    /// Run semantic early-error checks after parsing, including after recovery.
    pub semantic_errors: Option<bool>,
}

impl From<JsParserOptions> for Options {
    fn from(options: JsParserOptions) -> Self {
        let defaults = Self::default();
        Self {
            source_type: options.source_type.unwrap_or(defaults.source_type),
            lang: options.lang.unwrap_or(defaults.lang),
            preserve_parens: options.preserve_parens.unwrap_or(defaults.preserve_parens),
            comments: options.comments.unwrap_or(defaults.comments),
        }
    }
}

/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
/// `source_utf16_len` must match the UTF-16 code unit length of `source`.
pub unsafe fn parse_to_buffer(
    source: &[u8],
    source_utf16_len: u32,
    options: Option<JsParserOptions>,
) -> Vec<u8> {
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    let allocator = Allocator::default();
    let options = options.unwrap_or_default();
    let result = Parser::init(&allocator, source, options.into(), NoTokenParserConfig).parse();

    let mut diagnostics = result.diagnostics;
    if options.semantic_errors.unwrap_or(false) {
        diagnostics.extend(check_syntax(&result.ast, SyntaxChecks::all()));
    }

    let buffer_size = buffer_size(&result.ast, &diagnostics);
    let mut buffer = vec![0; buffer_size];
    serialize_into(
        &result.ast,
        &diagnostics,
        &mut buffer,
        source_utf16_len as usize,
    );
    buffer
}
