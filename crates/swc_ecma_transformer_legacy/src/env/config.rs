use crate::{
    BoolOr,
    es2017::AsyncToGeneratorOptions,
    es2018::ObjectRestSpreadOptions,
    es2020::{NullishCoalescingOptions, OptionalChainingOptions},
    es2022::{ClassFieldsUseSetOptions, ClassPropertiesOptions},
};

use super::{EnvOptions, Target, TargetError, compat::Feature};

/// Target selection and explicit SWC pass configuration.
///
/// Conversion starts with the target's required transforms. Unspecified settings
/// inherit that selection; explicit `false` disables a pass, `true` enables its
/// defaults, and an object enables it with the supplied SWC configuration.
/// Without a target, all passes start disabled. This API covers the selected
/// ES2016–ES2022/RegExp passes, not a complete compiler or polyfill preset.
///
/// ```
/// use swc_next_ecma_transformer_legacy::env::{EnvConfig, EsVersion, Target};
/// let config = EnvConfig {
///     target: Some(Target::EsVersion(EsVersion::Es2018)),
///     ..Default::default()
/// };
/// let options = config.to_env_options().unwrap();
/// assert!(options.es2019.optional_catch_binding);
/// assert!(!options.es2016.exponentiation);
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EnvConfig {
    pub target: Option<Target>,
    pub es2016: ES2016Config,
    pub es2017: ES2017Config,
    pub es2018: ES2018Config,
    pub es2019: ES2019Config,
    pub es2020: ES2020Config,
    pub es2021: ES2021Config,
    pub es2022: ES2022Config,
    pub regexp: RegExpConfig,
}

