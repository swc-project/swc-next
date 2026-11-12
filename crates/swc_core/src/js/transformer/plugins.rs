use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    AstCtx, Diagnostic, Lang, Severity, SourceType,
    ffi::ast::{BorrowedAst, buffer_size, serialize_into},
};
use swc_next_ecma_codegen::{Options as GeneratorOptions, generate};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze, check_syntax};
use swc_next_ecma_transformer::{Transformer, normalize::Normalizer};

use super::{JsTransformerConfig, encode_output};
use crate::{
    BoolOr,
    js::{CommentsOption, JsOutput, JsTransformApiOptions, serialize_source_map},
};

fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
}

fn comment_policy(options: &JsTransformApiOptions) -> CommentsOption {
    options
        .parser
        .unwrap_or_default()
        .comments
        .or(options.comments)
        .unwrap_or(CommentsOption::All)
}

fn comments_enabled(options: &JsTransformApiOptions) -> bool {
    options.comments.unwrap_or(CommentsOption::All) == CommentsOption::All
}

fn transform_options(options: &JsTransformApiOptions, lang: Lang) -> JsTransformerConfig {
    let mut transform = options.transformer.clone().unwrap_or_default();
    transform
        .typescript
        .get_or_insert(BoolOr::Bool(lang.is_typescript()));
    transform.jsx.get_or_insert(BoolOr::Bool(lang.is_jsx()));
    transform
}

fn parse_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: &JsTransformApiOptions,
) -> (AstCtx<'a>, Vec<Diagnostic>) {
    let parser_options = options.parser.unwrap_or_default();
    let lang = {
        let inferred = options.filename.as_deref().map(Lang::from_path);
        let explicit = parser_options.lang.map(Into::into);
        explicit.or(inferred).unwrap_or_default()
    };
    let transform = transform_options(options, lang);
    // An explicit parser comment policy wins. Otherwise retain JSX pragmas
    // for semantic transforms, then discard comments before generation if needed.
    let jsx_enabled = !matches!(&transform.jsx, None | Some(BoolOr::Bool(false)));
    let typescript_enabled = !matches!(&transform.typescript, None | Some(BoolOr::Bool(false)));
    let collect_pragmas =
        parser_options.comments.is_none() && (jsx_enabled || (typescript_enabled && lang.is_jsx()));
    let parser_comments = if collect_pragmas {
        CommentsOption::All
    } else {
        comment_policy(options)
    };
    let source_type = {
        let inferred = options.filename.as_deref().map(SourceType::from_path);
        let explicit = parser_options.source_type.map(Into::into);
        explicit.or(inferred).unwrap_or_default()
    };
    let parsed = Parser::init(
        allocator,
        source,
        ParserOptions {
            source_type,
            lang,
            comments: parser_comments.into(),
            ..parser_options.into()
        },
        NoTokenParserConfig,
    )
    .parse();
    (parsed.ast, parsed.diagnostics)
}

fn run_transforms(
    ast: &mut AstCtx<'_>,
    options: &JsTransformApiOptions,
    check_syntax: SyntaxChecks,
) -> Vec<Diagnostic> {
    let transform = transform_options(options, ast.lang());
    match &transform.typescript {
        None | Some(BoolOr::Bool(false)) => Normalizer::normalize(ast),
        _ => Normalizer::normalize_runtime(ast),
    }
    let transform = transform.to_transformer_options(options.filename.clone());
    let analyzed = analyze(&mut *ast, AnalyzeOptions { check_syntax });
    let mut diagnostics = analyzed.diagnostics;
    if has_errors(&diagnostics) {
        return diagnostics;
    }
    diagnostics.extend(Transformer::transform(analyzed.semantic, transform).diagnostics);
    diagnostics
}

fn print_ast(
    ast: &AstCtx<'_>,
    options: &JsTransformApiOptions,
    diagnostics: Vec<Diagnostic>,
) -> JsOutput {
    let mut output = JsOutput {
        diagnostics,
        ..Default::default()
    };
    let mut generator = GeneratorOptions::from(options.generator.unwrap_or_default());
    let source_map = options.source_map.unwrap_or(false);
    let filename = options.filename.as_deref().unwrap_or("input.js");
    generator.source_map_path = source_map.then(|| filename.into());
    generator.comments = comments_enabled(options);
    let generated = generate(ast, generator);
    output.code = generated.code;
    output.map = generated.map.as_ref().map(serialize_source_map);
    output
}

