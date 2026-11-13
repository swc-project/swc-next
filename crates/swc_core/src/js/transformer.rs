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

#[cfg(feature = "bridge")]
use swc_next_bridge::bridge;

#[cfg(all(feature = "bridge", not(target_family = "wasm")))]
use swc_next_bridge::napi;

/// JavaScript and TypeScript transform options.
#[cfg_attr(
    feature = "bridge",
    bridge(
        object,
        object_from_js = false,
        object_to_js = false,
        js_name = "TransformerConfig"
    )
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
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
        napi(
            ts_type = "Partial<Record<\"class-static-block\" | \"optional-catch-binding\", boolean>>"
        )
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
#[cfg_attr(
    feature = "bridge",
    bridge(
        object,
        object_from_js = false,
        object_to_js = false,
        js_name = "ProposalConfig"
    )
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
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
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum JsFeature {
    /// Lower class static blocks (`class-static-block`).
    ClassStaticBlock,
    /// Add a binding to catch clauses (`optional-catch-binding`).
    OptionalCatchBinding,
}

/// TypeScript transform options.
#[cfg_attr(
    feature = "bridge",
    bridge(
        object,
        object_from_js = false,
        object_to_js = false,
        js_name = "TypeScriptConfig"
    )
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
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

#[cfg_attr(feature = "bridge", bridge(string_enum = "lowercase"))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
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
#[cfg_attr(
    feature = "bridge",
    bridge(
        object,
        object_from_js = false,
        object_to_js = false,
        js_name = "JsxConfig"
    )
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
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
#[cfg_attr(
    feature = "bridge",
    bridge(
        object,
        object_from_js = false,
        object_to_js = false,
        js_name = "ReactRefreshConfig"
    )
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
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
#[cfg_attr(
    feature = "bridge",
    bridge(
        object,
        object_from_js = false,
        object_to_js = false,
        js_name = "Assumptions"
    )
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
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

#[cfg(feature = "bridge")]
pub mod bridge {
    //! Binary source transformation bridge. Options use a 48-byte little-endian
    //! header, optionally followed by nine length-prefixed UTF-8 strings. Empty options
    //! select defaults. Output starts with code length, optional map length
    //! (`u32::MAX` means absent), and diagnostic count, followed by UTF-8 code/map
    //! and the parser bridge's packed diagnostic records.

    use swc_next_bridge::{BridgeDecode, BridgeDecodeOptional, bridge};
    use swc_next_ecma_ast::ffi::ast::{diagnostics_buffer_size, serialize_diagnostics};

    use super::{
        JsAssumptions, JsFeature, JsProposalConfig, JsReactRefreshConfig, JsTransformerConfig,
        JsTypeScriptConfig, JsxConfig, JsxRuntime, transform_js,
    };
    use crate::{
        BoolOr,
        js::{
            CommentsOption, JsOutput, JsTransformApiOptions, generator::JsGeneratorConfig,
            parser::JsParserConfig,
        },
    };

    #[cfg(not(target_family = "wasm"))]
    use swc_next_bridge::napi;

    #[bridge(object, object_from_js = false, object_to_js = false)]
    pub struct TransformOutput {
        pub code: String,
        /// Source map as a JSON string. Absent when source maps are disabled.
        pub map: Option<String>,
        pub diagnostics: Vec<Diagnostic>,
    }

    #[bridge(object, object_from_js = false, object_to_js = false)]
    pub struct DiagnosticLabel {
        pub message: String,
        /// UTF-16 offset in the original source.
        pub start: u32,
        /// UTF-16 offset in the original source.
        pub end: u32,
    }

