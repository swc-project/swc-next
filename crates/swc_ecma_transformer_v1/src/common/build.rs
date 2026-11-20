//! Next AST construction and syntax-context adapters.
use crate::common::resolver::context::Resolver;
use swc_next_ecma_ast::*;

/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/utils.rs#L37>.
pub(crate) fn function(
    ast: &mut Ast<'_>,
    parameter: BindingIdentifier,
    statements: Vec<Stmt>,
) -> Expr {
    let parameter = ast.formal_parameter(
        DUMMY_SP,
        SubRange::empty(),
        FormalParameterPattern::BindingIdentifier(parameter),
        None,
        false,
    );
    let items = ast.add_sub_range([FormalParameterItem::FormalParameter(parameter)]);
    let params =
        ast.formal_parameters(DUMMY_SP, items, None, FormalParameterKind::FormalParameters);
    let statements = ast.add_sub_range(statements);
    let body = ast.function_body(DUMMY_SP, SubRange::empty(), statements);
    Expr::Function(ast.function(
        DUMMY_SP,
        FunctionType::FunctionExpression,
        None,
        None,
        None,
        params,
        None,
        body,
        false,
        false,
    ))
}

pub(crate) fn call(
    ast: &mut Ast<'_>,
    callee: Expr,
    arguments: impl IntoIterator<Item = Expr>,
    pure: bool,
) -> Expr {
    let arguments = ast.add_sub_range(arguments.into_iter().map(Argument::Expr));
    ast.expr_call_expression(DUMMY_SP, callee, None, arguments, false, pure)
}

pub(crate) fn member(ast: &mut Ast<'_>, object: Expr, name: &str) -> Expr {
    let name = ast.add_utf8(name);
    let key = ast.identifier_name(DUMMY_SP, name);
    ast.expr_member_expression(
        DUMMY_SP,
        object,
        PropertyKey::IdentifierName(key),
        false,
        false,
    )
}

pub(crate) fn computed(ast: &mut Ast<'_>, object: Expr, key: Expr) -> Expr {
    ast.expr_member_expression(DUMMY_SP, object, PropertyKey::Expr(key), true, false)
}

pub(crate) fn target(ast: &Ast<'_>, expression: Expr) -> SimpleAssignmentTarget {
    match ast.expr_data(expression) {
        ExprData::IdentifierReference(id) => SimpleAssignmentTarget::IdentifierReference(id),
        ExprData::MemberExpression(member) => SimpleAssignmentTarget::MemberExpression(member),
        _ => unreachable!("a generated assignment has an assignable left hand side"),
    }
}

pub(crate) fn assign(ast: &mut Ast<'_>, left: Expr, right: Expr) -> Expr {
    let left = AssignmentTarget::SimpleAssignmentTarget(target(ast, left));
    ast.expr_assignment_expression(DUMMY_SP, left, right, AssignmentOperator::Assign)
}

pub(crate) fn variable(
    ast: &mut Ast<'_>,
    kind: VariableKind,
    id: BindingIdentifier,
    init: Option<Expr>,
    span: Span,
) -> Decl {
    let declarator =
        ast.variable_declarator(span, BindingPattern::BindingIdentifier(id), init, false);
    let declarations = ast.add_sub_range([declarator]);
    Decl::VariableDeclaration(ast.variable_declaration(span, kind, declarations, false))
}

pub(crate) fn export(ast: &mut Ast<'_>, declaration: Decl) -> Stmt {
    Stmt::ExportNamedDeclaration(ast.export_named_declaration(
        DUMMY_SP,
        Some(declaration),
        SubRange::empty(),
        None,
        SubRange::empty(),
        ImportOrExportKind::Value,
    ))
}

/// SWC clones preserve syntax contexts. `CloneIn` copies the Next AST tree;
/// replaying the same ordinary visitor copies its side-table contexts as well.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_utils/src/constructor.rs#L92>.
pub(crate) fn clone_expr(ast: &mut Ast<'_>, resolver: &mut Resolver, expression: Expr) -> Expr {
    struct Nodes<'a, 'ast> {
        ast: &'a Ast<'ast>,
        nodes: Vec<NodeId>,
    }
    impl<'ast> Visit<'ast> for Nodes<'_, 'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }
        fn enter_node(&mut self, node: NodeKindData) {
            self.nodes.push(node.node_id());
        }
    }
    let mut original = Nodes {
        ast,
        nodes: Vec::new(),
    };
    expression.visit_with(&mut original);
    let original = original.nodes;
    let cloned = expression.clone_in(ast);
    let mut copies = Nodes {
        ast,
        nodes: Vec::new(),
    };
    cloned.visit_with(&mut copies);
    let copies = copies.nodes;
    for (original, copied) in original.into_iter().zip(copies) {
        resolver.copy_context(original, copied);
    }
    cloned
}

