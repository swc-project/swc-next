use swc_next_ecma_ast::{
    AwaitExpression, BinaryExpression, BreakStatement, ContinueStatement, LabelIdentifier,
    LabeledStatement, MetaProperty, NumericLiteral, ObjectExpression, RegExpLiteral, StringLiteral,
    YieldExpression,
};

use super::*;

pub(super) fn check_program(checker: &mut Checker<'_, '_>) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    checks.check_unresolved_exports(context.ast, &context.state.symbols);
}

pub(super) fn check_string_literal(checker: &mut Checker<'_, '_>, literal: StringLiteral) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if context.state.in_type_position() || !context.state.scope.is_strict() {
        return;
    }
    let span = context.ast.span(literal.node_id());
    if has_octal_escape(&context.ast.source().as_bytes()[span.start as usize..span.end as usize]) {
        checks.report(
            span,
            "Octal escape sequences are not allowed in strict mode",
            ReportOptions {
                help: Some(Cow::Borrowed(
                    "Use \\xHH (hex) or \\uHHHH (unicode) escape instead",
                )),
                ..ReportOptions::default()
            },
        );
    }
}

pub(super) fn check_numeric_literal(checker: &mut Checker<'_, '_>, literal: NumericLiteral) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if context.state.in_type_position() || !context.state.scope.is_strict() {
        return;
    }
    let span = context.ast.span(literal.node_id());
    let raw = &context.ast.source().as_bytes()[span.start as usize..span.end as usize];
    if !is_legacy_numeric_literal(raw) {
        return;
    }
    let is_octal = raw.iter().all(|byte| !matches!(byte, b'8' | b'9'));
    let (message, help) = if is_octal {
        (
            "Octal literals are not allowed in strict mode",
            "Use the 0o prefix for octal literals (e.g., 0o77), or a decimal equivalent",
        )
    } else {
        (
            "Decimals with leading zeros are not allowed in strict mode",
            "Remove the leading zero, or use 0o for octal, 0x for hex, or 0b for binary notation",
        )
    };
    checks.report(
        span,
        message,
        ReportOptions {
            help: Some(Cow::Borrowed(help)),
            ..ReportOptions::default()
        },
    );
}

