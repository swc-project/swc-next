use std::{collections::BTreeMap, io, path::Path};

use jsonc_parser::{ParseOptions, parse_to_serde_value};
use serde_json::Value;
use swc_core::{
    base::config::{
        Config, FileMatcher, InputSourceMap, IsModule, ModuleConfig, OutputCharset, Rc,
        SourceMapsConfig, TransformConfig,
    },
    ecma::{
        ast::EsVersion as SwcEsVersion,
        parser::Syntax,
        preset_env::{Feature, FeatureOrModule},
        transforms::react::Runtime,
    },
};
use swc_next::{
    BoolOr,
    js::{
        CommentsOption, JsTransformApiOptions,
        generator::JsGeneratorConfig,
        parser::JsParserConfig,
        transformer::{
            JsDecoratorConfig, JsDecoratorVersion, JsFeature, JsModuleConfig, JsModuleType,
            JsReactRefreshConfig, JsTarget, JsTransformerConfig, JsTypeScriptConfig, JsxConfig,
            JsxRuntime,
        },
    },
};
use swc_next_ecma_ast::{Lang, SourceType};

use crate::{cases::assumptions, util::read_optional};

#[derive(Debug)]
pub struct SwcCaseConfig {
    pub config: Config,
    pub env_target: Option<JsTarget>,
}

/// Select the nearest .swcrc; parent configurations are not merged.
pub fn from_test_path(path: &Path, root: &Path) -> io::Result<Option<SwcCaseConfig>> {
    for dir in path.parent().unwrap().ancestors() {
        if let Some(source) = read_optional(&dir.join(".swcrc"))? {
            return Ok(from_source(&source, path));
        }
        if dir == root {
            break;
        }
    }
    Ok(Rc::default()
        .into_config(Some(path))
        .ok()
        .flatten()
        .map(|config| SwcCaseConfig {
            config,
            env_target: None,
        }))
}

fn parse_env_target(value: &Value) -> Option<JsTarget> {
    let raw = value.get("env")?.get("targets")?;
    if raw.is_null() {
        return None;
    }
    if let Some(map) = raw.as_object()
        && map.get("esmodules").and_then(Value::as_bool) == Some(true)
        && map.len() == 1
    {
        return Some(JsTarget::Query("supports es6-module".into()));
    }
    serde_json::from_value(raw.clone()).ok()
}

fn from_source(source: &str, path: &Path) -> Option<SwcCaseConfig> {
    let source = source.trim_start_matches('\u{feff}');
    let options = ParseOptions {
        allow_comments: true,
        allow_trailing_commas: true,
        allow_loose_object_property_names: false,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
        allow_bare_decimal_point_numbers: false,
        allow_non_finite_numbers: false,
        allow_extended_string_escapes: false,
    };
    let value: Value = parse_to_serde_value(source, &options).ok()?;
    let values = match value {
        Value::Array(values) => values,
        value => vec![value],
    };
    for value in values {
        // Select before decoding options so an unsupported, unselected branch
        // does not hide a supported configuration.
        if let Some(test) = value.get("test").filter(|v| !v.is_null()) {
            let matcher: FileMatcher = serde_json::from_value(test.clone()).ok()?;
            if !matcher.matches(path).ok()? {
                continue;
            }
        }
        if let Some(exclude) = value.get("exclude").filter(|v| !v.is_null()) {
            let matcher: FileMatcher = serde_json::from_value(exclude.clone()).ok()?;
            if matcher.matches(path).ok()? {
                continue;
            }
        }
        let env_target = parse_env_target(&value);
        let mut config: Config = serde_json::from_value(value).ok()?;
        config.adjust(path);
        return Some(SwcCaseConfig { config, env_target });
    }
    None
}

pub fn source_type(config: &Config) -> SourceType {
    match config.is_module.unwrap_or_default() {
        IsModule::Bool(false) => SourceType::Script,
        IsModule::Bool(true) => SourceType::Module,
        IsModule::Unknown => SourceType::Unambiguous,
        IsModule::CommonJS => SourceType::CommonJs,
    }
}

pub fn lang(config: &Config) -> Lang {
    match config.jsc.syntax.unwrap_or_default() {
        Syntax::Typescript(parser) if parser.dts => Lang::Dts,
        Syntax::Typescript(parser) if parser.tsx => Lang::Tsx,
        Syntax::Typescript(_) => Lang::Ts,
        Syntax::Es(parser) if parser.jsx => Lang::Jsx,
        _ => Lang::Js,
    }
}

