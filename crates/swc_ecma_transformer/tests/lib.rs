use super::*;
use swc_next_allocator::Allocator;
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

fn transform_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: TransformerOptions,
) -> (Ast<'a>, Semantic<'a>) {
    let parsed = Parser::init(allocator, source, Options::default(), NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    let mut semantic = analyze(
        &ast,
        AnalyzeOptions {
            features: options.semantic_features(),
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
        es2015: es2015::ES2015Options {
            arrow_functions: Some(Default::default()),
            ..Default::default()
        },
        es2022: es2022::ES2022Options {
            static_blocks: true,
        },
        ..Default::default()
    }
}

fn assert_reference_scopes(ast: &Ast<'_>, semantic: &Semantic<'_>) {
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
fn evaluation_scopes() {
    let allocator = Allocator::new();
    let sticky = || TransformerOptions {
        es2015: es2015::ES2015Options {
            sticky_regex: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let source = "switch (/a/y) { case 0: let RegExp; }";
    let (ast, semantic) = transform_source(&allocator, source, sticky());
    assert_reference_scopes(&ast, &semantic);

    let (ast, semantic) = transform_source(&allocator, "with (/a/y) { let RegExp; }", sticky());
    assert_reference_scopes(&ast, &semantic);

    let (ast, semantic) = transform_source(&allocator, "function f() { return /a/y; }", sticky());
    assert_reference_scopes(&ast, &semantic);
}

#[test]
fn class_environments() {
    let allocator = Allocator::new();
    let source = "const f = () => class extends B { x = super.foo; };";
    let (ast, semantic) = transform_source(&allocator, source, options());
    let initializer = ast
        .root_program()
        .body(&ast)
        .get_node(&ast, 0)
        .unwrap()
        .as_declaration(&ast)
        .unwrap()
        .as_variable_declaration(&ast)
        .unwrap()
        .declarators(&ast)
        .get_node(&ast, 0)
        .unwrap()
        .init(&ast)
        .unwrap();
    assert!(initializer.as_function(&ast).is_some());
    assert_reference_scopes(&ast, &semantic);

    let source = "class A { field = class { [(() => this)()]() {} } }";
    let (ast, semantic) = transform_source(&allocator, source, options());
    let capture = ast
        .root_program()
        .body(&ast)
        .get_node(&ast, 0)
        .unwrap()
        .as_declaration(&ast)
        .unwrap()
        .as_variable_declaration(&ast)
        .unwrap()
        .declarators(&ast)
        .get_node(&ast, 0)
        .unwrap();
    assert!(capture.init(&ast).is_none());
    assert_reference_scopes(&ast, &semantic);

    let source = "class C extends B { constructor() { super(); const f = () => this; } }";
    let (ast, semantic) = transform_source(&allocator, source, options());
    let capture = semantic
        .iter_symbols()
        .find(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() == b"_this")
        .unwrap();
    assert!(
        semantic
            .uses(capture.id)
            .iter()
            .any(|id| semantic.reference(*id).flags.is_write_only())
    );
    assert_reference_scopes(&ast, &semantic);
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
    let source = "class C extends B { constructor() { const fn = () => this; super(); } }";
    let (ast, semantic) = transform_source(&allocator, source, options());
    let capture = semantic
        .iter_symbols()
        .find(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() == b"_this")
        .unwrap();
    assert!(
        semantic
            .uses(capture.id)
            .iter()
            .any(|id| semantic.reference(*id).flags.is_write_only())
    );
    assert!(
        semantic
            .uses(capture.id)
            .iter()
            .any(|id| semantic.reference(*id).flags.is_read_only())
    );

    let source = "function f() { return () => arguments; }";
    let (ast, semantic) = transform_source(&allocator, source, options());
    let capture = semantic
        .iter_symbols()
        .find(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() == b"_arguments")
        .unwrap();
    assert!(!semantic.uses(capture.id).is_empty());

    let source = "class C { static { let value = 1; (() => value)(); } }";
    let (_, semantic) = transform_source(&allocator, source, options());
    assert!(
        semantic
            .iter_scopes()
            .all(|entry| entry.scope.kind != swc_next_ecma_semantic::scope::ScopeKind::StaticBlock)
    );

    let source = "export default (() => this);";
    let (ast, semantic) = transform_source(&allocator, source, options());
    let swc_next_ecma_semantic::ExportTarget::Expression { node, .. } =
        semantic.module_record().exports()[0].target
    else {
        panic!("default export is an expression");
    };
    assert!(matches!(ast.kind_data(node), NodeKindData::Function(_)));

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
