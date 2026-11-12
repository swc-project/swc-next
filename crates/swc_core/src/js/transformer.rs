#[cfg(feature = "bridge")]
use swc_next_bridge::bridge;
#[cfg(all(feature = "bridge", not(target_family = "wasm")))]
use swc_next_bridge::napi;

use std::collections::BTreeMap;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Diagnostic, Lang, Severity, SourceType};
use swc_next_ecma_codegen::{Options as GeneratorOptions, generate};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SyntaxChecks, analyze};
use swc_next_ecma_transformer::{
    Transformer, TransformerOptions,
    assumptions::Assumptions,
    jsx::{JsxOptions, JsxRuntime as TransformJsxRuntime, ReactRefreshOptions},
    normalize::Normalizer,
    proposal::ProposalOptions,
    typescript::{JsxImportOptions, TypeScriptOptions},
};

use crate::{
    BoolOr,
    js::{CommentsOption, JsOutput, JsTransformApiOptions, serialize_source_map},
};

#[cfg(all(feature = "bridge", feature = "serde"))]
use swc_next_ecma_ast::ffi::ast::{diagnostics_buffer_size, serialize_diagnostics_into};

#[cfg(all(feature = "bridge", feature = "serde"))]
mod plugins;
#[cfg(all(feature = "bridge", feature = "serde"))]
pub use plugins::{
    parse_for_transform_to_buffer, parse_transform_to_buffer, print_transformed_from_parts,
    transform_from_parts, transform_print_from_parts,
};

/// JavaScript and TypeScript transform options.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[cfg_attr(
    feature = "bridge",
    bridge(
        object,
        object_from_js = false,
        object_to_js = false,
        js_name = "TransformerConfig"
    )
)]
#[derive(Clone, Debug, Default)]
pub struct JsTransformerConfig {
    /// Source filename used by transforms, including JSX development metadata.
    /// Overrides the API's shared filename when set.
    pub filename: Option<String>,
    /// TypeScript transform. `false` keeps TypeScript syntax in the output.
    ///
    /// @default enabled when `lang` is `"ts"`, `"tsx"`, or `"dts"`
    #[cfg_attr(
        all(feature = "bridge", not(target_family = "wasm")),
        napi(ts_type = "boolean | TypeScriptConfig")
    )]
    pub typescript: Option<BoolOr<JsTypeScriptConfig>>,
    /// JSX transform. `false` keeps JSX syntax in the output.
    ///
    /// @default enabled with the automatic runtime when `lang` is `"jsx"` or `"tsx"`
    #[cfg_attr(
        all(feature = "bridge", not(target_family = "wasm")),
        napi(ts_type = "boolean | JsxConfig")
    )]
    pub jsx: Option<BoolOr<JsxConfig>>,
    /// Per-feature overrides. `true` enables a transform; `false` disables it.
    #[cfg_attr(
        all(feature = "bridge", not(target_family = "wasm")),
        napi(ts_type = "Partial<Record<JsFeature, boolean>>")
    )]
    pub features: Option<BTreeMap<JsFeature, bool>>,
    /// Babel-compatible assumptions.
    #[cfg_attr(
        all(feature = "bridge", not(target_family = "wasm")),
        napi(ts_type = "Assumptions")
    )]
    pub assumptions: Option<JsAssumptions>,
    /// Proposal transform options.
    #[cfg_attr(
        all(feature = "bridge", not(target_family = "wasm")),
        napi(ts_type = "ProposalConfig")
    )]
    pub proposal: Option<JsProposalConfig>,
    // pub decorators: Option<JsDecorators>,
    // pub module: Option<JsModuleOptions>, // Defaults to the preserve format.
    // pub define: Option<BTreeMap<String, String>>, // Global expression replacements.
    // pub helpers: Option<JsHelpers>, // Defaults to inline helpers.
}

impl JsTransformerConfig {
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
            filename: self.filename.or(filename),
            assumptions: self
                .assumptions
                .map(Into::into)
                .unwrap_or(defaults.assumptions),
            typescript: enabled_options(self.typescript),
            jsx: enabled_options(self.jsx),
            proposal: self.proposal.map(Into::into).unwrap_or(defaults.proposal),
            ..defaults
        }
    }
}

/// Proposal transform options.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[cfg_attr(
    feature = "bridge",
    bridge(
        object,
        object_from_js = false,
        object_to_js = false,
        js_name = "ProposalConfig"
    )
)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JsProposalConfig {
    /// Preserve class static block behavior when a block makes the class
    /// object non-extensible with `Object.preventExtensions`, `Object.seal`,
    /// or `Object.freeze`.
    ///
    /// Applies when class static block lowering is enabled.
    ///
    /// @default false
    pub prevent_extensions: Option<bool>,
}