pub(crate) fn unwrap(ast: &Ast<'_>, mut expression: Expr) -> Expr {
    while let ExprData::ParenthesizedExpression(paren) = ast.expr_data(expression) {
        expression = paren.expression(ast);
    }
    expression
}

/// Whether this expression corresponds to a SWC `OptChainExpr` link.
pub(crate) fn chain_part(ast: &Ast<'_>, expression: Expr) -> bool {
    match ast.expr_data(expression) {
        ExprData::MemberExpression(member) => {
            member.optional(ast) || chain_part(ast, member.object(ast))
        }
        ExprData::CallExpression(call) => call.optional(ast) || chain_part(ast, call.callee(ast)),
        _ => false,
    }
}

/// Next AST adapter for SWC's `RefRewriter::exit_object_pat_prop`.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_utils/src/lib.rs#L2648>.
pub(crate) fn binding_shorthand(ast: &mut Ast<'_>, property: BindingProperty) {
    let value = property.value(ast);
    let value = value
        .as_assignment_pattern(ast)
        .map_or(value, |assignment| assignment.left(ast));
    if property.shorthand(ast)
        && !value.is_binding_identifier(ast)
        && !value
            .as_simple_assignment_target(ast)
            .is_some_and(|target| target.is_identifier_reference(ast))
    {
        property.set_shorthand(ast, false);
    }
}

/// SWC `sym_for_expr`/`alias_ident_for` on computed property expressions.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_utils/src/lib.rs#L1446>.
pub(crate) fn alias_ident_for(ast: &Ast<'_>, expression: Expr, default: &str) -> String {
    fn symbol(ast: &Ast<'_>, expression: Expr) -> Option<String> {
        match ast.expr_data(expression) {
            ExprData::StringLiteral(string) => {
                ast.get_wtf8(string.value(ast)).as_str().map(str::to_owned)
            }
            ExprData::ThisExpression(_) => Some("this".into()),
            ExprData::IdentifierReference(id) => Some(ast.get_utf8(id.name(ast)).to_owned()),
            ExprData::Function(function) => function
                .id(ast)
                .map(|id| ast.get_utf8(id.name(ast)).to_owned()),
            ExprData::Class(class) => class
                .id(ast)
                .map(|id| ast.get_utf8(id.name(ast)).to_owned()),
            ExprData::MemberExpression(member) => {
                let object = if member.object(ast).is_super(ast) {
                    "super".into()
                } else {
                    symbol(ast, member.object(ast)).unwrap_or_default()
                };
                let key = if member.computed(ast) {
                    symbol(ast, member.property(ast).as_expr(ast)?).unwrap_or_default()
                } else {
                    ast.get_utf8(member.property(ast).as_identifier_name(ast)?.name(ast))
                        .to_owned()
                };
                Some(format!("{object}_{key}"))
            }
            ExprData::CallExpression(call) => symbol(ast, call.callee(ast)),
            ExprData::ChainExpression(chain) => symbol(ast, chain.expression(ast)),
            _ => None,
        }
    }
    let name = symbol(ast, expression).unwrap_or_else(|| default.into());
    let mut name: String = name
        .chars()
        .filter(|c| is_identifier(format!("a{c}").as_bytes()))
        .collect();
    if name.is_empty() {
        name.push('_');
    }
    if name.starts_with('_') {
        name
    } else {
        format!("_{name}")
    }
}

pub(crate) fn undefined(ast: &mut Ast<'_>) -> Expr {
    let zero = ast.expr_numeric_literal(DUMMY_SP, 0.0, None);
    ast.expr_unary_expression(DUMMY_SP, zero, UnaryOperator::Void)
}

pub(crate) fn sequence(ast: &mut Ast<'_>, expressions: Vec<Expr>) -> Expr {
    if expressions.len() == 1 {
        expressions[0]
    } else {
        let expressions = ast.add_sub_range(expressions);
        ast.expr_sequence_expression(DUMMY_SP, expressions)
    }
}

