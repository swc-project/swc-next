//! Flat documents with arena-owned instructions and borrowed or generated text.
//!
//! Finishing seals an immutable slice; borrowed source retains its original owner.

mod printer;
#[cfg(test)]
mod tests;
mod writer;

pub(crate) use writer::Writer;

use crate::FormatOptions;

#[derive(Debug, Clone, Copy)]
enum FormatElement<'a> {
    Text(&'a str),
}

pub(crate) struct Doc<'a> {
    elements: &'a [FormatElement<'a>],
}

impl Doc<'_> {
    pub(crate) fn print(self, _options: FormatOptions) -> String {
        printer::print(self.elements)
    }
}
