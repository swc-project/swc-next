use std::{fs, path::Path};

use jsonc_parser::{ParseOptions, parse_to_serde_value};
use preset_env_base::query::Query;
use serde_json::Value;
use swc_core::{
    base::config::{
        Config, FileMatcher, InputSourceMap, IsModule, ModuleConfig, OutputCharset, Rc,
        SourceMapsConfig, TransformConfig,
    },
    ecma::{
        parser::Syntax,
        preset_env::{Caniuse, EnvConfig as SwcEnvConfig, Feature, FeatureOrModule, Targets},
        transforms::react::Runtime,
    },
};
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_codegen::Options as GeneratorOptions;
use swc_next_ecma_parser::Options as ParserOptions;
use swc_next_ecma_transformer_legacy::{
    BoolOr, TransformerOptions,
    env::{
        AsyncToGeneratorConfig, ClassPropertiesConfig, ES2016Config, ES2017Config, ES2018Config,
        ES2019Config, ES2020Config, ES2021Config, ES2022Config, Engine, EnvConfig, EsVersion,
        NullishCoalescingConfig, ObjectRestSpreadConfig, OptionalChainingConfig, RegExpConfig,
        Target,
    },
    jsx::{JsxOptions, JsxRuntime, ReactRefreshOptions},
    typescript::{JsxImportOptions, TsImportExportAssignConfig, TypeScriptOptions},
};

pub struct Options {
    pub parser: ParserOptions,
    pub transformer: TransformerOptions,
    pub generator: GeneratorOptions,
}

// Every pass has an explicit override, including disabled passes. Otherwise the
// target's conservative selection could re-enable a pass SWC did not select.
fn pass_override<C>(enabled: bool, config: C) -> BoolOr<C> {
    if enabled {
        BoolOr::Options(config)
    } else {
        BoolOr::Bool(false)
    }
}

/// Select the nearest .swcrc exactly as the SWC compile fixture suite does.
/// Parent files are not merged. Array matching happens before option decoding.
pub fn read(path: &Path, root: &Path) -> Result<Options, &'static str> {
    for directory in path.parent().unwrap().ancestors() {
        let config = directory.join(".swcrc");
        if config.is_file() {
            return from_source(&fs::read_to_string(config).unwrap(), path);
        }
        if directory == root {
            break;
        }
    }
    let config = Rc::default()
        .into_config(Some(path))
        .ok()
        .flatten()
        .ok_or("unselected configuration")?;
    convert(config, path)
}

fn from_source(source: &str, path: &Path) -> Result<Options, &'static str> {
    let value = parse_to_serde_value(
        source.trim_start_matches('\u{feff}'),
        &ParseOptions {
            allow_comments: true,
            allow_trailing_commas: true,
            ..Default::default()
        },
    )
    .map_err(|_| "invalid configuration")?;
    let values = match value {
        Value::Array(values) => values,
        value => vec![value],
    };
    for value in values {
        if let Some(test) = value.get("test").filter(|v| !v.is_null()) {
            let matcher: FileMatcher =
                serde_json::from_value(test.clone()).map_err(|_| "invalid configuration")?;
            if !matcher.matches(path).map_err(|_| "invalid configuration")? {
                continue;
            }
        }
        if let Some(exclude) = value.get("exclude").filter(|v| !v.is_null()) {
            let matcher: FileMatcher =
                serde_json::from_value(exclude.clone()).map_err(|_| "invalid configuration")?;
            if matcher.matches(path).map_err(|_| "invalid configuration")? {
                continue;
            }
        }
        let mut config: Config =
            serde_json::from_value(value).map_err(|_| "unsupported configuration schema")?;
        config.adjust(path);
        return convert(config, path);
    }
    Err("unselected configuration")
}

