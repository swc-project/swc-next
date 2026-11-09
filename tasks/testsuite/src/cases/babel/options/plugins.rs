use serde::Deserialize;
use swc_core::ecma::transforms::react::RefreshOptions;
use swc_next_core::{
    BoolOr,
    js::transform::{JsReactRefreshOptions, JsTypeScriptOptions, JsxOptions, JsxRuntime},
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

impl From<TransformTypeScriptOptions> for JsTypeScriptOptions {
    fn from(options: TransformTypeScriptOptions) -> Self {
        Self {
            verbatim_module_syntax: Some(options.only_remove_type_imports),
            preserve_const_enums: Some(!options.optimize_const_enums),
            jsx_pragma: options.jsx_pragma,
            jsx_pragma_frag: options.jsx_pragma_frag,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum BabelJsxRuntime {
    Classic,
    #[default]
    Automatic,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct BabelJsxOptions {
    runtime: BabelJsxRuntime,
    development: bool,
    source_self: bool,
    import_source: Option<String>,
    pragma: Option<String>,
    pragma_frag: Option<String>,
    throw_if_namespace: Option<bool>,
    pure: Option<bool>,
    display_name: bool,
    refresh: Option<RefreshOptions>,
}

impl From<BabelJsxOptions> for JsxOptions {
    fn from(options: BabelJsxOptions) -> Self {
        Self {
            runtime: Some(match options.runtime {
                BabelJsxRuntime::Classic => JsxRuntime::Classic,
                BabelJsxRuntime::Automatic => JsxRuntime::Automatic,
            }),
            development: Some(options.development),
            source_self: Some(options.source_self),
            import_source: options.import_source,
            pragma: options.pragma,
            pragma_frag: options.pragma_frag,
            throw_if_namespace: options.throw_if_namespace,
            pure: options.pure,
            display_name: Some(options.display_name),
            refresh: options.refresh.map(|options| {
                BoolOr::Options(JsReactRefreshOptions {
                    refresh_reg: Some(options.refresh_reg.to_string()),
                    refresh_sig: Some(options.refresh_sig.to_string()),
                    emit_full_signatures: Some(options.emit_full_signatures),
                })
            }),
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
    pub typescript: Option<JsTypeScriptOptions>,
    pub react_jsx: Option<JsxOptions>,
    pub react_jsx_dev: Option<JsxOptions>,
    pub react_display_name: bool,

    // ES2019
    pub optional_catch_binding: bool,

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
                        .value::<BabelJsxOptions>()
                        .map(Into::into)
                        .map_err(|error| plugins.errors.push(error))
                        .ok();
                }
                // Babel 8 removed the standalone JSX self/source plugins.
                // Metadata stays opt-in through this plugin's `sourceSelf` option.
                "transform-react-jsx-development" => {
                    plugins.react_jsx_dev = entry
                        .value::<BabelJsxOptions>()
                        .map(|mut options| {
                            options.development = true;
                            options.into()
                        })
                        .map_err(|error| plugins.errors.push(error))
                        .ok();
                }
                "transform-optional-catch-binding" => plugins.optional_catch_binding = true,
                "transform-class-static-block" => plugins.static_blocks = true,
                "transform-react-display-name" => plugins.react_display_name = true,
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
        assert_eq!(options.development, Some(true));
        assert_eq!(options.source_self, Some(false));

        let plugins = parse(r#"[["transform-react-jsx-development", {"sourceSelf": true}]]"#);
        let options = plugins.react_jsx_dev.unwrap();
        assert_eq!(options.development, Some(true));
        assert_eq!(options.source_self, Some(true));
    }
}
