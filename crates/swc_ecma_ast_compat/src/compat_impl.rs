use swc_core::atoms::{Atom, Wtf8Atom};
use swc_core::common::{BytePos, Span as SwcSpan, SyntaxContext};
use swc_core::ecma::ast::{self as legacy};
use swc_experimental_allocator::atom::{
    Atom as ExperimentalAtom, Wtf8Atom as ExperimentalWtf8Atom,
};
use swc_experimental_allocator::boxed::Box as AstBox;
use swc_experimental_allocator::vec::Vec as ArenaVec;
use swc_experimental_ecma_ast::{self as experimental};
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

pub(crate) trait CompatImpl {
    fn semantic(&self) -> &Semantic;

    fn alloc_box<T>(&self, value: T) -> Box<T>;

    fn compat_vec<T, U, F: Fn(&mut Self, T) -> U>(
        &mut self,
        items: ArenaVec<'_, T>,
        transformer: F,
    ) -> Vec<U>;

    fn compat_program_inner<'a>(&mut self, root: experimental::Program<'a>) -> legacy::Program {
        match root {
            experimental::Program::Module(module) => {
                legacy::Program::Module(self.compat_module_inner(AstBox::into_inner(module)))
            }
            experimental::Program::Script(script) => {
                legacy::Program::Script(self.compat_script_inner(AstBox::into_inner(script)))
            }
        }
    }

    fn compat_module_inner<'a>(&mut self, module: experimental::Module<'a>) -> legacy::Module {
        #[allow(unused_mut)]
        let mut inner = || legacy::Module {
            span: compat_span(module.span),
            shebang: self.compat_opt_utf8_ref(module.shebang),
            body: self.compat_vec(module.body, Self::compat_module_item),
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

    fn compat_script_inner<'a>(&mut self, script: experimental::Script<'a>) -> legacy::Script {
        #[allow(unused_mut)]
        let mut inner = || legacy::Script {
            span: compat_span(script.span),
            body: self.compat_vec(script.body, Self::compat_stmt),
            shebang: self.compat_opt_utf8_ref(script.shebang),
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

    fn compat_module_item<'a>(&mut self, item: experimental::ModuleItem<'a>) -> legacy::ModuleItem {
        match item {
            experimental::ModuleItem::ModuleDecl(module_decl) => legacy::ModuleItem::ModuleDecl(
                self.compat_module_decl(AstBox::into_inner(module_decl)),
            ),
            experimental::ModuleItem::Stmt(stmt) => {
                legacy::ModuleItem::Stmt(self.compat_stmt(AstBox::into_inner(stmt)))
            }
        }
    }

    fn compat_module_decl<'a>(
        &mut self,
        module_decl: experimental::ModuleDecl<'a>,
    ) -> legacy::ModuleDecl {
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

    fn compat_import_decl<'a>(
        &mut self,
        import_decl: AstBox<'a, experimental::ImportDecl<'a>>,
    ) -> legacy::ImportDecl {
        let import_decl = AstBox::into_inner(import_decl);
        legacy::ImportDecl {
            span: compat_span(import_decl.span),
            specifiers: self.compat_vec(import_decl.specifiers, Self::compat_import_specifier),
            src: alloc_box!(self, self.compat_str(import_decl.src)),
            type_only: import_decl.type_only,
            with: import_decl
                .with
                .map(|with| alloc_box!(self, self.compat_object_lit(with))),
            phase: match import_decl.phase {
                experimental::ImportPhase::Evaluation => legacy::ImportPhase::Evaluation,
                experimental::ImportPhase::Source => legacy::ImportPhase::Source,
                experimental::ImportPhase::Defer => legacy::ImportPhase::Defer,
            },
        }
    }

    fn compat_export_decl<'a>(
        &mut self,
        export_decl: AstBox<'a, experimental::ExportDecl<'a>>,
    ) -> legacy::ExportDecl {
        let export_decl = AstBox::into_inner(export_decl);
        legacy::ExportDecl {
            span: compat_span(export_decl.span),
            decl: self.compat_decl(export_decl.decl),
        }
    }

    fn compat_export_named<'a>(
        &mut self,

        export_named: AstBox<'a, experimental::NamedExport<'a>>,
    ) -> legacy::NamedExport {
        let export_named = AstBox::into_inner(export_named);
        legacy::NamedExport {
            span: compat_span(export_named.span),
            specifiers: self.compat_vec(export_named.specifiers, Self::compat_export_specifier),
            src: export_named
                .src
                .map(|s| alloc_box!(self, self.compat_str(s))),
            type_only: export_named.type_only,
            with: export_named
                .with
                .map(|o| alloc_box!(self, self.compat_object_lit(o))),
        }
    }

    fn compat_export_default_decl<'a>(
        &mut self,

        export_default_decl: AstBox<'a, experimental::ExportDefaultDecl<'a>>,
    ) -> legacy::ExportDefaultDecl {
        let export_default_decl = AstBox::into_inner(export_default_decl);
        legacy::ExportDefaultDecl {
            span: compat_span(export_default_decl.span),
            decl: match export_default_decl.decl {
                experimental::DefaultDecl::Class(cls) => {
                    legacy::DefaultDecl::Class(self.compat_class_expr(cls))
                }
                experimental::DefaultDecl::Fn(f) => legacy::DefaultDecl::Fn(self.compat_fn_expr(f)),
            },
        }
    }

    fn compat_export_default_expr<'a>(
        &mut self,

        export_default_expr: AstBox<'a, experimental::ExportDefaultExpr<'a>>,
    ) -> legacy::ExportDefaultExpr {
        let export_default_expr = AstBox::into_inner(export_default_expr);
        legacy::ExportDefaultExpr {
            span: compat_span(export_default_expr.span),
            expr: self.compat_expr(export_default_expr.expr),
        }
    }

    fn compat_export_all<'a>(
        &mut self,
        export_all: AstBox<'a, experimental::ExportAll<'a>>,
    ) -> legacy::ExportAll {
        let export_all = AstBox::into_inner(export_all);
        legacy::ExportAll {
            span: compat_span(export_all.span),
            src: alloc_box!(self, self.compat_str(export_all.src)),
            type_only: export_all.type_only,
            with: export_all
                .with
                .map(|o| alloc_box!(self, self.compat_object_lit(o))),
        }
    }

    fn compat_stmt<'a>(&mut self, stmt: experimental::Stmt<'a>) -> legacy::Stmt {
        match stmt {
            experimental::Stmt::Block(block_stmt) => {
                legacy::Stmt::Block(self.compat_block_stmt(block_stmt))
            }
            experimental::Stmt::Empty(empty_stmt) => {
                legacy::Stmt::Empty(self.compat_empty_stmt(empty_stmt))
            }
            experimental::Stmt::Debugger(debugger_stmt) => {
                let debugger_stmt = AstBox::into_inner(debugger_stmt);
                legacy::Stmt::Debugger(legacy::DebuggerStmt {
                    span: compat_span(debugger_stmt.span),
                })
            }
            experimental::Stmt::With(with_stmt) => {
                let with_stmt = AstBox::into_inner(with_stmt);
                legacy::Stmt::With(legacy::WithStmt {
                    span: compat_span(with_stmt.span),
                    obj: self.compat_expr(with_stmt.obj),
                    body: alloc_box!(self, self.compat_stmt(with_stmt.body)),
                })
            }
            experimental::Stmt::Return(return_stmt) => {
                let return_stmt = AstBox::into_inner(return_stmt);
                legacy::Stmt::Return(legacy::ReturnStmt {
                    span: compat_span(return_stmt.span),
                    arg: return_stmt.arg.map(|arg| self.compat_expr(arg)),
                })
            }
            experimental::Stmt::Labeled(labeled_stmt) => {
                let labeled_stmt = AstBox::into_inner(labeled_stmt);
                legacy::Stmt::Labeled(legacy::LabeledStmt {
                    span: compat_span(labeled_stmt.span),
                    label: self.compat_ident(labeled_stmt.label),
                    body: alloc_box!(self, self.compat_stmt(labeled_stmt.body)),
                })
            }
            experimental::Stmt::Break(break_stmt) => {
                let break_stmt = AstBox::into_inner(break_stmt);
                legacy::Stmt::Break(legacy::BreakStmt {
                    span: compat_span(break_stmt.span),
                    label: break_stmt.label.map(|label| self.compat_ident(label)),
                })
            }
            experimental::Stmt::Continue(continue_stmt) => {
                let continue_stmt = AstBox::into_inner(continue_stmt);
                legacy::Stmt::Continue(legacy::ContinueStmt {
                    span: compat_span(continue_stmt.span),
                    label: continue_stmt.label.map(|label| self.compat_ident(label)),
                })
            }
            experimental::Stmt::If(if_stmt) => {
                let if_stmt = AstBox::into_inner(if_stmt);
                legacy::Stmt::If(legacy::IfStmt {
                    span: compat_span(if_stmt.span),
                    test: self.compat_expr(if_stmt.test),
                    cons: alloc_box!(self, self.compat_stmt(if_stmt.cons)),
                    alt: if_stmt
                        .alt
                        .map(|alt| alloc_box!(self, self.compat_stmt(alt))),
                })
            }
            experimental::Stmt::Switch(switch_stmt) => {
                let switch_stmt = AstBox::into_inner(switch_stmt);
                legacy::Stmt::Switch(legacy::SwitchStmt {
                    span: compat_span(switch_stmt.span),
                    discriminant: self.compat_expr(switch_stmt.discriminant),
                    cases: self.compat_vec(switch_stmt.cases, Self::compat_switch_case),
                })
            }
            experimental::Stmt::Throw(throw_stmt) => {
                let throw_stmt = AstBox::into_inner(throw_stmt);
                legacy::Stmt::Throw(legacy::ThrowStmt {
                    span: compat_span(throw_stmt.span),
                    arg: self.compat_expr(throw_stmt.arg),
                })
            }
            experimental::Stmt::Try(try_stmt) => {
                let try_stmt = AstBox::into_inner(try_stmt);
                legacy::Stmt::Try(alloc_box!(
                    self,
                    legacy::TryStmt {
                        span: compat_span(try_stmt.span),
                        block: self.compat_block_stmt(try_stmt.block),
                        handler: try_stmt
                            .handler
                            .map(|handler| self.compat_catch_clause(handler)),
                        finalizer: try_stmt
                            .finalizer
                            .map(|finalizer| self.compat_block_stmt(finalizer)),
                    }
                ))
            }
            experimental::Stmt::While(while_stmt) => {
                let while_stmt = AstBox::into_inner(while_stmt);
                legacy::Stmt::While(legacy::WhileStmt {
                    span: compat_span(while_stmt.span),
                    test: self.compat_expr(while_stmt.test),
                    body: alloc_box!(self, self.compat_stmt(while_stmt.body)),
                })
            }
            experimental::Stmt::DoWhile(do_while_stmt) => {
                let do_while_stmt = AstBox::into_inner(do_while_stmt);
                legacy::Stmt::DoWhile(legacy::DoWhileStmt {
                    span: compat_span(do_while_stmt.span),
                    test: self.compat_expr(do_while_stmt.test),
                    body: alloc_box!(self, self.compat_stmt(do_while_stmt.body)),
                })
            }
            experimental::Stmt::For(for_stmt) => {
                let for_stmt = AstBox::into_inner(for_stmt);
                legacy::Stmt::For(legacy::ForStmt {
                    span: compat_span(for_stmt.span),
                    init: for_stmt.init.map(|i| self.compat_var_decl_or_expr(i)),
                    test: for_stmt.test.map(|e| self.compat_expr(e)),
                    update: for_stmt.update.map(|e| self.compat_expr(e)),
                    body: alloc_box!(self, self.compat_stmt(for_stmt.body)),
                })
            }
            experimental::Stmt::ForIn(for_in_stmt) => {
                let for_in_stmt = AstBox::into_inner(for_in_stmt);
                legacy::Stmt::ForIn(legacy::ForInStmt {
                    span: compat_span(for_in_stmt.span),
                    left: self.compat_for_head(for_in_stmt.left),
                    right: self.compat_expr(for_in_stmt.right),
                    body: alloc_box!(self, self.compat_stmt(for_in_stmt.body)),
                })
            }
            experimental::Stmt::ForOf(for_of_stmt) => {
                let for_of_stmt = AstBox::into_inner(for_of_stmt);
                legacy::Stmt::ForOf(legacy::ForOfStmt {
                    span: compat_span(for_of_stmt.span),
                    is_await: for_of_stmt.is_await,
                    left: self.compat_for_head(for_of_stmt.left),
                    right: self.compat_expr(for_of_stmt.right),
                    body: alloc_box!(self, self.compat_stmt(for_of_stmt.body)),
                })
            }
            experimental::Stmt::Decl(decl) => {
                legacy::Stmt::Decl(self.compat_decl(AstBox::into_inner(decl)))
            }
            experimental::Stmt::Expr(expr_stmt) => {
                let expr_stmt = AstBox::into_inner(expr_stmt);
                legacy::Stmt::Expr(legacy::ExprStmt {
                    span: compat_span(expr_stmt.span),
                    expr: self.compat_expr(expr_stmt.expr),
                })
            }
        }
    }

    fn compat_block_stmt<'a>(
        &mut self,
        block_stmt: AstBox<'a, experimental::BlockStmt<'a>>,
    ) -> legacy::BlockStmt {
        let block_stmt = AstBox::into_inner(block_stmt);
        let ctxt = SyntaxContext::from_u32(block_stmt.scope_id().raw());
        legacy::BlockStmt {
            span: compat_span(block_stmt.span),
            stmts: self.compat_vec(block_stmt.stmts, Self::compat_stmt),
            ctxt,
        }
    }

    fn compat_empty_stmt(
        &mut self,
        empty_stmt: AstBox<'_, experimental::EmptyStmt>,
    ) -> legacy::EmptyStmt {
        let empty_stmt = AstBox::into_inner(empty_stmt);
        legacy::EmptyStmt {
            span: compat_span(empty_stmt.span),
        }
    }

    fn compat_expr<'a>(&mut self, expr: experimental::Expr<'a>) -> Box<legacy::Expr> {
        alloc_box!(
            self,
            match expr {
                experimental::Expr::This(t) => {
                    let t = AstBox::into_inner(t);
                    legacy::Expr::This(legacy::ThisExpr {
                        span: compat_span(t.span),
                    })
                }
                experimental::Expr::Array(a) => {
                    let a = AstBox::into_inner(a);
                    legacy::Expr::Array(legacy::ArrayLit {
                        span: compat_span(a.span),
                        elems: self.compat_vec(a.elems, |this, e| {
                            e.map(|e| this.compat_expr_or_spread(AstBox::into_inner(e)))
                        }),
                    })
                }
                experimental::Expr::Object(o) => legacy::Expr::Object(self.compat_object_lit(o)),
                experimental::Expr::Fn(f) => legacy::Expr::Fn(self.compat_fn_expr(f)),
                experimental::Expr::Unary(u) => {
                    let u = AstBox::into_inner(u);
                    legacy::Expr::Unary(legacy::UnaryExpr {
                        span: compat_span(u.span),
                        op: match u.op {
                            experimental::UnaryOp::Minus => legacy::UnaryOp::Minus,
                            experimental::UnaryOp::Plus => legacy::UnaryOp::Plus,
                            experimental::UnaryOp::Bang => legacy::UnaryOp::Bang,
                            experimental::UnaryOp::Tilde => legacy::UnaryOp::Tilde,
                            experimental::UnaryOp::TypeOf => legacy::UnaryOp::TypeOf,
                            experimental::UnaryOp::Void => legacy::UnaryOp::Void,
                            experimental::UnaryOp::Delete => legacy::UnaryOp::Delete,
                        },
                        arg: self.compat_expr(u.arg),
                    })
                }
                experimental::Expr::Update(u) => {
                    let u = AstBox::into_inner(u);
                    legacy::Expr::Update(legacy::UpdateExpr {
                        span: compat_span(u.span),
                        op: match u.op {
                            experimental::UpdateOp::PlusPlus => legacy::UpdateOp::PlusPlus,
                            experimental::UpdateOp::MinusMinus => legacy::UpdateOp::MinusMinus,
                        },
                        prefix: u.prefix,
                        arg: self.compat_expr(u.arg),
                    })
                }
                experimental::Expr::Bin(b) => {
                    let b = AstBox::into_inner(b);
                    legacy::Expr::Bin(legacy::BinExpr {
                        span: compat_span(b.span),
                        op: match b.op {
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
                            experimental::BinaryOp::ZeroFillRShift => {
                                legacy::BinaryOp::ZeroFillRShift
                            }
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
                        left: self.compat_expr(b.left),
                        right: self.compat_expr(b.right),
                    })
                }
                experimental::Expr::Assign(a) => {
                    let a = AstBox::into_inner(a);
                    legacy::Expr::Assign(legacy::AssignExpr {
                        span: compat_span(a.span),
                        op: match a.op {
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
                            experimental::AssignOp::NullishAssign => {
                                legacy::AssignOp::NullishAssign
                            }
                        },
                        left: self.compat_assign_target(a.left),
                        right: self.compat_expr(a.right),
                    })
                }
                experimental::Expr::Member(m) => {
                    let m = AstBox::into_inner(m);
                    legacy::Expr::Member(legacy::MemberExpr {
                        span: compat_span(m.span),
                        obj: self.compat_expr(m.obj),
                        prop: self.compat_member_prop(m.prop),
                    })
                }
                experimental::Expr::SuperProp(s) => {
                    let s = AstBox::into_inner(s);
                    legacy::Expr::SuperProp(legacy::SuperPropExpr {
                        span: compat_span(s.span),
                        obj: legacy::Super {
                            span: compat_span(s.obj.span),
                        },
                        prop: match s.prop {
                            experimental::SuperProp::Ident(i) => {
                                let i = AstBox::into_inner(i);
                                legacy::SuperProp::Ident(legacy::IdentName {
                                    span: compat_span(i.span),
                                    sym: self.compat_utf8_ref(i.sym),
                                })
                            }
                            experimental::SuperProp::Computed(c) => {
                                let c = AstBox::into_inner(c);
                                legacy::SuperProp::Computed(legacy::ComputedPropName {
                                    span: compat_span(c.span),
                                    expr: self.compat_expr(c.expr),
                                })
                            }
                        },
                    })
                }
                experimental::Expr::Cond(c) => {
                    let c = AstBox::into_inner(c);
                    legacy::Expr::Cond(legacy::CondExpr {
                        span: compat_span(c.span),
                        test: self.compat_expr(c.test),
                        cons: self.compat_expr(c.cons),
                        alt: self.compat_expr(c.alt),
                    })
                }
                experimental::Expr::Call(c) => {
                    let c = AstBox::into_inner(c);
                    legacy::Expr::Call(legacy::CallExpr {
                        span: compat_span(c.span),
                        ctxt: Default::default(),
                        callee: match c.callee {
                            experimental::Callee::Super(s) => {
                                legacy::Callee::Super(legacy::Super {
                                    span: compat_span(s.span),
                                })
                            }
                            experimental::Callee::Import(i) => {
                                legacy::Callee::Import(legacy::Import {
                                    span: compat_span(i.span),
                                    phase: match i.phase {
                                        experimental::ImportPhase::Evaluation => {
                                            legacy::ImportPhase::Evaluation
                                        }
                                        experimental::ImportPhase::Source => {
                                            legacy::ImportPhase::Source
                                        }
                                        experimental::ImportPhase::Defer => {
                                            legacy::ImportPhase::Defer
                                        }
                                    },
                                })
                            }
                            experimental::Callee::Expr(e) => {
                                legacy::Callee::Expr(self.compat_expr(AstBox::into_inner(e)))
                            }
                        },
                        args: self.compat_vec(c.args, Self::compat_expr_or_spread),
                        type_args: None,
                    })
                }
                experimental::Expr::New(n) => {
                    let n = AstBox::into_inner(n);
                    legacy::Expr::New(legacy::NewExpr {
                        span: compat_span(n.span),
                        ctxt: Default::default(),
                        callee: self.compat_expr(n.callee),
                        args: n
                            .args
                            .map(|args| self.compat_vec(args, Self::compat_expr_or_spread)),
                        type_args: None,
                    })
                }
                experimental::Expr::Seq(s) => {
                    let s = AstBox::into_inner(s);
                    legacy::Expr::Seq(legacy::SeqExpr {
                        span: compat_span(s.span),
                        exprs: self.compat_vec(s.exprs, Self::compat_expr),
                    })
                }
                experimental::Expr::Ident(i) => legacy::Expr::Ident(self.compat_ident(i)),
                experimental::Expr::Lit(l) => legacy::Expr::Lit(self.compat_lit(l)),
                experimental::Expr::Tpl(t) => {
                    let t = AstBox::into_inner(t);
                    legacy::Expr::Tpl(legacy::Tpl {
                        span: compat_span(t.span),
                        exprs: self.compat_vec(t.exprs, Self::compat_expr),
                        quasis: self.compat_vec(t.quasis, Self::compat_tpl_element),
                    })
                }
                experimental::Expr::TaggedTpl(tt) => {
                    let tt = AstBox::into_inner(tt);
                    legacy::Expr::TaggedTpl(legacy::TaggedTpl {
                        span: compat_span(tt.span),
                        ctxt: Default::default(),
                        tag: self.compat_expr(tt.tag),
                        tpl: alloc_box!(self, self.compat_tpl(AstBox::into_inner(tt.tpl))),
                        type_params: None,
                    })
                }
                experimental::Expr::Arrow(a) => {
                    let a = AstBox::into_inner(a);
                    legacy::Expr::Arrow(legacy::ArrowExpr {
                        span: compat_span(a.span),
                        ctxt: Default::default(),
                        params: self.compat_vec(a.params, Self::compat_pat),
                        body: alloc_box!(
                            self,
                            match a.body {
                                experimental::BlockStmtOrExpr::BlockStmt(b) => {
                                    legacy::BlockStmtOrExpr::BlockStmt(self.compat_block_stmt(b))
                                }
                                experimental::BlockStmtOrExpr::Expr(e) => {
                                    legacy::BlockStmtOrExpr::Expr(
                                        self.compat_expr(AstBox::into_inner(e)),
                                    )
                                }
                            }
                        ),
                        is_async: a.is_async,
                        is_generator: false,
                        type_params: None,
                        return_type: None,
                    })
                }
                experimental::Expr::Class(c) => legacy::Expr::Class(self.compat_class_expr(c)),
                experimental::Expr::Yield(y) => {
                    let y = AstBox::into_inner(y);
                    legacy::Expr::Yield(legacy::YieldExpr {
                        span: compat_span(y.span),
                        arg: y.arg.map(|e| self.compat_expr(e)),
                        delegate: y.delegate,
                    })
                }
                experimental::Expr::MetaProp(m) => {
                    let m = AstBox::into_inner(m);
                    legacy::Expr::MetaProp(legacy::MetaPropExpr {
                        span: compat_span(m.span),
                        kind: match m.kind {
                            experimental::MetaPropKind::NewTarget => {
                                legacy::MetaPropKind::NewTarget
                            }
                            experimental::MetaPropKind::ImportMeta => {
                                legacy::MetaPropKind::ImportMeta
                            }
                        },
                    })
                }
                experimental::Expr::Await(a) => {
                    let a = AstBox::into_inner(a);
                    legacy::Expr::Await(legacy::AwaitExpr {
                        span: compat_span(a.span),
                        arg: self.compat_expr(a.arg),
                    })
                }
                experimental::Expr::Paren(p) => {
                    let p = AstBox::into_inner(p);
                    legacy::Expr::Paren(legacy::ParenExpr {
                        span: compat_span(p.span),
                        expr: self.compat_expr(p.expr),
                    })
                }
                experimental::Expr::PrivateName(p) => {
                    let p = AstBox::into_inner(p);
                    legacy::Expr::PrivateName(legacy::PrivateName {
                        span: compat_span(p.span),
                        name: self.compat_utf8_ref(p.name),
                    })
                }
                experimental::Expr::OptChain(o) => {
                    let o = AstBox::into_inner(o);
                    legacy::Expr::OptChain(legacy::OptChainExpr {
                        span: compat_span(o.span),
                        optional: o.optional,
                        base: alloc_box!(
                            self,
                            match o.base {
                                experimental::OptChainBase::Member(m) => {
                                    let m = AstBox::into_inner(m);
                                    legacy::OptChainBase::Member(legacy::MemberExpr {
                                        span: compat_span(m.span),
                                        obj: self.compat_expr(m.obj),
                                        prop: self.compat_member_prop(m.prop),
                                    })
                                }
                                experimental::OptChainBase::Call(c) => {
                                    let c = AstBox::into_inner(c);
                                    legacy::OptChainBase::Call(legacy::OptCall {
                                        span: compat_span(c.span),
                                        ctxt: Default::default(),
                                        callee: self.compat_expr(c.callee),
                                        args: self.compat_vec(c.args, Self::compat_expr_or_spread),
                                        type_args: None,
                                    })
                                }
                            }
                        ),
                    })
                }
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
                    legacy::Expr::JSXElement(alloc_box!(self, self.compat_jsx_element(j)))
                }
                experimental::Expr::JSXFragment(j) => {
                    legacy::Expr::JSXFragment(self.compat_jsx_fragment(j))
                }
                experimental::Expr::Invalid(_) => {
                    legacy::Expr::Invalid(legacy::Invalid {
                        span: Default::default(),
                    })
                }
            }
        )
    }

    fn compat_ident(&mut self, ident: AstBox<'_, experimental::Ident>) -> legacy::Ident {
        let ident = AstBox::into_inner(ident);
        legacy::Ident {
            span: compat_span(ident.span),
            ctxt: SyntaxContext::from_u32(self.semantic().node_scope(&ident).raw()),
            sym: self.compat_utf8_ref(ident.sym),
            optional: false,
        }
    }

    fn compat_switch_case<'a>(&mut self, c: experimental::SwitchCase<'a>) -> legacy::SwitchCase {
        legacy::SwitchCase {
            span: compat_span(c.span),
            test: c.test.map(|e| self.compat_expr(e)),
            cons: self.compat_vec(c.cons, Self::compat_stmt),
        }
    }

    fn compat_catch_clause<'a>(
        &mut self,
        c: AstBox<'a, experimental::CatchClause<'a>>,
    ) -> legacy::CatchClause {
        let c = AstBox::into_inner(c);
        legacy::CatchClause {
            span: compat_span(c.span),
            param: c.param.map(|p| self.compat_pat(p)),
            body: self.compat_block_stmt(c.body),
        }
    }

    fn compat_var_decl_or_expr<'a>(
        &mut self,
        v: experimental::VarDeclOrExpr<'a>,
    ) -> legacy::VarDeclOrExpr {
        match v {
            experimental::VarDeclOrExpr::VarDecl(d) => {
                legacy::VarDeclOrExpr::VarDecl(alloc_box!(self, self.compat_var_decl(d)))
            }
            experimental::VarDeclOrExpr::Expr(e) => {
                legacy::VarDeclOrExpr::Expr(self.compat_expr(AstBox::into_inner(e)))
            }
        }
    }

    fn compat_var_decl<'a>(&mut self, v: AstBox<'a, experimental::VarDecl<'a>>) -> legacy::VarDecl {
        let v = AstBox::into_inner(v);
        legacy::VarDecl {
            span: compat_span(v.span),
            ctxt: Default::default(),
            kind: match v.kind {
                experimental::VarDeclKind::Var => legacy::VarDeclKind::Var,
                experimental::VarDeclKind::Let => legacy::VarDeclKind::Let,
                experimental::VarDeclKind::Const => legacy::VarDeclKind::Const,
            },
            declare: v.declare,
            decls: self.compat_vec(v.decls, Self::compat_var_declarator),
        }
    }

    fn compat_for_head<'a>(&mut self, h: experimental::ForHead<'a>) -> legacy::ForHead {
        match h {
            experimental::ForHead::VarDecl(v) => {
                legacy::ForHead::VarDecl(alloc_box!(self, self.compat_var_decl(v)))
            }
            experimental::ForHead::UsingDecl(u) => {
                let u = AstBox::into_inner(u);
                legacy::ForHead::UsingDecl(alloc_box!(
                    self,
                    legacy::UsingDecl {
                        span: compat_span(u.span),
                        is_await: u.is_await,
                        decls: self.compat_vec(u.decls, Self::compat_var_declarator),
                    }
                ))
            }
            experimental::ForHead::Pat(p) => {
                legacy::ForHead::Pat(alloc_box!(self, self.compat_pat(AstBox::into_inner(p))))
            }
        }
    }

    // -------------------------------------------------------------------------------
    // Helpers for module declarations and common nodes

    fn compat_import_specifier<'a>(
        &mut self,

        s: experimental::ImportSpecifier<'a>,
    ) -> legacy::ImportSpecifier {
        match s {
            experimental::ImportSpecifier::Named(n) => {
                let n = AstBox::into_inner(n);
                legacy::ImportSpecifier::Named(legacy::ImportNamedSpecifier {
                    span: compat_span(n.span),
                    local: self.compat_ident(n.local),
                    imported: n.imported.map(|me| self.compat_module_export_name(me)),
                    is_type_only: n.is_type_only,
                })
            }
            experimental::ImportSpecifier::Default(d) => {
                let d = AstBox::into_inner(d);
                legacy::ImportSpecifier::Default(legacy::ImportDefaultSpecifier {
                    span: compat_span(d.span),
                    local: self.compat_ident(d.local),
                })
            }
            experimental::ImportSpecifier::Namespace(ns) => {
                let ns = AstBox::into_inner(ns);
                legacy::ImportSpecifier::Namespace(legacy::ImportStarAsSpecifier {
                    span: compat_span(ns.span),
                    local: self.compat_ident(ns.local),
                })
            }
        }
    }

    fn compat_export_specifier<'a>(
        &mut self,

        s: experimental::ExportSpecifier<'a>,
    ) -> legacy::ExportSpecifier {
        match s {
            experimental::ExportSpecifier::Namespace(ns) => {
                let ns = AstBox::into_inner(ns);
                legacy::ExportSpecifier::Namespace(legacy::ExportNamespaceSpecifier {
                    span: compat_span(ns.span),
                    name: self.compat_module_export_name(ns.name),
                })
            }
            experimental::ExportSpecifier::Default(d) => {
                let d = AstBox::into_inner(d);
                legacy::ExportSpecifier::Default(legacy::ExportDefaultSpecifier {
                    exported: self.compat_ident(d.exported),
                })
            }
            experimental::ExportSpecifier::Named(n) => {
                let n = AstBox::into_inner(n);
                legacy::ExportSpecifier::Named(legacy::ExportNamedSpecifier {
                    span: compat_span(n.span),
                    orig: self.compat_module_export_name(n.orig),
                    exported: n.exported.map(|me| self.compat_module_export_name(me)),
                    is_type_only: n.is_type_only,
                })
            }
        }
    }

    fn compat_module_export_name<'a>(
        &mut self,

        n: experimental::ModuleExportName<'a>,
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

    fn compat_object_lit<'a>(
        &mut self,
        o: AstBox<'a, experimental::ObjectLit<'a>>,
    ) -> legacy::ObjectLit {
        let o = AstBox::into_inner(o);
        legacy::ObjectLit {
            span: compat_span(o.span),
            props: self.compat_vec(o.props, Self::compat_prop_or_spread),
        }
    }

    fn compat_prop_or_spread<'a>(
        &mut self,
        p: experimental::PropOrSpread<'a>,
    ) -> legacy::PropOrSpread {
        match p {
            experimental::PropOrSpread::Spread(s) => {
                let s = AstBox::into_inner(s);
                legacy::PropOrSpread::Spread(legacy::SpreadElement {
                    dot3_token: compat_span(s.dot3_token),
                    expr: self.compat_expr(s.expr),
                })
            }
            experimental::PropOrSpread::Prop(prop) => {
                legacy::PropOrSpread::Prop(alloc_box!(self, self.compat_prop(prop)))
            }
        }
    }

    fn compat_prop<'a>(&mut self, p: AstBox<'a, experimental::Prop<'a>>) -> legacy::Prop {
        let p = AstBox::into_inner(p);
        match p {
            experimental::Prop::Shorthand(i) => legacy::Prop::Shorthand(self.compat_ident(i)),
            experimental::Prop::KeyValue(kv) => {
                let kv = AstBox::into_inner(kv);
                legacy::Prop::KeyValue(legacy::KeyValueProp {
                    key: self.compat_prop_name(kv.key),
                    value: self.compat_expr(kv.value),
                })
            }
            experimental::Prop::Assign(ap) => {
                let ap = AstBox::into_inner(ap);
                legacy::Prop::Assign(legacy::AssignProp {
                    span: compat_span(ap.span),
                    key: self.compat_ident(ap.key),
                    value: self.compat_expr(ap.value),
                })
            }
            experimental::Prop::Getter(g) => {
                let g = AstBox::into_inner(g);
                legacy::Prop::Getter(legacy::GetterProp {
                    span: compat_span(g.span),
                    key: self.compat_prop_name(g.key),
                    type_ann: None,
                    body: g.body.map(|b| self.compat_block_stmt(b)),
                })
            }
            experimental::Prop::Setter(s) => {
                let s = AstBox::into_inner(s);
                legacy::Prop::Setter(legacy::SetterProp {
                    span: compat_span(s.span),
                    key: self.compat_prop_name(s.key),
                    this_param: s.this_param.map(|p| self.compat_pat(p)),
                    param: alloc_box!(self, self.compat_pat(s.param)),
                    body: s.body.map(|b| self.compat_block_stmt(b)),
                })
            }
            experimental::Prop::Method(m) => {
                let m = AstBox::into_inner(m);
                legacy::Prop::Method(legacy::MethodProp {
                    key: self.compat_prop_name(m.key),
                    function: alloc_box!(self, self.compat_function(m.function)),
                })
            }
        }
    }

    fn compat_prop_name<'a>(&mut self, n: experimental::PropName<'a>) -> legacy::PropName {
        match n {
            experimental::PropName::Ident(i) => {
                let i = AstBox::into_inner(i);
                legacy::PropName::Ident(legacy::IdentName {
                    span: compat_span(i.span),
                    sym: self.compat_utf8_ref(i.sym),
                })
            }
            experimental::PropName::Str(s) => legacy::PropName::Str(self.compat_str(s)),
            experimental::PropName::Num(n) => {
                let n = AstBox::into_inner(n);
                legacy::PropName::Num(legacy::Number {
                    span: compat_span(n.span),
                    value: n.value,
                    raw: self.compat_opt_utf8_ref(n.raw),
                })
            }
            experimental::PropName::Computed(c) => {
                let c = AstBox::into_inner(c);
                legacy::PropName::Computed(legacy::ComputedPropName {
                    span: compat_span(c.span),
                    expr: self.compat_expr(c.expr),
                })
            }
            experimental::PropName::BigInt(b) => {
                let b = AstBox::into_inner(b);
                legacy::PropName::BigInt(legacy::BigInt {
                    span: compat_span(b.span),
                    value: alloc_box!(self, self.compat_big_int(b.value)),
                    raw: self.compat_opt_utf8_ref(b.raw),
                })
            }
        }
    }

    fn compat_str(&mut self, s: AstBox<'_, experimental::Str>) -> legacy::Str {
        let s = AstBox::into_inner(s);
        legacy::Str {
            span: compat_span(s.span),
            value: self.compat_wtf8_ref(s.value),
            raw: self.compat_opt_utf8_ref(s.raw),
        }
    }

    // -------------------------------------------------------------------------------
    // Function / Class basics used by export default

    fn compat_fn_expr<'a>(&mut self, f: AstBox<'a, experimental::FnExpr<'a>>) -> legacy::FnExpr {
        let f = AstBox::into_inner(f);
        legacy::FnExpr {
            ident: f.ident.map(|i| self.compat_ident(i)),
            function: alloc_box!(self, self.compat_function(f.function)),
        }
    }

    fn compat_class_expr<'a>(
        &mut self,
        c: AstBox<'a, experimental::ClassExpr<'a>>,
    ) -> legacy::ClassExpr {
        let c = AstBox::into_inner(c);
        legacy::ClassExpr {
            ident: c.ident.map(|i| self.compat_ident(i)),
            class: alloc_box!(self, self.compat_class(c.class)),
        }
    }

    fn compat_function<'a>(
        &mut self,
        f: AstBox<'a, experimental::Function<'a>>,
    ) -> legacy::Function {
        let f = AstBox::into_inner(f);
        legacy::Function {
            params: self.compat_vec(f.params, Self::compat_param),
            decorators: self.compat_vec(f.decorators, Self::compat_decorator),
            span: compat_span(f.span),
            ctxt: Default::default(),
            body: Some(self.compat_block_stmt(f.body)),
            is_generator: f.is_generator,
            is_async: f.is_async,
            type_params: None,
            return_type: None,
        }
    }

    fn compat_param<'a>(&mut self, p: experimental::Param<'a>) -> legacy::Param {
        legacy::Param {
            span: compat_span(p.span),
            decorators: self.compat_vec(p.decorators, Self::compat_decorator),
            pat: self.compat_pat(p.pat),
        }
    }

    fn compat_decorator<'a>(&mut self, d: experimental::Decorator<'a>) -> legacy::Decorator {
        legacy::Decorator {
            span: compat_span(d.span),
            expr: self.compat_expr(d.expr),
        }
    }

    fn compat_class<'a>(&mut self, c: AstBox<'a, experimental::Class<'a>>) -> legacy::Class {
        let c = AstBox::into_inner(c);
        legacy::Class {
            span: compat_span(c.span),
            ctxt: Default::default(),
            decorators: self.compat_vec(c.decorators, Self::compat_decorator),
            body: self.compat_vec(c.body, Self::compat_class_member),
            super_class: c.super_class.map(|e| self.compat_expr(e)),
            is_abstract: c.is_abstract,
            type_params: None,
            super_type_params: None,
            implements: Default::default(),
        }
    }

    fn compat_class_member<'a>(&mut self, m: experimental::ClassMember<'a>) -> legacy::ClassMember {
        match m {
            experimental::ClassMember::Constructor(k) => {
                let k = AstBox::into_inner(k);
                legacy::ClassMember::Constructor(legacy::Constructor {
                    span: compat_span(k.span),
                    ctxt: Default::default(),
                    key: self.compat_prop_name(k.key),
                    params: self.compat_vec(k.params, Self::compat_param_or_ts_param_prop),
                    body: k.body.map(|b| self.compat_block_stmt(b)),
                    accessibility: None,
                    is_optional: false,
                })
            }
            experimental::ClassMember::Method(me) => {
                let me = AstBox::into_inner(me);
                legacy::ClassMember::Method(legacy::ClassMethod {
                    span: compat_span(me.span),
                    key: self.compat_prop_name(me.key),
                    function: alloc_box!(self, self.compat_function(me.function)),
                    kind: match me.kind {
                        experimental::MethodKind::Method => legacy::MethodKind::Method,
                        experimental::MethodKind::Getter => legacy::MethodKind::Getter,
                        experimental::MethodKind::Setter => legacy::MethodKind::Setter,
                    },
                    is_static: me.is_static,
                    accessibility: None,
                    is_abstract: false,
                    is_optional: false,
                    is_override: false,
                })
            }
            experimental::ClassMember::PrivateMethod(pm) => {
                let pm = AstBox::into_inner(pm);
                let key = AstBox::into_inner(pm.key);
                legacy::ClassMember::PrivateMethod(legacy::PrivateMethod {
                    span: compat_span(pm.span),
                    key: legacy::PrivateName {
                        span: compat_span(key.span),
                        name: self.compat_utf8_ref(key.name),
                    },
                    function: alloc_box!(self, self.compat_function(pm.function)),
                    kind: match pm.kind {
                        experimental::MethodKind::Method => legacy::MethodKind::Method,
                        experimental::MethodKind::Getter => legacy::MethodKind::Getter,
                        experimental::MethodKind::Setter => legacy::MethodKind::Setter,
                    },
                    is_static: pm.is_static,
                    accessibility: None,
                    is_abstract: false,
                    is_optional: false,
                    is_override: false,
                })
            }
            experimental::ClassMember::ClassProp(cp) => {
                let cp = AstBox::into_inner(cp);
                legacy::ClassMember::ClassProp(legacy::ClassProp {
                    span: compat_span(cp.span),
                    key: self.compat_prop_name(cp.key),
                    value: cp.value.map(|e| self.compat_expr(e)),
                    type_ann: None,
                    is_static: cp.is_static,
                    decorators: self.compat_vec(cp.decorators, Self::compat_decorator),
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
                let pp = AstBox::into_inner(pp);
                let key = AstBox::into_inner(pp.key);
                legacy::ClassMember::PrivateProp(legacy::PrivateProp {
                    span: compat_span(pp.span),
                    ctxt: Default::default(),
                    key: legacy::PrivateName {
                        span: compat_span(key.span),
                        name: self.compat_utf8_ref(key.name),
                    },
                    value: pp.value.map(|e| self.compat_expr(e)),
                    type_ann: None,
                    is_static: pp.is_static,
                    decorators: self.compat_vec(pp.decorators, Self::compat_decorator),
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
                let sb = AstBox::into_inner(sb);
                legacy::ClassMember::StaticBlock(legacy::StaticBlock {
                    span: compat_span(sb.span),
                    body: self.compat_block_stmt(sb.body),
                })
            }
            experimental::ClassMember::AutoAccessor(a) => {
                let a = AstBox::into_inner(a);
                legacy::ClassMember::AutoAccessor(legacy::AutoAccessor {
                    span: compat_span(a.span),
                    key: match a.key {
                        experimental::Key::Private(p) => {
                            let p = AstBox::into_inner(p);
                            legacy::Key::Private(legacy::PrivateName {
                                span: compat_span(p.span),
                                name: self.compat_utf8_ref(p.name),
                            })
                        }
                        experimental::Key::Public(n) => {
                            legacy::Key::Public(self.compat_prop_name(AstBox::into_inner(n)))
                        }
                    },
                    value: a.value.map(|e| self.compat_expr(e)),
                    type_ann: None,
                    is_static: a.is_static,
                    decorators: self.compat_vec(a.decorators, Self::compat_decorator),
                    accessibility: None,
                    is_abstract: false,
                    is_override: false,
                    definite: false,
                })
            }
        }
    }

    fn compat_param_or_ts_param_prop<'a>(
        &mut self,

        p: experimental::ParamOrTsParamProp<'a>,
    ) -> legacy::ParamOrTsParamProp {
        match p {
            experimental::ParamOrTsParamProp::Param(pp) => {
                legacy::ParamOrTsParamProp::Param(self.compat_param(AstBox::into_inner(pp)))
            }
        }
    }

    // -------------------------------------------------------------------------------
    // Patterns and declarations

    fn compat_decl<'a>(&mut self, d: experimental::Decl<'a>) -> legacy::Decl {
        match d {
            experimental::Decl::Class(c) => {
                let c = AstBox::into_inner(c);
                legacy::Decl::Class(legacy::ClassDecl {
                    ident: self.compat_ident(c.ident),
                    declare: c.declare,
                    class: alloc_box!(self, self.compat_class(c.class)),
                })
            }
            experimental::Decl::Fn(f) => {
                let f = AstBox::into_inner(f);
                legacy::Decl::Fn(legacy::FnDecl {
                    ident: self.compat_ident(f.ident),
                    declare: f.declare,
                    function: alloc_box!(self, self.compat_function(f.function)),
                })
            }
            experimental::Decl::Var(v) => {
                legacy::Decl::Var(alloc_box!(self, self.compat_var_decl(v)))
            }
            experimental::Decl::Using(u) => {
                let u = AstBox::into_inner(u);
                legacy::Decl::Using(alloc_box!(
                    self,
                    legacy::UsingDecl {
                        span: compat_span(u.span),
                        is_await: u.is_await,
                        decls: self.compat_vec(u.decls, Self::compat_var_declarator),
                    }
                ))
            }
        }
    }

    fn compat_var_declarator<'a>(
        &mut self,
        d: experimental::VarDeclarator<'a>,
    ) -> legacy::VarDeclarator {
        legacy::VarDeclarator {
            span: compat_span(d.span),
            name: self.compat_pat(d.name),
            init: d.init.map(|e| self.compat_expr(e)),
            definite: false,
        }
    }

    fn compat_pat<'a>(&mut self, p: experimental::Pat<'a>) -> legacy::Pat {
        match p {
            experimental::Pat::Ident(b) => {
                let b = AstBox::into_inner(b);
                legacy::Pat::Ident(legacy::BindingIdent {
                    id: self.compat_ident(b.id),
                    type_ann: None,
                })
            }
            experimental::Pat::Array(a) => {
                let a = AstBox::into_inner(a);
                legacy::Pat::Array(legacy::ArrayPat {
                    span: compat_span(a.span),
                    elems: self.compat_vec(a.elems, |this, p| p.map(|p| this.compat_pat(p))),
                    optional: a.optional,
                    type_ann: None,
                })
            }
            experimental::Pat::Rest(r) => {
                let r = AstBox::into_inner(r);
                legacy::Pat::Rest(legacy::RestPat {
                    span: compat_span(r.span),
                    dot3_token: compat_span(r.dot3_token),
                    arg: alloc_box!(self, self.compat_pat(r.arg)),
                    type_ann: None,
                })
            }
            experimental::Pat::Object(o) => {
                let o = AstBox::into_inner(o);
                legacy::Pat::Object(legacy::ObjectPat {
                    span: compat_span(o.span),
                    props: self.compat_vec(o.props, Self::compat_object_pat_prop),
                    optional: o.optional,
                    type_ann: None,
                })
            }
            experimental::Pat::Assign(a) => {
                let a = AstBox::into_inner(a);
                legacy::Pat::Assign(legacy::AssignPat {
                    span: compat_span(a.span),
                    left: alloc_box!(self, self.compat_pat(a.left)),
                    right: self.compat_expr(a.right),
                })
            }
            experimental::Pat::Invalid(_) => legacy::Pat::Invalid(legacy::Invalid {
                span: Default::default(),
            }),
            experimental::Pat::Expr(e) => {
                legacy::Pat::Expr(self.compat_expr(AstBox::into_inner(e)))
            }
        }
    }

    fn compat_object_pat_prop<'a>(
        &mut self,
        p: experimental::ObjectPatProp<'a>,
    ) -> legacy::ObjectPatProp {
        match p {
            experimental::ObjectPatProp::KeyValue(kv) => {
                let kv = AstBox::into_inner(kv);
                legacy::ObjectPatProp::KeyValue(legacy::KeyValuePatProp {
                    key: self.compat_prop_name(kv.key),
                    value: alloc_box!(self, self.compat_pat(kv.value)),
                })
            }
            experimental::ObjectPatProp::Assign(ap) => {
                let ap = AstBox::into_inner(ap);
                let key = AstBox::into_inner(ap.key);
                legacy::ObjectPatProp::Assign(legacy::AssignPatProp {
                    span: compat_span(ap.span),
                    key: legacy::BindingIdent {
                        id: self.compat_ident(key.id),
                        type_ann: None,
                    },
                    value: ap.value.map(|e| self.compat_expr(e)),
                })
            }
            experimental::ObjectPatProp::Rest(r) => {
                let r = AstBox::into_inner(r);
                legacy::ObjectPatProp::Rest(legacy::RestPat {
                    span: compat_span(r.span),
                    dot3_token: compat_span(r.dot3_token),
                    arg: alloc_box!(self, self.compat_pat(r.arg)),
                    type_ann: None,
                })
            }
        }
    }
    fn compat_expr_or_spread<'a>(
        &mut self,
        e: experimental::ExprOrSpread<'a>,
    ) -> legacy::ExprOrSpread {
        legacy::ExprOrSpread {
            spread: e.spread.map(compat_span),
            expr: self.compat_expr(e.expr),
        }
    }

    fn compat_member_prop<'a>(&mut self, p: experimental::MemberProp<'a>) -> legacy::MemberProp {
        match p {
            experimental::MemberProp::Ident(i) => {
                let i = AstBox::into_inner(i);
                legacy::MemberProp::Ident(legacy::IdentName {
                    span: compat_span(i.span),
                    sym: self.compat_utf8_ref(i.sym),
                })
            }
            experimental::MemberProp::PrivateName(pn) => {
                let pn = AstBox::into_inner(pn);
                legacy::MemberProp::PrivateName(legacy::PrivateName {
                    span: compat_span(pn.span),
                    name: self.compat_utf8_ref(pn.name),
                })
            }
            experimental::MemberProp::Computed(c) => {
                let c = AstBox::into_inner(c);
                legacy::MemberProp::Computed(legacy::ComputedPropName {
                    span: compat_span(c.span),
                    expr: self.compat_expr(c.expr),
                })
            }
        }
    }

    fn compat_lit<'a>(&mut self, l: AstBox<'a, experimental::Lit<'a>>) -> legacy::Lit {
        let l = AstBox::into_inner(l);
        match l {
            experimental::Lit::Str(s) => legacy::Lit::Str(self.compat_str(s)),
            experimental::Lit::Bool(b) => {
                let b = AstBox::into_inner(b);
                legacy::Lit::Bool(legacy::Bool {
                    span: compat_span(b.span),
                    value: b.value,
                })
            }
            experimental::Lit::Null(n) => {
                let n = AstBox::into_inner(n);
                legacy::Lit::Null(legacy::Null {
                    span: compat_span(n.span),
                })
            }
            experimental::Lit::Num(n) => {
                let n = AstBox::into_inner(n);
                legacy::Lit::Num(legacy::Number {
                    span: compat_span(n.span),
                    value: n.value,
                    raw: self.compat_opt_utf8_ref(n.raw),
                })
            }
            experimental::Lit::BigInt(b) => {
                let b = AstBox::into_inner(b);
                legacy::Lit::BigInt(legacy::BigInt {
                    span: compat_span(b.span),
                    value: alloc_box!(self, self.compat_big_int(b.value)),
                    raw: self.compat_opt_utf8_ref(b.raw),
                })
            }
            experimental::Lit::Regex(r) => {
                let r = AstBox::into_inner(r);
                legacy::Lit::Regex(legacy::Regex {
                    span: compat_span(r.span),
                    exp: self.compat_utf8_ref(r.exp),
                    flags: self.compat_utf8_ref(r.flags),
                })
            }
        }
    }

    fn compat_tpl_element(&mut self, e: experimental::TplElement) -> legacy::TplElement {
        legacy::TplElement {
            span: compat_span(e.span),
            tail: e.tail,
            cooked: self.compat_opt_wtf8_ref(e.cooked),
            raw: self.compat_utf8_ref(e.raw),
        }
    }

    fn compat_tpl<'a>(&mut self, t: experimental::Tpl<'a>) -> legacy::Tpl {
        legacy::Tpl {
            span: compat_span(t.span),
            exprs: self.compat_vec(t.exprs, Self::compat_expr),
            quasis: self.compat_vec(t.quasis, Self::compat_tpl_element),
        }
    }

    fn compat_assign_target<'a>(
        &mut self,
        t: experimental::AssignTarget<'a>,
    ) -> legacy::AssignTarget {
        match t {
            experimental::AssignTarget::Simple(s) => {
                legacy::AssignTarget::Simple(match AstBox::into_inner(s) {
                    experimental::SimpleAssignTarget::Ident(b) => {
                        let b = AstBox::into_inner(b);
                        legacy::SimpleAssignTarget::Ident(legacy::BindingIdent {
                            id: self.compat_ident(b.id),
                            type_ann: None,
                        })
                    }
                    experimental::SimpleAssignTarget::Member(m) => {
                        let m = AstBox::into_inner(m);
                        legacy::SimpleAssignTarget::Member(legacy::MemberExpr {
                            span: compat_span(m.span),
                            obj: self.compat_expr(m.obj),
                            prop: self.compat_member_prop(m.prop),
                        })
                    }
                    experimental::SimpleAssignTarget::SuperProp(su) => {
                        let su = AstBox::into_inner(su);
                        legacy::SimpleAssignTarget::SuperProp(legacy::SuperPropExpr {
                            span: compat_span(su.span),
                            obj: legacy::Super {
                                span: compat_span(su.obj.span),
                            },
                            prop: match su.prop {
                                experimental::SuperProp::Ident(i) => {
                                    let i = AstBox::into_inner(i);
                                    legacy::SuperProp::Ident(legacy::IdentName {
                                        span: compat_span(i.span),
                                        sym: self.compat_utf8_ref(i.sym),
                                    })
                                }
                                experimental::SuperProp::Computed(c) => {
                                    let c = AstBox::into_inner(c);
                                    legacy::SuperProp::Computed(legacy::ComputedPropName {
                                        span: compat_span(c.span),
                                        expr: self.compat_expr(c.expr),
                                    })
                                }
                            },
                        })
                    }
                    experimental::SimpleAssignTarget::Paren(p) => {
                        let p = AstBox::into_inner(p);
                        legacy::SimpleAssignTarget::Paren(legacy::ParenExpr {
                            span: compat_span(p.span),
                            expr: self.compat_expr(p.expr),
                        })
                    }
                    experimental::SimpleAssignTarget::OptChain(o) => {
                        let o = AstBox::into_inner(o);
                        legacy::SimpleAssignTarget::OptChain(legacy::OptChainExpr {
                            span: compat_span(o.span),
                            optional: o.optional,
                            base: alloc_box!(
                                self,
                                match o.base {
                                    experimental::OptChainBase::Member(m) => {
                                        let m = AstBox::into_inner(m);
                                        legacy::OptChainBase::Member(legacy::MemberExpr {
                                            span: compat_span(m.span),
                                            obj: self.compat_expr(m.obj),
                                            prop: self.compat_member_prop(m.prop),
                                        })
                                    }
                                    experimental::OptChainBase::Call(c) => {
                                        let c = AstBox::into_inner(c);
                                        legacy::OptChainBase::Call(legacy::OptCall {
                                            span: compat_span(c.span),
                                            ctxt: Default::default(),
                                            callee: self.compat_expr(c.callee),
                                            args: self
                                                .compat_vec(c.args, Self::compat_expr_or_spread),
                                            type_args: None,
                                        })
                                    }
                                }
                            ),
                        })
                    }
                    experimental::SimpleAssignTarget::Invalid(_) => {
                        legacy::SimpleAssignTarget::Invalid(legacy::Invalid {
                            span: Default::default(),
                        })
                    }
                })
            }
            experimental::AssignTarget::Pat(p) => {
                legacy::AssignTarget::Pat(match AstBox::into_inner(p) {
                    experimental::AssignTargetPat::Array(a) => {
                        let a = AstBox::into_inner(a);
                        legacy::AssignTargetPat::Array(legacy::ArrayPat {
                            span: compat_span(a.span),
                            elems: self.compat_vec(a.elems, |this, pat| {
                                pat.map(|pat| this.compat_pat(pat))
                            }),
                            optional: false,
                            type_ann: None,
                        })
                    }
                    experimental::AssignTargetPat::Object(o) => {
                        let o = AstBox::into_inner(o);
                        legacy::AssignTargetPat::Object(legacy::ObjectPat {
                            span: compat_span(o.span),
                            props: self.compat_vec(o.props, Self::compat_object_pat_prop),
                            optional: false,
                            type_ann: None,
                        })
                    }
                    experimental::AssignTargetPat::Invalid(_) => {
                        legacy::AssignTargetPat::Invalid(legacy::Invalid {
                            span: Default::default(),
                        })
                    }
                })
            }
        }
    }

    // -------------------------------------------------------------------------------
    // JSX compatibility helpers

    fn compat_jsx_object<'a>(&mut self, o: experimental::JSXObject<'a>) -> legacy::JSXObject {
        match o {
            experimental::JSXObject::JSXMemberExpr(m) => {
                legacy::JSXObject::JSXMemberExpr(alloc_box!(self, self.compat_jsx_member_expr(m)))
            }
            experimental::JSXObject::Ident(i) => legacy::JSXObject::Ident(self.compat_ident(i)),
        }
    }

    fn compat_ident_name(&mut self, i: AstBox<'_, experimental::IdentName>) -> legacy::IdentName {
        let i = AstBox::into_inner(i);
        legacy::IdentName {
            span: compat_span(i.span),
            sym: self.compat_utf8_ref(i.sym),
        }
    }

    fn compat_jsx_member_expr<'a>(
        &mut self,
        j: AstBox<'a, experimental::JSXMemberExpr<'a>>,
    ) -> legacy::JSXMemberExpr {
        let j = AstBox::into_inner(j);
        legacy::JSXMemberExpr {
            span: compat_span(j.span),
            obj: self.compat_jsx_object(j.obj),
            prop: self.compat_ident_name(j.prop),
        }
    }

    fn compat_jsx_namespaced_name<'a>(
        &mut self,

        j: AstBox<'a, experimental::JSXNamespacedName<'a>>,
    ) -> legacy::JSXNamespacedName {
        let j = AstBox::into_inner(j);
        legacy::JSXNamespacedName {
            span: compat_span(j.span),
            ns: self.compat_ident_name(j.ns),
            name: self.compat_ident_name(j.name),
        }
    }

    fn compat_jsx_empty_expr(
        &mut self,
        j: AstBox<'_, experimental::JSXEmptyExpr>,
    ) -> legacy::JSXEmptyExpr {
        let j = AstBox::into_inner(j);
        legacy::JSXEmptyExpr {
            span: compat_span(j.span),
        }
    }

    fn compat_jsx_expr<'a>(&mut self, e: experimental::JSXExpr<'a>) -> legacy::JSXExpr {
        match e {
            experimental::JSXExpr::JSXEmptyExpr(ee) => {
                legacy::JSXExpr::JSXEmptyExpr(self.compat_jsx_empty_expr(ee))
            }
            experimental::JSXExpr::Expr(ex) => {
                legacy::JSXExpr::Expr(self.compat_expr(AstBox::into_inner(ex)))
            }
        }
    }

    fn compat_jsx_expr_container<'a>(
        &mut self,

        c: AstBox<'a, experimental::JSXExprContainer<'a>>,
    ) -> legacy::JSXExprContainer {
        let c = AstBox::into_inner(c);
        legacy::JSXExprContainer {
            span: compat_span(c.span),
            expr: self.compat_jsx_expr(c.expr),
        }
    }

    fn compat_spread_element<'a>(
        &mut self,
        s: AstBox<'a, experimental::SpreadElement<'a>>,
    ) -> legacy::SpreadElement {
        let s = AstBox::into_inner(s);
        legacy::SpreadElement {
            dot3_token: compat_span(s.dot3_token),
            expr: self.compat_expr(s.expr),
        }
    }

    fn compat_jsx_attr_name<'a>(
        &mut self,
        n: experimental::JSXAttrName<'a>,
    ) -> legacy::JSXAttrName {
        match n {
            experimental::JSXAttrName::Ident(i) => {
                legacy::JSXAttrName::Ident(self.compat_ident_name(i))
            }
            experimental::JSXAttrName::JSXNamespacedName(nn) => {
                legacy::JSXAttrName::JSXNamespacedName(self.compat_jsx_namespaced_name(nn))
            }
        }
    }

    fn compat_jsx_attr_value<'a>(
        &mut self,
        v: experimental::JSXAttrValue<'a>,
    ) -> legacy::JSXAttrValue {
        match v {
            experimental::JSXAttrValue::Str(s) => legacy::JSXAttrValue::Str(self.compat_str(s)),
            experimental::JSXAttrValue::JSXExprContainer(c) => {
                legacy::JSXAttrValue::JSXExprContainer(self.compat_jsx_expr_container(c))
            }
            experimental::JSXAttrValue::JSXElement(e) => {
                legacy::JSXAttrValue::JSXElement(alloc_box!(self, self.compat_jsx_element(e)))
            }
            experimental::JSXAttrValue::JSXFragment(f) => {
                legacy::JSXAttrValue::JSXFragment(self.compat_jsx_fragment(f))
            }
        }
    }

    fn compat_jsx_attr<'a>(&mut self, a: AstBox<'a, experimental::JSXAttr<'a>>) -> legacy::JSXAttr {
        let a = AstBox::into_inner(a);
        legacy::JSXAttr {
            span: compat_span(a.span),
            name: self.compat_jsx_attr_name(a.name),
            value: a.value.map(|v| self.compat_jsx_attr_value(v)),
        }
    }

    fn compat_jsx_attr_or_spread<'a>(
        &mut self,

        a: experimental::JSXAttrOrSpread<'a>,
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

    fn compat_jsx_element_name<'a>(
        &mut self,

        n: experimental::JSXElementName<'a>,
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

    fn compat_jsx_opening_element<'a>(
        &mut self,

        o: AstBox<'a, experimental::JSXOpeningElement<'a>>,
    ) -> legacy::JSXOpeningElement {
        let o = AstBox::into_inner(o);
        legacy::JSXOpeningElement {
            span: compat_span(o.span),
            name: self.compat_jsx_element_name(o.name),
            attrs: self.compat_vec(o.attrs, Self::compat_jsx_attr_or_spread),
            self_closing: o.self_closing,
            type_args: None,
        }
    }

    fn compat_jsx_closing_element<'a>(
        &mut self,

        c: AstBox<'a, experimental::JSXClosingElement<'a>>,
    ) -> legacy::JSXClosingElement {
        let c = AstBox::into_inner(c);
        legacy::JSXClosingElement {
            span: compat_span(c.span),
            name: self.compat_jsx_element_name(c.name),
        }
    }

    fn compat_jsx_text(&mut self, t: AstBox<'_, experimental::JSXText>) -> legacy::JSXText {
        let t = AstBox::into_inner(t);
        legacy::JSXText {
            span: compat_span(t.span),
            value: self.compat_utf8_ref(t.value),
            raw: self.compat_utf8_ref(t.raw),
        }
    }

    fn compat_jsx_spread_child<'a>(
        &mut self,

        s: AstBox<'a, experimental::JSXSpreadChild<'a>>,
    ) -> legacy::JSXSpreadChild {
        let s = AstBox::into_inner(s);
        legacy::JSXSpreadChild {
            span: compat_span(s.span),
            expr: self.compat_expr(s.expr),
        }
    }

    fn compat_jsx_element_child<'a>(
        &mut self,

        c: experimental::JSXElementChild<'a>,
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
                legacy::JSXElementChild::JSXElement(alloc_box!(self, self.compat_jsx_element(e)))
            }
            experimental::JSXElementChild::JSXFragment(f) => {
                legacy::JSXElementChild::JSXFragment(self.compat_jsx_fragment(f))
            }
        }
    }

    fn compat_jsx_element<'a>(
        &mut self,
        e: AstBox<'a, experimental::JSXElement<'a>>,
    ) -> legacy::JSXElement {
        let e = AstBox::into_inner(e);
        legacy::JSXElement {
            span: compat_span(e.span),
            opening: self.compat_jsx_opening_element(e.opening),
            children: self.compat_vec(e.children, Self::compat_jsx_element_child),
            closing: e.closing.map(|c| self.compat_jsx_closing_element(c)),
        }
    }

    fn compat_jsx_opening_fragment(
        &mut self,

        o: AstBox<'_, experimental::JSXOpeningFragment>,
    ) -> legacy::JSXOpeningFragment {
        let o = AstBox::into_inner(o);
        legacy::JSXOpeningFragment {
            span: compat_span(o.span),
        }
    }

    fn compat_jsx_closing_fragment(
        &mut self,

        c: AstBox<'_, experimental::JSXClosingFragment>,
    ) -> legacy::JSXClosingFragment {
        let c = AstBox::into_inner(c);
        legacy::JSXClosingFragment {
            span: compat_span(c.span),
        }
    }

    fn compat_jsx_fragment<'a>(
        &mut self,
        f: AstBox<'a, experimental::JSXFragment<'a>>,
    ) -> legacy::JSXFragment {
        let f = AstBox::into_inner(f);
        legacy::JSXFragment {
            span: compat_span(f.span),
            opening: self.compat_jsx_opening_fragment(f.opening),
            children: self.compat_vec(f.children, Self::compat_jsx_element_child),
            closing: self.compat_jsx_closing_fragment(f.closing),
        }
    }

    // ===============================================================================

    fn compat_utf8_ref(&mut self, atom: ExperimentalAtom<'_>) -> Atom {
        Atom::from(atom.as_str())
    }

    fn compat_opt_utf8_ref(&mut self, atom: Option<ExperimentalAtom<'_>>) -> Option<Atom> {
        atom.map(|atom| self.compat_utf8_ref(atom))
    }

    fn compat_wtf8_ref(&mut self, atom: ExperimentalWtf8Atom<'_>) -> Wtf8Atom {
        match atom.as_wtf8().as_str() {
            Some(s) => Wtf8Atom::from(s),
            None => Wtf8Atom::from(atom.as_wtf8().to_string_lossy().into_owned()),
        }
    }

    fn compat_opt_wtf8_ref(&mut self, atom: Option<ExperimentalWtf8Atom<'_>>) -> Option<Wtf8Atom> {
        atom.map(|atom| self.compat_wtf8_ref(atom))
    }

    fn compat_big_int(&mut self, value: ExperimentalAtom<'_>) -> legacy::BigIntValue {
        legacy::BigIntValue::parse_bytes(value.as_bytes(), 10).unwrap()
    }
}