fn semantic_checks(options: &JsTransformApiOptions) -> SyntaxChecks {
    if options
        .parser
        .unwrap_or_default()
        .semantic_errors
        .unwrap_or(false)
    {
        SyntaxChecks::all()
    } else {
        SyntaxChecks::empty()
    }
}

fn deserialize_options(options: &str) -> Result<JsTransformApiOptions, String> {
    serde_json::from_str(options).map_err(|error| format!("Invalid transform options: {error}"))
}

fn plugin_options(options: &str) -> Result<JsTransformApiOptions, String> {
    let options = deserialize_options(options)?;
    if options.source_map.unwrap_or(false) {
        return Err("Invalid transform options: sourceMap is unsupported with JavaScript plugins: AST transfers do not retain source text".into());
    }
    Ok(options)
}

/// Parse source for an `afterParse` plugin without running built-in transforms.
///
/// Configuration and source-dependent plugin restrictions are checked before
/// returning an AST. The result uses the parser's AST transfer format.
///
/// # Safety
///
/// `source` must contain valid UTF-8, and `source_utf16_len` must match its
/// UTF-16 code unit length.
pub unsafe fn parse_for_transform_to_buffer(
    source: &[u8],
    source_utf16_len: u32,
    options: &str,
) -> Result<Vec<u8>, String> {
    let options = plugin_options(options)?;
    // SAFETY: The caller guarantees valid UTF-8.
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    let allocator = Allocator::default();
    let (ast, mut diagnostics) = parse_source(&allocator, source, &options);
    if !has_errors(&diagnostics) {
        diagnostics.extend(check_syntax(&ast, semantic_checks(&options)));
    }
    if !has_errors(&diagnostics) {
        check_comment_transfer(&ast, &options)?;
        let jsx = transform_options(&options, ast.lang()).jsx;
        if let Some(BoolOr::Options(jsx)) = jsx
            && (jsx.source_self.unwrap_or(false)
                || !matches!(jsx.refresh, None | Some(BoolOr::Bool(false))))
        {
            return Err("Invalid transform options: JSX sourceSelf and refresh are unsupported with afterParse plugins: AST transfers do not retain source text".into());
        }
    }
    Ok(serialize_ast(&ast, &diagnostics, source_utf16_len))
}

/// Parse and transform source before a `beforeCodegen` plugin.
///
/// The returned AST refers to the original source for strings and uses UTF-16
/// spans. Its JavaScript decoder must receive the original source.
///
/// # Safety
///
/// `source` must contain valid UTF-8, and `source_utf16_len` must match its
/// UTF-16 code unit length.
pub unsafe fn parse_transform_to_buffer(
    source: &[u8],
    source_utf16_len: u32,
    options: &str,
) -> Result<Vec<u8>, String> {
    let options = plugin_options(options)?;
    // SAFETY: The caller guarantees valid UTF-8.
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    let allocator = Allocator::default();
    let (mut ast, mut diagnostics) = parse_source(&allocator, source, &options);
    if !has_errors(&diagnostics) {
        check_comment_transfer(&ast, &options)?;
        diagnostics.extend(run_transforms(
            &mut ast,
            &options,
            semantic_checks(&options),
        ));
    }
    Ok(serialize_ast(&ast, &diagnostics, source_utf16_len))
}

/// Transform trusted JavaScript encoder sections and return an AST transfer.
///
/// Edited ASTs receive enabled semantic checks, excluding source-only numeric
/// literal and escape checks. Their original source is absent; spans are already
/// UTF-16 offsets. Parser-only context checks are not rerun.
///
/// # Safety
///
/// Sections must be produced by the matching `@swc-next/encoder` and satisfy the
/// layout, alignment, and immutability requirements of
/// [`swc_next_ecma_ast::ffi::ast::deserialize_parts`].
pub unsafe fn transform_from_parts(parts: [&[u8]; 6], options: &str) -> Result<Vec<u8>, String> {
    let options = plugin_options(options)?;
    // SAFETY: The caller guarantees valid, immutable encoder sections.
    unsafe {
        with_ast_parts(parts, |ast| {
            let mut ast = ast.into_owned();
            let diagnostics = transform_edited_ast(&mut ast, &options);

            serialize_ast(&ast, &diagnostics, 0)
        })
    }
}

