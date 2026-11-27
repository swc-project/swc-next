use std::collections::BTreeMap;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{AstCtx, DUMMY_SP, Diagnostic, Lang, SourceType};
use swc_next_ecma_codegen::{Options as GeneratorOptions, generate};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{SyntaxChecks, check_syntax};
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
    js::{
        CommentsOption, JsOutput, JsTransformApiOptions, parser::JsParserConfig,
        serialize_source_map,
    },
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
                JsFeature::ExponentiationOperator => env.es2016.exponentiation = enabled,
                JsFeature::AsyncToGenerator => {
                    env.es2017.async_to_generator = enabled.then(Default::default);
                }
                JsFeature::AsyncGeneratorFunctions => {
                    env.es2017.async_generator_functions = enabled.then(Default::default);
                }
                JsFeature::ObjectRestSpread => {
                    env.es2018.object_rest_spread = enabled.then(Default::default);
                }
                JsFeature::OptionalCatchBinding => env.es2019.optional_catch_binding = enabled,
                JsFeature::ExportNamespaceFrom => env.es2020.export_namespace_from = enabled,
                JsFeature::NullishCoalescingOperator => {
                    env.es2020.nullish_coalescing = enabled.then(Default::default);
                }
                JsFeature::OptionalChaining => {
                    env.es2020.optional_chaining = enabled.then(Default::default);
                }
                JsFeature::LogicalAssignmentOperators => env.es2021.logical_assignments = enabled,
                JsFeature::ClassStaticBlock => env.es2022.static_blocks = enabled,
                JsFeature::ClassProperties => {
                    env.es2022.class_properties = enabled.then(Default::default);
                }
                JsFeature::PrivatePropertyInObject => env.es2022.private_in_object = enabled,
                JsFeature::DotallRegex => env.regexp.dot_all_regex = enabled,
                JsFeature::StickyRegex => env.regexp.sticky_regex = enabled,
                JsFeature::UnicodeRegex => env.regexp.unicode_regex = enabled,
                JsFeature::UnicodeSetsRegex => env.regexp.unicode_sets_regex = enabled,
            }
        }
    }

    Ok(env)
}

