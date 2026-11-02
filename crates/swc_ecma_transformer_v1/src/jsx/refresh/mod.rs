use rustc_hash::FxHashSet;
use swc_next_ecma_ast::*;

use crate::common::{build, resolver::context::Resolver};

use self::{
    hook::HookRegister,
    options::RefreshOptions,
    util::{
        argument_expr, collect_ident_in_jsx, is_body_arrow_fn, is_import_or_require,
        make_assign_stmt, span_to_snippet,
    },
};

mod hook;
pub(crate) mod options;
mod util;

type Id = (String, Option<SymbolId>);

struct Hoc {
    insert: bool,
    reg: Vec<(BindingIdentifier, Id)>,
    hook: Option<HocHook>,
}

struct HocHook {
    callee: Expr,
    rest_arg: Vec<Expr>,
}

enum Persist {
    Hoc(Hoc),
    Component(BindingIdentifier),
    None,
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/mod.rs#L36>.
fn get_persistent_id(ast: &Ast<'_>, ident: BindingIdentifier) -> Persist {
    if is_component_name(ast.get_utf8(ident.name(ast))) {
        Persist::Component(ident)
    } else {
        Persist::None
    }
}

fn is_component_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_uppercase())
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/mod.rs#L49>.
pub(super) fn hook<'a, 'ast>(
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    options: &'a RefreshOptions,
) -> Refresh<'a, 'ast> {
    Refresh {
        ast,
        resolver,
        options,
    }
}

pub(super) struct Refresh<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    resolver: &'a mut Resolver,
    options: &'a RefreshOptions,
}

