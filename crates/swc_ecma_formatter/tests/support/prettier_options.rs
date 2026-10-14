use serde_json::Value;
use swc_next_ecma_formatter::{
    ArrowParentheses, FormatOptions, IndentStyle, LineEnding, QuoteProperties, QuoteStyle,
    Semicolons, TrailingCommas,
};

pub fn format_options<'a>(
    spec: impl IntoIterator<Item = (&'a String, &'a Value)>,
    source: &str,
) -> Result<FormatOptions, String> {
    let mut options = FormatOptions::prettier();
    for (name, value) in spec {
        apply_prettier_option(&mut options, name, value, source)?;
    }
    Ok(options)
}

pub fn apply_prettier_option(
    options: &mut FormatOptions,
    name: &str,
    value: &Value,
    source: &str,
) -> Result<(), String> {
    let unsupported = || format!("unsupported Prettier option {name}={value}");
    match name {
        "printWidth" => {
            let width = value.as_u64().ok_or_else(unsupported)?;
            options.line_width = u16::try_from(width).map_err(|_| unsupported())?;
        }
        "tabWidth" => {
            let width = value.as_u64().ok_or_else(unsupported)?;
            let width = u8::try_from(width).map_err(|_| unsupported())?;
            options.indent_width = width.try_into().map_err(|_| unsupported())?;
        }
        "useTabs" => {
            let use_tabs = value.as_bool().ok_or_else(unsupported)?;
            options.indent_style = if use_tabs {
                IndentStyle::Tab
            } else {
                IndentStyle::Space
            };
        }
        "endOfLine" => {
            options.line_ending = match value.as_str() {
                Some("lf") => LineEnding::Lf,
                Some("crlf") => LineEnding::CrLf,
                Some("cr") => LineEnding::Cr,
                Some("auto") => detect_line_ending(source),
                _ => return Err(unsupported()),
            };
        }
        "semi" => {
            let semi = value.as_bool().ok_or_else(unsupported)?;
            options.semicolons = if semi {
                Semicolons::Always
            } else {
                Semicolons::AsNeeded
            };
        }
        "singleQuote" => {
            let single_quote = value.as_bool().ok_or_else(unsupported)?;
            options.quote_style = if single_quote {
                QuoteStyle::Single
            } else {
                QuoteStyle::Double
            };
        }
        "jsxSingleQuote" => {
            let single_quote = value.as_bool().ok_or_else(unsupported)?;
            options.jsx_quote_style = if single_quote {
                QuoteStyle::Single
            } else {
                QuoteStyle::Double
            };
        }
        "quoteProps" => {
            options.quote_properties = match value.as_str() {
                Some("as-needed") => QuoteProperties::AsNeeded,
                Some("preserve") => QuoteProperties::Preserve,
                _ => return Err(unsupported()),
            };
        }
        "trailingComma" => {
            options.trailing_commas = match value.as_str() {
                Some("all") => TrailingCommas::All,
                Some("es5") => TrailingCommas::Es5,
                Some("none") => TrailingCommas::None,
                _ => return Err(unsupported()),
            };
        }
        "arrowParens" => {
            options.arrow_parentheses = match value.as_str() {
                Some("always") => ArrowParentheses::Always,
                Some("avoid") => ArrowParentheses::AsNeeded,
                _ => return Err(unsupported()),
            };
        }
        "bracketSpacing" => {
            options.bracket_spacing = value.as_bool().ok_or_else(unsupported)?;
        }
        "bracketSameLine" | "jsxBracketSameLine" => {
            options.bracket_same_line |= value.as_bool().ok_or_else(unsupported)?;
        }
        "singleAttributePerLine" => {
            options.single_attribute_per_line = value.as_bool().ok_or_else(unsupported)?;
        }
        _ => return Err(unsupported()),
    }
    Ok(())
}

fn detect_line_ending(source: &str) -> LineEnding {
    let bytes = source.as_bytes();
    for index in 0..bytes.len() {
        match bytes[index] {
            b'\r' if bytes.get(index + 1) == Some(&b'\n') => return LineEnding::CrLf,
            b'\r' => return LineEnding::Cr,
            b'\n' => return LineEnding::Lf,
            _ => {}
        }
    }
    LineEnding::Lf
}