impl From<JsProposalConfig> for ProposalOptions {
    fn from(options: JsProposalConfig) -> Self {
        Self {
            prevent_extensions: options.prevent_extensions.unwrap_or_default(),
        }
    }
}

/// Names of individually controllable lowering transforms.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(
    all(feature = "bridge", not(target_family = "wasm")),
    swc_next_bridge::napi_derive::napi(
        string_enum = "kebab-case",
        object_from_js = false,
        object_to_js = false
    )
)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum JsFeature {
    /// Lower class static blocks (`class-static-block`).
    ClassStaticBlock,
    /// Add a binding to catch clauses (`optional-catch-binding`).
    OptionalCatchBinding,
}

/// TypeScript transform options.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[cfg_attr(
    feature = "bridge",
    bridge(
        object,
        object_from_js = false,
        object_to_js = false,
        js_name = "TypeScriptConfig"
    )
)]
#[derive(Clone, Debug, Default)]
pub struct JsTypeScriptConfig {
    /// TypeScript's `verbatimModuleSyntax`: preserve value imports and exports
    /// and disable const enum inlining and removal.
    ///
    /// @default false
    pub verbatim_module_syntax: Option<bool>,
    /// TypeScript's `preserveConstEnums`: preserve declarations without disabling inlining.
    ///
    /// @default false
    pub preserve_const_enums: Option<bool>,
    /// JSX runtime used to determine which factory imports to retain when JSX
    /// syntax is preserved. Does not enable JSX lowering. When JSX lowering is
    /// enabled, its runtime takes precedence.
    ///
    /// @default "classic"
    pub jsx_runtime: Option<JsxRuntime>,
    /// Classic JSX factory used to preserve its value import when JSX syntax
    /// is retained.
    ///
    /// @default "React.createElement"
    pub jsx_pragma: Option<String>,
    /// Classic JSX fragment factory used to preserve its value import when JSX
    /// syntax is retained.
    ///
    /// @default "React.Fragment"
    pub jsx_pragma_frag: Option<String>,
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

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(feature = "bridge", bridge(string_enum = "lowercase"))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum JsxRuntime {
    Classic,
    #[default]
    Automatic,
}

impl From<JsxRuntime> for TransformJsxRuntime {
    fn from(runtime: JsxRuntime) -> Self {
        match runtime {
            JsxRuntime::Classic => Self::Classic,
            JsxRuntime::Automatic => Self::Automatic,
        }
    }
}

/// JSX options targeting Babel 8. `useSpread` and `useBuiltIns` are not supported.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[cfg_attr(
    feature = "bridge",
    bridge(
        object,
        object_from_js = false,
        object_to_js = false,
        js_name = "JsxConfig"
    )
)]
#[derive(Clone, Debug, Default)]
pub struct JsxConfig {
    /// @default "automatic"
    pub runtime: Option<JsxRuntime>,
    /// @default false
    pub development: Option<bool>,
    /// Add `__source` and `__self` in development.
    ///
    /// @default false
    pub source_self: Option<bool>,
    /// Automatic runtime import source.
    ///
    /// @default "react"
    pub import_source: Option<String>,
    /// Classic runtime factory.
    ///
    /// @default "React.createElement"
    pub pragma: Option<String>,
    /// Classic runtime fragment factory.
    ///
    /// @default "React.Fragment"
    pub pragma_frag: Option<String>,
    /// @default true
    pub throw_if_namespace: Option<bool>,
    /// Mark factory calls as pure.
    pub pure: Option<bool>,
    /// Add `displayName` to `createReactClass` calls.
    ///
    /// @default false
    pub display_name: Option<bool>,
    /// React Refresh. `true` uses default options.
    ///
    /// @default false
    #[cfg_attr(
        all(feature = "bridge", not(target_family = "wasm")),
        napi(ts_type = "boolean | ReactRefreshConfig")
    )]
    pub refresh: Option<BoolOr<JsReactRefreshConfig>>,
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

/// React Refresh options.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[cfg_attr(
    feature = "bridge",
    bridge(
        object,
        object_from_js = false,
        object_to_js = false,
        js_name = "ReactRefreshConfig"
    )
)]
#[derive(Clone, Debug, Default)]
pub struct JsReactRefreshConfig {
    /// @default "$RefreshReg$"
    pub refresh_reg: Option<String>,
    /// @default "$RefreshSig$"
    pub refresh_sig: Option<String>,
    /// @default false
    pub emit_full_signatures: Option<bool>,
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

/// Babel assumptions. All default to false.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[cfg_attr(
    feature = "bridge",
    bridge(
        object,
        object_from_js = false,
        object_to_js = false,
        js_name = "Assumptions"
    )
)]
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