impl Refresh<'_, '_> {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/mod.rs#L234>.
    pub(super) fn enter_module_items(&mut self, program: Program) {
        let should_reset = self.visit_module(program);
        self.handle_module_items(program, should_reset);
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/mod.rs#L251>.
    fn visit_module(&self, program: Program) -> bool {
        struct SpanVisitor<'a, 'ast> {
            ast: &'a Ast<'ast>,
            leading: FxHashSet<u32>,
            trailing: FxHashSet<u32>,
            should_reset: bool,
        }

        impl<'ast> Visit<'ast> for SpanVisitor<'_, 'ast> {
            fn ast(&self) -> &Ast<'ast> {
                self.ast
            }

            fn enter_node(&mut self, node: NodeKindData) {
                let span = self.ast.span(node.node_id());
                self.should_reset |= self.leading.contains(&span.start)
                    || self.trailing.contains(&span.start)
                    || span
                        .end
                        .checked_sub(1)
                        .is_some_and(|end| self.leading.contains(&end));
            }
        }

        let mut visitor = SpanVisitor {
            ast: self.ast,
            leading: FxHashSet::default(),
            trailing: FxHashSet::default(),
            should_reset: false,
        };
        for comment in self.ast.comments() {
            let text = comment.value(self.ast.source().as_bytes());
            if self.ast.get_utf8(text).contains("@refresh reset") {
                match comment.position {
                    CommentPosition::Leading => visitor.leading.insert(comment.attached_to),
                    CommentPosition::Trailing => visitor.trailing.insert(comment.attached_to),
                };
            }
        }
        if !visitor.leading.is_empty() || !visitor.trailing.is_empty() {
            program.directives(self.ast).visit_with(&mut visitor);
            program.body(self.ast).visit_with(&mut visitor);
        }
        visitor.should_reset
    }

    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/mod.rs#L295>.
    fn handle_module_items(&mut self, program: Program, should_reset: bool) {
        let module_items = program.body(self.ast);
        let used_in_jsx = collect_ident_in_jsx(self.ast, self.resolver, module_items);
        let mut items = Vec::with_capacity(module_items.len());
        let mut refresh_regs = Vec::new();
        let mut hook_visitor =
            HookRegister::new(self.ast, self.resolver, self.options, should_reset);

        for slot in module_items.iter() {
            let item = hook_visitor.ast.get_node_in_sub_range(slot);
            let persistent_id = match hook_visitor.ast.stmt_data(item) {
                StmtData::Declaration(decl) => {
                    get_persistent_id_from_decl(decl, &used_in_jsx, &mut hook_visitor)
                }
                StmtData::ExportNamedDeclaration(export) => export
                    .declaration(hook_visitor.ast)
                    .map_or(Persist::None, |decl| {
                        get_persistent_id_from_decl(decl, &used_in_jsx, &mut hook_visitor)
                    }),
                StmtData::ExportDefaultDeclaration(export) => {
                    let ast = &*hook_visitor.ast;
                    match ast.export_default_declaration_kind_data(export.declaration(ast)) {
                        ExportDefaultDeclarationKindData::Function(function) => function
                            .id(ast)
                            .map_or(Persist::None, |ident| get_persistent_id(ast, ident)),
                        ExportDefaultDeclarationKindData::Expr(expr) => {
                            if let Some(call) = expr.as_call_expression(ast) {
                                let handle = hook_visitor
                                    .resolver
                                    .private_binding(hook_visitor.ast, "_c");
                                let reg = vec![(handle, ("%default%".to_owned(), None))];
                                if let Persist::Hoc(Hoc { reg, hook, .. }) =
                                    get_persistent_id_from_possible_hoc(
                                        call,
                                        reg,
                                        &mut hook_visitor,
                                    )
                                {
                                    let ast = &mut *hook_visitor.ast;
                                    let resolver = &mut *hook_visitor.resolver;
                                    let expr =
                                        hook.map_or(expr, |hook| make_hook_reg(ast, expr, hook));
                                    let expr = make_assign_stmt(ast, resolver, handle, expr);
                                    export.set_declaration(
                                        ast,
                                        ExportDefaultDeclarationKind::Expr(expr),
                                    );
                                    Persist::Hoc(Hoc {
                                        insert: false,
                                        reg,
                                        hook: None,
                                    })
                                } else {
                                    Persist::None
                                }
                            } else {
                                Persist::None
                            }
                        }
                        _ => Persist::None,
                    }
                }
                _ => Persist::None,
            };

            if matches!(persistent_id, Persist::Hoc(_)) {
                items.push(item);
            } else {
                let item = item
                    .visit_mut_children_with(&mut hook_visitor)
                    .unwrap_or(item);
                items.push(item);
                items.append(&mut hook_visitor.extra_stmt);
            }

            let ast = &mut *hook_visitor.ast;
            let resolver = &mut *hook_visitor.resolver;
            match persistent_id {
                Persist::None => {}
                Persist::Component(persistent_id) => {
                    let handle = resolver.private_binding(ast, "_c");
                    let name = ast.get_utf8(persistent_id.name(ast)).to_owned();
                    let symbol = resolver.symbol_of(persistent_id.node_id());
                    refresh_regs.push((handle, (name, Some(symbol))));
                    let expr = resolver.symbol_reference(ast, symbol);
                    let expr = make_assign_stmt(ast, resolver, handle, expr);
                    items.push(ast.stmt_expression_statement(DUMMY_SP, expr));
                }
                Persist::Hoc(mut hoc) => {
                    hoc.reg.reverse();
                    if hoc.insert {
                        let (ident, (_, symbol)) = hoc.reg.last().unwrap();
                        let expr = resolver.symbol_reference(ast, symbol.unwrap());
                        let expr = make_assign_stmt(ast, resolver, *ident, expr);
                        items.push(ast.stmt_expression_statement(DUMMY_SP, expr));
                    }
                    refresh_regs.append(&mut hoc.reg);
                }
            }
        }

        if !hook_visitor.ident.is_empty() {
            items.insert(0, hook_visitor.gen_hook_handle());
        }

        let ast = &mut *hook_visitor.ast;
        let resolver = &mut *hook_visitor.resolver;
        if !refresh_regs.is_empty() {
            let decls = refresh_regs
                .iter()
                .map(|&(handle, _)| {
                    ast.variable_declarator(
                        DUMMY_SP,
                        BindingPattern::BindingIdentifier(handle),
                        None,
                        false,
                    )
                })
                .collect::<Vec<_>>();
            items.push(build::declaration(ast, VariableKind::Var, decls));
        }

        for (handle, (persistent_id, _)) in refresh_regs {
            let callee = resolver.marked_reference(ast, &self.options.refresh_reg, true);
            let handle = resolver.reference(ast, handle);
            let value = ast.add_utf8(&persistent_id).into();
            let name = ast.expr_string_literal(DUMMY_SP, value, AstUtf8::EMPTY);
            let call = build::call(ast, callee, [handle, name], false);
            items.push(ast.stmt_expression_statement(DUMMY_SP, call));
        }

        let items = ast.add_sub_range(items);
        program.set_body(ast, items);
    }
}