fn supported(config: &Config) -> Result<(), &'static str> {
    let jsc = &config.jsc;
    if let Some(env) = &config.env {
        if jsc.target.is_some()
            || !matches!(env.targets, Some(Targets::Versions(_) | Targets::Query(_)))
            || env.path.is_some()
        {
            return Err("unsupported environment selection");
        }
        if env.mode.is_some()
            || env.core_js.is_some()
            || !env.skip.is_empty()
            || env
                .include
                .iter()
                .chain(&env.exclude)
                .any(|feature| matches!(feature, FeatureOrModule::CoreJsModule(_)))
        {
            return Err("polyfills outside scope");
        }
    }
    // Parser capability flags do not select Next parser features or prove that
    // an input needs a transform. Let Next parse it and compare the output.
    if let Some(transform) = jsc.transform.as_ref()
        && (transform.legacy_decorator.into_bool()
            || transform.decorator_metadata.into_bool()
            || transform.const_modules.is_some()
            || transform.optimizer.is_some()
            || transform.decorator_version.is_some()
            || transform.regenerator.import_path.is_some()
            || transform.react_compiler.is_true()
            || transform.react_compiler.is_obj()
            || transform.hidden.jest.into_bool())
    {
        return Err("transform outside scope");
    }
    if let Some(assumptions) = jsc.assumptions {
        let value = serde_json::to_value(assumptions).unwrap();
        if value.as_object().unwrap().iter().any(|(name, value)| {
            value == &Value::Bool(true)
                && !matches!(
                    name.as_str(),
                    "constantSuper"
                        | "ignoreFunctionLength"
                        | "noDocumentAll"
                        | "objectRestNoSymbols"
                        | "privateFieldsAsProperties"
                        | "pureGetters"
                        | "setPublicClassFields"
                        | "setSpreadProperties"
                )
        }) {
            return Err("assumption outside scope");
        }
    }
    if let Some(module) = &config.module {
        let value = serde_json::to_value(module).unwrap();
        if value != serde_json::json!({"type": "es6"})
            && value != serde_json::json!({"type": "nodenext"})
        {
            return Err("module transform outside scope");
        }
    }
    let experimental = &jsc.experimental;
    if !matches!(
        config.source_maps,
        None | Some(SourceMapsConfig::Bool(false))
    ) || !matches!(
        config.input_source_map,
        None | Some(InputSourceMap::Bool(false))
    ) || config.source_map_ignore_list.is_some()
        || jsc.keep_class_names.into_bool()
        || !jsc.base_url.as_os_str().is_empty()
        || !jsc.paths.is_empty()
        || jsc.minify.is_some()
        || jsc.rewrite_relative_import_extensions.into_bool()
        || jsc.preserve_symlinks.into_bool()
        || !jsc.output.preamble.is_empty()
        || jsc.output.source_map_url.is_some()
        || jsc.output.preserve_annotations.into_bool()
        || experimental.emit_assert_for_import_attributes.into_bool()
        || experimental.keep_import_attributes.into_bool()
        || experimental.emit_source_map_scopes.into_bool()
        || experimental.emit_isolated_dts.into_bool()
        || experimental
            .disable_builtin_transforms_for_internal_testing
            .into_bool()
        || experimental.plugins.is_some()
        || experimental.plugin_env_vars.is_some()
    {
        return Err("compiler option outside scope");
    }
    Ok(())
}

