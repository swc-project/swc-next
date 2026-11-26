use std::collections::BTreeMap;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{DUMMY_SP, Diagnostic, Lang, SourceType};
use swc_next_ecma_codegen::{Options as GeneratorOptions, generate};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze};
use swc_next_ecma_transformer_v1::{
    Transformer, TransformerOptions,
    decorators::{DecoratorOptions, DecoratorVersion},
    env::{Engine, EnvConfig, EnvOptions, EsVersion, Target},
    es2022::ClassFieldsUseSetOptions,
    jsx::{JsxOptions, JsxRuntime as V1JsxRuntime, ReactRefreshOptions},
    module::{ModuleOptions, ModuleType},
    typescript::{JsxImportOptions, TsImportExportAssignConfig, TypeScriptOptions},
};

use crate::{
    BoolOr,
    js::{CommentsOption, JsOutput, JsTransformApiOptions, serialize_source_map},
};

use super::{
    JsAssumptions, JsDecoratorConfig, JsDecoratorVersion, JsFeature, JsModuleConfig, JsModuleType,
    JsReactRefreshConfig, JsTarget, JsTransformerConfig, JsTypeScriptConfig, JsxConfig, JsxRuntime,
    enabled_options, has_errors,
};

fn target_diagnostic(error: swc_next_ecma_transformer_v1::env::TargetError) -> Diagnostic {
    let mut diagnostic = Diagnostic::error("Invalid transformer target", DUMMY_SP);
    diagnostic.help = Some(error.to_string().into());
    diagnostic
}

fn resolve_env(
    target: Option<&JsTarget>,
    features: Option<&BTreeMap<JsFeature, bool>>,
) -> Result<EnvOptions, Diagnostic> {
    let mut env = if let Some(target) = target {
        let target = to_v1_target(target)?;
        EnvConfig {
            target: Some(target),
            ..Default::default()
        }
        .to_env_options()
        .map_err(target_diagnostic)?
    } else {
        EnvOptions::default()
    };

    if let Some(features) = features {
        for (&feature, &enabled) in features {
            match feature {
                JsFeature::ClassStaticBlock => env.es2022.static_blocks = enabled,
                JsFeature::OptionalCatchBinding => env.es2019.optional_catch_binding = enabled,
            }
        }
    }

    Ok(env)
}

fn to_v1_target(target: &JsTarget) -> Result<Target, Diagnostic> {
    match target {
        JsTarget::Query(query) => {
            if let Ok(version) = query.parse::<EsVersion>() {
                Ok(Target::EsVersion(version))
            } else {
                Ok(Target::Browserslist(vec![query.clone()]))
            }
        }
        JsTarget::Queries(queries) => Ok(Target::Browserslist(queries.clone())),
        JsTarget::Engines(engines) => {
            let mut parsed = BTreeMap::new();
            for (name, version) in engines {
                let engine: Engine = name.parse().map_err(target_diagnostic)?;
                parsed.insert(engine, version.clone());
            }
            Ok(Target::Engines(parsed))
        }
    }
}

pub(super) fn to_options(
    config: JsTransformerConfig,
    filename: Option<String>,
) -> Result<TransformerOptions, Diagnostic> {
    let mut env = resolve_env(config.target.as_ref(), config.features.as_ref())?;
    let assumptions = config.assumptions.unwrap_or_default();
    apply_assumptions(&mut env, assumptions);

    let jsx: Option<JsxOptions> = enabled_options(config.jsx);
    let module = config.module.map(to_module_options).unwrap_or_default();

    let set_public_fields = assumptions.set_public_class_fields.unwrap_or(false);
    let native_class_properties = !set_public_fields && env.es2022.class_properties.is_none();

    let mut typescript: Option<TypeScriptOptions> = enabled_options(config.typescript);
    if let Some(ts) = &mut typescript {
        ts.native_class_properties = native_class_properties;
        ts.import_export_assign_config = match module.r#type {
            ModuleType::Preserve => TsImportExportAssignConfig::EsNext,
            ModuleType::CommonJs | ModuleType::Amd | ModuleType::Umd | ModuleType::SystemJs => {
                TsImportExportAssignConfig::Preserve
            }
        };
    }

    let decorators = enabled_options(config.decorators);

    Ok(TransformerOptions {
        filename: config.filename.or(filename),
        decorators,
        env,
        jsx,
        typescript,
        module,
    })
}