fn map_es_version(version: SwcEsVersion) -> Option<&'static str> {
    match version {
        // SwcEsVersion::Es3 => Some("es3"),
        // SwcEsVersion::Es5 => Some("es5"),
        SwcEsVersion::Es3 | SwcEsVersion::Es5 => None,
        SwcEsVersion::Es2015 => Some("es2015"),
        SwcEsVersion::Es2016 => Some("es2016"),
        SwcEsVersion::Es2017 => Some("es2017"),
        SwcEsVersion::Es2018 => Some("es2018"),
        SwcEsVersion::Es2019 => Some("es2019"),
        SwcEsVersion::Es2020 => Some("es2020"),
        SwcEsVersion::Es2021 => Some("es2021"),
        SwcEsVersion::Es2022 => Some("es2022"),
        SwcEsVersion::Es2023 => Some("es2023"),
        SwcEsVersion::Es2024 => Some("es2024"),
        SwcEsVersion::EsNext => Some("esnext"),
    }
}

fn map_decorators(config: &Config) -> Option<Option<JsDecoratorConfig>> {
    let syntax = config.jsc.syntax.unwrap_or_default();
    if !syntax.decorators() {
        return Some(None);
    }
    let default_transform = TransformConfig::default();
    let transform = config.jsc.transform.as_ref().unwrap_or(&default_transform);
    let version = transform
        .decorator_version
        .map(|version| format!("{version:?}"));
    match version.as_deref() {
        None | Some("V202112") if transform.legacy_decorator.into_bool() => {
            Some(Some(JsDecoratorConfig {
                version: Some(JsDecoratorVersion::Legacy),
                metadata: Some(transform.decorator_metadata.into_bool()),
            }))
        }
        Some("V202311") => Some(Some(JsDecoratorConfig {
            version: Some(JsDecoratorVersion::V2023_11),
            metadata: Some(false),
        })),
        _ => None,
    }
}

