use super::{CommentMode, Lang, SourceType};

/// Complete parser settings shared by standalone parsing and JavaScript pipelines.
///
/// The JavaScript facade selects the relevant settings for each pipeline.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsParserOptions {
    /// Determines how the source is parsed and evaluated.
    pub source_type: Option<SourceType>,
    /// Determines which syntax features are enabled.
    pub lang: Option<Lang>,
    /// Preserve explicit parenthesized AST nodes.
    pub preserve_parens: Option<bool>,
    /// Determines whether comments are collected.
    pub comments: Option<CommentMode>,
    /// Run semantic early-error checks after parsing, including after recovery.
    pub semantic_errors: Option<bool>,
}
