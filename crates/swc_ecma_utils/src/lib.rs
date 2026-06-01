use swc_experimental_ecma_ast::*;
use swc_experimental_ecma_semantic::resolver::Semantic;

#[derive(Clone, Copy)]
pub struct ExprCtx<'a> {
    pub semantic: &'a Semantic,

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

impl ExprCtx<'_> {
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
    fn is_pure_callee(&self, ctx: ExprCtx<'_>) -> bool;
    fn may_have_side_effects(&self, ctx: ExprCtx<'_>) -> bool;
    fn is_global_ref_to(&self, ctx: ExprCtx<'_>, id: &str) -> bool;
}

impl<'a> ExprExt for Expr<'a> {
    fn may_have_side_effects(&self, ctx: ExprCtx<'_>) -> bool {
        let Some(ctx) = ctx.consume_depth() else {
            return true;
        };

        if self.is_pure_callee(ctx) {
            return false;
        }

        match self {
            Expr::Ident(i) => {
                if ctx.is_unresolved_ref_safe {
                    return false;
                }

                if ctx.semantic.node_scope(i) == ctx.semantic.unresolved_scope_id() {
                    !matches!(
                        i.sym.as_str(),
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

            Expr::Paren(e) => e.expr.may_have_side_effects(ctx),

            // Function expression does not have any side effect if it's not used.
            Expr::Fn(..) | Expr::Arrow(..) => false,

            // It's annoying to pass in_strict
            Expr::Class(c) => class_has_side_effect(ctx, &c.class),
            Expr::Array(arr) => arr
                .elems
                .iter()
                .flatten()
                .any(|e| e.spread.is_some() || e.expr.may_have_side_effects(ctx)),
            Expr::Unary(u) => match u.op {
                UnaryOp::Delete => true,
                _ => u.arg.may_have_side_effects(ctx),
            },
            Expr::Bin(bin) => {
                bin.left.may_have_side_effects(ctx) || bin.right.may_have_side_effects(ctx)
            }

            Expr::Member(member)
                if matches!(
                    &member.obj,
                    Expr::Object(_) | Expr::Fn(_) | Expr::Arrow(_) | Expr::Class(_)
                ) =>
            {
                let obj = &member.obj;
                if obj.may_have_side_effects(ctx) {
                    return true;
                }
                match obj {
                    Expr::Class(c)
                        if c.class.body.iter().any(|member| {
                            matches!(
                                member,
                                ClassMember::Method(m)
                                    if (m.kind == MethodKind::Getter
                                        || m.kind == MethodKind::Setter)
                                        && m.is_static
                            )
                        }) =>
                    {
                        return true;
                    }
                    Expr::Object(obj_lit) => {
                        let can_have_side_effect = |prop: &PropOrSpread<'_>| match prop {
                            PropOrSpread::Spread(_) => true,
                            PropOrSpread::Prop(prop) => match &**prop {
                                Prop::Getter(_) | Prop::Setter(_) | Prop::Method(_) => true,
                                Prop::Shorthand(ident) => ident.sym == "__proto__",
                                Prop::KeyValue(kv) => match &kv.key {
                                    PropName::Ident(ident_name) => ident_name.sym == "__proto__",
                                    PropName::Str(str_lit) => {
                                        str_lit.value.as_wtf8().as_str() == Some("__proto__")
                                    }
                                    PropName::Computed(_) => true,
                                    _ => false,
                                },
                                _ => false,
                            },
                        };
                        if obj_lit.props.iter().any(can_have_side_effect) {
                            return true;
                        }
                    }
                    _ => {}
                };

                match &member.prop {
                    MemberProp::Computed(c) => c.expr.may_have_side_effects(ctx),
                    MemberProp::Ident(_) | MemberProp::PrivateName(_) => false,
                }
            }

            //TODO
            Expr::Tpl(_) => true,
            Expr::TaggedTpl(_) => true,
            Expr::MetaProp(_) => true,

            Expr::Await(_)
            | Expr::Yield(_)
            | Expr::Member(_)
            | Expr::SuperProp(_)
            | Expr::Update(_)
            | Expr::Assign(_) => true,

            Expr::OptChain(opt) if matches!(&opt.base, OptChainBase::Member(_)) => true,

            // A new expression is side-effect free if callee is pure for `new` and args are
            // side-effect free. Note: we use is_pure_new_callee instead of is_pure_callee because
            // class expressions are valid for `new` but calling them throws TypeError.
            Expr::New(new_expr) if is_pure_new_callee(&new_expr.callee, ctx) => new_expr
                .args
                .as_ref()
                .is_some_and(|args| args.iter().any(|arg| arg.expr.may_have_side_effects(ctx))),

            Expr::New(_) => true,

            Expr::Call(call_expr) => {
                let Callee::Expr(callee) = &call_expr.callee else {
                    return true;
                };

                if callee.is_pure_callee(ctx) {
                    call_expr
                        .args
                        .iter()
                        .any(|arg| arg.expr.may_have_side_effects(ctx))
                } else {
                    true
                }
            }

            Expr::OptChain(opt) => match &opt.base {
                OptChainBase::Call(call) if call.callee.is_pure_callee(ctx) => call
                    .args
                    .iter()
                    .any(|arg| arg.expr.may_have_side_effects(ctx)),
                _ => true,
            },

            Expr::Seq(seq) => seq.exprs.iter().any(|e| e.may_have_side_effects(ctx)),

            Expr::Cond(cond) => {
                cond.test.may_have_side_effects(ctx)
                    || cond.cons.may_have_side_effects(ctx)
                    || cond.alt.may_have_side_effects(ctx)
            }

            Expr::Object(obj) => obj.props.iter().any(|node| match node {
                PropOrSpread::Prop(node) => match &**node {
                    Prop::Shorthand(..) => false,
                    Prop::KeyValue(kv) => {
                        let k = match &kv.key {
                            PropName::Computed(e) => e.expr.may_have_side_effects(ctx),
                            _ => false,
                        };

                        k || kv.value.may_have_side_effects(ctx)
                    }
                    Prop::Getter(p) => match &p.key {
                        PropName::Computed(e) => e.expr.may_have_side_effects(ctx),
                        _ => false,
                    },
                    Prop::Setter(p) => match &p.key {
                        PropName::Computed(e) => e.expr.may_have_side_effects(ctx),
                        _ => false,
                    },
                    Prop::Method(p) => match &p.key {
                        PropName::Computed(e) => e.expr.may_have_side_effects(ctx),
                        _ => false,
                    },
                    Prop::Assign(_) => true,
                },
                // may trigger getter
                PropOrSpread::Spread(_) => true,
            }),

            Expr::JSXMember(..)
            | Expr::JSXNamespacedName(..)
            | Expr::JSXEmpty(..)
            | Expr::JSXElement(..)
            | Expr::JSXFragment(..) => true,

            Expr::Invalid(..) => true,
        }
    }

    fn is_pure_callee(&self, ctx: ExprCtx<'_>) -> bool {
        if self.is_global_ref_to(ctx, "Date") {
            return true;
        }

        match self {
            Expr::Member(member) => {
                let obj = &member.obj;
                let prop = &member.prop;

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
                        Expr::Ident(ident) => ident.sym == "Math",
                        Expr::Lit(lit) if matches!(&**lit, Lit::Str(..)) => {
                            is_pure_str_method(prop.sym.as_str())
                        }
                        Expr::Tpl(tpl) if tpl.exprs.is_empty() => {
                            is_pure_str_method(prop.sym.as_str())
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }

            Expr::Fn(fn_expr) => {
                let f = &fn_expr.function;
                f.params.iter().all(|p| matches!(&p.pat, Pat::Ident(_)))
                    && f.body.as_ref().is_some_and(|body| body.stmts.is_empty())
            }

            _ => false,
        }
    }

    fn is_global_ref_to(&self, ctx: ExprCtx<'_>, id: &str) -> bool {
        match self {
            Expr::Ident(i) => {
                ctx.semantic.node_scope(i) == ctx.semantic.unresolved_scope_id() && i.sym == id
            }
            _ => false,
        }
    }
}

pub trait StmtExt {
    fn may_have_side_effects(&self, ctx: ExprCtx) -> bool;
}

impl<'a> StmtExt for Stmt<'a> {
    fn may_have_side_effects(&self, ctx: ExprCtx) -> bool {
        match self {
            Stmt::Block(block_stmt) => block_stmt
                .stmts
                .iter()
                .any(|stmt| stmt.may_have_side_effects(ctx)),
            Stmt::Empty(_) => false,
            Stmt::Labeled(labeled_stmt) => labeled_stmt.body.may_have_side_effects(ctx),
            Stmt::If(if_stmt) => {
                if_stmt.test.may_have_side_effects(ctx)
                    || if_stmt.cons.may_have_side_effects(ctx)
                    || if_stmt
                        .alt
                        .as_ref()
                        .is_some_and(|stmt| stmt.may_have_side_effects(ctx))
            }
            Stmt::Switch(switch_stmt) => {
                switch_stmt.discriminant.may_have_side_effects(ctx)
                    || switch_stmt.cases.iter().any(|case| {
                        case.test
                            .as_ref()
                            .is_some_and(|expr| expr.may_have_side_effects(ctx))
                            || case.cons.iter().any(|con| con.may_have_side_effects(ctx))
                    })
            }
            Stmt::Try(try_stmt) => {
                try_stmt
                    .block
                    .stmts
                    .iter()
                    .any(|stmt| stmt.may_have_side_effects(ctx))
                    || try_stmt.handler.as_ref().is_some_and(|handler| {
                        handler
                            .body
                            .stmts
                            .iter()
                            .any(|stmt| stmt.may_have_side_effects(ctx))
                    })
                    || try_stmt.finalizer.as_ref().is_some_and(|finalizer| {
                        finalizer
                            .stmts
                            .iter()
                            .any(|stmt| stmt.may_have_side_effects(ctx))
                    })
            }
            Stmt::Decl(decl) => match &**decl {
                Decl::Class(class_decl) => class_has_side_effect(ctx, &class_decl.class),
                Decl::Fn(_) => !ctx.in_strict,
                Decl::Var(var_decl) => var_decl.kind == VarDeclKind::Var,
                _ => false,
            },
            Stmt::Expr(expr_stmt) => expr_stmt.expr.may_have_side_effects(ctx),
            _ => true,
        }
    }
}

pub fn class_has_side_effect(expr_ctx: ExprCtx, c: &Class<'_>) -> bool {
    if let Some(e) = &c.super_class
        && e.may_have_side_effects(expr_ctx)
    {
        return true;
    }

    for m in &c.body {
        match m {
            ClassMember::Method(p) => {
                if let PropName::Computed(key) = &p.key
                    && key.expr.may_have_side_effects(expr_ctx)
                {
                    return true;
                }
            }

            ClassMember::ClassProp(p) => {
                if let PropName::Computed(key) = &p.key
                    && key.expr.may_have_side_effects(expr_ctx)
                {
                    return true;
                }

                if let Some(v) = &p.value
                    && v.may_have_side_effects(expr_ctx)
                {
                    return true;
                }
            }
            ClassMember::PrivateProp(p) => {
                if let Some(v) = &p.value
                    && v.may_have_side_effects(expr_ctx)
                {
                    return true;
                }
            }
            ClassMember::StaticBlock(s)
                if s.body
                    .stmts
                    .iter()
                    .any(|stmt| stmt.may_have_side_effects(expr_ctx)) =>
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
fn is_pure_new_callee(expr: &Expr<'_>, ctx: ExprCtx<'_>) -> bool {
    match expr {
        // An empty function expression is also pure for `new`
        Expr::Fn(func) => {
            let func = &func.function;
            func.params.iter().all(|p| matches!(&p.pat, Pat::Ident(_)))
                && func.body.as_ref().is_some_and(|body| body.stmts.is_empty())
        }

        // A class expression is pure for `new` if:
        // 1. It has no side effects from definition (computed keys, property initializers, static
        //    blocks)
        // 2. It has no super class (calling super() may have side effects)
        // 3. Either has no constructor, or constructor body is empty
        // 4. Has no instance properties (they are initialized in the constructor)
        Expr::Class(c) => {
            let class = &c.class;

            // Check for super class - calling super() may have side effects
            if class.super_class.is_some() {
                return false;
            }

            // Check for side effects from class definition
            if class_has_side_effect(ctx, class) {
                return false;
            }

            // Check for instance properties (non-static) - they run during construction
            for member in &class.body {
                match member {
                    ClassMember::ClassProp(p) if !p.is_static => return false,
                    ClassMember::PrivateProp(p) if !p.is_static => return false,
                    _ => {}
                }
            }

            // Check constructor - must be empty or not present
            for member in &class.body {
                if let ClassMember::Constructor(ctor) = member
                    && let Some(body) = &ctor.body
                    && !body.stmts.is_empty()
                {
                    return false;
                }
            }

            true
        }

        _ => false,
    }
}
