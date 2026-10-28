//! Language-independent layout documents backed by a formatting arena.
//!
//! [`Writer`] constructs balanced scopes and shared [`Fragment`]s. Finishing a
//! writer seals its document; the arena and borrowed text must remain alive
//! until the document has been printed.

#![deny(clippy::all)]
#![deny(unused)]

mod document;
mod options;
mod text_width;

pub use document::{
    ConditionTarget, Doc, FillEntry, FillSeparatorPolicy, FlatFragment, Fragment, GroupId,
    GroupRequest, NeutralFragment, RecordedGroup, Writer,
};
pub use options::{IndentStyle, IndentWidth, InvalidIndentWidth, LineEnding, PrintOptions};
pub use text_width::width as measure_text_width;
