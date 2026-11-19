//! Scope-dependent syntax checks for JavaScript and TypeScript.

use std::borrow::Cow;

use swc_next_ecma_ast::{
    AssignmentExpression, AwaitExpression, BinaryExpression, BinaryOperator, BindingIdentifier,
    BreakStatement, CallExpression, ClassBody, ContinueStatement, Directive, ExportAllDeclaration,
    ExportDefaultDeclaration, ExportNamedDeclaration, ExportSpecifier, ForInStatement,
    ForOfStatement, Function, FunctionType, IdentifierReference, ImportDeclaration,
    ImportOrExportKind, LabelIdentifier, LabeledStatement, MemberExpression, MethodDefinition,
    NodeId, NodeKind, NodeKindData, NumericLiteral, ObjectExpression, RegExpLiteral, StringLiteral,
    SwitchStatement, TsGlobalDeclaration, TsModuleDeclaration, TsOptionalType, TsParameterProperty,
    TsRestType, UnaryExpression, UnaryOperator, UpdateExpression, VariableDeclaration,
    VariableKind, WithStatement, YieldExpression,
};

use super::{
    ReportOptions, SemanticBuilder,
    ecmascript::{self, PrivateElementKind},
    regex,
};
use crate::{ReferenceSpace, SymbolFlags};
use utils::{ContinueLabelSearch, LabelSearch, LegacyEscape, SuperCallValidity};

// Literals

/// Reports legacy escapes in strict-mode strings.
///
/// ## Example
/// ```js
/// "use strict";
/// const value = "\1";
/// ```
pub(super) fn check_string_literal(builder: &mut SemanticBuilder<'_, '_>, literal: StringLiteral) {
    if !builder.check_syntax.octal_escapes() {
        return;
    }
    if builder.default_reference_space == ReferenceSpace::Type || !builder.is_strict() {
        return;
    }
    let span = builder.ast.span(literal.node_id());
    let raw = &builder.ast.source().as_bytes()[span.start as usize..span.end as usize];
    if let Some(escape) = utils::last_legacy_escape(raw) {
        let message = match escape {
            LegacyEscape::NonOctalDecimal => "\\8 and \\9 are not allowed in strict mode.",
            LegacyEscape::Octal => "Octal escape sequences are not allowed in strict mode.",
        };
        builder.report(
            span,
            message,
            ReportOptions {
                help: Some(Cow::Borrowed(
                    "Use \\xHH (hex) or \\uHHHH (unicode) escape instead",
                )),
                ..ReportOptions::default()
            },
        );
    }
}

/// Reports legacy octal and leading-zero decimal literals in strict mode.
///
/// ## Example
/// ```js
/// "use strict";
/// const value = 077;
/// ```
pub(super) fn check_numeric_literal(
    builder: &mut SemanticBuilder<'_, '_>,
    literal: NumericLiteral,
) {
    if !builder.check_syntax.legacy_numeric_literals() {
        return;
    }
    if builder.default_reference_space == ReferenceSpace::Type || !builder.is_strict() {
        return;
    }
    let span = builder.ast.span(literal.node_id());
    let raw = &builder.ast.source().as_bytes()[span.start as usize..span.end as usize];
    if !utils::is_legacy_numeric_literal(raw) {
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
    builder.report(
        span,
        message,
        ReportOptions {
            help: Some(Cow::Borrowed(help)),
            ..ReportOptions::default()
        },
    );
}

/// Reports invalid regular expression patterns or flags.
///
/// ## Example
/// ```js
/// /a/gg;
/// ```
pub(super) fn check_reg_exp_literal(builder: &mut SemanticBuilder<'_, '_>, literal: RegExpLiteral) {
    if !builder.check_syntax.regexp_literals() {
        return;
    }
    let pattern = builder.ast.get_utf8(literal.pattern(builder.ast));
    let flags = builder.ast.get_utf8(literal.flags(builder.ast));
    let span = builder.ast.span(literal.node_id());
    if !regex::validate_literal(pattern, flags) {
        builder.report(
            span,
            "Invalid regular expression literal",
            ReportOptions::default(),
        );
    }
}

// Identifiers and assignments

/// Checks reserved identifier references and access to `arguments` in class field
/// initializers and static blocks.
///
/// ## Example
/// ```js
/// class C { field = arguments; }
/// ```
pub(super) fn check_identifier_reference(
    builder: &mut SemanticBuilder<'_, '_>,
    identifier: IdentifierReference,
    name: &[u8],
) {
    if builder.default_reference_space == ReferenceSpace::Type {
        return;
    }
    let node_id = identifier.node_id();
    utils::check_strict_reserved(builder, name, node_id, "an identifier");
    if builder.check_syntax.arguments()
        && name == b"arguments"
        && !utils::is_arguments_available(builder)
    {
        builder.report(
            builder.ast.span(node_id),
            "'arguments' is not allowed in class field initializers or static blocks",
            ReportOptions::default(),
        );
    }
}

/// Checks binding names and redeclarations and records directly exported names.
///
/// ## Example
/// ```js
/// "use strict";
/// let eval;
/// ```
pub(super) fn check_binding_identifier<'traverse>(
    builder: &mut SemanticBuilder<'traverse, '_>,
    identifier: BindingIdentifier,
    name: &'traverse [u8],
) {
    let node_id = identifier.node_id();
    let flags = builder.symbols.pending.flags;
    if flags.is_empty() {
        return;
    }
    let strict_binding = builder.default_reference_space != ReferenceSpace::Type
        && builder.is_strict()
        && !flags.contains(SymbolFlags::AMBIENT);
    if strict_binding {
        utils::check_strict_reserved(builder, name, node_id, "a binding identifier");
        if builder.check_syntax.strict_binding_names() && ecmascript::is_eval_or_arguments(name) {
            let message = format!(
                "'{}' is not allowed as a binding identifier in strict mode",
                String::from_utf8_lossy(name)
            );
            builder.report(builder.ast.span(node_id), message, ReportOptions::default());
        }
    }
    if builder.check_syntax.lexical_binding_names()
        && flags.contains(SymbolFlags::BLOCK_SCOPED_VAR)
        && name == b"let"
        && !(strict_binding && builder.check_syntax.strict_reserved_words())
    {
        builder.report(
            builder.ast.span(node_id),
            "'let' is not allowed as a variable name in a lexical declaration",
            ReportOptions::default(),
        );
    }
    if builder.check_syntax.redeclarations() {
        utils::check_redeclaration(builder, node_id, name, builder.symbols.pending);
    }
    if builder.symbols.export_state.is_named() {
        utils::record_exported_name(builder, name, node_id);
    }
}

/// Reports strict-mode reserved words used as labels.
///
/// ## Example
/// ```js
/// "use strict";
/// static: {}
/// ```
pub(super) fn check_label_identifier(
    builder: &mut SemanticBuilder<'_, '_>,
    identifier: LabelIdentifier,
) {
    let name = builder
        .ast
        .get_utf8(identifier.name(builder.ast))
        .as_bytes();
    utils::check_strict_reserved(builder, name, identifier.node_id(), "a label");
}

/// Reports `await` used as an identifier in a TypeScript module's top-level context.
///
/// ## Example
/// ```ts
/// export {};
/// import alias = await;
/// ```
#[cold]
pub(super) fn check_contextual_await(
    builder: &mut SemanticBuilder<'_, '_>,
    node: NodeId,
    message: &'static str,
) {
    if !builder.check_syntax.ts_contextual_await() {
        return;
    }
    if builder.default_reference_space == ReferenceSpace::Type
        || builder.symbols.in_ambient_context()
        || !builder.ast.is_module()
        || !utils::is_in_top_level_context(builder, node)
    {
        return;
    }
    builder.report(builder.ast.span(node), message, ReportOptions::default());
}

/// Reports assignments to `eval` or `arguments` in strict mode.
///
/// ## Example
/// ```js
/// "use strict";
/// eval = 1;
/// ```
pub(super) fn check_assignment_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: AssignmentExpression,
) {
    if builder.is_strict() {
        utils::check_assign_target_eval_arguments(builder, expression.left(builder.ast).node_id());
    }
}

/// Reports updates to `eval` or `arguments` in strict mode.
///
/// ## Example
/// ```js
/// "use strict";
/// arguments++;
/// ```
pub(super) fn check_update_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: UpdateExpression,
) {
    if !builder.check_syntax.strict_assignment_targets() {
        return;
    }
    if builder.is_strict()
        && ecmascript::is_eval_or_arguments_reference(
            builder.ast,
            expression.argument(builder.ast).node_id(),
        )
    {
        builder.report(
            builder.ast.span(expression.node_id()),
            "Cannot assign to 'eval' or 'arguments' in strict mode",
            ReportOptions::default(),
        );
    }
}

// Expressions and classes