/// Transform trusted JavaScript encoder sections and print the result.
///
/// Returns the packed output used by [`transform_source_to_buffer`](super::transform_source_to_buffer).
///
/// # Safety
///
/// Sections must satisfy the contracts of [`transform_from_parts`].
pub unsafe fn transform_print_from_parts(
    parts: [&[u8]; 6],
    options: &str,
) -> Result<Vec<u8>, String> {
    let options = plugin_options(options)?;
    // SAFETY: The caller guarantees valid, immutable encoder sections.
    unsafe {
        with_ast_parts(parts, |ast| {
            let mut ast = ast.into_owned();
            let diagnostics = transform_edited_ast(&mut ast, &options);
            let output = print_ast(&ast, &options, diagnostics);
            encode_output(&output, "", 0)
        })
    }
}

/// Print a transformed AST edited by `beforeCodegen` plugins.
///
/// This stage does not rerun transforms or semantic checks. Printing still uses
/// the core configuration, including the independent `codegen.minify` setting.
///
/// # Safety
///
/// Sections must satisfy the contracts of [`transform_from_parts`].
pub unsafe fn print_transformed_from_parts(
    parts: [&[u8]; 6],
    options: &str,
) -> Result<Vec<u8>, String> {
    let options = plugin_options(options)?;
    // SAFETY: The caller guarantees valid, immutable encoder sections.
    unsafe {
        with_ast_parts(parts, |ast| {
            let output = print_ast(&ast, &options, Vec::new());
            encode_output(&output, "", 0)
        })
    }
}

fn check_comment_transfer(ast: &AstCtx<'_>, options: &JsTransformApiOptions) -> Result<(), String> {
    if comments_enabled(options) && !ast.comments().is_empty() {
        return Err("Invalid transform options: Comment preservation across JavaScript plugins is unsupported; set comments: \"none\"".into());
    }
    Ok(())
}

fn transform_edited_ast(ast: &mut AstCtx<'_>, options: &JsTransformApiOptions) -> Vec<Diagnostic> {
    let lang = options
        .parser
        .unwrap_or_default()
        .lang
        .map(Into::into)
        .or_else(|| options.filename.as_deref().map(Lang::from_path))
        .unwrap_or_else(|| ast.lang());
    let source_type = ast.source_type();
    // Preserve the configured language (including JSX and declaration files).
    // The encoder infers only JS/TS from nodes; sourceType comes from the edited Program.
    unsafe { ast.ast_mut() }.set_source_metadata(source_type, lang);
    // Source-only checks run before crossing the JavaScript boundary.
    let checks = semantic_checks(options)
        - SyntaxChecks::OCTAL_ESCAPES
        - SyntaxChecks::LEGACY_NUMERIC_LITERALS;
    run_transforms(ast, options, checks)
}

unsafe fn with_ast_parts(
    parts: [&[u8]; 6],
    f: impl FnOnce(BorrowedAst<'_>) -> Vec<u8>,
) -> Result<Vec<u8>, String> {
    let allocator = Allocator::default();
    #[cfg(all(not(target_family = "wasm"), target_endian = "little"))]
    {
        let [header, spans, kinds, payloads, extra, strings] = parts;
        // SAFETY: The caller guarantees valid, immutable encoder sections.
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
        with_joined_parts(parts, |buffer| {
            // SAFETY: The trusted sections now form an aligned transfer buffer.
            let ast = unsafe { swc_next_ecma_ast::ffi::ast::deserialize_from(buffer, &allocator) }
                .map_err(|error| error.to_string())?;
            Ok(f(ast))
        })
    }
}

#[cfg(any(target_family = "wasm", target_endian = "big"))]
fn with_joined_parts<T>(parts: [&[u8]; 6], f: impl FnOnce(&[u8]) -> T) -> T {
    let [header, spans, kinds, payloads, extra, strings] = parts;
    let payload_offset = (header.len() + spans.len() + kinds.len() + 7) & !7;
    let extra_offset = payload_offset + payloads.len();
    let strings_offset = extra_offset + extra.len();
    let length = (strings_offset + strings.len() + 3) & !3;
    // u64 storage preserves the alignment required by the borrowed AST reader.
    let mut words = vec![0_u64; length.div_ceil(8)];
    // SAFETY: Every bit pattern is valid for u64 and the view stays in words.
    let buffer = unsafe { std::slice::from_raw_parts_mut(words.as_mut_ptr().cast::<u8>(), length) };
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
    f(buffer)
}

fn serialize_ast(ast: &AstCtx<'_>, diagnostics: &[Diagnostic], source_utf16_len: u32) -> Vec<u8> {
    let mut output = vec![0; buffer_size(ast, diagnostics)];
    serialize_into(ast, diagnostics, &mut output, source_utf16_len as usize);
    output
}
