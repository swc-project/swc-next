use super::options::TestOptions;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::Ast;
use swc_next_ecma_parser::{NoTokenParserConfig, Parser};
use swc_next_ecma_semantic2::{AnalyzeOptions, Semantic};

#[track_caller]
pub(crate) fn with_semantic<T>(
    source: &str,
    options: TestOptions,
    inspect: impl Fn(&Ast<'_>, Semantic<'_>) -> T,
) -> T {
    let allocator = Allocator::new();
    let parsed = Parser::init(&allocator, source, options.parser, NoTokenParserConfig).parse();
    // SAFETY: Every handle inspected below belongs to this AST and its allocator.
    let ast = unsafe { parsed.ast.ast() };
    let result = swc_next_ecma_semantic2::analyze(
        &parsed.ast,
        AnalyzeOptions {
            check_syntax: options.checks,
        },
    );
    inspect(ast, result.semantic.into_parts().1)
}
