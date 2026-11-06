use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Lang as AstLang, SourceType as AstSourceType,
    ffi::ast::{buffer_size, serialize_into},
};
use swc_next_ecma_parser::{
    CommentMode as AstCommentMode, NoTokenParserConfig, Options as AstParserOptions, Parser,
};
use swc_next_ecma_semantic::{SyntaxChecks, check_syntax};

#[cfg(all(feature = "bridge", not(target_family = "wasm")))]
use swc_next_bridge::napi;

#[cfg_attr(feature = "bridge", swc_next_bridge::bridge(string_enum = "lowercase"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceType {
    Module,
    Script,
    CommonJs,
    Unambiguous,
}

impl From<SourceType> for AstSourceType {
    fn from(value: SourceType) -> Self {
        match value {
            SourceType::Module => Self::Module,
            SourceType::Script => Self::Script,
            SourceType::CommonJs => Self::CommonJs,
            SourceType::Unambiguous => Self::Unambiguous,
        }
    }
}

impl From<AstSourceType> for SourceType {
    fn from(value: AstSourceType) -> Self {
        match value {
            AstSourceType::Module => Self::Module,
            AstSourceType::Script => Self::Script,
            AstSourceType::CommonJs => Self::CommonJs,
            AstSourceType::Unambiguous => Self::Unambiguous,
        }
    }
}

#[cfg_attr(feature = "bridge", swc_next_bridge::bridge(string_enum = "lowercase"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lang {
    Js,
    Ts,
    Jsx,
    Tsx,
    Dts,
}

impl From<Lang> for AstLang {
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

impl From<AstLang> for Lang {
    fn from(value: AstLang) -> Self {
        match value {
            AstLang::Js => Self::Js,
            AstLang::Ts => Self::Ts,
            AstLang::Jsx => Self::Jsx,
            AstLang::Tsx => Self::Tsx,
            AstLang::Dts => Self::Dts,
        }
    }
}

#[cfg_attr(feature = "bridge", swc_next_bridge::bridge(string_enum = "lowercase"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommentMode {
    None,
    Flat,
}

impl From<CommentMode> for AstCommentMode {
    fn from(value: CommentMode) -> Self {
        match value {
            CommentMode::None => Self::None,
            CommentMode::Flat => Self::Flat,
        }
    }
}

/// Complete parser settings shared by standalone parsing and JavaScript pipelines.
///
/// The JavaScript facade selects the relevant settings for each pipeline.
#[cfg_attr(
    feature = "bridge",
    swc_next_bridge::bridge(object, js_name = "ParserOptions")
)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JsParserOptions {
    /// Determines how the source is parsed and evaluated.
    ///
    /// @default "module"
    pub source_type: Option<SourceType>,
    /// Determines which syntax features are enabled.
    ///
    /// @default "js"
    pub lang: Option<Lang>,
    /// Preserve parenthesized expressions and TypeScript types as explicit
    /// parenthesized nodes.
    ///
    /// @default true
    pub preserve_parens: Option<bool>,
    /// Determines how comments are exposed.
    ///
    /// `"none"` disables comment collection. `"flat"` exposes a
    /// source-ordered comment list.
    ///
    /// @default "flat"
    pub comments: Option<CommentMode>,
    /// Run semantic early-error checks on the parsed AST, including after syntax
    /// error recovery, and append their diagnostics. This performs an additional
    /// AST pass.
    ///
    /// @default false
    pub semantic_errors: Option<bool>,
}

impl From<JsParserOptions> for AstParserOptions {
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
    use swc_next_bridge::{BridgeDecodeOptional, bridge};

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
                1 => Some(SourceType::Module),
                2 => Some(SourceType::Script),
                3 => Some(SourceType::CommonJs),
                4 => Some(SourceType::Unambiguous),
                _ => return None,
            };
            let lang = match field(value, LANG_MASK, LANG_SHIFT) {
                0 => None,
                1 => Some(Lang::Js),
                2 => Some(Lang::Ts),
                3 => Some(Lang::Jsx),
                4 => Some(Lang::Tsx),
                5 => Some(Lang::Dts),
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
                1 => Some(CommentMode::None),
                2 => Some(CommentMode::Flat),
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
