use serde::Deserialize;
use swc_next_ecma_transformer::{
    jsx::JsxOptions,
    typescript::{JsxImportOptions, TypeScriptOptions},
};

use super::PluginPresetEntries;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SyntaxTypeScriptOptions {
    pub dts: bool,
    pub disallow_ambiguous_jsx_like: bool,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct TransformTypeScriptOptions {
    #[serde(flatten)]
    syntax: SyntaxTypeScriptOptions,
    only_remove_type_imports: bool,
    optimize_const_enums: bool,
    jsx_pragma: Option<String>,
    jsx_pragma_frag: Option<String>,
}

impl From<TransformTypeScriptOptions> for TypeScriptOptions {
    fn from(options: TransformTypeScriptOptions) -> Self {
        Self {
            verbatim_module_syntax: options.only_remove_type_imports,
            preserve_const_enums: !options.optimize_const_enums,
            jsx: JsxImportOptions {
                pragma: options.jsx_pragma,
                pragma_frag: options.jsx_pragma_frag,
                ..JsxImportOptions::default()
            },
        }
    }
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
    pub typescript: Option<TypeScriptOptions>,
    pub react_jsx: Option<JsxOptions>,
    pub react_jsx_dev: Option<JsxOptions>,

    // ES2022
    pub static_blocks: bool,
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
                "transform-typescript" => match entry.value::<TransformTypeScriptOptions>() {
                    Ok(options) => {
                        plugins.syntax_typescript = Some(options.syntax);
                        plugins.typescript = Some(options.into());
                    }
                    Err(error) => plugins.errors.push(error),
                },
                "transform-react-jsx" => {
                    plugins.react_jsx = entry
                        .value::<JsxOptions>()
                        .map_err(|error| plugins.errors.push(error))
                        .ok();
                }
                // Babel 8 removed the standalone JSX self/source plugins.
                // Metadata stays opt-in through this plugin's `sourceSelf` option.
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
                "transform-class-static-block" => plugins.static_blocks = true,
                _ => plugins.unsupported.push(entry.name().to_owned()),
            }
        }
        Ok(plugins)
    }
}

#[cfg(test)]
mod tests {
    use super::BabelPlugins;

    fn parse(source: &str) -> BabelPlugins {
        serde_json::from_str(source).unwrap()
    }

    #[test]
    fn removed_react_options() {
        let plugins = parse(r#"[["transform-react-jsx", {"useSpread": false}]]"#);
        assert_eq!(plugins.errors.len(), 1);
        assert!(plugins.react_jsx.is_none());

        let plugins = parse(r#"[["transform-react-jsx", {"useSpread": true}]]"#);
        assert_eq!(plugins.errors.len(), 1);
        assert!(plugins.react_jsx.is_none());

        let plugins = parse(r#"[["transform-react-jsx-development", {"useBuiltIns": false}]]"#);
        assert_eq!(plugins.errors.len(), 1);
        assert!(plugins.react_jsx_dev.is_none());

        let plugins = parse(r#"[["transform-react-jsx-development", {"useBuiltIns": true}]]"#);
        assert_eq!(plugins.errors.len(), 1);
        assert!(plugins.react_jsx_dev.is_none());
    }

    #[test]
    fn removed_react_plugins() {
        let plugins = parse(r#"["transform-react-jsx-self"]"#);
        assert_eq!(plugins.unsupported, ["transform-react-jsx-self"]);

        let plugins = parse(r#"["transform-react-jsx-source"]"#);
        assert_eq!(plugins.unsupported, ["transform-react-jsx-source"]);
    }

    #[test]
    fn development_metadata() {
        let plugins = parse(r#"["transform-react-jsx-development"]"#);
        let options = plugins.react_jsx_dev.unwrap();
        assert!(options.development);
        assert!(!options.source_self);

        let plugins = parse(r#"[["transform-react-jsx-development", {"sourceSelf": true}]]"#);
        let options = plugins.react_jsx_dev.unwrap();
        assert!(options.development);
        assert!(options.source_self);
    }
}
