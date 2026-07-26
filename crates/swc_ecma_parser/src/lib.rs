//! An ECMAScript parser that writes directly into the flat SWC Next AST.

#![deny(clippy::all)]
#![deny(unused)]
#![allow(clippy::too_many_arguments)]

mod lexer;
mod parser;
mod token;
mod unicode_id;
mod util;

pub use lexer::{Hashbang, Lexer, LexerMode, LexicalError, RegexResult};
pub use parser::{
    ArrayCover, BodyKind, Checkpoint, CommentMode, Context, ObjectCover, Options,
    ParenthesizedCover, Parser, ParserState, Peek, ReportOptions, ScratchBuffer, TsContext,
    build_line_starts,
};
pub use swc_next_ecma_ast::{Diagnostic, Label, Lang, Severity};
pub use token::{Token, TokenFlag, TokenTag};

use swc_next_ecma_ast::{Ast, Program};

/// Parse one source file into `ast`.
pub fn parse<'a>(ast: &mut Ast<'a>, source: &[u8], options: Options) -> Option<Program> {
    Parser::init(ast, source, options).parse()
}
