use std::collections::BTreeMap;

use super::options::TestOptions;
use serde::Serialize;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::Ast;
use swc_next_ecma_parser::{NoTokenParserConfig, Parser};
use swc_next_ecma_semantic::{
    AnalyzeOptions, Declaration, Reference, Semantic, check_syntax, scope::Scope,
};

#[derive(Debug, Default, Eq, PartialEq, Serialize)]
pub(crate) struct DiagnosticCounts {
    pub(crate) parser: usize,
    pub(crate) semantic: usize,
}

#[track_caller]
pub(crate) fn with_semantic<T>(
    source: &str,
    options: TestOptions,
    inspect: impl Fn(&Ast<'_>, Semantic<'_>) -> T,
) -> T {
    let (result, diagnostics) = analyze_fixture(source, options, inspect);
    assert_eq!(diagnostics, DiagnosticCounts::default());
    result
}

#[track_caller]
pub(crate) fn analyze_fixture<T>(
    source: &str,
    options: TestOptions,
    inspect: impl Fn(&Ast<'_>, Semantic<'_>) -> T,
) -> (T, DiagnosticCounts) {
    let allocator = Allocator::new();
    let parsed = Parser::init(&allocator, source, options.parser, NoTokenParserConfig).parse();
    // SAFETY: Every handle inspected below belongs to this AST and its allocator.
    let ast = unsafe { parsed.ast.ast() };
    let result = swc_next_ecma_semantic::analyze(
        &parsed.ast,
        AnalyzeOptions {
            check_syntax: options.checks,
        },
    );
    assert_eq!(
        result.diagnostics,
        check_syntax(&parsed.ast, options.checks)
    );
    let diagnostics = DiagnosticCounts {
        parser: parsed.diagnostics.len(),
        semantic: result.diagnostics.len(),
    };
    let semantic = result.semantic.into_parts().1;
    assert_indexes(ast, &semantic);
    (inspect(ast, semantic), diagnostics)
}

// The iterators and ID-based accessors expose the same records.
fn assert_indexes(ast: &Ast<'_>, semantic: &Semantic<'_>) {
    let scope_fields = |s: Scope| {
        (
            s.active,
            s.node,
            s.kind,
            s.parent,
            s.hoist_target,
            s.function_body,
            s.flags.strict,
            s.flags.arguments_barrier,
        )
    };
    for entry in semantic.iter_scopes() {
        assert_eq!(
            scope_fields(semantic.scope(entry.id)),
            scope_fields(entry.scope)
        );
    }

    let reference_fields = |r: Reference| (r.node, r.name, r.scope, r.flags, r.symbol);
    for entry in semantic.iter_references() {
        assert_eq!(
            reference_fields(semantic.reference(entry.id)),
            reference_fields(entry.reference)
        );
    }

    let declaration_fields = |d: Declaration| (d.node, d.symbol, d.scope, d.flags);
    let mut by_symbol = BTreeMap::new();
    let mut previous = 0;
    for (index, entry) in semantic.iter_declarations().enumerate() {
        assert_eq!(entry.id.index(), index);
        let declaration = semantic.declaration(entry.id);
        assert_eq!(
            declaration_fields(declaration),
            declaration_fields(entry.declaration)
        );
        let start = ast.span(declaration.node).start;
        assert!(previous <= start);
        previous = start;
        by_symbol
            .entry(declaration.symbol)
            .or_insert_with(Vec::new)
            .push(entry.id);
    }
    for entry in semantic.iter_symbols() {
        let symbol = semantic.symbol(entry.id);
        let listed = entry.symbol;
        assert_eq!(
            (symbol.name, symbol.declaration, symbol.flags),
            (listed.name, listed.declaration, listed.flags)
        );
        let expected = &by_symbol[&entry.id];
        assert_eq!(entry.symbol.declaration, expected[0]);
        let ids = semantic.declaration_ids(entry.id);
        assert_eq!(ids.len(), expected.len());
        assert!(ids.clone().eq(expected.iter().copied()));
        assert!(ids.rev().eq(expected.iter().rev().copied()));
    }
}
