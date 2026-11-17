//! Internal adapters for synchronous JavaScript AST plugins.
//! These bindings use the matching AST and transform result protocols.

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::ffi::ast::{BorrowedAst, buffer_size, serialize_into};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{SyntaxChecks, check_syntax};

use super::bridge::{TransformApiOptions, output_to_buffer};
use crate::js::{CommentsOption, JsOutput};

fn options(value: &[u8]) -> Result<TransformApiOptions, String> {
    let options = super::bridge::decode_options(value)
        .ok_or_else(|| "invalid transform configuration".to_owned())?
        .unwrap_or_default();
    if options.source_map.unwrap_or(false) {
        return Err("sourceMap is unsupported with JavaScript plugins: AST transfers do not retain source text".into());
    }
    Ok(options)
}

/// Parse the full AST for JavaScript visitors using the current transform configuration.
///
/// # Safety
/// `source` must be valid UTF-8 and `source_utf16_len` must match its UTF-16 length.
pub unsafe fn parse_for_plugins(
    source: &[u8],
    source_utf16_len: u32,
    config: &[u8],
) -> Result<Vec<u8>, String> {
    let options = options(config)?;
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    let parser = options.parser.unwrap_or_default();
    let lang = parser
        .lang
        .map(Into::into)
        .or_else(|| {
            options
                .filename
                .as_deref()
                .map(swc_next_ecma_ast::Lang::from_path)
        })
        .unwrap_or_default();
    let source_type = parser
        .source_type
        .map(Into::into)
        .or_else(|| {
            options
                .filename
                .as_deref()
                .map(swc_next_ecma_ast::SourceType::from_path)
        })
        .unwrap_or_default();
    let allocator = Allocator::default();
    let parsed = Parser::init(
        &allocator,
        source,
        ParserOptions {
            lang,
            source_type,
            comments: parser
                .comments
                .or(options.comments)
                .unwrap_or(CommentsOption::All)
                .into(),
            ..parser.into()
        },
        NoTokenParserConfig,
    )
    .parse();
    let mut diagnostics = parsed.diagnostics;
    if parser.semantic_errors.unwrap_or(false) {
        diagnostics.extend(check_syntax(&parsed.ast, SyntaxChecks::all()));
    }
    if options.comments.unwrap_or(CommentsOption::All) == CommentsOption::All
        && !parsed.ast.comments().is_empty()
    {
        return Err(
            "Comment preservation across JavaScript plugins is unsupported; set comments: \"none\""
                .into(),
        );
    }
    let mut bytes = vec![0; buffer_size(&parsed.ast, &diagnostics)];
    serialize_into(
        &parsed.ast,
        &diagnostics,
        &mut bytes,
        source_utf16_len as usize,
    );
    Ok(bytes)
}

/// Check enabled semantic diagnostics on the AST edited by `afterParse` plugins.
///
/// # Safety
/// `parts` must come from the matching AST encoder and remain immutable.
pub unsafe fn check_edited(parts: [&[u8]; 6], config: &[u8]) -> Result<Vec<u8>, String> {
    let options = options(config)?;
    unsafe {
        with_parts(parts, |ast| {
            // The transfer spans are already UTF-16. Source-only checks ran before
            // callbacks and cannot be repeated without original source text.
            let checks = if options
                .parser
                .unwrap_or_default()
                .semantic_errors
                .unwrap_or(false)
            {
                SyntaxChecks::all()
                    - SyntaxChecks::OCTAL_ESCAPES
                    - SyntaxChecks::LEGACY_NUMERIC_LITERALS
            } else {
                SyntaxChecks::empty()
            };
            pack(JsOutput {
                diagnostics: check_syntax(&ast, checks),
                ..Default::default()
            })
        })
    }
}

/// Print the AST edited by JavaScript plugins; no transforms or semantic recheck.
///
/// # Safety
/// `parts` must come from the matching AST encoder and remain immutable.
pub unsafe fn print_edited(parts: [&[u8]; 6], config: &[u8]) -> Result<Vec<u8>, String> {
    let options = options(config)?;
    let [header, spans, kinds, payloads, extra, strings] = parts;
    let generated = unsafe {
        crate::js::generator::generate_from_parts(
            header,
            spans,
            kinds,
            payloads,
            extra,
            strings,
            options.generator,
        )
    }
    .map_err(|error| error.to_string())?;
    Ok(pack(JsOutput {
        code: generated.code,
        ..Default::default()
    }))
}

fn pack(output: JsOutput) -> Vec<u8> {
    let Ok(bytes) = output_to_buffer("", 0, output, |size| {
        Ok::<_, std::convert::Infallible>(vec![0; size])
    });
    bytes
}

unsafe fn with_parts(
    parts: [&[u8]; 6],
    f: impl FnOnce(BorrowedAst<'_>) -> Vec<u8>,
) -> Result<Vec<u8>, String> {
    let allocator = Allocator::default();
    #[cfg(all(not(target_family = "wasm"), target_endian = "little"))]
    {
        let [header, spans, kinds, payloads, extra, strings] = parts;
        let ast = unsafe {
            swc_next_ecma_ast::ffi::ast::deserialize_parts(
                header, spans, kinds, payloads, extra, strings, &allocator,
            )
        }
        .map_err(|error| error.to_string())?;
        Ok(f(ast))
    }
    #[cfg(any(target_family = "wasm", target_endian = "big"))]
    {
        let [header, spans, kinds, payloads, extra, strings] = parts;
        let payload_offset = (header.len() + spans.len() + kinds.len() + 7) & !7;
        let extra_offset = payload_offset + payloads.len();
        let strings_offset = extra_offset + extra.len();
        let length = (strings_offset + strings.len() + 3) & !3;
        let mut words = vec![0_u64; length.div_ceil(8)];
        // u64 storage provides the alignment required by the borrowed AST reader.
        // The zeroed allocation initializes every byte, including section padding.
        let buffer =
            unsafe { std::slice::from_raw_parts_mut(words.as_mut_ptr().cast::<u8>(), length) };
        let mut offset = 0;
        for part in [header, spans, kinds] {
            buffer[offset..offset + part.len()].copy_from_slice(part);
            offset += part.len();
        }
        offset = payload_offset;
        for part in [payloads, extra, strings] {
            buffer[offset..offset + part.len()].copy_from_slice(part);
            offset += part.len();
        }
        let ast = unsafe { swc_next_ecma_ast::ffi::ast::deserialize_from(buffer, &allocator) }
            .map_err(|error| error.to_string())?;
        Ok(f(ast))
    }
}
