//! Public formatting options and validated option values.

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
}

impl FormatOptions {
    /// Options matching Prettier's defaults.
    pub const fn prettier() -> Self {
        Self {
            line_width: 80,
            indent_style: IndentStyle::Space,
            indent_width: IndentWidth(2),
            line_ending: LineEnding::Lf,
            semicolons: Semicolons::Always,
            quote_style: QuoteStyle::Double,
            jsx_quote_style: QuoteStyle::Double,
            quote_properties: QuoteProperties::AsNeeded,
            trailing_commas: TrailingCommas::All,
            arrow_parentheses: ArrowParentheses::Always,
            bracket_spacing: true,
            bracket_same_line: false,
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

/// Indentation emitted at the start of a nested line.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum IndentStyle {
    Tab,
    Space,
}

/// A validated indentation width in the inclusive range `0..=64`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct IndentWidth(u8);

impl IndentWidth {
    pub const MAX: u8 = 64;

    pub const fn value(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for IndentWidth {
    type Error = InvalidIndentWidth;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(InvalidIndentWidth(value))
        }
    }
}

/// An indentation width outside the supported range.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct InvalidIndentWidth(pub u8);

impl std::fmt::Display for InvalidIndentWidth {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "indentation width {} exceeds {}",
            self.0,
            IndentWidth::MAX
        )
    }
}

impl std::error::Error for InvalidIndentWidth {}

/// Line ending emitted by the formatter.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LineEnding {
    Lf,
    CrLf,
    Cr,
}

#[cfg(test)]
impl LineEnding {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
            Self::Cr => "\r",
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
