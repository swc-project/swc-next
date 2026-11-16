use serde::Deserialize;
use swc_next_core::formatter::{IndentStyle, LineEnding};
use swc_next_core::js::formatter::{
    JsArrowParentheses, JsFormatterConfig, JsQuoteProperties, JsQuoteStyle, JsSemicolons,
    JsTrailingCommas,
};
use swc_next_formatter_js::FormatOptions;

use super::OptionSet;

/// Prettier fixture options supported by the SWC formatter.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct PrettierOptions {
    print_width: Option<u16>,
    tab_width: Option<u8>,
    use_tabs: Option<bool>,
    end_of_line: Option<EndOfLine>,
    semi: Option<bool>,
    single_quote: Option<bool>,
    jsx_single_quote: Option<bool>,
    quote_props: Option<JsQuoteProperties>,
    trailing_comma: Option<JsTrailingCommas>,
    arrow_parens: Option<ArrowParens>,
    bracket_spacing: Option<bool>,
    bracket_same_line: Option<bool>,
    jsx_bracket_same_line: Option<bool>,
    single_attribute_per_line: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum EndOfLine {
    Lf,
    CrLf,
    Cr,
    Auto,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ArrowParens {
    Always,
    Avoid,
}

impl PrettierOptions {
    fn into_swc(self, source: &str) -> JsFormatterConfig {
        JsFormatterConfig {
            line_width: Some(self.print_width.unwrap_or(80)),
            indent_style: Some(if self.use_tabs.unwrap_or(false) {
                IndentStyle::Tab
            } else {
                IndentStyle::Space
            }),
            indent_width: self.tab_width,
            line_ending: Some(match self.end_of_line.unwrap_or(EndOfLine::Lf) {
                EndOfLine::Lf => LineEnding::Lf,
                EndOfLine::CrLf => LineEnding::CrLf,
                EndOfLine::Cr => LineEnding::Cr,
                EndOfLine::Auto => detect_line_ending(source),
            }),
            semicolons: self.semi.map(|semi| {
                if semi {
                    JsSemicolons::Always
                } else {
                    JsSemicolons::AsNeeded
                }
            }),
            quote_style: self.single_quote.map(|single| {
                if single {
                    JsQuoteStyle::Single
                } else {
                    JsQuoteStyle::Double
                }
            }),
            jsx_quote_style: self.jsx_single_quote.map(|single| {
                if single {
                    JsQuoteStyle::Single
                } else {
                    JsQuoteStyle::Double
                }
            }),
            quote_properties: self.quote_props,
            trailing_commas: self.trailing_comma,
            arrow_parentheses: self.arrow_parens.map(|parens| match parens {
                ArrowParens::Always => JsArrowParentheses::Always,
                ArrowParens::Avoid => JsArrowParentheses::AsNeeded,
            }),
            bracket_spacing: self.bracket_spacing,
            bracket_same_line: Some(
                self.bracket_same_line.unwrap_or(false)
                    || self.jsx_bracket_same_line.unwrap_or(false),
            ),
            single_attribute_per_line: self.single_attribute_per_line,
        }
    }
}

pub(super) fn format_options(spec: &OptionSet, source: &str) -> Result<FormatOptions, String> {
    let value = serde_json::to_value(spec).map_err(|error| error.to_string())?;
    let options: PrettierOptions =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    FormatOptions::try_from(options.into_swc(source)).map_err(|error| error.to_string())
}

pub(super) fn unsupported_option_names(spec: &OptionSet) -> Vec<String> {
    spec.iter()
        .filter(|(name, value)| {
            let option = [((*name).clone(), (*value).clone())].into();
            value.is_null() || format_options(&option, "").is_err()
        })
        .map(|(name, value)| format!("unsupported Prettier option {name}={value}"))
        .collect()
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
