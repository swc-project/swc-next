mod options;
mod options_wire;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, Diagnostic, Lang, Severity,
    ffi::ast::{DeserializeError, buffer_size, deserialize_from, serialize_into},
};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze, check_syntax};
use swc_next_ecma_transformer::{
    Transformer, TransformerOptions, es2022::ES2022Options, normalize::Normalizer,
    typescript::TypeScriptOptions,
};

pub use self::options::TransformPassOptions;

/// Transform a trusted AST transfer buffer and serialize its AST and diagnostics.
///
/// The original source must have passed parser and semantic syntax checks.
/// Edited ASTs are checked again, except for source-only octal escape and legacy
/// numeric literal checks. Comments and comment-based JSX pragmas are absent
/// from the transfer. The returned buffer uses an empty source and UTF-16 spans.
///
/// # Safety
///
/// `buffer` must be produced by `@swc-next/encoder` matching the version of this
/// crate and follow the [AST transfer protocol](https://github.com/swc-project/swc-next/blob/main/crates/swc_ecma_ast/src/ffi/ast.rs).
/// It must satisfy the alignment requirements of [`deserialize_from`] and remain
/// immutable throughout this call.
pub unsafe fn transform_from_buffer(
    buffer: &[u8],
    options: Option<TransformPassOptions>,
) -> Result<Vec<u8>, DeserializeError> {
    let allocator = Allocator::default();
    // SAFETY: The caller guarantees a valid, immutable AST transfer buffer.
    let (ast, diagnostics) = unsafe { transform_buffer(&allocator, buffer, options) }?;
    Ok(serialize_ast(&ast, &diagnostics, 0))
}

/// Parse and transform before transferring the AST to JavaScript.
///
/// The returned AST buffer refers to the original source for its strings and
/// uses UTF-16 spans. The JavaScript decoder must receive the original source.
/// Parser and semantic errors stop transformation. Comments and comment-based
/// JSX pragmas are omitted, matching the other transform entry points.
///
/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
/// `source_utf16_len` must match the UTF-16 code unit length of `source`.
#[cfg(feature = "parser")]
pub unsafe fn parse_transform_to_buffer(
    source: &[u8],
    source_utf16_len: u32,
    parser_options: Option<crate::ParserOptions>,
    transform_options: Option<TransformPassOptions>,
) -> Vec<u8> {
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    let allocator = Allocator::default();
    let (ast, diagnostics) =
        parse_and_transform(&allocator, source, parser_options, transform_options);
    serialize_ast(&ast, &diagnostics, source_utf16_len)
}

/// Transform and print a trusted AST transfer buffer without returning an AST.
///
/// Uses the same AST validation as [`transform_from_buffer`]. The result starts
/// with little-endian `u32` code byte length and diagnostic count, followed by
/// UTF-8 code bytes and packed diagnostic records. Diagnostic spans remain
/// UTF-16 offsets from the transfer.
///
/// # Safety
///
/// `buffer` must be produced by `@swc-next/encoder` matching the version of this
/// crate and follow the [AST transfer protocol](https://github.com/swc-project/swc-next/blob/main/crates/swc_ecma_ast/src/ffi/ast.rs).
/// It must satisfy the alignment requirements of [`deserialize_from`] and remain
/// immutable throughout this call.
#[cfg(feature = "codegen")]
pub unsafe fn transform_print_from_buffer(
    buffer: &[u8],
    options: Option<TransformPassOptions>,
    codegen_options: Option<crate::CodegenOptions>,
) -> Result<Vec<u8>, DeserializeError> {
    let allocator = Allocator::default();
    // SAFETY: The caller guarantees a valid, immutable AST transfer buffer.
    let (ast, diagnostics) = unsafe { transform_buffer(&allocator, buffer, options) }?;
    Ok(print_to_buffer(&ast, &diagnostics, codegen_options, 0))
}

/// Parse, transform, and print without transferring the AST to JavaScript.
///
/// The result starts with two little-endian `u32` values: the UTF-8 code byte
/// length and diagnostic count. The code bytes are followed by packed records
/// from [`swc_next_ecma_ast::ffi::ast::serialize_diagnostics_into`]. Errors stop
/// before transformation and produce empty code. Only diagnostic spans are
/// converted to UTF-16; all AST spans remain byte offsets throughout the pipeline.
/// Comments and comment-based JSX pragmas are omitted, matching AST transfers.
///
/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
/// `source_utf16_len` must match the UTF-16 code unit length of `source`.
#[cfg(all(feature = "parser", feature = "codegen"))]
pub unsafe fn transform_source_to_buffer(
    source: &[u8],
    source_utf16_len: u32,
    parser_options: Option<crate::ParserOptions>,
    transform_options: Option<TransformPassOptions>,
    codegen_options: Option<crate::CodegenOptions>,
) -> Vec<u8> {
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    let allocator = Allocator::default();
    let (ast, diagnostics) =
        parse_and_transform(&allocator, source, parser_options, transform_options);
    print_to_buffer(&ast, &diagnostics, codegen_options, source_utf16_len)
}