fn map_module(config: &Config) -> Option<Option<JsModuleConfig>> {
    let Some(module) = config.module.as_ref() else {
        return Some(None);
    };
    let value = serde_json::to_value(module).ok()?;
    let map = value.as_object()?;
    if map.len() != 1 {
        return None;
    }
    let kind = match module {
        ModuleConfig::Es6(_) => JsModuleType::Preserve,
        ModuleConfig::CommonJs(_) => JsModuleType::CommonJs,
        ModuleConfig::Amd(_) => JsModuleType::Amd,
        ModuleConfig::Umd(_) => JsModuleType::Umd,
        ModuleConfig::SystemJs(_) => JsModuleType::SystemJs,
        _ => return None,
    };
    Some(Some(JsModuleConfig { r#type: Some(kind) }))
}

pub fn supports_options(case: &SwcCaseConfig) -> bool {
    let config = &case.config;
    let jsc = &config.jsc;
    if jsc.loose.into_bool() || config.env.as_ref().is_some_and(|env| env.loose) {
        return false;
    }
    if let Some(env) = &config.env {
        if jsc.target.is_some()
            || !env
                .include
                .iter()
                .chain(&env.exclude)
                .all(is_supported_feature)
            || env.mode.is_some()
            || env.force_all_transforms
            || env.core_js.is_some()
            || !env.skip.is_empty()
        {
            return false;
        }
        if env.targets.is_some() && case.env_target.is_none() {
            return false;
        }
    } else if map_es_version(jsc.target.unwrap_or_default()).is_none() {
        return false;
    }
    let syntax_supported = match jsc.syntax.unwrap_or_default() {
        Syntax::Es(parser) => {
            !parser.export_default_from && !parser.fn_bind && !parser.explicit_resource_management
        }
        Syntax::Typescript(parser) => !parser.disallow_ambiguous_jsx_like && !parser.no_early_errors,
    };
    if !syntax_supported || map_decorators(config).is_none() || map_module(config).is_none() {
        return false;
    }
    if let Some(transform) = jsc.transform.as_ref()
        && (transform.ts_enum_is_mutable.into_bool()
            || transform.const_modules.is_some()
            || transform.optimizer.is_some()
            || transform.regenerator.import_path.is_some()
            || transform.react_compiler.is_true()
            || transform.react_compiler.is_obj())
    {
        return false;
    }
    let experimental = &jsc.experimental;
    matches!(
        config.source_maps,
        None | Some(SourceMapsConfig::Bool(false))
    ) && matches!(
        config.input_source_map,
        None | Some(InputSourceMap::Bool(false))
    ) && config.source_map_ignore_list.is_none()
        && !jsc.keep_class_names.into_bool()
        && jsc.base_url.as_os_str().is_empty()
        && jsc.paths.is_empty()
        && jsc.minify.is_none()
        && !jsc.rewrite_relative_import_extensions.into_bool()
        && !jsc.preserve_symlinks.into_bool()
        && jsc.output.preamble.is_empty()
        && jsc.output.source_map_url.is_none()
        && !jsc.output.preserve_annotations.into_bool()
        && !experimental.emit_assert_for_import_attributes.into_bool()
        && !experimental.keep_import_attributes.into_bool()
        && !experimental.emit_source_map_scopes.into_bool()
        && !experimental.emit_isolated_dts.into_bool()
        && !experimental
            .disable_builtin_transforms_for_internal_testing
            .into_bool()
        && experimental.plugins.is_none()
        && experimental.plugin_env_vars.is_none()
}

fn is_supported_feature(feature: &FeatureOrModule) -> bool {
    matches!(
        feature,
        FeatureOrModule::Feature(Feature::ClassStaticBlock | Feature::OptionalCatchBinding)
    )
}

fn feature_overrides(config: &Config) -> Option<BTreeMap<JsFeature, bool>> {
    let env = config.env.as_ref()?;
    if env.include.is_empty() && env.exclude.is_empty() {
        return None;
    }
    let mut overrides = BTreeMap::new();
    for item in &env.include {
        match item {
            FeatureOrModule::Feature(Feature::ClassStaticBlock) => {
                overrides.insert(JsFeature::ClassStaticBlock, true);
            }
            FeatureOrModule::Feature(Feature::OptionalCatchBinding) => {
                overrides.insert(JsFeature::OptionalCatchBinding, true);
            }
            _ => {}
        }
    }
    for item in &env.exclude {
        match item {
            FeatureOrModule::Feature(Feature::ClassStaticBlock) => {
                overrides.insert(JsFeature::ClassStaticBlock, false);
            }
            FeatureOrModule::Feature(Feature::OptionalCatchBinding) => {
                overrides.insert(JsFeature::OptionalCatchBinding, false);
            }
            _ => {}
        }
    }
    (!overrides.is_empty()).then_some(overrides)
}

pub fn options(case: &SwcCaseConfig, path: &Path) -> JsTransformApiOptions {
    let config = &case.config;
    let default_transform = TransformConfig::default();
    let transform = config.jsc.transform.as_ref().unwrap_or(&default_transform);
    let react = &transform.react;
    let lang = lang(config);
    let jsx = if lang.is_jsx() && react.runtime != Some(Runtime::Preserve) {
        BoolOr::Options(JsxConfig {
            runtime: Some(match react.runtime {
                Some(Runtime::Automatic) => JsxRuntime::Automatic,
                _ => JsxRuntime::Classic,
            }),
            development: react.development,
            source_self: react.development,
            import_source: react.import_source.as_ref().map(ToString::to_string),
            pragma: react.pragma.as_ref().map(ToString::to_string),
            pragma_frag: react.pragma_frag.as_ref().map(ToString::to_string),
            throw_if_namespace: react.throw_if_namespace,
            display_name: Some(true),
            refresh: react.refresh.as_ref().map(|options| {
                BoolOr::Options(JsReactRefreshConfig {
                    refresh_reg: Some(options.refresh_reg.to_string()),
                    refresh_sig: Some(options.refresh_sig.to_string()),
                    emit_full_signatures: Some(options.emit_full_signatures),
                })
            }),
            ..Default::default()
        })
    } else {
        BoolOr::Bool(false)
    };
    let typescript = if lang.is_typescript() {
        BoolOr::Options(JsTypeScriptConfig {
            verbatim_module_syntax: Some(transform.verbatim_module_syntax.into_bool()),
            ..Default::default()
        })
    } else {
        BoolOr::Bool(false)
    };
    let target = if config.env.is_some() {
        case.env_target
            .clone()
            .or_else(|| Some(JsTarget::Queries(Vec::new())))
    } else {
        let es_version = config.jsc.target.unwrap_or_default();
        map_es_version(es_version).map(|v| JsTarget::Query(v.into()))
    };
    let decorators = map_decorators(config)
        .flatten()
        .map(BoolOr::Options);
    let module = map_module(config).flatten();
    let mut js_assumptions = config.jsc.assumptions.map(assumptions);
    if !transform.use_define_for_class_fields.into_bool() {
        js_assumptions
            .get_or_insert_with(Default::default)
            .set_public_class_fields = Some(true);
    }

    JsTransformApiOptions {
        // SWC's snapshots normalize the source file to its basename.
        filename: Some(path.file_name().unwrap().to_string_lossy().into_owned()),
        parser: Some(JsParserConfig {
            source_type: Some(source_type(config).into()),
            lang: Some(lang.into()),
            ..Default::default()
        }),
        transformer: Some(JsTransformerConfig {
            target,
            jsx: Some(jsx),
            typescript: Some(typescript),
            decorators,
            module,
            assumptions: js_assumptions,
            features: feature_overrides(config),
            ..Default::default()
        }),
        generator: Some(JsGeneratorConfig {
            minify: Some(config.minify.into_bool()),
            ascii_only: Some(matches!(
                config.jsc.output.charset,
                Some(OutputCharset::Ascii)
            )),
            ..Default::default()
        }),
        comments: Some(if config.jsc.preserve_all_comments.into_bool() {
            CommentsOption::All
        } else {
            CommentsOption::None
        }),
        ..Default::default()
    }
}
