use serde::{Deserialize, de::DeserializeOwned};
use swc_experimental_ecma_transformer::{
    TransformerOptions,
    assumptions::Assumptions,
    es2015::{ES2015Options, arrow::ArrowFunctionsOptions},
};

/// Babel options
///
/// <https://babel.dev/docs/options#plugin-and-preset-options>
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BabelOptions {
    // Plugin and Preset options
    /// Explicit plugin entries.
    #[serde(default)]
    pub plugins: BabelPlugins,

    // Misc options
    /// Parser source type mode (`"module"`, `"script"`, `"unambiguous"`, ...).
    pub source_type: Option<String>,

    /// Expected transform error. These cases do not have a comparable output fixture.
    pub throws: Option<String>,

    /// Compiler assumptions passed through Babel configuration.
    #[serde(default)]
    pub assumptions: Assumptions,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(try_from = "PluginPresetEntries")]
pub struct BabelPlugins {
    pub errors: Vec<String>,
    pub unsupported: Vec<String>,
    // ES2015
    pub arrow_function: Option<ArrowFunctionsOptions>,
    pub shorthand_properties: bool,
    pub sticky_regex: bool,
}

impl TryFrom<PluginPresetEntries> for BabelPlugins {
    type Error = String;

    fn try_from(entries: PluginPresetEntries) -> Result<Self, Self::Error> {
        let mut p = Self::default();
        for entry in entries.0 {
            match entry.normalized_name() {
                "transform-arrow-functions" => {
                    p.arrow_function = entry
                        .value::<ArrowFunctionsOptions>()
                        .map_err(|err| p.errors.push(err))
                        .ok();
                }
                "transform-shorthand-properties" => p.shorthand_properties = true,
                "transform-sticky-regex" => p.sticky_regex = true,
                _ => p.unsupported.push(entry.name().to_owned()),
            }
        }
        Ok(p)
    }
}

impl From<BabelOptions> for TransformerOptions {
    fn from(options: BabelOptions) -> Self {
        let plugins = options.plugins;
        Self {
            assumptions: options.assumptions,
            es2015: ES2015Options {
                arrow_functions: plugins.arrow_function,
                shorthand_properties: plugins.shorthand_properties,
                sticky_regex: plugins.sticky_regex,
            },
        }
    }
}

/// <https://babeljs.io/docs/options#pluginpreset-entries>
#[derive(Debug, Deserialize)]
struct PluginPresetEntries(Vec<PluginPresetEntry>);

/// <https://babeljs.io/docs/options#pluginpreset-entries>
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum PluginPresetEntry {
    String(String),
    Vec1([String; 1]),
    Tuple(String, serde_json::Value),
    Triple(String, serde_json::Value, #[expect(unused)] String),
}

impl PluginPresetEntry {
    fn name(&self) -> &str {
        match self {
            Self::String(s) | Self::Tuple(s, _) | Self::Triple(s, _, _) => s,
            Self::Vec1(s) => &s[0],
        }
    }

    fn normalized_name(&self) -> &str {
        self.name()
            .strip_prefix("@babel/plugin-")
            .or_else(|| self.name().strip_prefix("babel-plugin-"))
            .unwrap_or_else(|| self.name())
    }

    fn value<T: DeserializeOwned + Default>(self) -> Result<T, String> {
        match self {
            Self::String(_) | Self::Vec1(_) => Ok(T::default()),
            Self::Tuple(name, v) | Self::Triple(name, v, _) => {
                serde_json::from_value::<T>(v).map_err(|err| format!("{name}: {err}"))
            }
        }
    }
}
