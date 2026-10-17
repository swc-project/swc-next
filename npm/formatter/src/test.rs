use napi::{Error, Result};
use serde_json::{Map, Value};
use swc_next_bridge::napi_derive::napi;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_formatter::format;
use swc_next_ecma_parser::Options as ParserOptions;

#[path = "../../../crates/swc_ecma_formatter/tests/support/prettier_options.rs"]
mod prettier_options;

/// Call the native printer with fixture options, independently of the Doc bridge.
#[napi]
pub fn format_native_sync(
    source: String,
    filepath: String,
    options: Map<String, Value>,
) -> Result<String> {
    let options =
        prettier_options::format_options(&options, &source).map_err(Error::from_reason)?;
    let parser = ParserOptions {
        lang: Lang::from_path(&filepath),
        source_type: SourceType::from_path(&filepath),
        ..ParserOptions::default()
    };
    format(&source, parser, options)
        .map_err(|error| Error::from_reason(format!("{filepath}: {error}")))
}
