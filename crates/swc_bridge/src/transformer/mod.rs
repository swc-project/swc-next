mod options;
mod options_wire;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Lang, Severity,
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
    let mut ast = unsafe { deserialize_from(buffer, &allocator) }?.into_owned();
    let options = options.unwrap_or_default();
    let typescript = options.typescript.unwrap_or(ast.lang().is_typescript());
    // Literal raw spellings were checked before crossing the JS boundary. The
    // transfer retains UTF-16 spans but no source text to inspect at those spans.
    let checks =
        SyntaxChecks::all() - SyntaxChecks::OCTAL_ESCAPES - SyntaxChecks::LEGACY_NUMERIC_LITERALS;
    let diagnostics = check_syntax(&ast, checks);

    if !diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        if typescript {
            Normalizer::normalize_runtime(ast.root_program(), &mut ast);
        } else {
            Normalizer::normalize(ast.root_program(), &mut ast);
        }
        let analyzed = analyze(&ast, AnalyzeOptions::default());
        Transformer::transform(
            &mut ast,
            analyzed.semantic,
            TransformerOptions {
                typescript: typescript.then(TypeScriptOptions::default),
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

    let mut output = vec![0; buffer_size(&ast, &diagnostics)];
    serialize_into(&ast, &diagnostics, &mut output, 0);
    Ok(output)
}
