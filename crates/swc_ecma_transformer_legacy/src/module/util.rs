use swc_next_ecma_ast::*;

use crate::common::{build, helpers::Helpers, resolver::context::Resolver};

use super::{ModuleFeatures, module_record::LocalExportEntry};

pub(super) struct ModuleCtx<'a, 'ast> {
    pub ast: &'a mut Ast<'ast>,
    pub resolver: &'a mut Resolver,
    pub helpers: &'a mut Helpers,
    pub features: ModuleFeatures,
}

impl ModuleCtx<'_, '_> {
    pub fn private(&mut self, name: &str) -> BindingIdentifier {
        self.resolver.private_binding(self.ast, name)
    }

    pub fn reference(&mut self, binding: BindingIdentifier) -> Expr {
        self.resolver.reference(self.ast, binding)
    }

    pub fn global(&mut self, name: &str) -> Expr {
        self.resolver.marked_reference(self.ast, name, true)
    }

    pub fn global_binding(&mut self, name: &str) -> BindingIdentifier {
        let reference = self.global(name);
        let symbol = self.resolver.symbol_of(reference.node_id());
        self.resolver.binding(self.ast, symbol)
    }

    pub fn helper(&mut self, name: &'static str) -> Expr {
        self.helpers.helper_expr(self.ast, self.resolver, name)
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/path.rs#L49>.
    pub fn make_require_call(&mut self, source: StringLiteral) -> Expr {
        let require = self.global("require");
        build::call(self.ast, require, [Expr::StringLiteral(source)], false)
    }

    pub fn const_var_kind(&self) -> VariableKind {
        if self.features.block_scoping {
            VariableKind::Const
        } else {
            VariableKind::Var
        }
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_utils/src/factory.rs#L108>.
pub(super) fn into_lazy_auto(
    cx: &mut ModuleCtx<'_, '_>,
    params: impl IntoIterator<Item = BindingIdentifier>,
    value: Expr,
) -> Expr {
    if cx.features.arrow {
        let params = parameters(cx.ast, params, FormalParameterKind::ArrowFormalParameters);
        return Expr::ArrowFunctionExpression(cx.ast.arrow_function_expression(
            DUMMY_SP,
            true,
            false,
            None,
            params,
            None,
            ArrowFunctionBody::Expr(value),
        ));
    }
    let ret = cx.ast.stmt_return_statement(DUMMY_SP, Some(value));
    Expr::Function(function(
        cx.ast,
        None,
        params,
        SubRange::empty(),
        vec![ret],
        false,
    ))
}

pub(super) fn source_start(ast: &Ast<'_>) -> u32 {
    let program = ast.root_program();
    program
        .directives(ast)
        .iter()
        .next()
        .map(|slot| ast.get_node_in_sub_range(slot).span(ast).start)
        .or_else(|| {
            program
                .body(ast)
                .iter()
                .map(|slot| ast.get_node_in_sub_range(slot).span(ast))
                .find(|span| *span != DUMMY_SP)
                .map(|span| span.start)
        })
        .unwrap_or_else(|| program.span(ast).end)
}

pub(super) fn emit_wrapper(ast: &mut Ast<'_>, expression: Expr) {
    let program = ast.root_program();
    let span = Span::new(source_start(ast), program.span(ast).end);
    let statement = ast.stmt_expression_statement(span, expression);
    let body = ast.add_sub_range([statement]);
    program.set_body(ast, body);
    program.set_directives(ast, SubRange::empty());
}

pub(super) fn string(ast: &mut Ast<'_>, span: Span, value: &str) -> Expr {
    let value = ast.add_utf8(value).into();
    Expr::StringLiteral(ast.string_literal(span, value, AstUtf8::EMPTY))
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/wtf8.rs#L19>.
pub(super) fn source_literal(ast: &mut Ast<'_>, source: StringLiteral) -> StringLiteral {
    let value = ast.get_wtf8(source.value(ast)).to_string_lossy();
    let value = ast.add_utf8(&value).into();
    ast.string_literal(source.span(ast), value, AstUtf8::EMPTY)
}

pub(super) fn array(ast: &mut Ast<'_>, elements: impl IntoIterator<Item = Expr>) -> Expr {
    let elements = ast.add_sub_range(elements.into_iter().map(|expr| Some(Argument::Expr(expr))));
    ast.expr_array_expression(DUMMY_SP, elements)
}

pub(super) fn object(
    ast: &mut Ast<'_>,
    properties: impl IntoIterator<Item = ObjectPropertyKind>,
) -> Expr {
    let properties = ast.add_sub_range(properties);
    ast.expr_object_expression(DUMMY_SP, properties)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/util.rs#L361>.
pub(super) fn prop_name(ast: &mut Ast<'_>, name: &str, span: Span) -> PropertyKey {
    if is_identifier(name.as_bytes()) {
        let name = ast.add_utf8(name);
        ast.property_key_identifier_name(span, name)
    } else {
        PropertyKey::Expr(string(ast, span, name))
    }
}

pub(super) fn property(
    ast: &mut Ast<'_>,
    name: &str,
    value: Expr,
    kind: PropertyKind,
) -> ObjectPropertyKind {
    let key = prop_name(ast, name, DUMMY_SP);
    ast.object_property_kind_object_property(DUMMY_SP, key, value, kind, false, false, false)
}

pub(super) fn named_member(ast: &mut Ast<'_>, object: Expr, name: AstUtf8, span: Span) -> Expr {
    let name = ast.get_utf8(name);
    let key = prop_name(ast, name, DUMMY_SP);
    ast.expr_member_expression(span, object, key, key.is_expr(ast), false)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/util.rs#L217>.
pub(super) fn use_strict(
    ast: &mut Ast<'_>,
    directives: SubRange<Directive>,
) -> SubRange<Directive> {
    if directives.iter().any(|slot| {
        let directive = ast.get_node_in_sub_range(slot);
        ast.get_utf8(directive.value(ast)) == "use strict"
    }) {
        return directives;
    }
    let mut directives: Vec<_> = directives
        .iter()
        .map(|slot| ast.get_node_in_sub_range(slot))
        .collect();
    let value = ast.add_utf8("use strict");
    let literal = ast.string_literal(DUMMY_SP, value.into(), AstUtf8::EMPTY);
    directives.push(ast.directive(DUMMY_SP, literal, value));
    ast.add_sub_range(directives)
}

fn parameters(
    ast: &mut Ast<'_>,
    bindings: impl IntoIterator<Item = BindingIdentifier>,
    kind: FormalParameterKind,
) -> FormalParameters {
    let items: Vec<_> = bindings
        .into_iter()
        .map(|binding| {
            FormalParameterItem::FormalParameter(ast.formal_parameter(
                DUMMY_SP,
                SubRange::empty(),
                FormalParameterPattern::BindingIdentifier(binding),
                None,
                None,
                false,
            ))
        })
        .collect();
    let items = ast.add_sub_range(items);
    ast.formal_parameters(DUMMY_SP, items, None, kind)
}

pub(super) fn function(
    ast: &mut Ast<'_>,
    id: Option<BindingIdentifier>,
    params: impl IntoIterator<Item = BindingIdentifier>,
    directives: SubRange<Directive>,
    body: Vec<Stmt>,
    is_async: bool,
) -> Function {
    let params = parameters(ast, params, FormalParameterKind::FormalParameters);
    let statements = ast.add_sub_range(body);
    let body = ast.function_body(DUMMY_SP, directives, statements);
    let kind = if id.is_some() {
        FunctionType::FunctionDeclaration
    } else {
        FunctionType::FunctionExpression
    };
    ast.function(
        DUMMY_SP, kind, id, None, None, params, None, body, false, is_async,
    )
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/util.rs#L157>.
pub(super) fn local_name_for_src(src: &str) -> String {
    let src = src.rsplit('/').next().unwrap();
    let src = src
        .strip_suffix(".js")
        .or_else(|| src.strip_suffix(".mjs"))
        .unwrap_or(src);
    let mut name = String::with_capacity(src.len() + 1);
    for c in src.chars() {
        let mut buffer = [0; 5];
        buffer[0] = b'_';
        let length = c.encode_utf8(&mut buffer[1..]).len();
        if is_identifier(&buffer[..=length]) {
            name.push(c);
        }
    }
    if !name.starts_with('_') {
        name.insert(0, '_');
    }
    name
}

pub(super) fn binding_decl(
    ast: &mut Ast<'_>,
    kind: VariableKind,
    id: BindingIdentifier,
    init: Option<Expr>,
) -> Stmt {
    Stmt::Declaration(build::variable(ast, kind, id, init, DUMMY_SP))
}

pub(super) fn meta_member(ast: &Ast<'_>, member: MemberExpression) -> bool {
    member
        .object(ast)
        .as_meta_property(ast)
        .is_some_and(|meta| {
            ast.get_utf8(meta.meta(ast).name(ast)) == "import"
                && ast.get_utf8(meta.property(ast).name(ast)) == "meta"
        })
}

pub(super) fn member_name<'a>(
    ast: &Ast<'a>,
    member: MemberExpression,
) -> Option<std::borrow::Cow<'a, str>> {
    let property = member.property(ast);
    if let Some(id) = property.as_identifier_name(ast) {
        return Some(ast.get_utf8(id.name(ast)).into());
    }
    let string = property.as_expr(ast)?.as_string_literal(ast)?;
    Some(ast.get_wtf8(string.value(ast)).to_string_lossy())
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/util.rs#L201>.
pub(super) fn define_es_module(cx: &mut ModuleCtx<'_, '_>, exports: BindingIdentifier) -> Stmt {
    let target = cx.reference(exports);
    let key = string(cx.ast, DUMMY_SP, "__esModule");
    let value = cx.ast.expr_boolean_literal(DUMMY_SP, true);
    let value = property(cx.ast, "value", value, PropertyKind::Init);
    object_define_property(cx, target, key, [value])
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/util.rs#L184>.
fn object_define_property(
    cx: &mut ModuleCtx<'_, '_>,
    target: Expr,
    key: Expr,
    descriptor: impl IntoIterator<Item = ObjectPropertyKind>,
) -> Stmt {
    let object_ident = cx.global("Object");
    let define = build::member(cx.ast, object_ident, "defineProperty");
    let descriptor = object(cx.ast, descriptor);
    let expression = build::call(cx.ast, define, [target, key, descriptor], false);
    cx.ast.stmt_expression_statement(DUMMY_SP, expression)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/util.rs#L221>.
fn object_define_enumerable(
    cx: &mut ModuleCtx<'_, '_>,
    target: Expr,
    key: Expr,
    prop: ObjectPropertyKind,
) -> Stmt {
    let value = cx.ast.expr_boolean_literal(DUMMY_SP, true);
    let enumerable = property(cx.ast, "enumerable", value, PropertyKind::Init);
    object_define_property(cx, target, key, [enumerable, prop])
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/util.rs#L314>.
pub(super) fn sort_export_bindings(ast: &Ast<'_>, bindings: &mut [LocalExportEntry]) {
    bindings.sort_unstable_by_key(|binding| ast.get_utf8(binding.exported.name));
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/util.rs#L318>.
pub(super) fn emit_export_stmts(
    cx: &mut ModuleCtx<'_, '_>,
    exports: BindingIdentifier,
    bindings: Vec<LocalExportEntry>,
    body: &mut Vec<Stmt>,
) {
    let target = cx.reference(exports);
    if let [binding] = bindings.as_slice() {
        let key = string(
            cx.ast,
            binding.exported.span,
            cx.ast.get_utf8(binding.exported.name),
        );
        let get = prop_function(cx, *binding);
        body.push(object_define_enumerable(cx, target, key, get));
        return;
    }

    let all = bindings
        .into_iter()
        .map(|binding| getter_function(cx, binding))
        .collect::<Vec<_>>();
    let all = object(cx.ast, all);
    let binding = cx.private("_export");
    let export = cx.reference(binding);
    body.push(esm_export(cx, binding));
    let export = build::call(cx.ast, export, [target, all], false);
    body.push(cx.ast.stmt_expression_statement(DUMMY_SP, export));
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/util.rs#L402>.
fn prop_function(cx: &mut ModuleCtx<'_, '_>, binding: LocalExportEntry) -> ObjectPropertyKind {
    let value = export_getter(cx, binding);
    property(cx.ast, "get", value, PropertyKind::Init)
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/util.rs#L426>.
fn getter_function(cx: &mut ModuleCtx<'_, '_>, binding: LocalExportEntry) -> ObjectPropertyKind {
    let value = export_getter(cx, binding);
    let name = cx.ast.get_utf8(binding.exported.name);
    let key = prop_name(cx.ast, name, binding.exported.span);
    cx.ast.object_property_kind_object_property(
        DUMMY_SP,
        key,
        value,
        PropertyKind::Get,
        false,
        false,
        false,
    )
}

fn export_getter(cx: &mut ModuleCtx<'_, '_>, binding: LocalExportEntry) -> Expr {
    let value = cx.resolver.symbol_reference(cx.ast, binding.local);
    let ret = cx.ast.stmt_return_statement(DUMMY_SP, Some(value));
    Expr::Function(function(
        cx.ast,
        None,
        [],
        SubRange::empty(),
        vec![ret],
        false,
    ))
}

// function _export(target, all) {
//     for (var name in all) Object.defineProperty(target, name, {
//         enumerable: true, get: Object.getOwnPropertyDescriptor(all, name).get
//     });
// }
/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_module/src/util.rs#L258>.
fn esm_export(cx: &mut ModuleCtx<'_, '_>, export: BindingIdentifier) -> Stmt {
    let target = cx.private("target");
    let all = cx.private("all");
    let name = cx.private("name");
    let object = cx.global("Object");
    let descriptor = build::member(cx.ast, object, "getOwnPropertyDescriptor");
    let all_ref = cx.reference(all);
    let name_ref = cx.reference(name);
    let descriptor = build::call(cx.ast, descriptor, [all_ref, name_ref], false);
    let get = build::member(cx.ast, descriptor, "get");
    let get = property(cx.ast, "get", get, PropertyKind::Init);
    let target_ref = cx.reference(target);
    let name_ref = cx.reference(name);
    let stmt = object_define_enumerable(cx, target_ref, name_ref, get);
    let var = build::variable(cx.ast, VariableKind::Var, name, None, DUMMY_SP);
    let left = ForStatementLeft::VariableDeclaration(var.as_variable_declaration(cx.ast).unwrap());
    let right = cx.reference(all);
    let stmt = cx.ast.stmt_for_in_statement(DUMMY_SP, left, right, stmt);
    let function = function(
        cx.ast,
        Some(export),
        [target, all],
        SubRange::empty(),
        vec![stmt],
        false,
    );
    Stmt::Declaration(Decl::Function(function))
}
