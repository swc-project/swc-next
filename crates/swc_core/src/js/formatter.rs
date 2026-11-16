//! JavaScript and TypeScript formatting configuration and document transport.
//!
//! Experimental and unstable. APIs, document formats, and formatting output may
//! change without notice.
//!
//! Enable `serde` for camelCase configuration deserialization. Omitted fields
//! use the Rust formatter's defaults, including tabs and a 120-column line width.
//! The `bridge` feature enables the internal binary transport helpers.

use swc_next_formatter_js::{self as formatter, FormatOptions, InvalidIndentWidth};

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
    pub indent_style: Option<JsIndentStyle>,
    /// Indentation width in the inclusive range `0..=64`.
    ///
    /// @default 2
    pub indent_width: Option<u8>,
    /// Output line ending.
    ///
    /// @default "lf"
    pub line_ending: Option<JsLineEnding>,
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
        Ok(Self {
            line_width: options.line_width.unwrap_or(defaults.line_width),
            indent_style: options
                .indent_style
                .map(Into::into)
                .unwrap_or(defaults.indent_style),
            indent_width: options
                .indent_width
                .map(TryInto::try_into)
                .transpose()?
                .unwrap_or(defaults.indent_width),
            line_ending: options
                .line_ending
                .map(Into::into)
                .unwrap_or(defaults.line_ending),
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
        })
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsIndentStyle {
    Tab,
    Space,
}

impl From<JsIndentStyle> for swc_next_formatter_core::IndentStyle {
    fn from(value: JsIndentStyle) -> Self {
        match value {
            JsIndentStyle::Tab => Self::Tab,
            JsIndentStyle::Space => Self::Space,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsLineEnding {
    Lf,
    CrLf,
    Cr,
}

impl From<JsLineEnding> for swc_next_formatter_core::LineEnding {
    fn from(value: JsLineEnding) -> Self {
        match value {
            JsLineEnding::Lf => Self::Lf,
            JsLineEnding::CrLf => Self::CrLf,
            JsLineEnding::Cr => Self::Cr,
        }
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
    use swc_next_ecma_ast::Severity;
    use swc_next_ecma_parser::Options as ParserOptions;
    use swc_next_formatter_js::{
        ArrowParentheses, DOCUMENT_FORMAT_VERSION, FormatError, FormatOptions, QuoteProperties,
        QuoteStyle, Semicolons, TrailingCommas,
    };

    use crate::formatter::bridge::layout_options;

    /// Parse source and encode a document using version-matched formatter options.
    ///
    /// For internal SWC Next use only. This API and binary format do not follow
    /// semantic versioning and may change in any release.
    pub fn format_doc(
        source: String,
        parser: Option<ParserOptions>,
        options: &[u8],
    ) -> Result<Vec<u8>, String> {
        let options = decode_options(options).ok_or("invalid formatter options")?;
        let result = swc_next_formatter_js::format_to_doc_buffer(
            &source,
            parser.unwrap_or_default(),
            options,
        );
        match result {
            Ok(document) => Ok(document),
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
                let mut output = b"SWCE".to_vec();
                output.extend(DOCUMENT_FORMAT_VERSION.to_le_bytes());
                output.extend(span.start.to_le_bytes());
                output.extend(span.end.to_le_bytes());
                output.extend(message.as_bytes());
                Ok(output)
            }
        }
    }

    fn decode_options(bytes: &[u8]) -> Option<FormatOptions> {
        let &[
            version,
            width_low,
            width_high,
            indent_width,
            use_tabs,
            semi,
            single_quote,
            jsx_single_quote,
            quote_props,
            trailing_comma,
            arrow_parens,
            bracket_spacing,
            bracket_same_line,
            single_attribute_per_line,
            line_ending,
        ] = bytes
        else {
            return None;
        };
        if version != 0 {
            return None;
        }
        let boolean = |value| match value {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        };
        let quote = |value| {
            boolean(value).map(|single| {
                if single {
                    QuoteStyle::Single
                } else {
                    QuoteStyle::Double
                }
            })
        };
        let layout = layout_options(width_low, width_high, indent_width, use_tabs, line_ending)?;
        Some(FormatOptions {
            line_width: layout.line_width,
            indent_width: layout.indent_width,
            indent_style: layout.indent_style,
            semicolons: if boolean(semi)? {
                Semicolons::Always
            } else {
                Semicolons::AsNeeded
            },
            quote_style: quote(single_quote)?,
            jsx_quote_style: quote(jsx_single_quote)?,
            quote_properties: match quote_props {
                0 => QuoteProperties::AsNeeded,
                1 => QuoteProperties::Preserve,
                2 => QuoteProperties::Consistent,
                _ => return None,
            },
            trailing_commas: match trailing_comma {
                0 => TrailingCommas::All,
                1 => TrailingCommas::Es5,
                2 => TrailingCommas::None,
                _ => return None,
            },
            arrow_parentheses: if boolean(arrow_parens)? {
                ArrowParentheses::Always
            } else {
                ArrowParentheses::AsNeeded
            },
            bracket_spacing: boolean(bracket_spacing)?,
            bracket_same_line: boolean(bracket_same_line)?,
            single_attribute_per_line: boolean(single_attribute_per_line)?,
            line_ending: layout.line_ending,
        })
    }
}
