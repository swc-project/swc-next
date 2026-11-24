//! Environment configuration and resolved transform options.
//!
//! [`EnvConfig`] accepts targets and per-pass overrides. Its fallible conversion
//! resolves browser queries and engine versions into [`EnvOptions`], which the
//! transformer consumes without retaining target configuration or legacy ASTs.

mod compat;
mod config;
mod target;

pub use config::{
    AsyncToGeneratorConfig, ClassFieldsUseSetConfig, ClassPropertiesConfig, ES2016Config,
    ES2017Config, ES2018Config, ES2019Config, ES2020Config, ES2021Config, ES2022Config, EnvConfig,
    NullishCoalescingConfig, ObjectRestSpreadConfig, OptionalChainingConfig, RegExpConfig,
};
pub use target::{Engine, EsVersion, Target, TargetError};

use crate::{es2016, es2017, es2018, es2019, es2020, es2021, es2022, regexp};

/// Concrete execution options. All transforms are disabled by default.
///
/// Edition groups select passes rather than full language-target presets. This
/// crate currently covers ES2016–ES2022 and RegExp; choosing an older target
/// selects only these passes, not ES2015 or ES3 lowering.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EnvOptions {
    pub es2016: es2016::ES2016Options,
    pub es2017: es2017::ES2017Options,
    pub es2018: es2018::ES2018Options,
    pub es2019: es2019::ES2019Options,
    pub es2020: es2020::ES2020Options,
    pub es2021: es2021::ES2021Options,
    pub es2022: es2022::ES2022Options,
    pub regexp: regexp::RegExpOptions,
}