fn convert(config: Config, path: &Path) -> Result<Options, &'static str> {
    supported(&config)?;
    let env = config.env.clone().map(SwcEnvConfig::from);
    let features = env.as_ref().map(SwcEnvConfig::get_feature_config);
    let enabled = |feature| {
        !features.as_ref().map_or_else(
            || config.jsc.target.unwrap_or_default().caniuse(feature),
            |features| features.caniuse(feature),
        )
    };
    // These compiler stages are not represented by this crate's selected scope.
    if [
        Feature::TemplateLiterals,
        Feature::ArrowFunctions,
        Feature::BlockScopedFunctions,
        Feature::Classes,
        Feature::ObjectSuper,
        Feature::ShorthandProperties,
        Feature::DuplicateKeys,
        Feature::ComputedProperties,
        Feature::ForOf,
        Feature::Spread,
        Feature::Parameters,
        Feature::Destructuring,
        Feature::BlockScoping,
        Feature::TypeOfSymbol,
        Feature::NewTarget,
        Feature::Regenerator,
        Feature::FunctionName,
        Feature::MemberExpressionLiterals,
        Feature::PropertyLiterals,
        Feature::ReservedWords,
        Feature::BugfixAsyncArrowsInClass,
        Feature::BugfixEdgeDefaultParam,
        Feature::BugfixTaggedTemplateCaching,
        Feature::BugfixSafariIdDestructuringCollisionInFunctionExpression,
    ]
    .into_iter()
    .any(enabled)
    {
        return Err("target requires transforms outside scope");
    }
    let async_functions = enabled(Feature::AsyncToGenerator);
    if async_functions != enabled(Feature::AsyncGeneratorFunctions) {
        return Err("selective async lowering not representable");
    }
    let default_transform = TransformConfig::default();
    let transform = config.jsc.transform.as_ref().unwrap_or(&default_transform);
    let assumptions = config.jsc.assumptions.unwrap_or_default();
    let loose = config.jsc.loose.into_bool() || config.env.as_ref().is_some_and(|env| env.loose);
    let set_public_fields =
        assumptions.set_public_class_fields || !transform.use_define_for_class_fields.into_bool();
    if set_public_fields && !enabled(Feature::ClassProperties) {
        return Err("native class field set semantics outside scope");
    }
    let env = EnvConfig {
        target: Some(
            match config.env.as_ref().and_then(|env| env.targets.as_ref()) {
                Some(Targets::Query(Query::Single(query))) => {
                    Target::Browserslist(vec![query.clone()])
                }
                Some(Targets::Query(Query::Multiple(queries))) => {
                    Target::Browserslist(queries.clone())
                }
                Some(Targets::Versions(versions)) => {
                    let mut engines = std::collections::BTreeMap::new();
                    for (name, version) in versions.iter() {
                        if let Some(version) = version {
                            let engine: Engine =
                                name.parse().map_err(|_| "unsupported target engine")?;
                            engines.insert(
                                engine,
                                format!("{}.{}.{}", version.major, version.minor, version.patch),
                            );
                        }
                    }
                    Target::Engines(engines)
                }
                _ => Target::EsVersion(
                    serde_json::to_value(config.jsc.target.unwrap_or_default())
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .parse::<EsVersion>()
                        .map_err(|_| "unsupported ES target")?,
                ),
            },
        ),
        es2016: ES2016Config {
            exponentiation: Some(enabled(Feature::ExponentiationOperator)),
        },
        es2017: ES2017Config {
            async_to_generator: Some(pass_override(
                async_functions,
                AsyncToGeneratorConfig {
                    ignore_function_length: Some(assumptions.ignore_function_length),
                },
            )),
        },
        es2018: ES2018Config {
            object_rest_spread: Some(pass_override(
                enabled(Feature::ObjectRestSpread),
                ObjectRestSpreadConfig {
                    no_symbol: Some(assumptions.object_rest_no_symbols),
                    set_property: Some(assumptions.set_spread_properties),
                    pure_getters: Some(assumptions.pure_getters),
                },
            )),
        },
        es2019: ES2019Config {
            optional_catch_binding: Some(enabled(Feature::OptionalCatchBinding)),
        },
        es2020: ES2020Config {
            export_namespace_from: Some(enabled(Feature::ExportNamespaceFrom)),
            nullish_coalescing: Some(pass_override(
                enabled(Feature::NullishCoalescing),
                NullishCoalescingConfig {
                    no_document_all: Some(loose || assumptions.no_document_all),
                },
            )),
            optional_chaining: Some(pass_override(
                enabled(Feature::OptionalChaining),
                OptionalChainingConfig {
                    no_document_all: Some(loose || assumptions.no_document_all),
                    pure_getter: Some(loose || assumptions.pure_getters),
                },
            )),
        },
        es2021: ES2021Config {
            logical_assignments: Some(enabled(Feature::LogicalAssignmentOperators)),
        },
        es2022: ES2022Config {
            static_blocks: Some(enabled(Feature::ClassStaticBlock)),
            class_properties: Some(pass_override(
                enabled(Feature::ClassProperties),
                ClassPropertiesConfig {
                    private_as_properties: Some(loose || assumptions.private_fields_as_properties),
                    set_public_fields: Some(loose || set_public_fields),
                    constant_super: Some(loose || assumptions.constant_super),
                    no_document_all: Some(loose || assumptions.no_document_all),
                    pure_getter: Some(loose || assumptions.pure_getters),
                },
            )),
            private_in_object: Some(enabled(Feature::PrivatePropertyInObject)),
        },
        regexp: RegExpConfig {
            dot_all_regex: Some(enabled(Feature::DotAllRegex)),
            named_capturing_groups_regex: Some(enabled(Feature::NamedCapturingGroupsRegex)),
            duplicate_named_capturing_groups_regex: Some(enabled(
                Feature::DuplicateNamedCapturingGroupsRegex,
            )),
            sticky_regex: Some(enabled(Feature::StickyRegex)),
            unicode_property_regex: Some(enabled(Feature::UnicodePropertyRegex)),
            unicode_regex: Some(enabled(Feature::UnicodeRegex)),
            unicode_sets_regex: Some(enabled(Feature::UnicodeSetsRegex)),
            // SWC's preset-env has no target features for indices or lookbehind.
            ..Default::default()
        },
    };
    let mut options = TransformerOptions {
        // SWC normalizes filenames in development snapshots to the basename.
        filename: Some(path.file_name().unwrap().to_string_lossy().into_owned()),
        env: env
            .to_env_options()
            .map_err(|_| "invalid environment target")?,
        ..Default::default()
    };
    let lang = match config.jsc.syntax.unwrap_or_default() {
        Syntax::Typescript(parser) if parser.dts => Lang::Dts,
        Syntax::Typescript(parser) if parser.tsx => Lang::Tsx,
        Syntax::Typescript(_) => Lang::Ts,
        Syntax::Es(parser) if parser.jsx => Lang::Jsx,
        _ => Lang::Js,
    };
    let react = &transform.react;
    let jsx_enabled = lang.is_jsx() && react.runtime != Some(Runtime::Preserve);
    if jsx_enabled {
        #[allow(deprecated)]
        let jsx = JsxOptions {
            next: react.next,
            runtime: react.runtime.map(|runtime| match runtime {
                Runtime::Automatic => JsxRuntime::Automatic,
                Runtime::Classic => JsxRuntime::Classic,
                Runtime::Preserve => JsxRuntime::Preserve,
            }),
            import_source: react.import_source.as_ref().map(ToString::to_string),
            pragma: react.pragma.as_ref().map(ToString::to_string),
            pragma_frag: react.pragma_frag.as_ref().map(ToString::to_string),
            throw_if_namespace: react.throw_if_namespace,
            development: react.development,
            use_builtins: react.use_builtins,
            use_spread: react.use_spread,
            refresh: react.refresh.as_ref().map(|refresh| ReactRefreshOptions {
                refresh_reg: refresh.refresh_reg.to_string(),
                refresh_sig: refresh.refresh_sig.to_string(),
                emit_full_signatures: refresh.emit_full_signatures,
            }),
        };
        options.jsx = Some(jsx);
    }
    if lang.is_typescript() {
        options.typescript = Some(TypeScriptOptions {
            verbatim_module_syntax: transform.verbatim_module_syntax.into_bool(),
            native_class_properties: !set_public_fields && !enabled(Feature::ClassProperties),
            import_export_assign_config: match config.module {
                Some(ModuleConfig::Es6(..)) => TsImportExportAssignConfig::EsNext,
                Some(ModuleConfig::NodeNext(..)) => TsImportExportAssignConfig::NodeNext,
                _ => TsImportExportAssignConfig::Classic,
            },
            ts_enum_is_mutable: transform.ts_enum_is_mutable.into_bool(),
            jsx: if jsx_enabled {
                JsxImportOptions {
                    pragma: react.pragma.as_ref().map(ToString::to_string),
                    pragma_frag: react.pragma_frag.as_ref().map(ToString::to_string),
                }
            } else {
                JsxImportOptions::default()
            },
            ..Default::default()
        });
    }
    Ok(Options {
        parser: ParserOptions {
            lang,
            source_type: match config.is_module.unwrap_or_default() {
                IsModule::Bool(false) => SourceType::Script,
                IsModule::Bool(true) => SourceType::Module,
                IsModule::Unknown => SourceType::Unambiguous,
                IsModule::CommonJS => SourceType::CommonJs,
            },
            ..Default::default()
        },
        transformer: options,
        generator: GeneratorOptions {
            minify: config.minify.into_bool(),
            ascii_only: matches!(config.jsc.output.charset, Some(OutputCharset::Ascii)),
            comments: config.jsc.preserve_all_comments.into_bool(),
            ..Default::default()
        },
    })
}

