//! Construction adapters for SWC's expression factories and identifier contexts.

use crate::resolver::{Id, Resolver};
use swc_next_ecma_ast::*;

/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/utils.rs#L37>.
pub(super) fn function(
    ast: &mut Ast<'_>,
    parameter: BindingIdentifier,
    statements: Vec<Stmt>,
) -> Expr {
    let parameter = ast.formal_parameter(
        DUMMY_SP,
        SubRange::empty(),
        FormalParameterPattern::BindingIdentifier(parameter),
        None,
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

pub(super) fn call(
    ast: &mut Ast<'_>,
    callee: Expr,
    arguments: impl IntoIterator<Item = Expr>,
    pure: bool,
) -> Expr {
    let arguments = ast.add_sub_range(arguments.into_iter().map(Argument::Expr));
    ast.expr_call_expression(DUMMY_SP, callee, None, arguments, false, pure)
}

pub(super) fn member(ast: &mut Ast<'_>, object: Expr, name: &str) -> Expr {
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

pub(super) fn computed(ast: &mut Ast<'_>, object: Expr, key: Expr) -> Expr {
    ast.expr_member_expression(DUMMY_SP, object, PropertyKey::Expr(key), true, false)
}

pub(super) fn target(ast: &Ast<'_>, expression: Expr) -> SimpleAssignmentTarget {
    match ast.expr_data(expression) {
        ExprData::IdentifierReference(id) => SimpleAssignmentTarget::IdentifierReference(id),
        ExprData::MemberExpression(member) => SimpleAssignmentTarget::MemberExpression(member),
        _ => unreachable!("a generated assignment has an assignable left hand side"),
    }
}

pub(super) fn assign(ast: &mut Ast<'_>, left: Expr, right: Expr) -> Expr {
    let left = AssignmentTarget::SimpleAssignmentTarget(target(ast, left));
    ast.expr_assignment_expression(DUMMY_SP, left, right, AssignmentOperator::Assign)
}

pub(super) fn variable(
    ast: &mut Ast<'_>,
    kind: VariableKind,
    id: BindingIdentifier,
    init: Option<Expr>,
    span: Span,
) -> Decl {
    let declarator = ast.variable_declarator(
        span,
        BindingPattern::BindingIdentifier(id),
        None,
        init,
        false,
    );
    let declarations = ast.add_sub_range([declarator]);
    Decl::VariableDeclaration(ast.variable_declaration(span, kind, declarations, false))
}

pub(super) fn export(ast: &mut Ast<'_>, declaration: Decl) -> Stmt {
    Stmt::ExportNamedDeclaration(ast.export_named_declaration(
        DUMMY_SP,
        Some(declaration),
        SubRange::empty(),
        None,
        SubRange::empty(),
        ImportOrExportKind::Value,
    ))
}

pub(super) fn reference(ast: &mut Ast<'_>, resolver: &mut Resolver, id: &Id) -> Expr {
    let binding = resolver.binding(ast, id);
    resolver.reference(ast, binding)
}

/// SWC clones preserve syntax contexts. `CloneIn` copies the Next AST tree;
/// replaying the same ordinary visitor copies its side-table contexts as well.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_utils/src/constructor.rs#L92>.
pub(super) fn clone_expr(ast: &mut Ast<'_>, resolver: &mut Resolver, expression: Expr) -> Expr {
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
    resolver.register(ast);
    for (original, copied) in original.into_iter().zip(copies) {
        resolver.node_scopes[copied] = resolver.node_scopes[original];
    }
    cloned
}

/// Next's dotted namespace names are one qualified node; SWC represents each
/// segment as a nested declaration.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_transforms_typescript/src/transform/namespaces.rs#L279>.
pub(super) fn namespace_names(ast: &Ast<'_>, name: TsModuleName) -> Vec<(NodeId, AstUtf8)> {
    fn qualified(ast: &Ast<'_>, name: TsQualifiedName, names: &mut Vec<(NodeId, AstUtf8)>) {
        match ast.ts_qualified_name_left_data(name.left(ast)) {
            TsQualifiedNameLeftData::BindingIdentifier(id) => {
                names.push((id.node_id(), id.name(ast)))
            }
            TsQualifiedNameLeftData::IdentifierReference(id) => {
                names.push((id.node_id(), id.name(ast)))
            }
            TsQualifiedNameLeftData::IdentifierName(id) => names.push((id.node_id(), id.name(ast))),
            TsQualifiedNameLeftData::TsQualifiedName(name) => qualified(ast, name, names),
            TsQualifiedNameLeftData::ThisExpression(_) => unreachable!(),
        }
        let right = name.right(ast);
        names.push((right.node_id(), right.name(ast)));
    }
    let mut names = Vec::new();
    match ast.ts_module_name_data(name) {
        TsModuleNameData::BindingIdentifier(id) => names.push((id.node_id(), id.name(ast))),
        TsModuleNameData::TsQualifiedName(name) => qualified(ast, name, &mut names),
        TsModuleNameData::StringLiteral(_) => {}
    }
    names
}

pub(super) fn unwrap(ast: &Ast<'_>, mut expression: Expr) -> Expr {
    while let ExprData::ParenthesizedExpression(paren) = ast.expr_data(expression) {
        expression = paren.expression(ast);
    }
    expression
}

pub(super) fn enum_name(
    ast: &Ast<'_>,
    name: TsEnumMemberName,
) -> swc_next_allocator::wtf8::Wtf8Buf {
    use swc_next_allocator::wtf8::Wtf8Buf;
    match ast.ts_enum_member_name_data(name) {
        TsEnumMemberNameData::IdentifierName(id) => Wtf8Buf::from_str(ast.get_utf8(id.name(ast))),
        TsEnumMemberNameData::StringLiteral(string) => ast.get_wtf8(string.value(ast)).to_owned(),
        TsEnumMemberNameData::TemplateLiteral(template) => {
            let quasi = ast.get_node_in_sub_range(template.quasis(ast).iter().next().unwrap());
            ast.get_wtf8(quasi.cooked(ast)).to_owned()
        }
    }
}

/// SWC `find_pat_ids`: only declaration bindings, excluding initializer reads.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_utils/src/lib.rs#L1724>.
pub(super) fn binding_ids(ast: &Ast<'_>, pattern: BindingPattern) -> Vec<BindingIdentifier> {
    struct Bindings<'a, 'ast> {
        ast: &'a Ast<'ast>,
        ids: Vec<BindingIdentifier>,
    }
    impl<'ast> Visit<'ast> for Bindings<'_, 'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }
        fn visit_binding_identifier(&mut self, id: BindingIdentifier) {
            self.ids.push(id);
        }
        fn visit_expr(&mut self, _: Expr) {}
        fn visit_property_key(&mut self, _: PropertyKey) {}
    }
    let mut bindings = Bindings {
        ast,
        ids: Vec::new(),
    };
    pattern.visit_with(&mut bindings);
    bindings.ids
}

pub(super) fn entity_root(
    ast: &Ast<'_>,
    reference: TsModuleReference,
) -> Option<IdentifierReference> {
    let name = reference.as_ts_type_name(ast)?;
    if let Some(id) = name.as_identifier_reference(ast) {
        return Some(id);
    }
    let mut name = name.as_ts_qualified_name(ast)?;
    loop {
        match ast.ts_qualified_name_left_data(name.left(ast)) {
            TsQualifiedNameLeftData::IdentifierReference(id) => return Some(id),
            TsQualifiedNameLeftData::TsQualifiedName(qualified) => name = qualified,
            _ => return None,
        }
    }
}

/// Next AST adapter for SWC's `RefRewriter::exit_object_pat_prop`.
/// SWC: <https://github.com/swc-project/swc/blob/75848acc40ea51b8409ac57b97f2d7fbbc2a3bdd/crates/swc_ecma_utils/src/lib.rs#L2648>.
pub(super) fn binding_shorthand(ast: &mut Ast<'_>, property: BindingProperty) {
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
