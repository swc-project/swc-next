//! EcmaScript/TypeScript parser for the rust programming language.
//!
//! # Features
//!
//! ## Heavily tested
//!
//! Passes almost all tests from [tc39/test262][].
//!
//! ## Error reporting
//!
//! ```sh
//! error: 'implements', 'interface', 'let', 'package', 'private', 'protected',  'public', 'static', or 'yield' cannot be used as an identifier in strict mode
//!  --> invalid.js:3:10
//!   |
//! 3 | function yield() {
//!   |          ^^^^^
//! ```
//!
//! ## Error recovery
//!
//! The parser can recover from some parsing errors. For example, parser returns
//! `Ok(Module)` for the code below, while emitting error to handler.
//!
//! ```ts
//! const CONST = 9000 % 2;
//! const enum D {
//!     // Comma is required, but parser can recover because of the newline.
//!     d = 10
//!     g = CONST
//! }
//! ```
//!
//! # Example (lexer)
//!
//! See `lexer.rs` in examples directory.
//!
//! # Example (parser)
//!
//! ```ignore
//! #[macro_use]
//! extern crate swc_common;
//! extern crate swc_ecma_parser;
//! use swc_core::common::sync::Lrc;
//! use swc_core::common::{
//!     errors::{ColorConfig, Handler},
//!     FileName, FilePathMapping, SourceMap,
//! };
//! use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
//!
//! fn main() {
//!     let cm: Lrc<SourceMap> = Default::default();
//!     let handler =
//!         Handler::with_tty_emitter(ColorConfig::Auto, true, false,
//!         Some(cm.clone()));
//!
//!     // Real usage
//!     // let fm = cm
//!     //     .load_file(Path::new("test.js"))
//!     //     .expect("failed to load test.js");
//!     let fm = cm.new_source_file(
//!         FileName::Custom("test.js".into()).into(),
//!         "function foo() {}",
//!     );
//!     let lexer = Lexer::new(
//!         // We want to parse ecmascript
//!         Syntax::Es(Default::default()),
//!         // EsVersion defaults to es5
//!         Default::default(),
//!         StringInput::from(&*fm),
//!         None,
//!     );
//!
//!     let mut parser = Parser::new_from(lexer);
//!
//!     for e in parser.take_errors() {
//!         e.into_diagnostic(&handler).emit();
//!     }
//!
//!     let _module = parser
//!         .parse_module()
//!         .map_err(|mut e| {
//!             // Unrecoverable fatal error occurred
//!             e.into_diagnostic(&handler).emit()
//!         })
//!         .expect("failed to parser module");
//! }
//! ```
//!
//! ## Cargo features
//!
//! ### `typescript`
//!
//! Enables typescript parser.
//!
//! ### `verify`
//!
//! Verify more errors, using `swc_ecma_visit`.
//!
//! ## Known issues
//!
//! ### Null character after `\`
//!
//! Because [String] of rust should only contain valid utf-8 characters while
//! javascript allows non-utf8 characters, the parser stores invalid utf8
//! characters in escaped form.
//!
//! As a result, swc needs a way to distinguish invalid-utf8 code points and
//! input specified by the user. The parser stores a null character right after
//! `\\` for non-utf8 code points. Note that other parts of swc is aware of this
//! fact.
//!
//! Note that this can be changed at anytime with a breaking change.
//!
//! [tc39/test262]:https://github.com/tc39/test262

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(clippy::all)]
#![deny(unused)]
#![allow(unexpected_cfgs)]
#![allow(clippy::nonminimal_bool)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_unwrap)]
#![allow(clippy::vec_box)]
#![allow(clippy::wrong_self_convention)]
#![allow(clippy::match_like_matches_macro)]

#[cfg(feature = "unstable")]
pub mod unstable {
    //! This module expose tokens related to the `swc_ecma_parser::lexer`.
    //!
    //! Unlike the tokens re-exported from `swc_ecma_lexer`, the token kinds
    //! defined in the `swc_ecma_parser` here are non-strict for higher
    //! performance.
    //!
    //! Although it's marked as unstable, we can ensure that we will not
    //! introduce too many breaking changes. And we also encourage the
    //! applications to migrate to the lexer and tokens in terms of
    //! the performance.
    //!
    //! Also see the dicussion https://github.com/swc-project/swc/discussions/10683
    pub use crate::lexer::{
        capturing::Capturing,
        token::{NextTokenAndSpan, Token, TokenAndSpan, TokenValue},
    };
}

use error::Error;
use swc_core::common::comments::Comments;
use swc_experimental_ecma_ast::*;

mod context;
pub mod error;
pub mod lexer;
mod parser;
mod syntax;

pub use context::Context;
pub use lexer::Lexer;
pub use lexer::source::StringSource;
pub use parser::*;
pub use syntax::{EsSyntax, Syntax, SyntaxFlags, TsSyntax};

pub struct ParseRet<T, I> {
    pub ast: Ast,
    pub errors: Vec<Error>,
    pub root: T,
    pub input: I,
}

impl<T, I> ParseRet<T, I> {
    pub fn map_root<U, F: FnOnce(T) -> U>(self, op: F) -> ParseRet<U, I> {
        ParseRet {
            ast: self.ast,
            errors: self.errors,
            root: op(self.root),
            input: self.input,
        }
    }
}

pub fn with_file_parser<'a, T>(
    src: &'a str,
    syntax: Syntax,
    target: EsVersion,
    comments: Option<&'a dyn Comments>,
    op: impl FnOnce(Parser<self::Lexer<'a>>) -> T,
) -> T {
    let lexer = self::Lexer::new(syntax, target, StringSource::new(src), comments);
    let p = Parser::new_from(lexer);
    op(p)
}

macro_rules! expose {
    (
        $name:ident,
        $T:ty,
        $($t:tt)*
    ) => {
        /// Note: This is recommended way to parse a file.
        ///
        /// This is an alias for [Parser], [Lexer] and [SourceFileInput], but
        /// instantiation of generics occur in `swc_ecma_parser` crate.
        pub fn $name<'a>(
            src: &'a str,
            syntax: Syntax,
            target: EsVersion,
            comments: Option<&'a dyn Comments>,
        ) -> PResult<ParseRet<$T, self::Lexer<'a>>> {
            with_file_parser(src, syntax, target, comments, $($t)*)
        }
    };
}

expose!(parse_file_as_expr, Expr, |p| { p.parse_expr() });
expose!(parse_file_as_module, Module, |p| { p.parse_module() });
expose!(parse_file_as_script, Script, |p| { p.parse_script() });
expose!(parse_file_as_commonjs, Script, |p| { p.parse_commonjs() });
expose!(parse_file_as_program, Program, |p| { p.parse_program() });
