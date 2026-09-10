use swc_next_allocator::Allocator;
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

#[test]
fn structural_wrappers() {
    let allocator = Allocator::new();
    let parsed = Parser::init(
        &allocator,
        "keep(nested(value)); remove;",
        Options::default(),
        NoTokenParserConfig,
    )
    .parse();
    let mut ast = parsed.ast;
    let program = ast.root_program();
    let retained = program
        .body(&ast)
        .get_node(&ast, 0)
        .unwrap()
        .as_expression_statement(&ast)
        .unwrap()
        .expression(&ast);
    let mut semantic = crate::analyze(
        &ast,
        crate::AnalyzeOptions {
            features: crate::SemanticFeatureFlags::NODE_SCOPES,
            ..Default::default()
        },
    )
    .semantic;
    let statement = program
        .body(&ast)
        .get_node(&ast, 0)
        .unwrap()
        .as_expression_statement(&ast)
        .unwrap();
    let scope = semantic.scope_outside(retained.node_id()).unwrap();
    for _ in 0..64 {
        let start = ast.node_count();
        let expression = statement.expression(&ast);
        let wrapper = ast.parenthesized_expression(swc_next_ecma_ast::DUMMY_SP, expression);
        semantic.register_nodes(&ast, start, scope);
        statement.set_expression(
            &mut ast,
            swc_next_ecma_ast::Expr::ParenthesizedExpression(wrapper),
        );
    }
    while let Some(wrapper) = statement.expression(&ast).as_parenthesized_expression(&ast) {
        let expression = wrapper.expression(&ast);
        statement.set_expression(&mut ast, expression);
    }
    assert_eq!(statement.expression(&ast), retained);
    semantic.assert_invariants(&ast);
}

#[test]
fn scope_merging() {
    use crate::{AnalyzeOptions, SemanticFeatureFlags, analyze};
    use swc_next_ecma_ast::{ClassElement, DUMMY_SP, Expr};

    let allocator = Allocator::new();
    let source =
        "class C { static { eval(''); (class { [key]() { nested; } }); } static { outside; } }";
    let parsed = Parser::init(&allocator, source, Options::default(), NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    let options = AnalyzeOptions {
        features: SemanticFeatureFlags::NODE_SCOPES,
        ..Default::default()
    };
    let mut semantic = analyze(&ast, options).semantic;
    let class = ast
        .root_program()
        .body(&ast)
        .get_node(&ast, 0)
        .unwrap()
        .as_declaration(&ast)
        .unwrap()
        .as_class(&ast)
        .unwrap();
    let body = class.body(&ast);
    let source = body
        .body(&ast)
        .get_node(&ast, 0)
        .unwrap()
        .as_static_block(&ast)
        .unwrap();
    let target = body
        .body(&ast)
        .get_node(&ast, 1)
        .unwrap()
        .as_static_block(&ast)
        .unwrap();
    let source_scope = semantic.scope_at(source.node_id()).unwrap();
    let target_scope = semantic.scope_at(target.node_id()).unwrap();

    // Fresh nodes need not be adjacent to the source nodes in the arena.
    let start = ast.node_count();
    let name = ast.add_utf8("generated");
    let reference = ast.identifier_reference(DUMMY_SP, name);
    let generated = ast.stmt_expression_statement(DUMMY_SP, Expr::IdentifierReference(reference));
    semantic.register_nodes(&ast, start, source_scope);
    semantic.register_reference(&ast, reference, source_scope, None, Default::default());
    let references: Vec<_> = semantic
        .iter_references()
        .map(|entry| (entry.id, entry.reference.scope))
        .collect();
    let nested_scopes: Vec<_> = semantic
        .iter_scopes()
        .filter(|entry| entry.scope.parent == Some(source_scope))
        .map(|entry| entry.id)
        .collect();
    let source_statements: Vec<_> = source
        .body(&ast)
        .iter()
        .map(|slot| ast.get_node_in_sub_range(slot))
        .collect();
    let target_statement = target.body(&ast).get_node(&ast, 0).unwrap();
    let node_count = ast.node_count();

    semantic.merge_scope(source_scope, target_scope, Default::default());
    let statements = ast.add_sub_range(
        source_statements
            .into_iter()
            .chain([generated, target_statement]),
    );
    target.set_body(&mut ast, statements);
    let elements = ast.add_sub_range([ClassElement::StaticBlock(target)]);
    body.set_body(&mut ast, elements);

    assert!(!semantic.scope(source_scope).active);
    assert!(semantic.scope(target_scope).flags.direct_eval);
    assert_eq!(semantic.scope_at(generated.node_id()), Some(target_scope));
    assert_eq!(
        semantic.scope_at(target_statement.node_id()),
        Some(target_scope)
    );
    assert_eq!(ast.node_count(), node_count);
    assert_eq!(
        semantic
            .iter_references()
            .map(|entry| (entry.id, entry.reference.scope))
            .collect::<Vec<_>>(),
        references
            .into_iter()
            .map(|(id, scope)| (
                id,
                if scope == source_scope {
                    target_scope
                } else {
                    scope
                }
            ))
            .collect::<Vec<_>>()
    );
    assert!(
        nested_scopes
            .iter()
            .all(|&id| semantic.scope(id).parent == Some(target_scope))
    );
    assert!(
        nested_scopes
            .iter()
            .all(|&id| semantic.scope(id).hoist_target == target_scope)
    );
    semantic.assert_invariants(&ast);
}
