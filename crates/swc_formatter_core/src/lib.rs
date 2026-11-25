//! Language-independent layout documents backed by a formatting arena.
//!
//! Experimental and unstable. APIs, document formats, and formatting output may
//! change without notice.
//!
//! [`Writer`] constructs balanced scopes and shared [`Fragment`]s. Finishing a
//! writer seals its document; the arena and borrowed text must remain alive
//! until the document has been printed or encoded.

#![deny(clippy::all)]
#![deny(unused)]

mod document;
mod options;

use swc_next_simd::{cmp::SimdPartialOrd, u8x16};

pub use document::{
    ConditionTarget, CursorError, Doc, DocumentBufferError, FillEntry, FillSeparatorPolicy,
    FlatFragment, Fragment, GroupId, GroupRequest, GroupScope, IndentIfBreakOptions, IndentScope,
    LabelId, LineMode, LineOptions, NeutralFragment, PrintResult, RecordedGroup, Writer,
};
pub use options::{IndentStyle, IndentWidth, InvalidIndentWidth, LineEnding, PrintOptions};

/// Measures displayed columns using Unicode string width in a non-CJK context.
///
/// Text instructions and layout probes use the complete supplied string,
/// including control characters and Unicode sequences. Explicit document line
/// instructions determine layout breaks independently of text width.
pub fn measure_text_width(source: &str) -> usize {
    if single_width_ascii(source) {
        source.len()
    } else {
        unicode_width::UnicodeWidthStr::width(source)
    }
}

#[inline]
fn single_width_ascii(source: &str) -> bool {
    if source.len() < u8x16::LEN {
        source.bytes().all(|byte| (0x20..=0x7f).contains(&byte))
    } else {
        single_width_ascii_blocks(source)
    }
}

// Keep the vector loop out of callers that emit short punctuation and identifiers.
#[inline(never)]
fn single_width_ascii_blocks(source: &str) -> bool {
    let (blocks, tail) = source.as_bytes().as_chunks::<16>();
    blocks.iter().all(|block| {
        let bytes = u8x16::from_array(*block);
        !(bytes.simd_lt(u8x16::splat(0x20)) | bytes.simd_gt(u8x16::splat(0x7f))).any()
    }) && tail.iter().all(|byte| (0x20..=0x7f).contains(byte))
}

/// Version of the serialized document buffer.
pub const DOCUMENT_FORMAT_VERSION: u32 = 0;
