use std::{io, path::Path};

use crate::util::read_optional;
use jsonc_parser::{ParseOptions, parse_to_serde_value};
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_codegen::Options as CodegenOptions;
use swc_next_ecma_transformer::{
    TransformerOptions,
    assumptions::Assumptions,
    es2022::ES2022Options,
    jsx::{JsxOptions, JsxRuntime, ReactRefreshOptions},
    typescript::TypeScriptOptions,
};

/// Supported compiler configuration. Unknown fields leave the case skipped,
/// rather than silently running a different transform pipeline.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Swcrc {
    jsc: Jsc,
    env: Option<Env>,
    module: Option<Value>,
    minify: bool,
    is_module: Option<Value>,
    source_maps: Option<Value>,
    input_source_map: Option<Value>,
    inline_sources_content: Option<bool>,
    #[serde(rename = "$schema")]
    schema: Option<String>,
    test: Option<FileMatcher>,
    exclude: Option<FileMatcher>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct Jsc {
    parser: Parser,
    target: Option<String>,
    transform: Option<Transform>,
    assumptions: Assumptions,
    loose: bool,
    external_helpers: Option<bool>,
    preserve_all_comments: bool,
    keep_class_names: bool,
    experimental: Experimental,
    output: Output,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct Transform {
    react: React,
    verbatim_module_syntax: bool,
    ts_enum_is_mutable: bool,
    use_define_for_class_fields: Option<bool>,
    legacy_decorator: bool,
    decorator_metadata: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct React {
    runtime: Option<String>,
    development: bool,
    import_source: Option<String>,
    pragma: Option<String>,
    pragma_frag: Option<String>,
    throw_if_namespace: Option<bool>,
    refresh: Option<Refresh>,
    // SWC accepts these deprecated options without changing its JSX lowering.
    use_builtins: Option<bool>,
    use_spread: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Refresh {
    Enabled(bool),
    Options(ReactRefreshOptions),
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct Parser {
    syntax: Option<String>,
    jsx: bool,
    tsx: bool,
    dts: bool,
    decorators: bool,
    decorators_before_export: bool,
    export_default_from: bool,
    function_bind: bool,
    explicit_resource_management: bool,
    disallow_ambiguous_jsx_like: bool,
    no_early_errors: bool,
    // SWC still accepts these historical parser switches. Both parsers now
    // accept the corresponding standard syntax regardless of their values.
    dynamic_import: bool,
    class_property: bool,
    class_private_property: bool,
    private_method: bool,
    export_namespace_from: bool,
    import_meta: bool,
    top_level_await: bool,
    import_assertions: bool,
    import_attributes: bool,
    nullish_coalescing: bool,
    numeric_separator: bool,
    optional_chaining: bool,
    auto_accessors: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct Env {
    targets: Option<Value>,
    include: Vec<String>,
    exclude: Vec<String>,
    loose: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct Experimental {
    disable_all_lints: bool,
    #[serde(alias = "keepImportAssertions")]
    keep_import_attributes: bool,
    emit_assert_for_import_attributes: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct Output {
    charset: Option<String>,
    preamble: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FileMatcher {
    Pattern(String),
    Multi(Vec<FileMatcher>),
}

impl FileMatcher {
    fn matches(&self, filename: &str) -> Option<bool> {
        match self {
            Self::Pattern(pattern) => Some(Regex::new(pattern).ok()?.is_match(filename)),
            Self::Multi(patterns) => {
                for pattern in patterns {
                    if pattern.matches(filename)? {
                        return Some(true);
                    }
                }
                Some(false)
            }
        }
    }
}

impl Swcrc {
    /// The compiler reads only the nearest .swcrc. Arrays select the first
    /// matching configuration; parent files do not merge into that selection.
    pub fn from_test_path(path: &Path, root: &Path) -> io::Result<Option<Self>> {
        for dir in path.parent().unwrap().ancestors() {
            let config = dir.join(".swcrc");
            if let Some(source) = read_optional(&config)? {
                return Ok(Self::from_source(&source, path));
            }
            if dir == root {
                break;
            }
        }
        // With no .swcrc, SWC chooses its default parser from the extension.
        let mut config = Self::default();
        if matches!(
            path.extension().and_then(|v| v.to_str()),
            Some("ts" | "tsx")
        ) {
            config.jsc.parser.syntax = Some("typescript".into());
        }
        Ok(Some(config))
    }

    fn from_source(source: &str, path: &Path) -> Option<Self> {
        let source = source.trim_start_matches('\u{feff}');
        let options = ParseOptions {
            allow_comments: true,
            allow_trailing_commas: true,
            allow_loose_object_property_names: false,
        };
        let value = parse_to_serde_value(source, &options).ok()??;
        let values = match value {
            Value::Array(values) => values,
            value => vec![value],
        };
        let filename = path.to_string_lossy().replace('\\', "/");
        for value in values {
            // Match before decoding transform options so an unsupported,
            // unselected branch does not hide a supported configuration.
            let test = value.get("test").filter(|v| !v.is_null());
            let exclude = value.get("exclude").filter(|v| !v.is_null());
            if let Some(test) = test {
                let matcher: FileMatcher = serde_json::from_value(test.clone()).ok()?;
                if !matcher.matches(&filename)? {
                    continue;
                }
            }
            if let Some(exclude) = exclude {
                let matcher: FileMatcher = serde_json::from_value(exclude.clone()).ok()?;
                if matcher.matches(&filename)? {
                    continue;
                }
            }
            return serde_json::from_value(value).ok();
        }
        None
    }

    pub fn source_type(&self) -> SourceType {
        match self.is_module.as_ref() {
            Some(Value::Bool(false)) => SourceType::Script,
            Some(Value::String(v)) if v == "unknown" => SourceType::Unambiguous,
            Some(Value::String(v)) if v == "commonjs" => SourceType::CommonJs,
            _ => SourceType::Module,
        }
    }

    pub fn lang(&self, path: &Path) -> Lang {
        let parser = &self.jsc.parser;
        if parser.syntax.as_deref() == Some("typescript") {
            if parser.dts || path.to_string_lossy().ends_with(".d.ts") {
                return Lang::Dts;
            }
            let tsx = match path.extension().and_then(|v| v.to_str()) {
                Some("tsx") => true,
                Some("ts") => false,
                _ => parser.tsx,
            };
            if tsx { Lang::Tsx } else { Lang::Ts }
        } else if parser.jsx {
            Lang::Jsx
        } else {
            Lang::Js
        }
    }

    pub fn supports_options(&self) -> bool {
        let jsc = &self.jsc;
        if jsc.assumptions.set_public_class_fields
            || jsc.loose
            || self.env.as_ref().is_some_and(|env| env.loose)
        {
            // SWC lowers class fields in assignment mode even on modern targets.
            return false;
        }
        // Chrome 120 is the modern baseline used by SWC's standalone fixtures.
        if let Some(env) = &self.env {
            let mut features = env.include.iter().chain(&env.exclude);
            if jsc.target.is_some()
                || env.targets.as_ref() != Some(&serde_json::json!({"chrome": "120"}))
                || features.any(|feature| feature != "transform-class-static-block")
            {
                return false;
            }
        }
        let parser = &jsc.parser;
        let syntax = parser.syntax.as_deref();
        if !matches!(syntax, None | Some("ecmascript" | "typescript"))
            || parser.decorators
            || parser.export_default_from
            || parser.function_bind
            || parser.explicit_resource_management
            || parser.disallow_ambiguous_jsx_like
            || parser.no_early_errors
            || parser.auto_accessors
        {
            return false;
        }
        if let Some(transform) = &jsc.transform {
            let runtime = transform.react.runtime.as_deref();
            if transform.ts_enum_is_mutable
                || transform.use_define_for_class_fields == Some(false)
                || transform.legacy_decorator
                || transform.decorator_metadata
                || !matches!(runtime, None | Some("classic" | "automatic" | "preserve"))
            {
                return false;
            }
        }
        let module = self.module.as_ref();
        let source_maps = self.source_maps.as_ref();
        let input_source_map = self.input_source_map.as_ref();
        let experimental = &jsc.experimental;
        let output = &jsc.output;
        if module.is_some_and(|module| module != &serde_json::json!({"type": "es6"}))
            || source_maps.is_some_and(|value| value != &Value::Bool(false))
            || input_source_map.is_some_and(|value| value != &Value::Bool(false))
            || jsc.keep_class_names
            || !output.preamble.is_empty()
            || experimental.emit_assert_for_import_attributes
            || experimental.keep_import_attributes
            || !matches!(output.charset.as_deref(), None | Some("utf8" | "ascii"))
        {
            return false;
        }
        let is_module = self.is_module.as_ref();
        matches!(is_module, None | Some(Value::Bool(_)))
            || matches!(is_module, Some(Value::String(v)) if v == "unknown" || v == "commonjs")
    }

    pub fn transformer_options(&self, path: &Path) -> TransformerOptions {
        let default_transform = Transform::default();
        let transform = self.jsc.transform.as_ref().unwrap_or(&default_transform);
        let react = &transform.react;
        let runtime = match react.runtime.as_deref() {
            Some("automatic") => JsxRuntime::Automatic,
            _ => JsxRuntime::Classic,
        };
        let lang = self.lang(path);
        let transform_jsx =
            matches!(lang, Lang::Jsx | Lang::Tsx) && react.runtime.as_deref() != Some("preserve");
        let jsx = transform_jsx.then(|| JsxOptions {
            runtime,
            development: react.development,
            source_self: react.development,
            import_source: react.import_source.clone(),
            pragma: react.pragma.clone(),
            pragma_frag: react.pragma_frag.clone(),
            throw_if_namespace: react.throw_if_namespace.unwrap_or(true),
            display_name: true,
            refresh: match &react.refresh {
                Some(Refresh::Enabled(true)) => Some(ReactRefreshOptions::default()),
                Some(Refresh::Options(options)) => Some(options.clone()),
                _ => None,
            },
            ..JsxOptions::default()
        });
        let typescript =
            matches!(lang, Lang::Ts | Lang::Tsx | Lang::Dts).then(|| TypeScriptOptions {
                verbatim_module_syntax: transform.verbatim_module_syntax,
                ..TypeScriptOptions::default()
            });
        let static_blocks = match &self.env {
            None => {
                // SWC defaults to ES5. Use the target to enable implemented
                // passes; missing compat passes are measured by the snapshot.
                !matches!(
                    self.jsc.target.as_deref(),
                    Some("es2022" | "es2023" | "es2024" | "esnext")
                )
            }
            Some(env) => {
                let feature = "transform-class-static-block";
                env.include.iter().any(|f| f == feature)
                    && !env.exclude.iter().any(|f| f == feature)
            }
        };
        let filename = path.file_name().unwrap().to_string_lossy().into_owned();
        TransformerOptions {
            // SWC's fixture snapshots normalize the source file to its basename.
            filename: Some(filename),
            jsx,
            typescript,
            assumptions: self.jsc.assumptions,
            es2022: ES2022Options { static_blocks },
            ..TransformerOptions::default()
        }
    }

    pub fn codegen_options(&self) -> CodegenOptions {
        CodegenOptions {
            minify: self.minify,
            ascii_only: self.jsc.output.charset.as_deref() == Some("ascii"),
            comments: self.jsc.preserve_all_comments,
            ..CodegenOptions::default()
        }
    }
}
