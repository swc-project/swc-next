use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Diagnostic, Lang, SourceType};
use swc_next_ecma_codegen::{Options as GeneratorOptions, generate};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze};
use swc_next_ecma_transformer::{
    Transformer, TransformerOptions,
    assumptions::Assumptions,
    jsx::{JsxOptions, JsxRuntime as TransformJsxRuntime, ReactRefreshOptions},
    normalize::Normalizer,
    typescript::{JsxImportOptions, TypeScriptOptions},
};

use crate::{
    BoolOr,
    js::{CommentsOption, JsOutput, JsTransformApiOptions, serialize_source_map},
};

use super::{
    JsAssumptions, JsFeature, JsReactRefreshConfig, JsTarget, JsTransformerConfig,
    JsTypeScriptConfig, JsxConfig, JsxRuntime, enabled_options, has_errors,
};

pub(super) fn to_options(
    config: JsTransformerConfig,
    filename: Option<String>,
) -> Result<TransformerOptions, Diagnostic> {
    let mut defaults = TransformerOptions::default();
    if let Some(JsTarget::Query(query)) = config.target.as_ref() {
        match query.as_str() {
            "es2015" | "es2016" | "es2017" | "es2018" => {
                defaults.es2019.optional_catch_binding = true;
                defaults.es2022.static_blocks = true;
            }
            "es2019" | "es2020" | "es2021" => {
                defaults.es2022.static_blocks = true;
            }
            _ => {}
        }
    }
    if let Some(features) = config.features {
        for (feature, enabled) in features {
            match feature {
                JsFeature::ClassStaticBlock => defaults.es2022.static_blocks = enabled,
                JsFeature::OptionalCatchBinding => defaults.es2019.optional_catch_binding = enabled,
                feature => unimplemented!("transformer v2 does not support feature `{feature:?}`"),
            }
        }
    }
    Ok(TransformerOptions {
        filename: config.filename.or(filename),
        assumptions: config
            .assumptions
            .map(Into::into)
            .unwrap_or(defaults.assumptions),
        typescript: enabled_options(config.typescript),
        jsx: enabled_options(config.jsx),
        ..defaults
    })
}

/// Transform a source file using the v2 transformer pipeline.
///
/// Set `TRANSFORM` to `false` to skip native syntax transformations.
pub fn transform_js<const TRANSFORM: bool>(
    source: &str,
    options: JsTransformApiOptions,
) -> JsOutput {
    let mut output = JsOutput::default();

    // ======================= Options =======================
    let parser_options = options.parser.unwrap_or_default();
    let comments = parser_options
        .comments
        .or(options.comments)
        .unwrap_or(CommentsOption::All);
    let lang = {
        let inferred = options.filename.as_deref().map(Lang::from_path);
        let explicit = parser_options.lang.map(Into::into);
        explicit.or(inferred).unwrap_or_default()
    };
    let transform = {
        let mut transform = options.transformer.unwrap_or_default();
        let typescript = BoolOr::Bool(lang.is_typescript());
        let jsx = BoolOr::Bool(lang.is_jsx());
        transform.typescript.get_or_insert(typescript);
        transform.jsx.get_or_insert(jsx);
        transform
    };

    // ======================= Parse =======================
    let allocator = Allocator::default();
    let mut ast = {
        let jsx_enabled = !matches!(&transform.jsx, None | Some(BoolOr::Bool(false)));
        let typescript_enabled = !matches!(&transform.typescript, None | Some(BoolOr::Bool(false)));
        let collect_pragmas = TRANSFORM
            && parser_options.comments.is_none()
            && (jsx_enabled || (typescript_enabled && lang.is_jsx()));
        let parser_comments = if collect_pragmas {
            CommentsOption::All
        } else {
            comments
        };
        let source_type = {
            let inferred = options.filename.as_deref().map(SourceType::from_path);
            let explicit = parser_options.source_type.map(Into::into);
            explicit.or(inferred).unwrap_or_default()
        };
        let parser_options = ParserOptions {
            source_type,
            lang,
            comments: parser_comments.into(),
            ..parser_options.into()
        };
        let parser = Parser::init(&allocator, source, parser_options, NoTokenParserConfig);
        let parsed = parser.parse();
        output.diagnostics.extend(parsed.diagnostics);
        if has_errors(&output.diagnostics) {
            return output;
        }
        parsed.ast
    };

    // ======================= Normalize =======================
    if TRANSFORM {
        match &transform.typescript {
            None | Some(BoolOr::Bool(false)) => Normalizer::normalize(&mut ast),
            _ => Normalizer::normalize_runtime(&mut ast),
        }
    }

    // ======================= Analyze and Transform =======================
    let semantic_errors = parser_options.semantic_errors.unwrap_or(false);
    if TRANSFORM || semantic_errors {
        let check_syntax = if semantic_errors {
            SyntaxChecks::all()
        } else {
            SyntaxChecks::empty()
        };
        let analyzed = analyze(&mut ast, AnalyzeOptions { check_syntax });
        output.diagnostics.extend(analyzed.diagnostics);
        if has_errors(&output.diagnostics) {
            return output;
        }
        if TRANSFORM {
            match to_options(transform, options.filename.clone()) {
                Ok(transform_options) => {
                    let transformed = Transformer::transform(analyzed.semantic, transform_options);
                    output.diagnostics.extend(transformed.diagnostics);
                }
                Err(diagnostic) => {
                    output.diagnostics.push(diagnostic);
                    return output;
                }
            }
        }
    }

    // ======================= Generate =======================
    {
        if comments == CommentsOption::None {
            // SAFETY: This AST remains owned by the transform call.
            unsafe { ast.ast_mut() }.clear_attached_comments();
        }
        let mut generator = GeneratorOptions::from(options.generator.unwrap_or_default());
        let source_map = options.source_map.unwrap_or(false);
        let filename = options.filename.as_deref().unwrap_or("input.js");
        generator.source_map_path = source_map.then(|| filename.into());
        generator.comments = options.comments.unwrap_or(CommentsOption::All) == CommentsOption::All;
        let generated = generate(&ast, generator);
        output.code = generated.code;
        output.map = generated.map.as_ref().map(serialize_source_map);
    }

    output
}

