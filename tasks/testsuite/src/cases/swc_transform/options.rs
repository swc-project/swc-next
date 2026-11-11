use std::{io, path::Path};

use jsonc_parser::{ParseOptions, parse_to_serde_value};
use serde_json::Value;
use swc_core::{
    base::config::{
        Config, FileMatcher, InputSourceMap, IsModule, OutputCharset, Rc, SourceMapsConfig,
        TransformConfig,
    },
    ecma::{
        parser::Syntax,
        preset_env::{Caniuse, Feature, FeatureOrModule, Targets, Version, Versions},
        transforms::react::Runtime,
    },
};
use swc_next_core::{
    BoolOr,
    js::{
        CommentsOption, JsTransformApiOptions,
        generator::JsGeneratorConfig,
        parser::JsParserConfig,
        transformer::{
            JsFeature, JsReactRefreshConfig, JsTransformerConfig, JsTypeScriptConfig, JsxConfig,
            JsxRuntime,
        },
    },
};
use swc_next_ecma_ast::{Lang, SourceType};

use crate::{cases::assumptions, util::read_optional};

/// Select the nearest .swcrc; parent configurations are not merged.
pub fn from_test_path(path: &Path, root: &Path) -> io::Result<Option<Config>> {
    for dir in path.parent().unwrap().ancestors() {
        if let Some(source) = read_optional(&dir.join(".swcrc"))? {
            return Ok(from_source(&source, path));
        }
        if dir == root {
            break;
        }
    }
    Ok(Rc::default().into_config(Some(path)).ok().flatten())
}

fn from_source(source: &str, path: &Path) -> Option<Config> {
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
        let mut config: Config = serde_json::from_value(value).ok()?;
        config.adjust(path);
        return Some(config);
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

pub fn supports_options(config: &Config) -> bool {
    let jsc = &config.jsc;
    if jsc
        .assumptions
        .is_some_and(|options| options.set_public_class_fields)
        || jsc.loose.into_bool()
        || config.env.as_ref().is_some_and(|env| env.loose)
    {
        return false;
    }
    if let Some(env) = &config.env {
        let modern = Versions {
            chrome: Some(Version {
                major: 120,
                minor: 0,
                patch: 0,
            }),
            ..Default::default()
        };
        if jsc.target.is_some()
            || !matches!(&env.targets, Some(Targets::Versions(versions)) if *versions == modern)
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
    }
    let syntax_supported = match jsc.syntax.unwrap_or_default() {
        Syntax::Es(parser) => {
            !parser.decorators
                && !parser.export_default_from
                && !parser.fn_bind
                && !parser.explicit_resource_management
                && !parser.auto_accessors
        }
        Syntax::Typescript(parser) => {
            !parser.decorators && !parser.disallow_ambiguous_jsx_like && !parser.no_early_errors
        }
    };
    if !syntax_supported {
        return false;
    }
    if let Some(transform) = jsc.transform.as_ref()
        && (transform.ts_enum_is_mutable.into_bool()
            || !transform.use_define_for_class_fields.into_bool()
            || transform.legacy_decorator.into_bool()
            || transform.decorator_metadata.into_bool()
            || transform.const_modules.is_some()
            || transform.optimizer.is_some()
            || transform.decorator_version.is_some()
            || transform.regenerator.import_path.is_some()
            || transform.react_compiler.is_true()
            || transform.react_compiler.is_obj())
    {
        return false;
    }
    let module_supported = config.module.as_ref().is_none_or(|module| {
        serde_json::to_value(module).ok() == Some(serde_json::json!({"type": "es6"}))
    });
    let experimental = &jsc.experimental;
    module_supported
        && matches!(
            config.source_maps,
            None | Some(SourceMapsConfig::Bool(false))
        )
        && matches!(
            config.input_source_map,
            None | Some(InputSourceMap::Bool(false))
        )
        && config.source_map_ignore_list.is_none()
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

fn feature_enabled(config: &Config, feature: Feature) -> bool {
    match &config.env {
        None => !config.jsc.target.unwrap_or_default().caniuse(feature),
        Some(env) => {
            let contains = |features: &[FeatureOrModule]| {
                features.iter().any(
                    |value| matches!(value, FeatureOrModule::Feature(value) if *value == feature),
                )
            };
            contains(&env.include) && !contains(&env.exclude)
        }
    }
}

pub fn options(config: &Config, path: &Path) -> JsTransformApiOptions {
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
    let static_blocks = feature_enabled(config, Feature::ClassStaticBlock);
    let optional_catch_binding = feature_enabled(config, Feature::OptionalCatchBinding);
    JsTransformApiOptions {
        // SWC's snapshots normalize the source file to its basename.
        filename: Some(path.file_name().unwrap().to_string_lossy().into_owned()),
        parser: Some(JsParserConfig {
            source_type: Some(source_type(config).into()),
            lang: Some(lang.into()),
            ..Default::default()
        }),
        transformer: Some(JsTransformerConfig {
            jsx: Some(jsx),
            typescript: Some(typescript),
            assumptions: config.jsc.assumptions.map(assumptions),
            features: Some(
                [
                    (JsFeature::ClassStaticBlock, static_blocks),
                    (JsFeature::OptionalCatchBinding, optional_catch_binding),
                ]
                .into(),
            ),
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