pub(crate) fn target_expr(ast: &Ast<'_>, target: SimpleAssignmentTarget) -> Expr {
    match ast.simple_assignment_target_data(target) {
        SimpleAssignmentTargetData::IdentifierReference(id) => Expr::IdentifierReference(id),
        SimpleAssignmentTargetData::MemberExpression(member) => Expr::MemberExpression(member),
        SimpleAssignmentTargetData::TsAsExpression(node) => Expr::TsAsExpression(node),
        SimpleAssignmentTargetData::TsSatisfiesExpression(node) => {
            Expr::TsSatisfiesExpression(node)
        }
        SimpleAssignmentTargetData::TsTypeAssertion(node) => Expr::TsTypeAssertion(node),
        SimpleAssignmentTargetData::TsNonNullExpression(node) => Expr::TsNonNullExpression(node),
    }
}

/// SWC `find_pat_ids`: only declaration bindings, excluding initializer reads.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_utils/src/lib.rs#L1724>.
pub(crate) fn find_pat_ids(ast: &Ast<'_>, pattern: BindingPattern) -> Vec<BindingIdentifier> {
    let mut bindings = Vec::new();
    for_each_binding_ident(ast, pattern, |binding| bindings.push(binding));
    bindings
}

/// Next AST adapter for SWC's ECMAScript number-to-string conversion.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_utils/src/number.rs#L1>.
pub(crate) fn number_string(value: f64) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value == f64::INFINITY {
        return "Infinity".into();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".into();
    }
    if value == 0.0 {
        return "0".into();
    }
    if value.abs() < 1e-6 || value.abs() >= 1e21 {
        let value = format!("{value:e}");
        let (significand, exponent) = value.split_once('e').unwrap();
        if exponent.starts_with('-') {
            value
        } else {
            format!("{significand}e+{exponent}")
        }
    } else {
        value.to_string()
    }
}

pub(crate) fn assignment_target(
    ast: &mut Ast<'_>,
    resolver: &mut Resolver,
    pattern: BindingPattern,
) -> AssignmentTarget {
    struct Targets<'a, 'ast> {
        ast: &'a mut Ast<'ast>,
        resolver: &'a mut Resolver,
    }
    impl<'ast> VisitMut<'ast> for Targets<'_, 'ast> {
        fn ast(&mut self) -> &mut Ast<'ast> {
            self.ast
        }
        fn visit_mut_binding_pattern(&mut self, pattern: BindingPattern) -> Option<BindingPattern> {
            if let Some(binding) = pattern.as_binding_identifier(self.ast) {
                let reference = self.resolver.reference(self.ast, binding);
                Some(BindingPattern::SimpleAssignmentTarget(target(
                    self.ast, reference,
                )))
            } else {
                pattern.visit_mut_children_with(self)
            }
        }
    }
    let pattern = pattern
        .visit_mut_with(&mut Targets { ast, resolver })
        .unwrap_or(pattern);
    match ast.binding_pattern_data(pattern) {
        BindingPatternData::SimpleAssignmentTarget(target) => {
            AssignmentTarget::SimpleAssignmentTarget(target)
        }
        BindingPatternData::ArrayPattern(array) => AssignmentTarget::ArrayAssignmentTarget(array),
        BindingPatternData::ObjectPattern(object) => {
            AssignmentTarget::ObjectAssignmentTarget(object)
        }
        _ => unreachable!("a runtime assignment has an assignable pattern"),
    }
}

pub(crate) fn target_pattern(ast: &Ast<'_>, target: AssignmentTarget) -> BindingPattern {
    match ast.assignment_target_data(target) {
        AssignmentTargetData::SimpleAssignmentTarget(target) => {
            BindingPattern::SimpleAssignmentTarget(target)
        }
        AssignmentTargetData::ArrayAssignmentTarget(array) => BindingPattern::ArrayPattern(array),
        AssignmentTargetData::ObjectAssignmentTarget(object) => {
            BindingPattern::ObjectPattern(object)
        }
    }
}

pub(crate) fn prepend_stmts(
    ast: &mut Ast<'_>,
    statements: impl IntoIterator<Item = Stmt>,
    body: SubRange<Stmt>,
) -> SubRange<Stmt> {
    let statements: Vec<_> = statements
        .into_iter()
        .chain(body.iter().map(|slot| ast.get_node_in_sub_range(slot)))
        .collect();
    ast.add_sub_range(statements)
}

