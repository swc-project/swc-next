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

#[cfg(test)]
mod test {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::Ast;

    use crate::{NoTokenParserConfig, Options, Parser};

    #[test]
    fn parse_expression() {
        let allocator = Allocator::default();
        let source = "1 + 1";
        let mut ast = Ast::new(source, &allocator);
        let parser = Parser::init(&mut ast, Options::default(), NoTokenParserConfig);
        let expr = parser.parse_expression().unwrap();
        assert!(matches!(
            ast.expr_data(expr),
            swc_next_ecma_ast::ExprData::BinaryExpression(_)
        ));
    }
}
