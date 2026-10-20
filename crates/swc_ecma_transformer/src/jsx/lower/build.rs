use swc_next_ecma_ast::{
    Argument, Ast, AstUtf8, DUMMY_SP, Decl, Expr, ExprData, ImportDeclarationSpecifier,
    ImportOrExportKind, ModuleExportName, ObjectExpression, ObjectPropertyKind, PropertyKey,
    PropertyKind, Span, Stmt, SubRange, UnaryOperator, VariableDeclaration, VariableKind,
    is_identifier,
};

pub(super) struct QualifiedName<'options> {
    root: NameRoot<'options>,
    properties: Option<&'options str>,
}

enum NameRoot<'options> {
    Identifier(&'options str),
    This,
    ImportMeta,
}

impl<'options> QualifiedName<'options> {
    pub(super) fn parse(
        value: Option<&'options str>,
        default_root: &'options str,
        default_property: &'options str,
    ) -> Self {
        value.and_then(Self::parse_path).unwrap_or(Self {
            root: NameRoot::Identifier(default_root),
            properties: Some(default_property),
        })
    }

    fn parse_path(value: &'options str) -> Option<Self> {
        let (root, mut properties) = value
            .split_once('.')
            .map_or((value, None), |(root, properties)| (root, Some(properties)));
        let root = match root {
            "this" => NameRoot::This,
            "import" => {
                let members = properties?;
                let (meta, tail) = members
                    .split_once('.')
                    .map_or((members, None), |(meta, tail)| (meta, Some(tail)));
                if meta != "meta" {
                    return None;
                }
                properties = tail;
                NameRoot::ImportMeta
            }
            root if is_identifier(root.as_bytes()) && !is_reserved_root(root) => {
                NameRoot::Identifier(root)
            }
            _ => return None,
        };
        if properties.is_some_and(|properties| {
            properties
                .split('.')
                .any(|part| !is_identifier(part.as_bytes()))
        }) {
            return None;
        }
        Some(Self { root, properties })
    }

    pub(super) fn root(&self) -> Option<&[u8]> {
        match self.root {
            NameRoot::Identifier(root) => Some(root.as_bytes()),
            NameRoot::This | NameRoot::ImportMeta => None,
        }
    }

    pub(super) fn create_expression(&self, ast: &mut Ast<'_>, namespace: Option<AstUtf8>) -> Expr {
        let mut expression = match self.root {
            NameRoot::This => ast.expr_this_expression(DUMMY_SP),
            NameRoot::ImportMeta => {
                let import = ast.add_utf8("import");
                let meta = ast.add_utf8("meta");
                let import = ast.identifier_name(DUMMY_SP, import);
                let meta = ast.identifier_name(DUMMY_SP, meta);
                ast.expr_meta_property(DUMMY_SP, import, meta)
            }
            NameRoot::Identifier(root) => {
                if let Some(namespace) = namespace {
                    let object = ast.expr_identifier_reference(DUMMY_SP, namespace);
                    static_member(ast, object, root, DUMMY_SP)
                } else {
                    let name = ast.add_utf8(root);
                    ast.expr_identifier_reference(DUMMY_SP, name)
                }
            }
        };
        if let Some(properties) = self.properties {
            for part in properties.split('.') {
                expression = static_member(ast, expression, part, DUMMY_SP);
            }
        }
        expression
    }
}

fn is_reserved_root(root: &str) -> bool {
    matches!(
        root,
        "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "in"
            | "instanceof"
            | "let"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
            | "implements"
            | "interface"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "static"
    )
}

pub(super) fn call(
    ast: &mut Ast<'_>,
    span: Span,
    callee: Expr,
    arguments: Vec<Argument>,
    pure: bool,
) -> Expr {
    let arguments = ast.add_sub_range(arguments);
    ast.expr_call_expression(span, callee, None, arguments, false, pure)
}

pub(super) fn void_zero(ast: &mut Ast<'_>) -> Expr {
    let zero = ast.expr_numeric_literal(DUMMY_SP, 0.0, None);
    ast.expr_unary_expression(DUMMY_SP, zero, UnaryOperator::Void)
}

pub(super) fn object_property(
    ast: &mut Ast<'_>,
    name: &str,
    value: Expr,
    span: Span,
) -> ObjectPropertyKind {
    let name = ast.add_utf8(name);
    let key = PropertyKey::IdentifierName(ast.identifier_name(span, name));
    let shorthand = is_shorthand_property(key, value, ast);
    ObjectPropertyKind::ObjectProperty(ast.object_property(
        span,
        key,
        value,
        PropertyKind::Init,
        false,
        shorthand,
        false,
    ))
}

pub(super) fn is_shorthand_property(key: PropertyKey, value: Expr, ast: &Ast<'_>) -> bool {
    let Some(key) = key.as_identifier_name(ast) else {
        return false;
    };
    let ExprData::IdentifierReference(value) = ast.expr_data(value) else {
        return false;
    };
    ast.get_utf8(key.name(ast)) == ast.get_utf8(value.name(ast))
}

pub(super) fn string_expression(ast: &mut Ast<'_>, span: Span, value: &str) -> Expr {
    let value = ast.add_utf8(value);
    ast.expr_string_literal(span, value.into(), AstUtf8::EMPTY)
}

