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
pub use lexer::{Hashbang, Lexer, LexerMode, LexicalError, RegexResult};
pub use parser::{
    ArrayCover, Checkpoint, CommentMode, Context, ObjectCover, Options, ParenthesizedCover,
    ParseReturn, Parser, ParserState, Peek, ReportOptions, ScratchBuffer, TsContext,
    build_line_starts,
};
pub use swc_next_ecma_ast::{Diagnostic, Label, Lang, Severity};
pub use token::{Token, TokenFlag, TokenTag};
