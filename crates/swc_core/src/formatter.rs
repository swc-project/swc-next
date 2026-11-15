//! Language-independent formatting document transport.
//!
//! Experimental and unstable. APIs, document formats, and formatting output may
//! change without notice.
//!
//! These internal APIs and binary formats do not follow semantic versioning.
//! Use matching SWC Next JavaScript wrappers and native/Wasm bindings.
//!
//! The `bridge` feature enables the internal binary transport helpers.

#[cfg(feature = "bridge")]
pub mod bridge {
    use swc_next_formatter_core::{IndentStyle, LineEnding, PrintOptions};
    use swc_next_formatter_host::FormatSession;

    /// Print a versioned document using language-independent layout options.
    pub fn print_doc(bytes: &[u8], options: &[u8]) -> Result<String, String> {
        let options = decode_print_options(options).ok_or("invalid print options")?;
        let session = FormatSession::new(options);
        let document = session.decode(bytes).map_err(|error| error.to_string())?;
        Ok(document.print(session.print_options()))
    }

    /// Print a versioned document and encode its emitted cursor markers.
    pub fn print_doc_with_cursor(bytes: &[u8], options: &[u8]) -> Result<Vec<u8>, String> {
        let options = decode_print_options(options).ok_or("invalid print options")?;
        let session = FormatSession::new(options);
        let document = session.decode(bytes).map_err(|error| error.to_string())?;
        let result = document
            .print_with_cursor(session.print_options())
            .map_err(|error| error.to_string())?;
        let mut output = Vec::with_capacity(result.text.len() + 9);
        if let Some(range) = result.cursor_range {
            let start = u32::try_from(range.start).map_err(|error| error.to_string())?;
            let end = u32::try_from(range.end).map_err(|error| error.to_string())?;
            output.push(1);
            output.extend(start.to_le_bytes());
            output.extend(end.to_le_bytes());
        } else {
            output.push(0);
        }
        output.extend(result.text.into_bytes());
        Ok(output)
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

    pub(crate) fn layout_options(
        width_low: u8,
        width_high: u8,
        indent_width: u8,
        use_tabs: u8,
        line_ending: u8,
    ) -> Option<PrintOptions> {
        Some(PrintOptions {
            line_width: u16::from_le_bytes([width_low, width_high]),
            indent_width: indent_width.try_into().ok()?,
            indent_style: match use_tabs {
                0 => IndentStyle::Space,
                1 => IndentStyle::Tab,
                _ => return None,
            },
            line_ending: match line_ending {
                0 => LineEnding::Lf,
                1 => LineEnding::CrLf,
                2 => LineEnding::Cr,
                _ => return None,
            },
        })
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
}
