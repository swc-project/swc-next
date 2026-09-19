//! JavaScript minification for the flat SWC Next AST.
//!
//! Parsing and printing are separate operations, provided by
//! `swc_next_ecma_parser` and `swc_next_ecma_codegen`. Use the code generator's
//! `minify` option to print the optimized AST compactly.
//!
//! Compression currently shortens boolean expressions: `true` becomes `!0`
//! and `false` becomes `!1`.

#![deny(clippy::all)]
#![deny(unused)]

use swc_next_ecma_ast::Ast;

mod compress;
mod options;

pub use options::{CompressOptions, MinifierOptions};

/// Optimizes a JavaScript program in place.
pub struct Minifier;

impl Minifier {
    /// Minifies a valid JavaScript Program AST.
    ///
    /// The caller must handle parser and semantic errors before calling this
    /// method. Parsing, lowering non-JavaScript syntax, and printing remain the
    /// caller's responsibility.
    ///
    /// Transformations may replace nodes and invalidate semantic information.
    /// Recompute semantic analysis before querying the transformed program;
    /// do not reuse a model computed before minification.
    ///
    /// With `compress: None`, the AST is left unchanged.
    ///
    /// # Panics
    ///
    /// Panics if compression is enabled and the AST root is not a program.
    pub fn minify(ast: &mut Ast<'_>, options: MinifierOptions) {
        if let Some(options) = options.compress {
            compress::compress(ast, options);
        }
    }
}
