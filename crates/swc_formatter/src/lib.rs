//! Source formatting, configuration, and document transport for SWC Next.
//!
//! Experimental and unstable. APIs, document formats, and formatting output may
//! change without notice.
//!
//! These internal APIs and binary formats do not follow semantic versioning.
//! Use matching SWC Next JavaScript wrappers and native/Wasm bindings.
//!
//! Provides JavaScript, TypeScript and Markdown source formatting and configuration.
//! Enable `serde` for camelCase configuration deserialization.
//! The `bridge` feature enables the internal binary transport helpers.

use swc_next_formatter_core::{Doc, InvalidIndentWidth, PrintOptions};

mod builtins;
pub mod js;
pub mod markdown;

pub use builtins::BuiltinFormatters;

fn print_source_document(source: &str, document: Doc<'_>, options: PrintOptions) -> String {
    let mut output = document.print(options);
    if source.starts_with('\u{feff}') {
        output.insert(0, '\u{feff}');
    }
    output
}

/// Shared layout options. Omitted fields use the native document printer defaults.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PrintConfig {
    /// Maximum line width.
    ///
    /// @default 80
    pub line_width: Option<u16>,
    /// Indentation style.
    ///
    /// @default "space"
    pub indent_style: Option<IndentStyle>,
    /// Indentation width in the inclusive range `0..=64`.
    ///
    /// @default 2
    pub indent_width: Option<u8>,
    /// Output line ending.
    ///
    /// @default "lf"
    pub line_ending: Option<LineEnding>,
}

impl PrintConfig {
    fn resolve(self, defaults: PrintOptions) -> Result<PrintOptions, InvalidIndentWidth> {
        Ok(PrintOptions {
            line_width: self.line_width.unwrap_or(defaults.line_width),
            indent_style: self
                .indent_style
                .map(Into::into)
                .unwrap_or(defaults.indent_style),
            indent_width: self
                .indent_width
                .map(TryInto::try_into)
                .transpose()?
                .unwrap_or(defaults.indent_width),
            line_ending: self
                .line_ending
                .map(Into::into)
                .unwrap_or(defaults.line_ending),
        })
    }
}

impl TryFrom<PrintConfig> for PrintOptions {
    type Error = InvalidIndentWidth;

