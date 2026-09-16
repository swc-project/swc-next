use swc_next_ecma_ast::{
    AwaitExpression, BinaryExpression, BreakStatement, ContinueStatement, LabelIdentifier,
    LabeledStatement, MetaProperty, NumericLiteral, ObjectExpression, RegExpLiteral, StringLiteral,
    YieldExpression,
};

use super::*;

pub(super) fn check_string_literal(builder: &mut SemanticBuilder<'_, '_>, literal: StringLiteral) {
    if builder.context.state.in_type_position() || !builder.context.state.scope.is_strict() {
        return;
    }
    let span = builder.context.ast.span(literal.node_id());
    if has_octal_escape(
        &builder.context.ast.source().as_bytes()[span.start as usize..span.end as usize],
    ) {
        builder.report(
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

pub(super) fn check_numeric_literal(
    builder: &mut SemanticBuilder<'_, '_>,
    literal: NumericLiteral,
) {
    if builder.context.state.in_type_position() || !builder.context.state.scope.is_strict() {
        return;
    }
    let span = builder.context.ast.span(literal.node_id());
    let raw = &builder.context.ast.source().as_bytes()[span.start as usize..span.end as usize];
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
    builder.report(
        span,
        message,
        ReportOptions {
            help: Some(Cow::Borrowed(help)),
            ..ReportOptions::default()
        },
    );
}

pub(super) fn check_reg_exp_literal(builder: &mut SemanticBuilder<'_, '_>, literal: RegExpLiteral) {
    let pattern = builder
        .context
        .ast
        .get_utf8(literal.pattern(builder.context.ast));
    let flags = builder
        .context
        .ast
        .get_utf8(literal.flags(builder.context.ast));
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
        builder.report(
            builder.context.ast.span(literal.node_id()),
            "Invalid regular expression literal",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_identifier_reference(
    builder: &mut SemanticBuilder<'_, '_>,
    identifier: IdentifierReference,
) {
    if builder.context.state.in_type_position() {
        return;
    }
    let node_id = identifier.node_id();
    let name = builder
        .context
        .ast
        .get_utf8(identifier.name(builder.context.ast))
        .as_bytes();
    builder.check_strict_reserved(name, node_id, "an identifier");
    if name == b"arguments" && !is_arguments_available(&builder.context) {
        builder.report(
            builder.context.ast.span(node_id),
            "'arguments' is not allowed in class field initializers or static blocks",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_binding_identifier(
    builder: &mut SemanticBuilder<'_, '_>,
    identifier: BindingIdentifier,
) -> Option<PreparedDeclaration> {
    let node_id = identifier.node_id();
    let name = builder
        .context
        .ast
        .get_utf8(identifier.name(builder.context.ast))
        .as_bytes();
    let flags = builder.context.state.symbols.pending.flags;
    if flags.is_empty() {
        return None;
    }
    if !builder.context.state.in_type_position()
        && builder.context.state.scope.is_strict()
        && !flags.contains(SymbolFlags::AMBIENT)
    {
        builder.check_strict_reserved(name, node_id, "a binding identifier");
        if is_eval_or_arguments(name) {
            let message = builder.fmt(format_args!(
                "'{}' is not allowed as a binding identifier in strict mode",
                String::from_utf8_lossy(name)
            ));
            builder.report(
                builder.context.ast.span(node_id),
                message,
                ReportOptions::default(),
            );
        }
    } else if flags.contains(SymbolFlags::BLOCK_SCOPED_VAR) && name == b"let" {
        builder.report(
            builder.context.ast.span(node_id),
            "'let' is not allowed as a variable name in a lexical declaration",
            ReportOptions::default(),
        );
    }
    let prepared = builder.context.state.symbols.prepare_declaration(
        builder.context.ast,
        identifier.name(builder.context.ast).into(),
        node_id,
    );
    builder.check_redeclaration(node_id, name, prepared);
    if builder.context.state.symbols.export_state.named().is_some() {
        builder.record_exported_name(name, node_id);
    }
    Some(prepared)
}

pub(super) fn check_label_identifier(
    builder: &mut SemanticBuilder<'_, '_>,
    identifier: LabelIdentifier,
) {
    let name = builder
        .context
        .ast
        .get_utf8(identifier.name(builder.context.ast))
        .as_bytes();
    builder.check_strict_reserved(name, identifier.node_id(), "a label");
}

impl<'traverse, 'a> SemanticBuilder<'traverse, 'a> {
    /// <https://tc39.es/ecma262/#sec-keywords-and-reserved-words>
    fn check_strict_reserved(&mut self, name: &[u8], node: NodeId, as_what: &'static str) {
        if !self.context.state.scope.is_strict() {
            return;
        }
        if let Some(word) = match_strict_reserved(name) {
            let message = self.fmt(format_args!(
                "'{}' is reserved in strict mode and cannot be used as {}",
                word, as_what
            ));
            self.report(
                self.context.ast.span(node),
                message,
                ReportOptions::default(),
            );
        }
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

fn is_eval_or_arguments(name: &[u8]) -> bool {
    matches!(name, b"eval" | b"arguments")
}

fn is_eval_or_arguments_reference(tree: &Ast<'_>, node: NodeId) -> bool {
    let NodeKindData::IdentifierReference(identifier) = tree.kind_data(node) else {
        return false;
    };
    is_eval_or_arguments(tree.get_utf8(identifier.name(tree)).as_bytes())
}

/// Leading zero followed by digits: legacy octal (`077`) or
/// leading-zero decimal (`089`).
fn is_legacy_numeric_literal(raw: &[u8]) -> bool {
    raw.len() >= 2 && raw[0] == b'0' && raw[1].is_ascii_digit()
}

fn has_octal_escape(value: &[u8]) -> bool {
    let mut position = 0;
    while let Some(offset) = memchr::memchr(b'\\', &value[position..]) {
        let slash = position + offset;
        let next = slash + 1;
        if next >= value.len() {
            break;
        }
        match value[next] {
            b'1'..=b'9' => return true,
            b'0' if value.get(next + 1).is_some_and(u8::is_ascii_digit) => return true,
            _ => {}
        }
        position = next + 1;
    }
    false
}

pub(super) fn check_binary_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: BinaryExpression,
) {
    let left = expression.left(builder.context.ast).node_id();
    if expression.operator(builder.context.ast) == BinaryOperator::In
        && let NodeKindData::PrivateIdentifier(identifier) = builder.context.ast.kind_data(left)
    {
        let name = builder
            .context
            .ast
            .get_utf8(identifier.name(builder.context.ast))
            .as_bytes();
        if !is_private_name_declared(&builder.context, name) {
            let message = builder.fmt(format_args!(
                "Private field '#{}' must be declared in an enclosing class",
                String::from_utf8_lossy(name)
            ));
            builder.report(
                builder.context.ast.span(left),
                message,
                ReportOptions::default(),
            );
        }
    }
}

pub(super) fn check_unary_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: UnaryExpression,
) {
    if expression.operator(builder.context.ast) != UnaryOperator::Delete {
        return;
    }
    let mut target = unwrap_parentheses(
        builder.context.ast,
        expression.argument(builder.context.ast).node_id(),
    );
    if let NodeKindData::ChainExpression(chain) = builder.context.ast.kind_data(target) {
        target = unwrap_parentheses(
            builder.context.ast,
            chain.expression(builder.context.ast).node_id(),
        );
    }
    match builder.context.ast.kind_data(target) {
        NodeKindData::IdentifierReference(_) if builder.context.state.scope.is_strict() => {
            builder.report(
                builder.context.ast.span(expression.node_id()),
                "Deleting a variable in strict mode is not allowed",
                ReportOptions::default(),
            );
        }
        NodeKindData::MemberExpression(member)
            if !member.computed(builder.context.ast)
                && builder
                    .context
                    .ast
                    .node_kind(member.property(builder.context.ast).node_id())
                    == NodeKind::PrivateIdentifier =>
        {
            builder.report(
                builder.context.ast.span(expression.node_id()),
                "Private fields cannot be deleted",
                ReportOptions::default(),
            );
        }
        _ => {}
    }
}

pub(super) fn check_update_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: UpdateExpression,
) {
    if builder.context.state.scope.is_strict()
        && is_eval_or_arguments_reference(
            builder.context.ast,
            expression.argument(builder.context.ast).node_id(),
        )
    {
        builder.report(
            builder.context.ast.span(expression.node_id()),
            "Cannot assign to 'eval' or 'arguments' in strict mode",
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_assignment_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: AssignmentExpression,
) {
    if builder.context.state.scope.is_strict() {
        builder.check_assign_target_eval_arguments(expression.left(builder.context.ast).node_id());
    }
}

pub(super) fn check_object_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    object: ObjectExpression,
) {
    let mut first_proto = None;
    for property in object.properties(builder.context.ast).iter() {
        let property = builder
            .context
            .ast
            .get_node_in_sub_range(property)
            .node_id();
        let NodeKindData::ObjectProperty(item) = builder.context.ast.kind_data(property) else {
            continue;
        };
        if item.computed(builder.context.ast)
            || item.method(builder.context.ast)
            || item.shorthand(builder.context.ast)
            || item.kind(builder.context.ast) != swc_next_ecma_ast::PropertyKind::Init
        {
            continue;
        }
        let name = match builder
            .context
            .ast
            .kind_data(item.key(builder.context.ast).node_id())
        {
            NodeKindData::IdentifierName(identifier) => builder
                .context
                .ast
                .get_utf8(identifier.name(builder.context.ast))
                .as_bytes(),
            NodeKindData::StringLiteral(string) => builder
                .context
                .ast
                .get_wtf8(string.value(builder.context.ast))
                .as_bytes(),
            _ => continue,
        };
        if name != b"__proto__" {
            continue;
        }
        if let Some(first) = first_proto {
            builder.report(
                builder.context.ast.span(property),
                "Duplicate '__proto__' property in object literal",
                ReportOptions {
                    labels: builder.labels(&[
                        builder.label(builder.context.ast.span(first), "first defined here")
                    ]),
                    ..ReportOptions::default()
                },
            );
        } else {
            first_proto = Some(property);
        }
    }
}

pub(super) fn check_member_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: MemberExpression,
) {
    let object = expression.object(builder.context.ast).node_id();
    if builder.context.ast.node_kind(object) == NodeKind::Super
        && !is_super_property_valid(&builder.context)
    {
        builder.report(
            builder.context.ast.span(expression.node_id()),
            "'super' property access is only valid inside a method or class body",
            ReportOptions {
                help: Some(Cow::Borrowed(
                    "Use an arrow function instead of a regular function to inherit the 'super' binding",
                )),
                ..ReportOptions::default()
            },
        );
    }
    let property = expression.property(builder.context.ast).node_id();
    if !expression.computed(builder.context.ast)
        && let NodeKindData::PrivateIdentifier(identifier) = builder.context.ast.kind_data(property)
    {
        let name = builder
            .context
            .ast
            .get_utf8(identifier.name(builder.context.ast))
            .as_bytes();
        if builder.context.ast.node_kind(object) == NodeKind::Super {
            builder.report(
                builder.context.ast.span(expression.node_id()),
                "'super' keyword unexpected here",
                ReportOptions::default(),
            );
        } else if !is_private_name_declared(&builder.context, name) {
            let message = builder.fmt(format_args!(
                "Private field '#{}' must be declared in an enclosing class",
                String::from_utf8_lossy(name)
            ));
            builder.report(
                builder.context.ast.span(property),
                message,
                ReportOptions::default(),
            );
        }
    }
}

pub(super) fn check_call_expression(builder: &mut SemanticBuilder<'_, '_>, call: CallExpression) {
    if builder
        .context
        .ast
        .node_kind(call.callee(builder.context.ast).node_id())
        == NodeKind::Super
    {
        let message = "'super()' is only valid in a constructor of a derived class";
        match super_call_validity(&builder.context) {
            SuperCallValidity::Valid => {}
            SuperCallValidity::NotInConstructor => builder.report(
                builder.context.ast.span(call.node_id()),
                message,
                ReportOptions {
                    help: Some(Cow::Borrowed(
                        "Use an arrow function instead of a regular function to inherit the 'super' binding",
                    )),
                    ..ReportOptions::default()
                },
            ),
            SuperCallValidity::NoExtends => builder.report(
                builder.context.ast.span(call.node_id()),
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

pub(super) fn check_await_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: AwaitExpression,
) {
    let message = await_expression_error(&builder.context);
    if let Some(message) = message {
        builder.report(
            builder.context.ast.span(expression.node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_yield_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: YieldExpression,
) {
    if let Some(message) = yield_expression_error(&builder.context) {
        builder.report(
            builder.context.ast.span(expression.node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_meta_property(builder: &mut SemanticBuilder<'_, '_>, property: MetaProperty) {
    let name = builder
        .context
        .ast
        .get_utf8(property.meta(builder.context.ast).name(builder.context.ast));
    if name == "import" && !builder.context.ast.is_module() {
        builder.report(
            builder.context.ast.span(property.node_id()),
            "'import.meta' is only valid in module code",
            ReportOptions::default(),
        );
    }
    if name == "new" && !is_new_target_available(&builder.context) {
        builder.report(
            builder.context.ast.span(property.node_id()),
            "'new.target' is only valid inside functions, class field initializers, or static blocks",
            ReportOptions::default(),
        );
    }
}

impl<'traverse, 'a> SemanticBuilder<'traverse, 'a> {
    /// Walks a destructuring assignment target to find `eval`/`arguments` references.
    fn check_assign_target_eval_arguments(&mut self, node: NodeId) {
        match self.context.ast.kind_data(node) {
            NodeKindData::IdentifierReference(identifier) => {
                if is_eval_or_arguments(
                    self.context
                        .ast
                        .get_utf8(identifier.name(self.context.ast))
                        .as_bytes(),
                ) {
                    self.report(
                        self.context.ast.span(node),
                        "Cannot assign to 'eval' or 'arguments' in strict mode",
                        ReportOptions::default(),
                    );
                }
            }
            NodeKindData::ArrayPattern(array) => {
                let elements = array.elements(self.context.ast);
                for index in 0..elements.len() {
                    if let Some(Some(element)) = elements.get_node(self.context.ast, index) {
                        self.check_assign_target_eval_arguments(element.node_id());
                    }
                }
                if let Some(rest) = array.rest(self.context.ast) {
                    self.check_assign_target_eval_arguments(rest.node_id());
                }
            }
            NodeKindData::ObjectPattern(object) => {
                let properties = object.properties(self.context.ast);
                for index in 0..properties.len() {
                    let property = properties
                        .get_node(self.context.ast, index)
                        .expect("object pattern property exists");
                    self.check_assign_target_eval_arguments(
                        property.value(self.context.ast).node_id(),
                    );
                }
                if let Some(rest) = object.rest(self.context.ast) {
                    self.check_assign_target_eval_arguments(rest.node_id());
                }
            }
            NodeKindData::AssignmentPattern(pattern) => {
                self.check_assign_target_eval_arguments(pattern.left(self.context.ast).node_id());
            }
            NodeKindData::BindingRestElement(rest) => {
                self.check_assign_target_eval_arguments(rest.argument(self.context.ast).node_id());
            }
            _ => {}
        }
    }
}

fn unwrap_parentheses(tree: &Ast<'_>, node: NodeId) -> NodeId {
    let mut current = node;
    let mut depth = 0u32;
    // The parser builds at most one parenthesis node per syntactic
    // level, so this chain is bounded by source nesting depth.
    loop {
        debug_assert!(depth < 1_000_000);
        let NodeKindData::ParenthesizedExpression(parenthesized) = tree.kind_data(current) else {
            return current;
        };
        current = parenthesized.expression(tree).node_id();
        depth += 1;
    }
}

fn await_expression_error(context: &SemanticVisitCtx<'_, '_>) -> Option<&'static str> {
    const OUTSIDE_ASYNC: &str =
        "Await expressions are only allowed within async functions and at the top level of modules";

    let mut child = context.current_node()?;
    for ancestor in context.ancestors().skip(1) {
        match context.ast.kind_data(ancestor) {
            NodeKindData::FormalParameters(_) => {
                return Some("Await expression is not allowed in formal parameters");
            }
            NodeKindData::StaticBlock(_) => {
                return Some("Cannot use await in class static initialization block");
            }
            NodeKindData::TsEnumDeclaration(_)
            | NodeKindData::TsModuleBlock(_)
            | NodeKindData::TsModuleDeclaration(_) => return Some(OUTSIDE_ASYNC),
            NodeKindData::PropertyDefinition(property)
                if property
                    .value(context.ast)
                    .is_some_and(|value| value.node_id() == child) =>
            {
                return Some(OUTSIDE_ASYNC);
            }
            NodeKindData::Function(function) => {
                return (!function.r#async(context.ast)).then_some(OUTSIDE_ASYNC);
            }
            NodeKindData::ArrowFunctionExpression(function) => {
                return (!function.r#async(context.ast)).then_some(OUTSIDE_ASYNC);
            }
            NodeKindData::Program(_) => {
                return (context.ast.source_type() != swc_next_ecma_ast::SourceType::Module)
                    .then_some(OUTSIDE_ASYNC);
            }
            _ => {}
        }
        child = ancestor;
    }
    Some(OUTSIDE_ASYNC)
}

fn yield_expression_error(context: &SemanticVisitCtx<'_, '_>) -> Option<&'static str> {
    const OUTSIDE_GENERATOR: &str = "A yield expression is only allowed in a generator body";

    let mut child = context.current_node()?;
    for ancestor in context.ancestors().skip(1) {
        match context.ast.kind_data(ancestor) {
            NodeKindData::FormalParameters(_) => {
                return Some("Yield expression is not allowed in formal parameters");
            }
            NodeKindData::TsEnumDeclaration(_)
            | NodeKindData::TsModuleBlock(_)
            | NodeKindData::TsModuleDeclaration(_)
            | NodeKindData::StaticBlock(_)
            | NodeKindData::ArrowFunctionExpression(_)
            | NodeKindData::Program(_) => return Some(OUTSIDE_GENERATOR),
            NodeKindData::PropertyDefinition(property)
                if property
                    .value(context.ast)
                    .is_some_and(|value| value.node_id() == child) =>
            {
                return Some(OUTSIDE_GENERATOR);
            }
            NodeKindData::Function(function) => {
                return (!function.generator(context.ast)).then_some(OUTSIDE_GENERATOR);
            }
            _ => {}
        }
        child = ancestor;
    }
    Some(OUTSIDE_GENERATOR)
}

pub(super) fn check_variable_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: VariableDeclaration,
) {
    let kind = declaration.kind(builder.context.ast);
    if matches!(kind, VariableKind::Using | VariableKind::AwaitUsing)
        && builder.context.ast.source_type() == swc_next_ecma_ast::SourceType::Script
        && is_at_program_level(&builder.context)
    {
        let message = builder.fmt(format_args!(
            "'{}' is not allowed at the top level of a script",
            kind.as_str()
        ));
        builder.report(
            builder.context.ast.span(declaration.node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_switch_statement(
    builder: &mut SemanticBuilder<'_, '_>,
    statement: SwitchStatement,
) {
    let mut first_default = None;
    for case in statement.cases(builder.context.ast).iter() {
        let case = builder.context.ast.get_node_in_sub_range(case);
        if case.test(builder.context.ast).is_some() {
            continue;
        }
        if let Some(first) = first_default {
            builder.report(
                builder.context.ast.span(case.node_id()),
                "A switch statement can only have one default clause",
                ReportOptions {
                    labels: builder.labels(&[builder.label(
                        builder.context.ast.span(first),
                        "first default defined here",
                    )]),
                    ..ReportOptions::default()
                },
            );
        } else {
            first_default = Some(case.node_id());
        }
    }
}

pub(super) fn check_for_in_statement(
    builder: &mut SemanticBuilder<'_, '_>,
    statement: ForInStatement,
) {
    let left = statement.left(builder.context.ast).node_id();
    builder.check_for_in_of_initializer(left, true);
    if builder.context.state.scope.is_strict() {
        builder.check_assign_target_eval_arguments(left);
    }
}

pub(super) fn check_for_of_statement(
    builder: &mut SemanticBuilder<'_, '_>,
    statement: ForOfStatement,
) {
    if statement.r#await(builder.context.ast)
        && !builder.context.ast.is_ts()
        && is_inside_static_block(&builder.context)
    {
        builder.report(
            builder.context.ast.span(statement.node_id()),
            "Cannot use 'for await' in class static initialization block",
            ReportOptions::default(),
        );
    }
    let left = statement.left(builder.context.ast).node_id();
    builder.check_for_in_of_initializer(left, false);
    if builder.context.state.scope.is_strict() {
        builder.check_assign_target_eval_arguments(left);
    }
}

pub(super) fn check_break_statement(
    builder: &mut SemanticBuilder<'_, '_>,
    statement: BreakStatement,
) {
    if let Some(label) = statement.label(builder.context.ast) {
        let name = builder
            .context
            .ast
            .get_utf8(label.name(builder.context.ast))
            .as_bytes();
        let result = find_label(&builder.context, name);
        if result != LabelSearch::Found {
            let message = match result {
                LabelSearch::NotFound => builder.fmt(format_args!(
                    "Use of undefined label '{}'",
                    String::from_utf8_lossy(name)
                )),
                LabelSearch::CrossedBoundary => builder.fmt(format_args!(
                    "Cannot break to label '{}' across function boundaries",
                    String::from_utf8_lossy(name)
                )),
                LabelSearch::Found => unreachable!(),
            };
            builder.report(
                builder.context.ast.span(statement.node_id()),
                message,
                ReportOptions::default(),
            );
        }
    } else if !is_inside_breakable(&builder.context) {
        builder.report(
            builder.context.ast.span(statement.node_id()),
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
    builder: &mut SemanticBuilder<'_, '_>,
    statement: ContinueStatement,
) {
    if let Some(label) = statement.label(builder.context.ast) {
        let name = builder
            .context
            .ast
            .get_utf8(label.name(builder.context.ast))
            .as_bytes();
        let result = find_label_for_continue(&builder.context, name);
        if result != ContinueLabelSearch::Found {
            let (message, help) = match result {
                ContinueLabelSearch::NotFound => (
                    builder.fmt(format_args!(
                        "Use of undefined label '{}'",
                        String::from_utf8_lossy(name)
                    )),
                    None,
                ),
                ContinueLabelSearch::CrossedBoundary => (
                    builder.fmt(format_args!(
                        "Cannot continue to label '{}' across function boundaries",
                        String::from_utf8_lossy(name)
                    )),
                    None,
                ),
                ContinueLabelSearch::NotIteration => (
                    builder.fmt(format_args!(
                        "Label '{}' does not denote an iteration statement",
                        String::from_utf8_lossy(name)
                    )),
                    Some(Cow::Borrowed(
                        "A 'continue' statement can only jump to a label of an enclosing 'for', 'while', 'do-while', 'for-in', or 'for-of' statement",
                    )),
                ),
                ContinueLabelSearch::Found => unreachable!(),
            };
            builder.report(
                builder.context.ast.span(statement.node_id()),
                message,
                ReportOptions {
                    help,
                    ..ReportOptions::default()
                },
            );
        }
    } else if !is_inside_iteration(&builder.context) {
        builder.report(
            builder.context.ast.span(statement.node_id()),
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

pub(super) fn check_labeled_statement(
    builder: &mut SemanticBuilder<'_, '_>,
    statement: LabeledStatement,
) {
    let name = builder
        .context
        .ast
        .get_utf8(
            statement
                .label(builder.context.ast)
                .name(builder.context.ast),
        )
        .as_bytes();
    let duplicate = {
        let mut ancestors = builder.context.ancestors();
        let _ = ancestors.next();
        let mut duplicate = false;
        for ancestor in ancestors {
            match builder.context.ast.kind_data(ancestor) {
                NodeKindData::LabeledStatement(outer) => {
                    let outer_name = builder
                        .context
                        .ast
                        .get_utf8(outer.label(builder.context.ast).name(builder.context.ast))
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
        let message = builder.fmt(format_args!(
            "Duplicate label '{}'",
            String::from_utf8_lossy(name)
        ));
        builder.report(
            builder
                .context
                .ast
                .span(statement.label(builder.context.ast).node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_with_statement(
    builder: &mut SemanticBuilder<'_, '_>,
    statement: WithStatement,
) {
    if builder.context.state.scope.is_strict() {
        builder.report(
            builder.context.ast.span(statement.node_id()),
            "'with' statements are not allowed in strict mode",
            ReportOptions::default(),
        );
    }
}

impl<'traverse, 'a> SemanticBuilder<'traverse, 'a> {
    /// 14.7.5.1 ForBinding must contain exactly one BoundName and, outside
    /// Annex B.3.5, may not have an Initializer.
    fn check_for_in_of_initializer(&mut self, left: NodeId, is_for_in: bool) {
        let NodeKindData::VariableDeclaration(declaration) = self.context.ast.kind_data(left)
        else {
            return;
        };
        let declarators = declaration.declarators(self.context.ast);
        if declarators.len() > 1 {
            self.report(
                self.context.ast.span(left),
                "Only a single variable declaration is allowed in a for-in/of statement",
                ReportOptions::default(),
            );
            return;
        }
        for declarator in declarators.iter() {
            let declarator = self.context.ast.get_node_in_sub_range(declarator);
            if declarator.init(self.context.ast).is_none() {
                continue;
            }
            let annex_b = is_for_in
                && !self.context.ast.is_ts()
                && declaration.kind(self.context.ast) == VariableKind::Var
                && !self.context.state.scope.is_strict()
                && self
                    .context
                    .ast
                    .node_kind(declarator.id(self.context.ast).node_id())
                    == NodeKind::BindingIdentifier;
            if annex_b {
                continue;
            }
            self.report(
                self.context.ast.span(declarator.node_id()),
                "for-in/of loop variable declaration may not have an initializer",
                ReportOptions::default(),
            );
            return;
        }
    }
}

fn is_function_boundary(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Function | NodeKind::ArrowFunctionExpression | NodeKind::StaticBlock
    )
}

fn is_iteration(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::ForStatement
            | NodeKind::ForInStatement
            | NodeKind::ForOfStatement
            | NodeKind::WhileStatement
            | NodeKind::DoWhileStatement
    )
}

fn is_inside_breakable(context: &SemanticVisitCtx<'_, '_>) -> bool {
    for ancestor in context.ancestors() {
        let kind = context.ast.node_kind(ancestor);
        if is_iteration(kind) || kind == NodeKind::SwitchStatement {
            return true;
        }
        if is_function_boundary(kind) {
            return false;
        }
    }
    false
}

fn is_inside_iteration(context: &SemanticVisitCtx<'_, '_>) -> bool {
    for ancestor in context.ancestors() {
        let kind = context.ast.node_kind(ancestor);
        if is_iteration(kind) {
            return true;
        }
        if is_function_boundary(kind) {
            return false;
        }
    }
    false
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum LabelSearch {
    Found,
    NotFound,
    CrossedBoundary,
}

fn find_label(context: &SemanticVisitCtx<'_, '_>, name: &[u8]) -> LabelSearch {
    debug_assert!(!name.is_empty());
    let mut crossed_boundary = false;
    for ancestor in context.ancestors() {
        match context.ast.kind_data(ancestor) {
            NodeKindData::LabeledStatement(statement) => {
                let label = context
                    .ast
                    .get_utf8(statement.label(context.ast).name(context.ast))
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
enum ContinueLabelSearch {
    Found,
    NotFound,
    CrossedBoundary,
    NotIteration,
}

fn find_label_for_continue(context: &SemanticVisitCtx<'_, '_>, name: &[u8]) -> ContinueLabelSearch {
    debug_assert!(!name.is_empty());
    let mut crossed_boundary = false;
    for ancestor in context.ancestors() {
        match context.ast.kind_data(ancestor) {
            NodeKindData::LabeledStatement(statement) => {
                let label = context
                    .ast
                    .get_utf8(statement.label(context.ast).name(context.ast))
                    .as_bytes();
                if label == name {
                    if crossed_boundary {
                        return ContinueLabelSearch::CrossedBoundary;
                    }
                    let body = context.ast.node_kind(statement.body(context.ast).node_id());
                    return if is_iteration(body) || body == NodeKind::LabeledStatement {
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

pub(super) fn check_function(builder: &mut SemanticBuilder<'_, '_>, function: Function) {
    if function.function_type(builder.context.ast) != FunctionType::FunctionDeclaration {
        return;
    }
    let strict = builder.context.state.scope.is_strict();
    let mut through_labels = false;
    let mut ancestors = builder.context.ancestors();
    let _ = ancestors.next();
    let parent = loop {
        let Some(parent) = ancestors.next() else {
            return;
        };
        if builder.context.ast.node_kind(parent) != NodeKind::LabeledStatement {
            break builder.context.ast.node_kind(parent);
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
            builder.context.ast.span(function.node_id()),
            message,
            ReportOptions::default(),
        );
    }
}

pub(super) fn check_directive(builder: &mut SemanticBuilder<'_, '_>, directive: Directive) {
    if builder
        .context
        .ast
        .get_utf8(directive.value(builder.context.ast))
        != "use strict"
    {
        return;
    }
    let mut parameter = None;
    for node in builder.context.ancestors() {
        match builder.context.ast.kind_data(node) {
            NodeKindData::Function(function) => {
                parameter = ecmascript::find_non_simple_parameter(
                    builder.context.ast,
                    function.params(builder.context.ast),
                );
                break;
            }
            NodeKindData::ArrowFunctionExpression(arrow) => {
                parameter = ecmascript::find_non_simple_parameter(
                    builder.context.ast,
                    arrow.params(builder.context.ast),
                );
                break;
            }
            NodeKindData::Program(_) => break,
            _ => {}
        }
    }
    if let Some(parameter) = parameter {
        builder.report(
            builder.context.ast.span(directive.node_id()),
            "Illegal 'use strict' directive in function with non-simple parameter list",
            ReportOptions {
                labels: builder.labels(&[
                    builder.label(builder.context.ast.span(parameter), "non-simple parameter")
                ]),
                ..ReportOptions::default()
            },
        );
    }
}

pub(super) fn check_class_body(builder: &mut SemanticBuilder<'_, '_>, body: ClassBody) {
    let children = body.body(builder.context.ast);
    let mut first_constructor = None;
    for index in 0..children.len() {
        let child = children
            .get_node(builder.context.ast, index)
            .expect("class child exists")
            .node_id();
        // TypeScript permits constructor overload declarations and leaves duplicate
        // implementation diagnostics to its type checker.
        if !builder.context.ast.is_ts()
            && let NodeKindData::MethodDefinition(method) = builder.context.ast.kind_data(child)
            && method.kind(builder.context.ast)
                == swc_next_ecma_ast::MethodDefinitionKind::Constructor
        {
            if let Some(first) = first_constructor {
                builder.report(
                    builder
                        .context
                        .ast
                        .span(method.key(builder.context.ast).node_id()),
                    "A class can only have one constructor",
                    ReportOptions {
                        labels: builder.labels(&[builder.label(
                            builder.context.ast.span(first),
                            "first constructor defined here",
                        )]),
                        ..ReportOptions::default()
                    },
                );
            } else {
                first_constructor = Some(method.key(builder.context.ast).node_id());
            }
        }
        // Body-less TypeScript methods are erased overload signatures. They do not
        // introduce runtime private names and must not conflict with the implementation.
        if builder.context.ast.node_kind(child) == NodeKind::TsMethodDefinition {
            continue;
        }
        let Some(name) = private_element_name(builder.context.ast, child) else {
            continue;
        };
        let kind = private_element_kind(builder.context.ast, child);
        for previous_index in 0..index {
            let previous = children
                .get_node(builder.context.ast, previous_index)
                .expect("class child exists")
                .node_id();
            if builder.context.ast.node_kind(previous) == NodeKind::TsMethodDefinition {
                continue;
            }
            let Some(previous_name) = private_element_name(builder.context.ast, previous) else {
                continue;
            };
            if previous_name != name {
                continue;
            }
            let previous_kind = private_element_kind(builder.context.ast, previous);
            let get_set_pair = matches!(
                (previous_kind, kind),
                (PrivateElementKind::Getter, PrivateElementKind::Setter)
                    | (PrivateElementKind::Setter, PrivateElementKind::Getter)
            );
            if get_set_pair
                && private_element_is_static(builder.context.ast, previous)
                    == private_element_is_static(builder.context.ast, child)
            {
                continue;
            }
            let message = builder.fmt(format_args!(
                "Duplicate private name '#{}'",
                String::from_utf8_lossy(name)
            ));
            builder.report(
                builder.context.ast.span(child),
                message,
                ReportOptions {
                    labels: builder
                        .labels(&[builder
                            .label(builder.context.ast.span(previous), "first declared here")]),
                    ..ReportOptions::default()
                },
            );
            break;
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum PrivateElementKind {
    Field,
    Method,
    Getter,
    Setter,
}

/// Returns the StringValue of a class element's PrivateIdentifier key, if any.
fn private_element_name<'a>(tree: &'a Ast<'_>, node: NodeId) -> Option<&'a [u8]> {
    let key = match tree.kind_data(node) {
        NodeKindData::PropertyDefinition(property) => property.key(tree).node_id(),
        NodeKindData::MethodDefinition(method) => method.key(tree).node_id(),
        NodeKindData::TsMethodDefinition(method) => method.key(tree).node_id(),
        _ => return None,
    };
    let NodeKindData::PrivateIdentifier(identifier) = tree.kind_data(key) else {
        return None;
    };
    Some(tree.get_utf8(identifier.name(tree)).as_bytes())
}

fn private_element_kind(tree: &Ast<'_>, node: NodeId) -> PrivateElementKind {
    let kind = match tree.kind_data(node) {
        NodeKindData::MethodDefinition(method) => Some(method.kind(tree)),
        NodeKindData::TsMethodDefinition(method) => Some(method.kind(tree)),
        _ => None,
    };
    match kind {
        Some(swc_next_ecma_ast::MethodDefinitionKind::Get) => PrivateElementKind::Getter,
        Some(swc_next_ecma_ast::MethodDefinitionKind::Set) => PrivateElementKind::Setter,
        Some(_) => PrivateElementKind::Method,
        None => PrivateElementKind::Field,
    }
}

fn private_element_is_static(tree: &Ast<'_>, node: NodeId) -> bool {
    match tree.kind_data(node) {
        NodeKindData::PropertyDefinition(property) => property.r#static(tree),
        NodeKindData::MethodDefinition(method) => method.r#static(tree),
        NodeKindData::TsMethodDefinition(method) => method.r#static(tree),
        _ => false,
    }
}

/// 15.7.7 AllPrivateIdentifiersValid. Walks ancestor ClassBody nodes,
/// collecting PrivateBoundIdentifiers at each level.
fn is_private_name_declared(context: &SemanticVisitCtx<'_, '_>, name: &[u8]) -> bool {
    for ancestor in context.ancestors() {
        let NodeKindData::ClassBody(body) = context.ast.kind_data(ancestor) else {
            continue;
        };
        for child in body.body(context.ast).iter() {
            let child = context.ast.get_node_in_sub_range(child).node_id();
            if private_element_name(context.ast, child) == Some(name) {
                return true;
            }
        }
    }
    false
}

#[derive(Clone, Copy)]
enum SuperCallValidity {
    Valid,
    NotInConstructor,
    NoExtends,
}

/// `super()` is permitted only inside a constructor of a derived class.
/// Arrow functions are transparent (Section 8.5.1); regular functions are opaque.
fn super_call_validity(context: &SemanticVisitCtx<'_, '_>) -> SuperCallValidity {
    let mut ancestors = context.ancestors();
    while let Some(ancestor) = ancestors.next() {
        match context.ast.node_kind(ancestor) {
            NodeKind::ArrowFunctionExpression => {}
            NodeKind::Function => {
                if let Some(parent) = ancestors.next()
                    && let NodeKindData::MethodDefinition(method) = context.ast.kind_data(parent)
                    && method.kind(context.ast)
                        == swc_next_ecma_ast::MethodDefinitionKind::Constructor
                {
                    for class in ancestors {
                        if let NodeKindData::Class(class) = context.ast.kind_data(class) {
                            return if class.super_class(context.ast).is_some() {
                                SuperCallValidity::Valid
                            } else {
                                SuperCallValidity::NoExtends
                            };
                        }
                    }
                }
                return SuperCallValidity::NotInConstructor;
            }
            NodeKind::PropertyDefinition | NodeKind::StaticBlock | NodeKind::Program => {
                return SuperCallValidity::NotInConstructor;
            }
            _ => {}
        }
    }
    SuperCallValidity::NotInConstructor
}

/// `super.property` is valid in class methods, object methods, field
/// initializers, static blocks, and arrows inheriting from those contexts.
fn is_super_property_valid(context: &SemanticVisitCtx<'_, '_>) -> bool {
    let mut ancestors = context.ancestors();
    while let Some(ancestor) = ancestors.next() {
        match context.ast.node_kind(ancestor) {
            NodeKind::ArrowFunctionExpression => {}
            NodeKind::Function => {
                if let Some(parent) = ancestors.next() {
                    match context.ast.kind_data(parent) {
                        NodeKindData::MethodDefinition(_) => return true,
                        NodeKindData::ObjectProperty(property)
                            if property.method(context.ast)
                                || property.kind(context.ast)
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

fn is_new_target_available(context: &SemanticVisitCtx<'_, '_>) -> bool {
    for ancestor in context.ancestors() {
        match context.ast.node_kind(ancestor) {
            NodeKind::Function | NodeKind::StaticBlock | NodeKind::PropertyDefinition => {
                return true;
            }
            NodeKind::ArrowFunctionExpression => {}
            _ => {}
        }
    }
    // CommonJS top level is a function body.
    context.ast.source_type() == swc_next_ecma_ast::SourceType::CommonJs
}

// True when the nearest enclosing boundary is a static block.
fn is_inside_static_block(context: &SemanticVisitCtx<'_, '_>) -> bool {
    for ancestor in context.ancestors() {
        match context.ast.node_kind(ancestor) {
            NodeKind::StaticBlock => return true,
            NodeKind::Function | NodeKind::ArrowFunctionExpression => return false,
            _ => {}
        }
    }
    false
}

/// `arguments` is available inside regular functions but not in class field
/// initializers or static blocks (arrow functions are transparent).
fn is_arguments_available(context: &SemanticVisitCtx<'_, '_>) -> bool {
    let mut child = None;
    for ancestor in context.ancestors() {
        match context.ast.kind_data(ancestor) {
            NodeKindData::Function(_) => return true,
            NodeKindData::PropertyDefinition(property) => {
                if property
                    .value(context.ast)
                    .is_some_and(|value| Some(value.node_id()) == child)
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

pub(super) fn check_program(builder: &mut SemanticBuilder<'_, '_>) {
    builder.check_unresolved_exports();
}

pub(super) fn check_import_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: ImportDeclaration,
) {
    builder.check_import_export_position(
        declaration.node_id(),
        "import statement",
        "'import' declaration",
    );
    builder.check_duplicate_with_attributes(declaration.attributes(builder.context.ast));
}

pub(super) fn check_export_named_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: ExportNamedDeclaration,
) {
    builder.check_import_export_position(
        declaration.node_id(),
        "'export' declaration",
        "'export' declaration",
    );
    builder.check_duplicate_with_attributes(declaration.attributes(builder.context.ast));
}

pub(super) fn check_export_default_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: ExportDefaultDeclaration,
) {
    let export = declaration.declaration(builder.context.ast);
    if let ExportDefaultDeclarationKindData::Expr(expression) = builder
        .context
        .ast
        .export_default_declaration_kind_data(export)
    {
        ts::check_ts_ambient_export_assignment_expression(builder, expression.node_id());
    }
    builder.check_import_export_position(
        declaration.node_id(),
        "'export default' declaration",
        "'export default' declaration",
    );
    builder.record_exported_name(b"default", declaration.node_id());
}

pub(super) fn check_export_all_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: ExportAllDeclaration,
) {
    builder.check_import_export_position(
        declaration.node_id(),
        "'export *' declaration",
        "'export *' declaration",
    );
    if let Some(exported) = declaration.exported(builder.context.ast) {
        let name = module_export_name(builder.context.ast, exported.node_id());
        builder.record_exported_name(name, declaration.node_id());
    }
    builder.check_duplicate_with_attributes(declaration.attributes(builder.context.ast));
}

pub(super) fn check_export_specifier(
    builder: &mut SemanticBuilder<'_, '_>,
    specifier: ExportSpecifier,
) {
    let exported_name = module_export_name(
        builder.context.ast,
        specifier.exported(builder.context.ast).node_id(),
    );
    builder.record_exported_name(exported_name, specifier.node_id());
    // TypeScript export targets, including global augmentation members, are
    // resolved and checked together after traversal by the alias pass.
    if builder.context.ast.is_ts()
        || specifier.export_kind(builder.context.ast) == ImportOrExportKind::Type
        || builder.context.state.in_ts_namespace()
    {
        return;
    }
    if let Some(parent) = builder.context.parent()
        && let NodeKindData::ExportNamedDeclaration(declaration) =
            builder.context.ast.kind_data(parent)
        && declaration.source(builder.context.ast).is_none()
        && declaration.export_kind(builder.context.ast) != ImportOrExportKind::Type
    {
        builder.export_specifiers.push(specifier);
    }
}

impl<'traverse, 'a> SemanticBuilder<'traverse, 'a> {
    /// Post-traversal check that all local export specifiers refer to
    /// declared bindings.
    fn check_unresolved_exports(&mut self) {
        let tree = self.context.ast;
        if !tree.is_module() {
            return;
        }
        for index in 0..self.export_specifiers.len() {
            let specifier = self.export_specifiers[index];
            let name = module_export_name(tree, specifier.local(tree).node_id());
            if self
                .context
                .state
                .symbols
                .binding(ScopeId::MODULE, name)
                .is_none()
            {
                self.report_unresolved_export(tree, name, specifier.node_id());
            }
        }
    }

    fn check_import_export_position(
        &mut self,
        node: NodeId,
        out_of_module_label: &'static str,
        top_level_label: &'static str,
    ) {
        let Some(parent) = self.context.parent() else {
            return;
        };
        let message = match self.context.ast.node_kind(parent) {
            NodeKind::Program if !self.context.ast.is_module() => Some(self.fmt(format_args!(
                "Cannot use {} outside a module",
                out_of_module_label
            ))),
            NodeKind::Program | NodeKind::TsModuleBlock => None,
            _ => Some(self.fmt(format_args!(
                "{} may only appear at the top level",
                top_level_label
            ))),
        };
        if let Some(message) = message {
            self.report(
                self.context.ast.span(node),
                message,
                ReportOptions::default(),
            );
        }
    }

    fn check_duplicate_with_attributes(
        &mut self,
        attributes: swc_next_ecma_ast::SubRange<swc_next_ecma_ast::ImportAttribute>,
    ) {
        for index in 0..attributes.len() {
            let attribute = attributes
                .get_node(self.context.ast, index)
                .expect("import attribute exists");
            let key = attribute.key(self.context.ast).node_id();
            let Some(key_name) = import_attribute_name(self.context.ast, key) else {
                continue;
            };
            for previous_index in 0..index {
                let previous = attributes
                    .get_node(self.context.ast, previous_index)
                    .expect("import attribute exists");
                let previous_key = previous.key(self.context.ast).node_id();
                let Some(previous_key_name) = import_attribute_name(self.context.ast, previous_key)
                else {
                    continue;
                };
                if key_name == previous_key_name {
                    let message = self.fmt(format_args!(
                        "Duplicate import attribute key '{}'",
                        String::from_utf8_lossy(key_name)
                    ));
                    self.report(
                        self.context.ast.span(key),
                        message,
                        ReportOptions {
                            labels: self.labels(&[
                                self.label(self.context.ast.span(previous_key), "first used here")
                            ]),
                            ..ReportOptions::default()
                        },
                    );
                    break;
                }
            }
        }
    }

    /// Records an exported name and reports a duplicate if one already exists.
    fn record_exported_name(&mut self, name: &'traverse [u8], node: NodeId) {
        // TypeScript declarations can merge across the type and value namespaces. Duplicate
        // exports are therefore builder diagnostics rather than parser-level early errors.
        if self.context.ast.is_ts() {
            return;
        }
        // Exports inside a TS namespace are namespace-scoped, not module-scoped.
        if !self.context.ast.is_module() || self.context.state.in_ts_namespace() {
            return;
        }
        let first = match self.exported_names.entry(name) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                entry.insert(node);
                return;
            }
        };
        let message = self.fmt(format_args!(
            "Duplicate export of '{}'",
            String::from_utf8_lossy(name)
        ));
        self.report(
            self.context.ast.span(node),
            message,
            ReportOptions {
                labels: self.labels(&[
                    self.label(self.context.ast.span(first), "first exported here"),
                    self.label(self.context.ast.span(node), "exported again here"),
                ]),
                ..ReportOptions::default()
            },
        );
    }
}

fn is_at_program_level(context: &SemanticVisitCtx<'_, '_>) -> bool {
    context.parent().is_none_or(|parent| {
        matches!(
            context.ast.node_kind(parent),
            NodeKind::Program | NodeKind::TsModuleBlock
        )
    })
}

fn import_attribute_name<'a>(tree: &'a Ast<'_>, key: NodeId) -> Option<&'a [u8]> {
    match tree.kind_data(key) {
        NodeKindData::IdentifierName(identifier) => {
            Some(tree.get_utf8(identifier.name(tree)).as_bytes())
        }
        NodeKindData::StringLiteral(string) => Some(tree.get_wtf8(string.value(tree)).as_bytes()),
        _ => None,
    }
}
