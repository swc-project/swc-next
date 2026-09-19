use swc_next_ecma_ast::{Ast, Expr, ExprData, GetSpan, UnaryOperator, VisitMut, VisitMutWith};

use crate::CompressOptions;

pub(super) fn compress(ast: &mut Ast<'_>, options: CompressOptions) {
    ast.root_program()
        .visit_mut_with(&mut Compressor { ast, options });
}

struct Compressor<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    options: CompressOptions,
}

impl<'ast> VisitMut<'ast> for Compressor<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        if self.options.booleans
            && let ExprData::BooleanLiteral(literal) = self.ast.expr_data(node)
        {
            let span = literal.span(self.ast);
            let value = if literal.value(self.ast) { 0.0 } else { 1.0 };
            let argument = self.ast.numeric_literal(span, value, None);
            let replacement = self.ast.unary_expression(
                span,
                Expr::NumericLiteral(argument),
                UnaryOperator::LogicalNot,
            );
            return Some(Expr::UnaryExpression(replacement));
        }
        node.visit_mut_children_with(self)
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, Expr, ExprData, GetSpan, StmtData};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

    use crate::{Minifier, MinifierOptions};

    fn parse<'a>(allocator: &'a Allocator, source: &'a str) -> Ast<'a> {
        let result =
            Parser::init(allocator, source, Options::default(), NoTokenParserConfig).parse();
        assert!(result.diagnostics.is_empty());
        result.ast
    }

    fn expression(ast: &Ast<'_>) -> Expr {
        let statement =
            ast.get_node_in_sub_range(ast.root_program().body(ast).iter().next().unwrap());
        let StmtData::ExpressionStatement(statement) = ast.stmt_data(statement) else {
            panic!();
        };
        statement.expression(ast)
    }

    #[test]
    fn source_spans() {
        let allocator = Allocator::new();
        let mut ast = parse(&allocator, "  true;");
        let span = expression(&ast).span(&ast);
        Minifier::minify(&mut ast, MinifierOptions::default());
        let ExprData::UnaryExpression(replacement) = ast.expr_data(expression(&ast)) else {
            panic!();
        };
        assert_eq!(replacement.span(&ast), span);
        assert_eq!(replacement.argument(&ast).span(&ast), span);
    }
}
