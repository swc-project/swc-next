use swc_next_ecma_ast::Severity;
use swc_next_ecma_formatter::{
    ArrowParentheses, DOCUMENT_FORMAT_VERSION, FormatError, FormatOptions, QuoteProperties,
    QuoteStyle, Semicolons, TrailingCommas,
};

use swc_next_allocator::Allocator;
use swc_next_formatter_core::{Doc, IndentStyle, LineEnding, PrintOptions};

use crate::ParserOptions;

/// Print a versioned document using language-independent layout options.
pub fn print_doc(bytes: &[u8], options: &[u8]) -> Result<String, String> {
    let options = decode_print_options(options).ok_or("invalid print options")?;
    let allocator = Allocator::new();
    let document = Doc::decode(&allocator, bytes).map_err(|error| error.to_string())?;
    Ok(document.print(options))
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

fn layout_options(
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

/// Format options use a small fixed record, independent of the AST option bits.
pub fn format_doc(
    source: String,
    parser: Option<ParserOptions>,
    options: &[u8],
) -> Result<Vec<u8>, String> {
    let options = decode_options(options).ok_or("invalid formatter options")?;
    let result = swc_next_ecma_formatter::format_to_doc_buffer(
        &source,
        parser.unwrap_or_default().into(),
        options,
    );
    match result {
        Ok(document) => Ok(document),
        Err(error) => {
            let (span, message) = match &error {
                FormatError::Parse(diagnostics) => {
                    let diagnostic = diagnostics
                        .iter()
                        .find(|d| d.severity == Severity::Error)
                        .ok_or("formatter returned parse failure without error diagnostic")?;
                    (diagnostic.span, diagnostic.message.to_string())
                }
                FormatError::Unsupported { span, .. } => (*span, error.to_string()),
            };
            let mut output = b"SWCE".to_vec();
            output.extend(DOCUMENT_FORMAT_VERSION.to_le_bytes());
            output.extend(span.start.to_le_bytes());
            output.extend(span.end.to_le_bytes());
            output.extend(message.as_bytes());
            Ok(output)
        }
    }
}

fn decode_options(bytes: &[u8]) -> Option<FormatOptions> {
    let &[
        version,
        width_low,
        width_high,
        indent_width,
        use_tabs,
        semi,
        single_quote,
        jsx_single_quote,
        quote_props,
        trailing_comma,
        arrow_parens,
        bracket_spacing,
        bracket_same_line,
        single_attribute_per_line,
        line_ending,
    ] = bytes
    else {
        return None;
    };
    if version != 0 {
        return None;
    }
    let boolean = |value| match value {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    };
    let quote = |value| {
        boolean(value).map(|single| {
            if single {
                QuoteStyle::Single
            } else {
                QuoteStyle::Double
            }
        })
    };
    let layout = layout_options(width_low, width_high, indent_width, use_tabs, line_ending)?;
    Some(FormatOptions {
        line_width: layout.line_width,
        indent_width: layout.indent_width,
        indent_style: layout.indent_style,
        semicolons: if boolean(semi)? {
            Semicolons::Always
        } else {
            Semicolons::AsNeeded
        },
        quote_style: quote(single_quote)?,
        jsx_quote_style: quote(jsx_single_quote)?,
        quote_properties: match quote_props {
            0 => QuoteProperties::AsNeeded,
            1 => QuoteProperties::Preserve,
            2 => QuoteProperties::Consistent,
            _ => return None,
        },
        trailing_commas: match trailing_comma {
            0 => TrailingCommas::All,
            1 => TrailingCommas::Es5,
            2 => TrailingCommas::None,
            _ => return None,
        },
        arrow_parentheses: if boolean(arrow_parens)? {
            ArrowParentheses::Always
        } else {
            ArrowParentheses::AsNeeded
        },
        bracket_spacing: boolean(bracket_spacing)?,
        bracket_same_line: boolean(bracket_same_line)?,
        single_attribute_per_line: boolean(single_attribute_per_line)?,
        line_ending: layout.line_ending,
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