#[cfg(feature = "parser")]
fn parse_and_transform<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    parser_options: Option<crate::ParserOptions>,
    transform_options: Option<TransformPassOptions>,
) -> (Ast<'a>, Vec<Diagnostic<'a>>) {
    use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Parser};

    let mut parser_options: swc_next_ecma_parser::Options =
        parser_options.unwrap_or_default().into();
    parser_options.comments = CommentMode::None;
    let parsed = Parser::init(allocator, source, parser_options, NoTokenParserConfig).parse();
    let mut ast = parsed.ast;
    let mut diagnostics = parsed.diagnostics;
    diagnostics.extend(check_syntax(&ast, SyntaxChecks::all()));

    if !has_errors(&diagnostics) {
        run_passes(&mut ast, transform_options.unwrap_or_default());
    }
    (ast, diagnostics)
}

unsafe fn transform_buffer<'a>(
    allocator: &'a Allocator,
    buffer: &'a [u8],
    options: Option<TransformPassOptions>,
) -> Result<(Ast<'a>, Vec<Diagnostic<'a>>), DeserializeError> {
    // SAFETY: The caller guarantees a valid, immutable AST transfer buffer.
    let mut ast = unsafe { deserialize_from(buffer, allocator) }?.into_owned();
    // Literal raw spellings were checked before crossing the JS boundary. The
    // transfer retains UTF-16 spans but no source text to inspect at those spans.
    let checks =
        SyntaxChecks::all() - SyntaxChecks::OCTAL_ESCAPES - SyntaxChecks::LEGACY_NUMERIC_LITERALS;
    let diagnostics = check_syntax(&ast, checks);
    if !has_errors(&diagnostics) {
        run_passes(&mut ast, options.unwrap_or_default());
    }
    Ok((ast, diagnostics))
}

fn serialize_ast(ast: &Ast<'_>, diagnostics: &[Diagnostic<'_>], source_utf16_len: u32) -> Vec<u8> {
    let mut output = vec![0; buffer_size(ast, diagnostics)];
    serialize_into(ast, diagnostics, &mut output, source_utf16_len as usize);
    output
}

#[cfg(feature = "codegen")]
fn print_to_buffer(
    ast: &Ast<'_>,
    diagnostics: &[Diagnostic<'_>],
    options: Option<crate::CodegenOptions>,
    source_utf16_len: u32,
) -> Vec<u8> {
    use swc_next_ecma_ast::ffi::ast::{diagnostics_buffer_size, serialize_diagnostics_into};
    use swc_next_ecma_codegen::print;

    let code = if has_errors(diagnostics) {
        &[][..]
    } else {
        print(ast.allocator(), ast, options.unwrap_or_default().into())
            .code
            .as_bytes()
    };

    let code_len = u32::try_from(code.len()).expect("generated code length exceeds u32");
    let diagnostic_count = u32::try_from(diagnostics.len()).expect("diagnostic count exceeds u32");
    let mut output = vec![0; 8 + code.len() + diagnostics_buffer_size(diagnostics)];
    output[..4].copy_from_slice(&code_len.to_le_bytes());
    output[4..8].copy_from_slice(&diagnostic_count.to_le_bytes());
    output[8..8 + code.len()].copy_from_slice(code);
    serialize_diagnostics_into(
        diagnostics,
        ast.source(),
        &mut output[8 + code.len()..],
        source_utf16_len as usize,
    );
    output
}

fn has_errors(diagnostics: &[Diagnostic<'_>]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
}

fn run_passes(ast: &mut Ast<'_>, options: TransformPassOptions) {
    let typescript = options.typescript.unwrap_or(ast.lang().is_typescript());
    if typescript {
        Normalizer::normalize_runtime(ast.root_program(), ast);
    } else {
        Normalizer::normalize(ast.root_program(), ast);
    }
    let analyzed = analyze(ast, AnalyzeOptions::default());
    Transformer::transform(
        ast,
        analyzed.semantic,
        TransformerOptions {
            typescript: typescript.then(|| TypeScriptOptions {
                verbatim_module_syntax: options.verbatim_module_syntax.unwrap_or_default(),
                preserve_const_enums: options.preserve_const_enums.unwrap_or_default(),
                ..TypeScriptOptions::default()
            }),
            es2022: ES2022Options {
                static_blocks: options.static_blocks.unwrap_or_default(),
            },
            ..TransformerOptions::default()
        },
    );
    if typescript {
        let lang = if ast.lang().is_jsx() {
            Lang::Jsx
        } else {
            Lang::Js
        };
        ast.set_source_metadata(ast.source_type(), lang);
    }
}