pub(crate) fn declaration(
    ast: &mut Ast<'_>,
    kind: VariableKind,
    declarations: Vec<VariableDeclarator>,
) -> Stmt {
    let declarations = ast.add_sub_range(declarations);
    Stmt::Declaration(Decl::VariableDeclaration(ast.variable_declaration(
        DUMMY_SP,
        kind,
        declarations,
        false,
    )))
}

pub(crate) fn array_pattern(
    ast: &mut Ast<'_>,
    mut elements: Vec<Option<BindingPattern>>,
) -> BindingPattern {
    let rest = elements
        .last()
        .copied()
        .flatten()
        .and_then(|element| element.as_binding_rest_element(ast));
    if rest.is_some() {
        elements.pop();
    }
    let elements = ast.add_sub_range(elements);
    BindingPattern::ArrayPattern(ast.array_pattern(DUMMY_SP, elements, rest, None, false))
}

pub(crate) fn parameters(
    ast: &mut Ast<'_>,
    bindings: impl IntoIterator<Item = BindingIdentifier>,
) -> FormalParameters {
    let items: Vec<_> = bindings
        .into_iter()
        .map(|binding| {
            FormalParameterItem::FormalParameter(ast.formal_parameter(
                DUMMY_SP,
                SubRange::empty(),
                FormalParameterPattern::BindingIdentifier(binding),
                None,
                false,
            ))
        })
        .collect();
    let items = ast.add_sub_range(items);
    ast.formal_parameters(DUMMY_SP, items, None, FormalParameterKind::FormalParameters)
}

pub(crate) fn arrow(
    ast: &mut Ast<'_>,
    bindings: impl IntoIterator<Item = BindingIdentifier>,
    body: ArrowFunctionBody,
) -> Expr {
    let params = parameters(ast, bindings);
    Expr::ArrowFunctionExpression(ast.arrow_function_expression(
        DUMMY_SP,
        body.is_expr(ast),
        false,
        None,
        params,
        None,
        body,
    ))
}

pub(crate) fn private_name(ast: &Ast<'_>, key: PropertyKey) -> Option<String> {
    let private = key.as_expr(ast)?.as_private_identifier(ast)?;
    Some(ast.get_utf8(private.name(ast)).to_owned())
}

pub(crate) fn block(
    ast: &mut Ast<'_>,
    statements: impl IntoIterator<Item = Stmt>,
) -> BlockStatement {
    let statements = ast.add_sub_range(statements);
    ast.block_statement(DUMMY_SP, statements)
}
pub(crate) fn declarator(
    ast: &mut Ast<'_>,
    binding: BindingIdentifier,
    init: Option<Expr>,
) -> VariableDeclarator {
    ast.variable_declarator(
        DUMMY_SP,
        BindingPattern::BindingIdentifier(binding),
        init,
        false,
    )
}
pub(crate) fn function_body(
    ast: &mut Ast<'_>,
    statements: impl IntoIterator<Item = Stmt>,
) -> FunctionBody {
    let statements = ast.add_sub_range(statements);
    ast.function_body(DUMMY_SP, SubRange::empty(), statements)
}

pub(crate) fn function_with(
    ast: &mut Ast<'_>,
    bindings: impl IntoIterator<Item = BindingIdentifier>,
    statements: impl IntoIterator<Item = Stmt>,
) -> Expr {
    let params = parameters(ast, bindings);
    let body = function_body(ast, statements);
    Expr::Function(ast.function(
        DUMMY_SP,
        FunctionType::FunctionExpression,
        None,
        None,
        None,
        params,
        None,
        body,
        false,
        false,
    ))
}