pub(super) fn string_property(ast: &mut Ast<'_>, span: Span, value: &str) -> PropertyKey {
    let value = ast.add_utf8(value);
    PropertyKey::Expr(ast.expr_string_literal(span, value.into(), AstUtf8::EMPTY))
}

pub(super) fn static_member(ast: &mut Ast<'_>, object: Expr, property: &str, span: Span) -> Expr {
    let property = ast.add_utf8(property);
    let property = ast.identifier_name(span, property);
    ast.expr_member_expression(
        span,
        object,
        PropertyKey::IdentifierName(property),
        false,
        false,
    )
}

pub(super) fn static_member_with_name(ast: &mut Ast<'_>, object: AstUtf8, property: &str) -> Expr {
    let object = ast.expr_identifier_reference(DUMMY_SP, object);
    static_member(ast, object, property, DUMMY_SP)
}

pub(super) fn import_statement<I>(ast: &mut Ast<'_>, source: &str, specifiers: I) -> Stmt
where
    I: IntoIterator<Item = (&'static str, AstUtf8)>,
{
    let mut lowered = Vec::new();
    for (imported, local) in specifiers {
        let imported = ast.add_utf8(imported);
        let imported = ModuleExportName::IdentifierName(ast.identifier_name(DUMMY_SP, imported));
        let local = ast.binding_identifier(DUMMY_SP, local);
        lowered.push(ImportDeclarationSpecifier::ImportSpecifier(
            ast.import_specifier(DUMMY_SP, imported, local, ImportOrExportKind::Value),
        ));
    }
    let specifiers = ast.add_sub_range(lowered);
    let source = ast.add_utf8(source);
    let source = ast.string_literal(DUMMY_SP, source.into(), AstUtf8::EMPTY);
    ast.stmt_import_declaration(
        DUMMY_SP,
        specifiers,
        source,
        SubRange::empty(),
        None,
        ImportOrExportKind::Value,
    )
}

pub(super) fn require_statement(ast: &mut Ast<'_>, source: &str, local: AstUtf8) -> Stmt {
    let require = ast.add_utf8("require");
    let callee = ast.expr_identifier_reference(DUMMY_SP, require);
    let source = string_expression(ast, DUMMY_SP, source);
    let arguments = ast.add_sub_range([Argument::Expr(source)]);
    let init = ast.expr_call_expression(DUMMY_SP, callee, None, arguments, false, false);
    let id = ast.binding_pattern_binding_identifier(DUMMY_SP, local);
    let declarator = ast.variable_declarator(DUMMY_SP, id, None, Some(init), false);
    let declarations = ast.add_sub_range([declarator]);
    let declaration = ast.variable_declaration(DUMMY_SP, VariableKind::Var, declarations, false);
    Stmt::Declaration(Decl::VariableDeclaration(declaration))
}

pub(super) fn variable_declaration(
    ast: &mut Ast<'_>,
    local: AstUtf8,
    init: Expr,
) -> VariableDeclaration {
    let id = ast.binding_pattern_binding_identifier(DUMMY_SP, local);
    let declarator = ast.variable_declarator(DUMMY_SP, id, None, Some(init), false);
    let declarations = ast.add_sub_range([declarator]);
    ast.variable_declaration(DUMMY_SP, VariableKind::Var, declarations, false)
}

pub(super) fn has_accessor_property(object: ObjectExpression, ast: &Ast<'_>) -> bool {
    object.properties(ast).iter().any(|id| {
        ast.get_node_in_sub_range(id)
            .as_object_property(ast)
            .is_some_and(|property| property.kind(ast) != PropertyKind::Init)
    })
}

pub(super) fn has_proto_property(expression: Expr, ast: &Ast<'_>) -> bool {
    let ExprData::ObjectExpression(object) = ast.expr_data(expression) else {
        return false;
    };
    object.properties(ast).iter().any(|property_id| {
        let property = ast.get_node_in_sub_range(property_id);
        let Some(property) = property.as_object_property(ast) else {
            return false;
        };
        if property.computed(ast) || property.shorthand(ast) {
            return false;
        }
        let key = property.key(ast);
        if let Some(identifier) = key.as_identifier_name(ast) {
            return ast.get_utf8(identifier.name(ast)) == "__proto__";
        }
        key.as_expr(ast)
            .and_then(|expression| expression.as_string_literal(ast))
            .is_some_and(|literal| ast.get_wtf8(literal.value(ast)).as_bytes() == b"__proto__")
    })
}

#[cfg(test)]
mod tests {
    use super::QualifiedName;
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::Ast;

    #[test]
    fn pragma_roots() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let path = QualifiedName::parse(Some("import.meta.h"), "React", "createElement");
        let expression = path.create_expression(&mut ast, None);
        let member = expression.as_member_expression(&ast).unwrap();
        assert!(member.object(&ast).is_meta_property(&ast));

        let path = QualifiedName::parse(Some("this.h"), "React", "createElement");
        let expression = path.create_expression(&mut ast, None);
        let member = expression.as_member_expression(&ast).unwrap();
        assert!(member.object(&ast).is_this_expression(&ast));
    }
}
