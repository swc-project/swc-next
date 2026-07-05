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

fn convert_span(span: experimental::Span) -> SwcSpan {
    SwcSpan::new_with_checked(BytePos(span.start), BytePos(span.end))
}

macro_rules! alloc_box {
    ($self:expr, $value:expr) => {{
        let value = $value;
        $self.alloc_box(value)
    }};
}

pub(crate) trait AstConvert {
    fn semantic(&self) -> &Semantic;

    fn alloc_box<T>(&self, value: T) -> Box<T>;

    fn convert_vec<T, U, F: Fn(&mut Self, T) -> U>(
        &mut self,
        items: ArenaVec<'_, T>,
        transformer: F,
    ) -> Vec<U>;

    fn convert_program_inner<'a>(&mut self, root: experimental::Program<'a>) -> legacy::Program {
        match root {
            experimental::Program::Module(module) => {
                legacy::Program::Module(self.convert_module_inner(AstBox::into_inner(module)))
            }
            experimental::Program::Script(script) => {
                legacy::Program::Script(self.convert_script_inner(AstBox::into_inner(script)))
            }
        }
    }

    fn convert_module_inner<'a>(&mut self, module: experimental::Module<'a>) -> legacy::Module {
        #[allow(unused_mut)]
        let mut inner = || legacy::Module {
            span: convert_span(module.span),
            shebang: self.convert_opt_utf8_ref(module.shebang),
            body: self.convert_vec(module.body, Self::convert_module_item),
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

    fn convert_script_inner<'a>(&mut self, script: experimental::Script<'a>) -> legacy::Script {
        #[allow(unused_mut)]
        let mut inner = || legacy::Script {
            span: convert_span(script.span),
            body: self.convert_vec(script.body, Self::convert_stmt),
            shebang: self.convert_opt_utf8_ref(script.shebang),
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

    fn convert_module_item<'a>(
        &mut self,
        item: experimental::ModuleItem<'a>,
    ) -> legacy::ModuleItem {
        match item {
            experimental::ModuleItem::ModuleDecl(module_decl) => legacy::ModuleItem::ModuleDecl(
                self.convert_module_decl(AstBox::into_inner(module_decl)),
            ),
            experimental::ModuleItem::Stmt(stmt) => {
                legacy::ModuleItem::Stmt(self.convert_stmt(AstBox::into_inner(stmt)))
            }
        }
    }

    fn convert_module_decl<'a>(
        &mut self,
        module_decl: experimental::ModuleDecl<'a>,
    ) -> legacy::ModuleDecl {
        match module_decl {
            experimental::ModuleDecl::Import(import_decl) => {
                legacy::ModuleDecl::Import(self.convert_import_decl(import_decl))
            }
            experimental::ModuleDecl::ExportDecl(export_decl) => {
                legacy::ModuleDecl::ExportDecl(self.convert_export_decl(export_decl))
            }
            experimental::ModuleDecl::ExportNamed(named_export) => {
                legacy::ModuleDecl::ExportNamed(self.convert_export_named(named_export))
            }
            experimental::ModuleDecl::ExportDefaultDecl(export_default_decl) => {
                legacy::ModuleDecl::ExportDefaultDecl(
                    self.convert_export_default_decl(export_default_decl),
                )
            }
            experimental::ModuleDecl::ExportDefaultExpr(export_default_expr) => {
                legacy::ModuleDecl::ExportDefaultExpr(
                    self.convert_export_default_expr(export_default_expr),
                )
            }
            experimental::ModuleDecl::ExportAll(export_all) => {
                legacy::ModuleDecl::ExportAll(self.convert_export_all(export_all))
            }
        }
    }

    fn convert_import_decl<'a>(
        &mut self,
        import_decl: AstBox<'a, experimental::ImportDecl<'a>>,
    ) -> legacy::ImportDecl {
        let import_decl = AstBox::into_inner(import_decl);
        legacy::ImportDecl {
            span: convert_span(import_decl.span),
            specifiers: self.convert_vec(import_decl.specifiers, Self::convert_import_specifier),
            src: alloc_box!(self, self.convert_str(import_decl.src)),
            type_only: import_decl.type_only,
            with: import_decl
                .with
                .map(|with| alloc_box!(self, self.convert_object_lit(with))),
            phase: match import_decl.phase {
                experimental::ImportPhase::Evaluation => legacy::ImportPhase::Evaluation,
                experimental::ImportPhase::Source => legacy::ImportPhase::Source,
                experimental::ImportPhase::Defer => legacy::ImportPhase::Defer,
            },
        }
    }

    fn convert_export_decl<'a>(
        &mut self,
        export_decl: AstBox<'a, experimental::ExportDecl<'a>>,
    ) -> legacy::ExportDecl {
        let export_decl = AstBox::into_inner(export_decl);
        legacy::ExportDecl {
            span: convert_span(export_decl.span),
            decl: self.convert_decl(export_decl.decl),
        }
    }

    fn convert_export_named<'a>(
        &mut self,

        export_named: AstBox<'a, experimental::NamedExport<'a>>,
    ) -> legacy::NamedExport {
        let export_named = AstBox::into_inner(export_named);
        legacy::NamedExport {
            span: convert_span(export_named.span),
            specifiers: self.convert_vec(export_named.specifiers, Self::convert_export_specifier),
            src: export_named
                .src
                .map(|s| alloc_box!(self, self.convert_str(s))),
            type_only: export_named.type_only,
            with: export_named
                .with
                .map(|o| alloc_box!(self, self.convert_object_lit(o))),
        }
    }

    fn convert_export_default_decl<'a>(
        &mut self,

        export_default_decl: AstBox<'a, experimental::ExportDefaultDecl<'a>>,
    ) -> legacy::ExportDefaultDecl {
        let export_default_decl = AstBox::into_inner(export_default_decl);
        legacy::ExportDefaultDecl {
            span: convert_span(export_default_decl.span),
            decl: match export_default_decl.decl {
                experimental::DefaultDecl::Class(cls) => {
                    legacy::DefaultDecl::Class(self.convert_class_expr(cls))
                }
                experimental::DefaultDecl::Fn(f) => {
                    legacy::DefaultDecl::Fn(self.convert_fn_expr(f))
                }
            },
        }
    }

    fn convert_export_default_expr<'a>(
        &mut self,

        export_default_expr: AstBox<'a, experimental::ExportDefaultExpr<'a>>,
    ) -> legacy::ExportDefaultExpr {
        let export_default_expr = AstBox::into_inner(export_default_expr);
        legacy::ExportDefaultExpr {
            span: convert_span(export_default_expr.span),
            expr: self.convert_expr(export_default_expr.expr),
        }
    }

    fn convert_export_all<'a>(
        &mut self,
        export_all: AstBox<'a, experimental::ExportAll<'a>>,
    ) -> legacy::ExportAll {
        let export_all = AstBox::into_inner(export_all);
        legacy::ExportAll {
            span: convert_span(export_all.span),
            src: alloc_box!(self, self.convert_str(export_all.src)),
            type_only: export_all.type_only,
            with: export_all
                .with
                .map(|o| alloc_box!(self, self.convert_object_lit(o))),
        }
    }

    fn convert_stmt<'a>(&mut self, stmt: experimental::Stmt<'a>) -> legacy::Stmt {
        match stmt {
            experimental::Stmt::Block(block_stmt) => {
                legacy::Stmt::Block(self.convert_block_stmt(block_stmt))
            }
            experimental::Stmt::Empty(empty_stmt) => {
                legacy::Stmt::Empty(self.convert_empty_stmt(empty_stmt))
            }
            experimental::Stmt::Debugger(debugger_stmt) => {
                let debugger_stmt = AstBox::into_inner(debugger_stmt);
                legacy::Stmt::Debugger(legacy::DebuggerStmt {
                    span: convert_span(debugger_stmt.span),
                })
            }
            experimental::Stmt::With(with_stmt) => {
                let with_stmt = AstBox::into_inner(with_stmt);
                legacy::Stmt::With(legacy::WithStmt {
                    span: convert_span(with_stmt.span),
                    obj: self.convert_expr(with_stmt.obj),
                    body: alloc_box!(self, self.convert_stmt(with_stmt.body)),
                })
            }
            experimental::Stmt::Return(return_stmt) => {
                let return_stmt = AstBox::into_inner(return_stmt);
                legacy::Stmt::Return(legacy::ReturnStmt {
                    span: convert_span(return_stmt.span),
                    arg: return_stmt.arg.map(|arg| self.convert_expr(arg)),
                })
            }
            experimental::Stmt::Labeled(labeled_stmt) => {
                let labeled_stmt = AstBox::into_inner(labeled_stmt);
                legacy::Stmt::Labeled(legacy::LabeledStmt {
                    span: convert_span(labeled_stmt.span),
                    label: self.convert_ident(labeled_stmt.label),
                    body: alloc_box!(self, self.convert_stmt(labeled_stmt.body)),
                })
            }
            experimental::Stmt::Break(break_stmt) => {
                let break_stmt = AstBox::into_inner(break_stmt);
                legacy::Stmt::Break(legacy::BreakStmt {
                    span: convert_span(break_stmt.span),
                    label: break_stmt.label.map(|label| self.convert_ident(label)),
                })
            }
            experimental::Stmt::Continue(continue_stmt) => {
                let continue_stmt = AstBox::into_inner(continue_stmt);
                legacy::Stmt::Continue(legacy::ContinueStmt {
                    span: convert_span(continue_stmt.span),
                    label: continue_stmt.label.map(|label| self.convert_ident(label)),
                })
            }
            experimental::Stmt::If(if_stmt) => {
                let if_stmt = AstBox::into_inner(if_stmt);
                legacy::Stmt::If(legacy::IfStmt {
                    span: convert_span(if_stmt.span),
                    test: self.convert_expr(if_stmt.test),
                    cons: alloc_box!(self, self.convert_stmt(if_stmt.cons)),
                    alt: if_stmt
                        .alt
                        .map(|alt| alloc_box!(self, self.convert_stmt(alt))),
                })
            }
            experimental::Stmt::Switch(switch_stmt) => {
                let switch_stmt = AstBox::into_inner(switch_stmt);
                legacy::Stmt::Switch(legacy::SwitchStmt {
                    span: convert_span(switch_stmt.span),
                    discriminant: self.convert_expr(switch_stmt.discriminant),
                    cases: self.convert_vec(switch_stmt.cases, Self::convert_switch_case),
                })
            }
            experimental::Stmt::Throw(throw_stmt) => {
                let throw_stmt = AstBox::into_inner(throw_stmt);
                legacy::Stmt::Throw(legacy::ThrowStmt {
                    span: convert_span(throw_stmt.span),
                    arg: self.convert_expr(throw_stmt.arg),
                })
            }
            experimental::Stmt::Try(try_stmt) => {
                let try_stmt = AstBox::into_inner(try_stmt);
                legacy::Stmt::Try(alloc_box!(
                    self,
                    legacy::TryStmt {
                        span: convert_span(try_stmt.span),
                        block: self.convert_block_stmt(try_stmt.block),
                        handler: try_stmt
                            .handler
                            .map(|handler| self.convert_catch_clause(handler)),
                        finalizer: try_stmt
                            .finalizer
                            .map(|finalizer| self.convert_block_stmt(finalizer)),
                    }
                ))
            }
            experimental::Stmt::While(while_stmt) => {
                let while_stmt = AstBox::into_inner(while_stmt);
                legacy::Stmt::While(legacy::WhileStmt {
                    span: convert_span(while_stmt.span),
                    test: self.convert_expr(while_stmt.test),
                    body: alloc_box!(self, self.convert_stmt(while_stmt.body)),
                })
            }
            experimental::Stmt::DoWhile(do_while_stmt) => {
                let do_while_stmt = AstBox::into_inner(do_while_stmt);
                legacy::Stmt::DoWhile(legacy::DoWhileStmt {
                    span: convert_span(do_while_stmt.span),
                    test: self.convert_expr(do_while_stmt.test),
                    body: alloc_box!(self, self.convert_stmt(do_while_stmt.body)),
                })
            }
            experimental::Stmt::For(for_stmt) => {
                let for_stmt = AstBox::into_inner(for_stmt);
                legacy::Stmt::For(legacy::ForStmt {
                    span: convert_span(for_stmt.span),
                    init: for_stmt.init.map(|i| self.convert_var_decl_or_expr(i)),
                    test: for_stmt.test.map(|e| self.convert_expr(e)),
                    update: for_stmt.update.map(|e| self.convert_expr(e)),
                    body: alloc_box!(self, self.convert_stmt(for_stmt.body)),
                })
            }
            experimental::Stmt::ForIn(for_in_stmt) => {
                let for_in_stmt = AstBox::into_inner(for_in_stmt);
                legacy::Stmt::ForIn(legacy::ForInStmt {
                    span: convert_span(for_in_stmt.span),
                    left: self.convert_for_head(for_in_stmt.left),
                    right: self.convert_expr(for_in_stmt.right),
                    body: alloc_box!(self, self.convert_stmt(for_in_stmt.body)),
                })
            }
            experimental::Stmt::ForOf(for_of_stmt) => {
                let for_of_stmt = AstBox::into_inner(for_of_stmt);
                legacy::Stmt::ForOf(legacy::ForOfStmt {
                    span: convert_span(for_of_stmt.span),
                    is_await: for_of_stmt.is_await,
                    left: self.convert_for_head(for_of_stmt.left),
                    right: self.convert_expr(for_of_stmt.right),
                    body: alloc_box!(self, self.convert_stmt(for_of_stmt.body)),
                })
            }
            experimental::Stmt::Decl(decl) => {
                legacy::Stmt::Decl(self.convert_decl(AstBox::into_inner(decl)))
            }
            experimental::Stmt::Expr(expr_stmt) => {
                let expr_stmt = AstBox::into_inner(expr_stmt);
                legacy::Stmt::Expr(legacy::ExprStmt {
                    span: convert_span(expr_stmt.span),
                    expr: self.convert_expr(expr_stmt.expr),
                })
            }
        }
    }

    fn convert_block_stmt<'a>(
        &mut self,
        block_stmt: AstBox<'a, experimental::BlockStmt<'a>>,
    ) -> legacy::BlockStmt {
        let block_stmt = AstBox::into_inner(block_stmt);
        let ctxt = block_stmt
            .scope_id
            .get()
            .unwrap_or_else(|| self.semantic().top_level_scope_id());
        let ctxt = SyntaxContext::from_u32(ctxt.raw());
        legacy::BlockStmt {
            span: convert_span(block_stmt.span),
            stmts: self.convert_vec(block_stmt.stmts, Self::convert_stmt),
            ctxt,
        }
    }

    fn convert_empty_stmt(
        &mut self,
        empty_stmt: AstBox<'_, experimental::EmptyStmt>,
    ) -> legacy::EmptyStmt {
        let empty_stmt = AstBox::into_inner(empty_stmt);
        legacy::EmptyStmt {
            span: convert_span(empty_stmt.span),
        }
    }

    fn convert_expr<'a>(&mut self, expr: experimental::Expr<'a>) -> Box<legacy::Expr> {
        alloc_box!(
            self,
            match expr {
                experimental::Expr::This(t) => {
                    let t = AstBox::into_inner(t);
                    legacy::Expr::This(legacy::ThisExpr {
                        span: convert_span(t.span),
                    })
                }
                experimental::Expr::Array(a) => {
                    let a = AstBox::into_inner(a);
                    legacy::Expr::Array(legacy::ArrayLit {
                        span: convert_span(a.span),
                        elems: self.convert_vec(a.elems, |this, e| {
                            e.map(|e| this.convert_expr_or_spread(AstBox::into_inner(e)))
                        }),
                    })
                }
                experimental::Expr::Object(o) => legacy::Expr::Object(self.convert_object_lit(o)),
                experimental::Expr::Fn(f) => legacy::Expr::Fn(self.convert_fn_expr(f)),
                experimental::Expr::Unary(u) => {
                    let u = AstBox::into_inner(u);
                    legacy::Expr::Unary(legacy::UnaryExpr {
                        span: convert_span(u.span),
                        op: match u.op {
                            experimental::UnaryOp::Minus => legacy::UnaryOp::Minus,
                            experimental::UnaryOp::Plus => legacy::UnaryOp::Plus,
                            experimental::UnaryOp::Bang => legacy::UnaryOp::Bang,
                            experimental::UnaryOp::Tilde => legacy::UnaryOp::Tilde,
                            experimental::UnaryOp::TypeOf => legacy::UnaryOp::TypeOf,
                            experimental::UnaryOp::Void => legacy::UnaryOp::Void,
                            experimental::UnaryOp::Delete => legacy::UnaryOp::Delete,
                        },
                        arg: self.convert_expr(u.arg),
                    })
                }
                experimental::Expr::Update(u) => {
                    let u = AstBox::into_inner(u);
                    legacy::Expr::Update(legacy::UpdateExpr {
                        span: convert_span(u.span),
                        op: match u.op {
                            experimental::UpdateOp::PlusPlus => legacy::UpdateOp::PlusPlus,
                            experimental::UpdateOp::MinusMinus => legacy::UpdateOp::MinusMinus,
                        },
                        prefix: u.prefix,
                        arg: self.convert_simple_assign_target(u.arg).into(),
                    })
                }
                experimental::Expr::Bin(b) => {
                    let b = AstBox::into_inner(b);
                    legacy::Expr::Bin(legacy::BinExpr {
                        span: convert_span(b.span),
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
                        left: self.convert_expr(b.left),
                        right: self.convert_expr(b.right),
                    })
                }
                experimental::Expr::Assign(a) => {
                    let a = AstBox::into_inner(a);
                    legacy::Expr::Assign(legacy::AssignExpr {
                        span: convert_span(a.span),
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
                        left: self.convert_assign_target(a.left),
                        right: self.convert_expr(a.right),
                    })
                }
                experimental::Expr::Member(m) => {
                    let m = AstBox::into_inner(m);
                    legacy::Expr::Member(legacy::MemberExpr {
                        span: convert_span(m.span),
                        obj: self.convert_expr(m.obj),
                        prop: self.convert_member_prop(m.prop),
                    })
                }
                experimental::Expr::SuperProp(s) => {
                    let s = AstBox::into_inner(s);
                    legacy::Expr::SuperProp(legacy::SuperPropExpr {
                        span: convert_span(s.span),
                        obj: legacy::Super {
                            span: convert_span(s.obj.span),
                        },
                        prop: match s.prop {
                            experimental::SuperProp::Ident(i) => {
                                let i = AstBox::into_inner(i);
                                legacy::SuperProp::Ident(legacy::IdentName {
                                    span: convert_span(i.span),
                                    sym: self.convert_utf8_ref(i.sym),
                                })
                            }
                            experimental::SuperProp::Computed(c) => {
                                let c = AstBox::into_inner(c);
                                legacy::SuperProp::Computed(legacy::ComputedPropName {
                                    span: convert_span(c.span),
                                    expr: self.convert_expr(c.expr),
                                })
                            }
                        },
                    })
                }
                experimental::Expr::Cond(c) => {
                    let c = AstBox::into_inner(c);
                    legacy::Expr::Cond(legacy::CondExpr {
                        span: convert_span(c.span),
                        test: self.convert_expr(c.test),
                        cons: self.convert_expr(c.cons),
                        alt: self.convert_expr(c.alt),
                    })
                }
                experimental::Expr::Import(i) => {
                    let i = AstBox::into_inner(i);
                    let mut args =
                        std::vec::Vec::with_capacity(if i.options.is_some() { 2 } else { 1 });
                    args.push(legacy::ExprOrSpread {
                        spread: None,
                        expr: self.convert_expr(i.source),
                    });
                    if let Some(options) = i.options {
                        args.push(legacy::ExprOrSpread {
                            spread: None,
                            expr: self.convert_expr(options),
                        });
                    }
                    let callee_len = match i.phase {
                        experimental::ImportPhase::Evaluation => "import".len(),
                        experimental::ImportPhase::Source => "import.source".len(),
                        experimental::ImportPhase::Defer => "import.defer".len(),
                    } as u32;

                    legacy::Expr::Call(legacy::CallExpr {
                        span: convert_span(i.span),
                        ctxt: Default::default(),
                        callee: legacy::Callee::Import(legacy::Import {
                            // The experimental AST does not retain a separate span for the
                            // `import` callee.
                            span: convert_span(experimental::Span {
                                start: i.span.start,
                                end: i.span.end.min(i.span.start.saturating_add(callee_len)),
                            }),
                            phase: match i.phase {
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
                experimental::Expr::Call(c) => {
                    let c = AstBox::into_inner(c);
                    legacy::Expr::Call(legacy::CallExpr {
                        span: convert_span(c.span),
                        ctxt: Default::default(),
                        callee: match c.callee {
                            experimental::Callee::Super(s) => {
                                legacy::Callee::Super(legacy::Super {
                                    span: convert_span(s.span),
                                })
                            }
                            experimental::Callee::Expr(e) => {
                                legacy::Callee::Expr(self.convert_expr(AstBox::into_inner(e)))
                            }
                        },
                        args: self.convert_vec(c.args, Self::convert_expr_or_spread),
                        type_args: None,
                    })
                }
                experimental::Expr::New(n) => {
                    let n = AstBox::into_inner(n);
                    legacy::Expr::New(legacy::NewExpr {
                        span: convert_span(n.span),
                        ctxt: Default::default(),
                        callee: self.convert_expr(n.callee),
                        args: Some(self.convert_vec(n.args, Self::convert_expr_or_spread)),
                        type_args: None,
                    })
                }
                experimental::Expr::Seq(s) => {
                    let s = AstBox::into_inner(s);
                    legacy::Expr::Seq(legacy::SeqExpr {
                        span: convert_span(s.span),
                        exprs: self.convert_vec(s.exprs, Self::convert_expr),
                    })
                }
                experimental::Expr::Ident(i) => legacy::Expr::Ident(self.convert_ident(i)),
                experimental::Expr::Lit(l) => legacy::Expr::Lit(self.convert_lit(l)),
                experimental::Expr::Tpl(t) => {
                    let t = AstBox::into_inner(t);
                    legacy::Expr::Tpl(legacy::Tpl {
                        span: convert_span(t.span),
                        exprs: self.convert_vec(t.exprs, Self::convert_expr),
                        quasis: self.convert_vec(t.quasis, Self::convert_tpl_element),
                    })
                }
                experimental::Expr::TaggedTpl(tt) => {
                    let tt = AstBox::into_inner(tt);
                    legacy::Expr::TaggedTpl(legacy::TaggedTpl {
                        span: convert_span(tt.span),
                        ctxt: Default::default(),
                        tag: self.convert_expr(tt.tag),
                        tpl: alloc_box!(self, self.convert_tpl(AstBox::into_inner(tt.tpl))),
                        type_params: None,
                    })
                }
                experimental::Expr::Arrow(a) => {
                    let a = AstBox::into_inner(a);
                    legacy::Expr::Arrow(legacy::ArrowExpr {
                        span: convert_span(a.span),
                        ctxt: Default::default(),
                        params: self.convert_vec(a.params, Self::convert_pat),
                        body: alloc_box!(
                            self,
                            match a.body {
                                experimental::BlockStmtOrExpr::BlockStmt(b) => {
                                    legacy::BlockStmtOrExpr::BlockStmt(self.convert_block_stmt(b))
                                }
                                experimental::BlockStmtOrExpr::Expr(e) => {
                                    legacy::BlockStmtOrExpr::Expr(
                                        self.convert_expr(AstBox::into_inner(e)),
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
                experimental::Expr::Class(c) => legacy::Expr::Class(self.convert_class_expr(c)),
                experimental::Expr::Yield(y) => {
                    let y = AstBox::into_inner(y);
                    legacy::Expr::Yield(legacy::YieldExpr {
                        span: convert_span(y.span),
                        arg: y.arg.map(|e| self.convert_expr(e)),
                        delegate: y.delegate,
                    })
                }
                experimental::Expr::MetaProp(m) => {
                    let m = AstBox::into_inner(m);
                    legacy::Expr::MetaProp(legacy::MetaPropExpr {
                        span: convert_span(m.span),
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
                        span: convert_span(a.span),
                        arg: self.convert_expr(a.arg),
                    })
                }
                experimental::Expr::Paren(p) => {
                    let p = AstBox::into_inner(p);
                    legacy::Expr::Paren(legacy::ParenExpr {
                        span: convert_span(p.span),
                        expr: self.convert_expr(p.expr),
                    })
                }
                experimental::Expr::PrivateName(p) => {
                    let p = AstBox::into_inner(p);
                    legacy::Expr::PrivateName(legacy::PrivateName {
                        span: convert_span(p.span),
                        name: self.convert_utf8_ref(p.name),
                    })
                }
                experimental::Expr::OptChain(o) => {
                    let o = AstBox::into_inner(o);
                    legacy::Expr::OptChain(legacy::OptChainExpr {
                        span: convert_span(o.span),
                        optional: o.optional,
                        base: alloc_box!(
                            self,
                            match o.base {
                                experimental::OptChainBase::Member(m) => {
                                    let m = AstBox::into_inner(m);
                                    legacy::OptChainBase::Member(legacy::MemberExpr {
                                        span: convert_span(m.span),
                                        obj: self.convert_expr(m.obj),
                                        prop: self.convert_member_prop(m.prop),
                                    })
                                }
                                experimental::OptChainBase::Call(c) => {
                                    let c = AstBox::into_inner(c);
                                    legacy::OptChainBase::Call(legacy::OptCall {
                                        span: convert_span(c.span),
                                        ctxt: Default::default(),
                                        callee: self.convert_expr(c.callee),
                                        args: self
                                            .convert_vec(c.args, Self::convert_expr_or_spread),
                                        type_args: None,
                                    })
                                }
                            }
                        ),
                    })
                }
                experimental::Expr::JSXMember(j) => {
                    legacy::Expr::JSXMember(self.convert_jsx_member_expr(j))
                }
                experimental::Expr::JSXNamespacedName(j) => {
                    legacy::Expr::JSXNamespacedName(self.convert_jsx_namespaced_name(j))
                }
                experimental::Expr::JSXEmpty(j) => {
                    legacy::Expr::JSXEmpty(self.convert_jsx_empty_expr(j))
                }
                experimental::Expr::JSXElement(j) => {
                    legacy::Expr::JSXElement(alloc_box!(self, self.convert_jsx_element(j)))
                }
                experimental::Expr::JSXFragment(j) => {
                    legacy::Expr::JSXFragment(self.convert_jsx_fragment(j))
                }
                experimental::Expr::Invalid(_) => {
                    legacy::Expr::Invalid(legacy::Invalid {
                        span: Default::default(),
                    })
                }
            }
        )
    }

    fn convert_ident(&mut self, ident: AstBox<'_, experimental::Ident>) -> legacy::Ident {
        let ident = AstBox::into_inner(ident);
        let ctxt = ident
            .symbol_id
            .get()
            .map(|_| self.semantic().node_scope(&ident))
            .unwrap_or_else(|| self.semantic().unresolved_scope_id());
        legacy::Ident {
            span: convert_span(ident.span),
            ctxt: SyntaxContext::from_u32(ctxt.raw()),
            sym: self.convert_utf8_ref(ident.sym),
            optional: false,
        }
    }

    fn convert_switch_case<'a>(&mut self, c: experimental::SwitchCase<'a>) -> legacy::SwitchCase {
        legacy::SwitchCase {
            span: convert_span(c.span),
            test: c.test.map(|e| self.convert_expr(e)),
            cons: self.convert_vec(c.cons, Self::convert_stmt),
        }
    }

    fn convert_catch_clause<'a>(
        &mut self,
        c: AstBox<'a, experimental::CatchClause<'a>>,
    ) -> legacy::CatchClause {
        let c = AstBox::into_inner(c);
        legacy::CatchClause {
            span: convert_span(c.span),
            param: c.param.map(|p| self.convert_pat(p)),
            body: self.convert_block_stmt(c.body),
        }
    }

    fn convert_var_decl_or_expr<'a>(
        &mut self,
        v: experimental::VarDeclOrExpr<'a>,
    ) -> legacy::VarDeclOrExpr {
        match v {
            experimental::VarDeclOrExpr::VarDecl(d) => {
                legacy::VarDeclOrExpr::VarDecl(alloc_box!(self, self.convert_var_decl(d)))
            }
            experimental::VarDeclOrExpr::Expr(e) => {
                legacy::VarDeclOrExpr::Expr(self.convert_expr(AstBox::into_inner(e)))
            }
        }
    }

    fn convert_var_decl<'a>(
        &mut self,
        v: AstBox<'a, experimental::VarDecl<'a>>,
    ) -> legacy::VarDecl {
        let v = AstBox::into_inner(v);
        legacy::VarDecl {
            span: convert_span(v.span),
            ctxt: Default::default(),
            kind: match v.kind {
                experimental::VarDeclKind::Var => legacy::VarDeclKind::Var,
                experimental::VarDeclKind::Let => legacy::VarDeclKind::Let,
                experimental::VarDeclKind::Const => legacy::VarDeclKind::Const,
            },
            declare: v.declare,
            decls: self.convert_vec(v.decls, Self::convert_var_declarator),
        }
    }

    fn convert_for_head<'a>(&mut self, h: experimental::ForHead<'a>) -> legacy::ForHead {
        match h {
            experimental::ForHead::VarDecl(v) => {
                legacy::ForHead::VarDecl(alloc_box!(self, self.convert_var_decl(v)))
            }
            experimental::ForHead::UsingDecl(u) => {
                let u = AstBox::into_inner(u);
                legacy::ForHead::UsingDecl(alloc_box!(
                    self,
                    legacy::UsingDecl {
                        span: convert_span(u.span),
                        is_await: u.is_await,
                        decls: self.convert_vec(u.decls, Self::convert_var_declarator),
                    }
                ))
            }
            experimental::ForHead::Pat(p) => {
                legacy::ForHead::Pat(alloc_box!(self, self.convert_pat(AstBox::into_inner(p))))
            }
        }
    }

    // -------------------------------------------------------------------------------
    // Helpers for module declarations and common nodes

    fn convert_import_specifier<'a>(
        &mut self,

        s: experimental::ImportSpecifier<'a>,
    ) -> legacy::ImportSpecifier {
        match s {
            experimental::ImportSpecifier::Named(n) => {
                let n = AstBox::into_inner(n);
                let imported = if n.is_shorthand() {
                    None
                } else {
                    Some(self.convert_module_export_name(n.imported))
                };

                legacy::ImportSpecifier::Named(legacy::ImportNamedSpecifier {
                    span: convert_span(n.span),
                    local: self.convert_ident(n.local),
                    imported,
                    is_type_only: n.is_type_only,
                })
            }
            experimental::ImportSpecifier::Default(d) => {
                let d = AstBox::into_inner(d);
                legacy::ImportSpecifier::Default(legacy::ImportDefaultSpecifier {
                    span: convert_span(d.span),
                    local: self.convert_ident(d.local),
                })
            }
            experimental::ImportSpecifier::Namespace(ns) => {
                let ns = AstBox::into_inner(ns);
                legacy::ImportSpecifier::Namespace(legacy::ImportStarAsSpecifier {
                    span: convert_span(ns.span),
                    local: self.convert_ident(ns.local),
                })
            }
        }
    }

    fn convert_export_specifier<'a>(
        &mut self,

        s: experimental::ExportSpecifier<'a>,
    ) -> legacy::ExportSpecifier {
        match s {
            experimental::ExportSpecifier::Namespace(ns) => {
                let ns = AstBox::into_inner(ns);
                legacy::ExportSpecifier::Namespace(legacy::ExportNamespaceSpecifier {
                    span: convert_span(ns.span),
                    name: self.convert_module_export_name(ns.name),
                })
            }
            experimental::ExportSpecifier::Default(d) => {
                let d = AstBox::into_inner(d);
                legacy::ExportSpecifier::Default(legacy::ExportDefaultSpecifier {
                    exported: self.convert_ident(d.exported),
                })
            }
            experimental::ExportSpecifier::Named(n) => {
                let n = AstBox::into_inner(n);
                let exported = if n.is_shorthand() {
                    None
                } else {
                    Some(self.convert_module_export_name(n.exported))
                };
                legacy::ExportSpecifier::Named(legacy::ExportNamedSpecifier {
                    span: convert_span(n.span),
                    orig: self.convert_module_export_name(n.orig),
                    exported,
                    is_type_only: n.is_type_only,
                })
            }
        }
    }

    fn convert_module_export_name<'a>(
        &mut self,

        n: experimental::ModuleExportName<'a>,
    ) -> legacy::ModuleExportName {
        match n {
            experimental::ModuleExportName::Ident(i) => {
                legacy::ModuleExportName::Ident(self.convert_ident(i))
            }
            experimental::ModuleExportName::Str(s) => {
                legacy::ModuleExportName::Str(self.convert_str(s))
            }
        }
    }

    fn convert_object_lit<'a>(
        &mut self,
        o: AstBox<'a, experimental::ObjectLit<'a>>,
    ) -> legacy::ObjectLit {
        let o = AstBox::into_inner(o);
        legacy::ObjectLit {
            span: convert_span(o.span),
            props: self.convert_vec(o.props, Self::convert_prop_or_spread),
        }
    }

    fn convert_prop_or_spread<'a>(
        &mut self,
        p: experimental::PropOrSpread<'a>,
    ) -> legacy::PropOrSpread {
        match p {
            experimental::PropOrSpread::Spread(s) => {
                let s = AstBox::into_inner(s);
                legacy::PropOrSpread::Spread(legacy::SpreadElement {
                    dot3_token: convert_span(s.dot3_token),
                    expr: self.convert_expr(s.expr),
                })
            }
            experimental::PropOrSpread::Prop(prop) => {
                legacy::PropOrSpread::Prop(alloc_box!(self, self.convert_prop(prop)))
            }
        }
    }

    fn convert_prop<'a>(&mut self, p: AstBox<'a, experimental::Prop<'a>>) -> legacy::Prop {
        let p = AstBox::into_inner(p);
        match p {
            experimental::Prop::Shorthand(i) => legacy::Prop::Shorthand(self.convert_ident(i)),
            experimental::Prop::KeyValue(kv) => {
                let kv = AstBox::into_inner(kv);
                legacy::Prop::KeyValue(legacy::KeyValueProp {
                    key: self.convert_prop_name(kv.key),
                    value: self.convert_expr(kv.value),
                })
            }
            experimental::Prop::Assign(ap) => {
                let ap = AstBox::into_inner(ap);
                legacy::Prop::Assign(legacy::AssignProp {
                    span: convert_span(ap.span),
                    key: self.convert_ident(ap.key),
                    value: self.convert_expr(ap.value),
                })
            }
            experimental::Prop::Getter(g) => {
                let g = AstBox::into_inner(g);
                let function = AstBox::into_inner(g.function);
                legacy::Prop::Getter(legacy::GetterProp {
                    span: convert_span(g.span),
                    key: self.convert_prop_name(g.key),
                    type_ann: None,
                    body: Some(self.convert_block_stmt(function.body)),
                })
            }
            experimental::Prop::Setter(s) => {
                let s = AstBox::into_inner(s);
                let function = AstBox::into_inner(s.function);
                let function_span = function.span;
                let mut params = function.params;
                let this_param = if params.len() >= 2 {
                    Some(self.convert_pat(params.remove(0).pat))
                } else {
                    None
                };
                let param = params
                    .into_iter()
                    .next()
                    .map(|p| alloc_box!(self, self.convert_pat(p.pat)))
                    .unwrap_or_else(|| {
                        alloc_box!(
                            self,
                            legacy::Pat::Invalid(legacy::Invalid {
                                span: convert_span(function_span),
                            })
                        )
                    });
                legacy::Prop::Setter(legacy::SetterProp {
                    span: convert_span(s.span),
                    key: self.convert_prop_name(s.key),
                    this_param,
                    param,
                    body: Some(self.convert_block_stmt(function.body)),
                })
            }
            experimental::Prop::Method(m) => {
                let m = AstBox::into_inner(m);
                legacy::Prop::Method(legacy::MethodProp {
                    key: self.convert_prop_name(m.key),
                    function: alloc_box!(self, self.convert_function(m.function)),
                })
            }
        }
    }

    fn convert_prop_name<'a>(&mut self, n: experimental::PropName<'a>) -> legacy::PropName {
        match n {
            experimental::PropName::Ident(i) => {
                let i = AstBox::into_inner(i);
                legacy::PropName::Ident(legacy::IdentName {
                    span: convert_span(i.span),
                    sym: self.convert_utf8_ref(i.sym),
                })
            }
            experimental::PropName::Str(s) => legacy::PropName::Str(self.convert_str(s)),
            experimental::PropName::Num(n) => {
                let n = AstBox::into_inner(n);
                legacy::PropName::Num(legacy::Number {
                    span: convert_span(n.span),
                    value: n.value,
                    raw: self.convert_opt_utf8_ref(n.raw),
                })
            }
            experimental::PropName::Computed(c) => {
                let c = AstBox::into_inner(c);
                legacy::PropName::Computed(legacy::ComputedPropName {
                    span: convert_span(c.span),
                    expr: self.convert_expr(c.expr),
                })
            }
            experimental::PropName::BigInt(b) => {
                let b = AstBox::into_inner(b);
                legacy::PropName::BigInt(legacy::BigInt {
                    span: convert_span(b.span),
                    value: alloc_box!(self, self.convert_big_int(b.value)),
                    raw: self.convert_opt_utf8_ref(b.raw),
                })
            }
        }
    }

    fn convert_str(&mut self, s: AstBox<'_, experimental::Str>) -> legacy::Str {
        let s = AstBox::into_inner(s);
        legacy::Str {
            span: convert_span(s.span),
            value: self.convert_wtf8_ref(s.value),
            raw: self.convert_opt_utf8_ref(s.raw),
        }
    }

    // -------------------------------------------------------------------------------
    // Function / Class basics used by export default

    fn convert_fn_expr<'a>(&mut self, f: AstBox<'a, experimental::FnExpr<'a>>) -> legacy::FnExpr {
        let f = AstBox::into_inner(f);
        legacy::FnExpr {
            ident: f.ident.map(|i| self.convert_ident(i)),
            function: alloc_box!(self, self.convert_function(f.function)),
        }
    }

    fn convert_class_expr<'a>(
        &mut self,
        c: AstBox<'a, experimental::ClassExpr<'a>>,
    ) -> legacy::ClassExpr {
        let c = AstBox::into_inner(c);
        legacy::ClassExpr {
            ident: c.ident.map(|i| self.convert_ident(i)),
            class: alloc_box!(self, self.convert_class(c.class)),
        }
    }

    fn convert_function<'a>(
        &mut self,
        f: AstBox<'a, experimental::Function<'a>>,
    ) -> legacy::Function {
        let f = AstBox::into_inner(f);
        legacy::Function {
            params: self.convert_vec(f.params, Self::convert_param),
            decorators: self.convert_vec(f.decorators, Self::convert_decorator),
            span: convert_span(f.span),
            ctxt: Default::default(),
            body: Some(self.convert_block_stmt(f.body)),
            is_generator: f.is_generator,
            is_async: f.is_async,
            type_params: None,
            return_type: None,
        }
    }

    fn convert_param<'a>(&mut self, p: experimental::Param<'a>) -> legacy::Param {
        legacy::Param {
            span: convert_span(p.span),
            decorators: self.convert_vec(p.decorators, Self::convert_decorator),
            pat: self.convert_pat(p.pat),
        }
    }

    fn convert_decorator<'a>(&mut self, d: experimental::Decorator<'a>) -> legacy::Decorator {
        legacy::Decorator {
            span: convert_span(d.span),
            expr: self.convert_expr(d.expr),
        }
    }

    fn convert_class<'a>(&mut self, c: AstBox<'a, experimental::Class<'a>>) -> legacy::Class {
        let c = AstBox::into_inner(c);
        legacy::Class {
            span: convert_span(c.span),
            ctxt: Default::default(),
            decorators: self.convert_vec(c.decorators, Self::convert_decorator),
            body: self.convert_vec(c.body, Self::convert_class_member),
            super_class: c.super_class.map(|e| self.convert_expr(e)),
            is_abstract: c.is_abstract,
            type_params: None,
            super_type_params: None,
            implements: Default::default(),
        }
    }

    fn convert_class_member<'a>(
        &mut self,
        m: experimental::ClassMember<'a>,
    ) -> legacy::ClassMember {
        match m {
            experimental::ClassMember::Constructor(k) => {
                let k = AstBox::into_inner(k);
                legacy::ClassMember::Constructor(legacy::Constructor {
                    span: convert_span(k.span),
                    ctxt: Default::default(),
                    key: self.convert_prop_name(k.key),
                    params: self.convert_vec(k.params, Self::convert_param_or_ts_param_prop),
                    body: k.body.map(|b| self.convert_block_stmt(b)),
                    accessibility: None,
                    is_optional: false,
                })
            }
            experimental::ClassMember::Method(me) => {
                let me = AstBox::into_inner(me);
                legacy::ClassMember::Method(legacy::ClassMethod {
                    span: convert_span(me.span),
                    key: self.convert_prop_name(me.key),
                    function: alloc_box!(self, self.convert_function(me.function)),
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
                    span: convert_span(pm.span),
                    key: legacy::PrivateName {
                        span: convert_span(key.span),
                        name: self.convert_utf8_ref(key.name),
                    },
                    function: alloc_box!(self, self.convert_function(pm.function)),
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
                    span: convert_span(cp.span),
                    key: self.convert_prop_name(cp.key),
                    value: cp.value.map(|e| self.convert_expr(e)),
                    type_ann: None,
                    is_static: cp.is_static,
                    decorators: self.convert_vec(cp.decorators, Self::convert_decorator),
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
                    span: convert_span(pp.span),
                    ctxt: Default::default(),
                    key: legacy::PrivateName {
                        span: convert_span(key.span),
                        name: self.convert_utf8_ref(key.name),
                    },
                    value: pp.value.map(|e| self.convert_expr(e)),
                    type_ann: None,
                    is_static: pp.is_static,
                    decorators: self.convert_vec(pp.decorators, Self::convert_decorator),
                    accessibility: None,
                    is_optional: false,
                    is_override: false,
                    readonly: false,
                    definite: false,
                })
            }
            experimental::ClassMember::Empty(e) => {
                legacy::ClassMember::Empty(self.convert_empty_stmt(e))
            }
            experimental::ClassMember::StaticBlock(sb) => {
                let sb = AstBox::into_inner(sb);
                legacy::ClassMember::StaticBlock(legacy::StaticBlock {
                    span: convert_span(sb.span),
                    body: self.convert_block_stmt(sb.body),
                })
            }
            experimental::ClassMember::AutoAccessor(a) => {
                let a = AstBox::into_inner(a);
                legacy::ClassMember::AutoAccessor(legacy::AutoAccessor {
                    span: convert_span(a.span),
                    key: match a.key {
                        experimental::Key::Private(p) => {
                            let p = AstBox::into_inner(p);
                            legacy::Key::Private(legacy::PrivateName {
                                span: convert_span(p.span),
                                name: self.convert_utf8_ref(p.name),
                            })
                        }
                        experimental::Key::Public(n) => {
                            legacy::Key::Public(self.convert_prop_name(AstBox::into_inner(n)))
                        }
                    },
                    value: a.value.map(|e| self.convert_expr(e)),
                    type_ann: None,
                    is_static: a.is_static,
                    decorators: self.convert_vec(a.decorators, Self::convert_decorator),
                    accessibility: None,
                    is_abstract: false,
                    is_override: false,
                    definite: false,
                })
            }
        }
    }

    fn convert_param_or_ts_param_prop<'a>(
        &mut self,

        p: experimental::ParamOrTsParamProp<'a>,
    ) -> legacy::ParamOrTsParamProp {
        match p {
            experimental::ParamOrTsParamProp::Param(pp) => {
                legacy::ParamOrTsParamProp::Param(self.convert_param(AstBox::into_inner(pp)))
            }
        }
    }

    // -------------------------------------------------------------------------------
    // Patterns and declarations

    fn convert_decl<'a>(&mut self, d: experimental::Decl<'a>) -> legacy::Decl {
        match d {
            experimental::Decl::Class(c) => {
                let c = AstBox::into_inner(c);
                legacy::Decl::Class(legacy::ClassDecl {
                    ident: self.convert_ident(c.ident),
                    declare: c.declare,
                    class: alloc_box!(self, self.convert_class(c.class)),
                })
            }
            experimental::Decl::Fn(f) => {
                let f = AstBox::into_inner(f);
                legacy::Decl::Fn(legacy::FnDecl {
                    ident: self.convert_ident(f.ident),
                    declare: f.declare,
                    function: alloc_box!(self, self.convert_function(f.function)),
                })
            }
            experimental::Decl::Var(v) => {
                legacy::Decl::Var(alloc_box!(self, self.convert_var_decl(v)))
            }
            experimental::Decl::Using(u) => {
                let u = AstBox::into_inner(u);
                legacy::Decl::Using(alloc_box!(
                    self,
                    legacy::UsingDecl {
                        span: convert_span(u.span),
                        is_await: u.is_await,
                        decls: self.convert_vec(u.decls, Self::convert_var_declarator),
                    }
                ))
            }
        }
    }

    fn convert_var_declarator<'a>(
        &mut self,
        d: experimental::VarDeclarator<'a>,
    ) -> legacy::VarDeclarator {
        legacy::VarDeclarator {
            span: convert_span(d.span),
            name: self.convert_pat(d.name),
            init: d.init.map(|e| self.convert_expr(e)),
            definite: false,
        }
    }

    fn convert_pat<'a>(&mut self, p: experimental::Pat<'a>) -> legacy::Pat {
        match p {
            experimental::Pat::Ident(b) => {
                let b = AstBox::into_inner(b);
                legacy::Pat::Ident(legacy::BindingIdent {
                    id: self.convert_ident(b.id),
                    type_ann: None,
                })
            }
            experimental::Pat::Array(a) => {
                let a = AstBox::into_inner(a);
                legacy::Pat::Array(legacy::ArrayPat {
                    span: convert_span(a.span),
                    elems: self.convert_vec(a.elems, |this, p| p.map(|p| this.convert_pat(p))),
                    optional: a.optional,
                    type_ann: None,
                })
            }
            experimental::Pat::Rest(r) => {
                let r = AstBox::into_inner(r);
                legacy::Pat::Rest(legacy::RestPat {
                    span: convert_span(r.span),
                    dot3_token: convert_span(r.dot3_token),
                    arg: alloc_box!(self, self.convert_pat(r.arg)),
                    type_ann: None,
                })
            }
            experimental::Pat::Object(o) => {
                let o = AstBox::into_inner(o);
                legacy::Pat::Object(legacy::ObjectPat {
                    span: convert_span(o.span),
                    props: self.convert_vec(o.props, Self::convert_object_pat_prop),
                    optional: o.optional,
                    type_ann: None,
                })
            }
            experimental::Pat::Assign(a) => {
                let a = AstBox::into_inner(a);
                legacy::Pat::Assign(legacy::AssignPat {
                    span: convert_span(a.span),
                    left: alloc_box!(self, self.convert_pat(a.left)),
                    right: self.convert_expr(a.right),
                })
            }
            experimental::Pat::Invalid(_) => legacy::Pat::Invalid(legacy::Invalid {
                span: Default::default(),
            }),
            experimental::Pat::Expr(e) => {
                legacy::Pat::Expr(self.convert_expr(AstBox::into_inner(e)))
            }
        }
    }

    fn convert_object_pat_prop<'a>(
        &mut self,
        p: experimental::ObjectPatProp<'a>,
    ) -> legacy::ObjectPatProp {
        match p {
            experimental::ObjectPatProp::KeyValue(kv) => {
                let kv = AstBox::into_inner(kv);
                legacy::ObjectPatProp::KeyValue(legacy::KeyValuePatProp {
                    key: self.convert_prop_name(kv.key),
                    value: alloc_box!(self, self.convert_pat(kv.value)),
                })
            }
            experimental::ObjectPatProp::Assign(ap) => {
                let ap = AstBox::into_inner(ap);
                let key = AstBox::into_inner(ap.key);
                legacy::ObjectPatProp::Assign(legacy::AssignPatProp {
                    span: convert_span(ap.span),
                    key: legacy::BindingIdent {
                        id: self.convert_ident(key.id),
                        type_ann: None,
                    },
                    value: ap.value.map(|e| self.convert_expr(e)),
                })
            }
            experimental::ObjectPatProp::Rest(r) => {
                let r = AstBox::into_inner(r);
                legacy::ObjectPatProp::Rest(legacy::RestPat {
                    span: convert_span(r.span),
                    dot3_token: convert_span(r.dot3_token),
                    arg: alloc_box!(self, self.convert_pat(r.arg)),
                    type_ann: None,
                })
            }
        }
    }
    fn convert_expr_or_spread<'a>(
        &mut self,
        e: experimental::ExprOrSpread<'a>,
    ) -> legacy::ExprOrSpread {
        legacy::ExprOrSpread {
            spread: e.spread.map(convert_span),
            expr: self.convert_expr(e.expr),
        }
    }

    fn convert_member_prop<'a>(&mut self, p: experimental::MemberProp<'a>) -> legacy::MemberProp {
        match p {
            experimental::MemberProp::Ident(i) => {
                let i = AstBox::into_inner(i);
                legacy::MemberProp::Ident(legacy::IdentName {
                    span: convert_span(i.span),
                    sym: self.convert_utf8_ref(i.sym),
                })
            }
            experimental::MemberProp::PrivateName(pn) => {
                let pn = AstBox::into_inner(pn);
                legacy::MemberProp::PrivateName(legacy::PrivateName {
                    span: convert_span(pn.span),
                    name: self.convert_utf8_ref(pn.name),
                })
            }
            experimental::MemberProp::Computed(c) => {
                let c = AstBox::into_inner(c);
                legacy::MemberProp::Computed(legacy::ComputedPropName {
                    span: convert_span(c.span),
                    expr: self.convert_expr(c.expr),
                })
            }
        }
    }

    fn convert_lit<'a>(&mut self, l: AstBox<'a, experimental::Lit<'a>>) -> legacy::Lit {
        let l = AstBox::into_inner(l);
        match l {
            experimental::Lit::Str(s) => legacy::Lit::Str(self.convert_str(s)),
            experimental::Lit::Bool(b) => {
                let b = AstBox::into_inner(b);
                legacy::Lit::Bool(legacy::Bool {
                    span: convert_span(b.span),
                    value: b.value,
                })
            }
            experimental::Lit::Null(n) => {
                let n = AstBox::into_inner(n);
                legacy::Lit::Null(legacy::Null {
                    span: convert_span(n.span),
                })
            }
            experimental::Lit::Num(n) => {
                let n = AstBox::into_inner(n);
                legacy::Lit::Num(legacy::Number {
                    span: convert_span(n.span),
                    value: n.value,
                    raw: self.convert_opt_utf8_ref(n.raw),
                })
            }
            experimental::Lit::BigInt(b) => {
                let b = AstBox::into_inner(b);
                legacy::Lit::BigInt(legacy::BigInt {
                    span: convert_span(b.span),
                    value: alloc_box!(self, self.convert_big_int(b.value)),
                    raw: self.convert_opt_utf8_ref(b.raw),
                })
            }
            experimental::Lit::Regex(r) => {
                let r = AstBox::into_inner(r);
                legacy::Lit::Regex(legacy::Regex {
                    span: convert_span(r.span),
                    exp: self.convert_utf8_ref(r.exp),
                    flags: self.convert_utf8_ref(r.flags),
                })
            }
        }
    }

    fn convert_tpl_element(&mut self, e: experimental::TplElement) -> legacy::TplElement {
        legacy::TplElement {
            span: convert_span(e.span),
            tail: e.tail,
            cooked: self.convert_opt_wtf8_ref(e.cooked),
            raw: self.convert_utf8_ref(e.raw),
        }
    }

    fn convert_tpl<'a>(&mut self, t: experimental::Tpl<'a>) -> legacy::Tpl {
        legacy::Tpl {
            span: convert_span(t.span),
            exprs: self.convert_vec(t.exprs, Self::convert_expr),
            quasis: self.convert_vec(t.quasis, Self::convert_tpl_element),
        }
    }

    fn convert_simple_assign_target<'a>(
        &mut self,
        t: experimental::SimpleAssignTarget<'a>,
    ) -> legacy::SimpleAssignTarget {
        match t {
            experimental::SimpleAssignTarget::Ident(b) => {
                let b = AstBox::into_inner(b);
                legacy::SimpleAssignTarget::Ident(legacy::BindingIdent {
                    id: self.convert_ident(b.id),
                    type_ann: None,
                })
            }
            experimental::SimpleAssignTarget::Member(m) => {
                let m = AstBox::into_inner(m);
                legacy::SimpleAssignTarget::Member(legacy::MemberExpr {
                    span: convert_span(m.span),
                    obj: self.convert_expr(m.obj),
                    prop: self.convert_member_prop(m.prop),
                })
            }
            experimental::SimpleAssignTarget::SuperProp(su) => {
                let su = AstBox::into_inner(su);
                legacy::SimpleAssignTarget::SuperProp(legacy::SuperPropExpr {
                    span: convert_span(su.span),
                    obj: legacy::Super {
                        span: convert_span(su.obj.span),
                    },
                    prop: match su.prop {
                        experimental::SuperProp::Ident(i) => {
                            let i = AstBox::into_inner(i);
                            legacy::SuperProp::Ident(legacy::IdentName {
                                span: convert_span(i.span),
                                sym: self.convert_utf8_ref(i.sym),
                            })
                        }
                        experimental::SuperProp::Computed(c) => {
                            let c = AstBox::into_inner(c);
                            legacy::SuperProp::Computed(legacy::ComputedPropName {
                                span: convert_span(c.span),
                                expr: self.convert_expr(c.expr),
                            })
                        }
                    },
                })
            }
            experimental::SimpleAssignTarget::Paren(p) => {
                let p = AstBox::into_inner(p);
                legacy::SimpleAssignTarget::Paren(legacy::ParenExpr {
                    span: convert_span(p.span),
                    expr: self.convert_expr(p.expr),
                })
            }
            experimental::SimpleAssignTarget::OptChain(o) => {
                let o = AstBox::into_inner(o);
                legacy::SimpleAssignTarget::OptChain(legacy::OptChainExpr {
                    span: convert_span(o.span),
                    optional: o.optional,
                    base: alloc_box!(
                        self,
                        match o.base {
                            experimental::OptChainBase::Member(m) => {
                                let m = AstBox::into_inner(m);
                                legacy::OptChainBase::Member(legacy::MemberExpr {
                                    span: convert_span(m.span),
                                    obj: self.convert_expr(m.obj),
                                    prop: self.convert_member_prop(m.prop),
                                })
                            }
                            experimental::OptChainBase::Call(c) => {
                                let c = AstBox::into_inner(c);
                                legacy::OptChainBase::Call(legacy::OptCall {
                                    span: convert_span(c.span),
                                    ctxt: Default::default(),
                                    callee: self.convert_expr(c.callee),
                                    args: self.convert_vec(c.args, Self::convert_expr_or_spread),
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
        }
    }

    fn convert_assign_target<'a>(
        &mut self,
        t: experimental::AssignTarget<'a>,
    ) -> legacy::AssignTarget {
        match t {
            experimental::AssignTarget::Simple(s) => legacy::AssignTarget::Simple(
                self.convert_simple_assign_target(AstBox::into_inner(s)),
            ),
            experimental::AssignTarget::Pat(p) => {
                legacy::AssignTarget::Pat(match AstBox::into_inner(p) {
                    experimental::AssignTargetPat::Array(a) => {
                        let a = AstBox::into_inner(a);
                        legacy::AssignTargetPat::Array(legacy::ArrayPat {
                            span: convert_span(a.span),
                            elems: self.convert_vec(a.elems, |this, pat| {
                                pat.map(|pat| this.convert_pat(pat))
                            }),
                            optional: false,
                            type_ann: None,
                        })
                    }
                    experimental::AssignTargetPat::Object(o) => {
                        let o = AstBox::into_inner(o);
                        legacy::AssignTargetPat::Object(legacy::ObjectPat {
                            span: convert_span(o.span),
                            props: self.convert_vec(o.props, Self::convert_object_pat_prop),
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

    fn convert_jsx_object<'a>(&mut self, o: experimental::JSXObject<'a>) -> legacy::JSXObject {
        match o {
            experimental::JSXObject::JSXMemberExpr(m) => {
                legacy::JSXObject::JSXMemberExpr(alloc_box!(self, self.convert_jsx_member_expr(m)))
            }
            experimental::JSXObject::Ident(i) => legacy::JSXObject::Ident(self.convert_ident(i)),
        }
    }

    fn convert_ident_name(&mut self, i: AstBox<'_, experimental::IdentName>) -> legacy::IdentName {
        let i = AstBox::into_inner(i);
        legacy::IdentName {
            span: convert_span(i.span),
            sym: self.convert_utf8_ref(i.sym),
        }
    }

    fn convert_jsx_member_expr<'a>(
        &mut self,
        j: AstBox<'a, experimental::JSXMemberExpr<'a>>,
    ) -> legacy::JSXMemberExpr {
        let j = AstBox::into_inner(j);
        legacy::JSXMemberExpr {
            span: convert_span(j.span),
            obj: self.convert_jsx_object(j.obj),
            prop: self.convert_ident_name(j.prop),
        }
    }

    fn convert_jsx_namespaced_name<'a>(
        &mut self,

        j: AstBox<'a, experimental::JSXNamespacedName<'a>>,
    ) -> legacy::JSXNamespacedName {
        let j = AstBox::into_inner(j);
        legacy::JSXNamespacedName {
            span: convert_span(j.span),
            ns: self.convert_ident_name(j.ns),
            name: self.convert_ident_name(j.name),
        }
    }

    fn convert_jsx_empty_expr(
        &mut self,
        j: AstBox<'_, experimental::JSXEmptyExpr>,
    ) -> legacy::JSXEmptyExpr {
        let j = AstBox::into_inner(j);
        legacy::JSXEmptyExpr {
            span: convert_span(j.span),
        }
    }

    fn convert_jsx_expr<'a>(&mut self, e: experimental::JSXExpr<'a>) -> legacy::JSXExpr {
        match e {
            experimental::JSXExpr::JSXEmptyExpr(ee) => {
                legacy::JSXExpr::JSXEmptyExpr(self.convert_jsx_empty_expr(ee))
            }
            experimental::JSXExpr::Expr(ex) => {
                legacy::JSXExpr::Expr(self.convert_expr(AstBox::into_inner(ex)))
            }
        }
    }

    fn convert_jsx_expr_container<'a>(
        &mut self,

        c: AstBox<'a, experimental::JSXExprContainer<'a>>,
    ) -> legacy::JSXExprContainer {
        let c = AstBox::into_inner(c);
        legacy::JSXExprContainer {
            span: convert_span(c.span),
            expr: self.convert_jsx_expr(c.expr),
        }
    }

    fn convert_spread_element<'a>(
        &mut self,
        s: AstBox<'a, experimental::SpreadElement<'a>>,
    ) -> legacy::SpreadElement {
        let s = AstBox::into_inner(s);
        legacy::SpreadElement {
            dot3_token: convert_span(s.dot3_token),
            expr: self.convert_expr(s.expr),
        }
    }

    fn convert_jsx_attr_name<'a>(
        &mut self,
        n: experimental::JSXAttrName<'a>,
    ) -> legacy::JSXAttrName {
        match n {
            experimental::JSXAttrName::Ident(i) => {
                legacy::JSXAttrName::Ident(self.convert_ident_name(i))
            }
            experimental::JSXAttrName::JSXNamespacedName(nn) => {
                legacy::JSXAttrName::JSXNamespacedName(self.convert_jsx_namespaced_name(nn))
            }
        }
    }

    fn convert_jsx_attr_value<'a>(
        &mut self,
        v: experimental::JSXAttrValue<'a>,
    ) -> legacy::JSXAttrValue {
        match v {
            experimental::JSXAttrValue::Str(s) => legacy::JSXAttrValue::Str(self.convert_str(s)),
            experimental::JSXAttrValue::JSXExprContainer(c) => {
                legacy::JSXAttrValue::JSXExprContainer(self.convert_jsx_expr_container(c))
            }
            experimental::JSXAttrValue::JSXElement(e) => {
                legacy::JSXAttrValue::JSXElement(alloc_box!(self, self.convert_jsx_element(e)))
            }
            experimental::JSXAttrValue::JSXFragment(f) => {
                legacy::JSXAttrValue::JSXFragment(self.convert_jsx_fragment(f))
            }
        }
    }

    fn convert_jsx_attr<'a>(
        &mut self,
        a: AstBox<'a, experimental::JSXAttr<'a>>,
    ) -> legacy::JSXAttr {
        let a = AstBox::into_inner(a);
        legacy::JSXAttr {
            span: convert_span(a.span),
            name: self.convert_jsx_attr_name(a.name),
            value: a.value.map(|v| self.convert_jsx_attr_value(v)),
        }
    }

    fn convert_jsx_attr_or_spread<'a>(
        &mut self,

        a: experimental::JSXAttrOrSpread<'a>,
    ) -> legacy::JSXAttrOrSpread {
        match a {
            experimental::JSXAttrOrSpread::JSXAttr(attr) => {
                legacy::JSXAttrOrSpread::JSXAttr(self.convert_jsx_attr(attr))
            }
            experimental::JSXAttrOrSpread::SpreadElement(se) => {
                legacy::JSXAttrOrSpread::SpreadElement(self.convert_spread_element(se))
            }
        }
    }

    fn convert_jsx_element_name<'a>(
        &mut self,

        n: experimental::JSXElementName<'a>,
    ) -> legacy::JSXElementName {
        match n {
            experimental::JSXElementName::Ident(i) => {
                legacy::JSXElementName::Ident(self.convert_ident(i))
            }
            experimental::JSXElementName::JSXMemberExpr(m) => {
                legacy::JSXElementName::JSXMemberExpr(self.convert_jsx_member_expr(m))
            }
            experimental::JSXElementName::JSXNamespacedName(nn) => {
                legacy::JSXElementName::JSXNamespacedName(self.convert_jsx_namespaced_name(nn))
            }
        }
    }

    fn convert_jsx_opening_element<'a>(
        &mut self,

        o: AstBox<'a, experimental::JSXOpeningElement<'a>>,
    ) -> legacy::JSXOpeningElement {
        let o = AstBox::into_inner(o);
        legacy::JSXOpeningElement {
            span: convert_span(o.span),
            name: self.convert_jsx_element_name(o.name),
            attrs: self.convert_vec(o.attrs, Self::convert_jsx_attr_or_spread),
            self_closing: o.self_closing,
            type_args: None,
        }
    }

    fn convert_jsx_closing_element<'a>(
        &mut self,

        c: AstBox<'a, experimental::JSXClosingElement<'a>>,
    ) -> legacy::JSXClosingElement {
        let c = AstBox::into_inner(c);
        legacy::JSXClosingElement {
            span: convert_span(c.span),
            name: self.convert_jsx_element_name(c.name),
        }
    }

    fn convert_jsx_text(&mut self, t: AstBox<'_, experimental::JSXText>) -> legacy::JSXText {
        let t = AstBox::into_inner(t);
        legacy::JSXText {
            span: convert_span(t.span),
            value: self.convert_utf8_ref(t.value),
            raw: self.convert_utf8_ref(t.raw),
        }
    }

    fn convert_jsx_spread_child<'a>(
        &mut self,

        s: AstBox<'a, experimental::JSXSpreadChild<'a>>,
    ) -> legacy::JSXSpreadChild {
        let s = AstBox::into_inner(s);
        legacy::JSXSpreadChild {
            span: convert_span(s.span),
            expr: self.convert_expr(s.expr),
        }
    }

    fn convert_jsx_element_child<'a>(
        &mut self,

        c: experimental::JSXElementChild<'a>,
    ) -> legacy::JSXElementChild {
        match c {
            experimental::JSXElementChild::JSXText(t) => {
                legacy::JSXElementChild::JSXText(self.convert_jsx_text(t))
            }
            experimental::JSXElementChild::JSXExprContainer(ec) => {
                legacy::JSXElementChild::JSXExprContainer(self.convert_jsx_expr_container(ec))
            }
            experimental::JSXElementChild::JSXSpreadChild(sc) => {
                legacy::JSXElementChild::JSXSpreadChild(self.convert_jsx_spread_child(sc))
            }
            experimental::JSXElementChild::JSXElement(e) => {
                legacy::JSXElementChild::JSXElement(alloc_box!(self, self.convert_jsx_element(e)))
            }
            experimental::JSXElementChild::JSXFragment(f) => {
                legacy::JSXElementChild::JSXFragment(self.convert_jsx_fragment(f))
            }
        }
    }

    fn convert_jsx_element<'a>(
        &mut self,
        e: AstBox<'a, experimental::JSXElement<'a>>,
    ) -> legacy::JSXElement {
        let e = AstBox::into_inner(e);
        legacy::JSXElement {
            span: convert_span(e.span),
            opening: self.convert_jsx_opening_element(e.opening),
            children: self.convert_vec(e.children, Self::convert_jsx_element_child),
            closing: e.closing.map(|c| self.convert_jsx_closing_element(c)),
        }
    }

    fn convert_jsx_opening_fragment(
        &mut self,

        o: AstBox<'_, experimental::JSXOpeningFragment>,
    ) -> legacy::JSXOpeningFragment {
        let o = AstBox::into_inner(o);
        legacy::JSXOpeningFragment {
            span: convert_span(o.span),
        }
    }

    fn convert_jsx_closing_fragment(
        &mut self,

        c: AstBox<'_, experimental::JSXClosingFragment>,
    ) -> legacy::JSXClosingFragment {
        let c = AstBox::into_inner(c);
        legacy::JSXClosingFragment {
            span: convert_span(c.span),
        }
    }

    fn convert_jsx_fragment<'a>(
        &mut self,
        f: AstBox<'a, experimental::JSXFragment<'a>>,
    ) -> legacy::JSXFragment {
        let f = AstBox::into_inner(f);
        legacy::JSXFragment {
            span: convert_span(f.span),
            opening: self.convert_jsx_opening_fragment(f.opening),
            children: self.convert_vec(f.children, Self::convert_jsx_element_child),
            closing: self.convert_jsx_closing_fragment(f.closing),
        }
    }

    // ===============================================================================

    fn convert_utf8_ref(&mut self, atom: ExperimentalAtom<'_>) -> Atom {
        Atom::from(atom.as_str())
    }

    fn convert_opt_utf8_ref(&mut self, atom: Option<ExperimentalAtom<'_>>) -> Option<Atom> {
        atom.map(|atom| self.convert_utf8_ref(atom))
    }

    fn convert_wtf8_ref(&mut self, atom: ExperimentalWtf8Atom<'_>) -> Wtf8Atom {
        match atom.as_wtf8().as_str() {
            Some(s) => Wtf8Atom::from(s),
            None => Wtf8Atom::from(atom.as_wtf8().to_string_lossy().into_owned()),
        }
    }

    fn convert_opt_wtf8_ref(&mut self, atom: Option<ExperimentalWtf8Atom<'_>>) -> Option<Wtf8Atom> {
        atom.map(|atom| self.convert_wtf8_ref(atom))
    }

    fn convert_big_int(&mut self, value: ExperimentalAtom<'_>) -> legacy::BigIntValue {
        legacy::BigIntValue::parse_bytes(value.as_bytes(), 10).unwrap()
    }
}
