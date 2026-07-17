pub mod ast_convert;
pub mod codegen;
pub mod parser;
pub mod remove_paren;
pub mod semantic;

use std::panic::{AssertUnwindSafe, catch_unwind};

use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Ast, Program};
use swc_experimental_ecma_parser::{Parser, StringSource, Syntax, error::Error};

use crate::cases::{Case, IsModule};

pub enum ParseResult<'a> {
    Succ(Program, Ast<'a>),
    Fail(Vec<Error>),
    Panic,
    Ignore,
}

pub fn parse<'a, C: Case>(allocator: &'a Allocator, case: &'a C) -> ParseResult<'a> {
    parse_code(allocator, case.code(), case.syntax(), case.is_module())
}

pub fn parse_code<'a>(
    allocator: &'a Allocator,
    code: &'a str,
    syntax: Syntax,
    is_module: IsModule,
) -> ParseResult<'a> {
    let input = StringSource::new(code);
    let mut ast = Ast::new(input.source_len(), allocator);
    let mut parser = Parser::new(&mut ast, syntax, input, None);
    let ret = match is_module {
        IsModule::Script => catch_unwind(AssertUnwindSafe(|| {
            parser.parse_script().map(Program::Script)
        })),
        IsModule::Module => catch_unwind(AssertUnwindSafe(|| {
            parser.parse_module().map(Program::Module)
        })),
        IsModule::Unknown => catch_unwind(AssertUnwindSafe(|| parser.parse_program())),
        IsModule::Skip => {
            return ParseResult::Ignore;
        }
    };

    match ret {
        Ok(ret) => match ret {
            Ok(root) => {
                let errors = parser.take_errors();
                if !errors.is_empty() {
                    return ParseResult::Fail(errors);
                }
                drop(parser);
                ParseResult::Succ(root, ast)
            }
            Err(e) => ParseResult::Fail(vec![e]),
        },
        Err(_) => ParseResult::Panic,
    }
}