pub(crate) fn compound(ast: &mut Ast<'_>, op: AssignmentOperator, left: Expr, right: Expr) -> Expr {
    let binary = match op {
        AssignmentOperator::AddAssign => BinaryOperator::Add,
        AssignmentOperator::SubtractAssign => BinaryOperator::Subtract,
        AssignmentOperator::MultiplyAssign => BinaryOperator::Multiply,
        AssignmentOperator::DivideAssign => BinaryOperator::Divide,
        AssignmentOperator::ModuloAssign => BinaryOperator::Modulo,
        AssignmentOperator::ExponentAssign => BinaryOperator::Exponent,
        AssignmentOperator::LeftShiftAssign => BinaryOperator::LeftShift,
        AssignmentOperator::RightShiftAssign => BinaryOperator::RightShift,
        AssignmentOperator::UnsignedRightShiftAssign => BinaryOperator::UnsignedRightShift,
        AssignmentOperator::BitwiseOrAssign => BinaryOperator::BitwiseOr,
        AssignmentOperator::BitwiseXorAssign => BinaryOperator::BitwiseXor,
        AssignmentOperator::BitwiseAndAssign => BinaryOperator::BitwiseAnd,
        AssignmentOperator::LogicalOrAssign
        | AssignmentOperator::LogicalAndAssign
        | AssignmentOperator::NullishAssign => {
            let op = match op {
                AssignmentOperator::LogicalOrAssign => LogicalOperator::Or,
                AssignmentOperator::LogicalAndAssign => LogicalOperator::And,
                _ => LogicalOperator::NullishCoalescing,
            };
            return ast.expr_logical_expression(DUMMY_SP, left, right, op);
        }
        AssignmentOperator::Assign => return right,
    };
    ast.expr_binary_expression(DUMMY_SP, left, right, binary)
}

pub(crate) fn string(ast: &mut Ast<'_>, value: &str) -> Expr {
    let value = ast.add_wtf8(value.into());
    ast.expr_string_literal(DUMMY_SP, value, AstUtf8::EMPTY)
}
pub(crate) fn property(ast: &mut Ast<'_>, name: &str, value: Expr) -> ObjectPropertyKind {
    let name = ast.add_utf8(name);
    let key = PropertyKey::IdentifierName(ast.identifier_name(DUMMY_SP, name));
    ObjectPropertyKind::ObjectProperty(ast.object_property(
        DUMMY_SP,
        key,
        value,
        PropertyKind::Init,
        false,
        false,
        false,
    ))
}

/// SWC LiteralVisitor with allow_non_json_value=true; cost is unused here.
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L1174>.
pub(crate) fn is_literal(ast: &Ast<'_>, expression: Expr) -> bool {
    match ast.expr_data(expression) {
        ExprData::NullLiteral(_)
        | ExprData::BooleanLiteral(_)
        | ExprData::StringLiteral(_)
        | ExprData::NumericLiteral(_)
        | ExprData::BigIntLiteral(_) => true,
        ExprData::TemplateLiteral(template) => template.expressions(ast).is_empty(),
        ExprData::ParenthesizedExpression(paren) => is_literal(ast, paren.expression(ast)),
        ExprData::MemberExpression(member) => member
            .object(ast)
            .as_identifier_reference(ast)
            .is_some_and(|id| ast.get_utf8(id.name(ast)) == "Symbol"),
        ExprData::ArrayExpression(array) => array.elements(ast).iter().all(|slot| {
            ast.get_node_in_sub_range(slot).is_none_or(|element| {
                element
                    .as_expr(ast)
                    .is_some_and(|expression| is_literal(ast, expression))
            })
        }),
        ExprData::ObjectExpression(object) => object.properties(ast).iter().all(|slot| {
            ast.get_node_in_sub_range(slot)
                .as_object_property(ast)
                .is_some_and(|property| {
                    property.kind(ast) == PropertyKind::Init
                        && !property.method(ast)
                        && !property.shorthand(ast)
                        && !property.computed(ast)
                        && !property
                            .key(ast)
                            .as_expr(ast)
                            .is_some_and(|key| key.is_big_int_literal(ast))
                        && is_literal(ast, property.value(ast))
                })
        }),
        _ => false,
    }
}

struct BindingIdentifierVisitor<'op, 'ast, F> {
    ast: &'ast Ast<'ast>,
    op: &'op mut F,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L1724>.
pub(crate) fn for_each_binding_ident<'ast>(
    ast: &'ast Ast<'ast>,
    pattern: BindingPattern,
    mut op: impl FnMut(BindingIdentifier),
) {
    pattern.visit_with(&mut BindingIdentifierVisitor { ast, op: &mut op });
}

impl<'ast, F: FnMut(BindingIdentifier)> Visit<'ast> for BindingIdentifierVisitor<'_, 'ast, F> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L1724>.
    fn visit_binding_identifier(&mut self, identifier: BindingIdentifier) {
        (self.op)(identifier);
    }

    /// Initializers and computed expressions do not introduce bindings.
    ///
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L1724>.
    fn visit_expr(&mut self, _: Expr) {}

    /// A non-computed property name does not introduce a binding either.
    ///
    /// Next AST adapter for SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/lib.rs#L1724>.
    fn visit_property_key(&mut self, _: PropertyKey) {}
}