impl From<JsTypeScriptConfig> for TypeScriptOptions {
    fn from(options: JsTypeScriptConfig) -> Self {
        let defaults = Self::default();
        Self {
            verbatim_module_syntax: options
                .verbatim_module_syntax
                .unwrap_or(defaults.verbatim_module_syntax),
            preserve_const_enums: options
                .preserve_const_enums
                .unwrap_or(defaults.preserve_const_enums),
            jsx: JsxImportOptions {
                automatic: options.jsx_runtime == Some(JsxRuntime::Automatic),
                pragma: options.jsx_pragma,
                pragma_frag: options.jsx_pragma_frag,
            },
        }
    }
}

impl From<JsxRuntime> for TransformJsxRuntime {
    fn from(runtime: JsxRuntime) -> Self {
        match runtime {
            JsxRuntime::Classic => Self::Classic,
            JsxRuntime::Automatic => Self::Automatic,
        }
    }
}

impl From<JsxConfig> for JsxOptions {
    fn from(options: JsxConfig) -> Self {
        let defaults = Self::default();
        Self {
            runtime: options.runtime.map(Into::into).unwrap_or(defaults.runtime),
            development: options.development.unwrap_or(defaults.development),
            source_self: options.source_self.unwrap_or(defaults.source_self),
            import_source: options.import_source,
            pragma: options.pragma,
            pragma_frag: options.pragma_frag,
            throw_if_namespace: options
                .throw_if_namespace
                .unwrap_or(defaults.throw_if_namespace),
            pure: options.pure,
            display_name: options.display_name.unwrap_or(defaults.display_name),
            refresh: enabled_options(options.refresh),
        }
    }
}

impl From<JsReactRefreshConfig> for ReactRefreshOptions {
    fn from(options: JsReactRefreshConfig) -> Self {
        let defaults = Self::default();
        Self {
            refresh_reg: options.refresh_reg.unwrap_or(defaults.refresh_reg),
            refresh_sig: options.refresh_sig.unwrap_or(defaults.refresh_sig),
            emit_full_signatures: options
                .emit_full_signatures
                .unwrap_or(defaults.emit_full_signatures),
        }
    }
}

impl From<JsAssumptions> for Assumptions {
    fn from(options: JsAssumptions) -> Self {
        let mut assumptions = Self::default();
        assumptions.constant_super = options.constant_super.unwrap_or(assumptions.constant_super);
        assumptions.ignore_function_length = options
            .ignore_function_length
            .unwrap_or(assumptions.ignore_function_length);
        assumptions.no_document_all = options
            .no_document_all
            .unwrap_or(assumptions.no_document_all);
        assumptions.object_rest_no_symbols = options
            .object_rest_no_symbols
            .unwrap_or(assumptions.object_rest_no_symbols);
        assumptions.private_fields_as_properties = options
            .private_fields_as_properties
            .unwrap_or(assumptions.private_fields_as_properties);
        assumptions.pure_getters = options.pure_getters.unwrap_or(assumptions.pure_getters);
        assumptions.set_public_class_fields = options
            .set_public_class_fields
            .unwrap_or(assumptions.set_public_class_fields);
        assumptions.set_spread_properties = options
            .set_spread_properties
            .unwrap_or(assumptions.set_spread_properties);
        assumptions
    }
}
