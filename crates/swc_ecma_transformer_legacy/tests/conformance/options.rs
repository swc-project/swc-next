use std::{fs, path::Path};

use jsonc_parser::{ParseOptions, parse_to_serde_value};
use swc_core::{
    base::config::{Config, IsModule, ModuleConfig, OutputCharset, Rc, TransformConfig},
    ecma::{
        parser::Syntax,
        preset_env::{Caniuse, EnvConfig as SwcEnvConfig, Feature},
        transforms::react::Runtime,
    },
};
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_codegen::Options as GeneratorOptions;
use swc_next_ecma_parser::Options as ParserOptions;
use swc_next_ecma_transformer_legacy::{
    TransformerOptions,
    env::EnvOptions,
    es2016::ES2016Options,
    es2017::{AsyncToGeneratorOptions, ES2017Options},
    es2018::{ES2018Options, ObjectRestSpreadOptions},
    es2019::ES2019Options,
    es2020::{ES2020Options, NullishCoalescingOptions, OptionalChainingOptions},
    es2021::ES2021Options,
    es2022::{ClassPropertiesOptions, ES2022Options},
    jsx::{JsxOptions, JsxRuntime, ReactRefreshOptions},
    regexp::RegExpOptions,
    typescript::{JsxImportOptions, TsImportExportAssignConfig, TypeScriptOptions},
};

pub struct Options {
    pub parser: ParserOptions,
    pub transformer: TransformerOptions,
    pub generator: GeneratorOptions,
}

/// Read the nearest .swcrc; SWC selects and adjusts its configuration.
pub fn read(path: &Path, root: &Path) -> Result<Option<Options>, String> {
    for directory in path.parent().unwrap().ancestors() {
        let config = directory.join(".swcrc");
        if config.is_file() {
            let source = fs::read_to_string(config).map_err(|error| error.to_string())?;
            return from_source(&source, path);
        }
        if directory == root {
            break;
        }
    }
    select(Rc::default(), path)
}

fn from_source(source: &str, path: &Path) -> Result<Option<Options>, String> {
    let value = parse_to_serde_value(
        source.trim_start_matches('\u{feff}'),
        &ParseOptions {
            allow_comments: true,
            allow_trailing_commas: true,
            ..Default::default()
        },
    )
    .map_err(|error| error.to_string())?;
    let config = serde_json::from_value(value).map_err(|error| error.to_string())?;
    select(config, path)
}

fn select(config: Rc, path: &Path) -> Result<Option<Options>, String> {
    config
        .into_config(Some(path))
        .map(|config| config.map(|config| convert(config, path)))
        .map_err(|error| error.to_string())
}

fn convert(config: Config, path: &Path) -> Options {
    let loose = config.jsc.loose.into_bool() || config.env.as_ref().is_some_and(|env| env.loose);
    let features = config
        .env
        .map(|env| SwcEnvConfig::from(env).get_feature_config());
    let enabled = |feature| {
        !features.as_ref().map_or_else(
            || config.jsc.target.unwrap_or_default().caniuse(feature),
            |features| features.caniuse(feature),
        )
    };
    let default_transform = TransformConfig::default();
    let transform = config.jsc.transform.as_ref().unwrap_or(&default_transform);
    let assumptions = config.jsc.assumptions.unwrap_or_default();
    let set_public_fields =
        assumptions.set_public_class_fields || !transform.use_define_for_class_fields.into_bool();
    let mut options = TransformerOptions {
        filename: Some(path.file_name().unwrap().to_string_lossy().into_owned()),
        env: EnvOptions {
            es2016: ES2016Options {
                exponentiation: enabled(Feature::ExponentiationOperator),
            },
            es2017: ES2017Options {
                async_to_generator: (enabled(Feature::AsyncToGenerator)
                    || enabled(Feature::AsyncGeneratorFunctions))
                .then_some(AsyncToGeneratorOptions {
                    ignore_function_length: assumptions.ignore_function_length,
                }),
            },
            es2018: ES2018Options {
                object_rest_spread: enabled(Feature::ObjectRestSpread).then_some(
                    ObjectRestSpreadOptions {
                        no_symbol: assumptions.object_rest_no_symbols,
                        set_property: assumptions.set_spread_properties,
                        pure_getters: assumptions.pure_getters,
                    },
                ),
            },
            es2019: ES2019Options {
                optional_catch_binding: enabled(Feature::OptionalCatchBinding),
            },
            es2020: ES2020Options {
                export_namespace_from: enabled(Feature::ExportNamespaceFrom),
                nullish_coalescing: enabled(Feature::NullishCoalescing).then_some(
                    NullishCoalescingOptions {
                        no_document_all: loose || assumptions.no_document_all,
                    },
                ),
                optional_chaining: enabled(Feature::OptionalChaining).then_some(
                    OptionalChainingOptions {
                        no_document_all: loose || assumptions.no_document_all,
                        pure_getter: loose || assumptions.pure_getters,
                    },
                ),
            },
            es2021: ES2021Options {
                logical_assignments: enabled(Feature::LogicalAssignmentOperators),
            },
            es2022: ES2022Options {
                static_blocks: enabled(Feature::ClassStaticBlock),
                class_properties: enabled(Feature::ClassProperties).then_some(
                    ClassPropertiesOptions {
                        private_as_properties: loose || assumptions.private_fields_as_properties,
                        set_public_fields: loose || set_public_fields,
                        constant_super: loose || assumptions.constant_super,
                        no_document_all: loose || assumptions.no_document_all,
                        pure_getter: loose || assumptions.pure_getters,
                    },
                ),
                private_in_object: enabled(Feature::PrivatePropertyInObject),
            },
            regexp: RegExpOptions {
                dot_all_regex: enabled(Feature::DotAllRegex),
                named_capturing_groups_regex: enabled(Feature::NamedCapturingGroupsRegex),
                duplicate_named_capturing_groups_regex: enabled(
                    Feature::DuplicateNamedCapturingGroupsRegex,
                ),
                sticky_regex: enabled(Feature::StickyRegex),
                unicode_property_regex: enabled(Feature::UnicodePropertyRegex),
                unicode_regex: enabled(Feature::UnicodeRegex),
                unicode_sets_regex: enabled(Feature::UnicodeSetsRegex),
                ..Default::default()
            },
        },
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
                Some(
                    ModuleConfig::CommonJs(..)
                    | ModuleConfig::Amd(..)
                    | ModuleConfig::Umd(..)
                    | ModuleConfig::SystemJs(..),
                ) => TsImportExportAssignConfig::Preserve,
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
    Options {
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
    }
}
