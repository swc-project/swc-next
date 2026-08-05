use napi_derive::napi;
use swc_next_ecma_ast::{Lang as AstLang, SourceType as AstSourceType};
use swc_next_ecma_parser::{CommentMode as AstCommentMode, Options as AstParserOptions};

#[napi(string_enum = "lowercase")]
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

#[napi(string_enum = "lowercase")]
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

#[napi(string_enum = "lowercase")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommentMode {
    None,
    Attached,
}

impl From<CommentMode> for AstCommentMode {
    fn from(value: CommentMode) -> Self {
        match value {
            CommentMode::None => Self::None,
            CommentMode::Attached => Self::Attached,
        }
    }
}

#[napi(object)]
#[derive(Default)]
pub struct ParserOptions {
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
    /// Determines whether comments are attached to AST nodes.
    ///
    /// @default "attached"
    pub comments: Option<CommentMode>,
    /// Allows `return` statements outside functions.
    ///
    /// @default false
    pub allow_return_outside_function: Option<bool>,
}

impl From<ParserOptions> for AstParserOptions {
    fn from(options: ParserOptions) -> Self {
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
            allow_return_outside_function: options
                .allow_return_outside_function
                .unwrap_or(defaults.allow_return_outside_function),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_allow_return_outside_function() {
        let defaults: AstParserOptions = ParserOptions::default().into();
        assert!(!defaults.allow_return_outside_function);

        let options: AstParserOptions = ParserOptions {
            allow_return_outside_function: Some(true),
            ..ParserOptions::default()
        }
        .into();
        assert!(options.allow_return_outside_function);
    }
}