    #[bridge(object, object_from_js = false, object_to_js = false)]
    pub struct Diagnostic {
        #[cfg_attr(
            not(target_family = "wasm"),
            napi(ts_type = "\"error\" | \"warning\" | \"hint\" | \"info\"")
        )]
        pub severity: String,
        pub message: String,
        /// UTF-16 offset in the original source.
        pub start: u32,
        /// UTF-16 offset in the original source.
        pub end: u32,
        pub help: Option<String>,
        pub labels: Vec<DiagnosticLabel>,
    }

    use crate::js::JsTransformApiOptions as TransformApiOptions;

    #[bridge]
    impl BridgeDecodeOptional for TransformApiOptions {
        type Wire = Vec<u8>;

        fn decode_optional(value: Self::Wire) -> Option<Option<Self>> {
            decode_options(&value)
        }
    }

    /// Decode the packed transform configuration without copying the input buffer.
    pub fn decode_options(bytes: &[u8]) -> Option<Option<JsTransformApiOptions>> {
        if bytes.is_empty() {
            return Some(None);
        }
        let header = bytes.get(..48)?;
        let word =
            |index: usize| u32::from_le_bytes(header[index * 4..index * 4 + 4].try_into().unwrap());
        if word(0) != 1
            || word(3) & !0xf != 0
            || word(4) & !0x1f != 0
            || word(5) & !0x7 != 0
            || word(6) & !0xf != 0
            || word(8) & !0xfff != 0
            || word(9) & !0x7e != 0
            || word(10) & !0x3fff != 0
            || word(11) & !0x6 != 0
        {
            return None;
        }
        let parser = <Option<JsParserConfig> as BridgeDecode>::decode(word(1))?;
        let generator = <Option<JsGeneratorConfig> as BridgeDecode>::decode(word(2))?;
        let mut cursor = 48;
        let mut string = || -> Option<Option<String>> {
            if bytes.len() == 48 {
                return Some(None);
            }
            let length = u32::from_le_bytes(bytes.get(cursor..cursor + 4)?.try_into().ok()?);
            cursor += 4;
            if length == u32::MAX {
                return Some(None);
            }
            let end = cursor.checked_add(length as usize)?;
            let value = std::str::from_utf8(bytes.get(cursor..end)?)
                .ok()?
                .to_owned();
            cursor = end;
            Some(Some(value))
        };
        let filename = string()?;
        let transform_filename = string()?;
        let jsx_pragma = string()?;
        let jsx_pragma_frag = string()?;
        let import_source = string()?;
        let pragma = string()?;
        let pragma_frag = string()?;
        let refresh_reg = string()?;
        let refresh_sig = string()?;
        if cursor != bytes.len() {
            return None;
        }
        let source_map = boolean(word(3), 0)?;
        let comments = match (word(3) >> 2) & 3 {
            0 => None,
            1 => Some(CommentsOption::All),
            2 => Some(CommentsOption::None),
            _ => return None,
        };
        if word(4) & 1 == 0 {
            if (4..12).any(|index| word(index) != 0)
                || transform_filename.is_some()
                || jsx_pragma.is_some()
                || jsx_pragma_frag.is_some()
                || import_source.is_some()
                || pragma.is_some()
                || pragma_frag.is_some()
                || refresh_reg.is_some()
                || refresh_sig.is_some()
            {
                return None;
            }
            return Some(Some(JsTransformApiOptions {
                filename,
                source_map,
                comments,
                parser,
                generator,
                transformer: None,
            }));
        }
        let typescript = stage(word(4), 1, || -> Option<JsTypeScriptConfig> {
            Some(JsTypeScriptConfig {
                verbatim_module_syntax: boolean(word(9), 1)?,
                preserve_const_enums: boolean(word(9), 3)?,
                jsx_runtime: runtime(word(9), 5)?,
                jsx_pragma,
                jsx_pragma_frag,
            })
        })?;
        let jsx = stage(word(4), 3, || -> Option<JsxConfig> {
            Some(JsxConfig {
                runtime: runtime(word(10), 0)?,
                development: boolean(word(10), 2)?,
                source_self: boolean(word(10), 4)?,
                throw_if_namespace: boolean(word(10), 6)?,
                pure: boolean(word(10), 8)?,
                display_name: boolean(word(10), 10)?,
                import_source,
                pragma,
                pragma_frag,
                refresh: stage(word(10), 12, || -> Option<JsReactRefreshConfig> {
                    Some(JsReactRefreshConfig {
                        refresh_reg,
                        refresh_sig,
                        emit_full_signatures: boolean(word(11), 1)?,
                    })
                })?,
            })
        })?;
        let mut features = std::collections::BTreeMap::new();
        if let Some(value) = boolean(word(6), 0)? {
            features.insert(JsFeature::ClassStaticBlock, value);
        }
        if let Some(value) = boolean(word(6), 2)? {
            features.insert(JsFeature::OptionalCatchBinding, value);
        }
        let assumptions = JsAssumptions {
            array_like_is_iterable: boolean(word(7), 0)?,
            constant_reexports: boolean(word(7), 2)?,
            constant_super: boolean(word(7), 4)?,
            enumerable_module_meta: boolean(word(7), 6)?,
            ignore_function_length: boolean(word(7), 8)?,
            ignore_function_name: boolean(word(7), 10)?,
            ignore_to_primitive_hint: boolean(word(7), 12)?,
            iterable_is_array: boolean(word(7), 14)?,
            mutable_template_object: boolean(word(7), 16)?,
            no_class_calls: boolean(word(7), 18)?,
            no_document_all: boolean(word(7), 20)?,
            no_incomplete_ns_import_detection: boolean(word(7), 22)?,
            no_new_arrows: boolean(word(7), 24)?,
            object_rest_no_symbols: boolean(word(7), 26)?,
            private_fields_as_properties: boolean(word(7), 28)?,
            pure_getters: boolean(word(7), 30)?,
            set_class_methods: boolean(word(8), 0)?,
            set_computed_properties: boolean(word(8), 2)?,
            set_public_class_fields: boolean(word(8), 4)?,
            set_spread_properties: boolean(word(8), 6)?,
            skip_for_of_iterator_closing: boolean(word(8), 8)?,
            super_is_callable_constructor: boolean(word(8), 10)?,
        };
        let proposal = JsProposalConfig {
            prevent_extensions: boolean(word(5), 1)?,
        };
        Some(Some(JsTransformApiOptions {
            filename,
            source_map,
            comments,
            parser,
            generator,
            transformer: (word(4) & 1 != 0).then_some(JsTransformerConfig {
                filename: transform_filename,
                typescript,
                jsx,
                features: (!features.is_empty()).then_some(features),
                assumptions: (word(7) != 0 || word(8) != 0).then_some(assumptions),
                proposal: (word(5) & 1 != 0).then_some(proposal),
            }),
        }))
    }

    fn boolean(value: u32, shift: u32) -> Option<Option<bool>> {
        match (value >> shift) & 3 {
            0 => Some(None),
            1 => Some(Some(false)),
            2 => Some(Some(true)),
            _ => None,
        }
    }

    fn runtime(value: u32, shift: u32) -> Option<Option<JsxRuntime>> {
        match (value >> shift) & 3 {
            0 => Some(None),
            1 => Some(Some(JsxRuntime::Classic)),
            2 => Some(Some(JsxRuntime::Automatic)),
            _ => None,
        }
    }

    fn stage<T>(
        value: u32,
        shift: u32,
        options: impl FnOnce() -> Option<T>,
    ) -> Option<Option<BoolOr<T>>> {
        match (value >> shift) & 3 {
            0 => Some(None),
            1 => Some(Some(BoolOr::Bool(false))),
            2 => Some(Some(BoolOr::Bool(true))),
            3 => Some(Some(BoolOr::Options(options()?))),
            _ => unreachable!(),
        }
    }

    /// Transform and serialize into a buffer allocated after its exact size is known.
    ///
    /// # Safety
    /// `source` must be valid UTF-8 and `source_utf16_len` must match its UTF-16 length.
    fn output_to_buffer<T: AsMut<[u8]>, E>(
        source: &str,
        source_utf16_len: u32,
        output: JsOutput,
        allocate: impl FnOnce(usize) -> Result<T, E>,
    ) -> Result<T, E> {
        let map = output.map.as_deref().unwrap_or("");
        let size =
            12 + output.code.len() + map.len() + diagnostics_buffer_size(&output.diagnostics);
        let mut buffer = allocate(size)?;
        let bytes = buffer.as_mut();
        let header = [
            output.code.len() as u32,
            output.map.as_ref().map_or(u32::MAX, |map| map.len() as u32),
            output.diagnostics.len() as u32,
        ];
        for (word, bytes) in header.into_iter().zip(bytes[..12].as_chunks_mut::<4>().0) {
            bytes.copy_from_slice(&word.to_le_bytes());
        }
        let code_end = 12 + output.code.len();
        bytes[12..code_end].copy_from_slice(output.code.as_bytes());
        let map_end = code_end + map.len();
        bytes[code_end..map_end].copy_from_slice(map.as_bytes());
        serialize_diagnostics(
            source,
            &output.diagnostics,
            &mut bytes[map_end..],
            source_utf16_len as usize,
        );
        Ok(buffer)
    }

    /// # Safety
    /// `source` must be valid UTF-8 and `source_utf16_len` must match its UTF-16 length.
    pub unsafe fn transform_to_buffer(
        source: &[u8],
        source_utf16_len: u32,
        options: Option<JsTransformApiOptions>,
    ) -> Vec<u8> {
        let source = unsafe { std::str::from_utf8_unchecked(source) };
        let output = transform_js(source, options.unwrap_or_default());
        let Ok(buffer) = {
            output_to_buffer(source, source_utf16_len, output, |size| {
                Ok::<_, std::convert::Infallible>(vec![0; size])
            })
        };
        buffer
    }

    /// Return code through the native string bridge when no metadata is needed.
    /// Otherwise serialize into a Node-owned buffer.
    ///
    /// # Safety
    /// `source` must be valid UTF-8 and `source_utf16_len` must match its UTF-16 length.
    #[cfg(not(target_family = "wasm"))]
    pub unsafe fn transform_to_node_output(
        env: &napi::Env,
        source: &[u8],
        source_utf16_len: u32,
        options: &[u8],
    ) -> napi::Result<
        napi::bindgen_prelude::Either<
            <String as swc_next_bridge::NativeAbi>::Abi,
            napi::bindgen_prelude::Buffer,
        >,
    > {
        let options = decode_options(options).ok_or_else(|| {
            napi::Error::new(
                napi::Status::InvalidArg,
                "invalid bridge value for `options`",
            )
        })?;
        let source = unsafe { std::str::from_utf8_unchecked(source) };
        let output = transform_js(source, options.unwrap_or_default());
        if output.map.is_none() && output.diagnostics.is_empty() {
            return Ok(napi::bindgen_prelude::Either::A(
                <String as swc_next_bridge::NativeAbi>::into_native(output.code),
            ));
        }
        output_to_buffer(source, source_utf16_len, output, |size| {
            swc_next_bridge::create_buffer(env, size)
        })
        .map(napi::bindgen_prelude::Either::B)
    }
}
