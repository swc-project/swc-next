mod options;
mod options_wire;

pub use options::{CommentMode, Lang, ParserOptions, SourceType};

/// # Safety
///
/// The caller must ensure that `source` contains valid UTF-8.
/// `source_utf16_len` must match the UTF-16 code unit length of `source`.
pub unsafe fn parse_to_buffer(
    source: &[u8],
    source_utf16_len: u32,
    options: Option<ParserOptions>,
) -> Vec<u8> {
    unsafe {
        swc_next_core::js::parser::parse_to_buffer(
            source,
            source_utf16_len,
            options.map(Into::into),
        )
    }
}
