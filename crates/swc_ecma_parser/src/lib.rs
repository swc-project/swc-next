//! An ECMAScript parser that writes directly into the flat SWC Next AST.

#![deny(clippy::all)]
#![deny(unused)]
#![allow(clippy::too_many_arguments)]

mod config;
mod lexer;
mod parser;
mod token;
mod unicode_id;
mod util;

pub use config::{NoTokenParserConfig, ParserConfig, TokenParserConfig};
pub use parser::{CommentMode, Options, ParseReturn, Parser};
pub use swc_next_ecma_ast::{Diagnostic, Label, Lang, Severity};
pub use token::{Token, TokenFlag, TokenTag};

impl<'parser, 'a, C: ParserConfig> Parser<'parser, 'a, C> {
    pub fn parse(self) -> ParseReturn<'a> {
        self.parse_program()
    }
}