/// Transform a source file.
pub fn transform_js(source: &str, options: JsTransformApiOptions) -> JsOutput {
    let mut output = JsOutput::default();
    fn has_errors(diagnostics: &[Diagnostic]) -> bool {
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

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
        // An explicit parser comment policy wins. Otherwise retain JSX pragmas
        // for semantic transforms, then discard comments before generation if needed.
        let jsx_enabled = !matches!(&transform.jsx, None | Some(BoolOr::Bool(false)));
        let typescript_enabled = !matches!(&transform.typescript, None | Some(BoolOr::Bool(false)));
        let collect_pragmas = parser_options.comments.is_none()
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
    match &transform.typescript {
        None | Some(BoolOr::Bool(false)) => Normalizer::normalize(&mut ast),
        _ => Normalizer::normalize_runtime(&mut ast),
    }

    // ======================= Analyze =======================
    let semantic = {
        let check_syntax = if parser_options.semantic_errors.unwrap_or(false) {
            SyntaxChecks::all()
        } else {
            SyntaxChecks::empty()
        };
        let options = AnalyzeOptions { check_syntax };
        let analyzed = analyze(&mut ast, options);
        output.diagnostics.extend(analyzed.diagnostics);
        if has_errors(&output.diagnostics) {
            return output;
        }
        analyzed.semantic
    };

    // ======================= Transform =======================
    {
        let transform = transform.to_transformer_options(options.filename.clone());
        let transformed = Transformer::transform(semantic, transform);
        output.diagnostics.extend(transformed.diagnostics);
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

/// Transform source and encode the result for the JavaScript bindings.
///
/// The buffer contains three little-endian `u32` values (code byte length, source
/// map byte length, diagnostic count), then UTF-8 code, an optional UTF-8 source
/// map, and packed diagnostics with UTF-16 source offsets.
///
/// # Safety
///
/// `source` must contain valid UTF-8, and `source_utf16_len` must match its
/// UTF-16 code unit length.
#[cfg(all(feature = "bridge", feature = "serde"))]
pub unsafe fn transform_source_to_buffer(
    source: &[u8],
    source_utf16_len: u32,
    options: &str,
) -> Result<Vec<u8>, String> {
    let options: JsTransformApiOptions = serde_json::from_str(options)
        .map_err(|error| format!("Invalid transform options: {error}"))?;
    // SAFETY: The caller guarantees valid UTF-8.
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    let result = transform_js(source, options);
    Ok(encode_output(&result, source, source_utf16_len))
}

#[cfg(all(feature = "bridge", feature = "serde"))]
fn encode_output(result: &JsOutput, source: &str, source_utf16_len: u32) -> Vec<u8> {
    let code = result.code.as_bytes();
    let map = result.map.as_deref().unwrap_or_default().as_bytes();
    let code_len = u32::try_from(code.len()).expect("generated code length exceeds u32");
    let map_len = u32::try_from(map.len()).expect("source map length exceeds u32");
    let diagnostic_count =
        u32::try_from(result.diagnostics.len()).expect("diagnostic count exceeds u32");
    let diagnostics_offset = 12 + code.len() + map.len();
    let mut output = vec![0; diagnostics_offset + diagnostics_buffer_size(&result.diagnostics)];
    output[..4].copy_from_slice(&code_len.to_le_bytes());
    output[4..8].copy_from_slice(&map_len.to_le_bytes());
    output[8..12].copy_from_slice(&diagnostic_count.to_le_bytes());
    output[12..12 + code.len()].copy_from_slice(code);
    output[12 + code.len()..diagnostics_offset].copy_from_slice(map);
    serialize_diagnostics_into(
        &result.diagnostics,
        source,
        &mut output[diagnostics_offset..],
        source_utf16_len as usize,
    );
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::js::parser::JsParserConfig;

    #[test]
    fn comment_options() {
        let remove_output = JsTransformApiOptions {
            comments: Some(CommentsOption::None),
            parser: Some(JsParserConfig {
                comments: Some(CommentsOption::All),
                ..Default::default()
            }),
            ..Default::default()
        };
        let skip_collection = JsTransformApiOptions {
            comments: Some(CommentsOption::All),
            parser: Some(JsParserConfig {
                comments: Some(CommentsOption::None),
                ..Default::default()
            }),
            ..Default::default()
        };

        let output = transform_js("/* comment */ value();", JsTransformApiOptions::default());
        assert!(output.code.contains("/* comment */"));

        let output = transform_js("/* comment */ value();", remove_output);
        assert_eq!(output.code, "value();\n");

        let output = transform_js("/* comment */ value();", skip_collection);
        assert_eq!(output.code, "value();\n");
    }
}
