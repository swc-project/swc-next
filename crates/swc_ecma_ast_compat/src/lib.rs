use swc_core::atoms::{Atom, Wtf8Atom};
use swc_core::ecma::ast::{self as legacy};
use swc_experimental_ecma_ast::{
    self as experimental, Ast, BigIntId, ExtraDataCompact, OptionalUtf8Ref, OptionalWtf8Ref,
    Spanned, TypedSubRange, Utf8Ref, Wtf8Ref,
};
use swc_experimental_ecma_semantic::resolver::Semantic;

pub struct AstCompat<'a> {
    ast: &'a Ast,
    semantic: &'a Semantic,
}

impl<'a> AstCompat<'a> {
    pub fn new(ast: &'a Ast, semantic: &'a Semantic) -> Self {
        Self { ast, semantic }
    }
}

impl AstCompat<'_> {
    pub fn compat_program(&mut self, root: experimental::Program) -> legacy::Program {
        match root {
            experimental::Program::Module(module) => {
                legacy::Program::Module(self.compat_module(module))
            }
            experimental::Program::Script(script) => {
                legacy::Program::Script(self.compat_script(script))
            }
        }
    }

    pub fn compat_module(&mut self, module: experimental::Module) -> legacy::Module {
        #[allow(unused_mut)]
        let mut inner = || legacy::Module {
            span: module.span(self.ast),
            shebang: self.compat_opt_utf8_ref(module.shebang(self.ast)),
            body: self.compat_type_sub_range(module.body(self.ast), Self::compat_module_item),
        };

        #[cfg(all(debug_assertions, not(target_family = "wasm")))]
        {
            // Adjust stack to avoid stack overflow.
            stacker::maybe_grow(
                2 * 1024 * 1024, /* 2mb */
                4 * 1024 * 1024, /* 4mb */
                inner,
            )
        }

        #[cfg(any(not(debug_assertions), target_family = "wasm"))]
        inner()
    }

    pub fn compat_script(&mut self, script: experimental::Script) -> legacy::Script {
        #[allow(unused_mut)]
        let mut inner = || legacy::Script {
            span: script.span(self.ast),
            body: self.compat_type_sub_range(script.body(self.ast), Self::compat_stmt),
            shebang: self.compat_opt_utf8_ref(script.shebang(self.ast)),
        };

        #[cfg(all(debug_assertions, not(target_family = "wasm")))]
        {
            // Adjust stack to avoid stack overflow.
            stacker::maybe_grow(
                2 * 1024 * 1024, /* 2mb */
                4 * 1024 * 1024, /* 4mb */
                inner,
            )
        }

        #[cfg(any(not(debug_assertions), target_family = "wasm"))]
        inner()
    }

    fn compat_module_item(&mut self, item: experimental::ModuleItem) -> legacy::ModuleItem {
        match item {
            experimental::ModuleItem::ModuleDecl(module_decl) => {
                legacy::ModuleItem::ModuleDecl(self.compat_module_decl(module_decl))
            }
            experimental::ModuleItem::Stmt(stmt) => {
                legacy::ModuleItem::Stmt(self.compat_stmt(stmt))
            }
        }
    }

    fn compat_module_decl(&mut self, module_decl: experimental::ModuleDecl) -> legacy::ModuleDecl {
        match module_decl {
            experimental::ModuleDecl::Import(import_decl) => {
                legacy::ModuleDecl::Import(self.compat_import_decl(import_decl))
            }
            experimental::ModuleDecl::ExportDecl(export_decl) => {
                legacy::ModuleDecl::ExportDecl(self.compat_export_decl(export_decl))
            }
            experimental::ModuleDecl::ExportNamed(named_export) => {
                legacy::ModuleDecl::ExportNamed(self.compat_export_named(named_export))
            }
            experimental::ModuleDecl::ExportDefaultDecl(export_default_decl) => {
                legacy::ModuleDecl::ExportDefaultDecl(
                    self.compat_export_default_decl(export_default_decl),
                )
            }
            experimental::ModuleDecl::ExportDefaultExpr(export_default_expr) => {
                legacy::ModuleDecl::ExportDefaultExpr(
                    self.compat_export_default_expr(export_default_expr),
                )
            }
            experimental::ModuleDecl::ExportAll(export_all) => {
                legacy::ModuleDecl::ExportAll(self.compat_export_all(export_all))
            }
        }
    }

    fn compat_import_decl(&mut self, import_decl: experimental::ImportDecl) -> legacy::ImportDecl {
        legacy::ImportDecl {
            span: import_decl.span(self.ast),
            specifiers: self.compat_type_sub_range(
                import_decl.specifiers(self.ast),
                Self::compat_import_specifier,
            ),
            src: Box::new(self.compat_str(import_decl.src(self.ast))),
            type_only: import_decl.type_only(self.ast),
            with: import_decl
                .with(self.ast)
                .map(|with| Box::new(self.compat_object_lit(with))),
            phase: match import_decl.phase(self.ast) {
                experimental::ImportPhase::Evaluation => legacy::ImportPhase::Evaluation,
                experimental::ImportPhase::Source => legacy::ImportPhase::Source,
                experimental::ImportPhase::Defer => legacy::ImportPhase::Defer,
            },
        }
    }

    fn compat_export_decl(&mut self, export_decl: experimental::ExportDecl) -> legacy::ExportDecl {
        legacy::ExportDecl {
            span: export_decl.span(self.ast),
            decl: self.compat_decl(export_decl.decl(self.ast)),
        }
    }

    fn compat_export_named(
        &mut self,

        export_named: experimental::NamedExport,
    ) -> legacy::NamedExport {
        legacy::NamedExport {
            span: export_named.span(self.ast),
            specifiers: self.compat_type_sub_range(
                export_named.specifiers(self.ast),
                Self::compat_export_specifier,
            ),
            src: export_named
                .src(self.ast)
                .map(|s| Box::new(self.compat_str(s))),
            type_only: export_named.type_only(self.ast),
            with: export_named
                .with(self.ast)
                .map(|o| Box::new(self.compat_object_lit(o))),
        }
    }

    fn compat_export_default_decl(
        &mut self,

        export_default_decl: experimental::ExportDefaultDecl,
    ) -> legacy::ExportDefaultDecl {
        legacy::ExportDefaultDecl {
            span: export_default_decl.span(self.ast),
            decl: match export_default_decl.decl(self.ast) {
                experimental::DefaultDecl::Class(cls) => {
                    legacy::DefaultDecl::Class(self.compat_class_expr(cls))
                }
                experimental::DefaultDecl::Fn(f) => legacy::DefaultDecl::Fn(self.compat_fn_expr(f)),
            },
        }
    }

    fn compat_export_default_expr(
        &mut self,

        export_default_expr: experimental::ExportDefaultExpr,
    ) -> legacy::ExportDefaultExpr {
        legacy::ExportDefaultExpr {
            span: export_default_expr.span(self.ast),
            expr: self.compat_expr(export_default_expr.expr(self.ast)),
        }
    }

    fn compat_export_all(&mut self, export_all: experimental::ExportAll) -> legacy::ExportAll {
        legacy::ExportAll {
            span: export_all.span(self.ast),
            src: Box::new(self.compat_str(export_all.src(self.ast))),
            type_only: export_all.type_only(self.ast),
            with: export_all
                .with(self.ast)
                .map(|o| Box::new(self.compat_object_lit(o))),
        }
    }

    fn compat_stmt(&mut self, stmt: experimental::Stmt) -> legacy::Stmt {
        match stmt {
            experimental::Stmt::Block(block_stmt) => {
                legacy::Stmt::Block(self.compat_block_stmt(block_stmt))
            }
            experimental::Stmt::Empty(empty_stmt) => {
                legacy::Stmt::Empty(self.compat_empty_stmt(empty_stmt))
            }
            experimental::Stmt::Debugger(debugger_stmt) => {
                legacy::Stmt::Debugger(legacy::DebuggerStmt {
                    span: debugger_stmt.span(self.ast),
                })
            }
            experimental::Stmt::With(with_stmt) => legacy::Stmt::With(legacy::WithStmt {
                span: with_stmt.span(self.ast),
                obj: self.compat_expr(with_stmt.obj(self.ast)),
                body: Box::new(self.compat_stmt(with_stmt.body(self.ast))),
            }),
            experimental::Stmt::Return(return_stmt) => legacy::Stmt::Return(legacy::ReturnStmt {
                span: return_stmt.span(self.ast),
                arg: return_stmt.arg(self.ast).map(|arg| self.compat_expr(arg)),
            }),
            experimental::Stmt::Labeled(labeled_stmt) => {
                legacy::Stmt::Labeled(legacy::LabeledStmt {
                    span: labeled_stmt.span(self.ast),
                    label: self.compat_ident(labeled_stmt.label(self.ast)),
                    body: Box::new(self.compat_stmt(labeled_stmt.body(self.ast))),
                })
            }
            experimental::Stmt::Break(break_stmt) => legacy::Stmt::Break(legacy::BreakStmt {
                span: break_stmt.span(self.ast),
                label: break_stmt
                    .label(self.ast)
                    .map(|label| self.compat_ident(label)),
            }),
            experimental::Stmt::Continue(continue_stmt) => {
                legacy::Stmt::Continue(legacy::ContinueStmt {
                    span: continue_stmt.span(self.ast),
                    label: continue_stmt
                        .label(self.ast)
                        .map(|label| self.compat_ident(label)),
                })
            }
            experimental::Stmt::If(if_stmt) => legacy::Stmt::If(legacy::IfStmt {
                span: if_stmt.span(self.ast),
                test: self.compat_expr(if_stmt.test(self.ast)),
                cons: Box::new(self.compat_stmt(if_stmt.cons(self.ast))),
                alt: if_stmt
                    .alt(self.ast)
                    .map(|alt| Box::new(self.compat_stmt(alt))),
            }),
            experimental::Stmt::Switch(switch_stmt) => legacy::Stmt::Switch(legacy::SwitchStmt {
                span: switch_stmt.span(self.ast),
                discriminant: self.compat_expr(switch_stmt.discriminant(self.ast)),
                cases: self
                    .compat_type_sub_range(switch_stmt.cases(self.ast), Self::compat_switch_case),
            }),
            experimental::Stmt::Throw(throw_stmt) => legacy::Stmt::Throw(legacy::ThrowStmt {
                span: throw_stmt.span(self.ast),
                arg: self.compat_expr(throw_stmt.arg(self.ast)),
            }),
            experimental::Stmt::Try(try_stmt) => legacy::Stmt::Try(Box::new(legacy::TryStmt {
                span: try_stmt.span(self.ast),
                block: self.compat_block_stmt(try_stmt.block(self.ast)),
                handler: try_stmt
                    .handler(self.ast)
                    .map(|handler| self.compat_catch_clause(handler)),
                finalizer: try_stmt
                    .finalizer(self.ast)
                    .map(|finalizer| self.compat_block_stmt(finalizer)),
            })),
            experimental::Stmt::While(while_stmt) => legacy::Stmt::While(legacy::WhileStmt {
                span: while_stmt.span(self.ast),
                test: self.compat_expr(while_stmt.test(self.ast)),
                body: Box::new(self.compat_stmt(while_stmt.body(self.ast))),
            }),
            experimental::Stmt::DoWhile(do_while_stmt) => {
                legacy::Stmt::DoWhile(legacy::DoWhileStmt {
                    span: do_while_stmt.span(self.ast),
                    test: self.compat_expr(do_while_stmt.test(self.ast)),
                    body: Box::new(self.compat_stmt(do_while_stmt.body(self.ast))),
                })
            }
            experimental::Stmt::For(for_stmt) => legacy::Stmt::For(legacy::ForStmt {
                span: for_stmt.span(self.ast),
                init: for_stmt
                    .init(self.ast)
                    .map(|i| self.compat_var_decl_or_expr(i)),
                test: for_stmt.test(self.ast).map(|e| self.compat_expr(e)),
                update: for_stmt.update(self.ast).map(|e| self.compat_expr(e)),
                body: Box::new(self.compat_stmt(for_stmt.body(self.ast))),
            }),
            experimental::Stmt::ForIn(for_in_stmt) => legacy::Stmt::ForIn(legacy::ForInStmt {
                span: for_in_stmt.span(self.ast),
                left: self.compat_for_head(for_in_stmt.left(self.ast)),
                right: self.compat_expr(for_in_stmt.right(self.ast)),
                body: Box::new(self.compat_stmt(for_in_stmt.body(self.ast))),
            }),
            experimental::Stmt::ForOf(for_of_stmt) => legacy::Stmt::ForOf(legacy::ForOfStmt {
                span: for_of_stmt.span(self.ast),
                is_await: for_of_stmt.is_await(self.ast),
                left: self.compat_for_head(for_of_stmt.left(self.ast)),
                right: self.compat_expr(for_of_stmt.right(self.ast)),
                body: Box::new(self.compat_stmt(for_of_stmt.body(self.ast))),
            }),
            experimental::Stmt::Decl(decl) => legacy::Stmt::Decl(self.compat_decl(decl)),
            experimental::Stmt::Expr(expr_stmt) => legacy::Stmt::Expr(legacy::ExprStmt {
                span: expr_stmt.span(self.ast),
                expr: self.compat_expr(expr_stmt.expr(self.ast)),
            }),
        }
    }

    fn compat_block_stmt(&mut self, block_stmt: experimental::BlockStmt) -> legacy::BlockStmt {
        legacy::BlockStmt {
            span: block_stmt.span(self.ast),
            stmts: self.compat_type_sub_range(block_stmt.stmts(self.ast), Self::compat_stmt),
            ctxt: self.semantic.body_scope(block_stmt).to_ctxt(),
        }
    }

    fn compat_empty_stmt(&mut self, empty_stmt: experimental::EmptyStmt) -> legacy::EmptyStmt {
        legacy::EmptyStmt {
            span: empty_stmt.span(self.ast),
        }
    }

    fn compat_expr(&mut self, expr: experimental::Expr) -> Box<legacy::Expr> {
        Box::new(match expr {
            experimental::Expr::This(t) => legacy::Expr::This(legacy::ThisExpr {
                span: t.span(self.ast),
            }),
            experimental::Expr::Array(a) => legacy::Expr::Array(legacy::ArrayLit {
                span: a.span(self.ast),
                elems: self.compat_type_sub_range(a.elems(self.ast), |this, e| {
                    e.map(|e| this.compat_expr_or_spread(e))
                }),
            }),
            experimental::Expr::Object(o) => legacy::Expr::Object(self.compat_object_lit(o)),
            experimental::Expr::Fn(f) => legacy::Expr::Fn(self.compat_fn_expr(f)),
            experimental::Expr::Unary(u) => legacy::Expr::Unary(legacy::UnaryExpr {
                span: u.span(self.ast),
                op: match u.op(self.ast) {
                    experimental::UnaryOp::Minus => legacy::UnaryOp::Minus,
                    experimental::UnaryOp::Plus => legacy::UnaryOp::Plus,
                    experimental::UnaryOp::Bang => legacy::UnaryOp::Bang,
                    experimental::UnaryOp::Tilde => legacy::UnaryOp::Tilde,
                    experimental::UnaryOp::TypeOf => legacy::UnaryOp::TypeOf,
                    experimental::UnaryOp::Void => legacy::UnaryOp::Void,
                    experimental::UnaryOp::Delete => legacy::UnaryOp::Delete,
                },
                arg: self.compat_expr(u.arg(self.ast)),
            }),
            experimental::Expr::Update(u) => legacy::Expr::Update(legacy::UpdateExpr {
                span: u.span(self.ast),
                op: match u.op(self.ast) {
                    experimental::UpdateOp::PlusPlus => legacy::UpdateOp::PlusPlus,
                    experimental::UpdateOp::MinusMinus => legacy::UpdateOp::MinusMinus,
                },
                prefix: u.prefix(self.ast),
                arg: self.compat_expr(u.arg(self.ast)),
            }),
            experimental::Expr::Bin(b) => legacy::Expr::Bin(legacy::BinExpr {
                span: b.span(self.ast),
                op: match b.op(self.ast) {
                    experimental::BinaryOp::EqEq => legacy::BinaryOp::EqEq,
                    experimental::BinaryOp::NotEq => legacy::BinaryOp::NotEq,
                    experimental::BinaryOp::EqEqEq => legacy::BinaryOp::EqEqEq,
                    experimental::BinaryOp::NotEqEq => legacy::BinaryOp::NotEqEq,
                    experimental::BinaryOp::Lt => legacy::BinaryOp::Lt,
                    experimental::BinaryOp::LtEq => legacy::BinaryOp::LtEq,
                    experimental::BinaryOp::Gt => legacy::BinaryOp::Gt,
                    experimental::BinaryOp::GtEq => legacy::BinaryOp::GtEq,
                    experimental::BinaryOp::LShift => legacy::BinaryOp::LShift,
                    experimental::BinaryOp::RShift => legacy::BinaryOp::RShift,
                    experimental::BinaryOp::ZeroFillRShift => legacy::BinaryOp::ZeroFillRShift,
                    experimental::BinaryOp::Add => legacy::BinaryOp::Add,
                    experimental::BinaryOp::Sub => legacy::BinaryOp::Sub,
                    experimental::BinaryOp::Mul => legacy::BinaryOp::Mul,
                    experimental::BinaryOp::Div => legacy::BinaryOp::Div,
                    experimental::BinaryOp::Mod => legacy::BinaryOp::Mod,
                    experimental::BinaryOp::BitOr => legacy::BinaryOp::BitOr,
                    experimental::BinaryOp::BitXor => legacy::BinaryOp::BitXor,
                    experimental::BinaryOp::BitAnd => legacy::BinaryOp::BitAnd,
                    experimental::BinaryOp::LogicalOr => legacy::BinaryOp::LogicalOr,
                    experimental::BinaryOp::LogicalAnd => legacy::BinaryOp::LogicalAnd,
                    experimental::BinaryOp::In => legacy::BinaryOp::In,
                    experimental::BinaryOp::InstanceOf => legacy::BinaryOp::InstanceOf,
                    experimental::BinaryOp::Exp => legacy::BinaryOp::Exp,
                    experimental::BinaryOp::NullishCoalescing => {
                        legacy::BinaryOp::NullishCoalescing
                    }
                },
                left: self.compat_expr(b.left(self.ast)),
                right: self.compat_expr(b.right(self.ast)),
            }),
            experimental::Expr::Assign(a) => legacy::Expr::Assign(legacy::AssignExpr {
                span: a.span(self.ast),
                op: match a.op(self.ast) {
                    experimental::AssignOp::Assign => legacy::AssignOp::Assign,
                    experimental::AssignOp::AddAssign => legacy::AssignOp::AddAssign,
                    experimental::AssignOp::SubAssign => legacy::AssignOp::SubAssign,
                    experimental::AssignOp::MulAssign => legacy::AssignOp::MulAssign,
                    experimental::AssignOp::DivAssign => legacy::AssignOp::DivAssign,
                    experimental::AssignOp::ModAssign => legacy::AssignOp::ModAssign,
                    experimental::AssignOp::LShiftAssign => legacy::AssignOp::LShiftAssign,
                    experimental::AssignOp::RShiftAssign => legacy::AssignOp::RShiftAssign,
                    experimental::AssignOp::ZeroFillRShiftAssign => {
                        legacy::AssignOp::ZeroFillRShiftAssign
                    }
                    experimental::AssignOp::BitOrAssign => legacy::AssignOp::BitOrAssign,
                    experimental::AssignOp::BitXorAssign => legacy::AssignOp::BitXorAssign,
                    experimental::AssignOp::BitAndAssign => legacy::AssignOp::BitAndAssign,
                    experimental::AssignOp::ExpAssign => legacy::AssignOp::ExpAssign,
                    experimental::AssignOp::AndAssign => legacy::AssignOp::AndAssign,
                    experimental::AssignOp::OrAssign => legacy::AssignOp::OrAssign,
                    experimental::AssignOp::NullishAssign => legacy::AssignOp::NullishAssign,
                },
                left: self.compat_assign_target(a.left(self.ast)),
                right: self.compat_expr(a.right(self.ast)),
            }),
            experimental::Expr::Member(m) => legacy::Expr::Member(legacy::MemberExpr {
                span: m.span(self.ast),
                obj: self.compat_expr(m.obj(self.ast)),
                prop: self.compat_member_prop(m.prop(self.ast)),
            }),
            experimental::Expr::SuperProp(s) => legacy::Expr::SuperProp(legacy::SuperPropExpr {
                span: s.span(self.ast),
                obj: legacy::Super {
                    span: s.obj(self.ast).span(self.ast),
                },
                prop: match s.prop(self.ast) {
                    experimental::SuperProp::Ident(i) => {
                        legacy::SuperProp::Ident(legacy::IdentName {
                            span: i.span(self.ast),
                            sym: self.compat_utf8_ref(i.sym(self.ast)),
                        })
                    }
                    experimental::SuperProp::Computed(c) => {
                        legacy::SuperProp::Computed(legacy::ComputedPropName {
                            span: c.span(self.ast),
                            expr: self.compat_expr(c.expr(self.ast)),
                        })
                    }
                },
            }),
            experimental::Expr::Cond(c) => legacy::Expr::Cond(legacy::CondExpr {
                span: c.span(self.ast),
                test: self.compat_expr(c.test(self.ast)),
                cons: self.compat_expr(c.cons(self.ast)),
                alt: self.compat_expr(c.alt(self.ast)),
            }),
            experimental::Expr::Call(c) => legacy::Expr::Call(legacy::CallExpr {
                span: c.span(self.ast),
                ctxt: Default::default(),
                callee: match c.callee(self.ast) {
                    experimental::Callee::Super(s) => legacy::Callee::Super(legacy::Super {
                        span: s.span(self.ast),
                    }),
                    experimental::Callee::Import(i) => legacy::Callee::Import(legacy::Import {
                        span: i.span(self.ast),
                        phase: match i.phase(self.ast) {
                            experimental::ImportPhase::Evaluation => {
                                legacy::ImportPhase::Evaluation
                            }
                            experimental::ImportPhase::Source => legacy::ImportPhase::Source,
                            experimental::ImportPhase::Defer => legacy::ImportPhase::Defer,
                        },
                    }),
                    experimental::Callee::Expr(e) => legacy::Callee::Expr(self.compat_expr(e)),
                },
                args: self.compat_type_sub_range(c.args(self.ast), Self::compat_expr_or_spread),
                type_args: None,
            }),
            experimental::Expr::New(n) => legacy::Expr::New(legacy::NewExpr {
                span: n.span(self.ast),
                ctxt: Default::default(),
                callee: self.compat_expr(n.callee(self.ast)),
                args: n
                    .args(self.ast)
                    .map(|args| self.compat_type_sub_range(args, Self::compat_expr_or_spread)),
                type_args: None,
            }),
            experimental::Expr::Seq(s) => legacy::Expr::Seq(legacy::SeqExpr {
                span: s.span(self.ast),
                exprs: self.compat_type_sub_range(s.exprs(self.ast), Self::compat_expr),
            }),
            experimental::Expr::Ident(i) => legacy::Expr::Ident(self.compat_ident(i)),
            experimental::Expr::Lit(l) => legacy::Expr::Lit(self.compat_lit(l)),
            experimental::Expr::Tpl(t) => legacy::Expr::Tpl(legacy::Tpl {
                span: t.span(self.ast),
                exprs: self.compat_type_sub_range(t.exprs(self.ast), Self::compat_expr),
                quasis: self.compat_type_sub_range(t.quasis(self.ast), Self::compat_tpl_element),
            }),
            experimental::Expr::TaggedTpl(tt) => legacy::Expr::TaggedTpl(legacy::TaggedTpl {
                span: tt.span(self.ast),
                ctxt: Default::default(),
                tag: self.compat_expr(tt.tag(self.ast)),
                tpl: Box::new(self.compat_tpl(tt.tpl(self.ast))),
                type_params: None,
            }),
            experimental::Expr::Arrow(a) => legacy::Expr::Arrow(legacy::ArrowExpr {
                span: a.span(self.ast),
                ctxt: Default::default(),
                params: self.compat_type_sub_range(a.params(self.ast), Self::compat_pat),
                body: Box::new(match a.body(self.ast) {
                    experimental::BlockStmtOrExpr::BlockStmt(b) => {
                        legacy::BlockStmtOrExpr::BlockStmt(self.compat_block_stmt(b))
                    }
                    experimental::BlockStmtOrExpr::Expr(e) => {
                        legacy::BlockStmtOrExpr::Expr(self.compat_expr(e))
                    }
                }),
                is_async: a.is_async(self.ast),
                is_generator: a.is_generator(self.ast),
                type_params: None,
                return_type: None,
            }),
            experimental::Expr::Class(c) => legacy::Expr::Class(self.compat_class_expr(c)),
            experimental::Expr::Yield(y) => legacy::Expr::Yield(legacy::YieldExpr {
                span: y.span(self.ast),
                arg: y.arg(self.ast).map(|e| self.compat_expr(e)),
                delegate: y.delegate(self.ast),
            }),
            experimental::Expr::MetaProp(m) => legacy::Expr::MetaProp(legacy::MetaPropExpr {
                span: m.span(self.ast),
                kind: match m.kind(self.ast) {
                    experimental::MetaPropKind::NewTarget => legacy::MetaPropKind::NewTarget,
                    experimental::MetaPropKind::ImportMeta => legacy::MetaPropKind::ImportMeta,
                },
            }),
            experimental::Expr::Await(a) => legacy::Expr::Await(legacy::AwaitExpr {
                span: a.span(self.ast),
                arg: self.compat_expr(a.arg(self.ast)),
            }),
            experimental::Expr::Paren(p) => legacy::Expr::Paren(legacy::ParenExpr {
                span: p.span(self.ast),
                expr: self.compat_expr(p.expr(self.ast)),
            }),
            experimental::Expr::PrivateName(p) => legacy::Expr::PrivateName(legacy::PrivateName {
                span: p.span(self.ast),
                name: self.compat_utf8_ref(p.name(self.ast)),
            }),
            experimental::Expr::OptChain(o) => legacy::Expr::OptChain(legacy::OptChainExpr {
                span: o.span(self.ast),
                optional: o.optional(self.ast),
                base: Box::new(match o.base(self.ast) {
                    experimental::OptChainBase::Member(m) => {
                        legacy::OptChainBase::Member(legacy::MemberExpr {
                            span: m.span(self.ast),
                            obj: self.compat_expr(m.obj(self.ast)),
                            prop: self.compat_member_prop(m.prop(self.ast)),
                        })
                    }
                    experimental::OptChainBase::Call(c) => {
                        legacy::OptChainBase::Call(legacy::OptCall {
                            span: c.span(self.ast),
                            ctxt: Default::default(),
                            callee: self.compat_expr(c.callee(self.ast)),
                            args: self.compat_type_sub_range(
                                c.args(self.ast),
                                Self::compat_expr_or_spread,
                            ),
                            type_args: None,
                        })
                    }
                }),
            }),
            experimental::Expr::JSXMember(j) => {
                legacy::Expr::JSXMember(self.compat_jsx_member_expr(j))
            }
            experimental::Expr::JSXNamespacedName(j) => {
                legacy::Expr::JSXNamespacedName(self.compat_jsx_namespaced_name(j))
            }
            experimental::Expr::JSXEmpty(j) => {
                legacy::Expr::JSXEmpty(self.compat_jsx_empty_expr(j))
            }
            experimental::Expr::JSXElement(j) => {
                legacy::Expr::JSXElement(Box::new(self.compat_jsx_element(j)))
            }
            experimental::Expr::JSXFragment(j) => {
                legacy::Expr::JSXFragment(self.compat_jsx_fragment(j))
            }
            experimental::Expr::Invalid(i) => legacy::Expr::Invalid(legacy::Invalid {
                span: i.span(self.ast),
            }),
        })
    }

    fn compat_ident(&mut self, ident: experimental::Ident) -> legacy::Ident {
        legacy::Ident {
            span: ident.span(self.ast),
            ctxt: self.semantic.node_scope(ident).to_ctxt(),
            sym: self.compat_utf8_ref(ident.sym(self.ast)),
            optional: ident.optional(self.ast),
        }
    }

    fn compat_switch_case(&mut self, c: experimental::SwitchCase) -> legacy::SwitchCase {
        legacy::SwitchCase {
            span: c.span(self.ast),
            test: c.test(self.ast).map(|e| self.compat_expr(e)),
            cons: self.compat_type_sub_range(c.cons(self.ast), Self::compat_stmt),
        }
    }

    fn compat_catch_clause(&mut self, c: experimental::CatchClause) -> legacy::CatchClause {
        legacy::CatchClause {
            span: c.span(self.ast),
            param: c.param(self.ast).map(|p| self.compat_pat(p)),
            body: self.compat_block_stmt(c.body(self.ast)),
        }
    }

    fn compat_var_decl_or_expr(&mut self, v: experimental::VarDeclOrExpr) -> legacy::VarDeclOrExpr {
        match v {
            experimental::VarDeclOrExpr::VarDecl(d) => {
                legacy::VarDeclOrExpr::VarDecl(Box::new(self.compat_var_decl(d)))
            }
            experimental::VarDeclOrExpr::Expr(e) => {
                legacy::VarDeclOrExpr::Expr(self.compat_expr(e))
            }
        }
    }

    fn compat_var_decl(&mut self, v: experimental::VarDecl) -> legacy::VarDecl {
        legacy::VarDecl {
            span: v.span(self.ast),
            ctxt: Default::default(),
            kind: match v.kind(self.ast) {
                experimental::VarDeclKind::Var => legacy::VarDeclKind::Var,
                experimental::VarDeclKind::Let => legacy::VarDeclKind::Let,
                experimental::VarDeclKind::Const => legacy::VarDeclKind::Const,
            },
            declare: v.declare(self.ast),
            decls: self.compat_type_sub_range(v.decls(self.ast), Self::compat_var_declarator),
        }
    }

    fn compat_for_head(&mut self, h: experimental::ForHead) -> legacy::ForHead {
        match h {
            experimental::ForHead::VarDecl(v) => {
                legacy::ForHead::VarDecl(Box::new(self.compat_var_decl(v)))
            }
            experimental::ForHead::UsingDecl(u) => {
                legacy::ForHead::UsingDecl(Box::new(legacy::UsingDecl {
                    span: u.span(self.ast),
                    is_await: u.is_await(self.ast),
                    decls: self
                        .compat_type_sub_range(u.decls(self.ast), Self::compat_var_declarator),
                }))
            }
            experimental::ForHead::Pat(p) => legacy::ForHead::Pat(Box::new(self.compat_pat(p))),
        }
    }

    // -------------------------------------------------------------------------------
    // Helpers for module declarations and common nodes

    fn compat_import_specifier(
        &mut self,

        s: experimental::ImportSpecifier,
    ) -> legacy::ImportSpecifier {
        match s {
            experimental::ImportSpecifier::Named(n) => {
                legacy::ImportSpecifier::Named(legacy::ImportNamedSpecifier {
                    span: n.span(self.ast),
                    local: self.compat_ident(n.local(self.ast)),
                    imported: n
                        .imported(self.ast)
                        .map(|me| self.compat_module_export_name(me)),
                    is_type_only: n.is_type_only(self.ast),
                })
            }
            experimental::ImportSpecifier::Default(d) => {
                legacy::ImportSpecifier::Default(legacy::ImportDefaultSpecifier {
                    span: d.span(self.ast),
                    local: self.compat_ident(d.local(self.ast)),
                })
            }
            experimental::ImportSpecifier::Namespace(ns) => {
                legacy::ImportSpecifier::Namespace(legacy::ImportStarAsSpecifier {
                    span: ns.span(self.ast),
                    local: self.compat_ident(ns.local(self.ast)),
                })
            }
        }
    }

    fn compat_export_specifier(
        &mut self,

        s: experimental::ExportSpecifier,
    ) -> legacy::ExportSpecifier {
        match s {
            experimental::ExportSpecifier::Namespace(ns) => {
                legacy::ExportSpecifier::Namespace(legacy::ExportNamespaceSpecifier {
                    span: ns.span(self.ast),
                    name: self.compat_module_export_name(ns.name(self.ast)),
                })
            }
            experimental::ExportSpecifier::Default(d) => {
                legacy::ExportSpecifier::Default(legacy::ExportDefaultSpecifier {
                    exported: self.compat_ident(d.exported(self.ast)),
                })
            }
            experimental::ExportSpecifier::Named(n) => {
                legacy::ExportSpecifier::Named(legacy::ExportNamedSpecifier {
                    span: n.span(self.ast),
                    orig: self.compat_module_export_name(n.orig(self.ast)),
                    exported: n
                        .exported(self.ast)
                        .map(|me| self.compat_module_export_name(me)),
                    is_type_only: n.is_type_only(self.ast),
                })
            }
        }
    }

    fn compat_module_export_name(
        &mut self,

        n: experimental::ModuleExportName,
    ) -> legacy::ModuleExportName {
        match n {
            experimental::ModuleExportName::Ident(i) => {
                legacy::ModuleExportName::Ident(self.compat_ident(i))
            }
            experimental::ModuleExportName::Str(s) => {
                legacy::ModuleExportName::Str(self.compat_str(s))
            }
        }
    }

    fn compat_object_lit(&mut self, o: experimental::ObjectLit) -> legacy::ObjectLit {
        legacy::ObjectLit {
            span: o.span(self.ast),
            props: self.compat_type_sub_range(o.props(self.ast), Self::compat_prop_or_spread),
        }
    }

    fn compat_prop_or_spread(&mut self, p: experimental::PropOrSpread) -> legacy::PropOrSpread {
        match p {
            experimental::PropOrSpread::SpreadElement(s) => {
                legacy::PropOrSpread::Spread(legacy::SpreadElement {
                    dot3_token: s.dot_3_token(self.ast),
                    expr: self.compat_expr(s.expr(self.ast)),
                })
            }
            experimental::PropOrSpread::Prop(prop) => {
                legacy::PropOrSpread::Prop(Box::new(self.compat_prop(prop)))
            }
        }
    }

    fn compat_prop(&mut self, p: experimental::Prop) -> legacy::Prop {
        match p {
            experimental::Prop::Shorthand(i) => legacy::Prop::Shorthand(self.compat_ident(i)),
            experimental::Prop::KeyValue(kv) => legacy::Prop::KeyValue(legacy::KeyValueProp {
                key: self.compat_prop_name(kv.key(self.ast)),
                value: self.compat_expr(kv.value(self.ast)),
            }),
            experimental::Prop::Assign(ap) => legacy::Prop::Assign(legacy::AssignProp {
                span: ap.span(self.ast),
                key: self.compat_ident(ap.key(self.ast)),
                value: self.compat_expr(ap.value(self.ast)),
            }),
            experimental::Prop::Getter(g) => legacy::Prop::Getter(legacy::GetterProp {
                span: g.span(self.ast),
                key: self.compat_prop_name(g.key(self.ast)),
                type_ann: None,
                body: g.body(self.ast).map(|b| self.compat_block_stmt(b)),
            }),
            experimental::Prop::Setter(s) => legacy::Prop::Setter(legacy::SetterProp {
                span: s.span(self.ast),
                key: self.compat_prop_name(s.key(self.ast)),
                this_param: s.this_param(self.ast).map(|p| self.compat_pat(p)),
                param: Box::new(self.compat_pat(s.param(self.ast))),
                body: s.body(self.ast).map(|b| self.compat_block_stmt(b)),
            }),
            experimental::Prop::Method(m) => legacy::Prop::Method(legacy::MethodProp {
                key: self.compat_prop_name(m.key(self.ast)),
                function: Box::new(self.compat_function(m.function(self.ast))),
            }),
        }
    }

    fn compat_prop_name(&mut self, n: experimental::PropName) -> legacy::PropName {
        match n {
            experimental::PropName::Ident(i) => legacy::PropName::Ident(legacy::IdentName {
                span: i.span(self.ast),
                sym: self.compat_utf8_ref(i.sym(self.ast)),
            }),
            experimental::PropName::Str(s) => legacy::PropName::Str(self.compat_str(s)),
            experimental::PropName::Num(n) => legacy::PropName::Num(legacy::Number {
                span: n.span(self.ast),
                value: n.value(self.ast),
                raw: self.compat_opt_utf8_ref(n.raw(self.ast)),
            }),
            experimental::PropName::Computed(c) => {
                legacy::PropName::Computed(legacy::ComputedPropName {
                    span: c.span(self.ast),
                    expr: self.compat_expr(c.expr(self.ast)),
                })
            }
            experimental::PropName::BigInt(b) => legacy::PropName::BigInt(legacy::BigInt {
                span: b.span(self.ast),
                value: Box::new(self.compat_big_int(b.value(self.ast))),
                raw: self.compat_opt_utf8_ref(b.raw(self.ast)),
            }),
        }
    }

    fn compat_str(&mut self, s: experimental::Str) -> legacy::Str {
        legacy::Str {
            span: s.span(self.ast),
            value: self.compat_wtf8_ref(s.value(self.ast)),
            raw: self.compat_opt_utf8_ref(s.raw(self.ast)),
        }
    }

    // -------------------------------------------------------------------------------
    // Function / Class basics used by export default

    fn compat_fn_expr(&mut self, f: experimental::FnExpr) -> legacy::FnExpr {
        legacy::FnExpr {
            ident: f.ident(self.ast).map(|i| self.compat_ident(i)),
            function: Box::new(self.compat_function(f.function(self.ast))),
        }
    }

    fn compat_class_expr(&mut self, c: experimental::ClassExpr) -> legacy::ClassExpr {
        legacy::ClassExpr {
            ident: c.ident(self.ast).map(|i| self.compat_ident(i)),
            class: Box::new(self.compat_class(c.class(self.ast))),
        }
    }

    fn compat_function(&mut self, f: experimental::Function) -> legacy::Function {
        legacy::Function {
            params: self.compat_type_sub_range(f.params(self.ast), Self::compat_param),
            decorators: self.compat_type_sub_range(f.decorators(self.ast), Self::compat_decorator),
            span: f.span(self.ast),
            ctxt: Default::default(),
            body: f.body(self.ast).map(|b| self.compat_block_stmt(b)),
            is_generator: f.is_generator(self.ast),
            is_async: f.is_async(self.ast),
            type_params: None,
            return_type: None,
        }
    }

    fn compat_param(&mut self, p: experimental::Param) -> legacy::Param {
        legacy::Param {
            span: p.span(self.ast),
            decorators: self.compat_type_sub_range(p.decorators(self.ast), Self::compat_decorator),
            pat: self.compat_pat(p.pat(self.ast)),
        }
    }

    fn compat_decorator(&mut self, d: experimental::Decorator) -> legacy::Decorator {
        legacy::Decorator {
            span: d.span(self.ast),
            expr: self.compat_expr(d.expr(self.ast)),
        }
    }

    fn compat_class(&mut self, c: experimental::Class) -> legacy::Class {
        legacy::Class {
            span: c.span(self.ast),
            ctxt: Default::default(),
            decorators: self.compat_type_sub_range(c.decorators(self.ast), Self::compat_decorator),
            body: self.compat_type_sub_range(c.body(self.ast), Self::compat_class_member),
            super_class: c.super_class(self.ast).map(|e| self.compat_expr(e)),
            is_abstract: c.is_abstract(self.ast),
            type_params: None,
            super_type_params: None,
            implements: Default::default(),
        }
    }

    fn compat_class_member(&mut self, m: experimental::ClassMember) -> legacy::ClassMember {
        match m {
            experimental::ClassMember::Constructor(k) => {
                legacy::ClassMember::Constructor(legacy::Constructor {
                    span: k.span(self.ast),
                    ctxt: Default::default(),
                    key: self.compat_prop_name(k.key(self.ast)),
                    params: self.compat_type_sub_range(
                        k.params(self.ast),
                        Self::compat_param_or_ts_param_prop,
                    ),
                    body: k.body(self.ast).map(|b| self.compat_block_stmt(b)),
                    accessibility: None,
                    is_optional: false,
                })
            }
            experimental::ClassMember::Method(me) => {
                legacy::ClassMember::Method(legacy::ClassMethod {
                    span: me.span(self.ast),
                    key: self.compat_prop_name(me.key(self.ast)),
                    function: Box::new(self.compat_function(me.function(self.ast))),
                    kind: match me.kind(self.ast) {
                        experimental::MethodKind::Method => legacy::MethodKind::Method,
                        experimental::MethodKind::Getter => legacy::MethodKind::Getter,
                        experimental::MethodKind::Setter => legacy::MethodKind::Setter,
                    },
                    is_static: me.is_static(self.ast),
                    accessibility: None,
                    is_abstract: false,
                    is_optional: false,
                    is_override: false,
                })
            }
            experimental::ClassMember::PrivateMethod(pm) => {
                legacy::ClassMember::PrivateMethod(legacy::PrivateMethod {
                    span: pm.span(self.ast),
                    key: legacy::PrivateName {
                        span: pm.key(self.ast).span(self.ast),
                        name: self.compat_utf8_ref(pm.key(self.ast).name(self.ast)),
                    },
                    function: Box::new(self.compat_function(pm.function(self.ast))),
                    kind: match pm.kind(self.ast) {
                        experimental::MethodKind::Method => legacy::MethodKind::Method,
                        experimental::MethodKind::Getter => legacy::MethodKind::Getter,
                        experimental::MethodKind::Setter => legacy::MethodKind::Setter,
                    },
                    is_static: pm.is_static(self.ast),
                    accessibility: None,
                    is_abstract: false,
                    is_optional: false,
                    is_override: false,
                })
            }
            experimental::ClassMember::ClassProp(cp) => {
                legacy::ClassMember::ClassProp(legacy::ClassProp {
                    span: cp.span(self.ast),
                    key: self.compat_prop_name(cp.key(self.ast)),
                    value: cp.value(self.ast).map(|e| self.compat_expr(e)),
                    type_ann: None,
                    is_static: cp.is_static(self.ast),
                    decorators: self
                        .compat_type_sub_range(cp.decorators(self.ast), Self::compat_decorator),
                    accessibility: None,
                    is_abstract: false,
                    is_optional: false,
                    is_override: false,
                    readonly: false,
                    declare: false,
                    definite: false,
                })
            }
            experimental::ClassMember::PrivateProp(pp) => {
                legacy::ClassMember::PrivateProp(legacy::PrivateProp {
                    span: pp.span(self.ast),
                    ctxt: Default::default(),
                    key: legacy::PrivateName {
                        span: pp.key(self.ast).span(self.ast),
                        name: self.compat_utf8_ref(pp.key(self.ast).name(self.ast)),
                    },
                    value: pp.value(self.ast).map(|e| self.compat_expr(e)),
                    type_ann: None,
                    is_static: pp.is_static(self.ast),
                    decorators: self
                        .compat_type_sub_range(pp.decorators(self.ast), Self::compat_decorator),
                    accessibility: None,
                    is_optional: false,
                    is_override: false,
                    readonly: false,
                    definite: false,
                })
            }
            experimental::ClassMember::Empty(e) => {
                legacy::ClassMember::Empty(self.compat_empty_stmt(e))
            }
            experimental::ClassMember::StaticBlock(sb) => {
                legacy::ClassMember::StaticBlock(legacy::StaticBlock {
                    span: sb.span(self.ast),
                    body: self.compat_block_stmt(sb.body(self.ast)),
                })
            }
            experimental::ClassMember::AutoAccessor(a) => {
                legacy::ClassMember::AutoAccessor(legacy::AutoAccessor {
                    span: a.span(self.ast),
                    key: match a.key(self.ast) {
                        experimental::Key::Private(p) => {
                            legacy::Key::Private(legacy::PrivateName {
                                span: p.span(self.ast),
                                name: self.compat_utf8_ref(p.name(self.ast)),
                            })
                        }
                        experimental::Key::Public(n) => {
                            legacy::Key::Public(self.compat_prop_name(n))
                        }
                    },
                    value: a.value(self.ast).map(|e| self.compat_expr(e)),
                    type_ann: None,
                    is_static: a.is_static(self.ast),
                    decorators: self
                        .compat_type_sub_range(a.decorators(self.ast), Self::compat_decorator),
                    accessibility: None,
                    is_abstract: false,
                    is_override: false,
                    definite: false,
                })
            }
        }
    }

    fn compat_param_or_ts_param_prop(
        &mut self,

        p: experimental::ParamOrTsParamProp,
    ) -> legacy::ParamOrTsParamProp {
        match p {
            experimental::ParamOrTsParamProp::Param(pp) => {
                legacy::ParamOrTsParamProp::Param(self.compat_param(pp))
            }
        }
    }

    // -------------------------------------------------------------------------------
    // Patterns and declarations

    fn compat_decl(&mut self, d: experimental::Decl) -> legacy::Decl {
        match d {
            experimental::Decl::Class(c) => legacy::Decl::Class(legacy::ClassDecl {
                ident: self.compat_ident(c.ident(self.ast)),
                declare: c.declare(self.ast),
                class: Box::new(self.compat_class(c.class(self.ast))),
            }),
            experimental::Decl::Fn(f) => legacy::Decl::Fn(legacy::FnDecl {
                ident: self.compat_ident(f.ident(self.ast)),
                declare: f.declare(self.ast),
                function: Box::new(self.compat_function(f.function(self.ast))),
            }),
            experimental::Decl::Var(v) => legacy::Decl::Var(Box::new(self.compat_var_decl(v))),
            experimental::Decl::Using(u) => legacy::Decl::Using(Box::new(legacy::UsingDecl {
                span: u.span(self.ast),
                is_await: u.is_await(self.ast),
                decls: self.compat_type_sub_range(u.decls(self.ast), Self::compat_var_declarator),
            })),
        }
    }

    fn compat_var_declarator(&mut self, d: experimental::VarDeclarator) -> legacy::VarDeclarator {
        legacy::VarDeclarator {
            span: d.span(self.ast),
            name: self.compat_pat(d.name(self.ast)),
            init: d.init(self.ast).map(|e| self.compat_expr(e)),
            definite: false,
        }
    }

    fn compat_pat(&mut self, p: experimental::Pat) -> legacy::Pat {
        match p {
            experimental::Pat::Ident(b) => legacy::Pat::Ident(legacy::BindingIdent {
                id: self.compat_ident(b.id(self.ast)),
                type_ann: None,
            }),
            experimental::Pat::Array(a) => legacy::Pat::Array(legacy::ArrayPat {
                span: a.span(self.ast),
                elems: self.compat_type_sub_range(a.elems(self.ast), |this, p| {
                    p.map(|p| this.compat_pat(p))
                }),
                optional: a.optional(self.ast),
                type_ann: None,
            }),
            experimental::Pat::Rest(r) => legacy::Pat::Rest(legacy::RestPat {
                span: r.span(self.ast),
                dot3_token: r.dot_3_token(self.ast),
                arg: Box::new(self.compat_pat(r.arg(self.ast))),
                type_ann: None,
            }),
            experimental::Pat::Object(o) => legacy::Pat::Object(legacy::ObjectPat {
                span: o.span(self.ast),
                props: self.compat_type_sub_range(o.props(self.ast), Self::compat_object_pat_prop),
                optional: o.optional(self.ast),
                type_ann: None,
            }),
            experimental::Pat::Assign(a) => legacy::Pat::Assign(legacy::AssignPat {
                span: a.span(self.ast),
                left: Box::new(self.compat_pat(a.left(self.ast))),
                right: self.compat_expr(a.right(self.ast)),
            }),
            experimental::Pat::Invalid(i) => legacy::Pat::Invalid(legacy::Invalid {
                span: i.span(self.ast),
            }),
            experimental::Pat::Expr(e) => legacy::Pat::Expr(self.compat_expr(e)),
        }
    }

    fn compat_object_pat_prop(&mut self, p: experimental::ObjectPatProp) -> legacy::ObjectPatProp {
        match p {
            experimental::ObjectPatProp::KeyValue(kv) => {
                legacy::ObjectPatProp::KeyValue(legacy::KeyValuePatProp {
                    key: self.compat_prop_name(kv.key(self.ast)),
                    value: Box::new(self.compat_pat(kv.value(self.ast))),
                })
            }
            experimental::ObjectPatProp::Assign(ap) => {
                legacy::ObjectPatProp::Assign(legacy::AssignPatProp {
                    span: ap.span(self.ast),
                    key: legacy::BindingIdent {
                        id: self.compat_ident(ap.key(self.ast).id(self.ast)),
                        type_ann: None,
                    },
                    value: ap.value(self.ast).map(|e| self.compat_expr(e)),
                })
            }
            experimental::ObjectPatProp::Rest(r) => legacy::ObjectPatProp::Rest(legacy::RestPat {
                span: r.span(self.ast),
                dot3_token: r.dot_3_token(self.ast),
                arg: Box::new(self.compat_pat(r.arg(self.ast))),
                type_ann: None,
            }),
        }
    }
    fn compat_expr_or_spread(&mut self, e: experimental::ExprOrSpread) -> legacy::ExprOrSpread {
        legacy::ExprOrSpread {
            spread: e.spread(self.ast).map(|s| s.span(self.ast)),
            expr: self.compat_expr(e.expr(self.ast)),
        }
    }

    fn compat_member_prop(&mut self, p: experimental::MemberProp) -> legacy::MemberProp {
        match p {
            experimental::MemberProp::Ident(i) => legacy::MemberProp::Ident(legacy::IdentName {
                span: i.span(self.ast),
                sym: self.compat_utf8_ref(i.sym(self.ast)),
            }),
            experimental::MemberProp::PrivateName(pn) => {
                legacy::MemberProp::PrivateName(legacy::PrivateName {
                    span: pn.span(self.ast),
                    name: self.compat_utf8_ref(pn.name(self.ast)),
                })
            }
            experimental::MemberProp::Computed(c) => {
                legacy::MemberProp::Computed(legacy::ComputedPropName {
                    span: c.span(self.ast),
                    expr: self.compat_expr(c.expr(self.ast)),
                })
            }
        }
    }

    fn compat_lit(&mut self, l: experimental::Lit) -> legacy::Lit {
        match l {
            experimental::Lit::Str(s) => legacy::Lit::Str(self.compat_str(s)),
            experimental::Lit::Bool(b) => legacy::Lit::Bool(legacy::Bool {
                span: b.span(self.ast),
                value: b.value(self.ast),
            }),
            experimental::Lit::Null(n) => legacy::Lit::Null(legacy::Null {
                span: n.span(self.ast),
            }),
            experimental::Lit::Num(n) => legacy::Lit::Num(legacy::Number {
                span: n.span(self.ast),
                value: n.value(self.ast),
                raw: self.compat_opt_utf8_ref(n.raw(self.ast)),
            }),
            experimental::Lit::BigInt(b) => legacy::Lit::BigInt(legacy::BigInt {
                span: b.span(self.ast),
                value: Box::new(self.compat_big_int(b.value(self.ast))),
                raw: self.compat_opt_utf8_ref(b.raw(self.ast)),
            }),
            experimental::Lit::Regex(r) => legacy::Lit::Regex(legacy::Regex {
                span: r.span(self.ast),
                exp: self.compat_utf8_ref(r.exp(self.ast)),
                flags: self.compat_utf8_ref(r.flags(self.ast)),
            }),
        }
    }

    fn compat_tpl_element(&mut self, e: experimental::TplElement) -> legacy::TplElement {
        legacy::TplElement {
            span: e.span(self.ast),
            tail: e.tail(self.ast),
            cooked: self.compat_opt_wtf8_ref(e.cooked(self.ast)),
            raw: self.compat_utf8_ref(e.raw(self.ast)),
        }
    }

    fn compat_tpl(&mut self, t: experimental::Tpl) -> legacy::Tpl {
        legacy::Tpl {
            span: t.span(self.ast),
            exprs: self.compat_type_sub_range(t.exprs(self.ast), Self::compat_expr),
            quasis: self.compat_type_sub_range(t.quasis(self.ast), Self::compat_tpl_element),
        }
    }

    fn compat_assign_target(&mut self, t: experimental::AssignTarget) -> legacy::AssignTarget {
        match t {
            experimental::AssignTarget::Simple(s) => legacy::AssignTarget::Simple(match s {
                experimental::SimpleAssignTarget::Ident(b) => {
                    legacy::SimpleAssignTarget::Ident(legacy::BindingIdent {
                        id: self.compat_ident(b.id(self.ast)),
                        type_ann: None,
                    })
                }
                experimental::SimpleAssignTarget::Member(m) => {
                    legacy::SimpleAssignTarget::Member(legacy::MemberExpr {
                        span: m.span(self.ast),
                        obj: self.compat_expr(m.obj(self.ast)),
                        prop: self.compat_member_prop(m.prop(self.ast)),
                    })
                }
                experimental::SimpleAssignTarget::SuperProp(su) => {
                    legacy::SimpleAssignTarget::SuperProp(legacy::SuperPropExpr {
                        span: su.span(self.ast),
                        obj: legacy::Super {
                            span: su.obj(self.ast).span(self.ast),
                        },
                        prop: match su.prop(self.ast) {
                            experimental::SuperProp::Ident(i) => {
                                legacy::SuperProp::Ident(legacy::IdentName {
                                    span: i.span(self.ast),
                                    sym: self.compat_utf8_ref(i.sym(self.ast)),
                                })
                            }
                            experimental::SuperProp::Computed(c) => {
                                legacy::SuperProp::Computed(legacy::ComputedPropName {
                                    span: c.span(self.ast),
                                    expr: self.compat_expr(c.expr(self.ast)),
                                })
                            }
                        },
                    })
                }
                experimental::SimpleAssignTarget::Paren(p) => {
                    legacy::SimpleAssignTarget::Paren(legacy::ParenExpr {
                        span: p.span(self.ast),
                        expr: self.compat_expr(p.expr(self.ast)),
                    })
                }
                experimental::SimpleAssignTarget::OptChain(o) => {
                    legacy::SimpleAssignTarget::OptChain(legacy::OptChainExpr {
                        span: o.span(self.ast),
                        optional: o.optional(self.ast),
                        base: Box::new(match o.base(self.ast) {
                            experimental::OptChainBase::Member(m) => {
                                legacy::OptChainBase::Member(legacy::MemberExpr {
                                    span: m.span(self.ast),
                                    obj: self.compat_expr(m.obj(self.ast)),
                                    prop: self.compat_member_prop(m.prop(self.ast)),
                                })
                            }
                            experimental::OptChainBase::Call(c) => {
                                legacy::OptChainBase::Call(legacy::OptCall {
                                    span: c.span(self.ast),
                                    ctxt: Default::default(),
                                    callee: self.compat_expr(c.callee(self.ast)),
                                    args: self.compat_type_sub_range(
                                        c.args(self.ast),
                                        Self::compat_expr_or_spread,
                                    ),
                                    type_args: None,
                                })
                            }
                        }),
                    })
                }
                experimental::SimpleAssignTarget::Invalid(i) => {
                    legacy::SimpleAssignTarget::Invalid(legacy::Invalid {
                        span: i.span(self.ast),
                    })
                }
            }),
            experimental::AssignTarget::Pat(p) => legacy::AssignTarget::Pat(match p {
                experimental::AssignTargetPat::Array(a) => {
                    legacy::AssignTargetPat::Array(legacy::ArrayPat {
                        span: a.span(self.ast),
                        elems: self.compat_type_sub_range(a.elems(self.ast), |this, pat| {
                            pat.map(|pat| this.compat_pat(pat))
                        }),
                        optional: false,
                        type_ann: None,
                    })
                }
                experimental::AssignTargetPat::Object(o) => {
                    legacy::AssignTargetPat::Object(legacy::ObjectPat {
                        span: o.span(self.ast),
                        props: self
                            .compat_type_sub_range(o.props(self.ast), Self::compat_object_pat_prop),
                        optional: false,
                        type_ann: None,
                    })
                }
                experimental::AssignTargetPat::Invalid(i) => {
                    legacy::AssignTargetPat::Invalid(legacy::Invalid {
                        span: i.span(self.ast),
                    })
                }
            }),
        }
    }

    // -------------------------------------------------------------------------------
    // JSX compatibility helpers

    fn compat_jsx_object(&mut self, o: experimental::JSXObject) -> legacy::JSXObject {
        match o {
            experimental::JSXObject::JSXMemberExpr(m) => {
                legacy::JSXObject::JSXMemberExpr(Box::new(self.compat_jsx_member_expr(m)))
            }
            experimental::JSXObject::Ident(i) => legacy::JSXObject::Ident(self.compat_ident(i)),
        }
    }

    fn compat_ident_name(&mut self, i: experimental::IdentName) -> legacy::IdentName {
        legacy::IdentName {
            span: i.span(self.ast),
            sym: self.compat_utf8_ref(i.sym(self.ast)),
        }
    }

    fn compat_jsx_member_expr(&mut self, j: experimental::JSXMemberExpr) -> legacy::JSXMemberExpr {
        legacy::JSXMemberExpr {
            span: j.span(self.ast),
            obj: self.compat_jsx_object(j.obj(self.ast)),
            prop: self.compat_ident_name(j.prop(self.ast)),
        }
    }

    fn compat_jsx_namespaced_name(
        &mut self,

        j: experimental::JSXNamespacedName,
    ) -> legacy::JSXNamespacedName {
        legacy::JSXNamespacedName {
            span: j.span(self.ast),
            ns: self.compat_ident_name(j.ns(self.ast)),
            name: self.compat_ident_name(j.name(self.ast)),
        }
    }

    fn compat_jsx_empty_expr(&mut self, j: experimental::JSXEmptyExpr) -> legacy::JSXEmptyExpr {
        legacy::JSXEmptyExpr {
            span: j.span(self.ast),
        }
    }

    fn compat_jsx_expr(&mut self, e: experimental::JSXExpr) -> legacy::JSXExpr {
        match e {
            experimental::JSXExpr::JSXEmptyExpr(ee) => {
                legacy::JSXExpr::JSXEmptyExpr(self.compat_jsx_empty_expr(ee))
            }
            experimental::JSXExpr::Expr(ex) => legacy::JSXExpr::Expr(self.compat_expr(ex)),
        }
    }

    fn compat_jsx_expr_container(
        &mut self,

        c: experimental::JSXExprContainer,
    ) -> legacy::JSXExprContainer {
        legacy::JSXExprContainer {
            span: c.span(self.ast),
            expr: self.compat_jsx_expr(c.expr(self.ast)),
        }
    }

    fn compat_spread_element(&mut self, s: experimental::SpreadElement) -> legacy::SpreadElement {
        legacy::SpreadElement {
            dot3_token: s.dot_3_token(self.ast),
            expr: self.compat_expr(s.expr(self.ast)),
        }
    }

    fn compat_jsx_attr_name(&mut self, n: experimental::JSXAttrName) -> legacy::JSXAttrName {
        match n {
            experimental::JSXAttrName::Ident(i) => {
                legacy::JSXAttrName::Ident(self.compat_ident_name(i))
            }
            experimental::JSXAttrName::JSXNamespacedName(nn) => {
                legacy::JSXAttrName::JSXNamespacedName(self.compat_jsx_namespaced_name(nn))
            }
        }
    }

    fn compat_jsx_attr_value(&mut self, v: experimental::JSXAttrValue) -> legacy::JSXAttrValue {
        match v {
            experimental::JSXAttrValue::Str(s) => legacy::JSXAttrValue::Str(self.compat_str(s)),
            experimental::JSXAttrValue::JSXExprContainer(c) => {
                legacy::JSXAttrValue::JSXExprContainer(self.compat_jsx_expr_container(c))
            }
            experimental::JSXAttrValue::JSXElement(e) => {
                legacy::JSXAttrValue::JSXElement(Box::new(self.compat_jsx_element(e)))
            }
            experimental::JSXAttrValue::JSXFragment(f) => {
                legacy::JSXAttrValue::JSXFragment(self.compat_jsx_fragment(f))
            }
        }
    }

    fn compat_jsx_attr(&mut self, a: experimental::JSXAttr) -> legacy::JSXAttr {
        legacy::JSXAttr {
            span: a.span(self.ast),
            name: self.compat_jsx_attr_name(a.name(self.ast)),
            value: a.value(self.ast).map(|v| self.compat_jsx_attr_value(v)),
        }
    }

    fn compat_jsx_attr_or_spread(
        &mut self,

        a: experimental::JSXAttrOrSpread,
    ) -> legacy::JSXAttrOrSpread {
        match a {
            experimental::JSXAttrOrSpread::JSXAttr(attr) => {
                legacy::JSXAttrOrSpread::JSXAttr(self.compat_jsx_attr(attr))
            }
            experimental::JSXAttrOrSpread::SpreadElement(se) => {
                legacy::JSXAttrOrSpread::SpreadElement(self.compat_spread_element(se))
            }
        }
    }

    fn compat_jsx_element_name(
        &mut self,

        n: experimental::JSXElementName,
    ) -> legacy::JSXElementName {
        match n {
            experimental::JSXElementName::Ident(i) => {
                legacy::JSXElementName::Ident(self.compat_ident(i))
            }
            experimental::JSXElementName::JSXMemberExpr(m) => {
                legacy::JSXElementName::JSXMemberExpr(self.compat_jsx_member_expr(m))
            }
            experimental::JSXElementName::JSXNamespacedName(nn) => {
                legacy::JSXElementName::JSXNamespacedName(self.compat_jsx_namespaced_name(nn))
            }
        }
    }

    fn compat_jsx_opening_element(
        &mut self,

        o: experimental::JSXOpeningElement,
    ) -> legacy::JSXOpeningElement {
        legacy::JSXOpeningElement {
            span: o.span(self.ast),
            name: self.compat_jsx_element_name(o.name(self.ast)),
            attrs: self.compat_type_sub_range(o.attrs(self.ast), Self::compat_jsx_attr_or_spread),
            self_closing: o.self_closing(self.ast),
            type_args: None,
        }
    }

    fn compat_jsx_closing_element(
        &mut self,

        c: experimental::JSXClosingElement,
    ) -> legacy::JSXClosingElement {
        legacy::JSXClosingElement {
            span: c.span(self.ast),
            name: self.compat_jsx_element_name(c.name(self.ast)),
        }
    }

    fn compat_jsx_text(&mut self, t: experimental::JSXText) -> legacy::JSXText {
        legacy::JSXText {
            span: t.span(self.ast),
            value: self.compat_utf8_ref(t.value(self.ast)),
            raw: self.compat_utf8_ref(t.raw(self.ast)),
        }
    }

    fn compat_jsx_spread_child(
        &mut self,

        s: experimental::JSXSpreadChild,
    ) -> legacy::JSXSpreadChild {
        legacy::JSXSpreadChild {
            span: s.span(self.ast),
            expr: self.compat_expr(s.expr(self.ast)),
        }
    }

    fn compat_jsx_element_child(
        &mut self,

        c: experimental::JSXElementChild,
    ) -> legacy::JSXElementChild {
        match c {
            experimental::JSXElementChild::JSXText(t) => {
                legacy::JSXElementChild::JSXText(self.compat_jsx_text(t))
            }
            experimental::JSXElementChild::JSXExprContainer(ec) => {
                legacy::JSXElementChild::JSXExprContainer(self.compat_jsx_expr_container(ec))
            }
            experimental::JSXElementChild::JSXSpreadChild(sc) => {
                legacy::JSXElementChild::JSXSpreadChild(self.compat_jsx_spread_child(sc))
            }
            experimental::JSXElementChild::JSXElement(e) => {
                legacy::JSXElementChild::JSXElement(Box::new(self.compat_jsx_element(e)))
            }
            experimental::JSXElementChild::JSXFragment(f) => {
                legacy::JSXElementChild::JSXFragment(self.compat_jsx_fragment(f))
            }
        }
    }

    fn compat_jsx_element(&mut self, e: experimental::JSXElement) -> legacy::JSXElement {
        legacy::JSXElement {
            span: e.span(self.ast),
            opening: self.compat_jsx_opening_element(e.opening(self.ast)),
            children: self
                .compat_type_sub_range(e.children(self.ast), Self::compat_jsx_element_child),
            closing: e
                .closing(self.ast)
                .map(|c| self.compat_jsx_closing_element(c)),
        }
    }

    fn compat_jsx_opening_fragment(
        &mut self,

        o: experimental::JSXOpeningFragment,
    ) -> legacy::JSXOpeningFragment {
        legacy::JSXOpeningFragment {
            span: o.span(self.ast),
        }
    }

    fn compat_jsx_closing_fragment(
        &mut self,

        c: experimental::JSXClosingFragment,
    ) -> legacy::JSXClosingFragment {
        legacy::JSXClosingFragment {
            span: c.span(self.ast),
        }
    }

    fn compat_jsx_fragment(&mut self, f: experimental::JSXFragment) -> legacy::JSXFragment {
        legacy::JSXFragment {
            span: f.span(self.ast),
            opening: self.compat_jsx_opening_fragment(f.opening(self.ast)),
            children: self
                .compat_type_sub_range(f.children(self.ast), Self::compat_jsx_element_child),
            closing: self.compat_jsx_closing_fragment(f.closing(self.ast)),
        }
    }

    // ===============================================================================

    fn compat_utf8_ref(&mut self, utf8_ref: Utf8Ref) -> Atom {
        Atom::new(self.ast.get_utf8(utf8_ref))
    }

    fn compat_opt_utf8_ref(&mut self, utf8_ref: OptionalUtf8Ref) -> Option<Atom> {
        utf8_ref
            .to_option()
            .map(|utf8_ref| self.compat_utf8_ref(utf8_ref))
    }

    fn compat_wtf8_ref(&mut self, wtf8_ref: Wtf8Ref) -> Wtf8Atom {
        Wtf8Atom::new(self.ast.get_wtf8(wtf8_ref))
    }

    fn compat_opt_wtf8_ref(&mut self, wtf8_ref: OptionalWtf8Ref) -> Option<Wtf8Atom> {
        wtf8_ref
            .to_option()
            .map(|wtf8_ref| self.compat_wtf8_ref(wtf8_ref))
    }

    fn compat_big_int(&mut self, big_int: BigIntId) -> legacy::BigIntValue {
        self.ast.get_big_int(big_int).clone()
    }

    fn compat_type_sub_range<T: ExtraDataCompact, U, F: Fn(&mut Self, T) -> U>(
        &mut self,
        typed_range: TypedSubRange<T>,
        transformer: F,
    ) -> Vec<U> {
        let mut ret = Vec::with_capacity(typed_range.len());
        for item in typed_range.iter() {
            ret.push(transformer(self, self.ast.get_node_in_sub_range(item)));
        }
        ret
    }
}
