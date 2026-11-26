use swc_next::js::parser::{JsLang, JsParserConfig as ParserConfig, JsSourceType};
use swc_next_allocator::Allocator;
use swc_next_bridge::bridge;
#[cfg(not(target_family = "wasm"))]
use swc_next_bridge::{BridgeDecode, napi, napi_derive::napi};
use swc_next_ecma_ast::{Lang, SourceType, ffi2};
use swc_next_ecma_parser::{NoTokenParserConfig, Parser};
use swc_next_ecma_semantic::{SyntaxChecks, check_syntax};

fn parse_buffer(source: &str, source_utf16_len: u32, options: ParserConfig) -> Vec<u8> {
    let allocator = Allocator::default();
    let mut result = Parser::init(&allocator, source, options.into(), NoTokenParserConfig).parse();
    let mut diagnostics = result.diagnostics;
    if options.semantic_errors.unwrap_or(false) {
        diagnostics.extend(check_syntax(&result.ast, SyntaxChecks::all()));
    }
    // SAFETY: All handles used below come from this AST, and the edits only add nodes.
    let ast = unsafe { result.ast.ast_mut() };
    ffi2::split_export_specifier_names(ast);
    ffi2::serialize(ast, &diagnostics, source_utf16_len as usize)
        .expect("parser AST must fit the AST buffer format")
}

#[cfg(target_family = "wasm")]
#[bridge(ts_args_type = "source: Uint8Array, sourceUtf16Len: number, options: number")]
/// # Safety
/// The caller must supply valid UTF-8 and its UTF-16 code unit length.
pub unsafe fn parse_sync2(
    source: &[u8],
    source_utf16_len: u32,
    options: Option<ParserConfig>,
) -> Vec<u8> {
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    parse_buffer(source, source_utf16_len, options.unwrap_or_default())
}

#[cfg(not(target_family = "wasm"))]
#[napi(
    js_name = "parseSync2",
    ts_args_type = "source: Uint8Array, sourceUtf16Len: number, options: number",
    ts_return_type = "Uint8Array"
)]
/// # Safety
/// The caller must supply valid UTF-8 and its UTF-16 code unit length.
pub unsafe fn parse_sync2(
    source: swc_next_bridge::NativeBytes<'_>,
    source_utf16_len: u32,
    options: u32,
) -> napi::Result<napi::bindgen_prelude::Buffer> {
    let options = <Option<ParserConfig> as BridgeDecode>::decode(options).ok_or_else(|| {
        napi::Error::new(
            napi::Status::InvalidArg,
            "invalid bridge value for `options`",
        )
    })?;
    let source = unsafe { std::str::from_utf8_unchecked(&source) };
    Ok(napi::bindgen_prelude::Buffer::from(parse_buffer(
        source,
        source_utf16_len,
        options.unwrap_or_default(),
    )))
}

#[bridge(ts_return_type = "SourceType")]
pub fn source_type_from_path(path: String) -> JsSourceType {
    SourceType::from_path(&path).into()
}

#[bridge(ts_return_type = "Lang")]
pub fn lang_from_path(path: String) -> JsLang {
    Lang::from_path(&path).into()
}
