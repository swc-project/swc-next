//! Language-independent layout documents backed by a formatting arena.
//!
//! [`Writer`] constructs balanced scopes and shared [`Fragment`]s. Finishing a
//! writer seals its document; the arena and borrowed text must remain alive
//! until the document has been printed or encoded.

#![deny(clippy::all)]
#![deny(unused)]

mod document;
mod options;

pub use document::{
    ConditionTarget, Doc, FillEntry, FillSeparatorPolicy, FlatFragment, Fragment, GroupId,
    GroupRequest, NeutralFragment, RecordedGroup, Writer,
};
pub use options::{IndentStyle, IndentWidth, InvalidIndentWidth, LineEnding, PrintOptions};

/// Measures displayed columns using Unicode string width in a non-CJK context.
///
/// Text instructions and layout probes use the complete supplied string,
/// including control characters and Unicode sequences. Explicit document line
/// instructions determine layout breaks independently of text width.
pub fn measure_text_width(source: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(source)
}

/// Version of the serialized document buffer.
pub const DOCUMENT_FORMAT_VERSION: u32 = 0;
