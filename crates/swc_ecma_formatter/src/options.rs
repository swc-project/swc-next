//! Public formatting options and validated option values.

pub use swc_next_formatter_core::{IndentStyle, IndentWidth, InvalidIndentWidth, LineEnding};

/// JavaScript and TypeScript formatting options.
///
/// Native defaults use tabs and a 120-column line width.
/// [`FormatOptions::prettier`] selects Prettier's defaults independently.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FormatOptions {
    pub line_width: u16,
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_ending: LineEnding,
    pub semicolons: Semicolons,
    pub quote_style: QuoteStyle,
    pub jsx_quote_style: QuoteStyle,
    pub quote_properties: QuoteProperties,
    pub trailing_commas: TrailingCommas,
    pub arrow_parentheses: ArrowParentheses,
    pub bracket_spacing: bool,
    pub bracket_same_line: bool,
    /// Put each JSX attribute on its own line when an element has multiple attributes.
    pub single_attribute_per_line: bool,
}

impl FormatOptions {
    /// Options matching Prettier's defaults.
    pub const fn prettier() -> Self {
        Self {
            line_width: 80,
            indent_style: IndentStyle::Space,
            indent_width: IndentWidth::DEFAULT,
            line_ending: LineEnding::Lf,
            semicolons: Semicolons::Always,
            quote_style: QuoteStyle::Double,
            jsx_quote_style: QuoteStyle::Double,
            quote_properties: QuoteProperties::AsNeeded,
            trailing_commas: TrailingCommas::All,
            arrow_parentheses: ArrowParentheses::Always,
            bracket_spacing: true,
            bracket_same_line: false,
            single_attribute_per_line: false,
        }
    }
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            line_width: 120,
            indent_style: IndentStyle::Tab,
            ..Self::prettier()
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Semicolons {
    Always,
    AsNeeded,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum QuoteStyle {
    Double,
    Single,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum QuoteProperties {
    AsNeeded,
    Consistent,
    Preserve,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TrailingCommas {
    All,
    Es5,
    None,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ArrowParentheses {
    Always,
    AsNeeded,
}

impl From<FormatOptions> for swc_next_formatter_core::PrintOptions {
    fn from(options: FormatOptions) -> Self {
        Self {
            line_width: options.line_width,
            indent_style: options.indent_style,
            indent_width: options.indent_width,
            line_ending: options.line_ending,
        }
    }
}
