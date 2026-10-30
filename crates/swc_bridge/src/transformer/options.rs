use serde::{Deserialize, Deserializer, de};
use swc_next_ecma_ast::Lang;
use swc_next_ecma_transformer::{
    TransformerOptions, es2022::ES2022Options, proposal::ProposalOptions,
    typescript::TypeScriptOptions,
};

/// Rust transforms applied between JavaScript plugin stages.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct TransformPassOptions {
    /// Lower TypeScript using the Rust transform's options. Omission enables
    /// default lowering for TypeScript input; `false` retains TypeScript syntax.
    pub typescript: TypeScriptPassOptions,
    pub es2022: ES2022Options,
    pub proposal: ProposalOptions,
}

impl TransformPassOptions {
    pub fn from_json(value: &str) -> Result<Self, String> {
        serde_json::from_str(value).map_err(|error| format!("Invalid transform options: {error}"))
    }

    pub(super) fn normalize(self, lang: Lang) -> TransformerOptions {
        let typescript = match self.typescript {
            TypeScriptPassOptions::Auto => lang.is_typescript().then(TypeScriptOptions::default),
            TypeScriptPassOptions::Disabled => None,
            TypeScriptPassOptions::Enabled(options) => Some(options),
        };
        TransformerOptions {
            typescript,
            es2022: self.es2022,
            proposal: self.proposal,
            ..TransformerOptions::default()
        }
    }
}

/// Select TypeScript lowering without duplicating its Rust configuration.
#[derive(Debug, Default)]
pub enum TypeScriptPassOptions {
    #[default]
    Auto,
    Disabled,
    Enabled(TypeScriptOptions),
}

impl<'de> Deserialize<'de> for TypeScriptPassOptions {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TypeScriptVisitor;

        impl<'de> de::Visitor<'de> for TypeScriptVisitor {
            type Value = TypeScriptPassOptions;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("false or a TypeScript options object")
            }

            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
                if value {
                    Err(E::invalid_value(de::Unexpected::Bool(value), &self))
                } else {
                    Ok(TypeScriptPassOptions::Disabled)
                }
            }

            fn visit_map<M: de::MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
                TypeScriptOptions::deserialize(de::value::MapAccessDeserializer::new(map))
                    .map(TypeScriptPassOptions::Enabled)
            }
        }

        deserializer.deserialize_any(TypeScriptVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normalized(source: &str, lang: Lang) -> TransformerOptions {
        TransformPassOptions::from_json(source)
            .unwrap()
            .normalize(lang)
    }

    #[test]
    fn defaults() {
        let options = normalized("{}", Lang::Ts);
        let typescript = options.typescript.unwrap();
        assert!(!typescript.verbatim_module_syntax);
        assert!(!typescript.preserve_const_enums);
        assert!(!typescript.jsx.automatic);
        assert!(typescript.jsx.pragma.is_none());
        assert!(!options.es2022.static_blocks);
        assert!(!options.proposal.prevent_extensions);

        let options = normalized("{}", Lang::Js);
        assert!(options.typescript.is_none());

        let options = normalized(r#"{"typescript":false}"#, Lang::Ts);
        assert!(options.typescript.is_none());

        let options = normalized(r#"{"typescript":{}}"#, Lang::Js);
        assert!(options.typescript.is_some());
    }

    #[test]
    fn nested_options() {
        let options = normalized(
            r#"{"typescript":{"verbatimModuleSyntax":true,"preserveConstEnums":true,"jsx":{"automatic":true,"pragma":"h","pragmaFrag":"Fragment"}}}"#,
            Lang::Tsx,
        );
        let typescript = options.typescript.unwrap();
        assert!(typescript.verbatim_module_syntax);
        assert!(typescript.preserve_const_enums);
        assert!(typescript.jsx.automatic);
        assert_eq!(typescript.jsx.pragma.as_deref(), Some("h"));
        assert_eq!(typescript.jsx.pragma_frag.as_deref(), Some("Fragment"));

        let options = normalized(
            r#"{"es2022":{"staticBlocks":true},"proposal":{"preventExtensions":true}}"#,
            Lang::Js,
        );
        assert!(options.es2022.static_blocks);
        assert!(options.proposal.prevent_extensions);
    }

    #[test]
    fn invalid_options() {
        assert!(TransformPassOptions::from_json(r#"{"typescript":true}"#).is_err());

        assert!(TransformPassOptions::from_json(r#"{"typescript":null}"#).is_err());

        assert!(TransformPassOptions::from_json(r#"{"typescript":{"unknown":true}}"#).is_err());

        assert!(
            TransformPassOptions::from_json(r#"{"typescript":{"jsx":{"automatic":"true"}}}"#)
                .is_err()
        );

        assert!(TransformPassOptions::from_json(r#"{"es2022":{"unknown":true}}"#).is_err());

        assert!(TransformPassOptions::from_json(r#"{"proposal":{"unknown":true}}"#).is_err());
    }
}