fn get_persistent_id_from_decl(
    decl: Decl,
    used_in_jsx: &FxHashSet<SymbolId>,
    hook_reg: &mut HookRegister<'_, '_>,
) -> Persist {
    let ast = &*hook_reg.ast;
    match ast.decl_data(decl) {
        DeclData::Function(function) => function
            .id(ast)
            .map_or(Persist::None, |ident| get_persistent_id(ast, ident)),
        DeclData::VariableDeclaration(decl) => {
            get_persistent_id_from_var_decl(decl, used_in_jsx, hook_reg)
        }
        _ => Persist::None,
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/mod.rs#L76>.
fn get_persistent_id_from_var_decl(
    var_decl: VariableDeclaration,
    used_in_jsx: &FxHashSet<SymbolId>,
    hook_reg: &mut HookRegister<'_, '_>,
) -> Persist {
    let ast = &*hook_reg.ast;
    let decls = var_decl.declarators(ast);
    if decls.len() != 1 {
        return Persist::None;
    }
    let decl = decls.get_node(ast, 0).unwrap();
    let (Some(binding), Some(init_expr)) =
        (decl.id(ast).as_binding_identifier(ast), decl.init(ast))
    else {
        return Persist::None;
    };

    let symbol = hook_reg.resolver.symbol_of(binding.node_id());
    if used_in_jsx.contains(&symbol)
        && !is_import_or_require(ast, init_expr)
        && matches!(
            ast.expr_data(init_expr),
            ExprData::ArrowFunctionExpression(_)
                | ExprData::Function(_)
                | ExprData::TaggedTemplateExpression(_)
                | ExprData::CallExpression(_)
        )
    {
        return Persist::Component(binding);
    }

    let Persist::Component(persistent_id) = get_persistent_id(ast, binding) else {
        return Persist::None;
    };
    match ast.expr_data(init_expr) {
        ExprData::Function(_) => Persist::Component(persistent_id),
        ExprData::ArrowFunctionExpression(arrow) if !is_body_arrow_fn(ast, arrow.body(ast)) => {
            Persist::Component(persistent_id)
        }
        ExprData::CallExpression(call) => {
            let name = ast.get_utf8(persistent_id.name(ast)).to_owned();
            let handle = hook_reg.resolver.private_binding(hook_reg.ast, "_c");
            let res = get_persistent_id_from_possible_hoc(
                call,
                vec![(handle, (name, Some(symbol)))],
                hook_reg,
            );
            if let Persist::Hoc(Hoc {
                insert,
                reg,
                hook: Some(hook),
            }) = res
            {
                let expr = make_hook_reg(hook_reg.ast, init_expr, hook);
                decl.set_init(hook_reg.ast, Some(expr));
                Persist::Hoc(Hoc {
                    insert,
                    reg,
                    hook: None,
                })
            } else {
                res
            }
        }
        _ => Persist::None,
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/mod.rs#L141>.
fn get_persistent_id_from_possible_hoc(
    call_expr: CallExpression,
    mut reg: Vec<(BindingIdentifier, Id)>,
    hook_reg: &mut HookRegister<'_, '_>,
) -> Persist {
    let ast = &*hook_reg.ast;
    let args = call_expr.arguments(ast);
    let Some(first_arg) = args.get_node(ast, 0) else {
        return Persist::None;
    };
    let first = argument_expr(ast, first_arg);
    let hoc_name = match ast.expr_data(call_expr.callee(ast)) {
        ExprData::IdentifierReference(ident) => ast.get_utf8(ident.name(ast)),
        ExprData::MemberExpression(member) => span_to_snippet(ast, member.span(ast)),
        _ => return Persist::None,
    };
    let (_, (persistent_id, _)) = reg.last().unwrap();
    let reg_str = (format!("{persistent_id}${hoc_name}"), None);

    match ast.expr_data(first) {
        ExprData::CallExpression(call) => {
            let reg_ident = hook_reg.resolver.private_binding(hook_reg.ast, "_c");
            reg.push((reg_ident, reg_str));
            if let Persist::Hoc(hoc) = get_persistent_id_from_possible_hoc(call, reg, hook_reg) {
                let ast = &mut *hook_reg.ast;
                let first = if let Some(hook) = &hoc.hook {
                    let resolver = &mut *hook_reg.resolver;
                    let callee = build::clone_expr(ast, resolver, hook.callee);
                    let rest_arg = hook
                        .rest_arg
                        .iter()
                        .map(|&arg| build::clone_expr(ast, resolver, arg))
                        .collect();
                    make_hook_reg(ast, first, HocHook { callee, rest_arg })
                } else {
                    first
                };
                let expr = make_assign_stmt(ast, hook_reg.resolver, reg_ident, first);
                set_first_arg(ast, call_expr, args, first_arg, expr);

                Persist::Hoc(hoc)
            } else {
                Persist::None
            }
        }
        ExprData::Function(_) | ExprData::ArrowFunctionExpression(_) => {
            let reg_ident = hook_reg.resolver.private_binding(hook_reg.ast, "_c");
            let replacement = first.visit_mut_with(hook_reg);
            let ast = &mut *hook_reg.ast;
            let resolver = &mut *hook_reg.resolver;
            let hook = replacement.and_then(|first| {
                let call = first.as_call_expression(ast)?;
                let callee = build::clone_expr(ast, resolver, call.callee(ast));
                let rest_arg = call
                    .arguments(ast)
                    .iter()
                    .skip(1)
                    .map(|slot| argument_expr(ast, ast.get_node_in_sub_range(slot)))
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|arg| build::clone_expr(ast, resolver, arg))
                    .collect();
                Some(HocHook { callee, rest_arg })
            });
            let expr = make_assign_stmt(ast, resolver, reg_ident, replacement.unwrap_or(first));
            set_first_arg(ast, call_expr, args, first_arg, expr);

            reg.push((reg_ident, reg_str));
            Persist::Hoc(Hoc {
                reg,
                insert: true,
                hook,
            })
        }
        ExprData::IdentifierReference(ident)
            if is_component_name(ast.get_utf8(ident.name(ast))) =>
        {
            Persist::Hoc(Hoc {
                reg,
                insert: true,
                hook: None,
            })
        }
        _ => Persist::None,
    }
}

fn set_first_arg(
    ast: &mut Ast<'_>,
    call: CallExpression,
    args: SubRange<Argument>,
    first_arg: Argument,
    expr: Expr,
) {
    if let Some(spread) = first_arg.as_spread_element(ast) {
        spread.set_argument(ast, expr);
    } else {
        let args = ast.add_sub_range(
            [Argument::Expr(expr)]
                .into_iter()
                .chain(
                    args.iter()
                        .skip(1)
                        .map(|slot| ast.get_node_in_sub_range(slot)),
                )
                .collect::<Vec<_>>(),
        );
        call.set_arguments(ast, args);
    }
}

/// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_transforms_react/src/refresh/mod.rs#L497>.
fn make_hook_reg(ast: &mut Ast<'_>, expr: Expr, hook: HocHook) -> Expr {
    let span = expr.span(ast);
    let args = ast.add_sub_range([expr].into_iter().chain(hook.rest_arg).map(Argument::Expr));
    ast.expr_call_expression(span, hook.callee, None, args, false, false)
}
