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

/// JavaScript and TypeScript transform options.
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
    pub typescript: Option<BoolOr<JsTypeScriptConfig>>,
    /// JSX transform. `false` keeps JSX syntax in the output.
    ///
    /// @default enabled with the automatic runtime when `lang` is `"jsx"` or `"tsx"`
    pub jsx: Option<BoolOr<JsxConfig>>,
    /// Per-feature overrides. `true` enables a transform; `false` disables it.
    pub features: Option<BTreeMap<JsFeature, bool>>,
    /// Babel-compatible assumptions.
    pub assumptions: Option<JsAssumptions>,
    /// Proposal transform options.
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
///
/// Set `TRANSFORM` to `false` to skip native syntax transformations.
pub fn transform_js<const TRANSFORM: bool>(
    source: &str,
    options: JsTransformApiOptions,
) -> JsOutput {
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

    // ======================= Normalize =======================
    if TRANSFORM {
        match &transform.typescript {
            None | Some(BoolOr::Bool(false)) => Normalizer::normalize(&mut ast),
            _ => Normalizer::normalize_runtime(&mut ast),
        }
    }

    // ======================= Analyze and Transform =======================
    let semantic_errors = parser_options.semantic_errors.unwrap_or(false);
    if TRANSFORM || semantic_errors {
        let check_syntax = if semantic_errors {
            SyntaxChecks::all()
        } else {
            SyntaxChecks::empty()
        };
        let analyzed = analyze(&mut ast, AnalyzeOptions { check_syntax });
        output.diagnostics.extend(analyzed.diagnostics);
        if has_errors(&output.diagnostics) {
            return output;
        }
        if TRANSFORM {
            let transform = transform.to_transformer_options(options.filename.clone());
            let transformed = Transformer::transform(analyzed.semantic, transform);
            output.diagnostics.extend(transformed.diagnostics);
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

#[cfg(feature = "bridge")]
pub mod bridge {
    //! For internal SWC Next use only. These APIs and packed option/result
    //! formats do not follow semantic versioning and may change in any release.
    //! Use matching `@swc-next/encoder/transform` helpers and native/Wasm bindings.
    //!
    //! Binary source transformation bridge. Options use a 16-byte little-endian
    //! header, optionally followed by a length-prefixed UTF-8 filename. Empty options
    //! select defaults. Output starts with code length, optional map length
    //! (`u32::MAX` means absent), and diagnostic count, followed by UTF-8 code/map
    //! and the parser bridge's packed diagnostic records.

    use swc_next_bridge::{BridgeDecode, BridgeDecodeOptional, bridge};
    use swc_next_ecma_ast::ffi::ast::{diagnostics_buffer_size, serialize_diagnostics};

    use super::transform_js;
    use crate::js::{
        CommentsOption, JsOutput, JsTransformApiOptions, bridge::Diagnostic,
        generator::JsGeneratorConfig, parser::JsParserConfig,
    };

    #[cfg(not(target_family = "wasm"))]
    use swc_next_bridge::napi;

    #[bridge(
        object,
        js_name = "JsOutput",
        object_from_js = false,
        object_to_js = false
    )]
    pub struct TransformOutput {
        pub code: String,
        /// Source map as a JSON string. Absent when source maps are disabled.
        pub map: Option<String>,
        pub diagnostics: Vec<Diagnostic>,
    }

    /// Options for parsing and printing source.
    ///
    /// Built-in transformations, including TypeScript type erasure and JSX lowering,
    /// are not currently implemented by the JavaScript API.
    #[bridge(object, object_from_js = false, object_to_js = false)]
    #[derive(Default)]
    pub struct TransformApiOptions {
        /// Used to infer the language and source type, and for diagnostics and source maps.
        pub filename: Option<String>,
        /// Source map generation.
        ///
        /// @default false
        pub source_map: Option<bool>,
        /// Comment policy.
        pub comments: Option<CommentsOption>,
        /// Parser configuration.
        pub parser: Option<JsParserConfig>,
        /// Generator configuration.
        pub generator: Option<JsGeneratorConfig>,
    }

    impl From<TransformApiOptions> for JsTransformApiOptions {
        fn from(options: TransformApiOptions) -> Self {
            Self {
                filename: options.filename,
                source_map: options.source_map,
                comments: options.comments,
                parser: options.parser,
                generator: options.generator,
                transformer: None,
            }
        }
    }

    #[bridge]
    impl BridgeDecodeOptional for TransformApiOptions {
        type Wire = Vec<u8>;

        fn decode_optional(value: Self::Wire) -> Option<Option<Self>> {
            decode_options(&value)
        }
    }

    /// Decode the packed transform configuration without copying the input buffer.
    pub fn decode_options(bytes: &[u8]) -> Option<Option<TransformApiOptions>> {
        if bytes.is_empty() {
            return Some(None);
        }
        let header = bytes.get(..16)?;
        let word =
            |index: usize| u32::from_le_bytes(header[index * 4..index * 4 + 4].try_into().unwrap());
        if word(0) != 1 || word(3) & !0xf != 0 {
            return None;
        }
        let parser = <Option<JsParserConfig> as BridgeDecode>::decode(word(1))?;
        let generator = <Option<JsGeneratorConfig> as BridgeDecode>::decode(word(2))?;
        let filename = if bytes.len() == 16 {
            None
        } else {
            let length = u32::from_le_bytes(bytes.get(16..20)?.try_into().ok()?) as usize;
            let value = bytes.get(20..)?;
            if value.len() != length {
                return None;
            }
            Some(std::str::from_utf8(value).ok()?.to_owned())
        };
        let source_map = match word(3) & 3 {
            0 => None,
            1 => Some(false),
            2 => Some(true),
            _ => return None,
        };
        let comments = match (word(3) >> 2) & 3 {
            0 => None,
            1 => Some(CommentsOption::All),
            2 => Some(CommentsOption::None),
            _ => return None,
        };
        Some(Some(TransformApiOptions {
            filename,
            source_map,
            comments,
            parser,
            generator,
        }))
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
        options: Option<TransformApiOptions>,
    ) -> Vec<u8> {
        let source = unsafe { std::str::from_utf8_unchecked(source) };
        let output = transform_js::<false>(source, options.unwrap_or_default().into());
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
        let output = transform_js::<false>(source, options.unwrap_or_default().into());
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
