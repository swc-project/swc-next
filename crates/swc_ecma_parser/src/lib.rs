//! EcmaScript/TypeScript parser for the rust programming language.
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

pub fn with_file_parser<'p, 'a, 'cmt, T>(
    ast: &'p mut Ast<'a>,
    src: &'a str,
    syntax: Syntax,
    target: EsVersion,
    comments: Option<&'cmt mut Comments<'a>>,
    op: impl FnOnce(&mut Parser<'p, 'a, self::Lexer<'a, 'cmt>>) -> T,
) -> T {
    let lexer = self::Lexer::new(ast, syntax, target, StringSource::new(src), comments);
    let mut p = Parser::new_from(ast, lexer);
    op(&mut p)
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
        pub fn $name<'p, 'a, 'cmt>(
            ast: &'p mut Ast<'a>,
            src: &'a str,
            syntax: Syntax,
            target: EsVersion,
            comments: Option<&'cmt mut Comments<'a>>,
        ) -> PResult<$T> {
            with_file_parser(ast, src, syntax, target, comments, $($t)*)
        }
    };
}

expose!(parse_file_as_expr, Expr, |p: &mut Parser<
    '_,
    'a,
    Lexer<'a, '_>,
>| { p.parse_expr() });
expose!(parse_file_as_module, Module, |p: &mut Parser<
    '_,
    'a,
    Lexer<'a, '_>,
>| { p.parse_module() });
expose!(parse_file_as_script, Script, |p: &mut Parser<
    '_,
    'a,
    Lexer<'a, '_>,
>| { p.parse_script() });
expose!(parse_file_as_commonjs, Script, |p: &mut Parser<
    '_,
    'a,
    Lexer<'a, '_>,
>| { p.parse_commonjs() });
expose!(parse_file_as_program, Program, |p: &mut Parser<
    '_,
    'a,
    Lexer<'a, '_>,
>| { p.parse_program() });
