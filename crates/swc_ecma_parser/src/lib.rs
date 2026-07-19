//! An ECMAScript parser that writes directly into the flat experimental AST.

#![deny(clippy::all)]
#![deny(unused)]
#![allow(clippy::too_many_arguments)]

mod error;
mod lexer;
mod parser;
mod token;
mod unicode_id;
mod util;

pub use error::{Diagnostic, Error, Severity};
pub use lexer::{Hashbang, Lexer, LexerMode, LexicalError, RawComment, RegexResult};
pub use parser::{CommentMode, Lang, Options, Parser};
pub use token::{Precedence, Token, TokenFlag, TokenTag};

use swc_experimental_ecma_ast::{Ast, Comment, Program};

/// Parse one source file into `ast`.
pub fn parse<'a>(ast: &mut Ast<'a>, source: &[u8], options: Options) -> Result<Program, Error> {
    Parser::new(ast, source, options).parse_program()
}

/// Parse one source file and return its flat comments alongside the program.
pub fn parse_with_comments<'a>(
    ast: &mut Ast<'a>,
    source: &[u8],
    options: Options,
) -> Result<(Program, Vec<Comment<'a>>), Error> {
    let mut parser = Parser::new(ast, source, options);
    let program = parser.parse_program()?;
    let comments = parser.comments();
    Ok((program, comments))
}