macro_rules! config {
    ($(#[$attribute:meta])* $name:ident { $($(#[$field_attribute:meta])* $field:ident: $value:ty),* $(,)? }) => {
        $(#[$attribute])*
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            $($(#[$field_attribute])* pub $field: Option<$value>,)*
        }
    };
}

config!(
    /// Explicit ES2016 transform overrides.
    ES2016Config {
        exponentiation: bool
    }
);
config!(/// Explicit ES2017 transform overrides.
    ES2017Config {
        async_to_generator: BoolOr<AsyncToGeneratorConfig>,
        async_generator_functions: BoolOr<AsyncToGeneratorConfig>,
    }
);
config!(/// Explicit ES2018 transform overrides.
    ES2018Config { object_rest_spread: BoolOr<ObjectRestSpreadConfig> });
config!(
    /// Explicit ES2019 transform overrides.
    ES2019Config {
        optional_catch_binding: bool
    }
);
config!(/// Explicit ES2020 transform overrides.
    ES2020Config {
        export_namespace_from: bool,
        nullish_coalescing: BoolOr<NullishCoalescingConfig>,
        optional_chaining: BoolOr<OptionalChainingConfig>,
    }
);
config!(
    /// Explicit ES2021 transform overrides.
    ES2021Config {
        logical_assignments: bool
    }
);
config!(/// Explicit ES2022 transform overrides.
    ES2022Config {
        class_fields_use_set: BoolOr<ClassFieldsUseSetConfig>,
        static_blocks: bool,
        class_properties: BoolOr<ClassPropertiesConfig>,
        private_in_object: bool,
    }
);
config!(
    /// Explicit RegExp overrides. Indices and lookbehind have no automatic
    /// target selection in SWC's preset; they must be enabled explicitly.
    RegExpConfig {
        dot_all_regex: bool,
        has_indices: bool,
        lookbehind_assertion: bool,
        /// Reserved for named capture group lowering. Currently ignored.
        named_capturing_groups_regex: bool,
        /// Reserved for duplicate named capture group lowering. Currently ignored.
        duplicate_named_capturing_groups_regex: bool,
        sticky_regex: bool,
        /// Reserved for Unicode property escape lowering. Currently ignored.
        unicode_property_regex: bool,
        unicode_regex: bool,
        unicode_sets_regex: bool,
    }
);

macro_rules! pass_config {
    ($(#[$attribute:meta])* $config:ident => $options:ty { $($field:ident),* $(,)? }) => {
        config!($(#[$attribute])* $config { $($field: bool,)* });
        impl From<$config> for $options {
            fn from(config: $config) -> Self {
                Self { $($field: config.$field.unwrap_or_default(),)* }
            }
        }
    };
}

pass_config!(/// SWC async pass flags. Unset fields use SWC's `false` defaults.
    AsyncToGeneratorConfig => AsyncToGeneratorOptions { ignore_function_length });
pass_config!(/// SWC object-rest/spread flags; spelling follows the SWC pass.
    ObjectRestSpreadConfig => ObjectRestSpreadOptions { no_symbol, set_property, pure_getters });
pass_config!(/// SWC nullish-coalescing flags.
    NullishCoalescingConfig => NullishCoalescingOptions { no_document_all });
pass_config!(/// SWC optional-chaining flags; `pure_getter` is singular in SWC.
    OptionalChainingConfig => OptionalChainingOptions { no_document_all, pure_getter });
pass_config!(/// SWC class-field assignment flags.
    ClassFieldsUseSetConfig => ClassFieldsUseSetOptions { pure_getters });
pass_config!(/// SWC class-properties flags; `pure_getter` is singular in SWC.
    ClassPropertiesConfig => ClassPropertiesOptions {
        private_as_properties, set_public_fields, constant_super, no_document_all, pure_getter,
    }
);

fn apply_pass<C: Clone + Into<O>, O: Default>(config: &Option<BoolOr<C>>, options: &mut Option<O>) {
    match config {
        None => {}
        Some(BoolOr::Bool(false)) => *options = None,
        Some(BoolOr::Bool(true)) => *options = Some(O::default()),
        Some(BoolOr::Options(config)) => *options = Some(config.clone().into()),
    }
}

macro_rules! apply_flags {
    ($options:expr, $config:expr, $($field:ident),+ $(,)?) => {
        $(if let Some(value) = $config.$field { $options.$field = value; })+
    };
}

impl EnvConfig {
    /// Resolve targets, then apply explicit configuration over the target defaults.
    ///
    /// Errors are returned for invalid queries, malformed engine versions, and
    /// empty engine maps. Queries are explicit and do not discover local config
    /// files. Engines absent from a feature's compatibility data conservatively
    /// require lowering. This includes Bun, which has no entry in the pinned data.
    pub fn to_env_options(&self) -> Result<EnvOptions, TargetError> {
        let target = self.target.as_ref().map(Target::resolve).transpose()?;
        let needs = |feature| target.as_ref().is_some_and(|target| target.needs(feature));
        let mut options = EnvOptions::default();
        if let Some(target) = &target {
            options.module_features.arrow = !target.needs(Feature::ArrowFunctions);
            options.module_features.block_scoping = !target.needs(Feature::BlockScoping);
        }
        options.es2016.exponentiation = needs(Feature::Exponentiation);
        options.es2017.async_to_generator =
            needs(Feature::AsyncToGenerator).then_some(Default::default());
        options.es2017.async_generator_functions =
            needs(Feature::AsyncGeneratorFunctions).then_some(Default::default());
        options.es2018.object_rest_spread =
            needs(Feature::ObjectRestSpread).then_some(Default::default());
        options.es2019.optional_catch_binding = needs(Feature::OptionalCatchBinding);
        options.es2020.export_namespace_from = needs(Feature::ExportNamespaceFrom);
        options.es2020.nullish_coalescing =
            needs(Feature::NullishCoalescing).then_some(Default::default());
        options.es2020.optional_chaining =
            needs(Feature::OptionalChaining).then_some(Default::default());
        options.es2021.logical_assignments = needs(Feature::LogicalAssignments);
        options.es2022.static_blocks = needs(Feature::StaticBlocks);
        options.es2022.class_properties = (needs(Feature::ClassProperties)
            || needs(Feature::PrivateMethods))
        .then_some(Default::default());
        options.es2022.private_in_object = needs(Feature::PrivateInObject);
        options.regexp.dot_all_regex = needs(Feature::DotAllRegex);
        options.regexp.named_capturing_groups_regex = needs(Feature::NamedCapturingGroupsRegex);
        options.regexp.duplicate_named_capturing_groups_regex =
            needs(Feature::DuplicateNamedCapturingGroupsRegex);
        options.regexp.sticky_regex = needs(Feature::StickyRegex);
        options.regexp.unicode_property_regex = needs(Feature::UnicodePropertyRegex);
        options.regexp.unicode_regex = needs(Feature::UnicodeRegex);
        options.regexp.unicode_sets_regex = needs(Feature::UnicodeSetsRegex);

        apply_flags!(options.es2016, self.es2016, exponentiation);
        apply_pass(
            &self.es2017.async_to_generator,
            &mut options.es2017.async_to_generator,
        );
        apply_pass(
            &self.es2017.async_generator_functions,
            &mut options.es2017.async_generator_functions,
        );
        apply_pass(
            &self.es2018.object_rest_spread,
            &mut options.es2018.object_rest_spread,
        );
        apply_flags!(options.es2019, self.es2019, optional_catch_binding);
        apply_flags!(options.es2020, self.es2020, export_namespace_from);
        apply_pass(
            &self.es2020.nullish_coalescing,
            &mut options.es2020.nullish_coalescing,
        );
        apply_pass(
            &self.es2020.optional_chaining,
            &mut options.es2020.optional_chaining,
        );
        apply_flags!(options.es2021, self.es2021, logical_assignments);
        apply_flags!(
            options.es2022,
            self.es2022,
            static_blocks,
            private_in_object
        );
        apply_pass(
            &self.es2022.class_fields_use_set,
            &mut options.es2022.class_fields_use_set,
        );
        apply_pass(
            &self.es2022.class_properties,
            &mut options.es2022.class_properties,
        );
        apply_flags!(
            options.regexp,
            self.regexp,
            dot_all_regex,
            has_indices,
            lookbehind_assertion,
            named_capturing_groups_regex,
            duplicate_named_capturing_groups_regex,
            sticky_regex,
            unicode_property_regex,
            unicode_regex,
            unicode_sets_regex
        );
        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::env::{Engine, EsVersion, ModuleFeatures};

    fn resolve(target: Target) -> EnvOptions {
        EnvConfig {
            target: Some(target),
            ..Default::default()
        }
        .to_env_options()
        .unwrap()
    }

    fn engine(engine: Engine, version: &str) -> Target {
        Target::Engines(BTreeMap::from([(engine, version.into())]))
    }

    fn modern_env_options() -> EnvOptions {
        EnvOptions {
            module_features: ModuleFeatures {
                block_scoping: true,
                arrow: true,
            },
            ..Default::default()
        }
    }

    #[test]
    fn selection() {
        let options = EnvConfig::default().to_env_options().unwrap();
        assert_eq!(options, EnvOptions::default());

        let options = resolve(Target::EsVersion(EsVersion::Es2015));
        assert!(options.es2016.exponentiation);
        assert!(options.es2017.async_to_generator.is_some());
        assert!(options.es2018.object_rest_spread.is_some());
        assert!(options.es2019.optional_catch_binding);
        assert!(options.es2022.static_blocks);

        let options = resolve(Target::EsVersion(EsVersion::Es2018));
        assert!(!options.es2016.exponentiation);
        assert!(options.es2017.async_to_generator.is_none());
        assert!(options.es2019.optional_catch_binding);
        assert!(options.regexp.unicode_sets_regex);

        let options = resolve(Target::EsVersion(EsVersion::Es2017));
        assert!(options.es2017.async_to_generator.is_none());
        assert!(options.es2017.async_generator_functions.is_some());

        let options = resolve(Target::EsVersion(EsVersion::EsNext));
        assert_eq!(options, modern_env_options());

        let options = resolve(engine(Engine::Node, "12"));
        assert!(!options.es2019.optional_catch_binding);
        assert!(options.es2020.nullish_coalescing.is_some());
        assert!(options.es2022.class_properties.is_some());

        let options = resolve(engine(Engine::Node, "16"));
        assert!(options.es2022.class_properties.is_none());

        let options = resolve(engine(Engine::Chrome, "83"));
        assert!(options.es2022.class_properties.is_some());

        let options = resolve(engine(Engine::Chrome, "84"));
        assert!(options.es2022.class_properties.is_none());

        let options = resolve(engine(Engine::Node, "8.9"));
        assert!(options.regexp.dot_all_regex);

        let options = resolve(engine(Engine::Node, "8.10"));
        assert!(!options.regexp.dot_all_regex);

        let options = resolve(Target::Engines(BTreeMap::from([
            (Engine::Chrome, "120".into()),
            (Engine::Safari, "13".into()),
        ])));
        assert!(options.es2020.nullish_coalescing.is_some());
    }

    #[test]
    fn browser_queries() {
        let options = resolve(Target::Browserslist(vec![
            "chrome 80".into(),
            "chrome 120".into(),
        ]));
        assert!(!options.es2019.optional_catch_binding);
        assert!(options.es2020.optional_chaining.is_some());
        assert!(options.es2022.static_blocks);

        let options = resolve(Target::Browserslist(vec!["chrome > 9999".into()]));
        assert_eq!(options, modern_env_options());

        let options = resolve(Target::Browserslist(Vec::new()));
        assert_eq!(
            options,
            resolve(Target::Browserslist(vec!["defaults".into()]))
        );
    }

    #[test]
    fn overrides() {
        let config = EnvConfig {
            target: Some(Target::EsVersion(EsVersion::Es2015)),
            es2016: ES2016Config {
                exponentiation: Some(false),
            },
            es2020: ES2020Config {
                optional_chaining: Some(BoolOr::Bool(false)),
                ..Default::default()
            },
            es2022: ES2022Config {
                class_properties: Some(BoolOr::Options(ClassPropertiesConfig {
                    pure_getter: Some(true),
                    ..Default::default()
                })),
                ..Default::default()
            },
            ..Default::default()
        };
        let options = config.to_env_options().unwrap();
        assert!(!options.es2016.exponentiation);
        assert!(options.es2020.optional_chaining.is_none());
        assert!(options.es2022.class_properties.unwrap().pure_getter);
        assert!(options.es2019.optional_catch_binding);

        let config = EnvConfig {
            target: Some(Target::EsVersion(EsVersion::EsNext)),
            es2017: ES2017Config {
                async_to_generator: Some(BoolOr::Bool(true)),
                ..Default::default()
            },
            ..Default::default()
        };
        let options = config.to_env_options().unwrap();
        assert_eq!(
            options.es2017.async_to_generator,
            Some(AsyncToGeneratorOptions::default())
        );
    }

    #[test]
    fn target_errors() {
        let config = EnvConfig {
            target: Some(engine(Engine::Node, "18.invalid")),
            ..Default::default()
        };
        assert!(config.to_env_options().is_err());

        let config = EnvConfig {
            target: Some(engine(Engine::Node, "1.2.3.4")),
            ..Default::default()
        };
        assert!(config.to_env_options().is_err());

        let config = EnvConfig {
            target: Some(Target::Engines(BTreeMap::new())),
            ..Default::default()
        };
        assert!(config.to_env_options().is_err());

        let config = EnvConfig {
            target: Some(Target::Browserslist(vec!["not-a-browser 1".into()])),
            ..Default::default()
        };
        assert!(config.to_env_options().is_err());
    }
}
