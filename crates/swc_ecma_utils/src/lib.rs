use swc_experimental_ecma_ast::*;
use swc_experimental_ecma_semantic::resolver::Semantic;

#[derive(Clone, Copy)]
pub struct ExprCtx<'visit, 'ast> {
    pub ast: &'visit Ast<'ast>,
    pub semantic: &'visit Semantic,

    /// True for argument of `typeof`.
    pub is_unresolved_ref_safe: bool,

    /// True if we are in the strict mode. This will be set to `true` for
    /// statements **after** `'use strict'`
    pub in_strict: bool,

    /// Remaining depth of the current expression. If this is 0, it means the
    /// function should not operate and return the safe value.
    ///
    /// Default value is `4`
    pub remaining_depth: u8,
}

impl ExprCtx<'_, '_> {
    pub fn consume_depth(self) -> Option<Self> {
        if self.remaining_depth == 0 {
            return None;
        }

        Some(Self {
            remaining_depth: self.remaining_depth - 1,
            ..self
        })
    }
}

pub trait ExprExt {
    fn is_pure_callee(&self, ctx: ExprCtx<'_, '_>) -> bool;
    fn may_have_side_effects(&self, ctx: ExprCtx<'_, '_>) -> bool;
    fn is_global_ref_to(&self, ctx: ExprCtx<'_, '_>, id: &str) -> bool;
}

