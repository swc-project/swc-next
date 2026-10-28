//! Layout options shared by all language printers.

/// Options used only when laying out a completed document.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PrintOptions {
    pub line_width: u16,
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_ending: LineEnding,
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self {
            line_width: 80,
            indent_style: IndentStyle::Space,
            indent_width: IndentWidth::DEFAULT,
            line_ending: LineEnding::Lf,
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
    pub const DEFAULT: Self = Self(2);
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

/// Line ending emitted by the printer.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LineEnding {
    Lf,
    CrLf,
    Cr,
}

impl LineEnding {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
            Self::Cr => "\r",
        }
    }
}
