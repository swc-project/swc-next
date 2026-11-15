//! JavaScript and TypeScript APIs.

use swc_next_ecma_ast::Diagnostic;

#[cfg(all(feature = "bridge", feature = "js_parser"))]
use swc_next_bridge::bridge;

#[cfg(all(feature = "bridge", feature = "js_parser", not(target_family = "wasm")))]
use swc_next_bridge::napi;

/// Comment policy.
#[cfg_attr(
    all(feature = "bridge", feature = "js_parser"),
    bridge(string_enum = "lowercase", js_name = "CommentMode")
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommentsOption {
    /// Keep all comments.
    All,
    // Some,
    /// Remove all comments.
    None,
    // Options(CommentsOptions),
}

#[cfg(feature = "js_generator")]
pub mod generator;
#[cfg(feature = "js_minifier")]
pub mod minifier;
#[cfg(feature = "js_parser")]
pub mod parser;
#[cfg(feature = "js_transformer")]
pub mod transformer;

/// Options for transforming JavaScript and TypeScript source.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Debug, Default)]
pub struct JsTransformApiOptions {
    /// Used to infer the language and source type, and for diagnostics and source maps.
    pub filename: Option<String>,
    /// Source map generation.
    ///
    /// @default false
    pub source_map: Option<bool>,
    // Override the shared target for all JavaScript stages.
    // pub target: Option<Target>,
    /// Comment policy.
    pub comments: Option<CommentsOption>,
    /// Parser configuration.
    #[cfg(feature = "js_parser")]
    pub parser: Option<parser::JsParserConfig>,
    /// Transformer configuration.
    #[cfg(feature = "js_transformer")]
    pub transformer: Option<transformer::JsTransformerConfig>,
    /// Generator configuration.
    #[cfg(feature = "js_generator")]
    pub generator: Option<generator::JsGeneratorConfig>,
}

/// Options for minifying JavaScript source.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Debug, Default)]
pub struct JsMinifyApiOptions {
    /// Source filename used for source maps.
    pub filename: Option<String>,
    /// Source map generation.
    ///
    /// @default false
    pub source_map: Option<bool>,
    // Override the shared target for all JavaScript stages.
    // pub target: Option<Target>,
    /// Comment policy.
    pub comments: Option<CommentsOption>,
    /// Minifier configuration.
    #[cfg(feature = "js_minifier")]
    pub minifier: Option<minifier::JsMinifierConfig>,
    /// Generator configuration.
    #[cfg(feature = "js_generator")]
    pub generator: Option<generator::JsGeneratorConfig>,
}

// Draft-only shared types reachable from JsTransformApiOptions. These declarations remain
// commented out until their policies are implemented.
// pub enum Target {
//     EsVersion(EsVersion),
//     Browserslist(Vec<String>),
//     Engines(BTreeMap<Engine, String>),
// }
//
// pub enum EsVersion {
//     Es5,
//     Es2015,
//     Es2016,
//     Es2017,
//     Es2018,
//     Es2019,
//     Es2020,
//     Es2021,
//     Es2022,
//     Es2023,
//     Es2024,
//     Es2025,
//     EsNext,
// }
//
// pub enum Engine {
//     Chrome,
//     Edge,
//     Firefox,
//     Safari,
//     Ios,
//     Opera,
//     Samsung,
//     Node,
//     Deno,
//     Bun,
// }

// pub struct CommentsOptions {
//     pub other: Option<OtherComments>,
//     pub legal: Option<LegalComments>,
//     pub annotations: Option<bool>, // JavaScript annotations; defaults to true.
//     pub keep: Option<Vec<String>>, // Regular-expression patterns.
// }
//
// pub enum OtherComments {
//     All,
//     None,
// }
//
// pub enum LegalComments {
//     Inline,
//     Eof,
//     External,
//     None,
// }

/// Source processing output.
#[derive(Debug, Default)]
pub struct JsOutput {
    pub code: String,
    /// Source map as a JSON string. `None` when source maps are disabled.
    pub map: Option<String>,
    /// Legal comments when `comments.legal` is `"external"`.
    pub legal_comments: Option<Vec<String>>,
    /// Diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

#[cfg(any(feature = "js_transformer", feature = "js_minifier"))]
fn serialize_source_map(map: &swc_next_ecma_codegen::SourceMap<'_>) -> String {
    // Source and filenames are UTF-8; generator records names from &str and emits
    // ASCII VLQ mappings.
    let text = |bytes| std::str::from_utf8(bytes).expect("source map strings must be valid UTF-8");
    let sources = map.sources().iter().map(|source| text(source));
    let names = map.names().iter().map(|name| text(name));
    let mut value = serde_json::json!({
        "version": map.version(),
        "sources": sources.collect::<Vec<_>>(),
        "names": names.collect::<Vec<_>>(),
        "mappings": text(map.mappings()),
    });
    if let Some(file) = map.file() {
        value["file"] = text(file).into();
    }
    if let Some(root) = map.source_root() {
        value["sourceRoot"] = text(root).into();
    }
    if let Some(sources) = map.sources_content() {
        let sources_content = sources.iter().map(|source| source.map(text));
        value["sourcesContent"] = serde_json::json!(sources_content.collect::<Vec<_>>());
    }
    value.to_string()
}

#[cfg(all(feature = "bridge", feature = "js_parser"))]
pub mod bridge {
    //! Shared JavaScript bridge types.

    use swc_next_bridge::bridge;
    #[cfg(not(target_family = "wasm"))]
    use swc_next_bridge::napi;

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
}