/// Checks that a private name used with `in` is declared in an enclosing class.
///
/// ## Example
/// ```js
/// class C { has(value) { return #missing in value; } }
/// ```
pub(super) fn check_binary_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: BinaryExpression,
) {
    if !builder.check_syntax.private_name_references() {
        return;
    }
    let left = expression.left(builder.ast).node_id();
    if expression.operator(builder.ast) == BinaryOperator::In
        && let NodeKindData::PrivateIdentifier(identifier) = builder.ast.kind_data(left)
    {
        let name = builder
            .ast
            .get_utf8(identifier.name(builder.ast))
            .as_bytes();
        if !utils::is_private_name_declared(builder, name) {
            let message = format!(
                "Private field '#{}' must be declared in an enclosing class",
                String::from_utf8_lossy(name)
            );
            builder.report(builder.ast.span(left), message, ReportOptions::default());
        }
    }
}

/// Checks deletion of private properties and deletion of identifiers in strict mode.
///
/// ## Example
/// ```js
/// "use strict";
/// delete value;
/// ```
pub(super) fn check_unary_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: UnaryExpression,
) {
    if expression.operator(builder.ast) != UnaryOperator::Delete {
        return;
    }
    let mut target =
        ecmascript::unwrap_parentheses(builder.ast, expression.argument(builder.ast).node_id());
    if let NodeKindData::ChainExpression(chain) = builder.ast.kind_data(target) {
        target =
            ecmascript::unwrap_parentheses(builder.ast, chain.expression(builder.ast).node_id());
    }
    match builder.ast.kind_data(target) {
        NodeKindData::IdentifierReference(_)
            if builder.check_syntax.delete_identifiers() && builder.is_strict() =>
        {
            builder.report(
                builder.ast.span(expression.node_id()),
                "Deleting a variable in strict mode is not allowed",
                ReportOptions::default(),
            );
        }
        NodeKindData::MemberExpression(member)
            if builder.check_syntax.delete_private_fields()
                && !member.computed(builder.ast)
                && builder
                    .ast
                    .node_kind(member.property(builder.ast).node_id())
                    == NodeKind::PrivateIdentifier =>
        {
            builder.report(
                builder.ast.span(expression.node_id()),
                "Private properties cannot be deleted",
                ReportOptions::default(),
            );
        }
        _ => {}
    }
}

/// Reports multiple `__proto__` setters in an object literal.
///
/// ## Example
/// ```js
/// const object = { __proto__: null, __proto__: {} };
/// ```
pub(super) fn check_object_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    object: ObjectExpression,
) {
    if !builder.check_syntax.duplicate_proto() {
        return;
    }
    let mut first_proto = None;
    for property in object.properties(builder.ast).iter() {
        let property = builder.ast.get_node_in_sub_range(property).node_id();
        let NodeKindData::ObjectProperty(item) = builder.ast.kind_data(property) else {
            continue;
        };
        if item.computed(builder.ast)
            || item.method(builder.ast)
            || item.shorthand(builder.ast)
            || item.kind(builder.ast) != swc_next_ecma_ast::PropertyKind::Init
        {
            continue;
        }
        let name = match builder.ast.kind_data(item.key(builder.ast).node_id()) {
            NodeKindData::IdentifierName(identifier) => builder
                .ast
                .get_utf8(identifier.name(builder.ast))
                .as_bytes(),
            NodeKindData::StringLiteral(string) => {
                builder.ast.get_wtf8(string.value(builder.ast)).as_bytes()
            }
            _ => continue,
        };
        if name != b"__proto__" {
            continue;
        }
        if let Some(first) = first_proto {
            builder.report(
                builder.ast.span(property),
                "Duplicate '__proto__' property in object literal",
                ReportOptions {
                    labels: vec![builder.label(builder.ast.span(first), "first defined here")],
                    ..ReportOptions::default()
                },
            );
        } else {
            first_proto = Some(property);
        }
    }
}

/// Checks private name references and access to `super` properties.
///
/// ## Example
/// ```js
/// class C { read() { return this.#missing; } }
/// ```
pub(super) fn check_member_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: MemberExpression,
) {
    let object = expression.object(builder.ast).node_id();
    if builder.check_syntax.super_properties()
        && builder.ast.node_kind(object) == NodeKind::Super
        && !utils::is_super_property_valid(builder)
    {
        builder.report(
            builder.ast.span(expression.node_id()),
            "'super' property access is not allowed in this context",
            ReportOptions::default(),
        );
    }
    let property = expression.property(builder.ast).node_id();
    if !expression.computed(builder.ast)
        && let NodeKindData::PrivateIdentifier(identifier) = builder.ast.kind_data(property)
    {
        let name = builder
            .ast
            .get_utf8(identifier.name(builder.ast))
            .as_bytes();
        if builder.ast.node_kind(object) == NodeKind::Super {
            if !builder.check_syntax.super_properties() {
                return;
            }
            builder.report(
                builder.ast.span(expression.node_id()),
                "'super' keyword unexpected here",
                ReportOptions::default(),
            );
        } else if builder.check_syntax.private_name_references()
            && !utils::is_private_name_declared(builder, name)
        {
            let message = format!(
                "Private field '#{}' must be declared in an enclosing class",
                String::from_utf8_lossy(name)
            );
            builder.report(
                builder.ast.span(property),
                message,
                ReportOptions::default(),
            );
        }
    }
}

/// Checks that `super()` is called within a derived constructor.
///
/// ## Example
/// ```js
/// class C { constructor() { super(); } }
/// ```
pub(super) fn check_call_expression(builder: &mut SemanticBuilder<'_, '_>, call: CallExpression) {
    if !builder.check_syntax.super_calls() {
        return;
    }
    if builder.ast.node_kind(call.callee(builder.ast).node_id()) == NodeKind::Super {
        let message = "'super()' is only valid in a constructor of a derived class";
        match utils::super_call_validity(builder) {
            SuperCallValidity::Valid => {}
            SuperCallValidity::NotInConstructor => builder.report(
                builder.ast.span(call.node_id()),
                message,
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Use an arrow function instead of a regular function to inherit the 'super' binding",
                    )),
                    ..ReportOptions::default()
                },
            ),
            SuperCallValidity::NoExtends => builder.report(
                builder.ast.span(call.node_id()),
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

/// Checks class member names, duplicate constructors, and conflicting private names.
///
/// ## Example
/// ```js
/// class C { static prototype; }
/// ```
pub(super) fn check_class_body(builder: &mut SemanticBuilder<'_, '_>, body: ClassBody) {
    if !builder.check_syntax.class_body() {
        return;
    }
    let tree = builder.ast;
    let children = body.body(tree);
    let mut first_constructor = None;
    for (index, child) in children.iter().enumerate() {
        let child = tree.get_node_in_sub_range(child).node_id();
        if builder.check_syntax.class_member_names() {
            utils::check_class_member_name(builder, child);
        }
        // TypeScript permits constructor overload declarations and leaves duplicate
        // implementation diagnostics to its type checker.
        if builder.check_syntax.duplicate_constructors()
            && !tree.is_ts()
            && let NodeKindData::MethodDefinition(method) = tree.kind_data(child)
            && method.kind(tree) == swc_next_ecma_ast::MethodDefinitionKind::Constructor
        {
            if let Some(first) = first_constructor {
                builder.report(
                    tree.span(method.key(tree).node_id()),
                    "A class can only have one constructor",
                    ReportOptions {
                        labels: vec![
                            builder.label(tree.span(first), "first constructor defined here"),
                        ],
                        ..ReportOptions::default()
                    },
                );
            } else {
                first_constructor = Some(method.key(tree).node_id());
            }
        }
        // Body-less TypeScript methods are erased overload signatures. They do not
        // introduce runtime private names and must not conflict with the implementation.
        if !builder.check_syntax.duplicate_private_names()
            || tree.node_kind(child) == NodeKind::TsMethodDefinition
        {
            continue;
        }
        let Some(name) = ecmascript::private_element_name(tree, child) else {
            continue;
        };
        let kind = ecmascript::private_element_kind(tree, child);
        for previous in children.iter().take(index) {
            let previous = tree.get_node_in_sub_range(previous).node_id();
            if tree.node_kind(previous) == NodeKind::TsMethodDefinition {
                continue;
            }
            let Some(previous_name) = ecmascript::private_element_name(tree, previous) else {
                continue;
            };
            if previous_name != name {
                continue;
            }
            let previous_kind = ecmascript::private_element_kind(tree, previous);
            let get_set_pair = matches!(
                (previous_kind, kind),
                (PrivateElementKind::Getter, PrivateElementKind::Setter)
                    | (PrivateElementKind::Setter, PrivateElementKind::Getter)
            );
            if get_set_pair
                && ecmascript::private_element_is_static(tree, previous)
                    == ecmascript::private_element_is_static(tree, child)
            {
                continue;
            }
            let message = format!(
                "Duplicate private name '#{}'",
                String::from_utf8_lossy(name)
            );
            builder.report(
                tree.span(child),
                message,
                ReportOptions {
                    labels: vec![builder.label(tree.span(previous), "first declared here")],
                    ..ReportOptions::default()
                },
            );
            break;
        }
    }
}

/// Reports abstract methods with an implementation.
///
/// ## Example
/// ```ts
/// abstract class C { abstract run() {} }
/// ```
pub(super) fn check_method_definition(
    builder: &mut SemanticBuilder<'_, '_>,
    method: MethodDefinition,
) {
    if !builder.check_syntax.ts_abstract_methods() {
        return;
    }
    if method.r#abstract(builder.ast) {
        builder.report(
            builder.ast.span(method.node_id()),
            "An abstract method cannot have an implementation",
            ReportOptions::default(),
        );
    }
}

