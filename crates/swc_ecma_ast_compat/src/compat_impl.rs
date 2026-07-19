use swc_core::atoms::{Atom, Wtf8Atom};
use swc_core::common::{BytePos, Span as SwcSpan, SyntaxContext};
use swc_core::ecma::ast::{self as legacy};
use swc_experimental_ecma_ast::{
    self as experimental, Ast, Atom as ExperimentalAtom, ExtraDataCompact, GetSpan, TypedSubRange,
    Wtf8Atom as ExperimentalWtf8Atom,
};
use swc_experimental_ecma_semantic::resolver::Semantic;

fn compat_span(span: experimental::Span) -> SwcSpan {
    SwcSpan::new_with_checked(BytePos(span.start), BytePos(span.end))
}

macro_rules! alloc_box {
    ($self:expr, $value:expr) => {{
        let value = $value;
        $self.alloc_box(value)
    }};
}

pub(crate) trait CompatImpl<'ast> {
    fn ast(&self) -> &Ast<'ast>;

    fn semantic(&self) -> &Semantic;

    fn alloc_box<T>(&self, value: T) -> Box<T>;

    fn compat_type_sub_range<T: ExtraDataCompact<'ast>, U, F: Fn(&mut Self, T) -> U>(
        &mut self,
        typed_range: TypedSubRange<T>,
        transformer: F,
    ) -> Vec<U>;

    fn compat_program_inner(&mut self, root: experimental::Program) -> legacy::Program {
        match self.ast().program_data(root) {
            experimental::ProgramData::Module(module) => {
                legacy::Program::Module(self.compat_module_inner(module))
            }
            experimental::ProgramData::Script(script) => {
                legacy::Program::Script(self.compat_script_inner(script))
            }
        }
    }

    fn compat_module_inner(&mut self, module: experimental::Module) -> legacy::Module {
        #[allow(unused_mut)]
        let mut inner = || legacy::Module {
            span: compat_span(module.span(self.ast())),
            shebang: self.compat_opt_utf8_ref(module.shebang(self.ast())),
            body: self.compat_type_sub_range(module.body(self.ast()), Self::compat_module_item),
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

    fn compat_script_inner(&mut self, script: experimental::Script) -> legacy::Script {
        #[allow(unused_mut)]
        let mut inner = || legacy::Script {
            span: compat_span(script.span(self.ast())),
            body: self.compat_type_sub_range(script.body(self.ast()), Self::compat_stmt),
            shebang: self.compat_opt_utf8_ref(script.shebang(self.ast())),
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
        if let Some(module_decl) = item.as_module_decl(self.ast()) {
            legacy::ModuleItem::ModuleDecl(self.compat_module_decl(module_decl))
        } else {
            legacy::ModuleItem::Stmt(
                self.compat_stmt(
                    item.as_stmt(self.ast())
                        .expect("module item must be a statement"),
                ),
            )
        }
    }

    fn compat_module_decl(&mut self, module_decl: experimental::ModuleDecl) -> legacy::ModuleDecl {
        match self.ast().module_decl_data(module_decl) {
            experimental::ModuleDeclData::Import(import_decl) => {
                legacy::ModuleDecl::Import(self.compat_import_decl(import_decl))
            }
            experimental::ModuleDeclData::ExportDecl(export_decl) => {
                legacy::ModuleDecl::ExportDecl(self.compat_export_decl(export_decl))
            }
            experimental::ModuleDeclData::ExportNamed(named_export) => {
                legacy::ModuleDecl::ExportNamed(self.compat_export_named(named_export))
            }
            experimental::ModuleDeclData::ExportDefaultDecl(export_default_decl) => {
                legacy::ModuleDecl::ExportDefaultDecl(
                    self.compat_export_default_decl(export_default_decl),
                )
            }
            experimental::ModuleDeclData::ExportDefaultExpr(export_default_expr) => {
                legacy::ModuleDecl::ExportDefaultExpr(
                    self.compat_export_default_expr(export_default_expr),
                )
            }
            experimental::ModuleDeclData::ExportAll(export_all) => {
                legacy::ModuleDecl::ExportAll(self.compat_export_all(export_all))
            }
        }
    }

    fn compat_import_decl(&mut self, import_decl: experimental::ImportDecl) -> legacy::ImportDecl {
        legacy::ImportDecl {
            span: compat_span(import_decl.span(self.ast())),
            specifiers: self.compat_type_sub_range(
                import_decl.specifiers(self.ast()),
                Self::compat_import_specifier,
            ),
            src: alloc_box!(self, self.compat_str(import_decl.src(self.ast()))),
            type_only: import_decl.type_only(self.ast()),
            with: import_decl
                .with(self.ast())
                .map(|with| alloc_box!(self, self.compat_object_lit(with))),
            phase: match import_decl.phase(self.ast()) {
                experimental::ImportPhase::Evaluation => legacy::ImportPhase::Evaluation,
                experimental::ImportPhase::Source => legacy::ImportPhase::Source,
                experimental::ImportPhase::Defer => legacy::ImportPhase::Defer,
            },
        }
    }

    fn compat_export_decl(&mut self, export_decl: experimental::ExportDecl) -> legacy::ExportDecl {
        legacy::ExportDecl {
            span: compat_span(export_decl.span(self.ast())),
            decl: self.compat_decl(export_decl.decl(self.ast())),
        }
    }

    fn compat_export_named(
        &mut self,

        export_named: experimental::NamedExport,
    ) -> legacy::NamedExport {
        legacy::NamedExport {
            span: compat_span(export_named.span(self.ast())),
            specifiers: self.compat_type_sub_range(
                export_named.specifiers(self.ast()),
                Self::compat_export_specifier,
            ),
            src: export_named
                .src(self.ast())
                .map(|s| alloc_box!(self, self.compat_str(s))),
            type_only: export_named.type_only(self.ast()),
            with: export_named
                .with(self.ast())
                .map(|o| alloc_box!(self, self.compat_object_lit(o))),
        }
    }

    fn compat_export_default_decl(
        &mut self,

        export_default_decl: experimental::ExportDefaultDecl,
    ) -> legacy::ExportDefaultDecl {
        legacy::ExportDefaultDecl {
            span: compat_span(export_default_decl.span(self.ast())),
            decl: match self
                .ast()
                .default_decl_data(export_default_decl.decl(self.ast()))
            {
                experimental::DefaultDeclData::Class(cls) => {
                    legacy::DefaultDecl::Class(self.compat_class_expr(cls))
                }
                experimental::DefaultDeclData::Fn(f) => {
                    legacy::DefaultDecl::Fn(self.compat_fn_expr(f))
                }
            },
        }
    }

    fn compat_export_default_expr(
        &mut self,

        export_default_expr: experimental::ExportDefaultExpr,
    ) -> legacy::ExportDefaultExpr {
        legacy::ExportDefaultExpr {
            span: compat_span(export_default_expr.span(self.ast())),
            expr: self.compat_expr(export_default_expr.expr(self.ast())),
        }
    }

    fn compat_export_all(&mut self, export_all: experimental::ExportAll) -> legacy::ExportAll {
        legacy::ExportAll {
            span: compat_span(export_all.span(self.ast())),
            src: alloc_box!(self, self.compat_str(export_all.src(self.ast()))),
            type_only: export_all.type_only(self.ast()),
            with: export_all
                .with(self.ast())
                .map(|o| alloc_box!(self, self.compat_object_lit(o))),
        }
    }

    fn compat_stmt(&mut self, stmt: experimental::Stmt) -> legacy::Stmt {
        match self.ast().stmt_data(stmt) {
            experimental::StmtData::Block(block_stmt) => {
                legacy::Stmt::Block(self.compat_block_stmt(block_stmt))
            }
            experimental::StmtData::Empty(empty_stmt) => {
                legacy::Stmt::Empty(self.compat_empty_stmt(empty_stmt))
            }
            experimental::StmtData::Debugger(debugger_stmt) => {
                legacy::Stmt::Debugger(legacy::DebuggerStmt {
                    span: compat_span(debugger_stmt.span(self.ast())),
                })
            }
            experimental::StmtData::With(with_stmt) => legacy::Stmt::With(legacy::WithStmt {
                span: compat_span(with_stmt.span(self.ast())),
                obj: self.compat_expr(with_stmt.obj(self.ast())),
                body: alloc_box!(self, self.compat_stmt(with_stmt.body(self.ast()))),
            }),
            experimental::StmtData::Return(return_stmt) => {
                legacy::Stmt::Return(legacy::ReturnStmt {
                    span: compat_span(return_stmt.span(self.ast())),
                    arg: return_stmt.arg(self.ast()).map(|arg| self.compat_expr(arg)),
                })
            }
            experimental::StmtData::Labeled(labeled_stmt) => {
                legacy::Stmt::Labeled(legacy::LabeledStmt {
                    span: compat_span(labeled_stmt.span(self.ast())),
                    label: self.compat_ident(labeled_stmt.label(self.ast())),
                    body: alloc_box!(self, self.compat_stmt(labeled_stmt.body(self.ast()))),
                })
            }
            experimental::StmtData::Break(break_stmt) => legacy::Stmt::Break(legacy::BreakStmt {
                span: compat_span(break_stmt.span(self.ast())),
                label: break_stmt
                    .label(self.ast())
                    .map(|label| self.compat_ident(label)),
            }),
            experimental::StmtData::Continue(continue_stmt) => {
                legacy::Stmt::Continue(legacy::ContinueStmt {
                    span: compat_span(continue_stmt.span(self.ast())),
                    label: continue_stmt
                        .label(self.ast())
                        .map(|label| self.compat_ident(label)),
                })
            }
            experimental::StmtData::If(if_stmt) => legacy::Stmt::If(legacy::IfStmt {
                span: compat_span(if_stmt.span(self.ast())),
                test: self.compat_expr(if_stmt.test(self.ast())),
                cons: alloc_box!(self, self.compat_stmt(if_stmt.cons(self.ast()))),
                alt: if_stmt
                    .alt(self.ast())
                    .map(|alt| alloc_box!(self, self.compat_stmt(alt))),
            }),
            experimental::StmtData::Switch(switch_stmt) => {
                legacy::Stmt::Switch(legacy::SwitchStmt {
                    span: compat_span(switch_stmt.span(self.ast())),
                    discriminant: self.compat_expr(switch_stmt.discriminant(self.ast())),
                    cases: self.compat_type_sub_range(
                        switch_stmt.cases(self.ast()),
                        Self::compat_switch_case,
                    ),
                })
            }
            experimental::StmtData::Throw(throw_stmt) => legacy::Stmt::Throw(legacy::ThrowStmt {
                span: compat_span(throw_stmt.span(self.ast())),
                arg: self.compat_expr(throw_stmt.arg(self.ast())),
            }),
            experimental::StmtData::Try(try_stmt) => legacy::Stmt::Try(alloc_box!(
                self,
                legacy::TryStmt {
                    span: compat_span(try_stmt.span(self.ast())),
                    block: self.compat_block_stmt(try_stmt.block(self.ast())),
                    handler: try_stmt
                        .handler(self.ast())
                        .map(|handler| self.compat_catch_clause(handler)),
                    finalizer: try_stmt
                        .finalizer(self.ast())
                        .map(|finalizer| self.compat_block_stmt(finalizer)),
                }
            )),
            experimental::StmtData::While(while_stmt) => legacy::Stmt::While(legacy::WhileStmt {
                span: compat_span(while_stmt.span(self.ast())),
                test: self.compat_expr(while_stmt.test(self.ast())),
                body: alloc_box!(self, self.compat_stmt(while_stmt.body(self.ast()))),
            }),
            experimental::StmtData::DoWhile(do_while_stmt) => {
                legacy::Stmt::DoWhile(legacy::DoWhileStmt {
                    span: compat_span(do_while_stmt.span(self.ast())),
                    test: self.compat_expr(do_while_stmt.test(self.ast())),
                    body: alloc_box!(self, self.compat_stmt(do_while_stmt.body(self.ast()))),
                })
            }
            experimental::StmtData::For(for_stmt) => legacy::Stmt::For(legacy::ForStmt {
                span: compat_span(for_stmt.span(self.ast())),
                init: for_stmt
                    .init(self.ast())
                    .map(|i| self.compat_var_decl_or_expr(i)),
                test: for_stmt.test(self.ast()).map(|e| self.compat_expr(e)),
                update: for_stmt.update(self.ast()).map(|e| self.compat_expr(e)),
                body: alloc_box!(self, self.compat_stmt(for_stmt.body(self.ast()))),
            }),
            experimental::StmtData::ForIn(for_in_stmt) => legacy::Stmt::ForIn(legacy::ForInStmt {
                span: compat_span(for_in_stmt.span(self.ast())),
                left: self.compat_for_head(for_in_stmt.left(self.ast())),
                right: self.compat_expr(for_in_stmt.right(self.ast())),
                body: alloc_box!(self, self.compat_stmt(for_in_stmt.body(self.ast()))),
            }),
            experimental::StmtData::ForOf(for_of_stmt) => legacy::Stmt::ForOf(legacy::ForOfStmt {
                span: compat_span(for_of_stmt.span(self.ast())),
                is_await: for_of_stmt.is_await(self.ast()),
                left: self.compat_for_head(for_of_stmt.left(self.ast())),
                right: self.compat_expr(for_of_stmt.right(self.ast())),
                body: alloc_box!(self, self.compat_stmt(for_of_stmt.body(self.ast()))),
            }),
            experimental::StmtData::Decl(node) => legacy::Stmt::Decl(self.compat_decl(node)),
            experimental::StmtData::Expr(expr_stmt) => legacy::Stmt::Expr(legacy::ExprStmt {
                span: compat_span(expr_stmt.span(self.ast())),
                expr: self.compat_expr(expr_stmt.expr(self.ast())),
            }),
        }
    }

    fn compat_block_stmt(&mut self, block_stmt: experimental::BlockStmt) -> legacy::BlockStmt {
        legacy::BlockStmt {
            span: compat_span(block_stmt.span(self.ast())),
            stmts: self.compat_type_sub_range(block_stmt.stmts(self.ast()), Self::compat_stmt),
            ctxt: SyntaxContext::from_u32(self.semantic().body_scope(block_stmt, self.ast()).raw()),
        }
    }

    fn compat_empty_stmt(&mut self, empty_stmt: experimental::EmptyStmt) -> legacy::EmptyStmt {
        legacy::EmptyStmt {
            span: compat_span(empty_stmt.span(self.ast())),
        }
    }

    fn compat_expr(&mut self, expr: experimental::Expr) -> Box<legacy::Expr> {
        alloc_box!(
            self,
            match self.ast().expr_data(expr) {
                experimental::ExprData::This(t) => legacy::Expr::This(legacy::ThisExpr {
                    span: compat_span(t.span(self.ast())),
                }),
                experimental::ExprData::Array(a) => legacy::Expr::Array(legacy::ArrayLit {
                    span: compat_span(a.span(self.ast())),
                    elems: self.compat_type_sub_range(a.elems(self.ast()), |this, e| {
                        e.map(|e| this.compat_expr_or_spread(e))
                    }),
                }),
                experimental::ExprData::Object(o) =>
                    legacy::Expr::Object(self.compat_object_lit(o)),
                experimental::ExprData::Fn(f) => legacy::Expr::Fn(self.compat_fn_expr(f)),
                experimental::ExprData::Unary(u) => legacy::Expr::Unary(legacy::UnaryExpr {
                    span: compat_span(u.span(self.ast())),
                    op: match u.op(self.ast()) {
                        experimental::UnaryOp::Minus => legacy::UnaryOp::Minus,
                        experimental::UnaryOp::Plus => legacy::UnaryOp::Plus,
                        experimental::UnaryOp::Bang => legacy::UnaryOp::Bang,
                        experimental::UnaryOp::Tilde => legacy::UnaryOp::Tilde,
                        experimental::UnaryOp::TypeOf => legacy::UnaryOp::TypeOf,
                        experimental::UnaryOp::Void => legacy::UnaryOp::Void,
                        experimental::UnaryOp::Delete => legacy::UnaryOp::Delete,
                    },
                    arg: self.compat_expr(u.arg(self.ast())),
                }),
                experimental::ExprData::Update(u) => legacy::Expr::Update(legacy::UpdateExpr {
                    span: compat_span(u.span(self.ast())),
                    op: match u.op(self.ast()) {
                        experimental::UpdateOp::PlusPlus => legacy::UpdateOp::PlusPlus,
                        experimental::UpdateOp::MinusMinus => legacy::UpdateOp::MinusMinus,
                    },
                    prefix: u.prefix(self.ast()),
                    arg: match self.compat_assign_target(experimental::AssignTarget::Simple(
                        u.arg(self.ast()),
                    )) {
                        legacy::AssignTarget::Simple(target) => target.into(),
                        legacy::AssignTarget::Pat(_) => unreachable!(),
                    },
                }),
                experimental::ExprData::Bin(b) => legacy::Expr::Bin(legacy::BinExpr {
                    span: compat_span(b.span(self.ast())),
                    op: match b.op(self.ast()) {
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
                    left: self.compat_expr(b.left(self.ast())),
                    right: self.compat_expr(b.right(self.ast())),
                }),
                experimental::ExprData::Assign(a) => legacy::Expr::Assign(legacy::AssignExpr {
                    span: compat_span(a.span(self.ast())),
                    op: match a.op(self.ast()) {
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
                    left: self.compat_assign_target(a.left(self.ast())),
                    right: self.compat_expr(a.right(self.ast())),
                }),
                experimental::ExprData::Member(m) => legacy::Expr::Member(legacy::MemberExpr {
                    span: compat_span(m.span(self.ast())),
                    obj: self.compat_expr(m.obj(self.ast())),
                    prop: self.compat_member_prop(m.prop(self.ast())),
                }),
                experimental::ExprData::SuperProp(s) =>
                    legacy::Expr::SuperProp(legacy::SuperPropExpr {
                        span: compat_span(s.span(self.ast())),
                        obj: legacy::Super {
                            span: compat_span(s.obj(self.ast()).span(self.ast())),
                        },
                        prop: match self.ast().super_prop_data(s.prop(self.ast())) {
                            experimental::SuperPropData::Ident(i) => {
                                legacy::SuperProp::Ident(legacy::IdentName {
                                    span: compat_span(i.span(self.ast())),
                                    sym: self.compat_utf8_ref(i.sym(self.ast())),
                                })
                            }
                            experimental::SuperPropData::Computed(c) => {
                                legacy::SuperProp::Computed(legacy::ComputedPropName {
                                    span: compat_span(c.span(self.ast())),
                                    expr: self.compat_expr(c.expr(self.ast())),
                                })
                            }
                        },
                    }),
                experimental::ExprData::Cond(c) => legacy::Expr::Cond(legacy::CondExpr {
                    span: compat_span(c.span(self.ast())),
                    test: self.compat_expr(c.test(self.ast())),
                    cons: self.compat_expr(c.cons(self.ast())),
                    alt: self.compat_expr(c.alt(self.ast())),
                }),
                experimental::ExprData::Import(i) => {
                    let mut args = Vec::with_capacity(if i.options(self.ast()).is_some() {
                        2
                    } else {
                        1
                    });
                    args.push(legacy::ExprOrSpread {
                        spread: None,
                        expr: self.compat_expr(i.source(self.ast())),
                    });
                    if let Some(options) = i.options(self.ast()) {
                        args.push(legacy::ExprOrSpread {
                            spread: None,
                            expr: self.compat_expr(options),
                        });
                    }
                    legacy::Expr::Call(legacy::CallExpr {
                        span: compat_span(i.span(self.ast())),
                        ctxt: Default::default(),
                        callee: legacy::Callee::Import(legacy::Import {
                            span: compat_span(i.import_span(self.ast())),
                            phase: match i.phase(self.ast()) {
                                experimental::ImportPhase::Evaluation => {
                                    legacy::ImportPhase::Evaluation
                                }
                                experimental::ImportPhase::Source => legacy::ImportPhase::Source,
                                experimental::ImportPhase::Defer => legacy::ImportPhase::Defer,
                            },
                        }),
                        args,
                        type_args: None,
                    })
                }
                experimental::ExprData::Call(c) => legacy::Expr::Call(legacy::CallExpr {
                    span: compat_span(c.span(self.ast())),
                    ctxt: Default::default(),
                    callee: match self.ast().callee_data(c.callee(self.ast())) {
                        experimental::CalleeData::Super(s) =>
                            legacy::Callee::Super(legacy::Super {
                                span: compat_span(s.span(self.ast())),
                            }),
                        experimental::CalleeData::Expr(e) => {
                            legacy::Callee::Expr(self.compat_expr(e))
                        }
                    },
                    args: self
                        .compat_type_sub_range(c.args(self.ast()), Self::compat_expr_or_spread),
                    type_args: None,
                }),
                experimental::ExprData::New(n) =>
                    legacy::Expr::New(legacy::NewExpr {
                        span: compat_span(n.span(self.ast())),
                        ctxt: Default::default(),
                        callee: self.compat_expr(n.callee(self.ast())),
                        args: Some(self.compat_type_sub_range(
                            n.args(self.ast()),
                            Self::compat_expr_or_spread,
                        )),
                        type_args: None,
                    }),
                experimental::ExprData::Seq(s) => legacy::Expr::Seq(legacy::SeqExpr {
                    span: compat_span(s.span(self.ast())),
                    exprs: self.compat_type_sub_range(s.exprs(self.ast()), Self::compat_expr),
                }),
                experimental::ExprData::Ident(i) => legacy::Expr::Ident(self.compat_ident(i)),
                experimental::ExprData::Lit(l) => legacy::Expr::Lit(self.compat_lit(l)),
                experimental::ExprData::Tpl(t) => legacy::Expr::Tpl(legacy::Tpl {
                    span: compat_span(t.span(self.ast())),
                    exprs: self.compat_type_sub_range(t.exprs(self.ast()), Self::compat_expr),
                    quasis: self
                        .compat_type_sub_range(t.quasis(self.ast()), Self::compat_tpl_element),
                }),
                experimental::ExprData::TaggedTpl(tt) =>
                    legacy::Expr::TaggedTpl(legacy::TaggedTpl {
                        span: compat_span(tt.span(self.ast())),
                        ctxt: Default::default(),
                        tag: self.compat_expr(tt.tag(self.ast())),
                        tpl: alloc_box!(self, self.compat_tpl(tt.tpl(self.ast()))),
                        type_params: None,
                    }),
                experimental::ExprData::Arrow(a) => legacy::Expr::Arrow(legacy::ArrowExpr {
                    span: compat_span(a.span(self.ast())),
                    ctxt: Default::default(),
                    params: self.compat_type_sub_range(a.params(self.ast()), Self::compat_pat),
                    body: alloc_box!(
                        self,
                        match self.ast().block_stmt_or_expr_data(a.body(self.ast())) {
                            experimental::BlockStmtOrExprData::BlockStmt(b) => {
                                legacy::BlockStmtOrExpr::BlockStmt(self.compat_block_stmt(b))
                            }
                            experimental::BlockStmtOrExprData::Expr(e) => {
                                legacy::BlockStmtOrExpr::Expr(self.compat_expr(e))
                            }
                        }
                    ),
                    is_async: a.is_async(self.ast()),
                    is_generator: false,
                    type_params: None,
                    return_type: None,
                }),
                experimental::ExprData::Class(c) => legacy::Expr::Class(self.compat_class_expr(c)),
                experimental::ExprData::Yield(y) => legacy::Expr::Yield(legacy::YieldExpr {
                    span: compat_span(y.span(self.ast())),
                    arg: y.arg(self.ast()).map(|e| self.compat_expr(e)),
                    delegate: y.delegate(self.ast()),
                }),
                experimental::ExprData::MetaProp(m) =>
                    legacy::Expr::MetaProp(legacy::MetaPropExpr {
                        span: compat_span(m.span(self.ast())),
                        kind: match m.kind(self.ast()) {
                            experimental::MetaPropKind::NewTarget =>
                                legacy::MetaPropKind::NewTarget,
                            experimental::MetaPropKind::ImportMeta =>
                                legacy::MetaPropKind::ImportMeta,
                        },
                    }),
                experimental::ExprData::Await(a) => legacy::Expr::Await(legacy::AwaitExpr {
                    span: compat_span(a.span(self.ast())),
                    arg: self.compat_expr(a.arg(self.ast())),
                }),
                experimental::ExprData::Paren(p) => legacy::Expr::Paren(legacy::ParenExpr {
                    span: compat_span(p.span(self.ast())),
                    expr: self.compat_expr(p.expr(self.ast())),
                }),
                experimental::ExprData::PrivateName(p) =>
                    legacy::Expr::PrivateName(legacy::PrivateName {
                        span: compat_span(p.span(self.ast())),
                        name: self.compat_utf8_ref(p.name(self.ast())),
                    }),
                experimental::ExprData::OptChain(o) =>
                    legacy::Expr::OptChain(legacy::OptChainExpr {
                        span: compat_span(o.span(self.ast())),
                        optional: o.optional(self.ast()),
                        base: alloc_box!(
                            self,
                            match self.ast().opt_chain_base_data(o.base(self.ast())) {
                                experimental::OptChainBaseData::Member(m) => {
                                    legacy::OptChainBase::Member(legacy::MemberExpr {
                                        span: compat_span(m.span(self.ast())),
                                        obj: self.compat_expr(m.obj(self.ast())),
                                        prop: self.compat_member_prop(m.prop(self.ast())),
                                    })
                                }
                                experimental::OptChainBaseData::Call(c) => {
                                    legacy::OptChainBase::Call(legacy::OptCall {
                                        span: compat_span(c.span(self.ast())),
                                        ctxt: Default::default(),
                                        callee: self.compat_expr(c.callee(self.ast())),
                                        args: self.compat_type_sub_range(
                                            c.args(self.ast()),
                                            Self::compat_expr_or_spread,
                                        ),
                                        type_args: None,
                                    })
                                }
                            }
                        ),
                    }),
                experimental::ExprData::JSXMember(j) => {
                    legacy::Expr::JSXMember(self.compat_jsx_member_expr(j))
                }
                experimental::ExprData::JSXNamespacedName(j) => {
                    legacy::Expr::JSXNamespacedName(self.compat_jsx_namespaced_name(j))
                }
                experimental::ExprData::JSXEmpty(j) => {
                    legacy::Expr::JSXEmpty(self.compat_jsx_empty_expr(j))
                }
                experimental::ExprData::JSXElement(j) => {
                    legacy::Expr::JSXElement(alloc_box!(self, self.compat_jsx_element(j)))
                }
                experimental::ExprData::JSXFragment(j) => {
                    legacy::Expr::JSXFragment(self.compat_jsx_fragment(j))
                }
                experimental::ExprData::Invalid(i) => legacy::Expr::Invalid(legacy::Invalid {
                    span: compat_span(i.span(self.ast())),
                }),
            }
        )
    }

    fn compat_ident(&mut self, ident: experimental::Ident) -> legacy::Ident {
        legacy::Ident {
            span: compat_span(ident.span(self.ast())),
            ctxt: SyntaxContext::from_u32(self.semantic().node_scope(ident, self.ast()).raw()),
            sym: self.compat_utf8_ref(ident.sym(self.ast())),
            optional: false,
        }
    }

    fn compat_switch_case(&mut self, c: experimental::SwitchCase) -> legacy::SwitchCase {
        legacy::SwitchCase {
            span: compat_span(c.span(self.ast())),
            test: c.test(self.ast()).map(|e| self.compat_expr(e)),
            cons: self.compat_type_sub_range(c.cons(self.ast()), Self::compat_stmt),
        }
    }

    fn compat_catch_clause(&mut self, c: experimental::CatchClause) -> legacy::CatchClause {
        legacy::CatchClause {
            span: compat_span(c.span(self.ast())),
            param: c.param(self.ast()).map(|p| self.compat_pat(p)),
            body: self.compat_block_stmt(c.body(self.ast())),
        }
    }

    fn compat_var_decl_or_expr(&mut self, v: experimental::VarDeclOrExpr) -> legacy::VarDeclOrExpr {
        match self.ast().var_decl_or_expr_data(v) {
            experimental::VarDeclOrExprData::VarDecl(d) => {
                legacy::VarDeclOrExpr::VarDecl(alloc_box!(self, self.compat_var_decl(d)))
            }
            experimental::VarDeclOrExprData::Expr(e) => {
                legacy::VarDeclOrExpr::Expr(self.compat_expr(e))
            }
        }
    }

    fn compat_var_decl(&mut self, v: experimental::VarDecl) -> legacy::VarDecl {
        legacy::VarDecl {
            span: compat_span(v.span(self.ast())),
            ctxt: Default::default(),
            kind: match v.kind(self.ast()) {
                experimental::VarDeclKind::Var => legacy::VarDeclKind::Var,
                experimental::VarDeclKind::Let => legacy::VarDeclKind::Let,
                experimental::VarDeclKind::Const => legacy::VarDeclKind::Const,
            },
            declare: v.declare(self.ast()),
            decls: self.compat_type_sub_range(v.decls(self.ast()), Self::compat_var_declarator),
        }
    }

    fn compat_for_head(&mut self, h: experimental::ForHead) -> legacy::ForHead {
        match self.ast().for_head_data(h) {
            experimental::ForHeadData::VarDecl(v) => {
                legacy::ForHead::VarDecl(alloc_box!(self, self.compat_var_decl(v)))
            }
            experimental::ForHeadData::UsingDecl(u) => legacy::ForHead::UsingDecl(alloc_box!(
                self,
                legacy::UsingDecl {
                    span: compat_span(u.span(self.ast())),
                    is_await: u.is_await(self.ast()),
                    decls: self
                        .compat_type_sub_range(u.decls(self.ast()), Self::compat_var_declarator),
                }
            )),
            experimental::ForHeadData::Pat(p) => {
                legacy::ForHead::Pat(alloc_box!(self, self.compat_pat(p)))
            }
        }
    }

    // -------------------------------------------------------------------------------
    // Helpers for module declarations and common nodes

    fn compat_import_specifier(
        &mut self,

        s: experimental::ImportSpecifier,
    ) -> legacy::ImportSpecifier {
        match self.ast().import_specifier_data(s) {
            experimental::ImportSpecifierData::Named(n) => {
                legacy::ImportSpecifier::Named(legacy::ImportNamedSpecifier {
                    span: compat_span(n.span(self.ast())),
                    local: self.compat_ident(n.local(self.ast())),
                    imported: if n.is_shorthand(self.ast()) {
                        None
                    } else {
                        Some(self.compat_module_export_name(n.imported(self.ast())))
                    },
                    is_type_only: n.is_type_only(self.ast()),
                })
            }
            experimental::ImportSpecifierData::Default(d) => {
                legacy::ImportSpecifier::Default(legacy::ImportDefaultSpecifier {
                    span: compat_span(d.span(self.ast())),
                    local: self.compat_ident(d.local(self.ast())),
                })
            }
            experimental::ImportSpecifierData::Namespace(ns) => {
                legacy::ImportSpecifier::Namespace(legacy::ImportStarAsSpecifier {
                    span: compat_span(ns.span(self.ast())),
                    local: self.compat_ident(ns.local(self.ast())),
                })
            }
        }
    }

    fn compat_export_specifier(
        &mut self,

        s: experimental::ExportSpecifier,
    ) -> legacy::ExportSpecifier {
        match self.ast().export_specifier_data(s) {
            experimental::ExportSpecifierData::Namespace(ns) => {
                legacy::ExportSpecifier::Namespace(legacy::ExportNamespaceSpecifier {
                    span: compat_span(ns.span(self.ast())),
                    name: self.compat_module_export_name(ns.name(self.ast())),
                })
            }
            experimental::ExportSpecifierData::Default(d) => {
                legacy::ExportSpecifier::Default(legacy::ExportDefaultSpecifier {
                    exported: self.compat_ident(d.exported(self.ast())),
                })
            }
            experimental::ExportSpecifierData::Named(n) => {
                legacy::ExportSpecifier::Named(legacy::ExportNamedSpecifier {
                    span: compat_span(n.span(self.ast())),
                    orig: self.compat_module_export_name(n.orig(self.ast())),
                    exported: if n.is_shorthand(self.ast()) {
                        None
                    } else {
                        Some(self.compat_module_export_name(n.exported(self.ast())))
                    },
                    is_type_only: n.is_type_only(self.ast()),
                })
            }
        }
    }

    fn compat_module_export_name(
        &mut self,

        n: experimental::ModuleExportName,
    ) -> legacy::ModuleExportName {
        match self.ast().module_export_name_data(n) {
            experimental::ModuleExportNameData::Ident(i) => {
                legacy::ModuleExportName::Ident(self.compat_ident(i))
            }
            experimental::ModuleExportNameData::Str(s) => {
                legacy::ModuleExportName::Str(self.compat_str(s))
            }
        }
    }

    fn compat_object_lit(&mut self, o: experimental::ObjectLit) -> legacy::ObjectLit {
        legacy::ObjectLit {
            span: compat_span(o.span(self.ast())),
            props: self.compat_type_sub_range(o.props(self.ast()), Self::compat_prop_or_spread),
        }
    }

    fn compat_prop_or_spread(&mut self, p: experimental::PropOrSpread) -> legacy::PropOrSpread {
        match self.ast().prop_or_spread_data(p) {
            experimental::PropOrSpreadData::Spread(s) => {
                legacy::PropOrSpread::Spread(legacy::SpreadElement {
                    dot3_token: compat_span(s.dot_3_token(self.ast())),
                    expr: self.compat_expr(s.expr(self.ast())),
                })
            }
            experimental::PropOrSpreadData::Prop(prop) => {
                legacy::PropOrSpread::Prop(alloc_box!(self, self.compat_prop(prop)))
            }
        }
    }

    fn compat_prop(&mut self, p: experimental::Prop) -> legacy::Prop {
        match self.ast().prop_data(p) {
            experimental::PropData::Shorthand(i) => legacy::Prop::Shorthand(self.compat_ident(i)),
            experimental::PropData::KeyValue(kv) => legacy::Prop::KeyValue(legacy::KeyValueProp {
                key: self.compat_prop_name(kv.key(self.ast())),
                value: self.compat_expr(kv.value(self.ast())),
            }),
            experimental::PropData::Assign(ap) => legacy::Prop::Assign(legacy::AssignProp {
                span: compat_span(ap.span(self.ast())),
                key: self.compat_ident(ap.key(self.ast())),
                value: self.compat_expr(ap.value(self.ast())),
            }),
            experimental::PropData::Getter(g) => legacy::Prop::Getter(legacy::GetterProp {
                span: compat_span(g.span(self.ast())),
                key: self.compat_prop_name(g.key(self.ast())),
                type_ann: None,
                body: Some(self.compat_block_stmt(g.function(self.ast()).body(self.ast()))),
            }),
            experimental::PropData::Setter(s) => {
                let function = s.function(self.ast());
                let function_span = function.span(self.ast());
                let mut params =
                    self.compat_type_sub_range(function.params(self.ast()), Self::compat_param);
                let this_param = if params.len() >= 2 {
                    Some(params.remove(0).pat)
                } else {
                    None
                };
                let param = params
                    .into_iter()
                    .next()
                    .map(|p| alloc_box!(self, p.pat))
                    .unwrap_or_else(|| {
                        alloc_box!(
                            self,
                            legacy::Pat::Invalid(legacy::Invalid {
                                span: compat_span(function_span),
                            })
                        )
                    });
                legacy::Prop::Setter(legacy::SetterProp {
                    span: compat_span(s.span(self.ast())),
                    key: self.compat_prop_name(s.key(self.ast())),
                    this_param,
                    param,
                    body: Some(self.compat_block_stmt(function.body(self.ast()))),
                })
            }
            experimental::PropData::Method(m) => legacy::Prop::Method(legacy::MethodProp {
                key: self.compat_prop_name(m.key(self.ast())),
                function: alloc_box!(self, self.compat_function(m.function(self.ast()))),
            }),
        }
    }

    fn compat_prop_name(&mut self, n: experimental::PropName) -> legacy::PropName {
        match self.ast().prop_name_data(n) {
            experimental::PropNameData::Ident(i) => legacy::PropName::Ident(legacy::IdentName {
                span: compat_span(i.span(self.ast())),
                sym: self.compat_utf8_ref(i.sym(self.ast())),
            }),
            experimental::PropNameData::Str(s) => legacy::PropName::Str(self.compat_str(s)),
            experimental::PropNameData::Num(n) => legacy::PropName::Num(legacy::Number {
                span: compat_span(n.span(self.ast())),
                value: n.value(self.ast()),
                raw: self.compat_opt_utf8_ref(n.raw(self.ast())),
            }),
            experimental::PropNameData::Computed(c) => {
                legacy::PropName::Computed(legacy::ComputedPropName {
                    span: compat_span(c.span(self.ast())),
                    expr: self.compat_expr(c.expr(self.ast())),
                })
            }
            experimental::PropNameData::BigInt(b) => legacy::PropName::BigInt(legacy::BigInt {
                span: compat_span(b.span(self.ast())),
                value: alloc_box!(self, self.compat_big_int(b.value(self.ast()))),
                raw: self.compat_opt_utf8_ref(b.raw(self.ast())),
            }),
        }
    }

    fn compat_str(&mut self, s: experimental::Str) -> legacy::Str {
        legacy::Str {
            span: compat_span(s.span(self.ast())),
            value: self.compat_wtf8_ref(s.value(self.ast())),
            raw: self.compat_opt_utf8_ref(s.raw(self.ast())),
        }
    }

    // -------------------------------------------------------------------------------
    // Function / Class basics used by export default

    fn compat_fn_expr(&mut self, f: experimental::FnExpr) -> legacy::FnExpr {
        legacy::FnExpr {
            ident: f.ident(self.ast()).map(|i| self.compat_ident(i)),
            function: alloc_box!(self, self.compat_function(f.function(self.ast()))),
        }
    }

    fn compat_class_expr(&mut self, c: experimental::ClassExpr) -> legacy::ClassExpr {
        legacy::ClassExpr {
            ident: c.ident(self.ast()).map(|i| self.compat_ident(i)),
            class: alloc_box!(self, self.compat_class(c.class(self.ast()))),
        }
    }

    fn compat_function(&mut self, f: experimental::Function) -> legacy::Function {
        legacy::Function {
            params: self.compat_type_sub_range(f.params(self.ast()), Self::compat_param),
            decorators: self
                .compat_type_sub_range(f.decorators(self.ast()), Self::compat_decorator),
            span: compat_span(f.span(self.ast())),
            ctxt: Default::default(),
            body: Some(self.compat_block_stmt(f.body(self.ast()))),
            is_generator: f.is_generator(self.ast()),
            is_async: f.is_async(self.ast()),
            type_params: None,
            return_type: None,
        }
    }

    fn compat_param(&mut self, p: experimental::Param) -> legacy::Param {
        legacy::Param {
            span: compat_span(p.span(self.ast())),
            decorators: self
                .compat_type_sub_range(p.decorators(self.ast()), Self::compat_decorator),
            pat: self.compat_pat(p.pat(self.ast())),
        }
    }

    fn compat_decorator(&mut self, d: experimental::Decorator) -> legacy::Decorator {
        legacy::Decorator {
            span: compat_span(d.span(self.ast())),
            expr: self.compat_expr(d.expr(self.ast())),
        }
    }

    fn compat_class(&mut self, c: experimental::Class) -> legacy::Class {
        legacy::Class {
            span: compat_span(c.span(self.ast())),
            ctxt: Default::default(),
            decorators: self
                .compat_type_sub_range(c.decorators(self.ast()), Self::compat_decorator),
            body: self.compat_type_sub_range(c.body(self.ast()), Self::compat_class_member),
            super_class: c.super_class(self.ast()).map(|e| self.compat_expr(e)),
            is_abstract: c.is_abstract(self.ast()),
            type_params: None,
            super_type_params: None,
            implements: Default::default(),
        }
    }

    fn compat_class_member(&mut self, m: experimental::ClassMember) -> legacy::ClassMember {
        match self.ast().class_member_data(m) {
            experimental::ClassMemberData::Constructor(k) => {
                legacy::ClassMember::Constructor(legacy::Constructor {
                    span: compat_span(k.span(self.ast())),
                    ctxt: Default::default(),
                    key: self.compat_prop_name(k.key(self.ast())),
                    params: self.compat_type_sub_range(
                        k.params(self.ast()),
                        Self::compat_param_or_ts_param_prop,
                    ),
                    body: k.body(self.ast()).map(|b| self.compat_block_stmt(b)),
                    accessibility: None,
                    is_optional: false,
                })
            }
            experimental::ClassMemberData::Method(me) => {
                legacy::ClassMember::Method(legacy::ClassMethod {
                    span: compat_span(me.span(self.ast())),
                    key: self.compat_prop_name(me.key(self.ast())),
                    function: alloc_box!(self, self.compat_function(me.function(self.ast()))),
                    kind: match me.kind(self.ast()) {
                        experimental::MethodKind::Method => legacy::MethodKind::Method,
                        experimental::MethodKind::Getter => legacy::MethodKind::Getter,
                        experimental::MethodKind::Setter => legacy::MethodKind::Setter,
                    },
                    is_static: me.is_static(self.ast()),
                    accessibility: None,
                    is_abstract: false,
                    is_optional: false,
                    is_override: false,
                })
            }
            experimental::ClassMemberData::PrivateMethod(pm) => {
                legacy::ClassMember::PrivateMethod(legacy::PrivateMethod {
                    span: compat_span(pm.span(self.ast())),
                    key: legacy::PrivateName {
                        span: compat_span(pm.key(self.ast()).span(self.ast())),
                        name: self.compat_utf8_ref(pm.key(self.ast()).name(self.ast())),
                    },
                    function: alloc_box!(self, self.compat_function(pm.function(self.ast()))),
                    kind: match pm.kind(self.ast()) {
                        experimental::MethodKind::Method => legacy::MethodKind::Method,
                        experimental::MethodKind::Getter => legacy::MethodKind::Getter,
                        experimental::MethodKind::Setter => legacy::MethodKind::Setter,
                    },
                    is_static: pm.is_static(self.ast()),
                    accessibility: None,
                    is_abstract: false,
                    is_optional: false,
                    is_override: false,
                })
            }
            experimental::ClassMemberData::ClassProp(cp) => {
                legacy::ClassMember::ClassProp(legacy::ClassProp {
                    span: compat_span(cp.span(self.ast())),
                    key: self.compat_prop_name(cp.key(self.ast())),
                    value: cp.value(self.ast()).map(|e| self.compat_expr(e)),
                    type_ann: None,
                    is_static: cp.is_static(self.ast()),
                    decorators: self
                        .compat_type_sub_range(cp.decorators(self.ast()), Self::compat_decorator),
                    accessibility: None,
                    is_abstract: false,
                    is_optional: false,
                    is_override: false,
                    readonly: false,
                    declare: false,
                    definite: false,
                })
            }
            experimental::ClassMemberData::PrivateProp(pp) => {
                legacy::ClassMember::PrivateProp(legacy::PrivateProp {
                    span: compat_span(pp.span(self.ast())),
                    ctxt: Default::default(),
                    key: legacy::PrivateName {
                        span: compat_span(pp.key(self.ast()).span(self.ast())),
                        name: self.compat_utf8_ref(pp.key(self.ast()).name(self.ast())),
                    },
                    value: pp.value(self.ast()).map(|e| self.compat_expr(e)),
                    type_ann: None,
                    is_static: pp.is_static(self.ast()),
                    decorators: self
                        .compat_type_sub_range(pp.decorators(self.ast()), Self::compat_decorator),
                    accessibility: None,
                    is_optional: false,
                    is_override: false,
                    readonly: false,
                    definite: false,
                })
            }
            experimental::ClassMemberData::Empty(e) => {
                legacy::ClassMember::Empty(self.compat_empty_stmt(e))
            }
            experimental::ClassMemberData::StaticBlock(sb) => {
                legacy::ClassMember::StaticBlock(legacy::StaticBlock {
                    span: compat_span(sb.span(self.ast())),
                    body: self.compat_block_stmt(sb.body(self.ast())),
                })
            }
            experimental::ClassMemberData::AutoAccessor(a) => {
                legacy::ClassMember::AutoAccessor(legacy::AutoAccessor {
                    span: compat_span(a.span(self.ast())),
                    key: match self.ast().key_data(a.key(self.ast())) {
                        experimental::KeyData::Private(p) => {
                            legacy::Key::Private(legacy::PrivateName {
                                span: compat_span(p.span(self.ast())),
                                name: self.compat_utf8_ref(p.name(self.ast())),
                            })
                        }
                        experimental::KeyData::Public(n) => {
                            legacy::Key::Public(self.compat_prop_name(n))
                        }
                    },
                    value: a.value(self.ast()).map(|e| self.compat_expr(e)),
                    type_ann: None,
                    is_static: a.is_static(self.ast()),
                    decorators: self
                        .compat_type_sub_range(a.decorators(self.ast()), Self::compat_decorator),
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
        match self.ast().param_or_ts_param_prop_data(p) {
            experimental::ParamOrTsParamPropData::Param(pp) => {
                legacy::ParamOrTsParamProp::Param(self.compat_param(pp))
            }
        }
    }

    // -------------------------------------------------------------------------------
    // Patterns and declarations

    fn compat_decl(&mut self, d: experimental::Decl) -> legacy::Decl {
        match self.ast().decl_data(d) {
            experimental::DeclData::Class(c) => legacy::Decl::Class(legacy::ClassDecl {
                ident: self.compat_ident(c.ident(self.ast())),
                declare: c.declare(self.ast()),
                class: alloc_box!(self, self.compat_class(c.class(self.ast()))),
            }),
            experimental::DeclData::Fn(f) => legacy::Decl::Fn(legacy::FnDecl {
                ident: self.compat_ident(f.ident(self.ast())),
                declare: f.declare(self.ast()),
                function: alloc_box!(self, self.compat_function(f.function(self.ast()))),
            }),
            experimental::DeclData::Var(v) => {
                legacy::Decl::Var(alloc_box!(self, self.compat_var_decl(v)))
            }
            experimental::DeclData::Using(u) => legacy::Decl::Using(alloc_box!(
                self,
                legacy::UsingDecl {
                    span: compat_span(u.span(self.ast())),
                    is_await: u.is_await(self.ast()),
                    decls: self
                        .compat_type_sub_range(u.decls(self.ast()), Self::compat_var_declarator),
                }
            )),
        }
    }

    fn compat_var_declarator(&mut self, d: experimental::VarDeclarator) -> legacy::VarDeclarator {
        legacy::VarDeclarator {
            span: compat_span(d.span(self.ast())),
            name: self.compat_pat(d.name(self.ast())),
            init: d.init(self.ast()).map(|e| self.compat_expr(e)),
            definite: false,
        }
    }

    fn compat_pat(&mut self, p: experimental::Pat) -> legacy::Pat {
        match self.ast().pat_data(p) {
            experimental::PatData::Ident(b) => legacy::Pat::Ident(legacy::BindingIdent {
                id: self.compat_ident(b.id(self.ast())),
                type_ann: None,
            }),
            experimental::PatData::Array(a) => legacy::Pat::Array(legacy::ArrayPat {
                span: compat_span(a.span(self.ast())),
                elems: self.compat_type_sub_range(a.elems(self.ast()), |this, p| {
                    p.map(|p| this.compat_pat(p))
                }),
                optional: a.optional(self.ast()),
                type_ann: None,
            }),
            experimental::PatData::Rest(r) => legacy::Pat::Rest(legacy::RestPat {
                span: compat_span(r.span(self.ast())),
                dot3_token: compat_span(r.dot_3_token(self.ast())),
                arg: alloc_box!(self, self.compat_pat(r.arg(self.ast()))),
                type_ann: None,
            }),
            experimental::PatData::Object(o) => legacy::Pat::Object(legacy::ObjectPat {
                span: compat_span(o.span(self.ast())),
                props: self
                    .compat_type_sub_range(o.props(self.ast()), Self::compat_object_pat_prop),
                optional: o.optional(self.ast()),
                type_ann: None,
            }),
            experimental::PatData::Assign(a) => legacy::Pat::Assign(legacy::AssignPat {
                span: compat_span(a.span(self.ast())),
                left: alloc_box!(self, self.compat_pat(a.left(self.ast()))),
                right: self.compat_expr(a.right(self.ast())),
            }),
            experimental::PatData::Invalid(i) => legacy::Pat::Invalid(legacy::Invalid {
                span: compat_span(i.span(self.ast())),
            }),
            experimental::PatData::Expr(p) => legacy::Pat::Expr(self.compat_expr(p)),
        }
    }

    fn compat_object_pat_prop(&mut self, p: experimental::ObjectPatProp) -> legacy::ObjectPatProp {
        match self.ast().object_pat_prop_data(p) {
            experimental::ObjectPatPropData::KeyValue(kv) => {
                legacy::ObjectPatProp::KeyValue(legacy::KeyValuePatProp {
                    key: self.compat_prop_name(kv.key(self.ast())),
                    value: alloc_box!(self, self.compat_pat(kv.value(self.ast()))),
                })
            }
            experimental::ObjectPatPropData::Assign(ap) => {
                legacy::ObjectPatProp::Assign(legacy::AssignPatProp {
                    span: compat_span(ap.span(self.ast())),
                    key: legacy::BindingIdent {
                        id: self.compat_ident(ap.key(self.ast()).id(self.ast())),
                        type_ann: None,
                    },
                    value: ap.value(self.ast()).map(|e| self.compat_expr(e)),
                })
            }
            experimental::ObjectPatPropData::Rest(r) => {
                legacy::ObjectPatProp::Rest(legacy::RestPat {
                    span: compat_span(r.span(self.ast())),
                    dot3_token: compat_span(r.dot_3_token(self.ast())),
                    arg: alloc_box!(self, self.compat_pat(r.arg(self.ast()))),
                    type_ann: None,
                })
            }
        }
    }
    fn compat_expr_or_spread(&mut self, e: experimental::ExprOrSpread) -> legacy::ExprOrSpread {
        legacy::ExprOrSpread {
            spread: e.spread(self.ast()).map(compat_span),
            expr: self.compat_expr(e.expr(self.ast())),
        }
    }

    fn compat_member_prop(&mut self, p: experimental::MemberProp) -> legacy::MemberProp {
        match self.ast().member_prop_data(p) {
            experimental::MemberPropData::Ident(i) => {
                legacy::MemberProp::Ident(legacy::IdentName {
                    span: compat_span(i.span(self.ast())),
                    sym: self.compat_utf8_ref(i.sym(self.ast())),
                })
            }
            experimental::MemberPropData::PrivateName(pn) => {
                legacy::MemberProp::PrivateName(legacy::PrivateName {
                    span: compat_span(pn.span(self.ast())),
                    name: self.compat_utf8_ref(pn.name(self.ast())),
                })
            }
            experimental::MemberPropData::Computed(c) => {
                legacy::MemberProp::Computed(legacy::ComputedPropName {
                    span: compat_span(c.span(self.ast())),
                    expr: self.compat_expr(c.expr(self.ast())),
                })
            }
        }
    }

    fn compat_lit(&mut self, l: experimental::Lit) -> legacy::Lit {
        match self.ast().lit_data(l) {
            experimental::LitData::Str(s) => legacy::Lit::Str(self.compat_str(s)),
            experimental::LitData::Bool(b) => legacy::Lit::Bool(legacy::Bool {
                span: compat_span(b.span(self.ast())),
                value: b.value(self.ast()),
            }),
            experimental::LitData::Null(n) => legacy::Lit::Null(legacy::Null {
                span: compat_span(n.span(self.ast())),
            }),
            experimental::LitData::Num(n) => legacy::Lit::Num(legacy::Number {
                span: compat_span(n.span(self.ast())),
                value: n.value(self.ast()),
                raw: self.compat_opt_utf8_ref(n.raw(self.ast())),
            }),
            experimental::LitData::BigInt(b) => legacy::Lit::BigInt(legacy::BigInt {
                span: compat_span(b.span(self.ast())),
                value: alloc_box!(self, self.compat_big_int(b.value(self.ast()))),
                raw: self.compat_opt_utf8_ref(b.raw(self.ast())),
            }),
            experimental::LitData::Regex(r) => legacy::Lit::Regex(legacy::Regex {
                span: compat_span(r.span(self.ast())),
                exp: self.compat_utf8_ref(r.exp(self.ast())),
                flags: self.compat_utf8_ref(r.flags(self.ast())),
            }),
        }
    }

    fn compat_tpl_element(&mut self, e: experimental::TplElement) -> legacy::TplElement {
        legacy::TplElement {
            span: compat_span(e.span(self.ast())),
            tail: e.tail(self.ast()),
            cooked: self.compat_opt_wtf8_ref(e.cooked(self.ast())),
            raw: self.compat_utf8_ref(e.raw(self.ast())),
        }
    }

    fn compat_tpl(&mut self, t: experimental::Tpl) -> legacy::Tpl {
        legacy::Tpl {
            span: compat_span(t.span(self.ast())),
            exprs: self.compat_type_sub_range(t.exprs(self.ast()), Self::compat_expr),
            quasis: self.compat_type_sub_range(t.quasis(self.ast()), Self::compat_tpl_element),
        }
    }

    fn compat_assign_target(&mut self, t: experimental::AssignTarget) -> legacy::AssignTarget {
        match self.ast().assign_target_data(t) {
            experimental::AssignTargetData::Simple(s) => {
                legacy::AssignTarget::Simple(match self.ast().simple_assign_target_data(s) {
                    experimental::SimpleAssignTargetData::Ident(b) => {
                        legacy::SimpleAssignTarget::Ident(legacy::BindingIdent {
                            id: self.compat_ident(b.id(self.ast())),
                            type_ann: None,
                        })
                    }
                    experimental::SimpleAssignTargetData::Member(m) => {
                        legacy::SimpleAssignTarget::Member(legacy::MemberExpr {
                            span: compat_span(m.span(self.ast())),
                            obj: self.compat_expr(m.obj(self.ast())),
                            prop: self.compat_member_prop(m.prop(self.ast())),
                        })
                    }
                    experimental::SimpleAssignTargetData::SuperProp(su) => {
                        legacy::SimpleAssignTarget::SuperProp(legacy::SuperPropExpr {
                            span: compat_span(su.span(self.ast())),
                            obj: legacy::Super {
                                span: compat_span(su.obj(self.ast()).span(self.ast())),
                            },
                            prop: match self.ast().super_prop_data(su.prop(self.ast())) {
                                experimental::SuperPropData::Ident(i) => {
                                    legacy::SuperProp::Ident(legacy::IdentName {
                                        span: compat_span(i.span(self.ast())),
                                        sym: self.compat_utf8_ref(i.sym(self.ast())),
                                    })
                                }
                                experimental::SuperPropData::Computed(c) => {
                                    legacy::SuperProp::Computed(legacy::ComputedPropName {
                                        span: compat_span(c.span(self.ast())),
                                        expr: self.compat_expr(c.expr(self.ast())),
                                    })
                                }
                            },
                        })
                    }
                    experimental::SimpleAssignTargetData::Paren(p) => {
                        legacy::SimpleAssignTarget::Paren(legacy::ParenExpr {
                            span: compat_span(p.span(self.ast())),
                            expr: self.compat_expr(p.expr(self.ast())),
                        })
                    }
                    experimental::SimpleAssignTargetData::OptChain(o) => {
                        legacy::SimpleAssignTarget::OptChain(legacy::OptChainExpr {
                            span: compat_span(o.span(self.ast())),
                            optional: o.optional(self.ast()),
                            base: alloc_box!(
                                self,
                                match self.ast().opt_chain_base_data(o.base(self.ast())) {
                                    experimental::OptChainBaseData::Member(m) => {
                                        legacy::OptChainBase::Member(legacy::MemberExpr {
                                            span: compat_span(m.span(self.ast())),
                                            obj: self.compat_expr(m.obj(self.ast())),
                                            prop: self.compat_member_prop(m.prop(self.ast())),
                                        })
                                    }
                                    experimental::OptChainBaseData::Call(c) => {
                                        legacy::OptChainBase::Call(legacy::OptCall {
                                            span: compat_span(c.span(self.ast())),
                                            ctxt: Default::default(),
                                            callee: self.compat_expr(c.callee(self.ast())),
                                            args: self.compat_type_sub_range(
                                                c.args(self.ast()),
                                                Self::compat_expr_or_spread,
                                            ),
                                            type_args: None,
                                        })
                                    }
                                }
                            ),
                        })
                    }
                    experimental::SimpleAssignTargetData::Invalid(i) => {
                        legacy::SimpleAssignTarget::Invalid(legacy::Invalid {
                            span: compat_span(i.span(self.ast())),
                        })
                    }
                })
            }
            experimental::AssignTargetData::Pat(p) => {
                legacy::AssignTarget::Pat(match self.ast().assign_target_pat_data(p) {
                    experimental::AssignTargetPatData::Array(a) => {
                        legacy::AssignTargetPat::Array(legacy::ArrayPat {
                            span: compat_span(a.span(self.ast())),
                            elems: self.compat_type_sub_range(a.elems(self.ast()), |this, pat| {
                                pat.map(|pat| this.compat_pat(pat))
                            }),
                            optional: false,
                            type_ann: None,
                        })
                    }
                    experimental::AssignTargetPatData::Object(o) => {
                        legacy::AssignTargetPat::Object(legacy::ObjectPat {
                            span: compat_span(o.span(self.ast())),
                            props: self.compat_type_sub_range(
                                o.props(self.ast()),
                                Self::compat_object_pat_prop,
                            ),
                            optional: false,
                            type_ann: None,
                        })
                    }
                    experimental::AssignTargetPatData::Invalid(i) => {
                        legacy::AssignTargetPat::Invalid(legacy::Invalid {
                            span: compat_span(i.span(self.ast())),
                        })
                    }
                })
            }
        }
    }

    // -------------------------------------------------------------------------------
    // JSX compatibility helpers

    fn compat_jsx_object(&mut self, o: experimental::JSXObject) -> legacy::JSXObject {
        match self.ast().jsx_object_data(o) {
            experimental::JSXObjectData::JSXMemberExpr(m) => {
                legacy::JSXObject::JSXMemberExpr(alloc_box!(self, self.compat_jsx_member_expr(m)))
            }
            experimental::JSXObjectData::Ident(i) => legacy::JSXObject::Ident(self.compat_ident(i)),
        }
    }

    fn compat_ident_name(&mut self, i: experimental::IdentName) -> legacy::IdentName {
        legacy::IdentName {
            span: compat_span(i.span(self.ast())),
            sym: self.compat_utf8_ref(i.sym(self.ast())),
        }
    }

    fn compat_jsx_member_expr(&mut self, j: experimental::JSXMemberExpr) -> legacy::JSXMemberExpr {
        legacy::JSXMemberExpr {
            span: compat_span(j.span(self.ast())),
            obj: self.compat_jsx_object(j.obj(self.ast())),
            prop: self.compat_ident_name(j.prop(self.ast())),
        }
    }

    fn compat_jsx_namespaced_name(
        &mut self,

        j: experimental::JSXNamespacedName,
    ) -> legacy::JSXNamespacedName {
        legacy::JSXNamespacedName {
            span: compat_span(j.span(self.ast())),
            ns: self.compat_ident_name(j.ns(self.ast())),
            name: self.compat_ident_name(j.name(self.ast())),
        }
    }

    fn compat_jsx_empty_expr(&mut self, j: experimental::JSXEmptyExpr) -> legacy::JSXEmptyExpr {
        legacy::JSXEmptyExpr {
            span: compat_span(j.span(self.ast())),
        }
    }

    fn compat_jsx_expr(&mut self, e: experimental::JSXExpr) -> legacy::JSXExpr {
        match self.ast().jsx_expr_data(e) {
            experimental::JSXExprData::JSXEmptyExpr(ee) => {
                legacy::JSXExpr::JSXEmptyExpr(self.compat_jsx_empty_expr(ee))
            }
            experimental::JSXExprData::Expr(ex) => legacy::JSXExpr::Expr(self.compat_expr(ex)),
        }
    }

    fn compat_jsx_expr_container(
        &mut self,

        c: experimental::JSXExprContainer,
    ) -> legacy::JSXExprContainer {
        legacy::JSXExprContainer {
            span: compat_span(c.span(self.ast())),
            expr: self.compat_jsx_expr(c.expr(self.ast())),
        }
    }

    fn compat_spread_element(&mut self, s: experimental::SpreadElement) -> legacy::SpreadElement {
        legacy::SpreadElement {
            dot3_token: compat_span(s.dot_3_token(self.ast())),
            expr: self.compat_expr(s.expr(self.ast())),
        }
    }

    fn compat_jsx_attr_name(&mut self, n: experimental::JSXAttrName) -> legacy::JSXAttrName {
        match self.ast().jsx_attr_name_data(n) {
            experimental::JSXAttrNameData::Ident(i) => {
                legacy::JSXAttrName::Ident(self.compat_ident_name(i))
            }
            experimental::JSXAttrNameData::JSXNamespacedName(nn) => {
                legacy::JSXAttrName::JSXNamespacedName(self.compat_jsx_namespaced_name(nn))
            }
        }
    }

    fn compat_jsx_attr_value(&mut self, v: experimental::JSXAttrValue) -> legacy::JSXAttrValue {
        match self.ast().jsx_attr_value_data(v) {
            experimental::JSXAttrValueData::Str(s) => legacy::JSXAttrValue::Str(self.compat_str(s)),
            experimental::JSXAttrValueData::JSXExprContainer(c) => {
                legacy::JSXAttrValue::JSXExprContainer(self.compat_jsx_expr_container(c))
            }
            experimental::JSXAttrValueData::JSXElement(e) => {
                legacy::JSXAttrValue::JSXElement(alloc_box!(self, self.compat_jsx_element(e)))
            }
            experimental::JSXAttrValueData::JSXFragment(f) => {
                legacy::JSXAttrValue::JSXFragment(self.compat_jsx_fragment(f))
            }
        }
    }

    fn compat_jsx_attr(&mut self, a: experimental::JSXAttr) -> legacy::JSXAttr {
        legacy::JSXAttr {
            span: compat_span(a.span(self.ast())),
            name: self.compat_jsx_attr_name(a.name(self.ast())),
            value: a.value(self.ast()).map(|v| self.compat_jsx_attr_value(v)),
        }
    }

    fn compat_jsx_attr_or_spread(
        &mut self,

        a: experimental::JSXAttrOrSpread,
    ) -> legacy::JSXAttrOrSpread {
        match self.ast().jsx_attr_or_spread_data(a) {
            experimental::JSXAttrOrSpreadData::JSXAttr(attr) => {
                legacy::JSXAttrOrSpread::JSXAttr(self.compat_jsx_attr(attr))
            }
            experimental::JSXAttrOrSpreadData::SpreadElement(se) => {
                legacy::JSXAttrOrSpread::SpreadElement(self.compat_spread_element(se))
            }
        }
    }

    fn compat_jsx_element_name(
        &mut self,

        n: experimental::JSXElementName,
    ) -> legacy::JSXElementName {
        match self.ast().jsx_element_name_data(n) {
            experimental::JSXElementNameData::Ident(i) => {
                legacy::JSXElementName::Ident(self.compat_ident(i))
            }
            experimental::JSXElementNameData::JSXMemberExpr(m) => {
                legacy::JSXElementName::JSXMemberExpr(self.compat_jsx_member_expr(m))
            }
            experimental::JSXElementNameData::JSXNamespacedName(nn) => {
                legacy::JSXElementName::JSXNamespacedName(self.compat_jsx_namespaced_name(nn))
            }
        }
    }

    fn compat_jsx_opening_element(
        &mut self,

        o: experimental::JSXOpeningElement,
    ) -> legacy::JSXOpeningElement {
        legacy::JSXOpeningElement {
            span: compat_span(o.span(self.ast())),
            name: self.compat_jsx_element_name(o.name(self.ast())),
            attrs: self.compat_type_sub_range(o.attrs(self.ast()), Self::compat_jsx_attr_or_spread),
            self_closing: o.self_closing(self.ast()),
            type_args: None,
        }
    }

    fn compat_jsx_closing_element(
        &mut self,

        c: experimental::JSXClosingElement,
    ) -> legacy::JSXClosingElement {
        legacy::JSXClosingElement {
            span: compat_span(c.span(self.ast())),
            name: self.compat_jsx_element_name(c.name(self.ast())),
        }
    }

    fn compat_jsx_text(&mut self, t: experimental::JSXText) -> legacy::JSXText {
        legacy::JSXText {
            span: compat_span(t.span(self.ast())),
            value: self.compat_utf8_ref(t.value(self.ast())),
            raw: self.compat_utf8_ref(t.raw(self.ast())),
        }
    }

    fn compat_jsx_spread_child(
        &mut self,

        s: experimental::JSXSpreadChild,
    ) -> legacy::JSXSpreadChild {
        legacy::JSXSpreadChild {
            span: compat_span(s.span(self.ast())),
            expr: self.compat_expr(s.expr(self.ast())),
        }
    }

    fn compat_jsx_element_child(
        &mut self,

        c: experimental::JSXElementChild,
    ) -> legacy::JSXElementChild {
        match self.ast().jsx_element_child_data(c) {
            experimental::JSXElementChildData::JSXText(t) => {
                legacy::JSXElementChild::JSXText(self.compat_jsx_text(t))
            }
            experimental::JSXElementChildData::JSXExprContainer(ec) => {
                legacy::JSXElementChild::JSXExprContainer(self.compat_jsx_expr_container(ec))
            }
            experimental::JSXElementChildData::JSXSpreadChild(sc) => {
                legacy::JSXElementChild::JSXSpreadChild(self.compat_jsx_spread_child(sc))
            }
            experimental::JSXElementChildData::JSXElement(e) => {
                legacy::JSXElementChild::JSXElement(alloc_box!(self, self.compat_jsx_element(e)))
            }
            experimental::JSXElementChildData::JSXFragment(f) => {
                legacy::JSXElementChild::JSXFragment(self.compat_jsx_fragment(f))
            }
        }
    }

    fn compat_jsx_element(&mut self, e: experimental::JSXElement) -> legacy::JSXElement {
        legacy::JSXElement {
            span: compat_span(e.span(self.ast())),
            opening: self.compat_jsx_opening_element(e.opening(self.ast())),
            children: self
                .compat_type_sub_range(e.children(self.ast()), Self::compat_jsx_element_child),
            closing: e
                .closing(self.ast())
                .map(|c| self.compat_jsx_closing_element(c)),
        }
    }

    fn compat_jsx_opening_fragment(
        &mut self,

        o: experimental::JSXOpeningFragment,
    ) -> legacy::JSXOpeningFragment {
        legacy::JSXOpeningFragment {
            span: compat_span(o.span(self.ast())),
        }
    }

    fn compat_jsx_closing_fragment(
        &mut self,

        c: experimental::JSXClosingFragment,
    ) -> legacy::JSXClosingFragment {
        legacy::JSXClosingFragment {
            span: compat_span(c.span(self.ast())),
        }
    }

    fn compat_jsx_fragment(&mut self, f: experimental::JSXFragment) -> legacy::JSXFragment {
        legacy::JSXFragment {
            span: compat_span(f.span(self.ast())),
            opening: self.compat_jsx_opening_fragment(f.opening(self.ast())),
            children: self
                .compat_type_sub_range(f.children(self.ast()), Self::compat_jsx_element_child),
            closing: self.compat_jsx_closing_fragment(f.closing(self.ast())),
        }
    }

    // ===============================================================================

    fn compat_utf8_ref(&mut self, atom: ExperimentalAtom<'ast>) -> Atom {
        Atom::from(atom.as_str())
    }

    fn compat_opt_utf8_ref(&mut self, atom: Option<ExperimentalAtom<'ast>>) -> Option<Atom> {
        atom.map(|atom| self.compat_utf8_ref(atom))
    }

    fn compat_wtf8_ref(&mut self, atom: ExperimentalWtf8Atom<'ast>) -> Wtf8Atom {
        match atom.as_wtf8().as_str() {
            Some(s) => Wtf8Atom::from(s),
            None => Wtf8Atom::from(atom.as_wtf8().to_string_lossy().into_owned()),
        }
    }

    fn compat_opt_wtf8_ref(
        &mut self,
        atom: Option<ExperimentalWtf8Atom<'ast>>,
    ) -> Option<Wtf8Atom> {
        atom.map(|atom| self.compat_wtf8_ref(atom))
    }

    fn compat_big_int(&mut self, value: ExperimentalAtom<'ast>) -> legacy::BigIntValue {
        legacy::BigIntValue::parse_bytes(value.as_bytes(), 10).unwrap()
    }
}
