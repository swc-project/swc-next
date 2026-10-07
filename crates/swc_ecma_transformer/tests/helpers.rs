use super::*;
use crate::context::Mutate;
use crate::state::TransformerState;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{NodeKindData, SourceType};
use swc_next_ecma_ast::{TraverseMutCtx, VariableKind};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

fn with_context(source: &str, run: impl FnOnce(&mut Mutation<'_, '_, '_>)) {
    with_source_context(source, SourceType::Module, run);
}

fn with_source_context(
    source: &str,
    source_type: SourceType,
    run: impl FnOnce(&mut Mutation<'_, '_, '_>),
) {
    let allocator = Allocator::new();
    let options = Options {
        source_type,
        ..Options::default()
    };
    let parsed = Parser::init(&allocator, source, options, NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    let mut semantic = analyze(
        &ast,
        AnalyzeOptions {
            features: swc_next_ecma_semantic::SemanticFeatureFlags::GENERATED_NAMES,
            ..AnalyzeOptions::default()
        },
    )
    .semantic;
    let state = TransformerState::new(&mut semantic, Default::default());
    let mut traversal = TraverseMutCtx::new(&mut ast, state);
    let mut ctx = traversal.mutate();
    let program = ctx.ast().root_program();
    ctx.at_node(program.node_id(), run);
}

#[test]
fn helper_registration() {
    with_context("const _set_function_name = 0;", |ctx| {
        let program = ctx.ast().root_program();
        let original_body = program.body(ctx.ast());
        let program_scope = ctx.current_scope();
        let initial_scopes = ctx.semantic().iter_scopes().count();
        let callee = ctx.helper_callee(HelperKind::SetFunctionName, ctx.current_scope());
        let reference = ctx.semantic().reference_of(callee.node_id()).unwrap();
        let symbol = ctx.semantic().reference(reference).symbol.unwrap();
        assert_eq!(
            ctx.ast()
                .get_wtf8(ctx.semantic().symbol(symbol).name)
                .as_bytes(),
            b"_set_function_name2"
        );
        assert_eq!(ctx.semantic().scope_of(symbol), program_scope);
        assert_eq!(ctx.semantic().iter_scopes().count(), initial_scopes);
        assert!(
            ctx.semantic()
                .symbol(symbol)
                .flags
                .contains(swc_next_ecma_semantic::SymbolFlags::IMPORT)
        );
        assert_eq!(program.body(ctx.ast()).len(), original_body.len());
        let scopes = ctx.semantic().iter_scopes().count();
        let symbols = ctx.semantic().iter_symbols().count();
        let references = ctx.semantic().iter_references().count();

        let callee = ctx.helper_callee(HelperKind::SetFunctionName, ctx.current_scope());
        let reference = ctx.semantic().reference_of(callee.node_id()).unwrap();
        assert_eq!(ctx.semantic().reference(reference).symbol, Some(symbol));
        assert_eq!(ctx.semantic().iter_scopes().count(), scopes);
        assert_eq!(ctx.semantic().iter_symbols().count(), symbols);
        assert_eq!(ctx.semantic().iter_references().count(), references + 1);

        let nodes_before_injection = ctx.ast().node_count();
        ctx.inject_helpers(program);
        assert_eq!(ctx.ast().node_count(), nodes_before_injection);
        assert_eq!(program.body(ctx.ast()).len(), original_body.len());
        ctx.flush_statements(program.node_id());
        assert_eq!(program.body(ctx.ast()).len(), original_body.len() + 1);
        let first = program.body(ctx.ast()).get_node(ctx.ast(), 0).unwrap();
        assert!(matches!(
            ctx.ast().kind_data(first.node_id()),
            NodeKindData::ImportDeclaration(_)
        ));
        let import = first.as_import_declaration(ctx.ast()).unwrap();
        assert!(
            import
                .specifiers(ctx.ast())
                .get_node(ctx.ast(), 0)
                .unwrap()
                .as_import_default_specifier(ctx.ast())
                .is_some()
        );
        assert_eq!(
            ctx.ast()
                .get_wtf8(import.source(ctx.ast()).value(ctx.ast()))
                .as_bytes(),
            b"@babel/runtime/helpers/setFunctionName"
        );
        assert_eq!(ctx.semantic().iter_scopes().count(), scopes);
        ctx.inject_helpers(program);
        ctx.flush_statements(program.node_id());
        assert_eq!(program.body(ctx.ast()).len(), original_body.len() + 1);
    });

    with_context("function outer() {}", |ctx| {
        let program = ctx.ast().root_program();
        let function = program
            .body(ctx.ast())
            .get_node(ctx.ast(), 0)
            .unwrap()
            .as_declaration(ctx.ast())
            .unwrap()
            .as_function(ctx.ast())
            .unwrap();
        ctx.at_node(function.node_id(), |ctx| {
            let scope = ctx.current_scope();
            let callee = ctx.helper_callee(HelperKind::SetFunctionName, ctx.current_scope());
            let reference = ctx.semantic().reference_of(callee.node_id()).unwrap();
            let symbol = ctx.semantic().reference(reference).symbol.unwrap();
            assert_eq!(ctx.semantic().scope_of(symbol), ctx.program_scope());
            let statement = Stmt::ExpressionStatement(
                ctx.add_node(scope, |ast| ast.expression_statement(DUMMY_SP, callee)),
            );
            ctx.insert_statement(
                crate::context::StatementList::FunctionBody(function.body(ctx.ast())),
                0,
                statement,
            );
        });
        ctx.inject_helpers(program);
        let references = ctx.semantic().iter_references().count();
        ctx.flush_statements(program.node_id());
        assert_eq!(ctx.semantic().iter_references().count(), references);
        #[cfg(debug_assertions)]
        ctx.semantic().assert_invariants(ctx.ast());
    });
}

#[test]
fn external_helpers() {
    with_context("const _swc_helpers = 0;", |ctx| {
        let program = ctx.ast().root_program();
        let scopes = ctx.semantic().iter_scopes().count();
        let callee = ctx.helper_callee(HelperKind::ToPropertyKey, ctx.current_scope());
        let object = callee
            .as_member_expression(ctx.ast())
            .unwrap()
            .object(ctx.ast());
        let reference = ctx.semantic().reference_of(object.node_id()).unwrap();
        let namespace = ctx.semantic().reference(reference).name;
        let binding = ctx.generated_declaration(namespace);
        let symbol = ctx.semantic().symbol_of(binding.node_id()).unwrap();
        assert_eq!(ctx.ast().get_utf8(namespace), "_swc_helpers2");
        assert_eq!(ctx.semantic().uses(symbol).len(), 1);
        assert_eq!(ctx.semantic().iter_scopes().count(), scopes);

        let callee = ctx.helper_callee(HelperKind::ToPropertyKey, ctx.current_scope());
        let object = callee
            .as_member_expression(ctx.ast())
            .unwrap()
            .object(ctx.ast());
        let reference = ctx.semantic().reference_of(object.node_id()).unwrap();
        assert_eq!(ctx.semantic().reference(reference).symbol, Some(symbol));
        assert_eq!(ctx.semantic().uses(symbol).len(), 2);
        ctx.inject_helpers(program);
        ctx.flush_statements(program.node_id());
        assert_eq!(program.body(ctx.ast()).len(), 2);
        let first = program.body(ctx.ast()).get_node(ctx.ast(), 0).unwrap();
        assert!(matches!(
            ctx.ast().kind_data(first.node_id()),
            NodeKindData::ImportDeclaration(_)
        ));
    });

    with_source_context("", SourceType::Script, |ctx| {
        assert_external_require(
            ctx,
            HelperKind::SetFunctionName,
            "@babel/runtime/helpers/setFunctionName",
        )
    });

    with_source_context("", SourceType::Script, |ctx| {
        assert_external_require(ctx, HelperKind::ToPropertyKey, "@swc/helpers")
    });

    with_source_context("", SourceType::CommonJs, |ctx| {
        assert_external_require(ctx, HelperKind::ToPropertyKey, "@swc/helpers")
    });

    with_source_context("", SourceType::Unambiguous, |ctx| {
        assert_external_require(ctx, HelperKind::ToPropertyKey, "@swc/helpers")
    });
}

fn assert_external_require(ctx: &mut Mutation<'_, '_, '_>, helper: HelperKind, source: &str) {
    let program = ctx.ast().root_program();
    let callee = ctx.helper_callee(helper, ctx.current_scope());
    let statement = Stmt::ExpressionStatement(ctx.add_node(ctx.current_scope(), |ast| {
        ast.expression_statement(DUMMY_SP, callee)
    }));
    ctx.insert_statement(crate::StatementList::Program(program), 0, statement);
    ctx.inject_helpers(program);
    ctx.flush_statements(program.node_id());
    let declaration = program
        .body(ctx.ast())
        .get_node(ctx.ast(), 0)
        .unwrap()
        .as_declaration(ctx.ast())
        .unwrap()
        .as_variable_declaration(ctx.ast())
        .unwrap();
    assert_eq!(declaration.kind(ctx.ast()), VariableKind::Var);
    let declarator = declaration
        .declarators(ctx.ast())
        .get_node(ctx.ast(), 0)
        .unwrap();
    let binding = declarator
        .id(ctx.ast())
        .as_binding_identifier(ctx.ast())
        .unwrap();
    let symbol = ctx.semantic().symbol_of(binding.node_id()).unwrap();
    assert!(
        ctx.semantic()
            .symbol(symbol)
            .flags
            .contains(swc_next_ecma_semantic::SymbolFlags::FUNCTION_SCOPED_VAR)
    );
    assert!(
        !ctx.semantic()
            .symbol(symbol)
            .flags
            .contains(swc_next_ecma_semantic::SymbolFlags::IMPORT)
    );
    let call = declarator
        .init(ctx.ast())
        .unwrap()
        .as_call_expression(ctx.ast())
        .unwrap();
    let callee = call
        .callee(ctx.ast())
        .as_identifier_reference(ctx.ast())
        .unwrap();
    assert_eq!(ctx.ast().get_utf8(callee.name(ctx.ast())), "require");
    let reference = ctx
        .semantic()
        .reference(ctx.semantic().reference_of(callee.node_id()).unwrap());
    assert_eq!(reference.scope, ctx.program_scope());
    assert!(reference.symbol.is_none());
    let import_source = call
        .arguments(ctx.ast())
        .get_node(ctx.ast(), 0)
        .unwrap()
        .as_expr(ctx.ast())
        .unwrap()
        .as_string_literal(ctx.ast())
        .unwrap();
    assert_eq!(
        ctx.ast()
            .get_wtf8(import_source.value(ctx.ast()))
            .as_bytes(),
        source.as_bytes()
    );
    #[cfg(debug_assertions)]
    ctx.semantic().assert_invariants(ctx.ast());
}