/// Checks that a parameter property uses an identifier binding other than `constructor`.
///
/// ## Example
/// ```ts
/// class C { constructor(public constructor: number) {} }
/// ```
pub(super) fn check_parameter_property(
    builder: &mut SemanticBuilder<'_, '_>,
    property: TsParameterProperty,
) {
    if !builder.check_syntax.ts_parameter_properties() {
        return;
    }
    let Some(binding) = ecmascript::parameter_property_binding(builder.ast, property) else {
        builder.report(
            builder.ast.span(property.node_id()),
            "A parameter property must use an identifier binding",
            ReportOptions::default(),
        );
        return;
    };
    if builder.ast.get_utf8(binding.name(builder.ast)).as_bytes() == b"constructor" {
        builder.report(
            builder.ast.span(property.node_id()),
            "A parameter property cannot be named 'constructor'",
            ReportOptions::default(),
        );
    }
}

// Statements and functions

/// Checks whether the enclosing context permits an `await` expression.
///
/// ## Example
/// ```ts
/// class C { static { await load(); } }
/// ```
pub(super) fn check_await_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: AwaitExpression,
) {
    if !builder.check_syntax.await_expressions() {
        return;
    }
    if let Some(message) = utils::await_expression_error(builder) {
        builder.report(
            builder.ast.span(expression.node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

/// Checks whether a `yield` expression is in a generator body.
///
/// ## Example
/// ```js
/// function* f(value = yield item) {}
/// ```
pub(super) fn check_yield_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: YieldExpression,
) {
    if !builder.check_syntax.yield_expressions() {
        return;
    }
    if let Some(message) = utils::yield_expression_error(builder) {
        builder.report(
            builder.ast.span(expression.node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

/// Checks whether the enclosing context permits `using` or `await using`.
///
/// ## Example
/// A script cannot declare a resource at the top level.
///
/// ```ts
/// using resource = acquire();
/// ```
pub(super) fn check_variable_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: VariableDeclaration,
) {
    if !builder.check_syntax.using_declarations() {
        return;
    }
    let kind = declaration.kind(builder.ast);
    if !matches!(kind, VariableKind::Using | VariableKind::AwaitUsing) {
        return;
    }
    if kind == VariableKind::AwaitUsing && utils::is_inside_static_block(builder) {
        builder.report(
            builder.ast.span(declaration.node_id()),
            "Cannot use 'await using' in class static initialization block",
            ReportOptions::default(),
        );
    }
    if builder.symbols.in_ambient_context() {
        let message = format!(
            "'{}' declarations are not allowed in ambient contexts",
            kind.as_str()
        );
        builder.report(
            builder.ast.span(declaration.node_id()),
            message,
            ReportOptions::default(),
        );
        return;
    }
    if builder.ast.source_type() == swc_next_ecma_ast::SourceType::Script
        && utils::is_at_program_level(builder)
    {
        let message = format!(
            "'{}' is not allowed at the top level of a script",
            kind.as_str()
        );
        builder.report(
            builder.ast.span(declaration.node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

/// Reports multiple `default` clauses in a switch statement.
///
/// ## Example
/// ```js
/// switch (value) { default: break; default: break; }
/// ```
pub(super) fn check_switch_statement(
    builder: &mut SemanticBuilder<'_, '_>,
    statement: SwitchStatement,
) {
    if !builder.check_syntax.duplicate_switch_defaults() {
        return;
    }
    let mut first_default = None;
    for case in statement.cases(builder.ast).iter() {
        let case = builder.ast.get_node_in_sub_range(case);
        if case.test(builder.ast).is_some() {
            continue;
        }
        if let Some(first) = first_default {
            builder.report(
                builder.ast.span(case.node_id()),
                "A switch statement can only have one default clause",
                ReportOptions {
                    labels: vec![
                        builder.label(builder.ast.span(first), "first default defined here"),
                    ],
                    ..ReportOptions::default()
                },
            );
        } else {
            first_default = Some(case.node_id());
        }
    }
}

/// Checks declaration initializers and assignment targets in `for...in` loops.
///
/// ## Example
/// ```js
/// for (let key = 0 in object) {}
/// ```
pub(super) fn check_for_in_statement(
    builder: &mut SemanticBuilder<'_, '_>,
    statement: ForInStatement,
) {
    let left = statement.left(builder.ast).node_id();
    utils::check_for_in_of_initializer(builder, left, true);
    if builder.is_strict() {
        utils::check_assign_target_eval_arguments(builder, left);
    }
}

/// Checks declaration initializers, assignment targets, and `for await` placement.
///
/// ## Example
/// ```js
/// for (let value = 0 of values) {}
/// ```
pub(super) fn check_for_of_statement(
    builder: &mut SemanticBuilder<'_, '_>,
    statement: ForOfStatement,
) {
    if builder.check_syntax.for_await()
        && statement.r#await(builder.ast)
        && !builder.ast.is_ts()
        && utils::is_inside_static_block(builder)
    {
        builder.report(
            builder.ast.span(statement.node_id()),
            "Cannot use 'for await' in class static initialization block",
            ReportOptions::default(),
        );
    }
    let left = statement.left(builder.ast).node_id();
    utils::check_for_in_of_initializer(builder, left, false);
    if builder.is_strict() {
        utils::check_assign_target_eval_arguments(builder, left);
    }
}

/// Checks that a `break` statement has an enclosing target.
///
/// ## Example
/// ```js
/// break;
/// ```
pub(super) fn check_break_statement(
    builder: &mut SemanticBuilder<'_, '_>,
    statement: BreakStatement,
) {
    if !builder.check_syntax.break_statements() {
        return;
    }
    if let Some(label) = statement.label(builder.ast) {
        let name = builder.ast.get_utf8(label.name(builder.ast)).as_bytes();
        let message = match utils::find_label(builder, name) {
            LabelSearch::NotFound => {
                format!("Use of undefined label '{}'", String::from_utf8_lossy(name))
            }
            LabelSearch::CrossedBoundary => format!(
                "Cannot break to label '{}' across function boundaries",
                String::from_utf8_lossy(name)
            ),
            LabelSearch::Found => return,
        };
        builder.report(
            builder.ast.span(statement.node_id()),
            message,
            ReportOptions::default(),
        );
    } else if !utils::is_inside_breakable(builder) {
        builder.report(
            builder.ast.span(statement.node_id()),
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

/// Checks that a `continue` statement targets an enclosing loop.
///
/// ## Example
/// ```js
/// continue;
/// ```
pub(super) fn check_continue_statement(
    builder: &mut SemanticBuilder<'_, '_>,
    statement: ContinueStatement,
) {
    if !builder.check_syntax.continue_statements() {
        return;
    }
    if let Some(label) = statement.label(builder.ast) {
        let name = builder.ast.get_utf8(label.name(builder.ast)).as_bytes();
        let (message, help) = match utils::find_label_for_continue(builder, name) {
            ContinueLabelSearch::NotFound => (
                format!("Use of undefined label '{}'", String::from_utf8_lossy(name)),
                None,
            ),
            ContinueLabelSearch::CrossedBoundary => (
                format!(
                    "Cannot continue to label '{}' across function boundaries",
                    String::from_utf8_lossy(name)
                ),
                None,
            ),
            ContinueLabelSearch::NotIteration => (
                format!(
                    "Label '{}' does not denote an iteration statement",
                    String::from_utf8_lossy(name)
                ),
                Some(Cow::Borrowed(
                    "A 'continue' statement can only jump to a label of an enclosing 'for', 'while', 'do-while', 'for-in', or 'for-of' statement",
                )),
            ),
            ContinueLabelSearch::Found => return,
        };
        builder.report(
            builder.ast.span(statement.node_id()),
            message,
            ReportOptions {
                help,
                ..ReportOptions::default()
            },
        );
    } else if !utils::is_inside_iteration(builder) {
        builder.report(
            builder.ast.span(statement.node_id()),
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

/// Reports nested labels with the same name in a control-flow context.
///
/// ## Example
/// ```js
/// outer: { outer: {} }
/// ```
pub(super) fn check_labeled_statement(
    builder: &mut SemanticBuilder<'_, '_>,
    statement: LabeledStatement,
) {
    if !builder.check_syntax.labels() {
        return;
    }
    let name = builder
        .ast
        .get_utf8(statement.label(builder.ast).name(builder.ast))
        .as_bytes();
    let duplicate = {
        let mut ancestors = builder.ancestors();
        let _ = ancestors.next();
        let mut duplicate = false;
        for ancestor in ancestors {
            match builder.ast.kind_data(ancestor) {
                NodeKindData::LabeledStatement(outer) => {
                    let outer_name = builder
                        .ast
                        .get_utf8(outer.label(builder.ast).name(builder.ast))
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
        let message = format!("Duplicate label '{}'", String::from_utf8_lossy(name));
        builder.report(
            builder.ast.span(statement.label(builder.ast).node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

/// Reports `with` statements in strict mode.
///
/// ## Example
/// ```js
/// "use strict";
/// with (object) {}
/// ```
pub(super) fn check_with_statement(
    builder: &mut SemanticBuilder<'_, '_>,
    statement: WithStatement,
) {
    if !builder.check_syntax.with_statements() {
        return;
    }
    if builder.is_strict() {
        builder.report(
            builder.ast.span(statement.node_id()),
            "'with' statements are not allowed in strict mode",
            ReportOptions::default(),
        );
    }
}

/// Checks whether a function declaration is allowed in its statement position.
///
/// ## Example
/// ```js
/// "use strict";
/// if (condition) function f() {}
/// ```
pub(super) fn check_function(builder: &mut SemanticBuilder<'_, '_>, function: Function) {
    if !builder.check_syntax.function_declarations() {
        return;
    }
    if function.function_type(builder.ast) != FunctionType::FunctionDeclaration {
        return;
    }
    let scope = builder.current_scope();
    let enclosing = scope.parent.expect("check runs in the function scope");
    let strict = builder.scopes.get(enclosing).flags.strict;
    let mut through_labels = false;
    let mut ancestors = builder.ancestors();
    let _ = ancestors.next();
    let parent = loop {
        let Some(parent) = ancestors.next() else {
            return;
        };
        if builder.ast.node_kind(parent) != NodeKind::LabeledStatement {
            break builder.ast.node_kind(parent);
        }
        through_labels = true;
    };
    drop(ancestors);
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
        builder.report(
            builder.ast.span(function.node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

/// Reports a `"use strict"` directive in a function with non-simple parameters.
///
/// ## Example
/// ```js
/// function f(value = 0) { "use strict"; }
/// ```
pub(super) fn check_directive(builder: &mut SemanticBuilder<'_, '_>, directive: Directive) {
    if !builder.check_syntax.use_strict_directives() {
        return;
    }
    if builder.ast.get_utf8(directive.value(builder.ast)) != "use strict" {
        return;
    }
    let mut parameter = None;
    for node in builder.ancestors() {
        match builder.ast.kind_data(node) {
            NodeKindData::Function(function) => {
                parameter = ecmascript::find_non_simple_parameter(
                    builder.ast,
                    function.params(builder.ast),
                );
                break;
            }
            NodeKindData::ArrowFunctionExpression(arrow) => {
                parameter =
                    ecmascript::find_non_simple_parameter(builder.ast, arrow.params(builder.ast));
                break;
            }
            NodeKindData::Program(_) => break,
            _ => {}
        }
    }
    if let Some(parameter) = parameter {
        builder.report(
            builder.ast.span(directive.node_id()),
            "Illegal 'use strict' directive in function with non-simple parameter list",
            ReportOptions {
                labels: vec![builder.label(builder.ast.span(parameter), "non-simple parameter")],
                ..ReportOptions::default()
            },
        );
    }
}

// Imports, exports, and namespaces

/// Checks local export bindings after visiting every declaration in the program.
///
/// ## Example
/// ```js
/// export { missing };
/// ```
pub(super) fn check_program(builder: &mut SemanticBuilder<'_, '_>) {
    utils::check_unresolved_exports(builder);
}

/// Checks import placement and duplicate import attributes.
///
/// ## Example
/// ```js
/// import data from "./data.json" with { type: "json", type: "json" };
/// ```
pub(super) fn check_import_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: ImportDeclaration,
) {
    utils::check_import_export_position(
        builder,
        declaration.node_id(),
        "import statement",
        "'import' declaration",
    );
    utils::check_duplicate_with_attributes(builder, declaration.attributes(builder.ast));
}

/// Checks named export placement and duplicate re-export attributes.
///
/// ## Example
/// ```js
/// export { value } from "./data.json" with { type: "json", type: "json" };
/// ```
pub(super) fn check_export_named_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: ExportNamedDeclaration,
) {
    utils::check_import_export_position(
        builder,
        declaration.node_id(),
        "'export' declaration",
        "'export' declaration",
    );
    utils::check_duplicate_with_attributes(builder, declaration.attributes(builder.ast));
}

/// Checks default export placement and records the default name for duplicate detection.
///
/// ## Example
/// ```js
/// export default 1;
/// export default 2;
/// ```
pub(super) fn check_export_default_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: ExportDefaultDeclaration,
) {
    utils::check_import_export_position(
        builder,
        declaration.node_id(),
        "'export default' declaration",
        "'export default' declaration",
    );
    utils::record_exported_name(builder, b"default", declaration.node_id());
}

/// Checks wildcard export placement, attributes, and any exported namespace name.
///
/// ## Example
/// ```js
/// export * as ns from "./a.js";
/// export * as ns from "./b.js";
/// ```
pub(super) fn check_export_all_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: ExportAllDeclaration,
) {
    utils::check_import_export_position(
        builder,
        declaration.node_id(),
        "'export *' declaration",
        "'export *' declaration",
    );
    if let Some(exported) = declaration.exported(builder.ast) {
        let name = ecmascript::module_export_name(builder.ast, exported.node_id());
        utils::record_exported_name(builder, name, declaration.node_id());
    }
    utils::check_duplicate_with_attributes(builder, declaration.attributes(builder.ast));
}

/// Records the exported name and queues local value exports for binding validation.
///
/// ## Example
/// ```js
/// const item = 1;
/// export { item as value };
/// export { item as value };
/// ```
pub(super) fn check_export_specifier(
    builder: &mut SemanticBuilder<'_, '_>,
    specifier: ExportSpecifier,
) {
    let exported_name =
        ecmascript::module_export_name(builder.ast, specifier.exported(builder.ast).node_id());
    utils::record_exported_name(builder, exported_name, specifier.node_id());
    if !builder.check_syntax.unresolved_exports()
        || specifier.export_kind(builder.ast) == ImportOrExportKind::Type
        || builder.ts_namespace_depth > 0
    {
        return;
    }
    if let Some(parent) = builder.parent()
        && let NodeKindData::ExportNamedDeclaration(declaration) = builder.ast.kind_data(parent)
        && declaration.source(builder.ast).is_none()
        && declaration.export_kind(builder.ast) != ImportOrExportKind::Type
    {
        builder.export_specifiers.push(specifier);
    }
}

/// Checks where TypeScript namespaces and ambient module declarations can appear.
///
/// ## Example
/// ```ts
/// function f() { namespace N {} }
/// ```
pub(super) fn check_ts_module_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: TsModuleDeclaration,
) {
    if !builder.check_syntax.ts_module_declarations() {
        return;
    }
    let Some((node, message)) = utils::ts_module_declaration_context_error(builder, declaration)
    else {
        return;
    };
    builder.report(builder.ast.span(node), message, ReportOptions::default());
}

/// Checks a global augmentation's `declare` modifier and enclosing container.
///
/// ## Example
/// This augmentation omits `declare`.
///
/// ```ts
/// export {};
/// global {}
/// ```
pub(super) fn check_ts_global_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: TsGlobalDeclaration,
) {
    if !builder.check_syntax.ts_global_declarations() {
        return;
    }
    if !declaration.declare(builder.ast) && !utils::in_ts_syntactic_ambient_context(builder) {
        builder.report(
            builder.ast.span(declaration.id(builder.ast).node_id()),
            "Augmentations for the global scope should have 'declare' modifier unless they appear in already ambient context",
            ReportOptions::default(),
        );
    }
    let Some((node, message)) = utils::ts_global_declaration_context_error(builder, declaration)
    else {
        return;
    };
    builder.report(builder.ast.span(node), message, ReportOptions::default());
}

// Tuple types

/// Reports an optional marker placed after the type of a labeled tuple element.
///
/// ## Example
/// ```ts
/// type Pair = [value: number?];
/// ```
pub(super) fn check_tuple_optional_type(
    builder: &mut SemanticBuilder<'_, '_>,
    optional: TsOptionalType,
) {
    if !builder.check_syntax.ts_tuple_optional() {
        return;
    }
    if let Some(parent) = builder.parent()
        && builder.ast.node_kind(parent) == NodeKind::TsNamedTupleMember
    {
        builder.report(
            builder.ast.span(optional.node_id()),
            "A labeled tuple element is declared as optional with '?' after the name and before the colon, rather than after the type",
            ReportOptions::default(),
        );
    }
}

/// Checks rest markers on labeled tuple elements and rejects optional rest elements.
///
/// ## Example
/// ```ts
/// type Items = [values: ...number[]];
/// ```
pub(super) fn check_tuple_rest_type(builder: &mut SemanticBuilder<'_, '_>, rest: TsRestType) {
    if !builder.check_syntax.ts_tuple_rest() {
        return;
    }
    let annotation = rest.type_annotation(builder.ast);
    if let Some(member) = annotation.as_ts_named_tuple_member(builder.ast) {
        if member.optional(builder.ast) {
            builder.report(
                builder.ast.span(rest.node_id()),
                "A tuple member cannot be both optional and rest",
                ReportOptions::default(),
            );
        }
        return;
    }
    if let Some(parent) = builder.parent()
        && builder.ast.node_kind(parent) == NodeKind::TsNamedTupleMember
    {
        builder.report(
            builder.ast.span(rest.node_id()),
            "A labeled tuple element is declared as rest with '...' before the name, rather than before the type",
            ReportOptions::default(),
        );
    }
}

mod utils {
    use std::collections::hash_map::Entry;

    use swc_next_ecma_ast::{
        FunctionType, Lang, NodeId, NodeKind, NodeKindData, ScopeId, TsGlobalDeclaration,
        TsModuleDeclaration, TsModuleNameData, VariableKind,
    };

    use super::{ReportOptions, SemanticBuilder, ecmascript};
    use crate::{SymbolFlags, binder::PendingBinding, scope::ScopeKind};

    // Literals

    /// Leading zero followed by digits: legacy octal (`077`) or
    /// leading-zero decimal (`089`).
    pub(super) fn is_legacy_numeric_literal(raw: &[u8]) -> bool {
        raw.len() >= 2 && raw[0] == b'0' && raw[1].is_ascii_digit()
    }

    pub(super) enum LegacyEscape {
        Octal,
        NonOctalDecimal,
    }

    pub(super) fn last_legacy_escape(value: &[u8]) -> Option<LegacyEscape> {
        let mut position = 0;
        let mut escape = None;
        while let Some(offset) = memchr::memchr(b'\\', &value[position..]) {
            let slash = position + offset;
            let next = slash + 1;
            if next >= value.len() {
                break;
            }
            match value[next] {
                b'1'..=b'7' => escape = Some(LegacyEscape::Octal),
                b'8' | b'9' => escape = Some(LegacyEscape::NonOctalDecimal),
                b'0' if value.get(next + 1).is_some_and(u8::is_ascii_digit) => {
                    escape = Some(LegacyEscape::Octal);
                }
                _ => {}
            }
            position = next + 1;
        }
        escape
    }

    // Identifiers and assignments

    /// <https://tc39.es/ecma262/#sec-keywords-and-reserved-words>
    pub(super) fn check_strict_reserved(
        builder: &mut SemanticBuilder<'_, '_>,
        name: &[u8],
        node: NodeId,
        as_what: &'static str,
    ) {
        if !builder.check_syntax.strict_reserved_words() {
            return;
        }
        if !builder.is_strict() {
            return;
        }
        if let Some(word) = match_strict_reserved(name) {
            let message = format!(
                "'{}' is reserved in strict mode and cannot be used as {}",
                word, as_what
            );
            builder.report(builder.ast.span(node), message, ReportOptions::default());
        }
    }

    /// <https://tc39.es/ecma262/#sec-keywords-and-reserved-words>
    fn match_strict_reserved(name: &[u8]) -> Option<&'static str> {
        match name {
            b"let" => Some("let"),
            b"yield" => Some("yield"),
            b"static" => Some("static"),
            b"public" => Some("public"),
            b"private" => Some("private"),
            b"package" => Some("package"),
            b"protected" => Some("protected"),
            b"interface" => Some("interface"),
            b"implements" => Some("implements"),
            _ => None,
        }
    }

    /// `arguments` is available inside regular functions but not in class field
    /// initializers or static blocks (arrow functions are transparent).
    pub(super) fn is_arguments_available(builder: &SemanticBuilder<'_, '_>) -> bool {
        let tree = builder.ast;
        let mut child = None;
        for ancestor in builder.ancestors() {
            match tree.kind_data(ancestor) {
                NodeKindData::Function(_) => return true,
                NodeKindData::PropertyDefinition(property) => {
                    if let Some(value) = property.value(tree)
                        && Some(value.node_id()) == child
                    {
                        return false;
                    }
                }
                NodeKindData::StaticBlock(_) => return false,
                NodeKindData::ArrowFunctionExpression(_) | NodeKindData::Program(_) => {}
                _ => {}
            }
            child = Some(ancestor);
        }
        true
    }

    /// Mirrors TypeScript's `isInTopLevelContext` test for contextual identifiers.
    /// The parser owns `[Await]` grammar errors; this checker only adds the external
    /// module rule, whose boundary depends on the surrounding runtime container.
    pub(super) fn is_in_top_level_context(context: &SemanticBuilder<'_, '_>, node: NodeId) -> bool {
        let tree = context.ast;
        for ancestor in context.ancestors().skip(1) {
            match tree.kind_data(ancestor) {
                NodeKindData::Program(_) => return true,
                NodeKindData::Function(function) => {
                    let is_declaration_name = function.function_type(tree)
                        == FunctionType::FunctionDeclaration
                        && function.id(tree).is_some_and(|id| id.node_id() == node);
                    if !is_declaration_name {
                        return function.r#async(tree);
                    }
                }
                NodeKindData::TsFunction(function) => {
                    let is_declaration_name =
                        function.id(tree).is_some_and(|id| id.node_id() == node);
                    if !is_declaration_name {
                        return function.r#async(tree);
                    }
                }
                NodeKindData::ArrowFunctionExpression(_)
                | NodeKindData::StaticBlock(_)
                | NodeKindData::PropertyDefinition(_)
                | NodeKindData::MethodDefinition(_)
                | NodeKindData::TsMethodDefinition(_)
                | NodeKindData::TsEnumDeclaration(_)
                | NodeKindData::TsModuleDeclaration(_) => return false,
                _ => {}
            }
        }
        false
    }

    pub(super) fn check_redeclaration(
        builder: &mut SemanticBuilder<'_, '_>,
        node: NodeId,
        name: &[u8],
        binding: PendingBinding,
    ) {
        let tree = builder.ast;
        let target = binding.scope;
        let flags = binding.flags;
        let excludes = binding.excludes;
        debug_assert!(!name.is_empty() || flags.contains(SymbolFlags::ENUM_MEMBER));
        let existing = builder.symbols.own_binding(target, name).filter(|&id| {
            let flags = builder.symbols.symbol(id).flags;
            !flags.intersects(SymbolFlags::INTERNAL_NAME)
        });
        if let Some(existing_id) = existing {
            let existing_flags = builder.symbols.local_declaration_flags(existing_id, target);
            if tree.is_ts()
                && let Some(previous) = builder.symbols.ts_class_function_conflict(
                    existing_id,
                    existing_flags,
                    flags,
                    Some(target),
                )
            {
                builder.report_redeclaration(name, node, previous.node, previous.flags);
                return;
            }
            let existing_node = builder.symbols.first_decl_of(existing_id);
            let import_value_conflict = !tree.is_ts()
                && tree.is_module()
                && target == ScopeId::MODULE
                && ((flags.contains(SymbolFlags::IMPORT) && existing_flags.in_value_space())
                    || (flags.in_value_space() && existing_flags.contains(SymbolFlags::IMPORT)));
            let declaration_conflict = existing_flags.intersects(excludes) || import_value_conflict;
            if declaration_conflict {
                builder.report_redeclaration(name, node, existing_node, existing_flags);
                return;
            }

            // https://tc39.es/ecma262/#sec-parameter-lists-static-semantics-early-errors
            // Duplicate parameters are allowed in sloppy simple-parameter-list
            // functions. Re-add the conflict for strict, unique, arrow, and
            // non-simple cases.
            if existing_flags.contains(SymbolFlags::PARAMETER)
                && flags.contains(SymbolFlags::PARAMETER)
                && let Some(parameters) = find_formal_parameters(builder)
            {
                let kind = parameters.kind(tree);
                let must_be_unique = builder.is_strict()
                    || matches!(
                        kind,
                        swc_next_ecma_ast::FormalParameterKind::UniqueFormalParameters
                            | swc_next_ecma_ast::FormalParameterKind::ArrowFormalParameters
                    )
                    || ecmascript::find_non_simple_parameter(tree, parameters).is_some();
                if must_be_unique {
                    builder.report_redeclaration(name, node, existing_node, existing_flags);
                }
            }
        }

        if tree.is_ts()
            && let Some((node_id, flags)) = builder.symbols.ts_export_conflict(tree, name, binding)
        {
            builder.report_redeclaration(name, node, node_id, flags);
            return;
        }

        if let Some(existing) = builder.symbols.hoisting_conflict(binding, name) {
            builder.report_redeclaration(name, node, existing.node, existing.flags);
            return;
        }

        if !flags.contains(SymbolFlags::FUNCTION_SCOPED_VAR) {
            check_outer_parameter_conflict(builder, node, name, binding);
        }
        if flags.is_hoisting_var() {
            check_hoisting_var_conflict(builder, node, name, binding);
        }
    }

    fn check_outer_parameter_conflict(
        builder: &mut SemanticBuilder<'_, '_>,
        node: NodeId,
        name: &[u8],
        binding: PendingBinding,
    ) {
        let target = binding.scope;
        let excludes = binding.excludes;
        // A split body environment keeps the early errors: lexical body names
        // may not duplicate parameter names (15.2.1) or catch parameter names
        // (14.15.1) one scope up:
        //
        //   function f(a = 1) { let a }   // still an error
        //   function f(a = 1) { var a }   // legal, var stays out
        let target_scope = builder.scopes.get(target);
        let Some(outer) = target_scope.parent else {
            return;
        };
        let outer_holds_parameters = match target_scope.kind {
            ScopeKind::FunctionBody => true,
            ScopeKind::Block => match builder.ast.kind_data(builder.scopes.get(outer).node) {
                NodeKindData::CatchClause(catch) => {
                    catch.body(builder.ast).node_id() == target_scope.node
                }
                _ => false,
            },
            _ => false,
        };
        if !outer_holds_parameters {
            return;
        }
        let Some(symbol_id) = builder.symbols.own_binding(outer, name) else {
            return;
        };
        let existing = builder.symbols.symbol(symbol_id);
        if existing
            .flags
            .intersects(SymbolFlags::PARAMETER | SymbolFlags::CATCH_VAR)
            && existing.flags.intersects(excludes)
        {
            builder.report_redeclaration(
                name,
                node,
                builder.symbols.first_decl_of(symbol_id),
                existing.flags,
            );
        }
    }

    fn check_hoisting_var_conflict(
        builder: &mut SemanticBuilder<'_, '_>,
        node: NodeId,
        name: &[u8],
        binding: PendingBinding,
    ) {
        let target = binding.scope;
        // Per 14.2.1, a hoisting var conflicts with block-scoped names
        // in any intermediate block it passes through.
        for scope_id in builder.scopes.ancestors(Some(builder.scope)) {
            if scope_id == target {
                break;
            }
            let Some(symbol_id) = builder.symbols.own_binding(scope_id, name) else {
                continue;
            };
            let existing = builder.symbols.symbol(symbol_id);
            if existing.flags.is_block_scoped_like() {
                builder.report_redeclaration(
                    name,
                    node,
                    builder.symbols.first_decl_of(symbol_id),
                    existing.flags,
                );
                break;
            }
            if existing.flags.contains(SymbolFlags::CATCH_VAR) {
                // https://tc39.es/ecma262/#sec-variablestatements-in-catch-blocks
                // (14.15.1): a var colliding with a catch parameter is a
                // syntax error, except for the Annex B.3.4 simple-binding
                // web-compat carve-out.
                let catch_node = builder.scopes.get(scope_id).node;
                let NodeKindData::CatchClause(catch) = builder.ast.kind_data(catch_node) else {
                    unreachable!()
                };
                let simple_parameter = match catch.param(builder.ast) {
                    Some(parameter) => {
                        builder.ast.node_kind(parameter.node_id()) == NodeKind::BindingIdentifier
                    }
                    None => false,
                };
                if !simple_parameter {
                    builder.report_redeclaration(
                        name,
                        node,
                        builder.symbols.first_decl_of(symbol_id),
                        existing.flags,
                    );
                    break;
                }
            }
        }
    }

    fn find_formal_parameters(
        builder: &SemanticBuilder<'_, '_>,
    ) -> Option<swc_next_ecma_ast::FormalParameters> {
        for ancestor in builder.ancestors() {
            match builder.ast.kind_data(ancestor) {
                NodeKindData::FormalParameters(parameters) => {
                    return Some(parameters);
                }
                NodeKindData::Program(_)
                | NodeKindData::Function(_)
                | NodeKindData::ArrowFunctionExpression(_) => {
                    return None;
                }
                _ => {}
            }
        }
        None
    }

    /// Walks a destructuring assignment target to find `eval`/`arguments` references.
    pub(super) fn check_assign_target_eval_arguments(
        builder: &mut SemanticBuilder<'_, '_>,
        node: NodeId,
    ) {
        if !builder.check_syntax.strict_assignment_targets() {
            return;
        }
        let tree = builder.ast;
        match tree.kind_data(node) {
            NodeKindData::IdentifierReference(identifier) => {
                let name = tree.get_utf8(identifier.name(tree)).as_bytes();
                if ecmascript::is_eval_or_arguments(name) {
                    builder.report(
                        tree.span(node),
                        "Cannot assign to 'eval' or 'arguments' in strict mode",
                        ReportOptions::default(),
                    );
                }
            }
            NodeKindData::ArrayPattern(array) => {
                for element in array.elements(tree).iter() {
                    if let Some(element) = tree.get_node_in_sub_range(element) {
                        check_assign_target_eval_arguments(builder, element.node_id());
                    }
                }
                if let Some(rest) = array.rest(tree) {
                    check_assign_target_eval_arguments(builder, rest.node_id());
                }
            }
            NodeKindData::ObjectPattern(object) => {
                for property in object.properties(tree).iter() {
                    let property = tree.get_node_in_sub_range(property);
                    check_assign_target_eval_arguments(builder, property.value(tree).node_id());
                }
                if let Some(rest) = object.rest(tree) {
                    check_assign_target_eval_arguments(builder, rest.node_id());
                }
            }
            NodeKindData::AssignmentPattern(pattern) => {
                check_assign_target_eval_arguments(builder, pattern.left(tree).node_id());
            }
            NodeKindData::BindingRestElement(rest) => {
                check_assign_target_eval_arguments(builder, rest.argument(tree).node_id());
            }
            _ => {}
        }
    }

    // Expressions and classes

    /// 15.7.7 AllPrivateIdentifiersValid. Walks ancestor ClassBody nodes,
    /// collecting PrivateBoundIdentifiers at each level.
    pub(super) fn is_private_name_declared(builder: &SemanticBuilder<'_, '_>, name: &[u8]) -> bool {
        for ancestor in builder.ancestors() {
            let NodeKindData::ClassBody(body) = builder.ast.kind_data(ancestor) else {
                continue;
            };
            for child in body.body(builder.ast).iter() {
                let child = builder.ast.get_node_in_sub_range(child).node_id();
                if ecmascript::private_element_name(builder.ast, child) == Some(name) {
                    return true;
                }
            }
        }
        false
    }

    #[derive(Clone, Copy)]
    pub(super) enum SuperCallValidity {
        Valid,
        NotInConstructor,
        NoExtends,
    }

    /// `super()` is permitted only inside a constructor of a derived class.
    /// Arrow functions are transparent (Section 8.5.1); regular functions are opaque.
    pub(super) fn super_call_validity(builder: &SemanticBuilder<'_, '_>) -> SuperCallValidity {
        let tree = builder.ast;
        let mut child = None;
        let mut ancestors = builder.ancestors();
        while let Some(ancestor) = ancestors.next() {
            match tree.node_kind(ancestor) {
                NodeKind::ArrowFunctionExpression => {}
                NodeKind::Function => {
                    if let Some(parent) = ancestors.next()
                        && let NodeKindData::MethodDefinition(method) = tree.kind_data(parent)
                        && method.kind(tree) == swc_next_ecma_ast::MethodDefinitionKind::Constructor
                    {
                        for class in ancestors {
                            if let NodeKindData::Class(class) = tree.kind_data(class) {
                                return if class.super_class(tree).is_some() {
                                    SuperCallValidity::Valid
                                } else {
                                    SuperCallValidity::NoExtends
                                };
                            }
                        }
                    }
                    return SuperCallValidity::NotInConstructor;
                }
                NodeKind::PropertyDefinition => {
                    if let NodeKindData::PropertyDefinition(property) = tree.kind_data(ancestor)
                        && let Some(value) = property.value(tree)
                        && Some(value.node_id()) == child
                    {
                        return SuperCallValidity::NotInConstructor;
                    }
                }
                NodeKind::StaticBlock | NodeKind::Program => {
                    return SuperCallValidity::NotInConstructor;
                }
                _ => {}
            }
            child = Some(ancestor);
        }
        SuperCallValidity::NotInConstructor
    }

    /// `super.property` is valid in class methods, object methods, field
    /// initializers, static blocks, and arrows inheriting from those contexts.
    pub(super) fn is_super_property_valid(builder: &SemanticBuilder<'_, '_>) -> bool {
        let mut ancestors = builder.ancestors();
        while let Some(ancestor) = ancestors.next() {
            match builder.ast.node_kind(ancestor) {
                NodeKind::ArrowFunctionExpression => {}
                NodeKind::Function => {
                    if let Some(parent) = ancestors.next() {
                        match builder.ast.kind_data(parent) {
                            NodeKindData::MethodDefinition(_) => return true,
                            NodeKindData::ObjectProperty(property)
                                if property.method(builder.ast)
                                    || property.kind(builder.ast)
                                        != swc_next_ecma_ast::PropertyKind::Init =>
                            {
                                return true;
                            }
                            _ => {}
                        }
                    }
                    return false;
                }
                NodeKind::PropertyDefinition | NodeKind::StaticBlock => return true,
                NodeKind::Program => return false,
                _ => {}
            }
        }
        false
    }

    pub(super) fn check_class_member_name(builder: &mut SemanticBuilder<'_, '_>, node: NodeId) {
        let tree = builder.ast;
        let (key, computed, is_static) = match tree.kind_data(node) {
            NodeKindData::PropertyDefinition(property) => (
                property.key(tree),
                property.computed(tree),
                property.r#static(tree),
            ),
            NodeKindData::MethodDefinition(method) => (
                method.key(tree),
                method.computed(tree),
                method.r#static(tree),
            ),
            NodeKindData::TsMethodDefinition(method) => (
                method.key(tree),
                method.computed(tree),
                method.r#static(tree),
            ),
            _ => return,
        };
        if computed || !is_static || in_ts_syntactic_ambient_context(builder) {
            return;
        }
        let name = match tree.kind_data(key.node_id()) {
            NodeKindData::IdentifierName(identifier) => {
                tree.get_utf8(identifier.name(tree)).as_bytes()
            }
            NodeKindData::StringLiteral(literal) => tree.get_wtf8(literal.value(tree)).as_bytes(),
            _ => return,
        };
        if name == b"prototype" {
            builder.report(
                tree.span(key.node_id()),
                "Classes may not have a static property named 'prototype'",
                ReportOptions::default(),
            );
        }
    }

    // Statements and functions

    pub(super) fn await_expression_error(
        builder: &SemanticBuilder<'_, '_>,
    ) -> Option<&'static str> {
        const OUTSIDE_ASYNC: &str = "Await expressions are only allowed within async functions and at the top level of modules";

        let tree = builder.ast;
        let mut child = builder.current_node()?;
        for ancestor in builder.ancestors().skip(1) {
            match tree.kind_data(ancestor) {
                NodeKindData::FormalParameters(_) => {
                    return Some("Await expression is not allowed in formal parameters");
                }
                NodeKindData::StaticBlock(_) => {
                    return Some("Cannot use await in class static initialization block");
                }
                NodeKindData::TsEnumDeclaration(_)
                | NodeKindData::TsModuleBlock(_)
                | NodeKindData::TsModuleDeclaration(_) => return Some(OUTSIDE_ASYNC),
                NodeKindData::PropertyDefinition(property) => {
                    if let Some(value) = property.value(tree)
                        && value.node_id() == child
                    {
                        return Some(OUTSIDE_ASYNC);
                    }
                }
                NodeKindData::Function(function) => {
                    return (!function.r#async(tree)).then_some(OUTSIDE_ASYNC);
                }
                NodeKindData::ArrowFunctionExpression(function) => {
                    return (!function.r#async(tree)).then_some(OUTSIDE_ASYNC);
                }
                NodeKindData::Program(_) => {
                    let is_module = tree.source_type() == swc_next_ecma_ast::SourceType::Module;
                    return (!is_module).then_some(OUTSIDE_ASYNC);
                }
                _ => {}
            }
            child = ancestor;
        }
        Some(OUTSIDE_ASYNC)
    }

    pub(super) fn yield_expression_error(
        builder: &SemanticBuilder<'_, '_>,
    ) -> Option<&'static str> {
        const OUTSIDE_GENERATOR: &str = "A yield expression is only allowed in a generator body";

        let tree = builder.ast;
        let mut child = builder.current_node()?;
        for ancestor in builder.ancestors().skip(1) {
            match tree.kind_data(ancestor) {
                NodeKindData::FormalParameters(_) => {
                    return Some("Yield expression is not allowed in formal parameters");
                }
                NodeKindData::TsEnumDeclaration(_)
                | NodeKindData::TsModuleBlock(_)
                | NodeKindData::TsModuleDeclaration(_)
                | NodeKindData::StaticBlock(_)
                | NodeKindData::ArrowFunctionExpression(_)
                | NodeKindData::Program(_) => return Some(OUTSIDE_GENERATOR),
                NodeKindData::PropertyDefinition(property) => {
                    if let Some(value) = property.value(tree)
                        && value.node_id() == child
                    {
                        return Some(OUTSIDE_GENERATOR);
                    }
                }
                NodeKindData::Function(function) => {
                    return (!function.generator(tree)).then_some(OUTSIDE_GENERATOR);
                }
                _ => {}
            }
            child = ancestor;
        }
        Some(OUTSIDE_GENERATOR)
    }

    // True when the nearest enclosing boundary is a static block.
    pub(super) fn is_inside_static_block(builder: &SemanticBuilder<'_, '_>) -> bool {
        for ancestor in builder.ancestors() {
            match builder.ast.node_kind(ancestor) {
                NodeKind::StaticBlock => return true,
                NodeKind::Function | NodeKind::ArrowFunctionExpression => return false,
                _ => {}
            }
        }
        false
    }

    pub(super) fn is_at_program_level(builder: &SemanticBuilder<'_, '_>) -> bool {
        let Some(parent) = builder.parent() else {
            return false;
        };
        builder.ast.node_kind(parent) == NodeKind::Program
    }

    /// 14.7.5.1 ForBinding must contain exactly one BoundName and, outside
    /// Annex B.3.5, may not have an Initializer.
    pub(super) fn check_for_in_of_initializer(
        builder: &mut SemanticBuilder<'_, '_>,
        left: NodeId,
        is_for_in: bool,
    ) {
        if !builder.check_syntax.for_in_of_initializers() {
            return;
        }
        let NodeKindData::VariableDeclaration(declaration) = builder.ast.kind_data(left) else {
            return;
        };
        let declarators = declaration.declarators(builder.ast);
        if declarators.len() > 1 {
            builder.report(
                builder.ast.span(left),
                "Only a single variable declaration is allowed in a for-in/of statement",
                ReportOptions::default(),
            );
            return;
        }
        for declarator in declarators.iter() {
            let declarator = builder.ast.get_node_in_sub_range(declarator);
            if declarator.init(builder.ast).is_none() {
                continue;
            }
            let annex_b = is_for_in
                && !builder.ast.is_ts()
                && declaration.kind(builder.ast) == VariableKind::Var
                && !builder.is_strict()
                && builder.ast.node_kind(declarator.id(builder.ast).node_id())
                    == NodeKind::BindingIdentifier;
            if annex_b {
                continue;
            }
            builder.report(
                builder.ast.span(declarator.node_id()),
                "for-in/of loop variable declaration may not have an initializer",
                ReportOptions::default(),
            );
            return;
        }
    }

    pub(super) fn is_inside_breakable(builder: &SemanticBuilder<'_, '_>) -> bool {
        for ancestor in builder.ancestors() {
            let kind = builder.ast.node_kind(ancestor);
            if ecmascript::is_iteration(kind) || kind == NodeKind::SwitchStatement {
                return true;
            }
            if ecmascript::is_function_boundary(kind) {
                return false;
            }
        }
        false
    }

    pub(super) fn is_inside_iteration(builder: &SemanticBuilder<'_, '_>) -> bool {
        for ancestor in builder.ancestors() {
            let kind = builder.ast.node_kind(ancestor);
            if ecmascript::is_iteration(kind) {
                return true;
            }
            if ecmascript::is_function_boundary(kind) {
                return false;
            }
        }
        false
    }

    #[derive(Clone, Copy, Eq, PartialEq)]
    pub(super) enum LabelSearch {
        Found,
        NotFound,
        CrossedBoundary,
    }

    pub(super) fn find_label(builder: &SemanticBuilder<'_, '_>, name: &[u8]) -> LabelSearch {
        debug_assert!(!name.is_empty());
        let mut crossed_boundary = false;
        for ancestor in builder.ancestors() {
            match builder.ast.kind_data(ancestor) {
                NodeKindData::LabeledStatement(statement) => {
                    let label = builder
                        .ast
                        .get_utf8(statement.label(builder.ast).name(builder.ast))
                        .as_bytes();
                    if label == name {
                        return if crossed_boundary {
                            LabelSearch::CrossedBoundary
                        } else {
                            LabelSearch::Found
                        };
                    }
                }
                NodeKindData::Function(_)
                | NodeKindData::ArrowFunctionExpression(_)
                | NodeKindData::StaticBlock(_) => crossed_boundary = true,
                _ => {}
            }
        }
        LabelSearch::NotFound
    }

    #[derive(Clone, Copy, Eq, PartialEq)]
    pub(super) enum ContinueLabelSearch {
        Found,
        NotFound,
        CrossedBoundary,
        NotIteration,
    }

    pub(super) fn find_label_for_continue(
        builder: &SemanticBuilder<'_, '_>,
        name: &[u8],
    ) -> ContinueLabelSearch {
        debug_assert!(!name.is_empty());
        let mut crossed_boundary = false;
        for ancestor in builder.ancestors() {
            match builder.ast.kind_data(ancestor) {
                NodeKindData::LabeledStatement(statement) => {
                    let label = builder
                        .ast
                        .get_utf8(statement.label(builder.ast).name(builder.ast))
                        .as_bytes();
                    if label == name {
                        if crossed_boundary {
                            return ContinueLabelSearch::CrossedBoundary;
                        }
                        let body = builder.ast.node_kind(statement.body(builder.ast).node_id());
                        return if ecmascript::is_iteration(body)
                            || body == NodeKind::LabeledStatement
                        {
                            ContinueLabelSearch::Found
                        } else {
                            ContinueLabelSearch::NotIteration
                        };
                    }
                }
                NodeKindData::Function(_)
                | NodeKindData::ArrowFunctionExpression(_)
                | NodeKindData::StaticBlock(_) => crossed_boundary = true,
                _ => {}
            }
        }
        ContinueLabelSearch::NotFound
    }

    // Imports, exports, and namespaces

    /// Post-traversal check that all local export specifiers refer to
    /// declared bindings.
    pub(super) fn check_unresolved_exports(builder: &mut SemanticBuilder<'_, '_>) {
        if !builder.check_syntax.unresolved_exports() {
            return;
        }
        let tree = builder.ast;
        if !tree.is_module() {
            return;
        }
        for index in 0..builder.export_specifiers.len() {
            let specifier = builder.export_specifiers[index];
            let name = ecmascript::module_export_name(tree, specifier.local(tree).node_id());
            if builder.symbols.binding(ScopeId::MODULE, name).is_none() {
                builder.report_unresolved_export(name, specifier.node_id());
            }
        }
    }

    pub(super) fn check_import_export_position(
        builder: &mut SemanticBuilder<'_, '_>,
        node: NodeId,
        out_of_module_label: &'static str,
        top_level_label: &'static str,
    ) {
        if !builder.check_syntax.import_export_position() {
            return;
        }
        let Some(parent) = builder.parent() else {
            return;
        };
        let message = match builder.ast.node_kind(parent) {
            NodeKind::Program if !builder.ast.is_module() => Some(format!(
                "Cannot use {} outside a module",
                out_of_module_label
            )),
            NodeKind::Program | NodeKind::TsModuleBlock => None,
            _ => Some(format!(
                "{} may only appear at the top level",
                top_level_label
            )),
        };
        if let Some(message) = message {
            builder.report(builder.ast.span(node), message, ReportOptions::default());
        }
    }

    pub(super) fn check_duplicate_with_attributes(
        builder: &mut SemanticBuilder<'_, '_>,
        attributes: swc_next_ecma_ast::SubRange<swc_next_ecma_ast::ImportAttribute>,
    ) {
        if !builder.check_syntax.duplicate_import_attributes() {
            return;
        }
        let tree = builder.ast;
        for (index, attribute) in attributes.iter().enumerate() {
            let attribute = tree.get_node_in_sub_range(attribute);
            let key = attribute.key(tree).node_id();
            let Some(key_name) = ecmascript::import_attribute_name(tree, key) else {
                continue;
            };
            for previous in attributes.iter().take(index) {
                let previous = tree.get_node_in_sub_range(previous);
                let previous_key = previous.key(tree).node_id();
                let Some(previous_key_name) = ecmascript::import_attribute_name(tree, previous_key)
                else {
                    continue;
                };
                if key_name != previous_key_name {
                    continue;
                }
                let message = format!(
                    "Duplicate import attribute key '{}'",
                    String::from_utf8_lossy(key_name)
                );
                builder.report(
                    tree.span(key),
                    message,
                    ReportOptions {
                        labels: vec![builder.label(tree.span(previous_key), "first used here")],
                        ..ReportOptions::default()
                    },
                );
                break;
            }
        }
    }

    /// Records an exported name and reports a duplicate if one already exists.
    pub(super) fn record_exported_name<'traverse>(
        builder: &mut SemanticBuilder<'traverse, '_>,
        name: &'traverse [u8],
        node: NodeId,
    ) {
        if !builder.check_syntax.duplicate_exports() {
            return;
        }
        // TypeScript declarations can merge across the type and value namespaces. Duplicate
        // exports are therefore checker diagnostics rather than parser-level early errors.
        if builder.ast.is_ts() {
            return;
        }
        // Exports inside a TS namespace are namespace-scoped, not module-scoped.
        if !builder.ast.is_module() || builder.ts_namespace_depth > 0 {
            return;
        }
        let first = match builder.exported_names.entry(name) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                entry.insert(node);
                return;
            }
        };
        let message = format!("Duplicate export of '{}'", String::from_utf8_lossy(name));
        builder.report(
            builder.ast.span(node),
            message,
            ReportOptions {
                labels: vec![
                    builder.label(builder.ast.span(first), "first exported here"),
                    builder.label(builder.ast.span(node), "exported again here"),
                ],
                ..ReportOptions::default()
            },
        );
    }

    pub(super) fn ts_module_declaration_context_error(
        builder: &SemanticBuilder<'_, '_>,
        declaration: TsModuleDeclaration,
    ) -> Option<(NodeId, &'static str)> {
        let (parent, ancestors) = ts_module_element_context(builder)?;
        let parent_kind = builder.ast.node_kind(parent);
        let quoted = matches!(
            builder.ast.ts_module_name_data(declaration.id(builder.ast)),
            TsModuleNameData::StringLiteral(_)
        );
        if !ecmascript::is_ts_module_element_container(parent_kind) {
            let message = if quoted {
                "An ambient module declaration is only allowed at the top level in a file"
            } else {
                "A namespace declaration is only allowed at the top level of a namespace or module"
            };
            return Some((declaration.node_id(), message));
        }
        if quoted
            && parent_kind != NodeKind::Program
            && !ecmascript::is_ts_module_augmentation_external(builder.ast, parent, ancestors)
        {
            return Some((
                declaration.id(builder.ast).node_id(),
                "Ambient modules cannot be nested in other modules or namespaces",
            ));
        }
        None
    }

    pub(super) fn ts_global_declaration_context_error(
        builder: &SemanticBuilder<'_, '_>,
        declaration: TsGlobalDeclaration,
    ) -> Option<(NodeId, &'static str)> {
        let (parent, ancestors) = ts_module_element_context(builder)?;
        let parent_kind = builder.ast.node_kind(parent);
        if !ecmascript::is_ts_module_element_container(parent_kind) {
            return Some((
                declaration.node_id(),
                "An ambient module declaration is only allowed at the top level in a file",
            ));
        }

        if ecmascript::is_ts_module_augmentation_external(builder.ast, parent, ancestors) {
            return None;
        }
        Some((
            declaration.id(builder.ast).node_id(),
            "Augmentations for the global scope can only be directly nested in external modules or ambient module declarations",
        ))
    }

    pub(super) fn in_ts_syntactic_ambient_context(builder: &SemanticBuilder<'_, '_>) -> bool {
        let tree = builder.ast;
        if tree.lang() == Lang::Dts {
            return true;
        }
        // Binding recovery also makes bare global bodies ambient. Only syntax can
        // exempt an inner augmentation from requiring its own declare modifier.
        for node in builder.ancestors().skip(1) {
            let declared = match tree.kind_data(node) {
                NodeKindData::VariableDeclaration(node) => node.declare(tree),
                NodeKindData::Class(node) => node.declare(tree),
                NodeKindData::TsFunction(node) => node.declare(tree),
                NodeKindData::TsTypeAliasDeclaration(node) => node.declare(tree),
                NodeKindData::TsInterfaceDeclaration(node) => node.declare(tree),
                NodeKindData::TsEnumDeclaration(node) => node.declare(tree),
                NodeKindData::TsModuleDeclaration(node) => node.declare(tree),
                NodeKindData::TsGlobalDeclaration(node) => node.declare(tree),
                _ => false,
            };
            if declared {
                return true;
            }
        }
        false
    }

    fn ts_module_element_context<'b>(
        builder: &'b SemanticBuilder<'_, '_>,
    ) -> Option<(NodeId, impl Iterator<Item = NodeId> + 'b)> {
        let mut ancestors = builder.ancestors().skip(1);
        let mut parent = ancestors.next()?;
        if builder.ast.node_kind(parent) == NodeKind::ExportNamedDeclaration {
            parent = ancestors.next()?;
        }
        Some((parent, ancestors))
    }
}
