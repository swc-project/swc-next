mod options;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, Diagnostic,
    ffi::ast::{buffer_size, serialize_into},
};
use swc_next_ecma_parser::{FragmentContext as AstFragmentContext, NoTokenParserConfig, Parser};

pub use self::options::{
    CommentMode, FragmentContext, FunctionFragmentContext, Lang, ParserOptions, SourceType,
};

pub fn parse_to_buffer(source: &str, options: Option<ParserOptions>) -> Vec<u8> {
    let allocator = Allocator::default();
    let result = Parser::init(
        &allocator,
        source,
        options.unwrap_or_default().into(),
        NoTokenParserConfig,
    )
    .parse();

    serialize_ast(&result.ast)
}

pub fn parse_expression_fragment_to_buffer(
    source: &str,
    fragment_context: FragmentContext,
    options: Option<ParserOptions>,
) -> napi::Result<Vec<u8>> {
    let allocator = Allocator::default();
    let ast = Parser::init(
        &allocator,
        source,
        options.unwrap_or_default().into(),
        NoTokenParserConfig,
    )
    .parse_expression_fragment(AstFragmentContext::from(fragment_context))
    .map_err(diagnostics_to_error)?;
    Ok(serialize_ast(&ast))
}

pub fn parse_statement_fragment_to_buffer(
    source: &str,
    fragment_context: FragmentContext,
    options: Option<ParserOptions>,
) -> napi::Result<Vec<u8>> {
    let allocator = Allocator::default();
    let ast = Parser::init(
        &allocator,
        source,
        options.unwrap_or_default().into(),
        NoTokenParserConfig,
    )
    .parse_statement_fragment(AstFragmentContext::from(fragment_context))
    .map_err(diagnostics_to_error)?;
    Ok(serialize_ast(&ast))
}

fn serialize_ast(ast: &Ast<'_>) -> Vec<u8> {
    let size = buffer_size(ast);
    let mut buffer = vec![0; size];
    serialize_into(ast, &mut buffer);
    buffer
}

fn diagnostics_to_error(diagnostics: Vec<Diagnostic<'_>>) -> napi::Error {
    let mut reason = String::new();
    for diagnostic in diagnostics {
        if !reason.is_empty() {
            reason.push('\n');
        }
        reason.push_str(&diagnostic.message);
    }
    if reason.is_empty() {
        reason.push_str("Failed to parse fragment");
    }
    napi::Error::from_reason(reason)
}
