use std::collections::BTreeMap;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Diagnostic, Label, Severity, Span};
use swc_next_ecma_codegen::{Options as CodegenOptions, SourceMap, print};
use swc_next_ecma_minifier::{Minifier, MinifierOptions};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze};
use swc_next_ecma_transformer::{
    Transformer, TransformerOptions,
    assumptions::Assumptions,
    jsx::{JsxOptions as TransformJsxOptions, ReactRefreshOptions},
    normalize::Normalizer,
    typescript::TypeScriptOptions,
};

use crate::{
    BoolOr,
    js::{CommentsOption, JsOptions, JsOutput},
};

pub use swc_next_ecma_transformer::jsx::JsxRuntime;

/// Currently implemented JavaScript and TypeScript transforms.
#[derive(Clone, Debug, Default)]
pub struct JsTransformOptions {
    /// Disable TypeScript lowering or enable it with defaults or explicit settings.
    pub typescript: Option<BoolOr<JsTypeScriptOptions>>,
    /// Disable JSX lowering or enable it with defaults or explicit settings.
    pub jsx: Option<BoolOr<JsxOptions>>,
    /// Explicitly enable or disable implemented lowering transforms.
    pub features: Option<BTreeMap<JsFeature, bool>>,
    /// Assumptions forwarded to the transformer. Unset fields use its defaults.
    pub assumptions: Option<JsAssumptions>,
    // pub decorators: Option<JsDecorators>,
    // pub module: Option<JsModuleOptions>, // Defaults to the preserve format.
    // pub define: Option<BTreeMap<String, String>>, // Global expression replacements.
    // pub helpers: Option<JsHelpers>, // Defaults to inline helpers.
}

impl JsTransformOptions {
    pub fn to_transformer_options(self, filename: Option<String>) -> TransformerOptions {
        let mut defaults = TransformerOptions::default();
        if let Some(features) = self.features {
            for (feature, enabled) in features {
                match feature {
                    JsFeature::ClassStaticBlock => defaults.es2022.static_blocks = enabled,
                    JsFeature::OptionalCatchBinding => {
                        defaults.es2019.optional_catch_binding = enabled
                    }
                }
            }
        }
        TransformerOptions {
            filename,
            assumptions: self
                .assumptions
                .map(Into::into)
                .unwrap_or(defaults.assumptions),
            typescript: enabled_options(self.typescript),
            jsx: enabled_options(self.jsx),
            ..defaults
        }
    }
}

/// Implemented, individually controllable JavaScript transforms.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum JsFeature {
    /// Lower class static blocks (`class-static-block`).
    ClassStaticBlock,
    /// Add a binding to catch clauses (`optional-catch-binding`).
    OptionalCatchBinding,
}

/// Public TypeScript lowering settings.
///
/// JSX import dependencies used internally by the transformer are derived from
/// the JSX settings rather than exposed in this section.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsTypeScriptOptions {
    /// Preserve value imports and exports, and disable const-enum inlining and
    /// removal.
    pub verbatim_module_syntax: Option<bool>,
    /// Preserve const-enum declarations without disabling inlining.
    pub preserve_const_enums: Option<bool>,
}

impl From<JsTypeScriptOptions> for TypeScriptOptions {
    fn from(options: JsTypeScriptOptions) -> Self {
        let defaults = Self::default();
        Self {
            verbatim_module_syntax: options
                .verbatim_module_syntax
                .unwrap_or(defaults.verbatim_module_syntax),
            preserve_const_enums: options
                .preserve_const_enums
                .unwrap_or(defaults.preserve_const_enums),
            ..defaults
        }
    }
}

/// JSX lowering settings targeting Babel 8.
#[derive(Clone, Debug, Default)]
pub struct JsxOptions {
    pub runtime: Option<JsxRuntime>,
    pub development: Option<bool>,
    /// Include development source and self metadata.
    pub source_self: Option<bool>,
    /// Import source for the automatic runtime.
    pub import_source: Option<String>,
    /// Factory expression for the classic runtime.
    pub pragma: Option<String>,
    /// Fragment factory expression for the classic runtime.
    pub pragma_frag: Option<String>,
    pub throw_if_namespace: Option<bool>,
    /// Mark factory calls as pure. The default depends on whether a custom
    /// pragma is supplied.
    pub pure: Option<bool>,
    /// Add display names to `createReactClass` calls.
    pub display_name: Option<bool>,
    pub refresh: Option<BoolOr<JsReactRefreshOptions>>,
}

