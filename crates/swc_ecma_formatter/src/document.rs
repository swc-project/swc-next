//! Flat documents with arena-owned instructions and borrowed or generated text.
//!
//! Finishing seals an immutable slice; borrowed source retains its original owner.

mod printer;
#[cfg(test)]
mod tests;
mod text;
mod writer;

pub(crate) use writer::Writer;

use crate::FormatOptions;
use text::Text;

/// Explicit layout requests stay distinct from preserved source layout.
#[derive(Debug, Clone, Copy)]
pub(crate) enum GroupRequest {
    Auto,
    Expanded,
    PreserveSource,
}

impl From<bool> for GroupRequest {
    fn from(expand: bool) -> Self {
        if expand { Self::Expanded } else { Self::Auto }
    }
}

/// Effects join within each complete scope. Source preferences remain effective
/// when printing, while required expansion also constrains canonical measurement.
#[derive(Debug, Clone, Copy, Default)]
struct BreakEffect(u8);

impl BreakEffect {
    const SOURCE: Self = Self(1);
    const REQUIRED: Self = Self(2);

    fn from_request(request: GroupRequest) -> Self {
        match request {
            GroupRequest::Auto => Self::default(),
            GroupRequest::Expanded => Self::REQUIRED,
            GroupRequest::PreserveSource => Self::SOURCE,
        }
    }

    fn join(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    fn preferred_breaks(self) -> bool {
        self.0 != 0
    }
}

#[derive(Debug, Clone, Copy)]
enum FormatElement<'a> {
    Text(Text<'a>),
    Space,
    SoftLine,
    Line,
    BreakParent,
    StartGroup { effect: BreakEffect },
    EndGroup,
    HardLine,
    LiteralLine,
    StartIndent,
    StartAbsoluteIndent { width: usize },
    EndIndent,
    StartAlign { width: usize },
    EndAlign { width: usize },
}

pub(crate) struct Doc<'a> {
    elements: &'a [FormatElement<'a>],
}

impl Doc<'_> {
    pub(crate) fn print(self, options: FormatOptions) -> String {
        printer::print(self.elements, options)
    }
}
