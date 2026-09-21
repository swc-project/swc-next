use serde::Deserialize;
use swc_next_ecma_transformer::es2015::arrow::ArrowFunctionsOptions;
use swc_next_ecma_transformer::jsx::JsxOptions;

use super::PluginPresetEntries;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SyntaxTypeScriptOptions {
    pub dts: bool,
    pub disallow_ambiguous_jsx_like: bool,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(try_from = "PluginPresetEntries")]
pub struct BabelPlugins {
    pub errors: Vec<String>,
    pub unsupported: Vec<String>,

    // Syntax
    pub syntax_typescript: Option<SyntaxTypeScriptOptions>,
    pub syntax_jsx: bool,

    // TypeScript / JSX
    pub typescript: bool,
    pub react_jsx: Option<JsxOptions>,
    pub react_jsx_dev: Option<JsxOptions>,
    pub react_jsx_self: bool,
    pub react_jsx_source: bool,

    // ES2015
    pub arrow_function: Option<ArrowFunctionsOptions>,
    pub shorthand_properties: bool,
    pub sticky_regex: bool,
}

impl TryFrom<PluginPresetEntries> for BabelPlugins {
    type Error = String;

    fn try_from(entries: PluginPresetEntries) -> Result<Self, Self::Error> {
        let mut plugins = Self::default();
        for entry in entries.0 {
            match entry.name() {
                "typescript" | "syntax-typescript" => {
                    plugins.syntax_typescript = Some(entry.value::<SyntaxTypeScriptOptions>()?);
                }
                "jsx" | "syntax-jsx" => plugins.syntax_jsx = true,
                "transform-typescript" => {
                    plugins.typescript = true;
                    plugins.unsupported.push(entry.name().to_owned());
                }
                "transform-react-jsx" => {
                    plugins.react_jsx = entry
                        .value::<JsxOptions>()
                        .map_err(|error| plugins.errors.push(error))
                        .ok();
                }
                "transform-react-jsx-development" => {
                    plugins.react_jsx_dev = entry
                        .value::<JsxOptions>()
                        .map(|mut options| {
                            options.development = true;
                            options
                        })
                        .map_err(|error| plugins.errors.push(error))
                        .ok();
                }
                "transform-react-jsx-self" => plugins.react_jsx_self = true,
                "transform-react-jsx-source" => plugins.react_jsx_source = true,
                "transform-arrow-functions" => {
                    plugins.arrow_function = entry
                        .value::<ArrowFunctionsOptions>()
                        .map_err(|error| plugins.errors.push(error))
                        .ok();
                }
                "transform-shorthand-properties" => plugins.shorthand_properties = true,
                "transform-sticky-regex" => plugins.sticky_regex = true,
                _ => plugins.unsupported.push(entry.name().to_owned()),
            }
        }
        Ok(plugins)
    }
}
