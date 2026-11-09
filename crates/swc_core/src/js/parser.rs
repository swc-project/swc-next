use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Lang, SourceType,
    ffi::ast::{buffer_size, serialize_into},
};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{SyntaxChecks, check_syntax};

use super::CommentsOption;

#[cfg(feature = "bridge")]
use swc_next_bridge::{BridgeDecodeOptional, bridge};

#[cfg(all(feature = "bridge", not(target_family = "wasm")))]
use swc_next_bridge::napi;

#[cfg_attr(
    feature = "bridge",
    bridge(string_enum = "lowercase", js_name = "SourceType")
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsSourceType {
    Module,
    Script,
    CommonJs,
    Unambiguous,
}

impl From<JsSourceType> for SourceType {
    fn from(value: JsSourceType) -> Self {
        match value {
            JsSourceType::Module => Self::Module,
            JsSourceType::Script => Self::Script,
            JsSourceType::CommonJs => Self::CommonJs,
            JsSourceType::Unambiguous => Self::Unambiguous,
        }
    }
}

impl From<SourceType> for JsSourceType {
    fn from(value: SourceType) -> Self {
        match value {
            SourceType::Module => Self::Module,
            SourceType::Script => Self::Script,
            SourceType::CommonJs => Self::CommonJs,
            SourceType::Unambiguous => Self::Unambiguous,
        }
    }
}

#[cfg_attr(
    feature = "bridge",
    bridge(string_enum = "lowercase", js_name = "Lang")
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsLang {
    Js,
    Ts,
    Jsx,
    Tsx,
    Dts,
}

impl From<JsLang> for Lang {
    fn from(value: JsLang) -> Self {
        match value {
            JsLang::Js => Self::Js,
            JsLang::Ts => Self::Ts,
            JsLang::Jsx => Self::Jsx,
            JsLang::Tsx => Self::Tsx,
            JsLang::Dts => Self::Dts,
        }
    }
}

impl From<Lang> for JsLang {
    fn from(value: Lang) -> Self {
        match value {
            Lang::Js => Self::Js,
            Lang::Ts => Self::Ts,
            Lang::Jsx => Self::Jsx,
            Lang::Tsx => Self::Tsx,
            Lang::Dts => Self::Dts,
        }
    }
}

impl From<CommentsOption> for CommentMode {
    fn from(value: CommentsOption) -> Self {
        match value {
            CommentsOption::All => Self::Flat,
            CommentsOption::None => Self::None,
        }
    }
}

/// JavaScript and TypeScript parser options.
#[cfg_attr(feature = "bridge", bridge(object, js_name = "ParserOptions"))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JsParserOptions {
    /// Determines how the source is parsed and evaluated.
    ///
    /// @default "module"
    pub source_type: Option<JsSourceType>,
    /// Determines which syntax features are enabled.
    ///
    /// @default "js"
    pub lang: Option<JsLang>,
    /// Preserve parenthesized expressions and TypeScript types as explicit
    /// parenthesized nodes.
    ///
    /// @default true
    pub preserve_parens: Option<bool>,
    /// Comment policy.
    ///
    /// @default "all"
    pub comments: Option<CommentsOption>,
    /// Run semantic early-error checks on the parsed AST, including after syntax
    /// error recovery, and append their diagnostics. This performs an additional
    /// AST pass.
    ///
    /// @default false
    pub semantic_errors: Option<bool>,
}

impl From<JsParserOptions> for Options {
    fn from(options: JsParserOptions) -> Self {
        let defaults = Self::default();
        Self {
            source_type: options
                .source_type
                .map(Into::into)
                .unwrap_or(defaults.source_type),
            lang: options.lang.map(Into::into).unwrap_or(defaults.lang),
            preserve_parens: options.preserve_parens.unwrap_or(defaults.preserve_parens),
            comments: options
                .comments
                .map(Into::into)
                .unwrap_or(defaults.comments),
        }
    }
}

