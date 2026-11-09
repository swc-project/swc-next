use swc_next_formatter_core::PrintOptions;
use swc_next_formatter_host::FormatSession;

use crate::layout::layout_options;

/// Print a versioned document using language-independent layout options.
pub fn print_doc(bytes: &[u8], options: &[u8]) -> Result<String, String> {
    let options = decode_print_options(options).ok_or("invalid print options")?;
    let session = FormatSession::new(options);
    let document = session.decode(bytes).map_err(|error| error.to_string())?;
    Ok(document.print(session.print_options()))
}

/// Print a versioned document and collect its emitted cursor markers.
pub fn print_doc_with_cursor(
    bytes: &[u8],
    options: &[u8],
) -> Result<swc_next_formatter_core::PrintResult, String> {
    let options = decode_print_options(options).ok_or("invalid print options")?;
    let session = FormatSession::new(options);
    let document = session.decode(bytes).map_err(|error| error.to_string())?;
    document
        .print_with_cursor(session.print_options())
        .map_err(|error| error.to_string())
}

/// Version 0: u32 version, u16 width, u8 indent style, width and line ending.
fn decode_print_options(bytes: &[u8]) -> Option<PrintOptions> {
    let &[
        0,
        0,
        0,
        0,
        width_low,
        width_high,
        use_tabs,
        indent_width,
        line_ending,
    ] = bytes
    else {
        return None;
    };
    layout_options(width_low, width_high, indent_width, use_tabs, line_ending)
}

#[cfg(test)]
mod tests {
    use super::{decode_print_options, print_doc};
    use swc_next_formatter_core::{IndentStyle, LineEnding};

    #[test]
    fn print_options() {
        let options = decode_print_options(&[0, 0, 0, 0, 100, 0, 1, 4, 1]).unwrap();
        assert_eq!(options.line_width, 100);
        assert_eq!(options.indent_style, IndentStyle::Tab);
        assert_eq!(options.indent_width.value(), 4);
        assert_eq!(options.line_ending, LineEnding::CrLf);

        assert!(decode_print_options(&[1, 0, 0, 0, 80, 0, 0, 2, 0]).is_none());
        assert!(decode_print_options(&[0, 0, 0, 0, 80, 0, 2, 2, 0]).is_none());
        assert!(decode_print_options(&[0, 0, 0, 0, 80, 0, 0, 65, 0]).is_none());
        assert!(decode_print_options(&[0, 0, 0, 0, 80, 0, 0, 2, 3]).is_none());
        assert!(decode_print_options(&[0]).is_none());
    }

    #[test]
    fn print_buffer() {
        let options = [0, 0, 0, 0, 80, 0, 0, 2, 0];
        let doc = b"SWCD\0\0\0\0\x01\0\0\0\x01\0\0\0\x09\0\0\0\0\0\0\0\0\x01\0\0\0x";
        assert_eq!(print_doc(doc, &options).unwrap(), "x");
        assert!(print_doc(b"invalid", &options).is_err());
        assert!(print_doc(doc, &[0]).is_err());
    }
}
