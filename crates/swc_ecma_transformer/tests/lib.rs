use crate::normalize::Normalizer;

use super::*;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::NodeKindData;
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

pub(super) fn transform_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: TransformerOptions,
) -> (Ast<'a>, Semantic<'a>) {
    transform_with_parser_options(allocator, source, options, Options::default())
}

fn transform_with_parser_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: TransformerOptions,
    parser_options: Options,
) -> (Ast<'a>, Semantic<'a>) {
    let parser_options = Options {
        preserve_parens: true,
        ..parser_options
    };
    let parsed = Parser::init(allocator, source, parser_options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    Normalizer::normalize(ast.root_program(), &mut ast);
    let mut semantic = analyze(
        &ast,
        AnalyzeOptions {
            build_module_record: true,
            ..AnalyzeOptions::default()
        },
    )
    .semantic;
    Transformer::transform(&mut ast, &mut semantic, options);
    (ast, semantic)
}

fn options() -> TransformerOptions {
    TransformerOptions {
        es2022: es2022::ES2022Options {
            static_blocks: true,
        },
        ..Default::default()
    }
}

pub(super) fn assert_reference_scopes(ast: &Ast<'_>, semantic: &Semantic<'_>) {
    let fresh = analyze(ast, AnalyzeOptions::default()).semantic;
    for entry in semantic.iter_references() {
        let fresh_reference = fresh.reference(fresh.reference_of(entry.reference.node).unwrap());
        assert_eq!(
            semantic.scope(entry.reference.scope).node,
            fresh.scope(fresh_reference.scope).node
        );
        assert_eq!(
            entry.reference.symbol.map(|symbol| semantic
                .declaration(semantic.symbol(symbol).declaration)
                .node),
            fresh_reference
                .symbol
                .map(|symbol| fresh.declaration(fresh.symbol(symbol).declaration).node)
        );
        assert_eq!(
            entry.reference.flags.is_dynamic(),
            fresh_reference.flags.is_dynamic()
        );
    }
}

#[test]
fn inferred_names() {
    let allocator = Allocator::new();
    let closure = || TransformerOptions {
        es2022: es2022::ES2022Options {
            static_blocks: true,
        },
        proposal: proposal::ProposalOptions {
            prevent_extensions: true,
        },
        ..Default::default()
    };
    let (ast, semantic) = transform_source(
        &allocator,
        "let C; ({ C = class { static { 0; } } } = {});",
        closure(),
    );
    assert_eq!(inferred_name_argument(&ast), b"C");
    assert_reference_scopes(&ast, &semantic);

    let (ast, semantic) = transform_source(
        &allocator,
        "let item; [item = class { static { 0; } }] = [];",
        closure(),
    );
    assert_eq!(inferred_name_argument(&ast), b"item");
    assert_reference_scopes(&ast, &semantic);
}

fn inferred_name_argument<'a>(ast: &'a Ast<'_>) -> &'a [u8] {
    (0..ast.node_count())
        .map(swc_next_ecma_ast::NodeId::from_usize)
        .find_map(|node| {
            let NodeKindData::CallExpression(call) = ast.kind_data(node) else {
                return None;
            };
            let callee = call.callee(ast).as_identifier_reference(ast)?;
            if ast.get_utf8(callee.name(ast)) != "_set_function_name" {
                return None;
            }
            let name = call
                .arguments(ast)
                .get_node(ast, 1)?
                .as_expr(ast)?
                .as_string_literal(ast)?;
            Some(ast.get_wtf8(name.value(ast)).as_bytes())
        })
        .expect("class name is restored")
}

#[test]
fn transformed_semantics() {
    let allocator = Allocator::new();
    let source = "class C { static { let value = 1; (() => value)(); } }";
    let (_, semantic) = transform_source(&allocator, source, options());
    assert!(
        semantic
            .iter_scopes()
            .all(|entry| entry.scope.kind != swc_next_ecma_semantic::scope::ScopeKind::StaticBlock)
    );

    let closure_options = || TransformerOptions {
        proposal: proposal::ProposalOptions {
            prevent_extensions: true,
        },
        ..options()
    };
    let source = "class C { static { eval('x'); } }";
    let (_, semantic) = transform_source(&allocator, source, closure_options());
    assert!(semantic.iter_scopes().any(|entry| entry.scope.kind
        == swc_next_ecma_semantic::scope::ScopeKind::Function
        && entry.scope.flags.contains_direct_eval()));

    let source = "class C extends B { static { super.x; } static value = 1; static { super.y; } static { let local = 0; (() => local)(); } }";
    let (ast, semantic) = transform_source(&allocator, source, closure_options());
    assert_reference_scopes(&ast, &semantic);
}
