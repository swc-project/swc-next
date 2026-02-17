use swc_experimental_ecma_ast::*;
use swc_experimental_ecma_semantic::resolver::Semantic;

#[derive(Clone, Copy)]
pub struct ExprCtx<'a> {
    pub ast: &'a Ast,
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
    #[allow(clippy::wrong_self_convention)]
    fn is_pure_callee(self, ctx: ExprCtx<'_>) -> bool;
    #[allow(clippy::wrong_self_convention)]
    fn may_have_side_effects(self, ctx: ExprCtx<'_>) -> bool;
    #[allow(clippy::wrong_self_convention)]
    fn is_global_ref_to(self, ctx: ExprCtx<'_>, id: &str) -> bool;
}

impl ExprExt for Expr {
    fn may_have_side_effects(self, ctx: ExprCtx<'_>) -> bool {
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
                        ctx.ast.get_utf8(i.sym(ctx.ast)),
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

            Expr::Paren(e) => e.expr(ctx.ast).may_have_side_effects(ctx),

            // Function expression does not have any side effect if it's not used.
            Expr::Fn(..) | Expr::Arrow(..) => false,

            // It's annoying to pass in_strict
            Expr::Class(c) => class_has_side_effect(ctx, c.class(ctx.ast)),
            Expr::Array(arr) => arr
                .elems(ctx.ast)
                .iter()
                .filter_map(|id| ctx.ast.get_node_in_sub_range(id))
                .any(|e| e.spread(ctx.ast).is_some() || e.expr(ctx.ast).may_have_side_effects(ctx)),
            Expr::Unary(u) => match u.op(ctx.ast) {
                UnaryOp::Delete => true,
                _ => u.arg(ctx.ast).may_have_side_effects(ctx),
            },
            Expr::Bin(bin) => {
                bin.left(ctx.ast).may_have_side_effects(ctx)
                    || bin.right(ctx.ast).may_have_side_effects(ctx)
            }

            Expr::Member(member)
                if matches!(
                    member.obj(ctx.ast),
                    Expr::Object(_) | Expr::Fn(_) | Expr::Arrow(_) | Expr::Class(_)
                ) =>
            {
                let obj = member.obj(ctx.ast);
                if obj.may_have_side_effects(ctx) {
                    return true;
                }
                match obj {
                    Expr::Class(c) => {
                        let class = c.class(ctx.ast);
                        let is_static_accessor = |member: &ClassMember| {
                            if let ClassMember::Method(m) = member {
                                let kind = m.kind(ctx.ast);
                                if (kind == MethodKind::Getter || kind == MethodKind::Setter)
                                    && m.is_static(ctx.ast)
                                {
                                    return true;
                                }
                            }
                            false
                        };
                        if class
                            .body(ctx.ast)
                            .iter()
                            .any(|id| is_static_accessor(&ctx.ast.get_node_in_sub_range(id)))
                        {
                            return true;
                        }
                    }
                    Expr::Object(obj_lit) => {
                        let can_have_side_effect = |prop: PropOrSpread| match prop {
                            PropOrSpread::SpreadElement(_) => true,
                            PropOrSpread::Prop(prop) => match prop {
                                Prop::Getter(_) | Prop::Setter(_) | Prop::Method(_) => true,
                                Prop::Shorthand(ident) => {
                                    ctx.ast.get_utf8(ident.sym(ctx.ast)) == "__proto__"
                                }
                                Prop::KeyValue(kv) => match kv.key(ctx.ast) {
                                    PropName::Ident(ident_name) => {
                                        ctx.ast.get_utf8(ident_name.sym(ctx.ast)) == "__proto__"
                                    }
                                    PropName::Str(str_lit) => {
                                        ctx.ast.get_wtf8(str_lit.value(ctx.ast)) == "__proto__"
                                    }
                                    PropName::Computed(_) => true,
                                    _ => false,
                                },
                                _ => false,
                            },
                        };
                        if obj_lit
                            .props(ctx.ast)
                            .iter()
                            .map(|id| ctx.ast.get_node_in_sub_range(id))
                            .any(can_have_side_effect)
                        {
                            return true;
                        }
                    }
                    _ => {}
                };

                match member.prop(ctx.ast) {
                    MemberProp::Computed(c) => c.expr(ctx.ast).may_have_side_effects(ctx),
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

            Expr::OptChain(opt) if matches!(opt.base(ctx.ast), OptChainBase::Member(_)) => true,

            // A new expression is side-effect free if callee is pure for `new` and args are
            // side-effect free. Note: we use is_pure_new_callee instead of is_pure_callee because
            // class expressions are valid for `new` but calling them throws TypeError.
            Expr::New(new_expr) if is_pure_new_callee(new_expr.callee(ctx.ast), ctx) => {
                if let Some(args) = new_expr.args(ctx.ast) {
                    args.iter()
                        .map(|id| ctx.ast.get_node_in_sub_range(id))
                        .any(|arg| arg.expr(ctx.ast).may_have_side_effects(ctx))
                } else {
                    false
                }
            }

            Expr::New(_) => true,

            Expr::Call(call_expr) => {
                let callee = match call_expr.callee(ctx.ast) {
                    Callee::Expr(e) => e,
                    _ => return true,
                };

                if callee.is_pure_callee(ctx) {
                    call_expr
                        .args(ctx.ast)
                        .iter()
                        .map(|id| ctx.ast.get_node_in_sub_range(id))
                        .any(|arg| arg.expr(ctx.ast).may_have_side_effects(ctx))
                } else {
                    true
                }
            }

            Expr::OptChain(opt) => match opt.base(ctx.ast) {
                OptChainBase::Call(call) => {
                    if call.callee(ctx.ast).is_pure_callee(ctx) {
                        call.args(ctx.ast)
                            .iter()
                            .map(|id| ctx.ast.get_node_in_sub_range(id))
                            .any(|arg| arg.expr(ctx.ast).may_have_side_effects(ctx))
                    } else {
                        true
                    }
                }
                _ => true,
            },

            Expr::Seq(seq) => seq
                .exprs(ctx.ast)
                .iter()
                .map(|id| ctx.ast.get_node_in_sub_range(id))
                .any(|e| e.may_have_side_effects(ctx)),

            Expr::Cond(cond) => {
                cond.test(ctx.ast).may_have_side_effects(ctx)
                    || cond.cons(ctx.ast).may_have_side_effects(ctx)
                    || cond.alt(ctx.ast).may_have_side_effects(ctx)
            }

            Expr::Object(obj) => obj
                .props(ctx.ast)
                .iter()
                .map(|id| ctx.ast.get_node_in_sub_range(id))
                .any(|node| match node {
                    PropOrSpread::Prop(node) => match node {
                        Prop::Shorthand(..) => false,
                        Prop::KeyValue(kv) => {
                            let k = match kv.key(ctx.ast) {
                                PropName::Computed(e) => e.expr(ctx.ast).may_have_side_effects(ctx),
                                _ => false,
                            };

                            k || kv.value(ctx.ast).may_have_side_effects(ctx)
                        }
                        Prop::Getter(p) => match p.key(ctx.ast) {
                            PropName::Computed(e) => e.expr(ctx.ast).may_have_side_effects(ctx),
                            _ => false,
                        },
                        Prop::Setter(p) => match p.key(ctx.ast) {
                            PropName::Computed(e) => e.expr(ctx.ast).may_have_side_effects(ctx),
                            _ => false,
                        },
                        Prop::Method(p) => match p.key(ctx.ast) {
                            PropName::Computed(e) => e.expr(ctx.ast).may_have_side_effects(ctx),
                            _ => false,
                        },
                        Prop::Assign(_) => true,
                    },
                    // may trigger getter
                    PropOrSpread::SpreadElement(_) => true,
                }),

            Expr::JSXMember(..)
            | Expr::JSXNamespacedName(..)
            | Expr::JSXEmpty(..)
            | Expr::JSXElement(..)
            | Expr::JSXFragment(..) => true,

            Expr::Invalid(..) => true,
        }
    }

    fn is_pure_callee(self, ctx: ExprCtx<'_>) -> bool {
        if self.is_global_ref_to(ctx, "Date") {
            return true;
        }

        match self {
            Expr::Member(member) => {
                let obj = member.obj(ctx.ast);
                let prop = member.prop(ctx.ast);

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
                        Expr::Ident(ident) => {
                            let sym = ident.sym(ctx.ast);
                            if ctx.ast.get_utf8(sym) == "Math" {
                                return true;
                            }
                            false
                        }
                        Expr::Lit(Lit::Str(..)) => {
                            is_pure_str_method(ctx.ast.get_utf8(prop.sym(ctx.ast)))
                        }
                        Expr::Tpl(tpl) if tpl.exprs(ctx.ast).is_empty() => {
                            is_pure_str_method(ctx.ast.get_utf8(prop.sym(ctx.ast)))
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }

            Expr::Fn(fn_expr) => {
                let f = fn_expr.function(ctx.ast);
                f.params(ctx.ast)
                    .iter()
                    .map(|id| ctx.ast.get_node_in_sub_range(id))
                    .all(|p| matches!(p.pat(ctx.ast), Pat::Ident(_)))
                    && f.body(ctx.ast).is_some()
                    && f.body(ctx.ast).unwrap().stmts(ctx.ast).is_empty()
            }

            _ => false,
        }
    }

    fn is_global_ref_to(self, ctx: ExprCtx<'_>, id: &str) -> bool {
        match self {
            Expr::Ident(i) => {
                ctx.semantic.node_scope(i) == ctx.semantic.unresolved_scope_id()
                    && ctx.ast.get_utf8(i.sym(ctx.ast)) == id
            }
            _ => false,
        }
    }
}

pub trait StmtExt {
    fn may_have_side_effects(self, ctx: ExprCtx) -> bool;
}

impl StmtExt for Stmt {
    fn may_have_side_effects(self, ctx: ExprCtx) -> bool {
        match self {
            Stmt::Block(block_stmt) => block_stmt
                .stmts(ctx.ast)
                .iter()
                .map(|i| ctx.ast.get_node_in_sub_range(i))
                .any(|stmt| stmt.may_have_side_effects(ctx)),
            Stmt::Empty(_) => false,
            Stmt::Labeled(labeled_stmt) => labeled_stmt.body(ctx.ast).may_have_side_effects(ctx),
            Stmt::If(if_stmt) => {
                if_stmt.test(ctx.ast).may_have_side_effects(ctx)
                    || if_stmt.cons(ctx.ast).may_have_side_effects(ctx)
                    || if_stmt
                        .alt(ctx.ast)
                        .is_some_and(|stmt| stmt.may_have_side_effects(ctx))
            }
            Stmt::Switch(switch_stmt) => {
                switch_stmt.discriminant(ctx.ast).may_have_side_effects(ctx)
                    || switch_stmt
                        .cases(ctx.ast)
                        .iter()
                        .map(|i| ctx.ast.get_node_in_sub_range(i))
                        .any(|case| {
                            case.test(ctx.ast)
                                .is_some_and(|expr| expr.may_have_side_effects(ctx))
                                || case
                                    .cons(ctx.ast)
                                    .iter()
                                    .map(|i| ctx.ast.get_node_in_sub_range(i))
                                    .any(|con| con.may_have_side_effects(ctx))
                        })
            }
            Stmt::Try(try_stmt) => {
                try_stmt
                    .block(ctx.ast)
                    .stmts(ctx.ast)
                    .iter()
                    .map(|i| ctx.ast.get_node_in_sub_range(i))
                    .any(|stmt| stmt.may_have_side_effects(ctx))
                    || try_stmt.handler(ctx.ast).is_some_and(|handler| {
                        handler
                            .body(ctx.ast)
                            .stmts(ctx.ast)
                            .iter()
                            .map(|i| ctx.ast.get_node_in_sub_range(i))
                            .any(|stmt| stmt.may_have_side_effects(ctx))
                    })
                    || try_stmt.finalizer(ctx.ast).is_some_and(|finalizer| {
                        finalizer
                            .stmts(ctx.ast)
                            .iter()
                            .map(|i| ctx.ast.get_node_in_sub_range(i))
                            .any(|stmt| stmt.may_have_side_effects(ctx))
                    })
            }
            Stmt::Decl(decl) => match decl {
                Decl::Class(class_decl) => class_has_side_effect(ctx, class_decl.class(ctx.ast)),
                Decl::Fn(_) => !ctx.in_strict,
                Decl::Var(var_decl) => var_decl.kind(ctx.ast) == VarDeclKind::Var,
                _ => false,
            },
            Stmt::Expr(expr_stmt) => expr_stmt.expr(ctx.ast).may_have_side_effects(ctx),
            _ => true,
        }
    }
}

pub fn class_has_side_effect(expr_ctx: ExprCtx, c: Class) -> bool {
    if let Some(e) = c.super_class(expr_ctx.ast)
        && e.may_have_side_effects(expr_ctx)
    {
        return true;
    }

    for m in c
        .body(expr_ctx.ast)
        .iter()
        .map(|id| expr_ctx.ast.get_node_in_sub_range(id))
    {
        match m {
            ClassMember::Method(p) => {
                if let PropName::Computed(key) = p.key(expr_ctx.ast)
                    && key.expr(expr_ctx.ast).may_have_side_effects(expr_ctx)
                {
                    return true;
                }
            }

            ClassMember::ClassProp(p) => {
                if let PropName::Computed(key) = p.key(expr_ctx.ast)
                    && key.expr(expr_ctx.ast).may_have_side_effects(expr_ctx)
                {
                    return true;
                }

                if let Some(v) = p.value(expr_ctx.ast)
                    && v.may_have_side_effects(expr_ctx)
                {
                    return true;
                }
            }
            ClassMember::PrivateProp(p) => {
                if let Some(v) = p.value(expr_ctx.ast)
                    && v.may_have_side_effects(expr_ctx)
                {
                    return true;
                }
            }
            ClassMember::StaticBlock(s) => {
                if s.body(expr_ctx.ast)
                    .stmts(expr_ctx.ast)
                    .iter()
                    .map(|id| expr_ctx.ast.get_node_in_sub_range(id))
                    .any(|stmt| stmt.may_have_side_effects(expr_ctx))
                {
                    return true;
                }
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
fn is_pure_new_callee(expr: Expr, ctx: ExprCtx<'_>) -> bool {
    match expr {
        // An empty function expression is also pure for `new`
        Expr::Fn(func) => {
            let func = func.function(ctx.ast);
            func.params(ctx.ast)
                .iter()
                .all(|p| ctx.ast.get_node_in_sub_range(p).pat(ctx.ast).is_ident())
                && func.body(ctx.ast).is_some()
                && func.body(ctx.ast).unwrap().stmts(ctx.ast).is_empty()
        }

        // A class expression is pure for `new` if:
        // 1. It has no side effects from definition (computed keys, property initializers, static
        //    blocks)
        // 2. It has no super class (calling super() may have side effects)
        // 3. Either has no constructor, or constructor body is empty
        // 4. Has no instance properties (they are initialized in the constructor)
        Expr::Class(c) => {
            let class = c.class(ctx.ast);

            // Check for super class - calling super() may have side effects
            if class.super_class(ctx.ast).is_some() {
                return false;
            }

            // Check for side effects from class definition
            if class_has_side_effect(ctx, class) {
                return false;
            }

            // Check for instance properties (non-static) - they run during construction
            for member in class.body(ctx.ast).iter() {
                let member = ctx.ast.get_node_in_sub_range(member);
                match member {
                    ClassMember::ClassProp(p) if !p.is_static(ctx.ast) => return false,
                    ClassMember::PrivateProp(p) if !p.is_static(ctx.ast) => return false,
                    _ => {}
                }
            }

            // Check constructor - must be empty or not present
            for member in class.body(ctx.ast).iter() {
                let member = ctx.ast.get_node_in_sub_range(member);
                if let ClassMember::Constructor(ctor) = member
                    && let Some(body) = ctor.body(ctx.ast)
                    && !body.stmts(ctx.ast).is_empty()
                {
                    return false;
                }
            }

            true
        }

        _ => false,
    }
}