impl ExprExt for Expr {
    fn may_have_side_effects(&self, ctx: ExprCtx<'_, '_>) -> bool {
        let Some(ctx) = ctx.consume_depth() else {
            return true;
        };
        let ast = ctx.ast;

        if self.is_pure_callee(ctx) {
            return false;
        }

        match *self {
            Expr::Ident(i) => {
                if ctx.is_unresolved_ref_safe {
                    return false;
                }

                if ctx.semantic.node_scope(i, ast) == ctx.semantic.unresolved_scope_id() {
                    !matches!(
                        i.sym(ast).as_str(ast),
                        "Infinity"
                            | "NaN"
                            | "Math"
                            | "undefined"
                            | "Object"
                            | "Array"
                            | "Promise"
                            | "Boolean"
                            | "Number"
                            | "String"
                            | "BigInt"
                            | "Error"
                            | "RegExp"
                            | "Function"
                            | "document"
                    )
                } else {
                    false
                }
            }

            Expr::Lit(..) | Expr::This(..) | Expr::PrivateName(..) => false,

            Expr::Paren(e) => e.expr(ast).may_have_side_effects(ctx),

            // Function expression does not have any side effect if it's not used.
            Expr::Fn(..) | Expr::Arrow(..) => false,

            // It's annoying to pass in_strict
            Expr::Class(c) => class_has_side_effect(ctx, c.class(ast)),
            Expr::Array(arr) => arr.elems(ast).iter().any(|elem| {
                ast.get_node_in_sub_range(elem).is_some_and(|elem| {
                    elem.spread(ast).is_some() || elem.expr(ast).may_have_side_effects(ctx)
                })
            }),
            Expr::Unary(u) => match u.op(ast) {
                UnaryOp::Delete => true,
                _ => u.arg(ast).may_have_side_effects(ctx),
            },
            Expr::Bin(bin) => {
                bin.left(ast).may_have_side_effects(ctx)
                    || bin.right(ast).may_have_side_effects(ctx)
            }

            Expr::Member(member)
                if matches!(
                    member.obj(ast),
                    Expr::Object(_) | Expr::Fn(_) | Expr::Arrow(_) | Expr::Class(_)
                ) =>
            {
                let obj = member.obj(ast);
                if obj.may_have_side_effects(ctx) {
                    return true;
                }
                match obj {
                    Expr::Class(c)
                        if c.class(ast).body(ast).iter().any(|member| {
                            matches!(
                                ast.get_node_in_sub_range(member),
                                ClassMember::Method(m)
                                    if (m.kind(ast) == MethodKind::Getter
                                        || m.kind(ast) == MethodKind::Setter)
                                        && m.is_static(ast)
                            )
                        }) =>
                    {
                        return true;
                    }
                    Expr::Object(obj_lit) => {
                        let can_have_side_effect = |prop| match ast.get_node_in_sub_range(prop) {
                            PropOrSpread::Spread(_) => true,
                            PropOrSpread::Prop(prop) => match prop {
                                Prop::Getter(_) | Prop::Setter(_) | Prop::Method(_) => true,
                                Prop::Shorthand(ident) => ident.sym(ast).as_str(ast) == "__proto__",
                                Prop::KeyValue(kv) => match kv.key(ast) {
                                    PropName::Ident(ident_name) => {
                                        ident_name.sym(ast).as_str(ast) == "__proto__"
                                    }
                                    PropName::Str(str_lit) => {
                                        str_lit.value(ast).as_wtf8(ast).as_str()
                                            == Some("__proto__")
                                    }
                                    PropName::Computed(_) => true,
                                    _ => false,
                                },
                                _ => false,
                            },
                        };
                        if obj_lit.props(ast).iter().any(can_have_side_effect) {
                            return true;
                        }
                    }
                    _ => {}
                };

                match member.prop(ast) {
                    MemberProp::Computed(c) => c.expr(ast).may_have_side_effects(ctx),
                    MemberProp::Ident(_) | MemberProp::PrivateName(_) => false,
                }
            }

            // TODO
            Expr::Tpl(_) => true,
            Expr::TaggedTpl(_) => true,
            Expr::MetaProp(_) => true,

            Expr::Await(_)
            | Expr::Yield(_)
            | Expr::Member(_)
            | Expr::SuperProp(_)
            | Expr::Import(_)
            | Expr::Update(_)
            | Expr::Assign(_) => true,

            Expr::OptChain(opt) if matches!(opt.base(ast), OptChainBase::Member(_)) => true,

            // A new expression is side-effect free if callee is pure for `new` and args are
            // side-effect free. Note: we use is_pure_new_callee instead of is_pure_callee because
            // class expressions are valid for `new` but calling them throws TypeError.
            Expr::New(new_expr) if is_pure_new_callee(new_expr.callee(ast), ctx) => {
                new_expr.args(ast).iter().any(|arg| {
                    ast.get_node_in_sub_range(arg)
                        .expr(ast)
                        .may_have_side_effects(ctx)
                })
            }

            Expr::New(_) => true,

            Expr::Call(call_expr) => {
                let Callee::Expr(callee) = call_expr.callee(ast) else {
                    return true;
                };

                if callee.is_pure_callee(ctx) {
                    call_expr.args(ast).iter().any(|arg| {
                        ast.get_node_in_sub_range(arg)
                            .expr(ast)
                            .may_have_side_effects(ctx)
                    })
                } else {
                    true
                }
            }

            Expr::OptChain(opt) => match opt.base(ast) {
                OptChainBase::Call(call) if call.callee(ast).is_pure_callee(ctx) => {
                    call.args(ast).iter().any(|arg| {
                        ast.get_node_in_sub_range(arg)
                            .expr(ast)
                            .may_have_side_effects(ctx)
                    })
                }
                _ => true,
            },

            Expr::Seq(seq) => seq
                .exprs(ast)
                .iter()
                .any(|expr| ast.get_node_in_sub_range(expr).may_have_side_effects(ctx)),

            Expr::Cond(cond) => {
                cond.test(ast).may_have_side_effects(ctx)
                    || cond.cons(ast).may_have_side_effects(ctx)
                    || cond.alt(ast).may_have_side_effects(ctx)
            }

            Expr::Object(obj) => obj.props(ast).iter().any(|node| {
                match ast.get_node_in_sub_range(node) {
                    PropOrSpread::Prop(node) => match node {
                        Prop::Shorthand(..) => false,
                        Prop::KeyValue(kv) => {
                            let key_has_side_effect = match kv.key(ast) {
                                PropName::Computed(e) => e.expr(ast).may_have_side_effects(ctx),
                                _ => false,
                            };

                            key_has_side_effect || kv.value(ast).may_have_side_effects(ctx)
                        }
                        Prop::Getter(p) => match p.key(ast) {
                            PropName::Computed(e) => e.expr(ast).may_have_side_effects(ctx),
                            _ => false,
                        },
                        Prop::Setter(p) => match p.key(ast) {
                            PropName::Computed(e) => e.expr(ast).may_have_side_effects(ctx),
                            _ => false,
                        },
                        Prop::Method(p) => match p.key(ast) {
                            PropName::Computed(e) => e.expr(ast).may_have_side_effects(ctx),
                            _ => false,
                        },
                        Prop::Assign(_) => true,
                    },
                    // may trigger getter
                    PropOrSpread::Spread(_) => true,
                }
            }),

            Expr::JSXMember(..)
            | Expr::JSXNamespacedName(..)
            | Expr::JSXEmpty(..)
            | Expr::JSXElement(..)
            | Expr::JSXFragment(..) => true,

            Expr::Invalid(..) => true,
        }
    }

