use std::{
    io::{self, ErrorKind},
    path::Path,
};

use serde::{Deserialize, de::DeserializeOwned};
use swc_experimental_ecma_transformer::{
    TransformerOptions, assumptions::Assumptions, es2015::ES2015Options,
};

mod plugins;
mod presets;

pub use plugins::BabelPlugins;
pub use presets::BabelPresets;

/// Babel options
///
/// <https://babel.dev/docs/options#plugin-and-preset-options>
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BabelOptions {
    /// Explicit plugin entries.
    #[serde(default)]
    pub plugins: BabelPlugins,

    /// Explicit preset entries.
    #[serde(default)]
    pub presets: BabelPresets,

    /// Parser source type mode (`"module"`, `"script"`, `"unambiguous"`, ...).
    pub source_type: Option<String>,

    /// Expected transform error.
    pub throws: Option<String>,

    /// Compiler assumptions passed through Babel configuration.
    #[serde(default)]
    pub assumptions: Assumptions,
}

impl BabelOptions {
    /// Read and merge the case, suite, and category `options.json` files.
    ///
    /// A suite-level plugin or preset list replaces the category-level list.
    pub fn from_test_path(path: &Path) -> io::Result<Self> {
        let mut babel_options: Option<Self> = None;
        let mut plugins_json = None;
        let mut presets_json = None;
        let mut suite_has_options = false;

        for (level, path) in path.ancestors().take(3).enumerate() {
            let file = path.join("options.json");
            if !file.is_file() {
                continue;
            }

            if level == 1 {
                suite_has_options = true;
            }

            let content = std::fs::read_to_string(&file)?;
            let mut value = serde_json::from_str::<serde_json::Value>(&content)
                .map_err(|error| invalid_options(&file, error))?;
            let Some(object) = value.as_object_mut() else {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("{}: Babel options must be an object", file.display()),
                ));
            };

            let new_plugins = object.remove("plugins");
            if plugins_json.is_none() && !(level == 2 && suite_has_options) {
                plugins_json = new_plugins;
            }

            let new_presets = object.remove("presets");
            if presets_json.is_none() && !(level == 2 && suite_has_options) {
                presets_json = new_presets;
            }

            let new_options = serde_json::from_value::<Self>(value)
                .map_err(|error| invalid_options(&file, error))?;
            if let Some(options) = babel_options.as_mut() {
                if options.source_type.is_none() {
                    options.source_type = new_options.source_type;
                }
                if options.throws.is_none() {
                    options.throws = new_options.throws;
                }
            } else {
                babel_options = Some(new_options);
            }
        }

        let mut options = babel_options.unwrap_or_default();
        if let Some(plugins) = plugins_json {
            options.plugins =
                serde_json::from_value(plugins).map_err(|error| invalid_options(path, error))?;
        }
        if let Some(presets) = presets_json {
            options.presets =
                serde_json::from_value(presets).map_err(|error| invalid_options(path, error))?;
        }
        Ok(options)
    }

    pub fn is_jsx(&self) -> bool {
        self.plugins.syntax_jsx
            || self.presets.jsx
            || self.plugins.react_jsx
            || self.plugins.react_jsx_dev
    }

    pub fn is_typescript(&self) -> bool {
        self.plugins.syntax_typescript.is_some()
    }

    pub fn is_typescript_definition(&self) -> bool {
        self.plugins
            .syntax_typescript
            .is_some_and(|options| options.dts)
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

fn invalid_options(path: &Path, error: serde_json::Error) -> io::Error {
    io::Error::new(
        ErrorKind::InvalidData,
        format!("{}: {error}", path.display()),
    )
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
            Self::String(name) | Self::Tuple(name, _) | Self::Triple(name, _, _) => name,
            Self::Vec1(name) => &name[0],
        }
    }

    fn normalized_name(&self) -> &str {
        self.name()
            .strip_prefix("@babel/plugin-")
            .or_else(|| self.name().strip_prefix("babel-plugin-"))
            .or_else(|| self.name().strip_prefix("@babel/preset-"))
            .or_else(|| self.name().strip_prefix("babel-preset-"))
            .unwrap_or_else(|| self.name())
    }

    fn value<T: DeserializeOwned + Default>(self) -> Result<T, String> {
        match self {
            Self::String(_) | Self::Vec1(_) => Ok(T::default()),
            Self::Tuple(name, value) | Self::Triple(name, value, _) => {
                serde_json::from_value::<T>(value).map_err(|error| format!("{name}: {error}"))
            }
        }
    }
}
