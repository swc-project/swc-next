//! Legacy SWC TypeScript erasure, runtime lowering, and TSX import retention.

mod config;
mod retain;
mod semantic;
mod shared;
mod transform;
mod ts_enum;
mod typescript;
mod utils;

pub use config::{
    Config as TypeScriptOptions, ImportsNotUsedAsValues, TsImportExportAssignConfig,
    TsxConfig as JsxImportOptions,
};
pub(crate) use typescript::typescript;
