use super::Config;
use crate::common::super_field::prop_arg;
use crate::common::{TraverseCtx, build as b};
use swc_next_ecma_ast::*;

pub(super) enum MemberInit {
    PubProp(PubProp),
    PrivProp(PrivProp),
    PrivMethod(PrivMethod),
    PrivAccessor(PrivAccessor),
    StaticBlock(Expr),
}

pub(super) struct PubProp {
    pub name: PropertyKey,
    pub computed: bool,
    pub value: Expr,
}

pub(super) struct PrivProp {
    pub name: BindingIdentifier,
    pub value: Expr,
}

pub(super) struct PrivMethod {
    pub name: BindingIdentifier,
    pub fn_name: BindingIdentifier,
}

pub(super) struct PrivAccessor {
    pub name: BindingIdentifier,
    pub getter: Option<BindingIdentifier>,
    pub setter: Option<BindingIdentifier>,
}

#[derive(Default)]
pub(super) struct MemberInitRecord {
    record: Vec<MemberInit>,
}

impl MemberInitRecord {
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/member_init.rs#L61>.
    pub(super) fn push(&mut self, ast: &Ast<'_>, init: MemberInit) -> bool {
        if let MemberInit::PrivAccessor(accessor) = &init
            && let Some(previous) = self.record.iter_mut().find_map(|item| match item {
                MemberInit::PrivAccessor(previous)
                    if ast.get_utf8(previous.name.name(ast))
                        == ast.get_utf8(accessor.name.name(ast)) =>
                {
                    Some(previous)
                }
                _ => None,
            })
        {
            previous.getter = previous.getter.or(accessor.getter);
            previous.setter = previous.setter.or(accessor.setter);
            return false;
        }
        self.record.push(init);
        true
    }