fn to_v1_target(target: &JsTarget) -> Result<Target, Diagnostic> {
    match target {
        JsTarget::Query(query) => {
            if matches!(query.as_str(), "es3" | "es5") {
                let mut diagnostic = Diagnostic::error("Unsupported transformer target", DUMMY_SP);
                diagnostic.help = Some(
                    format!(
                        "Unsupported ES edition `{query}`; supported editions are es2015–es2026 and esnext"
                    )
                    .into(),
                );
                return Err(diagnostic);
            }
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

    let decorators: Option<DecoratorOptions> = enabled_options(config.decorators);
    if let Some(decorators) = decorators
        && decorators.metadata
        && decorators.version != DecoratorVersion::Legacy
    {
        let mut diagnostic = Diagnostic::error("Invalid decorator configuration", DUMMY_SP);
        diagnostic.help =
            Some("`decorators.metadata` requires `decorators.version` to be \"legacy\"".into());
        return Err(diagnostic);
    }

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
    let Ok(output) = transform_with_hooks::<TRANSFORM, _>(source, options, &mut NoHooks);
    output
}

pub(super) enum Stage {
    AfterParse,
    BeforeCodegen,
}

pub(super) trait Hooks {
    const HAS_JS_PLUGINS: bool = true;
    type Error;

    fn run_js_plugins<'a>(
        &mut self,
        stage: Stage,
        ast: &mut AstCtx<'a>,
        allocator: &'a Allocator,
    ) -> Result<bool, Self::Error>;
}

struct NoHooks;

impl Hooks for NoHooks {
    const HAS_JS_PLUGINS: bool = false;
    type Error = std::convert::Infallible;

    #[inline(always)]
    fn run_js_plugins(
        &mut self,
        _stage: Stage,
        _ast: &mut AstCtx<'_>,
        _allocator: &Allocator,
    ) -> Result<bool, Self::Error> {
        Ok(false)
    }
}

pub(super) fn transform_with_hooks<const TRANSFORM: bool, H: Hooks>(
    source: &str,
    options: JsTransformApiOptions,
    hooks: &mut H,
) -> Result<JsOutput, H::Error> {
    let parser_options = options.parser.unwrap_or_default();
    let output_comments = options.comments.unwrap_or(CommentsOption::All);
    let allocator = Allocator::default();
    let parsed_options = parser_config::<TRANSFORM>(
        parser_options,
        options.filename.as_deref(),
        output_comments,
        options.transformer.as_ref(),
    );
    let parsed = Parser::init(&allocator, source, parsed_options, NoTokenParserConfig).parse();
    let mut ast = parsed.ast;
    let mut output = JsOutput {
        diagnostics: parsed.diagnostics,
        ..Default::default()
    };
    check_input(&ast, parser_options, &mut output.diagnostics);
    if has_errors(&output.diagnostics) {
        return Ok(output);
    }
    if hooks.run_js_plugins(Stage::AfterParse, &mut ast, &allocator)? {
        check_input(&ast, parser_options, &mut output.diagnostics);
        if has_errors(&output.diagnostics) {
            return Ok(output);
        }
    }
    if TRANSFORM
        && let Err(diagnostic) = transform_ast(
            &mut ast,
            options.transformer,
            options.filename.clone(),
            &mut output.diagnostics,
        )
    {
        output.diagnostics.push(diagnostic);
        return Ok(output);
    }
    if H::HAS_JS_PLUGINS && !has_errors(&output.diagnostics) {
        hooks.run_js_plugins(Stage::BeforeCodegen, &mut ast, &allocator)?;
    }

    // ======================= Generate =======================
    {
        if output_comments == CommentsOption::None {
            // SAFETY: This AST remains owned by the transform call.
            unsafe { ast.ast_mut() }.clear_attached_comments();
        }
        let mut generator = GeneratorOptions::from(options.generator.unwrap_or_default());
        let source_map = options.source_map.unwrap_or(false);
        let filename = options.filename.as_deref().unwrap_or("input.js");
        generator.source_map_path = source_map.then(|| filename.into());
        generator.comments = output_comments == CommentsOption::All;
        let generated = generate(&ast, generator);
        output.code = generated.code;
        output.map = generated.map.as_ref().map(serialize_source_map);
    }

    Ok(output)
}

fn parser_config<const TRANSFORM: bool>(
    config: JsParserConfig,
    filename: Option<&str>,
    comments: CommentsOption,
    transformer: Option<&JsTransformerConfig>,
) -> ParserOptions {
    let lang = config
        .lang
        .map(Into::into)
        .or_else(|| filename.map(Lang::from_path))
        .unwrap_or_default();
    let source_type = config
        .source_type
        .map(Into::into)
        .or_else(|| filename.map(SourceType::from_path))
        .unwrap_or_default();
    let typescript = transform_enabled(
        transformer.and_then(|config| config.typescript.as_ref()),
        lang.is_typescript(),
    );
    let jsx = transform_enabled(
        transformer.and_then(|config| config.jsx.as_ref()),
        lang.is_jsx(),
    );
    let amd = matches!(
        transformer
            .and_then(|config| config.module)
            .and_then(|module| module.r#type),
        Some(JsModuleType::Amd)
    );
    let comments = config.comments.unwrap_or_else(|| {
        if TRANSFORM && (jsx || (typescript && lang.is_jsx()) || amd) {
            CommentsOption::All
        } else {
            comments
        }
    });
    ParserOptions {
        lang,
        source_type,
        comments: comments.into(),
        ..config.into()
    }
}

fn transform_enabled<T>(option: Option<&BoolOr<T>>, default: bool) -> bool {
    match option {
        None => default,
        Some(BoolOr::Bool(value)) => *value,
        Some(BoolOr::Options(_)) => true,
    }
}

fn check_input(ast: &AstCtx<'_>, parser: JsParserConfig, diagnostics: &mut Vec<Diagnostic>) {
    if parser.semantic_errors.unwrap_or(false) && !has_errors(diagnostics) {
        diagnostics.extend(check_syntax(ast, SyntaxChecks::all()));
    }
}

fn transform_ast(
    ast: &mut AstCtx<'_>,
    config: Option<JsTransformerConfig>,
    filename: Option<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), Diagnostic> {
    let mut config = config.unwrap_or_default();
    config
        .typescript
        .get_or_insert(BoolOr::Bool(ast.lang().is_typescript()));
    config.jsx.get_or_insert(BoolOr::Bool(ast.lang().is_jsx()));
    let options = to_options(config, filename)?;
    diagnostics.extend(Transformer::transform(ast, options).diagnostics);
    Ok(())
}
