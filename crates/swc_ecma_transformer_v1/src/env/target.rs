use std::{collections::BTreeMap, error::Error, fmt, str::FromStr};

use super::compat::Feature;

/// Environment input. Explicit transform configuration takes precedence over it.
///
/// Engine versions are numeric minimum versions (`"18"`, `"18.12"`, `"18.12.1"`),
/// not semver ranges. These are Rust configuration types; the fixture adapter
/// converts JSON through SWC's configuration types before constructing them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    EsVersion(EsVersion),
    /// Queries are resolved from bundled Browserslist data; configuration files
    /// are not searched. An empty list uses the `defaults` query.
    Browserslist(Vec<String>),
    /// The oldest supported release of each engine. Every specified engine must
    /// support a feature before its lowering can be omitted.
    Engines(BTreeMap<Engine, String>),
}

macro_rules! named_enum {
    ($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name {
            $($variant,)+
        }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $text,)+ }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = TargetError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($text => Ok(Self::$variant),)+
                    _ => Err(TargetError(format!("Unknown {}: {value}", stringify!($name)))),
                }
            }
        }
    };
}

named_enum!(EsVersion {
    // Es3 => "es3", Es5 => "es5",
    Es2015 => "es2015", Es2016 => "es2016",
    Es2017 => "es2017", Es2018 => "es2018", Es2019 => "es2019", Es2020 => "es2020",
    Es2021 => "es2021", Es2022 => "es2022", Es2023 => "es2023", Es2024 => "es2024",
    Es2025 => "es2025", Es2026 => "es2026", EsNext => "esnext",
});

named_enum!(Engine {
    Chrome => "chrome", Edge => "edge", Firefox => "firefox", Safari => "safari",
    Ios => "ios", Opera => "opera", Samsung => "samsung", Node => "node",
    Deno => "deno", /* Bun => "bun", */ Electron => "electron", Ie => "ie",
    Android => "android", OperaMobile => "opera_mobile", OperaMini => "op_mini",
    Blackberry => "bb", Baidu => "baidu", Kaios => "kaios", Qq => "and_qq", Uc => "and_uc",
});

/// Invalid browser queries, engine versions, or target descriptions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetError(pub(crate) String);

impl fmt::Display for TargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for TargetError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Version(pub u32, pub u32, pub u32);

impl FromStr for Version {
    type Err = TargetError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let mut parts = source.split('.');
        let error = || TargetError(format!("Invalid engine version: {source}"));
        let mut result = [0; 3];
        for (index, part) in parts.by_ref().enumerate() {
            if index >= result.len()
                || part.is_empty()
                || !part.bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(error());
            }
            result[index] = part.parse().map_err(|_| error())?;
        }
        Ok(Self(result[0], result[1], result[2]))
    }
}

pub(super) enum ResolvedTarget {
    Edition(EsVersion),
    Engines(BTreeMap<Engine, Version>),
    /// A valid query matched no releases. SWC treats this as a future target.
    EmptyQuery,
}

impl Target {
    pub(super) fn resolve(&self) -> Result<ResolvedTarget, TargetError> {
        match self {
            Self::EsVersion(version) => Ok(ResolvedTarget::Edition(*version)),
            Self::Engines(engines) => {
                if engines.is_empty() {
                    return Err(TargetError(
                        "An engine target must contain at least one engine".into(),
                    ));
                }
                engines
                    .iter()
                    .map(|(engine, version)| Ok((*engine, version.parse()?)))
                    .collect::<Result<_, _>>()
                    .map(ResolvedTarget::Engines)
            }
            Self::Browserslist(queries) => {
                let queries = if queries.is_empty() {
                    vec!["defaults".into()]
                } else {
                    queries.clone()
                };
                let distributions = browserslist::resolve(
                    &queries,
                    &browserslist::Opts {
                        mobile_to_desktop: true,
                        ignore_unknown_versions: true,
                        ..Default::default()
                    },
                )
                .map_err(|error| TargetError(error.to_string()))?;
                if distributions.is_empty() {
                    return Ok(ResolvedTarget::EmptyQuery);
                }
                let mut engines = BTreeMap::<Engine, Version>::new();
                for distribution in distributions {
                    let name = match distribution.name() {
                        "and_chr" => "chrome",
                        "and_ff" => "firefox",
                        "ios_saf" => "ios",
                        "op_mob" => "opera_mobile",
                        "ie_mob" => "ie",
                        name => name,
                    };
                    let engine: Engine = name.parse()?;
                    let version = match distribution.version() {
                        "TP" => Version(u32::MAX, 0, 0),
                        "all" => Version(0, 0, 0),
                        version => version.split('-').next().unwrap().parse()?,
                    };
                    engines
                        .entry(engine)
                        .and_modify(|current| *current = (*current).min(version))
                        .or_insert(version);
                }
                Ok(ResolvedTarget::Engines(engines))
            }
        }
    }
}

impl ResolvedTarget {
    pub(super) fn needs(&self, feature: Feature) -> bool {
        match self {
            Self::Edition(version) => *version < feature.edition(),
            Self::Engines(engines) => engines.iter().any(|(engine, version)| {
                feature
                    .support()
                    .iter()
                    .find(|(supported, _)| engine == supported)
                    .is_none_or(|(_, minimum)| version < minimum)
            }),
            Self::EmptyQuery => false,
        }
    }
}
