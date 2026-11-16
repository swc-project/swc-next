//! Public formatting options and validated option values.

use swc_next_formatter_core::PrintOptions;
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
        Self::from_parts(
            PrintOptions {
                line_width: 80,
                indent_style: IndentStyle::Space,
                indent_width: IndentWidth::DEFAULT,
                line_ending: LineEnding::Lf,
            },
            SyntaxOptions::prettier(),
        )
    }

    /// Combines shared host layout with JavaScript and TypeScript syntax options.
    pub const fn from_parts(layout: PrintOptions, syntax: SyntaxOptions) -> Self {
        Self {
            line_width: layout.line_width,
            indent_style: layout.indent_style,
            indent_width: layout.indent_width,
            line_ending: layout.line_ending,
            semicolons: syntax.semicolons,
            quote_style: syntax.quote_style,
            jsx_quote_style: syntax.jsx_quote_style,
            quote_properties: syntax.quote_properties,
            trailing_commas: syntax.trailing_commas,
            arrow_parentheses: syntax.arrow_parentheses,
            bracket_spacing: syntax.bracket_spacing,
            bracket_same_line: syntax.bracket_same_line,
            single_attribute_per_line: syntax.single_attribute_per_line,
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

/// JavaScript and TypeScript document construction options.
///
/// Layout comes from the host's [`PrintOptions`]. Defaults match Prettier's
/// syntax choices; [`FormatOptions`] remains the combined convenience API.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SyntaxOptions {
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

impl SyntaxOptions {
    /// Syntax options matching Prettier's defaults.
    pub const fn prettier() -> Self {
        Self {
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

impl Default for SyntaxOptions {
    fn default() -> Self {
        Self::prettier()
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

impl From<FormatOptions> for PrintOptions {
    fn from(options: FormatOptions) -> Self {
        Self {
            line_width: options.line_width,
            indent_style: options.indent_style,
            indent_width: options.indent_width,
            line_ending: options.line_ending,
        }
    }
}

impl From<FormatOptions> for SyntaxOptions {
    fn from(options: FormatOptions) -> Self {
        Self {
            semicolons: options.semicolons,
            quote_style: options.quote_style,
            jsx_quote_style: options.jsx_quote_style,
            quote_properties: options.quote_properties,
            trailing_commas: options.trailing_commas,
            arrow_parentheses: options.arrow_parentheses,
            bracket_spacing: options.bracket_spacing,
            bracket_same_line: options.bracket_same_line,
            single_attribute_per_line: options.single_attribute_per_line,
        }
    }
}