fn apply_assumptions(env: &mut EnvOptions, assumptions: JsAssumptions) {
    let ignore_function_length = assumptions.ignore_function_length.unwrap_or(false);
    if let Some(opts) = &mut env.es2017.async_to_generator {
        opts.ignore_function_length = ignore_function_length;
    }
    if let Some(opts) = &mut env.es2017.async_generator_functions {
        opts.ignore_function_length = ignore_function_length;
    }

    if let Some(opts) = &mut env.es2018.object_rest_spread {
        opts.no_symbol = assumptions.object_rest_no_symbols.unwrap_or(false);
        opts.set_property = assumptions.set_spread_properties.unwrap_or(false);
        opts.pure_getters = assumptions.pure_getters.unwrap_or(false);
    }

    if let Some(opts) = &mut env.es2020.nullish_coalescing {
        opts.no_document_all = assumptions.no_document_all.unwrap_or(false);
    }
    if let Some(opts) = &mut env.es2020.optional_chaining {
        opts.no_document_all = assumptions.no_document_all.unwrap_or(false);
        opts.pure_getter = assumptions.pure_getters.unwrap_or(false);
    }

    let set_public_fields = assumptions.set_public_class_fields.unwrap_or(false);
    let pure_getters = assumptions.pure_getters.unwrap_or(false);
    if set_public_fields {
        env.es2022.class_fields_use_set = Some(ClassFieldsUseSetOptions { pure_getters });
    }
    if let Some(opts) = &mut env.es2022.class_properties {
        opts.private_as_properties = assumptions.private_fields_as_properties.unwrap_or(false);
        opts.set_public_fields = set_public_fields;
        opts.constant_super = assumptions.constant_super.unwrap_or(false);
        opts.no_document_all = assumptions.no_document_all.unwrap_or(false);
        opts.pure_getter = pure_getters;
    }
}

impl From<JsTypeScriptConfig> for TypeScriptOptions {
    fn from(options: JsTypeScriptConfig) -> Self {
        Self {
            verbatim_module_syntax: options.verbatim_module_syntax.unwrap_or(false),
            jsx: JsxImportOptions {
                automatic: options.jsx_runtime == Some(JsxRuntime::Automatic),
                pragma: options.jsx_pragma,
                pragma_frag: options.jsx_pragma_frag,
            },
            ..Default::default()
        }
    }
}

fn to_module_options(options: JsModuleConfig) -> ModuleOptions {
    ModuleOptions {
        r#type: match options.r#type.unwrap_or_default() {
            JsModuleType::Preserve => ModuleType::Preserve,
            JsModuleType::CommonJs => ModuleType::CommonJs,
            JsModuleType::Amd => ModuleType::Amd,
            JsModuleType::Umd => ModuleType::Umd,
            JsModuleType::SystemJs => ModuleType::SystemJs,
        },
    }
}

impl From<JsxRuntime> for V1JsxRuntime {
    fn from(runtime: JsxRuntime) -> Self {
        match runtime {
            JsxRuntime::Classic => Self::Classic,
            JsxRuntime::Automatic => Self::Automatic,
        }
    }
}

impl From<JsxConfig> for JsxOptions {
    fn from(options: JsxConfig) -> Self {
        Self {
            runtime: Some(options.runtime.unwrap_or_default().into()),
            development: Some(options.development.unwrap_or(false)),
            source_self: Some(options.source_self.unwrap_or(false)),
            pure: options.pure,
            display_name: Some(options.display_name.unwrap_or(false)),
            import_source: options.import_source,
            pragma: options.pragma,
            pragma_frag: options.pragma_frag,
            throw_if_namespace: Some(options.throw_if_namespace.unwrap_or(true)),
            refresh: enabled_options(options.refresh),
            ..Default::default()
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

impl From<JsDecoratorConfig> for DecoratorOptions {
    fn from(options: JsDecoratorConfig) -> Self {
        Self {
            version: match options.version.unwrap_or_default() {
                JsDecoratorVersion::V2023_11 => DecoratorVersion::V2023_11,
                JsDecoratorVersion::Legacy => DecoratorVersion::Legacy,
            },
            metadata: options.metadata.unwrap_or(false),
        }
    }
}

/// Transform a source file using the v1 transformer pipeline.
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

    // ======================= Analyze and Transform =======================
    let semantic_errors = parser_options.semantic_errors.unwrap_or(false);
    if semantic_errors {
        let analyzed = analyze(
            &mut ast,
            AnalyzeOptions {
                check_syntax: SyntaxChecks::all(),
            },
        );
        output.diagnostics.extend(analyzed.diagnostics);
        if has_errors(&output.diagnostics) {
            return output;
        }
    }
    if TRANSFORM {
        match to_options(transform, options.filename.clone()) {
            Ok(transform_options) => {
                let transformed = Transformer::transform(&mut ast, transform_options);
                output.diagnostics.extend(transformed.diagnostics);
            }
            Err(diagnostic) => {
                output.diagnostics.push(diagnostic);
                return output;
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
