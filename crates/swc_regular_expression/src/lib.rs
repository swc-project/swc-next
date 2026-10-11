//! Based on Oxc's `oxc_regular_expression` 0.137.0.
//!
//! Modified for SWC Next's allocator, diagnostics, and source spans, focusing on validation.

#![expect(clippy::missing_errors_doc)]

mod diagnostic;
mod diagnostics;
mod display;
mod options;
mod parser;
mod span;
mod string;
mod surrogate_pair;

pub mod ast;
#[cfg(test)]
pub use crate::parser::ConstructorParser;
pub use crate::{options::Options, parser::LiteralParser};
