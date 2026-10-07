use super::*;
use crate::context::Mutate;
use crate::state::TransformerState;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{BindingPattern, DUMMY_SP, VariableKind};
use swc_next_ecma_ast::{Decl, TraverseMutCtx};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SemanticFeatureFlags, SymbolFlags, analyze};

fn with_context(source: &str, run: impl FnOnce(&mut Mutation<'_, '_, '_>)) {
    let allocator = Allocator::new();
    let parsed = Parser::init(&allocator, source, Options::default(), NoTokenParserConfig).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut ast = parsed.ast;
    let options = AnalyzeOptions {
        features: SemanticFeatureFlags::all(),
        build_module_record: true,
        ..AnalyzeOptions::default()
    };
    let mut semantic = analyze(&ast, options).semantic;
    let state = TransformerState::new(&mut semantic, Default::default());
    let mut traversal = TraverseMutCtx::new(&mut ast, state);
    let mut ctx = traversal.mutate();
    let program = ctx.ast().root_program();
    ctx.at_node(program.node_id(), run);
    // Check the mutation itself, without Mutation::finish cleanup.
    #[cfg(debug_assertions)]
    ctx.semantic().assert_invariants(ctx.ast());
    let rebuilt = analyze(ctx.ast(), options).semantic;
    assert_eq!(
        ctx.semantic().iter_references().count(),
        rebuilt.iter_references().count()
    );
    for entry in ctx.semantic().iter_references() {
        let reference = entry.reference;
        let expected = rebuilt.reference(rebuilt.reference_of(reference.node).unwrap());
        assert_eq!(reference.flags, expected.flags);
        if let Some(symbol) = reference.symbol {
            let declaration = ctx
                .semantic()
                .declaration(ctx.semantic().symbol(symbol).declaration)
                .node;
            assert!(
                rebuilt
                    .declaration_ids(expected.symbol.unwrap())
                    .any(|id| rebuilt.declaration(id).node == declaration)
            );
        } else {
            assert_eq!(expected.symbol, None);
        }
    }
}

fn blocks(ctx: &Mutation<'_, '_, '_>) -> (BlockStatement, BlockStatement) {
    let body = ctx.ast().root_program().body(ctx.ast());
    let first = body
        .get_node(ctx.ast(), 0)
        .unwrap()
        .as_block_statement(ctx.ast())
        .unwrap();
    let second = body
        .get_node(ctx.ast(), 1)
        .unwrap()
        .as_block_statement(ctx.ast())
        .unwrap();
    (first, second)
}

fn first_statement(ctx: &Mutation<'_, '_, '_>) -> Stmt {
    ctx.ast()
        .root_program()
        .body(ctx.ast())
        .get_node(ctx.ast(), 0)
        .unwrap()
}

fn replace_expression(ctx: &mut Mutation<'_, '_, '_>, parent: NodeId, old: Expr) -> Expr {
    ctx.at_node(old.node_id(), |ctx| {
        assert_eq!(ctx.parent(), Some(parent));
        let scope = ctx.semantic().scope_outside(old.node_id()).unwrap();
        let replacement = Expr::NumericLiteral(
            ctx.add_node(scope, |ast| ast.numeric_literal(DUMMY_SP, 1.0, None)),
        );
        ctx.replace_current_expr(replacement);
        assert_eq!(ctx.semantic().reference_of(old.node_id()), None);
        replacement
    })
}

fn first_class_body(ctx: &Mutation<'_, '_, '_>) -> ClassBody {
    first_statement(ctx)
        .as_declaration(ctx.ast())
        .unwrap()
        .as_class(ctx.ast())
        .unwrap()
        .body(ctx.ast())
}

fn rebuild_class_element(ctx: &mut Mutation<'_, '_, '_>, element: ClassElement) -> ClassElement {
    let scope = ctx.semantic().scope_at(element.node_id()).unwrap();
    ctx.add_node(scope, |ast| match ast.kind_data(element.node_id()) {
        NodeKindData::PropertyDefinition(node) => {
            ClassElement::PropertyDefinition(ast.property_definition(
                DUMMY_SP,
                node.decorators(ast),
                node.key(ast),
                node.type_annotation(ast),
                node.value(ast),
                node.computed(ast),
                node.r#static(ast),
                node.accessor(ast),
                node.declare(ast),
                node.r#override(ast),
                node.optional(ast),
                node.definite(ast),
                node.readonly(ast),
                node.r#abstract(ast),
                node.accessibility(ast),
            ))
        }
        NodeKindData::MethodDefinition(node) => {
            ClassElement::MethodDefinition(ast.method_definition(
                DUMMY_SP,
                node.decorators(ast),
                node.key(ast),
                node.value(ast),
                node.kind(ast),
                node.computed(ast),
                node.r#static(ast),
                node.r#override(ast),
                node.optional(ast),
                node.r#abstract(ast),
                node.accessibility(ast),
            ))
        }
        _ => unreachable!(),
    })
}

#[test]
fn class_elements() {
    with_context(
        "class C extends Base { static {} field = () => value; constructor() { super(); } static {} }",
        |ctx| {
            let body = first_class_body(ctx);
            let original = body.body(ctx.ast());
            let property = original
                .get_node(ctx.ast(), 1)
                .unwrap()
                .as_property_definition(ctx.ast())
                .unwrap();
            let arrow = property
                .value(ctx.ast())
                .unwrap()
                .as_arrow_function_expression(ctx.ast())
                .unwrap();
            let method = original
                .get_node(ctx.ast(), 2)
                .unwrap()
                .as_method_definition(ctx.ast())
                .unwrap();
            let function = method.value(ctx.ast());
            let scope = ctx.semantic().scope_at(function.node_id());
            ctx.filter_map_class_elements(body, |ctx, element| {
                if element.as_static_block(ctx.ast()).is_some() {
                    ClassElementAction::Delete
                } else {
                    ClassElementAction::Update(rebuild_class_element(ctx, element))
                }
            });
            let elements = body.body(ctx.ast());
            let replacement = elements
                .get_node(ctx.ast(), 0)
                .unwrap()
                .as_property_definition(ctx.ast())
                .unwrap();
            let new_method = elements
                .get_node(ctx.ast(), 1)
                .unwrap()
                .as_method_definition(ctx.ast())
                .unwrap();
            assert_eq!(elements.len(), 2);
            assert_eq!(
                elements.get_node(ctx.ast(), 0),
                original.get_node(ctx.ast(), 0)
            );
            assert_ne!(replacement.node_id(), property.node_id());
            assert_eq!(replacement.value(ctx.ast()), property.value(ctx.ast()));
            assert_eq!(
                ctx.semantic()
                    .class()
                    .property_initializer_owner(arrow.node_id()),
                Some(replacement.node_id())
            );
            assert_ne!(new_method.node_id(), method.node_id());
            assert_eq!(new_method.value(ctx.ast()), function);
            assert_eq!(ctx.semantic().scope_at(function.node_id()), scope);
            assert!(
                ctx.semantic()
                    .class()
                    .is_derived_constructor(function.node_id())
            );
        },
    );
}

#[test]
fn expression_edges() {
    with_context("value;", |ctx| {
        let statement = first_statement(ctx)
            .as_expression_statement(ctx.ast())
            .unwrap();
        let old = statement.expression(ctx.ast());
        let replacement = replace_expression(ctx, statement.node_id(), old);
        assert_eq!(
            statement.expression(ctx.ast()).node_id(),
            replacement.node_id()
        );
    });

    with_context("let result = value;", |ctx| {
        let declaration = first_statement(ctx)
            .as_declaration(ctx.ast())
            .unwrap()
            .as_variable_declaration(ctx.ast())
            .unwrap();
        let declarator = declaration
            .declarators(ctx.ast())
            .get_node(ctx.ast(), 0)
            .unwrap();
        let old = declarator.init(ctx.ast()).unwrap();
        let replacement = replace_expression(ctx, declarator.node_id(), old);
        assert_eq!(
            declarator.init(ctx.ast()).unwrap().node_id(),
            replacement.node_id()
        );
    });

    with_context("call(value);", |ctx| {
        let call = first_statement(ctx)
            .as_expression_statement(ctx.ast())
            .unwrap()
            .expression(ctx.ast())
            .as_call_expression(ctx.ast())
            .unwrap();
        let callee = call.callee(ctx.ast()).node_id();
        let callee_reference = ctx.semantic().reference_of(callee);
        let old = call
            .arguments(ctx.ast())
            .get_node(ctx.ast(), 0)
            .unwrap()
            .as_expr(ctx.ast())
            .unwrap();
        let replacement = replace_expression(ctx, call.node_id(), old);
        assert_eq!(
            call.arguments(ctx.ast())
                .get_node(ctx.ast(), 0)
                .unwrap()
                .node_id(),
            replacement.node_id()
        );
        assert_eq!(ctx.semantic().reference_of(callee), callee_reference);
    });

    with_context("[, value];", |ctx| {
        let array = first_statement(ctx)
            .as_expression_statement(ctx.ast())
            .unwrap()
            .expression(ctx.ast())
            .as_array_expression(ctx.ast())
            .unwrap();
        let old = array
            .elements(ctx.ast())
            .get_node(ctx.ast(), 1)
            .unwrap()
            .unwrap()
            .as_expr(ctx.ast())
            .unwrap();
        let replacement = replace_expression(ctx, array.node_id(), old);
        assert!(
            array
                .elements(ctx.ast())
                .get_node(ctx.ast(), 0)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            array
                .elements(ctx.ast())
                .get_node(ctx.ast(), 1)
                .unwrap()
                .unwrap()
                .node_id(),
            replacement.node_id()
        );
    });

    with_context("let value; value + value;", |ctx| {
        let statement = ctx
            .ast()
            .root_program()
            .body(ctx.ast())
            .get_node(ctx.ast(), 1)
            .unwrap();
        let binary = statement
            .as_expression_statement(ctx.ast())
            .unwrap()
            .expression(ctx.ast())
            .as_binary_expression(ctx.ast())
            .unwrap();
        let old = binary.left(ctx.ast());
        let remaining = ctx
            .semantic()
            .reference_of(binary.right(ctx.ast()).node_id())
            .unwrap();
        let symbol = ctx.semantic().reference(remaining).symbol.unwrap();
        replace_expression(ctx, binary.node_id(), old);
        assert_eq!(ctx.semantic().uses(symbol), &[remaining]);
        assert!(matches!(
            ctx.ast().kind_data(old.node_id()),
            NodeKindData::IdentifierReference(_)
        ));
    });

    with_context("export default value", |ctx| {
        let NodeKindData::ExportDefaultDeclaration(export) =
            ctx.ast().kind_data(first_statement(ctx).node_id())
        else {
            unreachable!()
        };
        let expression = export.declaration(ctx.ast()).as_expr(ctx.ast()).unwrap();
        let replacement = replace_expression(ctx, export.node_id(), expression);
        assert!(matches!(ctx.semantic().module_record().exports()[0].target,
            swc_next_ecma_semantic::ExportTarget::Expression { node, .. } if node == replacement.node_id()));
    });
}

#[test]
fn statement_edits() {
    with_context("value;", |ctx| {
        let program = ctx.ast().root_program();
        let scope = ctx.program_scope();
        let reference = ctx.semantic().iter_references().next().unwrap().id;
        let binding = ctx.create_named_binding("value", scope, SymbolFlags::BLOCK_SCOPED_VAR);
        let symbol = ctx.semantic().symbol_of(binding.node_id()).unwrap();
        assert_eq!(ctx.semantic().reference(reference).symbol, Some(symbol));
        let declaration = ctx.add_node(ctx.current_scope(), |ast| {
            ast.variable_declarator(
                DUMMY_SP,
                BindingPattern::BindingIdentifier(binding),
                None,
                None,
                false,
            )
        });
        let declarations = ctx.add_sub_range([declaration]);
        let statement = Stmt::Declaration(Decl::VariableDeclaration(
            ctx.add_node(ctx.current_scope(), |ast| {
                ast.variable_declaration(DUMMY_SP, VariableKind::Let, declarations, false)
            }),
        ));
        ctx.insert_statement(StatementList::Program(program), 0, statement);
        assert_eq!(program.body(ctx.ast()).len(), 2);
        assert_eq!(ctx.semantic().uses(symbol), &[reference]);
    });
}

#[test]
fn deferred_statements() {
    with_context("first; second;", |ctx| {
        let program = ctx.ast().root_program();
        let first = program.body(ctx.ast()).get_node(ctx.ast(), 0).unwrap();
        let second = program.body(ctx.ast()).get_node(ctx.ast(), 1).unwrap();
        let one = Expr::NumericLiteral(ctx.add_node(ctx.program_scope(), |ast| {
            ast.numeric_literal(DUMMY_SP, 1.0, None)
        }));
        let one = Stmt::ExpressionStatement(ctx.add_node(ctx.program_scope(), |ast| {
            ast.expression_statement(DUMMY_SP, one)
        }));
        let two = Expr::NumericLiteral(ctx.add_node(ctx.program_scope(), |ast| {
            ast.numeric_literal(DUMMY_SP, 2.0, None)
        }));
        let two = Stmt::ExpressionStatement(ctx.add_node(ctx.program_scope(), |ast| {
            ast.expression_statement(DUMMY_SP, two)
        }));
        ctx.inject_statement_after(program.node_id(), second.node_id(), two);
        ctx.inject_statement_after(program.node_id(), first.node_id(), one);
        assert_eq!(program.body(ctx.ast()).len(), 2);
        assert_eq!(
            ctx.semantic().scope_at(one.node_id()),
            Some(ctx.program_scope())
        );
        let references = ctx.semantic().iter_references().count();
        ctx.flush_statements(program.node_id());
        let statements: Vec<_> = program
            .body(ctx.ast())
            .iter()
            .map(|slot| ctx.ast().get_node_in_sub_range(slot))
            .collect();
        assert_eq!(statements, vec![first, one, second, two]);
        assert_eq!(ctx.semantic().iter_references().count(), references);
    });
}

#[test]
fn rejected_edits() {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[cfg(debug_assertions)]
    with_context("class C { field = value; }", |ctx| {
        let body = first_class_body(ctx);
        let original = body.body(ctx.ast()).get_node(ctx.ast(), 0).unwrap();
        assert!(
            catch_unwind(AssertUnwindSafe(
                || ctx.filter_map_class_elements(body, |_, _| ClassElementAction::Delete)
            ))
            .is_err()
        );
        assert_eq!(body.body(ctx.ast()).get_node(ctx.ast(), 0), Some(original));
    });

    #[cfg(debug_assertions)]
    with_context("class C { method() { return value; } }", |ctx| {
        let body = first_class_body(ctx);
        let original = body.body(ctx.ast()).get_node(ctx.ast(), 0).unwrap();
        assert!(
            catch_unwind(AssertUnwindSafe(
                || ctx.filter_map_class_elements(body, |_, _| ClassElementAction::Delete)
            ))
            .is_err()
        );
        assert_eq!(body.body(ctx.ast()).get_node(ctx.ast(), 0), Some(original));
    });

    with_context("left + right;", |ctx| {
        let statement = first_statement(ctx)
            .as_expression_statement(ctx.ast())
            .unwrap();
        let binary = statement
            .expression(ctx.ast())
            .as_binary_expression(ctx.ast())
            .unwrap();
        let left = binary.left(ctx.ast());
        let right = binary.right(ctx.ast());
        let reference = ctx.semantic().reference_of(right.node_id());
        ctx.at_node(binary.node_id(), |ctx| {
            assert!(catch_unwind(AssertUnwindSafe(|| ctx.wrap_current_expr(left))).is_err());
            assert_eq!(statement.expression(ctx.ast()).node_id(), binary.node_id());
            assert!(catch_unwind(AssertUnwindSafe(|| ctx.replace_current_expr(left))).is_err());
            assert_eq!(statement.expression(ctx.ast()).node_id(), binary.node_id());
            assert_eq!(ctx.semantic().reference_of(right.node_id()), reference);
        });
    });

    with_context("{ value; } {}", |ctx| {
        let (source, target) = blocks(ctx);
        let statement = source.body(ctx.ast()).get_node(ctx.ast(), 0).unwrap();
        let reference = ctx.semantic().iter_references().next().unwrap().id;
        let scope = ctx.semantic().reference(reference).scope;
        assert!(
            catch_unwind(AssertUnwindSafe(|| ctx.insert_statement(
                StatementList::Block(target),
                0,
                statement
            )))
            .is_err()
        );
        assert!(
            catch_unwind(AssertUnwindSafe(
                || ctx.inject_statement(target.node_id(), statement)
            ))
            .is_err()
        );
        assert!(target.body(ctx.ast()).is_empty());
        assert_eq!(source.body(ctx.ast()).len(), 1);
        assert_eq!(ctx.semantic().reference(reference).scope, scope);
        assert!(ctx.state.statement_injector.is_empty());
    });

    with_context("{ let value; }", |ctx| {
        let block = first_statement(ctx).as_block_statement(ctx.ast()).unwrap();
        let scope = ctx.scope_for_node(block.node_id()).unwrap();
        assert!(
            catch_unwind(AssertUnwindSafe(|| ctx.merge_scope(
                block.node_id(),
                ctx.program_scope(),
                Default::default()
            )))
            .is_err()
        );
        assert!(ctx.semantic().scope(scope).active);
        assert!(ctx.semantic().binding(scope, b"value").is_some());
    });

    with_context("value;", |ctx| {
        let statement = first_statement(ctx);
        let reference = ctx
            .semantic()
            .iter_references()
            .next()
            .unwrap()
            .reference
            .node;
        let program = ctx.ast().root_program();
        assert!(
            catch_unwind(AssertUnwindSafe(|| ctx.inject_statement_after(
                program.node_id(),
                reference,
                statement
            )))
            .is_err()
        );
        assert_eq!(program.body(ctx.ast()).len(), 1);
        assert!(ctx.state.statement_injector.is_empty());
    });
}

#[test]
fn retained_subtrees() {
    with_context("value;", |ctx| {
        let statement = first_statement(ctx)
            .as_expression_statement(ctx.ast())
            .unwrap();
        let original = statement.expression(ctx.ast());
        let reference = ctx.semantic().reference_of(original.node_id()).unwrap();
        let scope = ctx.semantic().scope_outside(original.node_id()).unwrap();
        let mut expressions = vec![original];
        let wrapper = ctx.add_node(scope, |ast| {
            expressions.push(ast.expr_numeric_literal(DUMMY_SP, 1.0, None));
            let expressions = ast.add_sub_range(expressions);
            ast.expr_sequence_expression(DUMMY_SP, expressions)
        });
        ctx.at_node(original.node_id(), |ctx| ctx.wrap_current_expr(wrapper));
        let sequence = wrapper.as_sequence_expression(ctx.ast()).unwrap();
        let created = sequence
            .expressions(ctx.ast())
            .get_node(ctx.ast(), 1)
            .unwrap();
        assert_eq!(ctx.semantic().scope_at(created.node_id()), Some(scope));
        assert_eq!(ctx.semantic().scope_at(wrapper.node_id()), Some(scope));
        assert_eq!(
            ctx.semantic().reference_of(original.node_id()),
            Some(reference)
        );
    });

    with_context("left + right;", |ctx| {
        let statement = first_statement(ctx)
            .as_expression_statement(ctx.ast())
            .unwrap();
        let original = statement.expression(ctx.ast());
        let references: Vec<_> = ctx
            .semantic()
            .iter_references()
            .map(|entry| entry.id)
            .collect();
        let mut expression = original;
        for _ in 0..64 {
            let wrapper = ctx.at_node(expression.node_id(), |ctx| {
                let wrapper =
                    Expr::ParenthesizedExpression(ctx.add_node(ctx.current_scope(), |ast| {
                        ast.parenthesized_expression(DUMMY_SP, expression)
                    }));
                ctx.wrap_current_expr(wrapper);
                wrapper
            });
            expression = wrapper;
        }
        assert_eq!(
            ctx.semantic().scope_at(original.node_id()),
            Some(ctx.current_scope())
        );
        let node_count = ctx.ast().node_count();
        ctx.normalize();
        assert_eq!(ctx.ast().node_count(), node_count);
        assert_eq!(
            statement.expression(ctx.ast()).node_id(),
            original.node_id()
        );
        assert_eq!(
            ctx.semantic()
                .iter_references()
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            references
        );
    });
}