/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
/// `source_utf16_len` must match the UTF-16 code unit length of `source`.
pub unsafe fn parse_to_buffer(
    source: &[u8],
    source_utf16_len: u32,
    options: Option<JsParserOptions>,
) -> Vec<u8> {
    let source = unsafe { std::str::from_utf8_unchecked(source) };
    let allocator = Allocator::default();
    let options = options.unwrap_or_default();
    let result = Parser::init(&allocator, source, options.into(), NoTokenParserConfig).parse();

    let mut diagnostics = result.diagnostics;
    if options.semantic_errors.unwrap_or(false) {
        diagnostics.extend(check_syntax(&result.ast, SyntaxChecks::all()));
    }

    let buffer_size = buffer_size(&result.ast, &diagnostics);
    let mut buffer = vec![0; buffer_size];
    serialize_into(
        &result.ast,
        &diagnostics,
        &mut buffer,
        source_utf16_len as usize,
    );
    buffer
}

#[cfg(feature = "bridge")]
const _: () = {
    use self::JsParserOptions as ParserOptions;

    const OPTIONS_PRESENT: u32 = 1 << 0;

    const SOURCE_TYPE_SHIFT: u32 = 1;
    const SOURCE_TYPE_MASK: u32 = 0b111 << SOURCE_TYPE_SHIFT;

    const LANG_SHIFT: u32 = 4;
    const LANG_MASK: u32 = 0b111 << LANG_SHIFT;

    const PRESERVE_PARENS_SHIFT: u32 = 7;
    const PRESERVE_PARENS_MASK: u32 = 0b11 << PRESERVE_PARENS_SHIFT;

    const COMMENTS_SHIFT: u32 = 9;
    const COMMENTS_MASK: u32 = 0b11 << COMMENTS_SHIFT;

    const SEMANTIC_ERRORS_SHIFT: u32 = 11;
    const SEMANTIC_ERRORS_MASK: u32 = 0b11 << SEMANTIC_ERRORS_SHIFT;

    const KNOWN_MASK: u32 = OPTIONS_PRESENT
        | SOURCE_TYPE_MASK
        | LANG_MASK
        | PRESERVE_PARENS_MASK
        | COMMENTS_MASK
        | SEMANTIC_ERRORS_MASK;

    #[bridge]
    impl BridgeDecodeOptional for ParserOptions {
        type Wire = u32;

        fn decode_optional(value: Self::Wire) -> Option<Option<Self>> {
            if value == 0 {
                return Some(None);
            }
            if value & OPTIONS_PRESENT == 0 || value & !KNOWN_MASK != 0 {
                return None;
            }

            let source_type = match field(value, SOURCE_TYPE_MASK, SOURCE_TYPE_SHIFT) {
                0 => None,
                1 => Some(JsSourceType::Module),
                2 => Some(JsSourceType::Script),
                3 => Some(JsSourceType::CommonJs),
                4 => Some(JsSourceType::Unambiguous),
                _ => return None,
            };
            let lang = match field(value, LANG_MASK, LANG_SHIFT) {
                0 => None,
                1 => Some(JsLang::Js),
                2 => Some(JsLang::Ts),
                3 => Some(JsLang::Jsx),
                4 => Some(JsLang::Tsx),
                5 => Some(JsLang::Dts),
                _ => return None,
            };
            let preserve_parens = match field(value, PRESERVE_PARENS_MASK, PRESERVE_PARENS_SHIFT) {
                0 => None,
                1 => Some(false),
                2 => Some(true),
                _ => return None,
            };
            let comments = match field(value, COMMENTS_MASK, COMMENTS_SHIFT) {
                0 => None,
                1 => Some(CommentsOption::All),
                2 => Some(CommentsOption::None),
                _ => return None,
            };

            let semantic_errors = match field(value, SEMANTIC_ERRORS_MASK, SEMANTIC_ERRORS_SHIFT) {
                0 => None,
                1 => Some(false),
                2 => Some(true),
                _ => return None,
            };

            Some(Some(JsParserOptions {
                source_type,
                lang,
                preserve_parens,
                comments,
                semantic_errors,
            }))
        }
    }

    fn field(value: u32, mask: u32, shift: u32) -> u32 {
        (value & mask) >> shift
    }
};