    fn try_from(options: PrintConfig) -> Result<Self, Self::Error> {
        options.resolve(Self::default())
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndentStyle {
    Tab,
    Space,
}

impl From<IndentStyle> for swc_next_formatter_core::IndentStyle {
    fn from(value: IndentStyle) -> Self {
        match value {
            IndentStyle::Tab => Self::Tab,
            IndentStyle::Space => Self::Space,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineEnding {
    Lf,
    CrLf,
    Cr,
}

impl From<LineEnding> for swc_next_formatter_core::LineEnding {
    fn from(value: LineEnding) -> Self {
        match value {
            LineEnding::Lf => Self::Lf,
            LineEnding::CrLf => Self::CrLf,
            LineEnding::Cr => Self::Cr,
        }
    }
}

#[cfg(feature = "bridge")]
pub mod bridge {
    use swc_next_bridge::{BridgeDecodeOptional, bridge};
    use swc_next_formatter_core::{Doc, PrintOptions};
    use swc_next_formatter_host::FormatSession;

    use super::{IndentStyle, LineEnding, PrintConfig};

    /// Print a versioned document using language-independent layout options.
    pub fn print_doc(bytes: &[u8], options: Option<PrintConfig>) -> Result<String, String> {
        with_decoded_doc(
            bytes,
            options,
            |document, layout| Ok(document.print(layout)),
        )
    }

    /// Print a versioned document and encode its emitted cursor markers.
    pub fn print_doc_with_cursor(
        bytes: &[u8],
        options: Option<PrintConfig>,
    ) -> Result<Vec<u8>, String> {
        let result = with_decoded_doc(bytes, options, |document, layout| {
            document
                .print_with_cursor(layout)
                .map_err(|error| error.to_string())
        })?;
        encode_print_result(result)
    }

    fn with_decoded_doc<T>(
        bytes: &[u8],
        options: Option<PrintConfig>,
        print: impl FnOnce(Doc<'_>, PrintOptions) -> Result<T, String>,
    ) -> Result<T, String> {
        let layout = PrintOptions::try_from(options.unwrap_or_default())
            .map_err(|error| error.to_string())?;
        let session = FormatSession::new(layout);
        let document = session.decode(bytes).map_err(|error| error.to_string())?;
        print(document, layout)
    }

    fn encode_print_result(
        result: swc_next_formatter_core::PrintResult,
    ) -> Result<Vec<u8>, String> {
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

    #[bridge]
    impl BridgeDecodeOptional for PrintConfig {
        type Wire = Vec<u8>;

        fn decode_optional(bytes: Self::Wire) -> Option<Option<Self>> {
            if bytes.is_empty() {
                return Some(None);
            }
            Some(Some(decode_layout(&bytes, 0xf)?.1))
        }
    }

    // Version u32, presence mask u32, line width u16, style, indent width, ending.
    pub(crate) fn decode_layout(bytes: &[u8], known_mask: u32) -> Option<(u32, PrintConfig)> {
        let &[1, 0, 0, 0, a, b, c, d, lo, hi, style, width, ending] = bytes else {
            return None;
        };
        let mask = u32::from_le_bytes([a, b, c, d]);
        if mask & !known_mask != 0 || width > 64 {
            return None;
        }
        let line_width = u16::from_le_bytes([lo, hi]);
        if mask & 1 == 0 && line_width != 0 {
            return None;
        }
        let indent_width = if mask & 4 != 0 {
            Some(width)
        } else {
            if width != 0 {
                return None;
            }
            None
        };
        Some((
            mask,
            PrintConfig {
                line_width: (mask & 1 != 0).then_some(line_width),
                indent_style: field(mask, 1, style, &[IndentStyle::Tab, IndentStyle::Space])?,
                indent_width,
                line_ending: field(
                    mask,
                    3,
                    ending,
                    &[LineEnding::Lf, LineEnding::CrLf, LineEnding::Cr],
                )?,
            },
        ))
    }

    pub(crate) fn field<T: Copy>(
        mask: u32,
        index: u32,
        value: u8,
        values: &[T],
    ) -> Option<Option<T>> {
        if mask & (1 << index) == 0 {
            return (value == 0).then_some(None);
        }
        Some(Some(*values.get(usize::from(value))?))
    }

    #[cfg(test)]
    mod tests {
        use super::{PrintConfig, print_doc};
        use swc_next_bridge::BridgeDecodeOptional;
        use swc_next_formatter_core::{IndentStyle, LineEnding, PrintOptions};

        fn options(bytes: &[u8]) -> Option<PrintOptions> {
            let config = PrintConfig::decode_optional(bytes.to_vec())?.unwrap_or_default();
            config.try_into().ok()
        }

        #[test]
        fn print_options() {
            let configured = options(&[1, 0, 0, 0, 15, 0, 0, 0, 100, 0, 0, 4, 1]).unwrap();
            assert_eq!(configured.line_width, 100);
            assert_eq!(configured.indent_style, IndentStyle::Tab);
            assert_eq!(configured.indent_width.value(), 4);
            assert_eq!(configured.line_ending, LineEnding::CrLf);

            let omitted = options(&[]).unwrap();
            assert_eq!(omitted, PrintOptions::default());

            let defaults = options(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
            assert_eq!(defaults, PrintOptions::default());

            assert!(options(&[1, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0]).is_none());

            assert!(options(&[1, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 65, 0]).is_none());

            assert!(options(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).is_none());

            assert!(options(&[1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 2, 0, 0]).is_none());

            assert!(options(&[1, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 3]).is_none());

            assert!(options(&[1]).is_none());
        }

        #[test]
        fn print_buffer() {
            let doc = b"SWCD\0\0\0\0\x01\0\0\0\x01\0\0\0\x09\0\0\0\0\0\0\0\0\x01\0\0\0x";
            assert_eq!(print_doc(doc, None).unwrap(), "x");
            assert!(print_doc(b"invalid", None).is_err());
        }
    }
}