    fn is_pure_callee(&self, ctx: ExprCtx<'_, '_>) -> bool {
        let ast = ctx.ast;
        if self.is_global_ref_to(ctx, "Date") {
            return true;
        }

        match *self {
            Expr::Member(member) => {
                let obj = member.obj(ast);
                let prop = member.prop(ast);

                if let MemberProp::Ident(prop) = prop {
                    // Some methods of string are pure
                    fn is_pure_str_method(method: &str) -> bool {
                        matches!(
                            method,
                            "charAt"
                                | "charCodeAt"
                                | "concat"
                                | "endsWith"
                                | "includes"
                                | "indexOf"
                                | "lastIndexOf"
                                | "localeCompare"
                                | "slice"
                                | "split"
                                | "startsWith"
                                | "substr"
                                | "substring"
                                | "toLocaleLowerCase"
                                | "toLocaleUpperCase"
                                | "toLowerCase"
                                | "toString"
                                | "toUpperCase"
                                | "trim"
                                | "trimEnd"
                                | "trimStart"
                        )
                    }

                    if obj.is_global_ref_to(ctx, "Math") {
                        return true;
                    }

                    match obj {
                        Expr::Ident(ident) => ident.sym(ast).as_str(ast) == "Math",
                        Expr::Lit(Lit::Str(..)) => is_pure_str_method(prop.sym(ast).as_str(ast)),
                        Expr::Tpl(tpl) if tpl.exprs(ast).is_empty() => {
                            is_pure_str_method(prop.sym(ast).as_str(ast))
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }

            Expr::Fn(fn_expr) => {
                let function = fn_expr.function(ast);
                function
                    .params(ast)
                    .iter()
                    .all(|param| matches!(ast.get_node_in_sub_range(param).pat(ast), Pat::Ident(_)))
                    && function.body(ast).stmts(ast).is_empty()
            }

            _ => false,
        }
    }

    fn is_global_ref_to(&self, ctx: ExprCtx<'_, '_>, id: &str) -> bool {
        match *self {
            Expr::Ident(i) => {
                ctx.semantic.node_scope(i, ctx.ast) == ctx.semantic.unresolved_scope_id()
                    && i.sym(ctx.ast).as_str(ctx.ast) == id
            }
            _ => false,
        }
    }
}

pub trait StmtExt {
    fn may_have_side_effects(&self, ctx: ExprCtx<'_, '_>) -> bool;
}

impl StmtExt for Stmt {
    fn may_have_side_effects(&self, ctx: ExprCtx<'_, '_>) -> bool {
        let ast = ctx.ast;
        match *self {
            Stmt::Block(block_stmt) => block_stmt
                .stmts(ast)
                .iter()
                .any(|stmt| ast.get_node_in_sub_range(stmt).may_have_side_effects(ctx)),
            Stmt::Empty(_) => false,
            Stmt::Labeled(labeled_stmt) => labeled_stmt.body(ast).may_have_side_effects(ctx),
            Stmt::If(if_stmt) => {
                if_stmt.test(ast).may_have_side_effects(ctx)
                    || if_stmt.cons(ast).may_have_side_effects(ctx)
                    || if_stmt
                        .alt(ast)
                        .is_some_and(|stmt| stmt.may_have_side_effects(ctx))
            }
            Stmt::Switch(switch_stmt) => {
                switch_stmt.discriminant(ast).may_have_side_effects(ctx)
                    || switch_stmt.cases(ast).iter().any(|case| {
                        let case = ast.get_node_in_sub_range(case);
                        case.test(ast)
                            .is_some_and(|expr| expr.may_have_side_effects(ctx))
                            || case.cons(ast).iter().any(|stmt| {
                                ast.get_node_in_sub_range(stmt).may_have_side_effects(ctx)
                            })
                    })
            }
            Stmt::Try(try_stmt) => {
                try_stmt
                    .block(ast)
                    .stmts(ast)
                    .iter()
                    .any(|stmt| ast.get_node_in_sub_range(stmt).may_have_side_effects(ctx))
                    || try_stmt.handler(ast).is_some_and(|handler| {
                        handler
                            .body(ast)
                            .stmts(ast)
                            .iter()
                            .any(|stmt| ast.get_node_in_sub_range(stmt).may_have_side_effects(ctx))
                    })
                    || try_stmt.finalizer(ast).is_some_and(|finalizer| {
                        finalizer
                            .stmts(ast)
                            .iter()
                            .any(|stmt| ast.get_node_in_sub_range(stmt).may_have_side_effects(ctx))
                    })
            }
            Stmt::Decl(decl) => match decl {
                Decl::Class(class_decl) => class_has_side_effect(ctx, class_decl.class(ast)),
                Decl::Fn(_) => !ctx.in_strict,
                Decl::Var(var_decl) => var_decl.kind(ast) == VarDeclKind::Var,
                _ => false,
            },
            Stmt::Expr(expr_stmt) => expr_stmt.expr(ast).may_have_side_effects(ctx),
            _ => true,
        }
    }
}

pub fn class_has_side_effect(expr_ctx: ExprCtx<'_, '_>, class: Class) -> bool {
    let ast = expr_ctx.ast;
    if let Some(expr) = class.super_class(ast)
        && expr.may_have_side_effects(expr_ctx)
    {
        return true;
    }

    for member in class.body(ast).iter() {
        match ast.get_node_in_sub_range(member) {
            ClassMember::Method(method) => {
                if let PropName::Computed(key) = method.key(ast)
                    && key.expr(ast).may_have_side_effects(expr_ctx)
                {
                    return true;
                }
            }

            ClassMember::ClassProp(prop) => {
                if let PropName::Computed(key) = prop.key(ast)
                    && key.expr(ast).may_have_side_effects(expr_ctx)
                {
                    return true;
                }

                if let Some(value) = prop.value(ast)
                    && value.may_have_side_effects(expr_ctx)
                {
                    return true;
                }
            }
            ClassMember::PrivateProp(prop) => {
                if let Some(value) = prop.value(ast)
                    && value.may_have_side_effects(expr_ctx)
                {
                    return true;
                }
            }
            ClassMember::StaticBlock(block)
                if block.body(ast).stmts(ast).iter().any(|stmt| {
                    ast.get_node_in_sub_range(stmt)
                        .may_have_side_effects(expr_ctx)
                }) =>
            {
                return true;
            }
            _ => {}
        }
    }

    false
}

/// Check if a class expression is pure when used with `new`.
/// This is different from `is_pure_callee` because:
/// - Calling a class as a function (`(class {})()`) throws TypeError
/// - But `new (class {})()` can be pure if the class has no side effects
fn is_pure_new_callee(expr: Expr, ctx: ExprCtx<'_, '_>) -> bool {
    let ast = ctx.ast;
    match expr {
        // An empty function expression is also pure for `new`
        Expr::Fn(func) => {
            let function = func.function(ast);
            function
                .params(ast)
                .iter()
                .all(|param| matches!(ast.get_node_in_sub_range(param).pat(ast), Pat::Ident(_)))
                && function.body(ast).stmts(ast).is_empty()
        }

        // A class expression is pure for `new` if:
        // 1. It has no side effects from definition (computed keys, property initializers, static
        //    blocks)
        // 2. It has no super class (calling super() may have side effects)
        // 3. Either has no constructor, or constructor body is empty
        // 4. Has no instance properties (they are initialized in the constructor)
        Expr::Class(class_expr) => {
            let class = class_expr.class(ast);

            // Check for super class - calling super() may have side effects
            if class.super_class(ast).is_some() {
                return false;
            }

            // Check for side effects from class definition
            if class_has_side_effect(ctx, class) {
                return false;
            }

            // Check for instance properties (non-static) - they run during construction
            for member in class.body(ast).iter() {
                match ast.get_node_in_sub_range(member) {
                    ClassMember::ClassProp(prop) if !prop.is_static(ast) => return false,
                    ClassMember::PrivateProp(prop) if !prop.is_static(ast) => return false,
                    _ => {}
                }
            }

            // Check constructor - must be empty or not present
            for member in class.body(ast).iter() {
                if let ClassMember::Constructor(constructor) = ast.get_node_in_sub_range(member)
                    && let Some(body) = constructor.body(ast)
                    && !body.stmts(ast).is_empty()
                {
                    return false;
                }
            }

            true
        }

        _ => false,
    }
}
