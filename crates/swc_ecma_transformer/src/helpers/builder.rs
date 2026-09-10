use super::HelperKind;
use crate::context::Mutation;
use swc_next_ecma_ast::{
    Argument, AstUtf8, BindingIdentifier, BindingPattern, DUMMY_SP, Decl, Expr,
    ImportDeclarationSpecifier, ImportOrExportKind, PropertyKey, ScopeId, Stmt, SubRange,
    VariableKind,
};
use swc_next_ecma_semantic::ReferenceFlags;

fn helper_var(binding: BindingIdentifier, init: Expr, ctx: &mut Mutation<'_, '_, '_>) -> Stmt {
    let scope = ctx.semantic().scope_of(
        ctx.semantic()
            .symbol_of(binding.node_id())
            .expect("helper binding exists"),
    );
    let declarator = ctx.add_node(scope, |ast| {
        ast.variable_declarator(
            DUMMY_SP,
            BindingPattern::BindingIdentifier(binding),
            None,
            Some(init),
            false,
        )
    });
    let declarators = ctx.add_sub_range([declarator]);
    Stmt::Declaration(Decl::VariableDeclaration(ctx.add_node(scope, |ast| {
        ast.variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false)
    })))
}

pub(super) fn call<const N: usize>(
    callee: Expr,
    arguments: [Argument; N],
    ctx: &mut Mutation<'_, '_, '_>,
) -> Expr {
    let scope = ctx
        .semantic()
        .scope_at(callee.node_id())
        .expect("callee is registered");
    let arguments = ctx.add_sub_range(arguments);
    Expr::CallExpression(ctx.add_node(scope, |ast| {
        ast.call_expression(DUMMY_SP, callee, None, arguments, false)
    }))
}

pub(super) fn external_member(
    namespace: AstUtf8,
    export: &str,
    scope: ScopeId,
    ctx: &mut Mutation<'_, '_, '_>,
) -> Expr {
    let object =
        Expr::IdentifierReference(ctx.generated_reference_in_scope(namespace, DUMMY_SP, scope));
    let property = ctx.intern_utf8(export);
    let property = PropertyKey::IdentifierName(
        ctx.add_node(scope, |ast| ast.identifier_name(DUMMY_SP, property)),
    );
    Expr::MemberExpression(ctx.add_node(scope, |ast| {
        ast.member_expression(DUMMY_SP, object, property, false, false)
    }))
}

pub(super) fn external_import(
    name: AstUtf8,
    helper: HelperKind,
    ctx: &mut Mutation<'_, '_, '_>,
) -> Stmt {
    let source = match helper {
        HelperKind::SetFunctionName => "@babel/runtime/helpers/setFunctionName",
        HelperKind::ToPropertyKey => "@swc/helpers",
    };
    let scope = ctx.program_scope();
    let local = ctx.generated_declaration(name);
    if !ctx.ast().is_module() {
        let require = ctx.intern_utf8("require");
        let require = Expr::IdentifierReference(ctx.create_named_reference(
            require,
            ctx.program_scope(),
            ReferenceFlags::default(),
        ));
        let source = ctx.intern_wtf8(source);
        let source = Expr::StringLiteral(ctx.add_node(scope, |ast| {
            ast.string_literal(DUMMY_SP, source, ast.empty_utf8())
        }));
        let initializer = call(require, [Argument::Expr(source)], ctx);
        return helper_var(local, initializer, ctx);
    }
    let specifier = match helper {
        HelperKind::SetFunctionName => ImportDeclarationSpecifier::ImportDefaultSpecifier(
            ctx.add_node(scope, |ast| ast.import_default_specifier(DUMMY_SP, local)),
        ),
        HelperKind::ToPropertyKey => ImportDeclarationSpecifier::ImportNamespaceSpecifier(
            ctx.add_node(scope, |ast| ast.import_namespace_specifier(DUMMY_SP, local)),
        ),
    };
    let specifiers = ctx.add_sub_range([specifier]);
    let source = ctx.intern_wtf8(source);
    let source = ctx.add_node(scope, |ast| {
        ast.string_literal(DUMMY_SP, source, ast.empty_utf8())
    });
    Stmt::ImportDeclaration(ctx.add_node(scope, |ast| {
        ast.import_declaration(
            DUMMY_SP,
            specifiers,
            source,
            SubRange::empty(),
            None,
            ImportOrExportKind::Value,
        )
    }))
}