#[cfg(test)]
mod tests {
    use swc_next_ecma_transformer_legacy::es2020::OptionalChainingOptions;

    use super::*;

    fn fixture_options(source: &str) -> Options {
        from_source(source, Path::new("input.js")).unwrap()
    }

    #[test]
    fn target_selection() {
        let options = fixture_options(r#"{"jsc":{"target":"es2015"}}"#);
        assert!(options.transformer.env.es2016.exponentiation);
        assert!(options.transformer.env.es2017.async_to_generator.is_some());
        assert!(options.transformer.env.es2018.object_rest_spread.is_some());
        assert!(options.transformer.env.es2019.optional_catch_binding);
        assert!(options.transformer.env.es2020.optional_chaining.is_some());
        assert!(options.transformer.env.es2021.logical_assignments);
        assert!(options.transformer.env.es2022.class_properties.is_some());
        assert!(options.transformer.env.regexp.dot_all_regex);
        assert!(!options.transformer.env.regexp.has_indices);

        let options = fixture_options(r#"{"jsc":{"target":"es2022"}}"#);
        assert_eq!(
            options.transformer,
            TransformerOptions {
                filename: Some("input.js".into()),
                ..Default::default()
            }
        );

        let options = from_source(r#"{"jsc":{"target":"es5"}}"#, Path::new("input.js"));
        assert!(options.is_err());

        let options = from_source(r#"{"jsc":{"target":"es2017"}}"#, Path::new("input.js"));
        assert!(options.is_err());
    }

    #[test]
    fn environment_selection() {
        let source = r#"{"env":{"targets":{"chrome":"120"},"include":["transform-optional-chaining"],"exclude":["transform-class-static-block"]},"jsc":{"assumptions":{"noDocumentAll":true,"pureGetters":true}}}"#;
        let options = fixture_options(source);
        assert_eq!(
            options.transformer.env.es2020.optional_chaining,
            Some(OptionalChainingOptions {
                no_document_all: true,
                pure_getter: true,
            })
        );
        assert!(!options.transformer.env.es2022.static_blocks);
        assert!(options.transformer.env.es2020.nullish_coalescing.is_none());

        let source = r#"{"env":{"targets":{"chrome":"120"},"include":["transform-optional-catch-binding"]}}"#;
        let options = fixture_options(source);
        assert!(options.transformer.env.es2019.optional_catch_binding);
        assert!(options.transformer.env.es2017.async_to_generator.is_none());

        let source =
            r#"{"env":{"targets":["chrome 120"],"include":["transform-optional-catch-binding"]}}"#;
        let options = fixture_options(source);
        assert!(options.transformer.env.es2019.optional_catch_binding);
        assert!(!options.transformer.env.es2016.exponentiation);

        let source = r#"{"env":{"targets":{"chrome":"120"},"mode":"usage","coreJs":"3"}}"#;
        let options = from_source(source, Path::new("input.js"));
        assert!(options.is_err());
    }

    #[test]
    fn language_options() {
        let source = r#"{"jsc":{"target":"es2022","parser":{"syntax":"typescript","tsx":true},"transform":{"verbatimModuleSyntax":true,"tsEnumIsMutable":true,"react":{"runtime":"automatic","development":true,"refresh":true,"pragma":"h","pragmaFrag":"Fragment"}}},"module":{"type":"es6"}}"#;
        let options = fixture_options(source);
        assert_eq!(options.parser.lang, Lang::Tsx);
        let jsx = options.transformer.jsx.unwrap();
        assert_eq!(jsx.runtime, Some(JsxRuntime::Automatic));
        assert_eq!(jsx.development, Some(true));
        assert_eq!(jsx.refresh, Some(ReactRefreshOptions::default()));
        let typescript = options.transformer.typescript.unwrap();
        assert!(typescript.verbatim_module_syntax);
        assert!(typescript.ts_enum_is_mutable);
        assert!(typescript.native_class_properties);
        assert_eq!(
            typescript.import_export_assign_config,
            TsImportExportAssignConfig::EsNext
        );
        assert_eq!(typescript.jsx.pragma.as_deref(), Some("h"));

        let source = r#"{"jsc":{"target":"es2015","parser":{"syntax":"typescript"},"transform":{"useDefineForClassFields":false}},"module":{"type":"nodenext"}}"#;
        let options = fixture_options(source);
        assert!(
            options
                .transformer
                .env
                .es2022
                .class_properties
                .unwrap()
                .set_public_fields
        );
        let typescript = options.transformer.typescript.unwrap();
        assert!(!typescript.native_class_properties);
        assert_eq!(
            typescript.import_export_assign_config,
            TsImportExportAssignConfig::NodeNext
        );

        let source = r#"{"jsc":{"target":"esnext","parser":{"syntax":"ecmascript","jsx":true},"transform":{"react":{"runtime":"preserve"}}},"isModule":false}"#;
        let options = fixture_options(source);
        assert_eq!(options.parser.lang, Lang::Jsx);
        assert_eq!(options.parser.source_type, SourceType::Script);
        assert!(options.transformer.jsx.is_none());
    }

    #[test]
    fn parser_options() {
        let source = r#"{"jsc":{"target":"esnext","parser":{"syntax":"ecmascript","autoAccessors":true,"allowReturnOutsideFunction":true,"allowSuperOutsideMethod":true}}}"#;
        let options = fixture_options(source);
        assert_eq!(options.parser.lang, Lang::Js);

        let source = r#"{"jsc":{"target":"esnext","parser":{"syntax":"typescript","noEarlyErrors":true,"disallowAmbiguousJSXLike":true}}}"#;
        let options = fixture_options(source);
        assert_eq!(options.parser.lang, Lang::Ts);

        let source =
            r#"{"jsc":{"target":"esnext","parser":{"syntax":"ecmascript","decorators":true}}}"#;
        let options = fixture_options(source);
        assert_eq!(options.parser.lang, Lang::Js);

        let source = r#"{"jsc":{"target":"esnext","parser":{"syntax":"ecmascript","decorators":true},"transform":{"legacyDecorator":true}}}"#;
        let options = from_source(source, Path::new("input.js"));
        assert!(options.is_err());
    }

    #[test]
    fn configuration_selection() {
        let source = "\u{feff}[{\"test\":\"other\\\\.ts$\",\"invalid\":true},/* selected */{\"jsc\":{\"target\":\"esnext\"},}]";
        let options = fixture_options(source);
        assert!(options.transformer.typescript.is_none());

        let source = r#"[{"exclude":"input","jsc":{"target":"esnext"}}]"#;
        let options = from_source(source, Path::new("input.js"));
        assert!(options.is_err());

        let source = r#"{"jsc":{"target":"esnext"},"module":{"type":"commonjs"}}"#;
        let options = from_source(source, Path::new("input.js"));
        assert!(options.is_err());
    }
}
