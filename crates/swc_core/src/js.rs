//! JavaScript and TypeScript APIs.
//!
//! The JavaScript facade resolves top-level settings and composes pipeline options.
//! Optional fields retain absence, and `Default` leaves them unset.

#[cfg(feature = "js_codegen")]
pub mod codegen;
#[cfg(feature = "js_transform")]
pub mod minify;
#[cfg(feature = "js_parser")]
pub mod parser;
#[cfg(feature = "js_transform")]
pub mod transform;

/// JavaScript and TypeScript stage settings.
///
/// Each stage field requires its corresponding `js_*` crate feature.
/// Minification settings are included with `js_transform`.
#[derive(Clone, Debug, Default)]
pub struct JsOptions {
    // Override the shared target for all JavaScript stages.
    // pub target: Option<Target>,
    // Override the shared comment policy.
    // pub comments: Option<CommentsOption>,
    #[cfg(feature = "js_parser")]
    pub parser: Option<parser::JsParserOptions>,
    #[cfg(feature = "js_transform")]
    pub transform: Option<transform::JsTransformOptions>,
    /// A boolean shorthand or explicit compression and mangling settings.
    #[cfg(feature = "js_transform")]
    pub minify: Option<crate::BoolOr<minify::JsMinifyOptions>>,
    #[cfg(feature = "js_codegen")]
    pub codegen: Option<codegen::JsCodegenOptions>,
}

// Draft-only shared types reachable from JsOptions. These declarations remain
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
//
// pub enum CommentsOption {
//     All,
//     Some,
//     None,
//     Options(CommentsOptions),
// }
//
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
