use swc_next_ecma_ast::Severity;
use swc_next_ecma_formatter::{
    ArrowParentheses, DOCUMENT_FORMAT_VERSION, FormatError, FormatOptions, QuoteProperties,
    QuoteStyle, Semicolons, TrailingCommas,
};
use swc_next_ecma_parser::Options as ParserOptions;

use crate::layout::layout_options;

/// Format options use a small fixed record, independent of the AST option bits.
pub fn format_doc(
    source: String,
    parser: Option<ParserOptions>,
    options: &[u8],
) -> Result<Vec<u8>, String> {
    let options = decode_options(options).ok_or("invalid formatter options")?;
    let result =
        swc_next_ecma_formatter::format_to_doc_buffer(&source, parser.unwrap_or_default(), options);
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
