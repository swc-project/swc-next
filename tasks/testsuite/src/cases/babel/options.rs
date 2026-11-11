// https://github.com/oxc-project/oxc/tree/main/crates/oxc_transformer/src/options/babel
use std::path::{Path, PathBuf};

use serde::{Deserialize, de::DeserializeOwned};
use swc_next_core::{
    BoolOr,
    js::{
        JsTransformApiOptions,
        generator::JsGeneratorConfig,
        transformer::{JsAssumptions, JsFeature, JsProposalConfig, JsTransformerConfig},
    },
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
    // Primary options
    /// Working directory used to resolve relative paths in configuration.
    pub cwd: Option<PathBuf>,

    // Plugin and Preset options
    /// Explicit plugin entries.
    #[serde(default)]
    pub plugins: BabelPlugins,

    /// Explicit preset entries.
    #[serde(default)]
    pub presets: BabelPresets,

    // Misc options
    /// Parser source type mode (`"module"`, `"script"`, `"unambiguous"`, ...).
    pub source_type: Option<String>,

    /// Emit compact output using Babel's minified generator mode.
    pub minified: Option<bool>,

    /// Literal escaping options.
    pub jsesc_option: Option<JsescOptions>,

    /// Compiler assumptions passed through Babel configuration.
    #[serde(default)]
    pub assumptions: BabelAssumptions,

    // Test options
    /// Expected error message used by Babel fixture tests.
    pub throws: Option<String>,

    /// Babel 8 compatibility toggle used by fixture tests.
    #[serde(rename = "BABEL_8_BREAKING")]
    pub babel_8_breaking: Option<bool>,

    // Parser options for babel-parser
    /// Allow `return` statements outside function bodies.
    #[serde(default)]
    pub allow_return_outside_function: bool,

    /// Allow `await` outside async functions.
    #[serde(default)]
    pub allow_await_outside_function: bool,

    /// Allow exported names without local declarations.
    #[serde(default)]
    pub allow_undeclared_exports: bool,

    /// Allow `new.target` outside function bodies.
    #[serde(default)]
    pub allow_new_target_outside_function: bool,

    /// Allow `super` outside class methods.
    #[serde(default)]
    pub allow_super_outside_method: bool,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsescOptions {
    pub numbers: Option<JsescNumberFormat>,
    #[serde(default)]
    pub lowercase_hex: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JsescNumberFormat {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct BabelAssumptions {
    #[serde(flatten)]
    pub options: JsAssumptions,
}

impl From<BabelOptions> for JsTransformApiOptions {
    fn from(options: BabelOptions) -> Self {
        let plugins = options.plugins;
        let mut jsx = plugins.react_jsx_dev.or(plugins.react_jsx);
        if plugins.react_display_name {
            jsx.get_or_insert_with(Default::default).display_name = Some(true);
        }
        Self {
            transformer: Some(JsTransformerConfig {
                assumptions: Some(options.assumptions.options),
                proposal: Some(JsProposalConfig {
                    prevent_extensions: Some(plugins.static_blocks),
                }),
                typescript: Some(match plugins.typescript {
                    Some(options) => BoolOr::Options(options),
                    None => BoolOr::Bool(false),
                }),
                jsx: Some(match jsx {
                    Some(options) => BoolOr::Options(options),
                    None => BoolOr::Bool(false),
                }),
                features: Some(
                    [
                        (
                            JsFeature::OptionalCatchBinding,
                            plugins.optional_catch_binding,
                        ),
                        (JsFeature::ClassStaticBlock, plugins.static_blocks),
                    ]
                    .into(),
                ),
            }),
            generator: Some(JsGeneratorConfig {
                minify: options.minified,
                ..Default::default()
            }),
            ..Default::default()
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

    fn value<T: DeserializeOwned + Default>(self) -> Result<T, String> {
        match self {
            Self::String(_) | Self::Vec1(_) => Ok(T::default()),
            Self::Tuple(name, v) | Self::Triple(name, v, _) => {
                serde_json::from_value::<T>(v).map_err(|err| format!("{name}: {err}"))
            }
        }
    }
}

impl BabelOptions {
    /// Read options.json and merge them with options.json from ancestors directories.
    ///
    /// Babel's fixture hierarchy: task → suite → category.
    /// Suite options.json REPLACES category plugins (not merge).
    pub fn from_test_path(path: &Path) -> Self {
        let mut babel_options: Option<Self> = None;
        let mut plugins_json = None;
        let mut presets_json = None;
        // Track if suite level (level 1) has options.json - if so, skip category plugins
        let mut suite_has_options = false;

        for (level, path) in path.ancestors().take(3).enumerate() {
            let file = path.join("options.json");
            if !file.exists() {
                continue;
            }

            if level == 1 {
                suite_has_options = true;
            }

            let content = std::fs::read_to_string(&file).unwrap();
            let mut new_value = serde_json::from_str::<serde_json::Value>(&content).unwrap();

            let new_plugins = new_value.as_object_mut().unwrap().remove("plugins");
            // Skip category (level 2) plugins if suite (level 1) has options.json
            if plugins_json.is_none() && !(level == 2 && suite_has_options) {
                plugins_json = new_plugins;
            }

            let new_presets = new_value.as_object_mut().unwrap().remove("presets");
            if presets_json.is_none() && !(level == 2 && suite_has_options) {
                presets_json = new_presets;
            }

            let new_options: Self = serde_json::from_value(new_value).unwrap();

            if let Some(existing_options) = babel_options.as_mut() {
                if existing_options.source_type.is_none() {
                    existing_options.source_type = new_options.source_type;
                }
                if existing_options.throws.is_none() {
                    existing_options.throws = new_options.throws;
                }
                if existing_options.minified.is_none() {
                    existing_options.minified = new_options.minified;
                }
                if existing_options.jsesc_option.is_none() {
                    existing_options.jsesc_option = new_options.jsesc_option;
                }
            } else {
                babel_options = Some(new_options);
            }
        }

        let mut options = babel_options.unwrap_or_default();
        if let Some(plugins_json) = plugins_json {
            options.plugins = serde_json::from_value(plugins_json).unwrap();
        }
        if let Some(presets_json) = presets_json {
            options.presets = serde_json::from_value(presets_json).unwrap();
        }
        options
    }

    /// Returns `true` when JSX parsing/transform should be enabled.
    pub fn is_jsx(&self) -> bool {
        self.plugins.syntax_jsx
            || self.presets.jsx
            || self.plugins.react_jsx.is_some()
            || self.plugins.react_jsx_dev.is_some()
    }

    /// Returns `true` when TypeScript parsing is enabled.
    pub fn is_typescript(&self) -> bool {
        self.plugins.syntax_typescript.is_some()
    }

    /// Returns `true` when parsing a TypeScript declaration file.
    pub fn is_typescript_definition(&self) -> bool {
        self.plugins.syntax_typescript.is_some_and(|o| o.dts)
    }
}
