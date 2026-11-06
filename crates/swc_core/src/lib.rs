//! Core APIs for swc-next.
//!
//! Enable `parser` or `codegen` for the corresponding JavaScript APIs.
//! The optional `bridge` feature enables FFI conversions and type generation for
//! the enabled APIs.

pub mod js;
// pub mod css; // Unimplemented.
// pub mod html; // Unimplemented.

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
