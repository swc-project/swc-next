use std::panic::{AssertUnwindSafe, catch_unwind};

use swc_core::{
    common::BytePos,
    ecma::{
        ast::Program,
        parser::{EsSyntax as LegacyEsSyntax, Lexer, Parser, StringInput, Syntax as LegacySyntax},
    },
};
use swc_experimental_ecma_parser::Syntax;

use crate::cases::{Case, IsModule};

pub enum ParseResult {
    Succ(Program),
    Fail,
    Panic,
    Ignore,
}

pub fn parse<C: Case>(case: &C) -> ParseResult {
    let input = StringInput::new(
        case.code(),
        BytePos(1),
        BytePos(case.code().len() as u32 + 1),
    );
    let lexer = Lexer::new(
        legacy_syntax(case.syntax()),
        Default::default(),
        input,
        None,
    );
    let mut parser = Parser::new_from(lexer);

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
        Ok(Ok(root)) => {
            if parser.take_errors().is_empty() {
                ParseResult::Succ(root)
            } else {
                ParseResult::Fail
            }
        }
        Ok(Err(_)) => ParseResult::Fail,
        Err(_) => ParseResult::Panic,
    }
}

fn legacy_syntax(syntax: Syntax) -> LegacySyntax {
    match syntax {
        Syntax::Es(es) => LegacySyntax::Es(LegacyEsSyntax {
            jsx: es.jsx,
            fn_bind: es.fn_bind,
            decorators: es.decorators,
            decorators_before_export: es.decorators_before_export,
            export_default_from: es.export_default_from,
            import_attributes: es.import_attributes,
            allow_super_outside_method: es.allow_super_outside_method,
            allow_return_outside_function: es.allow_return_outside_function,
            auto_accessors: es.auto_accessors,
            explicit_resource_management: es.explicit_resource_management,
        }),
    }
}
