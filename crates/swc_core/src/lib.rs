//! Core APIs for swc-next.
//!
//! Enable `js` for JavaScript configuration types, and select stages with
//! `js_parser`, `js_codegen`, and `js_transform`. The `js_transform` feature includes
//! parsing, code generation, and minification.
//! The optional `bridge` feature enables FFI conversions and type generation for
//! the enabled APIs.
//! Enable `serde` to deserialize configuration types. JSON uses camelCase fields
//! and rejects unknown fields.

#[cfg(feature = "js")]
pub mod js;

/// A boolean shorthand or an explicit options object.
///
/// `Bool(false)` disables a stage, `Bool(true)` enables its defaults, and
/// `Options` enables it with explicit settings. An enclosing `Option` preserves
/// the distinction between an omitted setting and an explicit boolean.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Clone, Copy, Debug)]
pub enum BoolOr<T> {
    Bool(bool),
    Options(T),
}
