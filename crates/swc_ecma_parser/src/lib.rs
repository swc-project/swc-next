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
    ArrayCover, Checkpoint, CommentMode, Context, ObjectCover, Options, ParenthesizedCover, Parser,
    ParserState, Peek, ReportOptions, ScratchBuffer, TsContext, build_line_starts,
};
pub use swc_next_ecma_ast::{Diagnostic, Label, Lang, Severity};
pub use token::{Token, TokenFlag, TokenTag};

use swc_next_ecma_ast::{Ast, Program};

/// Parse one source file into `ast`.
pub fn parse(ast: &mut Ast<'_>, source: &[u8], options: Options) -> Option<Program> {
    assert_eq!(
        ast.source(),
        source,
        "Ast and Parser must be initialized with the same source"
    );
    Parser::init(ast, options).parse()
}
