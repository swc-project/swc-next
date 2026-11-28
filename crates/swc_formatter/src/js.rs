//! JavaScript and TypeScript source formatting, configuration, and document transport.
//!
//! Experimental and unstable. APIs, document formats, and formatting output may
//! change without notice.
//!
//! Enable `serde` for camelCase configuration deserialization. Omitted fields
//! use the Rust formatter's defaults, including tabs and a 120-column line width.
//! The `bridge` feature enables the internal binary transport helpers.

use swc_next_ecma_ast::AstCtx;
use swc_next_ecma_parser::Options as ParserOptions;
use swc_next_formatter_host::FormatSession;
use swc_next_formatter_js::{self as formatter, EcmaDocumentProducer, InvalidIndentWidth};

pub use swc_next_formatter_js::{FormatError, FormatOptions};

use crate::{IndentStyle, LineEnding, PrintConfig, print_source_document};

/// Parse and format source text.
///
/// Comment collection and parenthesis preservation are always enabled because
/// the formatter needs both to preserve annotation boundaries.
pub fn format(
    source: &str,
    parser_options: ParserOptions,
    options: FormatOptions,
) -> Result<String, FormatError> {
    let session = FormatSession::new(options.into());
    let producer = EcmaDocumentProducer::new(parser_options);
    let document = session.produce(&producer, source, &options.into())?;
    Ok(print_source_document(
        source,
        document,
        session.print_options(),
    ))
}

/// Format an already parsed program AST.
///
/// The AST must have a program root, as produced by
/// [`swc_next_ecma_parser::Parser::parse`]. Expression and statement fragment roots
/// are not supported.
///
/// The caller must provide an AST parsed with
/// [`swc_next_ecma_parser::CommentMode::Flat`] and `preserve_parens: true`,
/// retaining its original source and complete comment
/// list. These prerequisites are not checked; missing comments or parentheses
/// can lose annotations or change which expression they apply to.
/// Use [`format()`] to enable both parsing requirements automatically.
///
/// # Panics
///
/// Panics if the AST has no root or its root is not a program.
pub fn format_ast(ast: &AstCtx<'_>, options: FormatOptions) -> Result<String, FormatError> {
    let session = FormatSession::new(options.into());
    let document = formatter::ast_to_doc(ast, session.context(), options.into())?;
    Ok(print_source_document(
        ast.source(),
        document,
        session.print_options(),
    ))
}

/// JavaScript and TypeScript formatting options.
///
/// JSON uses SWC option names and rejects unknown fields. Conversion validates
/// indentation widths before constructing the formatter's native options.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JsFormatterConfig {
    /// Maximum line width.
    ///
    /// @default 120
    pub line_width: Option<u16>,
    /// Indentation style.
    ///
    /// @default "tab"
    pub indent_style: Option<IndentStyle>,
    /// Indentation width in the inclusive range `0..=64`.
    ///
    /// @default 2
    pub indent_width: Option<u8>,
    /// Output line ending.
    ///
    /// @default "lf"
    pub line_ending: Option<LineEnding>,
    /// Semicolon policy.
    ///
    /// @default "always"
    pub semicolons: Option<JsSemicolons>,
    /// JavaScript string quote style.
    ///
    /// @default "double"
    pub quote_style: Option<JsQuoteStyle>,
    /// JSX attribute quote style.
    ///
    /// @default "double"
    pub jsx_quote_style: Option<JsQuoteStyle>,
    /// Object property quoting policy.
    ///
    /// @default "as-needed"
    pub quote_properties: Option<JsQuoteProperties>,
    /// Trailing comma policy.
    ///
    /// @default "all"
    pub trailing_commas: Option<JsTrailingCommas>,
    /// Arrow function parameter parentheses.
    ///
    /// @default "always"
    pub arrow_parentheses: Option<JsArrowParentheses>,
    /// Spaces inside object literal braces.
    ///
    /// @default true
    pub bracket_spacing: Option<bool>,
    /// Place a multiline JSX opening tag's closing bracket on the last attribute line.
    ///
    /// @default false
    pub bracket_same_line: Option<bool>,
    /// Put each JSX attribute on its own line when an element has multiple attributes.
    ///
    /// @default false
    pub single_attribute_per_line: Option<bool>,
}

