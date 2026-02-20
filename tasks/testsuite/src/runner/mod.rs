pub mod parser;
pub mod semantic;
pub mod transform_remove_paren;

use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};

use swc_core::common::comments::SingleThreadedComments;
use swc_experimental_ecma_ast::{Ast, Program};
use swc_experimental_ecma_parser::{Parser, StringSource, error::Error};

use crate::cases::{Case, IsModule};

pub enum ParseResult {
    Succ((Program, Ast)),
    Fail(Vec<Error>),
    Panic,
    Ignore,
}

pub fn parse<C: Case>(case: &C) -> ParseResult {
    let input = StringSource::new(case.code());
    let comments = SingleThreadedComments::default();
    let mut ast = Ast::new(input.source_len(), Rc::default());
    let mut parser = Parser::new(&mut ast, case.syntax(), input, Some(&comments));
    let ret = match case.is_module() {
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
                ParseResult::Succ((root, ast))
            }
            Err(e) => ParseResult::Fail(vec![e]),
        },
        Err(_) => ParseResult::Panic,
    }
}
