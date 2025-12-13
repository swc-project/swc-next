pub mod parser;
pub mod parser_no_memory_hole;
pub mod semantic;
pub mod transform_remove_paren;

use std::panic::{AssertUnwindSafe, catch_unwind};

use swc_core::common::comments::SingleThreadedComments;
use swc_experimental_ecma_ast::{Ast, Program};
use swc_experimental_ecma_parser::{Lexer, Parser, StringSource, error::Error};

use crate::cases::{Case, IsModule};

pub enum ParseResult {
    Succ((Program, Ast)),
    Fail(Vec<Error>),
    Panic,
    Ignore,
}

pub fn parse<C: Case>(case: &C) -> ParseResult {
    let syntax = case.syntax();
    let input = StringSource::new(case.code());
    let comments = SingleThreadedComments::default();
    let lexer = Lexer::new(syntax, Default::default(), input, Some(&comments));
    let parser = Parser::new_from(lexer);
    let ret = match case.is_module() {
        IsModule::Script => catch_unwind(AssertUnwindSafe(|| {
            parser
                .parse_script()
                .map(|ret| ret.map_root(Program::Script))
        })),
        IsModule::Module => catch_unwind(AssertUnwindSafe(|| {
            parser
                .parse_module()
                .map(|ret| ret.map_root(Program::Module))
        })),
        IsModule::Unknown => catch_unwind(AssertUnwindSafe(|| parser.parse_program())),
        IsModule::Skip => {
            return ParseResult::Ignore;
        }
    };

    match ret {
        Ok(ret) => match ret {
            Ok(ret) => {
                if !ret.errors.is_empty() {
                    return ParseResult::Fail(ret.errors);
                }
                ParseResult::Succ((ret.root, ret.ast))
            }
            Err(e) => ParseResult::Fail(vec![e]),
        },
        Err(_) => ParseResult::Panic,
    }
}
