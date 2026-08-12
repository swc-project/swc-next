use serde::Deserialize;
use swc_next_ecma_transformer::es2015::arrow::ArrowFunctionsOptions;

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
    pub react_jsx: bool,
    pub react_jsx_dev: bool,

    // ES2015
    pub arrow_function: Option<ArrowFunctionsOptions>,
    pub shorthand_properties: bool,
    pub sticky_regex: bool,

    // ES2019
    pub optional_catch_binding: bool,
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
                    plugins.react_jsx = true;
                    plugins.unsupported.push(entry.name().to_owned());
                }
                "transform-react-jsx-development" => {
                    plugins.react_jsx_dev = true;
                    plugins.unsupported.push(entry.name().to_owned());
                }
                "transform-arrow-functions" => {
                    plugins.arrow_function = entry
                        .value::<ArrowFunctionsOptions>()
                        .map_err(|error| plugins.errors.push(error))
                        .ok();
                }
                "transform-shorthand-properties" => plugins.shorthand_properties = true,
                "transform-sticky-regex" => plugins.sticky_regex = true,
                "transform-optional-catch-binding" => plugins.optional_catch_binding = true,
                _ => plugins.unsupported.push(entry.name().to_owned()),
            }
        }
        Ok(plugins)
    }
}