pub(super) fn check_reg_exp_literal(checker: &mut Checker<'_, '_>, literal: RegExpLiteral) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let pattern = context.ast.get_utf8(literal.pattern(context.ast));
    let flags = context.ast.get_utf8(literal.flags(context.ast));
    let allocator = Allocator::new();
    if swc_next_regular_expression::LiteralParser::new(
        &allocator,
        pattern,
        Some(flags),
        swc_next_regular_expression::Options::default(),
    )
    .parse()
    .is_err()
    {
        checks.report(
            context.ast.span(literal.node_id()),
            "Invalid regular expression literal",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_identifier_reference(
    checker: &mut Checker<'_, '_>,
    identifier: IdentifierReference,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if context.state.in_type_position() {
        return;
    }
    let node_id = identifier.node_id();
    let name = context
        .ast
        .get_utf8(identifier.name(context.ast))
        .as_bytes();
    checks.check_strict_reserved(name, node_id, context, "an identifier");
    if name == b"arguments" && !is_arguments_available(context) {
        checks.report(
            context.ast.span(node_id),
            "'arguments' is not allowed in class field initializers or static blocks",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_binding_identifier(
    checker: &mut Checker<'_, '_>,
    identifier: BindingIdentifier,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let node_id = identifier.node_id();
    let name = context
        .ast
        .get_utf8(identifier.name(context.ast))
        .as_bytes();
    let flags = context.state.symbols.pending.flags;
    if context.state.in_type_position() && !flags.contains(SymbolFlags::TYPE_PARAMETER) {
        return;
    }
    if context.state.scope.is_strict() && !flags.contains(SymbolFlags::AMBIENT) {
        checks.check_strict_reserved(name, node_id, context, "a binding identifier");
        if is_eval_or_arguments(name) {
            let message = checks.fmt(format_args!(
                "'{}' is not allowed as a binding identifier in strict mode",
                String::from_utf8_lossy(name)
            ));
            checks.report(context.ast.span(node_id), message, ReportOptions::default());
        }
    } else if flags.contains(SymbolFlags::BLOCK_SCOPED_VAR) && name == b"let" {
        checks.report(
            context.ast.span(node_id),
            "'let' is not allowed as a variable name in a lexical declaration",
            ReportOptions::default(),
        );
    }
    let existing = context
        .state
        .symbols
        .binding(context.state.symbols.pending.scope, name);
    checks.check_redeclaration(identifier, node_id, name, flags, context, existing);
    if context.state.symbols.export_state.named().is_some() {
        let skip = existing.is_some() && context.ast.is_ts();
        if !skip {
            checks.record_exported_name(name, node_id, context);
        }
    }
}

pub(super) fn check_label_identifier(checker: &mut Checker<'_, '_>, identifier: LabelIdentifier) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let name = context
        .ast
        .get_utf8(identifier.name(context.ast))
        .as_bytes();
    checks.check_strict_reserved(name, identifier.node_id(), context, "a label");
}

pub(super) fn check_binary_expression(checker: &mut Checker<'_, '_>, expression: BinaryExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let left = expression.left(context.ast).node_id();
    if expression.operator(context.ast) == BinaryOperator::In
        && let NodeKindData::PrivateIdentifier(identifier) = context.ast.kind_data(left)
    {
        let name = context
            .ast
            .get_utf8(identifier.name(context.ast))
            .as_bytes();
        if !is_private_name_declared(context, name) {
            let message = checks.fmt(format_args!(
                "Private field '#{}' must be declared in an enclosing class",
                String::from_utf8_lossy(name)
            ));
            checks.report(context.ast.span(left), message, ReportOptions::default());
        }
    }
}

pub(super) fn check_unary_expression(checker: &mut Checker<'_, '_>, expression: UnaryExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if expression.operator(context.ast) != UnaryOperator::Delete {
        return;
    }
    let mut target = unwrap_parentheses(context.ast, expression.argument(context.ast).node_id());
    if let NodeKindData::ChainExpression(chain) = context.ast.kind_data(target) {
        target = unwrap_parentheses(context.ast, chain.expression(context.ast).node_id());
    }
    match context.ast.kind_data(target) {
        NodeKindData::IdentifierReference(_) if context.state.scope.is_strict() => {
            checks.report(
                context.ast.span(expression.node_id()),
                "Deleting a variable in strict mode is not allowed",
                ReportOptions::default(),
            );
        }
        NodeKindData::MemberExpression(member)
            if !member.computed(context.ast)
                && context
                    .ast
                    .node_kind(member.property(context.ast).node_id())
                    == NodeKind::PrivateIdentifier =>
        {
            checks.report(
                context.ast.span(expression.node_id()),
                "Private fields cannot be deleted",
                ReportOptions::default(),
            );
        }
        _ => {}
    }
}

pub(super) fn check_update_expression(checker: &mut Checker<'_, '_>, expression: UpdateExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if context.state.scope.is_strict()
        && is_eval_or_arguments_reference(context.ast, expression.argument(context.ast).node_id())
    {
        checks.report(
            context.ast.span(expression.node_id()),
            "Cannot assign to 'eval' or 'arguments' in strict mode",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_assignment_expression(
    checker: &mut Checker<'_, '_>,
    expression: AssignmentExpression,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if context.state.scope.is_strict() {
        checks.check_assign_target_eval_arguments(expression.left(context.ast).node_id(), context);
    }
}

pub(super) fn check_variable_declaration(
    checker: &mut Checker<'_, '_>,
    declaration: VariableDeclaration,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let kind = declaration.kind(context.ast);
    if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing)
        && context.ast.source_type() == swc_next_ecma_ast::SourceType::Script
        && is_at_program_level(context)
    {
        let message = checks.fmt(format_args!(
            "'{}' is not allowed at the top level of a script",
            kind.as_str()
        ));
        checks.report(
            context.ast.span(declaration.node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_switch_statement(checker: &mut Checker<'_, '_>, statement: SwitchStatement) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let mut first_default = None;
    for case in statement.cases(context.ast).iter() {
        let case = context.ast.get_node_in_sub_range(case);
        if case.test(context.ast).is_some() {
            continue;
        }
        if let Some(first) = first_default {
            checks.report(
                context.ast.span(case.node_id()),
                "A switch statement can only have one default clause",
                ReportOptions {
                    labels: checks.labels(&[
                        checks.label(context.ast.span(first), "first default defined here")
                    ]),
                    ..ReportOptions::default()
                },
            );
        } else {
            first_default = Some(case.node_id());
        }
    }
}

pub(super) fn check_for_in_statement(checker: &mut Checker<'_, '_>, statement: ForInStatement) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let left = statement.left(context.ast).node_id();
    checks.check_for_in_of_initializer(context, left, true);
    if context.state.scope.is_strict() {
        checks.check_assign_target_eval_arguments(left, context);
    }
}

pub(super) fn check_for_of_statement(checker: &mut Checker<'_, '_>, statement: ForOfStatement) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if statement.r#await(context.ast) && !context.ast.is_ts() && is_inside_static_block(context) {
        checks.report(
            context.ast.span(statement.node_id()),
            "Cannot use 'for await' in class static initialization block",
            ReportOptions::default(),
        );
    }
    let left = statement.left(context.ast).node_id();
    checks.check_for_in_of_initializer(context, left, false);
    if context.state.scope.is_strict() {
        checks.check_assign_target_eval_arguments(left, context);
    }
}

pub(super) fn check_break_statement(checker: &mut Checker<'_, '_>, statement: BreakStatement) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if let Some(label) = statement.label(context.ast) {
        let name = context.ast.get_utf8(label.name(context.ast)).as_bytes();
        let result = find_label(context, name);
        if result != LabelSearch::Found {
            let message = match result {
                LabelSearch::NotFound => checks.fmt(format_args!(
                    "Use of undefined label '{}'",
                    String::from_utf8_lossy(name)
                )),
                LabelSearch::CrossedBoundary => checks.fmt(format_args!(
                    "Cannot break to label '{}' across function boundaries",
                    String::from_utf8_lossy(name)
                )),
                LabelSearch::Found => unreachable!(),
            };
            checks.report(
                context.ast.span(statement.node_id()),
                message,
                ReportOptions::default(),
            );
        }
    } else if !is_inside_breakable(context) {
        checks.report(
            context.ast.span(statement.node_id()),
            "Illegal break statement",
            ReportOptions {
                help: Some(Cow::Borrowed(
                    "A 'break' statement can only be used within an enclosing iteration or switch statement",
                )),
                ..ReportOptions::default()
            },
        );
    }
}

pub(super) fn check_continue_statement(
    checker: &mut Checker<'_, '_>,
    statement: ContinueStatement,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if let Some(label) = statement.label(context.ast) {
        let name = context.ast.get_utf8(label.name(context.ast)).as_bytes();
        let result = find_label_for_continue(context, name);
        if result != ContinueLabelSearch::Found {
            let (message, help) = match result {
                ContinueLabelSearch::NotFound => (
                    checks.fmt(format_args!(
                        "Use of undefined label '{}'",
                        String::from_utf8_lossy(name)
                    )),
                    None,
                ),
                ContinueLabelSearch::CrossedBoundary => (
                    checks.fmt(format_args!(
                        "Cannot continue to label '{}' across function boundaries",
                        String::from_utf8_lossy(name)
                    )),
                    None,
                ),
                ContinueLabelSearch::NotIteration => (
                    checks.fmt(format_args!(
                        "Label '{}' does not denote an iteration statement",
                        String::from_utf8_lossy(name)
                    )),
                    Some(Cow::Borrowed(
                        "A 'continue' statement can only jump to a label of an enclosing 'for', 'while', 'do-while', 'for-in', or 'for-of' statement",
                    )),
                ),
                ContinueLabelSearch::Found => unreachable!(),
            };
            checks.report(
                context.ast.span(statement.node_id()),
                message,
                ReportOptions {
                    help,
                    ..ReportOptions::default()
                },
            );
        }
    } else if !is_inside_iteration(context) {
        checks.report(
            context.ast.span(statement.node_id()),
            "Illegal continue statement",
            ReportOptions {
                help: Some(Cow::Borrowed(
                    "A 'continue' statement can only be used within an enclosing iteration statement",
                )),
                ..ReportOptions::default()
            },
        );
    }
}

pub(super) fn check_labeled_statement(checker: &mut Checker<'_, '_>, statement: LabeledStatement) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let name = context
        .ast
        .get_utf8(statement.label(context.ast).name(context.ast))
        .as_bytes();
    let duplicate = {
        let mut ancestors = context.ancestors();
        let _ = ancestors.next();
        let mut duplicate = false;
        for ancestor in ancestors {
            match context.ast.kind_data(ancestor) {
                NodeKindData::LabeledStatement(outer) => {
                    let outer_name = context
                        .ast
                        .get_utf8(outer.label(context.ast).name(context.ast))
                        .as_bytes();
                    if outer_name == name {
                        duplicate = true;
                    }
                }
                NodeKindData::Function(_)
                | NodeKindData::ArrowFunctionExpression(_)
                | NodeKindData::StaticBlock(_) => break,
                _ => {}
            }
        }
        duplicate
    };
    if duplicate {
        let message = checks.fmt(format_args!(
            "Duplicate label '{}'",
            String::from_utf8_lossy(name)
        ));
        checks.report(
            context.ast.span(statement.label(context.ast).node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_directive(checker: &mut Checker<'_, '_>, directive: Directive) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if context.ast.get_utf8(directive.value(context.ast)) != "use strict" {
        return;
    }
    let mut parameter = None;
    for node in context.ancestors() {
        match context.ast.kind_data(node) {
            NodeKindData::Function(function) => {
                parameter = ecmascript::find_non_simple_parameter(
                    context.ast,
                    function.params(context.ast),
                );
                break;
            }
            NodeKindData::ArrowFunctionExpression(arrow) => {
                parameter =
                    ecmascript::find_non_simple_parameter(context.ast, arrow.params(context.ast));
                break;
            }
            NodeKindData::Program(_) => break,
            _ => {}
        }
    }
    if let Some(parameter) = parameter {
        checks.report(
            context.ast.span(directive.node_id()),
            "Illegal 'use strict' directive in function with non-simple parameter list",
            ReportOptions {
                labels: checks
                    .labels(&[checks.label(context.ast.span(parameter), "non-simple parameter")]),
                ..ReportOptions::default()
            },
        );
    }
}

pub(super) fn check_class_body(checker: &mut Checker<'_, '_>, body: ClassBody) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let children = body.body(context.ast);
    let mut first_constructor = None;
    for index in 0..children.len() {
        let child = children
            .get_node(context.ast, index)
            .expect("class child exists")
            .node_id();
        // TypeScript permits constructor overload declarations and leaves duplicate
        // implementation diagnostics to its type checker.
        if !context.ast.is_ts()
            && let NodeKindData::MethodDefinition(method) = context.ast.kind_data(child)
            && method.kind(context.ast) == swc_next_ecma_ast::MethodDefinitionKind::Constructor
        {
            if let Some(first) = first_constructor {
                checks.report(
                    context.ast.span(method.key(context.ast).node_id()),
                    "A class can only have one constructor",
                    ReportOptions {
                        labels: checks
                            .labels(&[checks
                                .label(context.ast.span(first), "first constructor defined here")]),
                        ..ReportOptions::default()
                    },
                );
            } else {
                first_constructor = Some(method.key(context.ast).node_id());
            }
        }
        // Body-less TypeScript methods are erased overload signatures. They do not
        // introduce runtime private names and must not conflict with the implementation.
        if context.ast.node_kind(child) == NodeKind::TsMethodDefinition {
            continue;
        }
        let Some(name) = private_element_name(context.ast, child) else {
            continue;
        };
        let kind = private_element_kind(context.ast, child);
        for previous_index in 0..index {
            let previous = children
                .get_node(context.ast, previous_index)
                .expect("class child exists")
                .node_id();
            if context.ast.node_kind(previous) == NodeKind::TsMethodDefinition {
                continue;
            }
            let Some(previous_name) = private_element_name(context.ast, previous) else {
                continue;
            };
            if previous_name != name {
                continue;
            }
            let previous_kind = private_element_kind(context.ast, previous);
            let get_set_pair = matches!(
                (previous_kind, kind),
                (PrivateElementKind::Getter, PrivateElementKind::Setter)
                    | (PrivateElementKind::Setter, PrivateElementKind::Getter)
            );
            if get_set_pair
                && private_element_is_static(context.ast, previous)
                    == private_element_is_static(context.ast, child)
            {
                continue;
            }
            let message = checks.fmt(format_args!(
                "Duplicate private name '#{}'",
                String::from_utf8_lossy(name)
            ));
            checks.report(
                context.ast.span(child),
                message,
                ReportOptions {
                    labels: checks
                        .labels(&[checks.label(context.ast.span(previous), "first declared here")]),
                    ..ReportOptions::default()
                },
            );
            break;
        }
    }
}

pub(super) fn check_function(checker: &mut Checker<'_, '_>, function: Function) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if function.function_type(context.ast) != FunctionType::FunctionDeclaration {
        return;
    }
    let strict = context.state.scope.is_strict();
    let mut through_labels = false;
    let mut ancestors = context.ancestors();
    let _ = ancestors.next();
    let parent = loop {
        let Some(parent) = ancestors.next() else {
            return;
        };
        if context.ast.node_kind(parent) != NodeKind::LabeledStatement {
            break context.ast.node_kind(parent);
        }
        through_labels = true;
    };
    let valid = match parent {
        NodeKind::Program
        | NodeKind::FunctionBody
        | NodeKind::ClassBody
        | NodeKind::StaticBlock
        | NodeKind::BlockStatement
        | NodeKind::SwitchCase
        | NodeKind::ExportNamedDeclaration
        | NodeKind::ExportDefaultDeclaration
        | NodeKind::TsModuleBlock => !strict || !through_labels,
        NodeKind::IfStatement => !strict && !through_labels,
        _ => false,
    };
    if !valid {
        let message = if strict {
            "In strict mode code, functions can only be declared at top level or inside a block"
        } else {
            "In non-strict mode code, functions can only be declared at top level, inside a block, or as the body of an if statement"
        };
        checks.report(
            context.ast.span(function.node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_with_statement(checker: &mut Checker<'_, '_>, statement: WithStatement) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if context.state.scope.is_strict() {
        checks.report(
            context.ast.span(statement.node_id()),
            "'with' statements are not allowed in strict mode",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_object_expression(checker: &mut Checker<'_, '_>, object: ObjectExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let mut first_proto = None;
    for property in object.properties(context.ast).iter() {
        let property = context.ast.get_node_in_sub_range(property).node_id();
        let NodeKindData::ObjectProperty(item) = context.ast.kind_data(property) else {
            continue;
        };
        if item.computed(context.ast)
            || item.method(context.ast)
            || item.shorthand(context.ast)
            || item.kind(context.ast) != swc_next_ecma_ast::PropertyKind::Init
        {
            continue;
        }
        let name = match context.ast.kind_data(item.key(context.ast).node_id()) {
            NodeKindData::IdentifierName(identifier) => context
                .ast
                .get_utf8(identifier.name(context.ast))
                .as_bytes(),
            NodeKindData::StringLiteral(string) => {
                context.ast.get_wtf8(string.value(context.ast)).as_bytes()
            }
            _ => continue,
        };
        if name != b"__proto__" {
            continue;
        }
        if let Some(first) = first_proto {
            checks.report(
                context.ast.span(property),
                "Duplicate '__proto__' property in object literal",
                ReportOptions {
                    labels: checks
                        .labels(&[checks.label(context.ast.span(first), "first defined here")]),
                    ..ReportOptions::default()
                },
            );
        } else {
            first_proto = Some(property);
        }
    }
}

pub(super) fn check_member_expression(checker: &mut Checker<'_, '_>, expression: MemberExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let object = expression.object(context.ast).node_id();
    if context.ast.node_kind(object) == NodeKind::Super && !is_super_property_valid(context) {
        checks.report(
            context.ast.span(expression.node_id()),
            "'super' property access is only valid inside a method or class body",
            ReportOptions {
                help: Some(Cow::Borrowed(
                    "Use an arrow function instead of a regular function to inherit the 'super' binding",
                )),
                ..ReportOptions::default()
            },
        );
    }
    let property = expression.property(context.ast).node_id();
    if !expression.computed(context.ast)
        && let NodeKindData::PrivateIdentifier(identifier) = context.ast.kind_data(property)
    {
        let name = context
            .ast
            .get_utf8(identifier.name(context.ast))
            .as_bytes();
        if context.ast.node_kind(object) == NodeKind::Super {
            checks.report(
                context.ast.span(expression.node_id()),
                "'super' keyword unexpected here",
                ReportOptions::default(),
            );
        } else if !is_private_name_declared(context, name) {
            let message = checks.fmt(format_args!(
                "Private field '#{}' must be declared in an enclosing class",
                String::from_utf8_lossy(name)
            ));
            checks.report(
                context.ast.span(property),
                message,
                ReportOptions::default(),
            );
        }
    }
}

pub(super) fn check_call_expression(checker: &mut Checker<'_, '_>, call: CallExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if context.ast.node_kind(call.callee(context.ast).node_id()) == NodeKind::Super {
        let message = "'super()' is only valid in a constructor of a derived class";
        match super_call_validity(context) {
            SuperCallValidity::Valid => {}
            SuperCallValidity::NotInConstructor => checks.report(
                context.ast.span(call.node_id()),
                message,
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Use an arrow function instead of a regular function to inherit the 'super' binding",
                    )),
                    ..ReportOptions::default()
                },
            ),
            SuperCallValidity::NoExtends => checks.report(
                context.ast.span(call.node_id()),
                message,
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Add an 'extends' clause to the class or remove the 'super()' call",
                    )),
                    ..ReportOptions::default()
                },
            ),
        }
    }
}

pub(super) fn check_await_expression(checker: &mut Checker<'_, '_>, expression: AwaitExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let message = {
        let mut message = None;
        for ancestor in context.ancestors() {
            match context.ast.node_kind(ancestor) {
                NodeKind::FormalParameters => {
                    message = Some("Await expression is not allowed in formal parameters");
                    break;
                }
                NodeKind::StaticBlock => {
                    message = Some("Cannot use await in class static initialization block");
                    break;
                }
                NodeKind::Function | NodeKind::ArrowFunctionExpression | NodeKind::Program => break,
                _ => {}
            }
        }
        message
    };
    if let Some(message) = message {
        checks.report(
            context.ast.span(expression.node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_yield_expression(checker: &mut Checker<'_, '_>, expression: YieldExpression) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    if is_in_formal_parameters(context) {
        checks.report(
            context.ast.span(expression.node_id()),
            "Yield expression is not allowed in formal parameters",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_meta_property(checker: &mut Checker<'_, '_>, property: MetaProperty) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let name = context
        .ast
        .get_utf8(property.meta(context.ast).name(context.ast));
    if name == "import" && !context.ast.is_module() {
        checks.report(
            context.ast.span(property.node_id()),
            "'import.meta' is only valid in module code",
            ReportOptions::default(),
        );
    }
    if name == "new" && !is_new_target_available(context) {
        checks.report(
            context.ast.span(property.node_id()),
            "'new.target' is only valid inside functions, class field initializers, or static blocks",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_import_declaration(
    checker: &mut Checker<'_, '_>,
    declaration: ImportDeclaration,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    checks.check_import_export_position(
        declaration.node_id(),
        "import statement",
        "'import' declaration",
        context,
    );
    checks.check_duplicate_with_attributes(declaration.attributes(context.ast), context);
}

pub(super) fn check_export_named_declaration(
    checker: &mut Checker<'_, '_>,
    declaration: ExportNamedDeclaration,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    checks.check_import_export_position(
        declaration.node_id(),
        "'export' declaration",
        "'export' declaration",
        context,
    );
    checks.check_duplicate_with_attributes(declaration.attributes(context.ast), context);
}

pub(super) fn check_export_default_declaration(
    checker: &mut Checker<'_, '_>,
    declaration: ExportDefaultDeclaration,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    checks.check_import_export_position(
        declaration.node_id(),
        "'export default' declaration",
        "'export default' declaration",
        context,
    );
    checks.record_exported_name(b"default", declaration.node_id(), context);
}

pub(super) fn check_export_all_declaration(
    checker: &mut Checker<'_, '_>,
    declaration: ExportAllDeclaration,
) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    checks.check_import_export_position(
        declaration.node_id(),
        "'export *' declaration",
        "'export *' declaration",
        context,
    );
    if let Some(exported) = declaration.exported(context.ast) {
        let name = module_export_name(context.ast, exported.node_id());
        checks.record_exported_name(name, declaration.node_id(), context);
    }
    checks.check_duplicate_with_attributes(declaration.attributes(context.ast), context);
}

pub(super) fn check_export_specifier(checker: &mut Checker<'_, '_>, specifier: ExportSpecifier) {
    let checks = &mut checker.checks;
    let context = &mut checker.context;
    let exported_name = module_export_name(context.ast, specifier.exported(context.ast).node_id());
    checks.record_exported_name(exported_name, specifier.node_id(), context);
    if specifier.export_kind(context.ast) == ImportOrExportKind::Type
        || context.state.in_ts_namespace()
    {
        return;
    }
    if let Some(parent) = context.parent()
        && let NodeKindData::ExportNamedDeclaration(declaration) = context.ast.kind_data(parent)
        && declaration.source(context.ast).is_none()
        && declaration.export_kind(context.ast) != ImportOrExportKind::Type
    {
        let local = module_export_name(context.ast, specifier.local(context.ast).node_id());
        checks.export_specifiers.push(ExportSpecifierInfo {
            local_name: local.to_vec(),
            node: specifier.node_id(),
        });
    }
}