    pub(super) fn is_empty(&self) -> bool {
        self.record.is_empty()
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/member_init.rs#L79>.
    pub(super) fn into_init(
        self,
        ast: &mut Ast<'_>,
        ctx: &mut TraverseCtx<'_>,
        options: Config,
    ) -> Vec<Expr> {
        let mut normal_init = Vec::new();
        let mut value_init = Vec::new();
        for init in self.record {
            let is_value_init = matches!(init, MemberInit::PubProp(_) | MemberInit::PrivProp(_));
            let this = ast.expr_this_expression(DUMMY_SP);
            let expression = match init {
                MemberInit::PubProp(PubProp {
                    name,
                    computed,
                    value,
                }) => public(ast, ctx, options, this, name, computed, value),
                MemberInit::PrivProp(PrivProp { name, value }) => {
                    let descriptor = get_value_desc(ast, value);
                    private_init(
                        ast,
                        ctx,
                        options,
                        this,
                        name,
                        descriptor,
                        "class_private_field_init",
                    )
                }
                MemberInit::PrivAccessor(PrivAccessor {
                    name,
                    getter: get,
                    setter: set,
                }) => {
                    let descriptor = get_accessor_desc(ast, ctx, get, set);
                    private_init(
                        ast,
                        ctx,
                        options,
                        this,
                        name,
                        descriptor,
                        "class_private_field_init",
                    )
                }
                MemberInit::PrivMethod(PrivMethod {
                    name,
                    fn_name: function,
                }) => {
                    let value = ctx.resolver.reference(ast, function);
                    let descriptor = object(ast, [("value", value)]);
                    if options.private_as_properties {
                        private_init(
                            ast,
                            ctx,
                            options,
                            this,
                            name,
                            descriptor,
                            "class_private_method_init",
                        )
                    } else {
                        let helper = ctx.helper(ast, "class_private_method_init");
                        let name = ctx.resolver.reference(ast, name);
                        b::call(ast, helper, [this, name], false)
                    }
                }
                MemberInit::StaticBlock(_) => unreachable!(),
            };
            if is_value_init {
                value_init.push(expression);
            } else {
                normal_init.push(expression);
            }
        }
        normal_init.extend(value_init);
        normal_init
    }
    /// SWC: <https://github.com/swc-project/swc/blob/3ad7952d41f6ee568ce1e7084a44e2b6aafa3752/crates/swc_ecma_compat_es2022/src/class_properties/member_init.rs#L197>.
    pub(super) fn into_init_static(
        self,
        ast: &mut Ast<'_>,
        ctx: &mut TraverseCtx<'_>,
        options: Config,
        class: BindingIdentifier,
    ) -> Vec<Stmt> {
        let mut normal_init = Vec::new();
        let mut value_init = Vec::new();
        for init in self.record {
            let is_value_init = matches!(
                init,
                MemberInit::PubProp(_) | MemberInit::PrivProp(_) | MemberInit::StaticBlock(_)
            );
            let target = ctx.resolver.reference(ast, class);
            let statement = match init {
                MemberInit::PubProp(PubProp {
                    name,
                    computed,
                    value,
                }) => {
                    let expression = public(ast, ctx, options, target, name, computed, value);
                    ast.stmt_expression_statement(DUMMY_SP, expression)
                }
                MemberInit::StaticBlock(expression) => {
                    ast.stmt_expression_statement(DUMMY_SP, expression)
                }
                init => {
                    let (name, descriptor) = match init {
                        MemberInit::PrivProp(PrivProp { name, value }) => {
                            (name, get_value_desc(ast, value))
                        }
                        MemberInit::PrivAccessor(PrivAccessor {
                            name,
                            getter: get,
                            setter: set,
                        }) => (name, get_accessor_desc(ast, ctx, get, set)),
                        MemberInit::PrivMethod(PrivMethod {
                            name,
                            fn_name: function,
                        }) => {
                            let value = ctx.resolver.reference(ast, function);
                            (name, object(ast, [("value", value)]))
                        }
                        _ => unreachable!(),
                    };
                    if options.private_as_properties {
                        let expression = private_init(
                            ast,
                            ctx,
                            options,
                            target,
                            name,
                            descriptor,
                            "class_private_field_init",
                        );
                        ast.stmt_expression_statement(DUMMY_SP, expression)
                    } else {
                        Stmt::Declaration(b::variable(
                            ast,
                            VariableKind::Var,
                            name,
                            Some(descriptor),
                            DUMMY_SP,
                        ))
                    }
                }
            };
            if is_value_init {
                value_init.push(statement);
            } else {
                normal_init.push(statement);
            }
        }
        normal_init.extend(value_init);
        normal_init
    }
}
fn private_init(
    ast: &mut Ast<'_>,
    ctx: &mut TraverseCtx<'_>,
    options: Config,
    target: Expr,
    name: BindingIdentifier,
    descriptor: Expr,
    helper: &'static str,
) -> Expr {
    let callee = if options.private_as_properties {
        obj_def_prop(ast, ctx)
    } else {
        ctx.helper(ast, helper)
    };
    let name = ctx.resolver.reference(ast, name);
    b::call(ast, callee, [target, name, descriptor], false)
}
fn public(
    ast: &mut Ast<'_>,
    ctx: &mut TraverseCtx<'_>,
    options: Config,
    target: Expr,
    key: PropertyKey,
    computed: bool,
    value: Expr,
) -> Expr {
    if options.set_public_fields {
        let member = ast.expr_member_expression(
            DUMMY_SP,
            target,
            key,
            computed || !key.is_identifier_name(ast),
            false,
        );
        b::assign(ast, member, value)
    } else {
        let key = prop_arg(ast, key);
        let callee = ctx.helper(ast, "define_property");
        b::call(ast, callee, [target, key, value], false)
    }
}
pub(super) fn object(
    ast: &mut Ast<'_>,
    values: impl IntoIterator<Item = (&'static str, Expr)>,
) -> Expr {
    let properties: Vec<_> = values
        .into_iter()
        .map(|(name, value)| b::property(ast, name, value))
        .collect();
    let properties = ast.add_sub_range(properties);
    ast.expr_object_expression(DUMMY_SP, properties)
}
fn get_value_desc(ast: &mut Ast<'_>, value: Expr) -> Expr {
    let true_ = ast.expr_boolean_literal(DUMMY_SP, true);
    object(ast, [("writable", true_), ("value", value)])
}
fn get_accessor_desc(
    ast: &mut Ast<'_>,
    ctx: &mut TraverseCtx<'_>,
    get: Option<BindingIdentifier>,
    set: Option<BindingIdentifier>,
) -> Expr {
    let get = if let Some(binding) = get {
        ctx.resolver.reference(ast, binding)
    } else {
        b::undefined(ast)
    };
    let set = if let Some(binding) = set {
        ctx.resolver.reference(ast, binding)
    } else {
        b::undefined(ast)
    };
    object(ast, [("get", get), ("set", set)])
}
fn obj_def_prop(ast: &mut Ast<'_>, ctx: &mut TraverseCtx<'_>) -> Expr {
    let object = ctx.resolver.marked_reference(ast, "Object", true);
    b::member(ast, object, "defineProperty")
}
