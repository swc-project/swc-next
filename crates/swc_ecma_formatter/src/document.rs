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

#[derive(Debug, Clone, Copy)]
enum FormatElement<'a> {
    Text(Text<'a>),
    Space,
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
