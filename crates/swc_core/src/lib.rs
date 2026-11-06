//! Core APIs for swc-next.
//!
//! Enable `js` for JavaScript configuration types, and select stages with
//! `js_parser`, `js_codegen`, `js_transform`, and `js_minify`.
//! The optional `bridge` feature enables FFI conversions and type generation for
//! the enabled APIs.

#[cfg(feature = "js")]
pub mod js;

/// A boolean shorthand or an explicit options object.
///
/// `Bool(false)` disables a stage, `Bool(true)` enables its defaults, and
/// `Options` enables it with explicit settings. An enclosing `Option` preserves
/// the distinction between an omitted setting and an explicit boolean.
#[derive(Clone, Copy, Debug)]
pub enum BoolOr<T> {
    Bool(bool),
    Options(T),
}