impl TryFrom<JsFormatterConfig> for FormatOptions {
    type Error = InvalidIndentWidth;

    fn try_from(options: JsFormatterConfig) -> Result<Self, Self::Error> {
        let defaults = Self::default();
        let layout = PrintConfig {
            line_width: options.line_width,
            indent_style: options.indent_style,
            indent_width: options.indent_width,
            line_ending: options.line_ending,
        }
        .resolve(defaults.into())?;
        let syntax = formatter::SyntaxOptions {
            semicolons: options
                .semicolons
                .map(Into::into)
                .unwrap_or(defaults.semicolons),
            quote_style: options
                .quote_style
                .map(Into::into)
                .unwrap_or(defaults.quote_style),
            jsx_quote_style: options
                .jsx_quote_style
                .map(Into::into)
                .unwrap_or(defaults.jsx_quote_style),
            quote_properties: options
                .quote_properties
                .map(Into::into)
                .unwrap_or(defaults.quote_properties),
            trailing_commas: options
                .trailing_commas
                .map(Into::into)
                .unwrap_or(defaults.trailing_commas),
            arrow_parentheses: options
                .arrow_parentheses
                .map(Into::into)
                .unwrap_or(defaults.arrow_parentheses),
            bracket_spacing: options.bracket_spacing.unwrap_or(defaults.bracket_spacing),
            bracket_same_line: options
                .bracket_same_line
                .unwrap_or(defaults.bracket_same_line),
            single_attribute_per_line: options
                .single_attribute_per_line
                .unwrap_or(defaults.single_attribute_per_line),
        };
        Ok(Self::from_parts(layout, syntax))
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsSemicolons {
    Always,
    AsNeeded,
}

impl From<JsSemicolons> for formatter::Semicolons {
    fn from(value: JsSemicolons) -> Self {
        match value {
            JsSemicolons::Always => Self::Always,
            JsSemicolons::AsNeeded => Self::AsNeeded,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsQuoteStyle {
    Double,
    Single,
}

impl From<JsQuoteStyle> for formatter::QuoteStyle {
    fn from(value: JsQuoteStyle) -> Self {
        match value {
            JsQuoteStyle::Double => Self::Double,
            JsQuoteStyle::Single => Self::Single,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsQuoteProperties {
    AsNeeded,
    Consistent,
    Preserve,
}

impl From<JsQuoteProperties> for formatter::QuoteProperties {
    fn from(value: JsQuoteProperties) -> Self {
        match value {
            JsQuoteProperties::AsNeeded => Self::AsNeeded,
            JsQuoteProperties::Consistent => Self::Consistent,
            JsQuoteProperties::Preserve => Self::Preserve,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsTrailingCommas {
    All,
    Es5,
    None,
}

impl From<JsTrailingCommas> for formatter::TrailingCommas {
    fn from(value: JsTrailingCommas) -> Self {
        match value {
            JsTrailingCommas::All => Self::All,
            JsTrailingCommas::Es5 => Self::Es5,
            JsTrailingCommas::None => Self::None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsArrowParentheses {
    Always,
    AsNeeded,
}

impl From<JsArrowParentheses> for formatter::ArrowParentheses {
    fn from(value: JsArrowParentheses) -> Self {
        match value {
            JsArrowParentheses::Always => Self::Always,
            JsArrowParentheses::AsNeeded => Self::AsNeeded,
        }
    }
}

#[cfg(feature = "bridge")]
pub mod bridge {
    use swc_next_bridge::{BridgeDecodeOptional, BridgeEncode, bridge};
    use swc_next_ecma_ast::Severity;
    use swc_next_ecma_parser::Options as ParserOptions;
    use swc_next_formatter_js::{DOCUMENT_FORMAT_VERSION, FormatError, FormatOptions};

    use super::{
        JsArrowParentheses, JsFormatterConfig, JsQuoteProperties, JsQuoteStyle, JsSemicolons,
        JsTrailingCommas,
    };
    use crate::bridge::{decode_layout, field};

    /// Internal document result. Syntax failures are transported separately from documents.
    pub enum FormatDocOutput {
        Document(Vec<u8>),
        Error {
            start: u32,
            end: u32,
            message: String,
        },
    }

    #[bridge]
    impl BridgeEncode for FormatDocOutput {
        type Wire = Vec<u8>;

        fn encode(self) -> Self::Wire {
            match self {
                Self::Document(document) => document,
                Self::Error {
                    start,
                    end,
                    message,
                } => {
                    let mut output = b"SWCE".to_vec();
                    output.extend(DOCUMENT_FORMAT_VERSION.to_le_bytes());
                    output.extend(start.to_le_bytes());
                    output.extend(end.to_le_bytes());
                    output.extend(message.as_bytes());
                    output
                }
            }
        }
    }

    /// Parse source and produce a document or a source diagnostic.
    pub fn format_doc(
        source: String,
        parser: Option<ParserOptions>,
        options: Option<JsFormatterConfig>,
    ) -> Result<FormatDocOutput, String> {
        let options = FormatOptions::try_from(options.unwrap_or_default())
            .map_err(|error| error.to_string())?;
        match swc_next_formatter_js::format_to_doc_buffer(
            &source,
            parser.unwrap_or_default(),
            options,
        ) {
            Ok(document) => Ok(FormatDocOutput::Document(document)),
            Err(error) => {
                let (span, message) = match &error {
                    FormatError::Parse(diagnostics) => {
                        let diagnostic = diagnostics
                            .iter()
                            .find(|d| d.severity == Severity::Error)
                            .ok_or("formatter returned parse failure without error diagnostic")?;
                        (diagnostic.span, diagnostic.message.to_string())
                    }
                    FormatError::Unsupported { span, .. } => (*span, error.to_string()),
                };
                Ok(FormatDocOutput::Error {
                    start: span.start,
                    end: span.end,
                    message,
                })
            }
        }
    }

    #[bridge]
    impl BridgeDecodeOptional for JsFormatterConfig {
        type Wire = Vec<u8>;

        fn decode_optional(bytes: Self::Wire) -> Option<Option<Self>> {
            if bytes.is_empty() {
                return Some(None);
            }
            if bytes.len() != 22 {
                return None;
            }
            let (mask, layout) = decode_layout(&bytes[..13], 0x1fff)?;
            Some(Some(Self {
                line_width: layout.line_width,
                indent_style: layout.indent_style,
                indent_width: layout.indent_width,
                line_ending: layout.line_ending,
                semicolons: field(
                    mask,
                    4,
                    bytes[13],
                    &[JsSemicolons::Always, JsSemicolons::AsNeeded],
                )?,
                quote_style: field(
                    mask,
                    5,
                    bytes[14],
                    &[JsQuoteStyle::Double, JsQuoteStyle::Single],
                )?,
                jsx_quote_style: field(
                    mask,
                    6,
                    bytes[15],
                    &[JsQuoteStyle::Double, JsQuoteStyle::Single],
                )?,
                quote_properties: field(
                    mask,
                    7,
                    bytes[16],
                    &[
                        JsQuoteProperties::AsNeeded,
                        JsQuoteProperties::Consistent,
                        JsQuoteProperties::Preserve,
                    ],
                )?,
                trailing_commas: field(
                    mask,
                    8,
                    bytes[17],
                    &[
                        JsTrailingCommas::All,
                        JsTrailingCommas::Es5,
                        JsTrailingCommas::None,
                    ],
                )?,
                arrow_parentheses: field(
                    mask,
                    9,
                    bytes[18],
                    &[JsArrowParentheses::Always, JsArrowParentheses::AsNeeded],
                )?,
                bracket_spacing: field(mask, 10, bytes[19], &[false, true])?,
                bracket_same_line: field(mask, 11, bytes[20], &[false, true])?,
                single_attribute_per_line: field(mask, 12, bytes[21], &[false, true])?,
            }))
        }
    }
}