impl From<JsxOptions> for TransformJsxOptions {
    fn from(options: JsxOptions) -> Self {
        let defaults = Self::default();
        Self {
            runtime: options.runtime.unwrap_or(defaults.runtime),
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

/// React Refresh instrumentation settings.
#[derive(Clone, Debug, Default)]
pub struct JsReactRefreshOptions {
    /// Registration expression. Defaults to `$RefreshReg$` when enabled.
    pub refresh_reg: Option<String>,
    /// Signature expression. Defaults to `$RefreshSig$` when enabled.
    pub refresh_sig: Option<String>,
    pub emit_full_signatures: Option<bool>,
}

impl From<JsReactRefreshOptions> for ReactRefreshOptions {
    fn from(options: JsReactRefreshOptions) -> Self {
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

fn enabled_options<T, U: Default + From<T>>(options: Option<BoolOr<T>>) -> Option<U> {
    match options {
        None | Some(BoolOr::Bool(false)) => None,
        Some(BoolOr::Bool(true)) => Some(U::default()),
        Some(BoolOr::Options(options)) => Some(options.into()),
    }
}

// pub enum JsDecorators {
//     Disabled, // false
//     Options(JsDecoratorOptions),
// }
//
// pub struct JsDecoratorOptions {
//     pub version: JsDecoratorVersion,
//     pub metadata: Option<bool>, // Legacy decorators only; defaults to false.
// }
//
// pub enum JsDecoratorVersion {
//     V2023_11, // "2023-11"
//     Legacy, // "legacy"
// }

/// All resolved fields default to false.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsAssumptions {
    pub array_like_is_iterable: Option<bool>,
    pub constant_reexports: Option<bool>,
    pub constant_super: Option<bool>,
    pub enumerable_module_meta: Option<bool>,
    pub ignore_function_length: Option<bool>,
    pub ignore_function_name: Option<bool>,
    pub ignore_to_primitive_hint: Option<bool>,
    pub iterable_is_array: Option<bool>,
    pub mutable_template_object: Option<bool>,
    pub no_class_calls: Option<bool>,
    pub no_document_all: Option<bool>,
    pub no_incomplete_ns_import_detection: Option<bool>,
    pub no_new_arrows: Option<bool>,
    pub object_rest_no_symbols: Option<bool>,
    pub private_fields_as_properties: Option<bool>,
    pub pure_getters: Option<bool>,
    pub set_class_methods: Option<bool>,
    pub set_computed_properties: Option<bool>,
    pub set_public_class_fields: Option<bool>,
    pub set_spread_properties: Option<bool>,
    pub skip_for_of_iterator_closing: Option<bool>,
    pub super_is_callable_constructor: Option<bool>,
}

impl From<JsAssumptions> for Assumptions {
    fn from(options: JsAssumptions) -> Self {
        let mut assumptions = Self::default();
        assumptions.array_like_is_iterable = options
            .array_like_is_iterable
            .unwrap_or(assumptions.array_like_is_iterable);
        assumptions.constant_reexports = options
            .constant_reexports
            .unwrap_or(assumptions.constant_reexports);
        assumptions.constant_super = options.constant_super.unwrap_or(assumptions.constant_super);
        assumptions.enumerable_module_meta = options
            .enumerable_module_meta
            .unwrap_or(assumptions.enumerable_module_meta);
        assumptions.ignore_function_length = options
            .ignore_function_length
            .unwrap_or(assumptions.ignore_function_length);
        assumptions.ignore_function_name = options
            .ignore_function_name
            .unwrap_or(assumptions.ignore_function_name);
        assumptions.ignore_to_primitive_hint = options
            .ignore_to_primitive_hint
            .unwrap_or(assumptions.ignore_to_primitive_hint);
        assumptions.iterable_is_array = options
            .iterable_is_array
            .unwrap_or(assumptions.iterable_is_array);
        assumptions.mutable_template_object = options
            .mutable_template_object
            .unwrap_or(assumptions.mutable_template_object);
        assumptions.no_class_calls = options.no_class_calls.unwrap_or(assumptions.no_class_calls);
        assumptions.no_document_all = options
            .no_document_all
            .unwrap_or(assumptions.no_document_all);
        assumptions.no_incomplete_ns_import_detection = options
            .no_incomplete_ns_import_detection
            .unwrap_or(assumptions.no_incomplete_ns_import_detection);
        assumptions.no_new_arrows = options.no_new_arrows.unwrap_or(assumptions.no_new_arrows);
        assumptions.object_rest_no_symbols = options
            .object_rest_no_symbols
            .unwrap_or(assumptions.object_rest_no_symbols);
        assumptions.private_fields_as_properties = options
            .private_fields_as_properties
            .unwrap_or(assumptions.private_fields_as_properties);
        assumptions.pure_getters = options.pure_getters.unwrap_or(assumptions.pure_getters);
        assumptions.set_class_methods = options
            .set_class_methods
            .unwrap_or(assumptions.set_class_methods);
        assumptions.set_computed_properties = options
            .set_computed_properties
            .unwrap_or(assumptions.set_computed_properties);
        assumptions.set_public_class_fields = options
            .set_public_class_fields
            .unwrap_or(assumptions.set_public_class_fields);
        assumptions.set_spread_properties = options
            .set_spread_properties
            .unwrap_or(assumptions.set_spread_properties);
        assumptions.skip_for_of_iterator_closing = options
            .skip_for_of_iterator_closing
            .unwrap_or(assumptions.skip_for_of_iterator_closing);
        assumptions.super_is_callable_constructor = options
            .super_is_callable_constructor
            .unwrap_or(assumptions.super_is_callable_constructor);
        assumptions
    }
}

// pub struct JsModuleOptions {
//     pub r#type: JsModuleType,
// }
//
// pub enum JsModuleType {
//     Preserve,
//     CommonJs,
// }
//
// pub enum JsHelpers {
//     Inline,
//     External, // Import from @swc/helpers.
// }

/// Transform source code using the currently implemented JavaScript stages.
pub fn transform_js(source: &str, options: JsOptions) -> JsOutput {
    let ParserOptions {
        source_type, lang, ..
    } = options.parser.unwrap_or_default().into();
    let allocator = Allocator::default();
    let parsed = Parser::init(
        &allocator,
        source,
        ParserOptions {
            source_type,
            lang,
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    let mut output = JsOutput {
        diagnostics: parsed.diagnostics,
        ..JsOutput::default()
    };
    if has_errors(&output.diagnostics) {
        return output;
    }

    let mut ast = parsed.ast;
    let mut transform = options.transform.unwrap_or_default();
    if transform.typescript.is_none() && lang.is_typescript() {
        transform.typescript = Some(BoolOr::Bool(true));
    }
    if transform.jsx.is_none() && lang.is_jsx() {
        transform.jsx = Some(BoolOr::Bool(true));
    }
    let source_map_path = options
        .source_maps
        .unwrap_or(false)
        .then(|| options.filename.as_deref().unwrap_or("input.js").into());
    let transform = transform.to_transformer_options(options.filename);
    let minify: Option<MinifierOptions> = enabled_options(options.minify);
    if minify
        .as_ref()
        .is_some_and(|options| options.compress.is_some() || options.mangle)
        && ((lang.is_typescript() && transform.typescript.is_none())
            || (lang.is_jsx() && transform.jsx.is_none()))
    {
        output.diagnostics.push(Diagnostic::error(
            "AST minification requires TypeScript and JSX lowering",
            Span::new(0, source.len() as u32),
        ));
        return output;
    }
    if transform.typescript.is_some() {
        Normalizer::normalize_runtime(&mut ast);
    } else {
        Normalizer::normalize(&mut ast);
    }
    let analyzed = analyze(
        &mut ast,
        AnalyzeOptions {
            check_syntax: SyntaxChecks::all(),
        },
    );
    output
        .diagnostics
        .extend(analyzed.diagnostics.into_iter().map(own_diagnostic));
    if has_errors(&output.diagnostics) {
        return output;
    }
    let transformed = Transformer::transform(analyzed.semantic, transform);
    output.diagnostics.extend(transformed.diagnostics);
    if has_errors(&output.diagnostics) {
        return output;
    }

    let mut codegen = options.codegen.unwrap_or_default();
    codegen.minify.get_or_insert(minify.is_some());
    let preserve_comments = match options.comments {
        Some(CommentsOption::All) => true,
        Some(CommentsOption::None) => false,
        None => minify.is_none(),
    };
    if let Some(minify) = minify {
        Minifier::minify(&mut ast, minify);
    }
    let mut codegen = CodegenOptions::from(codegen);
    codegen.source_map_path = source_map_path;
    codegen.comments = preserve_comments;
    let printed = print(&ast, codegen);
    output.map = printed.map.as_ref().map(serialize_source_map);
    output.code = printed.code;
    output
}

fn serialize_source_map(map: &SourceMap<'_>) -> String {
    // The pipeline supplies UTF-8 source and filenames; codegen records names
    // from &str and emits ASCII VLQ mappings.
    let text = |bytes| std::str::from_utf8(bytes).expect("source map strings must be valid UTF-8");
    let mut value = serde_json::json!({
        "version": map.version(),
        "sources": map.sources().iter().map(|source| text(source)).collect::<Vec<_>>(),
        "names": map.names().iter().map(|name| text(name)).collect::<Vec<_>>(),
        "mappings": text(map.mappings()),
    });
    if let Some(file) = map.file() {
        value["file"] = text(file).into();
    }
    if let Some(root) = map.source_root() {
        value["sourceRoot"] = text(root).into();
    }
    if let Some(sources) = map.sources_content() {
        value["sourcesContent"] = serde_json::json!(
            sources
                .iter()
                .map(|source| source.map(text))
                .collect::<Vec<_>>()
        );
    }
    value.to_string()
}

fn has_errors(diagnostics: &[Diagnostic<'_>]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
}

fn own_diagnostic(diagnostic: Diagnostic<'_>) -> Diagnostic<'static> {
    Diagnostic {
        severity: diagnostic.severity,
        message: diagnostic.message.into_owned().into(),
        span: diagnostic.span,
        help: diagnostic.help.map(|help| help.into_owned().into()),
        labels: diagnostic
            .labels
            .into_iter()
            .map(|label| Label {
                span: label.span,
                message: label.message.into_owned().into(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use sourcemap::SourceMap as DecodedSourceMap;

    use super::{
        JsFeature, JsReactRefreshOptions, JsTransformOptions, JsxOptions, JsxRuntime, transform_js,
    };
    use crate::{
        BoolOr,
        js::{
            CommentsOption, JsOptions,
            codegen::JsCodegenOptions,
            minify::{JsCompressOptions, JsMinifyOptions},
            parser::{JsCommentMode, JsLang, JsParserOptions},
        },
    };

    fn options(lang: JsLang) -> JsOptions {
        JsOptions {
            parser: Some(JsParserOptions {
                lang: Some(lang),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn code(source: &str, options: &JsOptions) -> String {
        let output = transform_js(source, options.clone());
        assert!(output.diagnostics.is_empty());
        assert!(output.map.is_none());
        assert!(output.legal_comments.is_none());
        output.code
    }

    fn mapped_code(source: &str, options: &JsOptions) -> (String, DecodedSourceMap) {
        let output = transform_js(source, options.clone());
        assert!(output.diagnostics.is_empty());
        let json = output.map.unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["version"], 3);
        let map = DecodedSourceMap::from_slice(json.as_bytes()).unwrap();
        (output.code, map)
    }

    #[test]
    fn source_maps() {
        let source = "/*😀*/ export const 名 = \"\\\"\";\n";
        let filename = "目录/\"源文件\".js";
        let mut mapped = JsOptions {
            filename: Some(filename.into()),
            source_maps: Some(true),
            ..Default::default()
        };
        for minify in [false, true] {
            mapped.minify = Some(BoolOr::Bool(minify));
            let (output, map) = mapped_code(source, &mapped);
            assert_eq!(map.get_source(0), Some(filename));
            assert_eq!(map.get_source_contents(0), Some(source));
            let prefix = &output[..output.find('名').unwrap()];
            let line = prefix.bytes().filter(|&byte| byte == b'\n').count() as u32;
            let column = prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32;
            let token = map.lookup_token(line, column).unwrap();
            assert_eq!(token.get_name(), Some("名"));
            assert_eq!(token.get_src_line(), 0);
            assert_eq!(
                token.get_src_col(),
                source[..source.find('名').unwrap()].encode_utf16().count() as u32
            );
        }

        let mut typescript = options(JsLang::Ts);
        typescript.source_maps = Some(true);
        let source = "export const value: number = 1;";
        let (output, map) = mapped_code(source, &typescript);
        assert_eq!(map.get_source(0), Some("input.js"));
        assert_eq!(map.get_source_contents(0), Some(source));
        assert!(!output.contains(": number"));
        let column = output.find('1').unwrap() as u32;
        let token = map.lookup_token(0, column).unwrap();
        assert_eq!(token.get_src_col(), source.find('1').unwrap() as u32);

        mapped.source_maps = Some(false);
        let output = transform_js("export const value = 1;", mapped);
        assert!(output.diagnostics.is_empty());
        assert!(output.map.is_none());

        let output = transform_js("const = ;", typescript);
        assert!(!output.diagnostics.is_empty());
        assert!(output.map.is_none());
    }

    #[test]
    fn comment_options() {
        let source = "// ordinary\n/*! legal */\nexport const value = 1 + 2;";
        for minify in [false, true] {
            let mut options = JsOptions {
                minify: Some(BoolOr::Bool(minify)),
                ..Default::default()
            };
            let output = code(source, &options);
            assert_eq!(output.contains("ordinary"), !minify);
            assert_eq!(output.contains("legal"), !minify);

            options.comments = Some(CommentsOption::All);
            let output = code(source, &options);
            assert!(output.contains("ordinary"));
            assert!(output.contains("legal"));

            options.comments = Some(CommentsOption::None);
            let output = code(source, &options);
            assert!(!output.contains("ordinary"));
            assert!(!output.contains("legal"));
        }

        let mut printed = JsOptions {
            comments: Some(CommentsOption::All),
            ..Default::default()
        };
        for compact in [false, true] {
            printed.codegen = Some(JsCodegenOptions {
                minify: Some(compact),
                ..Default::default()
            });
            let output = code("/*#__PURE__*/ call();", &printed);
            assert!(output.contains("#__PURE__"));

            printed.comments = Some(CommentsOption::None);
            let output = code("/*#__PURE__*/ call();", &printed);
            assert!(!output.contains("#__PURE__"));
            printed.comments = Some(CommentsOption::All);
        }

        let mut jsx = options(JsLang::Jsx);
        jsx.comments = Some(CommentsOption::None);
        let output = code("/** @jsxRuntime classic */ const node = <div />;", &jsx);
        assert!(output.contains("React.createElement("));
        assert!(!output.contains("/*"));
    }

    #[test]
    fn syntax_options() {
        let mut typescript = options(JsLang::Ts);
        let output = code("const value: number = 1;", &typescript);
        assert_eq!(output.trim(), "const value = 1;\nexport {};");

        typescript.transform = Some(JsTransformOptions {
            typescript: Some(BoolOr::Bool(false)),
            ..Default::default()
        });
        let output = code("const value: number = 1;", &typescript);
        assert_eq!(output.trim(), "const value: number = 1;");

        let mut jsx = options(JsLang::Jsx);
        let output = code("const node = <div />;", &jsx);
        assert!(output.contains("react/jsx-runtime"));

        jsx.transform = Some(JsTransformOptions {
            jsx: Some(BoolOr::Bool(false)),
            ..Default::default()
        });
        let output = code("const node = <div />;", &jsx);
        assert!(output.contains("<div"));

        jsx.transform = Some(JsTransformOptions {
            jsx: Some(BoolOr::Options(JsxOptions {
                runtime: Some(JsxRuntime::Classic),
                pragma: Some("h".into()),
                pure: Some(false),
                ..Default::default()
            })),
            ..Default::default()
        });
        let output = code("const node = <div />;", &jsx);
        assert_eq!(output.trim(), "const node = h(\"div\", null);");

        let tsx = options(JsLang::Tsx);
        let output = code("const node: unknown = <div />;", &tsx);
        assert!(!output.contains(": unknown"));
        assert!(output.contains("react/jsx-runtime"));
    }

    #[test]
    fn lowering_features() {
        let mut options = JsOptions::default();
        let output = code("try {} catch {}", &options);
        assert!(output.contains("catch {"));

        options.transform = Some(JsTransformOptions {
            features: Some(
                [
                    (JsFeature::OptionalCatchBinding, true),
                    (JsFeature::ClassStaticBlock, true),
                ]
                .into(),
            ),
            ..Default::default()
        });
        let output = code("try {} catch {}", &options);
        assert!(output.contains("catch ("));

        let output = code("class Value { static { use(this); } }", &options);
        assert!(!output.contains("static {"));

        options.transform.as_mut().unwrap().features = Some(
            [
                (JsFeature::OptionalCatchBinding, false),
                (JsFeature::ClassStaticBlock, false),
            ]
            .into(),
        );
        let output = code("try {} catch {}", &options);
        assert!(output.contains("catch {"));

        let output = code("class Value { static { use(this); } }", &options);
        assert!(output.contains("static {"));
    }

    #[test]
    fn minification_options() {
        let mut options = JsOptions {
            minify: Some(BoolOr::Bool(true)),
            ..Default::default()
        };
        let output = code("const value = 1 + 2;", &options);
        assert_eq!(output, "const value=3;");

        options.codegen = Some(JsCodegenOptions {
            minify: Some(false),
            ..Default::default()
        });
        let output = code("const value = 1 + 2;", &options);
        assert_eq!(output.trim(), "const value = 3;");

        options.codegen = Some(JsCodegenOptions {
            minify: Some(true),
            ..Default::default()
        });
        options.minify = Some(BoolOr::Bool(false));
        let output = code("const value = 1 + 2;", &options);
        assert_eq!(output, "const value=1+2;");

        options.minify = Some(BoolOr::Options(JsMinifyOptions {
            compress: Some(BoolOr::Bool(false)),
            mangle: Some(false),
        }));
        let output = code("const value = 1 + 2;", &options);
        assert_eq!(output, "const value=1+2;");

        options.minify = Some(BoolOr::Options(JsMinifyOptions {
            compress: Some(BoolOr::Options(JsCompressOptions {
                evaluate: Some(false),
                ..Default::default()
            })),
            mangle: Some(false),
        }));
        let output = code("const value = 1 + 2;", &options);
        assert_eq!(output, "const value=1+2;");
    }

    #[test]
    fn jsx_metadata() {
        let mut options = options(JsLang::Jsx);
        options.filename = Some("component.jsx".into());
        options.transform = Some(JsTransformOptions {
            jsx: Some(BoolOr::Options(JsxOptions {
                development: Some(true),
                source_self: Some(true),
                refresh: Some(BoolOr::Options(JsReactRefreshOptions {
                    refresh_reg: Some("register".into()),
                    ..Default::default()
                })),
                ..Default::default()
            })),
            ..Default::default()
        });
        let output = code("export function App() { return <div />; }", &options);
        assert!(output.contains("component.jsx"));
        assert!(output.contains("register("));
        assert!(output.contains("react/jsx-dev-runtime"));
    }

    #[test]
    fn pipeline_diagnostics() {
        let output = transform_js("const = ;", JsOptions::default());
        assert!(output.code.is_empty());
        assert!(!output.diagnostics.is_empty());

        let output = transform_js("let value; let value;", JsOptions::default());
        assert!(output.code.is_empty());
        assert!(!output.diagnostics.is_empty());

        let output = transform_js("const node = <svg:path />;", options(JsLang::Jsx));
        assert!(output.code.is_empty());
        assert!(!output.diagnostics.is_empty());

        let mut preserved = options(JsLang::Ts);
        preserved.transform = Some(JsTransformOptions {
            typescript: Some(BoolOr::Bool(false)),
            ..Default::default()
        });
        preserved.minify = Some(BoolOr::Bool(true));
        let output = transform_js("const value: number = 1;", preserved);
        assert!(output.code.is_empty());
        assert!(!output.diagnostics.is_empty());
    }

    #[test]
    fn pipeline_parser_settings() {
        let mut options = options(JsLang::Jsx);
        options.parser.as_mut().unwrap().comments = Some(JsCommentMode::None);
        let output = code("/** @jsxRuntime classic */ const node = <div />;", &options);
        assert!(output.contains("React.createElement("));
        assert!(!output.contains("react/jsx-runtime"));

        options.parser.as_mut().unwrap().semantic_errors = Some(false);
        let output = transform_js("let value; let value;", options);
        assert!(output.code.is_empty());
        assert!(!output.diagnostics.is_empty());
    }
}
