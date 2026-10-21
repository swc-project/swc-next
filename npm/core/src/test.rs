use swc_next_allocator::Allocator;
use swc_next_bridge::{CodegenOptions, ParserOptions, bridge};
use swc_next_ecma_codegen::print;
use swc_next_ecma_parser::{NoTokenParserConfig, Parser};

/// Parse and print directly in Rust, without serializing or converting the AST.
#[bridge]
pub fn native_parse_print_sync(
    source: String,
    parser_options: Option<ParserOptions>,
    codegen_options: Option<CodegenOptions>,
) -> Result<String, String> {
    let allocator = Allocator::default();
    let result = Parser::init(
        &allocator,
        &source,
        parser_options.unwrap_or_default().into(),
        NoTokenParserConfig,
    )
    .parse();
    let output = print(&result.ast, codegen_options.unwrap_or_default().into());
    Ok(output.code)
}
