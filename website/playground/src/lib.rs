use serde::Deserialize;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Lang, Severity, SourceType,
    ffi::ast::{buffer_size, serialize_into},
};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze, check_syntax};
use swc_next_ecma_transformer::{
    Transformer, TransformerOptions, es2022::ES2022Options, normalize::Normalizer,
    proposal::ProposalOptions, typescript::TypeScriptOptions,
};

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    parser: ParserConfig,
    transform: TransformConfig,
    codegen: CodegenConfig,
}

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct ParserConfig {
    lang: String,
    source_type: String,
    preserve_parens: bool,
    semantic_errors: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            lang: "js".into(),
            source_type: "module".into(),
            preserve_parens: true,
            semantic_errors: false,
        }
    }
}

impl ParserConfig {
    fn options(&self) -> Result<ParserOptions, String> {
        let lang = match self.lang.as_str() {
            "js" => Lang::Js,
            "jsx" => Lang::Jsx,
            "ts" => Lang::Ts,
            "tsx" => Lang::Tsx,
            "dts" => Lang::Dts,
            _ => return Err(format!("Unknown parser language: {}", self.lang)),
        };
        let source_type = match self.source_type.as_str() {
            "module" => SourceType::Module,
            "script" => SourceType::Script,
            "commonjs" => SourceType::CommonJs,
            "unambiguous" => SourceType::Unambiguous,
            _ => return Err(format!("Unknown source type: {}", self.source_type)),
        };
        Ok(ParserOptions {
            lang,
            source_type,
            preserve_parens: self.preserve_parens,
            ..ParserOptions::default()
        })
    }
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct TransformConfig {
    ts: bool,
    es2022: ES2022Config,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct ES2022Config {
    class_static_block: bool,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct CodegenConfig {
    minify: bool,
    ascii_only: bool,
    inline_script: bool,
}

/// Run the playground pipeline. The result contains a little-endian AST byte
/// length, the original parser AST and diagnostics padded to four bytes, and
/// UTF-8 generated code. AST spans refer to the original source in UTF-16 units.
#[cfg_attr(target_family = "wasm", swc_next_bridge::bridge)]
pub fn run(source: String, options: String) -> Result<Vec<u8>, String> {
    let options: Options = serde_json::from_str(&options).map_err(|error| error.to_string())?;
    let parser = options.parser.options()?;
    let allocator = Allocator::default();
    let parsed = Parser::init(&allocator, &source, parser, NoTokenParserConfig).parse();
    let mut ast = parsed.ast;
    let mut diagnostics = parsed.diagnostics;
    let typescript = options.transform.ts && parser.lang.is_typescript();
    let static_blocks = options.transform.es2022.class_static_block;
    let transform = typescript || static_blocks;

    // Transforms require valid scopes even when optional semantic checks are off.
    if options.parser.semantic_errors || transform {
        diagnostics.extend(check_syntax(&ast, SyntaxChecks::all()));
    }
    let has_errors = diagnostics
        .iter()
        .any(|item| item.severity == Severity::Error);
    let ast_length = buffer_size(&ast, &diagnostics);
    let aligned_length = (ast_length + 3) & !3;
    let mut result = vec![0; 4 + aligned_length];
    result[..4].copy_from_slice(&(ast_length as u32).to_le_bytes());
    serialize_into(
        &ast,
        &diagnostics,
        &mut result[4..4 + ast_length],
        source.encode_utf16().count(),
    );

    if has_errors {
        return Ok(result);
    }
    if transform {
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
                es2022: ES2022Options { static_blocks },
                proposal: ProposalOptions {
                    prevent_extensions: true,
                },
                ..TransformerOptions::default()
            },
        );
    }
    let output = print(
        &ast,
        CodegenOptions {
            minify: options.codegen.minify,
            ascii_only: options.codegen.ascii_only,
            inline_script: options.codegen.inline_script,
            ..CodegenOptions::default()
        },
    );
    result.extend_from_slice(output.code.as_bytes());
    Ok(result)
}
