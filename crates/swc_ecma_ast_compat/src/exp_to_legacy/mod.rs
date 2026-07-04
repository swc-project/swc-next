use swc_core::atoms::{Atom, Wtf8Atom};
use swc_core::common::Span as SwcSpan;
use swc_core::ecma::ast::{self as legacy};
use swc_experimental_allocator::Allocator;
use swc_experimental_allocator::atom::{
    Atom as ExperimentalAtom, Wtf8Atom as ExperimentalWtf8Atom,
};
use swc_experimental_allocator::boxed::Box as AstBox;
use swc_experimental_allocator::vec::Vec as ArenaVec;
use swc_experimental_ecma_ast::{self as experimental, AstBuilder};

pub struct AstConvert<'a> {
    allocator: &'a Allocator,
    ast: AstBuilder<'a>,
}

impl<'a> AstConvert<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            ast: AstBuilder { allocator },
        }
    }

    pub fn convert_program(&self, root: legacy::Program) -> experimental::Program<'a> {
        match root {
            legacy::Program::Module(module) => self.ast.program_module(
                convert_span(module.span),
                self.vec(module.body, Self::convert_module_item),
                self.convert_opt_atom(module.shebang),
            ),
            legacy::Program::Script(script) => self.ast.program_script(
                convert_span(script.span),
                self.vec(script.body, Self::convert_stmt),
                self.convert_opt_atom(script.shebang),
            ),
        }
    }

    fn vec<T, U, F>(&self, items: std::vec::Vec<T>, mut f: F) -> ArenaVec<'a, U>
    where
        F: FnMut(&Self, T) -> U,
    {
        let mut ret = ArenaVec::with_capacity_in(items.len(), self.allocator);
        for item in items {
            ret.push(f(self, item));
        }
        ret
    }

    fn empty_vec<T>(&self) -> ArenaVec<'a, T> {
        self.allocator.vec()
    }

    fn boxed<T>(&self, value: T) -> AstBox<'a, T> {
        self.ast.allocator.boxed(value)
    }

    fn convert_module_item(&self, item: legacy::ModuleItem) -> experimental::ModuleItem<'a> {
        match item {
            legacy::ModuleItem::ModuleDecl(decl) => experimental::ModuleItem::ModuleDecl(
                self.ast.allocator.boxed(self.convert_module_decl(decl)),
            ),
            legacy::ModuleItem::Stmt(stmt) => {
                experimental::ModuleItem::Stmt(self.ast.allocator.boxed(self.convert_stmt(stmt)))
            }
        }
    }

    fn convert_module_decl(&self, decl: legacy::ModuleDecl) -> experimental::ModuleDecl<'a> {
        match decl {
            legacy::ModuleDecl::Import(import) => self.ast.module_decl_import_decl(
                convert_span(import.span),
                self.vec(import.specifiers, Self::convert_import_specifier),
                self.ast.box_str(
                    convert_span(import.src.span),
                    self.convert_wtf8_atom(import.src.value),
                    self.convert_opt_atom(import.src.raw),
                ),
                import.type_only,
                import.with.map(|with| {
                    self.ast.box_object_lit(
                        convert_span(with.span),
                        self.vec(with.props, Self::convert_prop_or_spread),
                    )
                }),
                convert_import_phase(import.phase),
            ),
            legacy::ModuleDecl::ExportDecl(export) => {
                experimental::ModuleDecl::ExportDecl(self.boxed(self.convert_export_decl(export)))
            }
            legacy::ModuleDecl::ExportNamed(export) => {
                experimental::ModuleDecl::ExportNamed(self.boxed(self.convert_export_named(export)))
            }
            legacy::ModuleDecl::ExportDefaultDecl(export) => {
                experimental::ModuleDecl::ExportDefaultDecl(
                    self.boxed(self.convert_export_default_decl(export)),
                )
            }
            legacy::ModuleDecl::ExportDefaultExpr(export) => {
                experimental::ModuleDecl::ExportDefaultExpr(
                    self.boxed(self.convert_export_default_expr(export)),
                )
            }
            legacy::ModuleDecl::ExportAll(export) => {
                experimental::ModuleDecl::ExportAll(self.boxed(self.convert_export_all(export)))
            }
            legacy::ModuleDecl::TsImportEquals(_)
            | legacy::ModuleDecl::TsExportAssignment(_)
            | legacy::ModuleDecl::TsNamespaceExport(_) => {
                unimplemented!(
                    "typescript module declarations are not represented in experimental AST"
                )
            }
        }
    }

    fn convert_export_decl(&self, export: legacy::ExportDecl) -> experimental::ExportDecl<'a> {
        self.ast
            .export_decl(convert_span(export.span), self.convert_decl(export.decl))
    }

    fn convert_export_named(&self, export: legacy::NamedExport) -> experimental::NamedExport<'a> {
        self.ast.named_export(
            convert_span(export.span),
            self.vec(export.specifiers, Self::convert_export_specifier),
            export.src.map(|src| self.boxed(self.convert_str(*src))),
            export.type_only,
            export
                .with
                .map(|with| self.boxed(self.convert_object_lit(*with))),
        )
    }

    fn convert_export_default_decl(
        &self,
        export: legacy::ExportDefaultDecl,
    ) -> experimental::ExportDefaultDecl<'a> {
        let decl = match export.decl {
            legacy::DefaultDecl::Class(class) => {
                experimental::DefaultDecl::Class(self.boxed(self.convert_class_expr(class)))
            }
            legacy::DefaultDecl::Fn(function) => {
                experimental::DefaultDecl::Fn(self.boxed(self.convert_fn_expr(function)))
            }
            legacy::DefaultDecl::TsInterfaceDecl(_) => {
                unimplemented!(
                    "typescript default declarations are not represented in experimental AST"
                )
            }
        };
        self.ast
            .export_default_decl(convert_span(export.span), decl)
    }

    fn convert_export_default_expr(
        &self,
        export: legacy::ExportDefaultExpr,
    ) -> experimental::ExportDefaultExpr<'a> {
        self.ast
            .export_default_expr(convert_span(export.span), self.convert_expr(*export.expr))
    }

    fn convert_export_all(&self, export: legacy::ExportAll) -> experimental::ExportAll<'a> {
        self.ast.export_all(
            convert_span(export.span),
            self.boxed(self.convert_str(*export.src)),
            export.type_only,
            export
                .with
                .map(|with| self.boxed(self.convert_object_lit(*with))),
        )
    }

    fn convert_stmt(&self, stmt: legacy::Stmt) -> experimental::Stmt<'a> {
        match stmt {
            legacy::Stmt::Block(block) => {
                let span = convert_span(block.span);
                self.ast
                    .stmt_block_stmt(span, self.vec(block.stmts, Self::convert_stmt))
            }
            legacy::Stmt::Empty(empty) => self.ast.stmt_empty_stmt(convert_span(empty.span)),
            legacy::Stmt::Debugger(debugger) => {
                self.ast.stmt_debugger_stmt(convert_span(debugger.span))
            }
            legacy::Stmt::With(with) => self.ast.stmt_with_stmt(
                convert_span(with.span),
                self.convert_expr(*with.obj),
                self.convert_stmt(*with.body),
            ),
            legacy::Stmt::Return(ret) => self.ast.stmt_return_stmt(
                convert_span(ret.span),
                ret.arg.map(|arg| self.convert_expr(*arg)),
            ),
            legacy::Stmt::Labeled(labeled) => self.ast.stmt_labeled_stmt(
                convert_span(labeled.span),
                self.ast.box_ident(
                    convert_span(labeled.label.span),
                    self.convert_atom(labeled.label.sym),
                ),
                self.convert_stmt(*labeled.body),
            ),
            legacy::Stmt::Break(break_stmt) => self.ast.stmt_break_stmt(
                convert_span(break_stmt.span),
                break_stmt.label.map(|label| {
                    self.ast
                        .box_ident(convert_span(label.span), self.convert_atom(label.sym))
                }),
            ),
            legacy::Stmt::Continue(continue_stmt) => self.ast.stmt_continue_stmt(
                convert_span(continue_stmt.span),
                continue_stmt.label.map(|label| {
                    self.ast
                        .box_ident(convert_span(label.span), self.convert_atom(label.sym))
                }),
            ),
            legacy::Stmt::If(if_stmt) => self.ast.stmt_if_stmt(
                convert_span(if_stmt.span),
                self.convert_expr(*if_stmt.test),
                self.convert_stmt(*if_stmt.cons),
                if_stmt.alt.map(|alt| self.convert_stmt(*alt)),
            ),
            legacy::Stmt::Switch(switch) => self.ast.stmt_switch_stmt(
                convert_span(switch.span),
                self.convert_expr(*switch.discriminant),
                self.vec(switch.cases, Self::convert_switch_case),
            ),
            legacy::Stmt::Throw(throw) => self
                .ast
                .stmt_throw_stmt(convert_span(throw.span), self.convert_expr(*throw.arg)),
            legacy::Stmt::Try(try_stmt) => {
                let try_stmt = *try_stmt;
                self.ast.stmt_try_stmt(
                    convert_span(try_stmt.span),
                    self.boxed(self.convert_block_stmt(try_stmt.block)),
                    try_stmt
                        .handler
                        .map(|handler| self.boxed(self.convert_catch_clause(handler))),
                    try_stmt
                        .finalizer
                        .map(|finalizer| self.boxed(self.convert_block_stmt(finalizer))),
                )
            }
            legacy::Stmt::While(while_stmt) => self.ast.stmt_while_stmt(
                convert_span(while_stmt.span),
                self.convert_expr(*while_stmt.test),
                self.convert_stmt(*while_stmt.body),
            ),
            legacy::Stmt::DoWhile(do_while) => self.ast.stmt_do_while_stmt(
                convert_span(do_while.span),
                self.convert_expr(*do_while.test),
                self.convert_stmt(*do_while.body),
            ),
            legacy::Stmt::For(for_stmt) => self.ast.stmt_for_stmt(
                convert_span(for_stmt.span),
                for_stmt
                    .init
                    .map(|init| self.convert_var_decl_or_expr(init)),
                for_stmt.test.map(|test| self.convert_expr(*test)),
                for_stmt.update.map(|update| self.convert_expr(*update)),
                self.convert_stmt(*for_stmt.body),
            ),
            legacy::Stmt::ForIn(for_in) => self.ast.stmt_for_in_stmt(
                convert_span(for_in.span),
                self.convert_for_head(for_in.left),
                self.convert_expr(*for_in.right),
                self.convert_stmt(*for_in.body),
            ),
            legacy::Stmt::ForOf(for_of) => self.ast.stmt_for_of_stmt(
                convert_span(for_of.span),
                for_of.is_await,
                self.convert_for_head(for_of.left),
                self.convert_expr(*for_of.right),
                self.convert_stmt(*for_of.body),
            ),
            legacy::Stmt::Decl(decl) => {
                experimental::Stmt::Decl(self.boxed(self.convert_decl(decl)))
            }
            legacy::Stmt::Expr(expr) => self
                .ast
                .stmt_expr_stmt(convert_span(expr.span), self.convert_expr(*expr.expr)),
        }
    }

    fn convert_block_stmt(&self, block: legacy::BlockStmt) -> experimental::BlockStmt<'a> {
        self.ast.block_stmt(
            convert_span(block.span),
            self.vec(block.stmts, Self::convert_stmt),
        )
    }

    fn convert_expr(&self, expr: legacy::Expr) -> experimental::Expr<'a> {
        match expr {
            legacy::Expr::This(this) => self.ast.expr_this_expr(convert_span(this.span)),
            legacy::Expr::Array(array) => self.ast.expr_array_lit(
                convert_span(array.span),
                self.vec(array.elems, |this, elem| {
                    elem.map(|elem| this.boxed(this.convert_expr_or_spread(elem)))
                }),
            ),
            legacy::Expr::Object(object) => self.ast.expr_object_lit(
                convert_span(object.span),
                self.vec(object.props, Self::convert_prop_or_spread),
            ),
            legacy::Expr::Fn(function) => {
                experimental::Expr::Fn(self.boxed(self.convert_fn_expr(function)))
            }
            legacy::Expr::Unary(unary) => self.ast.expr_unary_expr(
                convert_span(unary.span),
                convert_unary_op(unary.op),
                self.convert_expr(*unary.arg),
            ),
            legacy::Expr::Update(update) => self.ast.expr_update_expr(
                convert_span(update.span),
                convert_update_op(update.op),
                update.prefix,
                self.convert_expr_to_simple_assign_target(update.arg),
            ),
            legacy::Expr::Bin(binary) => self.ast.expr_bin_expr(
                convert_span(binary.span),
                convert_binary_op(binary.op),
                self.convert_expr(*binary.left),
                self.convert_expr(*binary.right),
            ),
            legacy::Expr::Assign(assign) => self.ast.expr_assign_expr(
                convert_span(assign.span),
                convert_assign_op(assign.op),
                self.convert_assign_target(assign.left),
                self.convert_expr(*assign.right),
            ),
            legacy::Expr::Member(member) => {
                experimental::Expr::Member(self.boxed(self.convert_member_expr(member)))
            }
            legacy::Expr::SuperProp(super_prop) => {
                experimental::Expr::SuperProp(self.boxed(self.convert_super_prop_expr(super_prop)))
            }
            legacy::Expr::Cond(cond) => self.ast.expr_cond_expr(
                convert_span(cond.span),
                self.convert_expr(*cond.test),
                self.convert_expr(*cond.cons),
                self.convert_expr(*cond.alt),
            ),
            legacy::Expr::Call(call) => self.convert_call_expr(call),
            legacy::Expr::New(new) => self.ast.expr_new_expr(
                convert_span(new.span),
                self.convert_expr(*new.callee),
                new.args
                    .map(|args| self.vec(args, Self::convert_expr_or_spread))
                    .unwrap_or_else(|| self.empty_vec()),
            ),
            legacy::Expr::Seq(seq) => self.ast.expr_seq_expr(
                convert_span(seq.span),
                self.vec(seq.exprs, |this, expr| this.convert_expr(*expr)),
            ),
            legacy::Expr::Ident(ident) => self
                .ast
                .expr_ident(convert_span(ident.span), self.convert_atom(ident.sym)),
            legacy::Expr::Lit(lit) => self.convert_lit_expr(lit),
            legacy::Expr::Tpl(tpl) => self.ast.expr_tpl(
                convert_span(tpl.span),
                self.vec(tpl.exprs, |this, expr| this.convert_expr(*expr)),
                self.vec(tpl.quasis, Self::convert_tpl_element),
            ),
            legacy::Expr::TaggedTpl(tagged) => self.ast.expr_tagged_tpl(
                convert_span(tagged.span),
                self.convert_expr(*tagged.tag),
                self.boxed(self.convert_tpl(*tagged.tpl)),
            ),
            legacy::Expr::Arrow(arrow) => self.ast.expr_arrow_expr(
                convert_span(arrow.span),
                self.vec(arrow.params, Self::convert_pat),
                match *arrow.body {
                    legacy::BlockStmtOrExpr::BlockStmt(block) => {
                        self.ast.block_stmt_or_expr_block_stmt(
                            convert_span(block.span),
                            self.vec(block.stmts, Self::convert_stmt),
                        )
                    }
                    legacy::BlockStmtOrExpr::Expr(expr) => {
                        experimental::BlockStmtOrExpr::Expr(self.boxed(self.convert_expr(*expr)))
                    }
                },
                arrow.is_async,
            ),
            legacy::Expr::Class(class) => {
                experimental::Expr::Class(self.boxed(self.convert_class_expr(class)))
            }
            legacy::Expr::Yield(yield_expr) => self.ast.expr_yield_expr(
                convert_span(yield_expr.span),
                yield_expr.arg.map(|arg| self.convert_expr(*arg)),
                yield_expr.delegate,
            ),
            legacy::Expr::MetaProp(meta) => self
                .ast
                .expr_meta_prop_expr(convert_span(meta.span), convert_meta_prop_kind(meta.kind)),
            legacy::Expr::Await(await_expr) => self.ast.expr_await_expr(
                convert_span(await_expr.span),
                self.convert_expr(*await_expr.arg),
            ),
            legacy::Expr::Paren(paren) => self
                .ast
                .expr_paren_expr(convert_span(paren.span), self.convert_expr(*paren.expr)),
            legacy::Expr::JSXMember(member) => {
                experimental::Expr::JSXMember(self.boxed(self.convert_jsx_member_expr(member)))
            }
            legacy::Expr::JSXNamespacedName(name) => experimental::Expr::JSXNamespacedName(
                self.boxed(self.convert_jsx_namespaced_name(name)),
            ),
            legacy::Expr::JSXEmpty(empty) => self.ast.expr_jsx_empty_expr(convert_span(empty.span)),
            legacy::Expr::JSXElement(element) => {
                experimental::Expr::JSXElement(self.boxed(self.convert_jsx_element(*element)))
            }
            legacy::Expr::JSXFragment(fragment) => {
                experimental::Expr::JSXFragment(self.boxed(self.convert_jsx_fragment(fragment)))
            }
            legacy::Expr::PrivateName(private) => self
                .ast
                .expr_private_name(convert_span(private.span), self.convert_atom(private.name)),
            legacy::Expr::OptChain(opt_chain) => {
                experimental::Expr::OptChain(self.boxed(self.convert_opt_chain_expr(opt_chain)))
            }
            legacy::Expr::Invalid(_) => self.ast.expr_invalid(),
            legacy::Expr::TsTypeAssertion(ts) => self.convert_expr(*ts.expr),
            legacy::Expr::TsConstAssertion(ts) => self.convert_expr(*ts.expr),
            legacy::Expr::TsNonNull(ts) => self.convert_expr(*ts.expr),
            legacy::Expr::TsAs(ts) => self.convert_expr(*ts.expr),
            legacy::Expr::TsInstantiation(ts) => self.convert_expr(*ts.expr),
            legacy::Expr::TsSatisfies(ts) => self.convert_expr(*ts.expr),
        }
    }

    fn convert_call_expr(&self, call: legacy::CallExpr) -> experimental::Expr<'a> {
        match call.callee {
            legacy::Callee::Import(import) => {
                let mut args = call.args.into_iter();
                let source = args
                    .next()
                    .map(|arg| self.convert_expr(*arg.expr))
                    .unwrap_or_else(|| self.ast.expr_invalid());
                let options = args.next().map(|arg| self.convert_expr(*arg.expr));

                self.ast.expr_import_expr(
                    convert_span(call.span),
                    source,
                    options,
                    convert_import_phase(import.phase),
                )
            }
            callee => self.ast.expr_call_expr(
                convert_span(call.span),
                match callee {
                    legacy::Callee::Super(super_) => {
                        self.ast.callee_super(convert_span(super_.span))
                    }
                    legacy::Callee::Expr(expr) => {
                        experimental::Callee::Expr(self.boxed(self.convert_expr(*expr)))
                    }
                    legacy::Callee::Import(_) => unreachable!(),
                },
                self.vec(call.args, Self::convert_expr_or_spread),
            ),
        }
    }

    fn convert_ident(&self, ident: legacy::Ident) -> experimental::Ident<'a> {
        self.ast
            .ident(convert_span(ident.span), self.convert_atom(ident.sym))
    }

    fn convert_ident_name(&self, ident: legacy::IdentName) -> experimental::IdentName<'a> {
        self.ast
            .ident_name(convert_span(ident.span), self.convert_atom(ident.sym))
    }

    fn convert_private_name(&self, private: legacy::PrivateName) -> experimental::PrivateName<'a> {
        self.ast
            .private_name(convert_span(private.span), self.convert_atom(private.name))
    }

    fn convert_switch_case(&self, case: legacy::SwitchCase) -> experimental::SwitchCase<'a> {
        self.ast.switch_case(
            convert_span(case.span),
            case.test.map(|test| self.convert_expr(*test)),
            self.vec(case.cons, Self::convert_stmt),
        )
    }

    fn convert_catch_clause(&self, catch: legacy::CatchClause) -> experimental::CatchClause<'a> {
        self.ast.catch_clause(
            convert_span(catch.span),
            catch.param.map(|param| self.convert_pat(param)),
            self.boxed(self.convert_block_stmt(catch.body)),
        )
    }

    fn convert_var_decl_or_expr(
        &self,
        var_decl_or_expr: legacy::VarDeclOrExpr,
    ) -> experimental::VarDeclOrExpr<'a> {
        match var_decl_or_expr {
            legacy::VarDeclOrExpr::VarDecl(var) => {
                experimental::VarDeclOrExpr::VarDecl(self.boxed(self.convert_var_decl(*var)))
            }
            legacy::VarDeclOrExpr::Expr(expr) => {
                experimental::VarDeclOrExpr::Expr(self.boxed(self.convert_expr(*expr)))
            }
        }
    }

    fn convert_for_head(&self, head: legacy::ForHead) -> experimental::ForHead<'a> {
        match head {
            legacy::ForHead::VarDecl(var) => {
                experimental::ForHead::VarDecl(self.boxed(self.convert_var_decl(*var)))
            }
            legacy::ForHead::UsingDecl(using) => {
                experimental::ForHead::UsingDecl(self.boxed(self.convert_using_decl(*using)))
            }
            legacy::ForHead::Pat(pat) => {
                experimental::ForHead::Pat(self.boxed(self.convert_pat(*pat)))
            }
        }
    }

    fn convert_import_specifier(
        &self,
        specifier: legacy::ImportSpecifier,
    ) -> experimental::ImportSpecifier<'a> {
        match specifier {
            legacy::ImportSpecifier::Named(named) => {
                let local = self.convert_ident(named.local);
                let imported = named.imported.map_or_else(
                    || experimental::ModuleExportName::Ident(self.boxed(local.clone())),
                    |imported| self.convert_module_export_name(imported),
                );

                self.ast.import_specifier_import_named_specifier(
                    convert_span(named.span),
                    self.boxed(local),
                    imported,
                    named.is_type_only,
                )
            }
            legacy::ImportSpecifier::Default(default) => {
                self.ast.import_specifier_import_default_specifier(
                    convert_span(default.span),
                    self.boxed(self.convert_ident(default.local)),
                )
            }
            legacy::ImportSpecifier::Namespace(namespace) => {
                self.ast.import_specifier_import_star_as_specifier(
                    convert_span(namespace.span),
                    self.boxed(self.convert_ident(namespace.local)),
                )
            }
        }
    }

    fn convert_export_specifier(
        &self,
        specifier: legacy::ExportSpecifier,
    ) -> experimental::ExportSpecifier<'a> {
        match specifier {
            legacy::ExportSpecifier::Namespace(namespace) => {
                self.ast.export_specifier_export_namespace_specifier(
                    convert_span(namespace.span),
                    self.convert_module_export_name(namespace.name),
                )
            }
            legacy::ExportSpecifier::Default(default) => {
                self.ast.export_specifier_export_default_specifier(
                    self.boxed(self.convert_ident(default.exported)),
                )
            }
            legacy::ExportSpecifier::Named(named) => {
                let exported = named.exported.map_or_else(
                    || self.convert_module_export_name(named.orig.clone()),
                    |exported| self.convert_module_export_name(exported),
                );
                self.ast.export_specifier_export_named_specifier(
                    convert_span(named.span),
                    self.convert_module_export_name(named.orig),
                    exported,
                    named.is_type_only,
                )
            }
        }
    }

    fn convert_module_export_name(
        &self,
        name: legacy::ModuleExportName,
    ) -> experimental::ModuleExportName<'a> {
        match name {
            legacy::ModuleExportName::Ident(ident) => {
                experimental::ModuleExportName::Ident(self.boxed(self.convert_ident(ident)))
            }
            legacy::ModuleExportName::Str(str) => {
                experimental::ModuleExportName::Str(self.boxed(self.convert_str(str)))
            }
        }
    }

    fn convert_object_lit(&self, object: legacy::ObjectLit) -> experimental::ObjectLit<'a> {
        self.ast.object_lit(
            convert_span(object.span),
            self.vec(object.props, Self::convert_prop_or_spread),
        )
    }

    fn convert_prop_or_spread(
        &self,
        prop_or_spread: legacy::PropOrSpread,
    ) -> experimental::PropOrSpread<'a> {
        match prop_or_spread {
            legacy::PropOrSpread::Spread(spread) => self.ast.prop_or_spread_spread_element(
                convert_span(spread.dot3_token),
                self.convert_expr(*spread.expr),
            ),
            legacy::PropOrSpread::Prop(prop) => {
                experimental::PropOrSpread::Prop(self.boxed(self.convert_prop(*prop)))
            }
        }
    }

    fn convert_prop(&self, prop: legacy::Prop) -> experimental::Prop<'a> {
        match prop {
            legacy::Prop::Shorthand(ident) => self
                .ast
                .prop_ident(convert_span(ident.span), self.convert_atom(ident.sym)),
            legacy::Prop::KeyValue(key_value) => self.ast.prop_key_value_prop(
                self.convert_prop_name(key_value.key),
                self.convert_expr(*key_value.value),
            ),
            legacy::Prop::Assign(assign) => self.ast.prop_assign_prop(
                convert_span(assign.span),
                self.boxed(self.convert_ident(assign.key)),
                self.convert_expr(*assign.value),
            ),
            legacy::Prop::Getter(getter) => self.ast.prop_getter_prop(
                convert_span(getter.span),
                self.convert_prop_name(getter.key),
                getter
                    .body
                    .map(|body| self.boxed(self.convert_block_stmt(body))),
            ),
            legacy::Prop::Setter(setter) => self.ast.prop_setter_prop(
                convert_span(setter.span),
                self.convert_prop_name(setter.key),
                setter.this_param.map(|param| self.convert_pat(param)),
                self.convert_pat(*setter.param),
                setter
                    .body
                    .map(|body| self.boxed(self.convert_block_stmt(body))),
            ),
            legacy::Prop::Method(method) => self.ast.prop_method_prop(
                self.convert_prop_name(method.key),
                self.boxed(self.convert_function(*method.function)),
            ),
        }
    }

    fn convert_prop_name(&self, name: legacy::PropName) -> experimental::PropName<'a> {
        match name {
            legacy::PropName::Ident(ident) => {
                experimental::PropName::Ident(self.boxed(self.convert_ident_name(ident)))
            }
            legacy::PropName::Str(str) => {
                experimental::PropName::Str(self.boxed(self.convert_str(str)))
            }
            legacy::PropName::Num(number) => experimental::PropName::Num(self.ast.box_number(
                convert_span(number.span),
                number.value,
                self.convert_opt_atom(number.raw),
            )),
            legacy::PropName::Computed(computed) => {
                experimental::PropName::Computed(self.ast.box_computed_prop_name(
                    convert_span(computed.span),
                    self.convert_expr(*computed.expr),
                ))
            }
            legacy::PropName::BigInt(big_int) => {
                experimental::PropName::BigInt(self.ast.box_big_int(
                    convert_span(big_int.span),
                    self.convert_atom(Atom::from(big_int.value.to_string())),
                    self.convert_opt_atom(big_int.raw),
                ))
            }
        }
    }

    fn convert_str(&self, str: legacy::Str) -> experimental::Str<'a> {
        self.ast.str(
            convert_span(str.span),
            self.convert_wtf8_atom(str.value),
            self.convert_opt_atom(str.raw),
        )
    }

    fn convert_fn_expr(&self, function: legacy::FnExpr) -> experimental::FnExpr<'a> {
        self.ast.fn_expr(
            function
                .ident
                .map(|ident| self.boxed(self.convert_ident(ident))),
            self.boxed(self.convert_function(*function.function)),
        )
    }

    fn convert_class_expr(&self, class: legacy::ClassExpr) -> experimental::ClassExpr<'a> {
        self.ast.class_expr(
            class
                .ident
                .map(|ident| self.boxed(self.convert_ident(ident))),
            self.boxed(self.convert_class(*class.class)),
        )
    }

    fn convert_function(&self, function: legacy::Function) -> experimental::Function<'a> {
        let span = convert_span(function.span);
        self.ast.function(
            span,
            self.vec(function.params, Self::convert_param),
            self.vec(function.decorators, Self::convert_decorator),
            function.body.map_or_else(
                || self.ast.box_block_stmt(span, self.empty_vec()),
                |body| self.boxed(self.convert_block_stmt(body)),
            ),
            function.is_generator,
            function.is_async,
        )
    }

    fn convert_param(&self, param: legacy::Param) -> experimental::Param<'a> {
        self.ast.param(
            convert_span(param.span),
            self.vec(param.decorators, Self::convert_decorator),
            self.convert_pat(param.pat),
        )
    }

    fn convert_decorator(&self, decorator: legacy::Decorator) -> experimental::Decorator<'a> {
        self.ast.decorator(
            convert_span(decorator.span),
            self.convert_expr(*decorator.expr),
        )
    }

    fn convert_class(&self, class: legacy::Class) -> experimental::Class<'a> {
        self.ast.class(
            convert_span(class.span),
            self.vec(class.decorators, Self::convert_decorator),
            self.vec(class.body, Self::convert_class_member),
            class
                .super_class
                .map(|super_class| self.convert_expr(*super_class)),
            class.is_abstract,
        )
    }

    fn convert_class_member(&self, member: legacy::ClassMember) -> experimental::ClassMember<'a> {
        match member {
            legacy::ClassMember::Constructor(constructor) => {
                experimental::ClassMember::Constructor(
                    self.ast.box_constructor(
                        convert_span(constructor.span),
                        self.convert_prop_name(constructor.key),
                        self.vec(constructor.params, Self::convert_param_or_ts_param_prop),
                        constructor
                            .body
                            .map(|body| self.boxed(self.convert_block_stmt(body))),
                    ),
                )
            }
            legacy::ClassMember::Method(method) => {
                experimental::ClassMember::Method(self.ast.box_class_method(
                    convert_span(method.span),
                    self.convert_prop_name(method.key),
                    self.boxed(self.convert_function(*method.function)),
                    convert_method_kind(method.kind),
                    method.is_static,
                ))
            }
            legacy::ClassMember::PrivateMethod(method) => {
                experimental::ClassMember::PrivateMethod(self.ast.box_private_method(
                    convert_span(method.span),
                    self.boxed(self.convert_private_name(method.key)),
                    self.boxed(self.convert_function(*method.function)),
                    convert_method_kind(method.kind),
                    method.is_static,
                ))
            }
            legacy::ClassMember::ClassProp(prop) => {
                experimental::ClassMember::ClassProp(self.ast.box_class_prop(
                    convert_span(prop.span),
                    self.convert_prop_name(prop.key),
                    prop.value.map(|value| self.convert_expr(*value)),
                    prop.is_static,
                    self.vec(prop.decorators, Self::convert_decorator),
                ))
            }
            legacy::ClassMember::PrivateProp(prop) => {
                experimental::ClassMember::PrivateProp(self.ast.box_private_prop(
                    convert_span(prop.span),
                    self.boxed(self.convert_private_name(prop.key)),
                    prop.value.map(|value| self.convert_expr(*value)),
                    prop.is_static,
                    self.vec(prop.decorators, Self::convert_decorator),
                ))
            }
            legacy::ClassMember::Empty(empty) => {
                experimental::ClassMember::Empty(self.ast.box_empty_stmt(convert_span(empty.span)))
            }
            legacy::ClassMember::StaticBlock(block) => {
                experimental::ClassMember::StaticBlock(self.ast.box_static_block(
                    convert_span(block.span),
                    self.boxed(self.convert_block_stmt(block.body)),
                ))
            }
            legacy::ClassMember::AutoAccessor(accessor) => {
                experimental::ClassMember::AutoAccessor(self.ast.box_auto_accessor(
                    convert_span(accessor.span),
                    self.convert_key(accessor.key),
                    accessor.value.map(|value| self.convert_expr(*value)),
                    accessor.is_static,
                    self.vec(accessor.decorators, Self::convert_decorator),
                ))
            }
            legacy::ClassMember::TsIndexSignature(_) => {
                unimplemented!("typescript class members are not represented in experimental AST")
            }
        }
    }

    fn convert_key(&self, key: legacy::Key) -> experimental::Key<'a> {
        match key {
            legacy::Key::Private(private) => {
                experimental::Key::Private(self.boxed(self.convert_private_name(private)))
            }
            legacy::Key::Public(public) => match public {
                legacy::PropName::Ident(ident) => experimental::Key::Public(
                    self.boxed(self.convert_prop_name(legacy::PropName::Ident(ident))),
                ),
                legacy::PropName::Str(str) => experimental::Key::Public(
                    self.boxed(self.convert_prop_name(legacy::PropName::Str(str))),
                ),
                legacy::PropName::Num(number) => experimental::Key::Public(
                    self.boxed(self.convert_prop_name(legacy::PropName::Num(number))),
                ),
                legacy::PropName::Computed(computed) => experimental::Key::Public(
                    self.boxed(self.convert_prop_name(legacy::PropName::Computed(computed))),
                ),
                legacy::PropName::BigInt(big_int) => experimental::Key::Public(
                    self.boxed(self.convert_prop_name(legacy::PropName::BigInt(big_int))),
                ),
            },
        }
    }

    fn convert_param_or_ts_param_prop(
        &self,
        param: legacy::ParamOrTsParamProp,
    ) -> experimental::ParamOrTsParamProp<'a> {
        match param {
            legacy::ParamOrTsParamProp::Param(param) => {
                experimental::ParamOrTsParamProp::Param(self.boxed(self.convert_param(param)))
            }
            legacy::ParamOrTsParamProp::TsParamProp(_) => {
                unimplemented!(
                    "typescript parameter properties are not represented in experimental AST"
                )
            }
        }
    }

    fn convert_decl(&self, decl: legacy::Decl) -> experimental::Decl<'a> {
        match decl {
            legacy::Decl::Class(class) => self.ast.decl_class_decl(
                self.boxed(self.convert_ident(class.ident)),
                class.declare,
                self.boxed(self.convert_class(*class.class)),
            ),
            legacy::Decl::Fn(function) => self.ast.decl_fn_decl(
                self.boxed(self.convert_ident(function.ident)),
                function.declare,
                self.boxed(self.convert_function(*function.function)),
            ),
            legacy::Decl::Var(var) => {
                experimental::Decl::Var(self.boxed(self.convert_var_decl(*var)))
            }
            legacy::Decl::Using(using) => {
                experimental::Decl::Using(self.boxed(self.convert_using_decl(*using)))
            }
            legacy::Decl::TsInterface(_)
            | legacy::Decl::TsTypeAlias(_)
            | legacy::Decl::TsEnum(_)
            | legacy::Decl::TsModule(_) => {
                unimplemented!("typescript declarations are not represented in experimental AST")
            }
        }
    }

    fn convert_var_decl(&self, var: legacy::VarDecl) -> experimental::VarDecl<'a> {
        self.ast.var_decl(
            convert_span(var.span),
            match var.kind {
                legacy::VarDeclKind::Var => experimental::VarDeclKind::Var,
                legacy::VarDeclKind::Let => experimental::VarDeclKind::Let,
                legacy::VarDeclKind::Const => experimental::VarDeclKind::Const,
            },
            var.declare,
            self.vec(var.decls, Self::convert_var_declarator),
        )
    }

    fn convert_using_decl(&self, using: legacy::UsingDecl) -> experimental::UsingDecl<'a> {
        self.ast.using_decl(
            convert_span(using.span),
            using.is_await,
            self.vec(using.decls, Self::convert_var_declarator),
        )
    }

    fn convert_var_declarator(
        &self,
        declarator: legacy::VarDeclarator,
    ) -> experimental::VarDeclarator<'a> {
        self.ast.var_declarator(
            convert_span(declarator.span),
            self.convert_pat(declarator.name),
            declarator.init.map(|init| self.convert_expr(*init)),
        )
    }

    fn convert_pat(&self, pat: legacy::Pat) -> experimental::Pat<'a> {
        match pat {
            legacy::Pat::Ident(binding) => self
                .ast
                .pat_binding_ident(self.boxed(self.convert_ident(binding.id))),
            legacy::Pat::Array(array) => self.ast.pat_array_pat(
                convert_span(array.span),
                self.vec(array.elems, |this, pat| {
                    pat.map(|pat| this.convert_pat(pat))
                }),
                array.optional,
            ),
            legacy::Pat::Rest(rest) => self.ast.pat_rest_pat(
                convert_span(rest.span),
                convert_span(rest.dot3_token),
                self.convert_pat(*rest.arg),
            ),
            legacy::Pat::Object(object) => self.ast.pat_object_pat(
                convert_span(object.span),
                self.vec(object.props, Self::convert_object_pat_prop),
                object.optional,
            ),
            legacy::Pat::Assign(assign) => self.ast.pat_assign_pat(
                convert_span(assign.span),
                self.convert_pat(*assign.left),
                self.convert_expr(*assign.right),
            ),
            legacy::Pat::Invalid(_) => self.ast.pat_invalid(),
            legacy::Pat::Expr(expr) => {
                experimental::Pat::Expr(self.boxed(self.convert_expr(*expr)))
            }
        }
    }

    fn convert_binding_ident(
        &self,
        binding: legacy::BindingIdent,
    ) -> experimental::BindingIdent<'a> {
        self.ast
            .binding_ident(self.boxed(self.convert_ident(binding.id)))
    }

    fn convert_object_pat_prop(
        &self,
        prop: legacy::ObjectPatProp,
    ) -> experimental::ObjectPatProp<'a> {
        match prop {
            legacy::ObjectPatProp::KeyValue(key_value) => {
                self.ast.object_pat_prop_key_value_pat_prop(
                    self.convert_prop_name(key_value.key),
                    self.convert_pat(*key_value.value),
                )
            }
            legacy::ObjectPatProp::Assign(assign) => self.ast.object_pat_prop_assign_pat_prop(
                convert_span(assign.span),
                self.boxed(self.convert_binding_ident(assign.key)),
                assign.value.map(|value| self.convert_expr(*value)),
            ),
            legacy::ObjectPatProp::Rest(rest) => self.ast.object_pat_prop_rest_pat(
                convert_span(rest.span),
                convert_span(rest.dot3_token),
                self.convert_pat(*rest.arg),
            ),
        }
    }

    fn convert_expr_or_spread(
        &self,
        expr_or_spread: legacy::ExprOrSpread,
    ) -> experimental::ExprOrSpread<'a> {
        self.ast.expr_or_spread(
            expr_or_spread.spread.map(convert_span),
            self.convert_expr(*expr_or_spread.expr),
        )
    }

    fn convert_member_expr(&self, member: legacy::MemberExpr) -> experimental::MemberExpr<'a> {
        self.ast.member_expr(
            convert_span(member.span),
            self.convert_expr(*member.obj),
            self.convert_member_prop(member.prop),
        )
    }

    fn convert_member_prop(&self, prop: legacy::MemberProp) -> experimental::MemberProp<'a> {
        match prop {
            legacy::MemberProp::Ident(ident) => {
                experimental::MemberProp::Ident(self.boxed(self.convert_ident_name(ident)))
            }
            legacy::MemberProp::PrivateName(private) => experimental::MemberProp::PrivateName(
                self.boxed(self.convert_private_name(private)),
            ),
            legacy::MemberProp::Computed(computed) => {
                experimental::MemberProp::Computed(self.ast.box_computed_prop_name(
                    convert_span(computed.span),
                    self.convert_expr(*computed.expr),
                ))
            }
        }
    }

    fn convert_super_prop_expr(
        &self,
        super_prop: legacy::SuperPropExpr,
    ) -> experimental::SuperPropExpr<'a> {
        self.ast.super_prop_expr(
            convert_span(super_prop.span),
            self.ast.box_super(convert_span(super_prop.obj.span)),
            match super_prop.prop {
                legacy::SuperProp::Ident(ident) => {
                    experimental::SuperProp::Ident(self.boxed(self.convert_ident_name(ident)))
                }
                legacy::SuperProp::Computed(computed) => {
                    experimental::SuperProp::Computed(self.ast.box_computed_prop_name(
                        convert_span(computed.span),
                        self.convert_expr(*computed.expr),
                    ))
                }
            },
        )
    }

    fn convert_lit_expr(&self, lit: legacy::Lit) -> experimental::Expr<'a> {
        match lit {
            legacy::Lit::Str(str) => self.ast.expr_lit_str(
                convert_span(str.span),
                self.convert_wtf8_atom(str.value),
                self.convert_opt_atom(str.raw),
            ),
            legacy::Lit::Bool(bool_) => self
                .ast
                .expr_lit_bool(convert_span(bool_.span), bool_.value),
            legacy::Lit::Null(null) => self.ast.expr_lit_null(convert_span(null.span)),
            legacy::Lit::Num(number) => self.ast.expr_lit_number(
                convert_span(number.span),
                number.value,
                self.convert_opt_atom(number.raw),
            ),
            legacy::Lit::BigInt(big_int) => self.ast.expr_lit_big_int(
                convert_span(big_int.span),
                self.convert_atom(Atom::from(big_int.value.to_string())),
                self.convert_opt_atom(big_int.raw),
            ),
            legacy::Lit::Regex(regex) => self.ast.expr_lit_regex(
                convert_span(regex.span),
                self.convert_atom(regex.exp),
                self.convert_atom(regex.flags),
            ),
            legacy::Lit::JSXText(text) => self.ast.expr_lit_str(
                convert_span(text.span),
                self.convert_wtf8_atom(Wtf8Atom::from(text.value.as_ref())),
                Some(self.convert_atom(text.raw)),
            ),
        }
    }

    fn convert_tpl_element(&self, element: legacy::TplElement) -> experimental::TplElement<'a> {
        self.ast.tpl_element(
            convert_span(element.span),
            element.tail,
            element.cooked.map(|cooked| self.convert_wtf8_atom(cooked)),
            self.convert_atom(element.raw),
        )
    }

    fn convert_tpl(&self, tpl: legacy::Tpl) -> experimental::Tpl<'a> {
        self.ast.tpl(
            convert_span(tpl.span),
            self.vec(tpl.exprs, |this, expr| this.convert_expr(*expr)),
            self.vec(tpl.quasis, Self::convert_tpl_element),
        )
    }

    fn convert_simple_assign_target(
        &self,
        target: legacy::SimpleAssignTarget,
    ) -> experimental::SimpleAssignTarget<'a> {
        match target {
            legacy::SimpleAssignTarget::Ident(binding) => experimental::SimpleAssignTarget::Ident(
                self.boxed(self.convert_binding_ident(binding)),
            ),
            legacy::SimpleAssignTarget::Member(member) => experimental::SimpleAssignTarget::Member(
                self.boxed(self.convert_member_expr(member)),
            ),
            legacy::SimpleAssignTarget::SuperProp(super_prop) => {
                experimental::SimpleAssignTarget::SuperProp(
                    self.boxed(self.convert_super_prop_expr(super_prop)),
                )
            }
            legacy::SimpleAssignTarget::Paren(paren) => experimental::SimpleAssignTarget::Paren(
                self.ast
                    .box_paren_expr(convert_span(paren.span), self.convert_expr(*paren.expr)),
            ),
            legacy::SimpleAssignTarget::OptChain(opt_chain) => {
                experimental::SimpleAssignTarget::OptChain(
                    self.boxed(self.convert_opt_chain_expr(opt_chain)),
                )
            }
            legacy::SimpleAssignTarget::TsAs(ts) => self.convert_simple_assign_target(
                legacy::SimpleAssignTarget::try_from(ts.expr)
                    .unwrap_or_else(|_| legacy::Invalid { span: ts.span }.into()),
            ),
            legacy::SimpleAssignTarget::TsSatisfies(ts) => self.convert_simple_assign_target(
                legacy::SimpleAssignTarget::try_from(ts.expr)
                    .unwrap_or_else(|_| legacy::Invalid { span: ts.span }.into()),
            ),
            legacy::SimpleAssignTarget::TsNonNull(ts) => self.convert_simple_assign_target(
                legacy::SimpleAssignTarget::try_from(ts.expr)
                    .unwrap_or_else(|_| legacy::Invalid { span: ts.span }.into()),
            ),
            legacy::SimpleAssignTarget::TsTypeAssertion(ts) => self.convert_simple_assign_target(
                legacy::SimpleAssignTarget::try_from(ts.expr)
                    .unwrap_or_else(|_| legacy::Invalid { span: ts.span }.into()),
            ),
            legacy::SimpleAssignTarget::TsInstantiation(ts) => self.convert_simple_assign_target(
                legacy::SimpleAssignTarget::try_from(ts.expr)
                    .unwrap_or_else(|_| legacy::Invalid { span: ts.span }.into()),
            ),
            legacy::SimpleAssignTarget::Invalid(_) => {
                experimental::SimpleAssignTarget::Invalid(self.ast.box_invalid())
            }
        }
    }

    fn convert_expr_to_simple_assign_target(
        &self,
        expr: Box<legacy::Expr>,
    ) -> experimental::SimpleAssignTarget<'a> {
        match legacy::SimpleAssignTarget::try_from(expr) {
            Ok(target) => self.convert_simple_assign_target(target),
            Err(_) => experimental::SimpleAssignTarget::Invalid(self.ast.box_invalid()),
        }
    }

    fn convert_assign_target(
        &self,
        target: legacy::AssignTarget,
    ) -> experimental::AssignTarget<'a> {
        match target {
            legacy::AssignTarget::Simple(simple) => experimental::AssignTarget::Simple(
                self.boxed(self.convert_simple_assign_target(simple)),
            ),
            legacy::AssignTarget::Pat(pat) => match pat {
                legacy::AssignTargetPat::Array(array) => {
                    experimental::AssignTarget::Pat(self.boxed(
                        experimental::AssignTargetPat::Array(self.ast.box_array_pat(
                            convert_span(array.span),
                            self.vec(array.elems, |this, pat| {
                                pat.map(|pat| this.convert_pat(pat))
                            }),
                            array.optional,
                        )),
                    ))
                }
                legacy::AssignTargetPat::Object(object) => {
                    experimental::AssignTarget::Pat(self.boxed(
                        experimental::AssignTargetPat::Object(self.ast.box_object_pat(
                            convert_span(object.span),
                            self.vec(object.props, Self::convert_object_pat_prop),
                            object.optional,
                        )),
                    ))
                }
                legacy::AssignTargetPat::Invalid(_) => experimental::AssignTarget::Pat(self.boxed(
                    experimental::AssignTargetPat::Invalid(self.ast.box_invalid()),
                )),
            },
        }
    }

    fn convert_opt_chain_expr(
        &self,
        opt_chain: legacy::OptChainExpr,
    ) -> experimental::OptChainExpr<'a> {
        self.ast.opt_chain_expr(
            convert_span(opt_chain.span),
            opt_chain.optional,
            match *opt_chain.base {
                legacy::OptChainBase::Member(member) => {
                    experimental::OptChainBase::Member(self.boxed(self.convert_member_expr(member)))
                }
                legacy::OptChainBase::Call(call) => {
                    experimental::OptChainBase::Call(self.ast.box_opt_call(
                        convert_span(call.span),
                        self.convert_expr(*call.callee),
                        self.vec(call.args, Self::convert_expr_or_spread),
                    ))
                }
            },
        )
    }

    fn convert_jsx_object(&self, object: legacy::JSXObject) -> experimental::JSXObject<'a> {
        match object {
            legacy::JSXObject::JSXMemberExpr(member) => experimental::JSXObject::JSXMemberExpr(
                self.boxed(self.convert_jsx_member_expr(*member)),
            ),
            legacy::JSXObject::Ident(ident) => self
                .ast
                .jsx_object_ident(convert_span(ident.span), self.convert_atom(ident.sym)),
        }
    }

    fn convert_jsx_member_expr(
        &self,
        member: legacy::JSXMemberExpr,
    ) -> experimental::JSXMemberExpr<'a> {
        self.ast.jsx_member_expr(
            convert_span(member.span),
            self.convert_jsx_object(member.obj),
            self.boxed(self.convert_ident_name(member.prop)),
        )
    }

    fn convert_jsx_namespaced_name(
        &self,
        name: legacy::JSXNamespacedName,
    ) -> experimental::JSXNamespacedName<'a> {
        self.ast.jsx_namespaced_name(
            convert_span(name.span),
            self.boxed(self.convert_ident_name(name.ns)),
            self.boxed(self.convert_ident_name(name.name)),
        )
    }

    fn convert_jsx_expr(&self, expr: legacy::JSXExpr) -> experimental::JSXExpr<'a> {
        match expr {
            legacy::JSXExpr::JSXEmptyExpr(empty) => {
                self.ast.jsx_expr_jsx_empty_expr(convert_span(empty.span))
            }
            legacy::JSXExpr::Expr(expr) => {
                experimental::JSXExpr::Expr(self.boxed(self.convert_expr(*expr)))
            }
        }
    }

    fn convert_jsx_attr_name(&self, name: legacy::JSXAttrName) -> experimental::JSXAttrName<'a> {
        match name {
            legacy::JSXAttrName::Ident(ident) => {
                experimental::JSXAttrName::Ident(self.boxed(self.convert_ident_name(ident)))
            }
            legacy::JSXAttrName::JSXNamespacedName(name) => {
                experimental::JSXAttrName::JSXNamespacedName(
                    self.boxed(self.convert_jsx_namespaced_name(name)),
                )
            }
        }
    }

    fn convert_jsx_attr_value(
        &self,
        value: legacy::JSXAttrValue,
    ) -> experimental::JSXAttrValue<'a> {
        match value {
            legacy::JSXAttrValue::Str(str) => {
                experimental::JSXAttrValue::Str(self.boxed(self.convert_str(str)))
            }
            legacy::JSXAttrValue::JSXExprContainer(container) => {
                experimental::JSXAttrValue::JSXExprContainer(self.ast.box_jsx_expr_container(
                    convert_span(container.span),
                    self.convert_jsx_expr(container.expr),
                ))
            }
            legacy::JSXAttrValue::JSXElement(element) => experimental::JSXAttrValue::JSXElement(
                self.boxed(self.convert_jsx_element(*element)),
            ),
            legacy::JSXAttrValue::JSXFragment(fragment) => experimental::JSXAttrValue::JSXFragment(
                self.boxed(self.convert_jsx_fragment(fragment)),
            ),
        }
    }

    fn convert_jsx_attr_or_spread(
        &self,
        attr: legacy::JSXAttrOrSpread,
    ) -> experimental::JSXAttrOrSpread<'a> {
        match attr {
            legacy::JSXAttrOrSpread::JSXAttr(attr) => self.ast.jsx_attr_or_spread_jsx_attr(
                convert_span(attr.span),
                self.convert_jsx_attr_name(attr.name),
                attr.value.map(|value| self.convert_jsx_attr_value(value)),
            ),
            legacy::JSXAttrOrSpread::SpreadElement(spread) => {
                self.ast.jsx_attr_or_spread_spread_element(
                    convert_span(spread.dot3_token),
                    self.convert_expr(*spread.expr),
                )
            }
        }
    }

    fn convert_jsx_element_name(
        &self,
        name: legacy::JSXElementName,
    ) -> experimental::JSXElementName<'a> {
        match name {
            legacy::JSXElementName::Ident(ident) => {
                experimental::JSXElementName::Ident(self.boxed(self.convert_ident(ident)))
            }
            legacy::JSXElementName::JSXMemberExpr(member) => {
                experimental::JSXElementName::JSXMemberExpr(
                    self.boxed(self.convert_jsx_member_expr(member)),
                )
            }
            legacy::JSXElementName::JSXNamespacedName(name) => {
                experimental::JSXElementName::JSXNamespacedName(
                    self.boxed(self.convert_jsx_namespaced_name(name)),
                )
            }
        }
    }

    fn convert_jsx_opening_element(
        &self,
        opening: legacy::JSXOpeningElement,
    ) -> experimental::JSXOpeningElement<'a> {
        self.ast.jsx_opening_element(
            convert_span(opening.span),
            self.convert_jsx_element_name(opening.name),
            self.vec(opening.attrs, Self::convert_jsx_attr_or_spread),
            opening.self_closing,
        )
    }

    fn convert_jsx_closing_element(
        &self,
        closing: legacy::JSXClosingElement,
    ) -> experimental::JSXClosingElement<'a> {
        self.ast.jsx_closing_element(
            convert_span(closing.span),
            self.convert_jsx_element_name(closing.name),
        )
    }

    fn convert_jsx_element_child(
        &self,
        child: legacy::JSXElementChild,
    ) -> experimental::JSXElementChild<'a> {
        match child {
            legacy::JSXElementChild::JSXText(text) => {
                experimental::JSXElementChild::JSXText(self.ast.box_jsx_text(
                    convert_span(text.span),
                    self.convert_atom(text.value),
                    self.convert_atom(text.raw),
                ))
            }
            legacy::JSXElementChild::JSXExprContainer(container) => {
                experimental::JSXElementChild::JSXExprContainer(self.ast.box_jsx_expr_container(
                    convert_span(container.span),
                    self.convert_jsx_expr(container.expr),
                ))
            }
            legacy::JSXElementChild::JSXSpreadChild(spread) => {
                experimental::JSXElementChild::JSXSpreadChild(self.ast.box_jsx_spread_child(
                    convert_span(spread.span),
                    self.convert_expr(*spread.expr),
                ))
            }
            legacy::JSXElementChild::JSXElement(element) => {
                experimental::JSXElementChild::JSXElement(
                    self.boxed(self.convert_jsx_element(*element)),
                )
            }
            legacy::JSXElementChild::JSXFragment(fragment) => {
                experimental::JSXElementChild::JSXFragment(
                    self.boxed(self.convert_jsx_fragment(fragment)),
                )
            }
        }
    }

    fn convert_jsx_element(&self, element: legacy::JSXElement) -> experimental::JSXElement<'a> {
        self.ast.jsx_element(
            convert_span(element.span),
            self.boxed(self.convert_jsx_opening_element(element.opening)),
            self.vec(element.children, Self::convert_jsx_element_child),
            element
                .closing
                .map(|closing| self.boxed(self.convert_jsx_closing_element(closing))),
        )
    }

    fn convert_jsx_fragment(&self, fragment: legacy::JSXFragment) -> experimental::JSXFragment<'a> {
        self.ast.jsx_fragment(
            convert_span(fragment.span),
            self.ast
                .box_jsx_opening_fragment(convert_span(fragment.opening.span)),
            self.vec(fragment.children, Self::convert_jsx_element_child),
            self.ast
                .box_jsx_closing_fragment(convert_span(fragment.closing.span)),
        )
    }

    fn convert_atom(&self, atom: Atom) -> ExperimentalAtom<'a> {
        ExperimentalAtom::new_in(atom.as_ref(), self.allocator)
    }

    fn convert_opt_atom(&self, atom: Option<Atom>) -> Option<ExperimentalAtom<'a>> {
        atom.map(|atom| self.convert_atom(atom))
    }

    fn convert_wtf8_atom(&self, atom: Wtf8Atom) -> ExperimentalWtf8Atom<'a> {
        match atom.as_wtf8().as_str() {
            Some(s) => ExperimentalWtf8Atom::new_in(s, self.allocator),
            None => ExperimentalWtf8Atom::new_in(
                atom.as_wtf8().to_string_lossy().into_owned(),
                self.allocator,
            ),
        }
    }
}

fn convert_span(span: SwcSpan) -> experimental::Span {
    experimental::Span {
        start: span.lo.0,
        end: span.hi.0,
    }
}

fn convert_import_phase(phase: legacy::ImportPhase) -> experimental::ImportPhase {
    match phase {
        legacy::ImportPhase::Evaluation => experimental::ImportPhase::Evaluation,
        legacy::ImportPhase::Source => experimental::ImportPhase::Source,
        legacy::ImportPhase::Defer => experimental::ImportPhase::Defer,
    }
}

fn convert_unary_op(op: legacy::UnaryOp) -> experimental::UnaryOp {
    match op {
        legacy::UnaryOp::Minus => experimental::UnaryOp::Minus,
        legacy::UnaryOp::Plus => experimental::UnaryOp::Plus,
        legacy::UnaryOp::Bang => experimental::UnaryOp::Bang,
        legacy::UnaryOp::Tilde => experimental::UnaryOp::Tilde,
        legacy::UnaryOp::TypeOf => experimental::UnaryOp::TypeOf,
        legacy::UnaryOp::Void => experimental::UnaryOp::Void,
        legacy::UnaryOp::Delete => experimental::UnaryOp::Delete,
    }
}

fn convert_update_op(op: legacy::UpdateOp) -> experimental::UpdateOp {
    match op {
        legacy::UpdateOp::PlusPlus => experimental::UpdateOp::PlusPlus,
        legacy::UpdateOp::MinusMinus => experimental::UpdateOp::MinusMinus,
    }
}

fn convert_binary_op(op: legacy::BinaryOp) -> experimental::BinaryOp {
    match op {
        legacy::BinaryOp::EqEq => experimental::BinaryOp::EqEq,
        legacy::BinaryOp::NotEq => experimental::BinaryOp::NotEq,
        legacy::BinaryOp::EqEqEq => experimental::BinaryOp::EqEqEq,
        legacy::BinaryOp::NotEqEq => experimental::BinaryOp::NotEqEq,
        legacy::BinaryOp::Lt => experimental::BinaryOp::Lt,
        legacy::BinaryOp::LtEq => experimental::BinaryOp::LtEq,
        legacy::BinaryOp::Gt => experimental::BinaryOp::Gt,
        legacy::BinaryOp::GtEq => experimental::BinaryOp::GtEq,
        legacy::BinaryOp::LShift => experimental::BinaryOp::LShift,
        legacy::BinaryOp::RShift => experimental::BinaryOp::RShift,
        legacy::BinaryOp::ZeroFillRShift => experimental::BinaryOp::ZeroFillRShift,
        legacy::BinaryOp::Add => experimental::BinaryOp::Add,
        legacy::BinaryOp::Sub => experimental::BinaryOp::Sub,
        legacy::BinaryOp::Mul => experimental::BinaryOp::Mul,
        legacy::BinaryOp::Div => experimental::BinaryOp::Div,
        legacy::BinaryOp::Mod => experimental::BinaryOp::Mod,
        legacy::BinaryOp::BitOr => experimental::BinaryOp::BitOr,
        legacy::BinaryOp::BitXor => experimental::BinaryOp::BitXor,
        legacy::BinaryOp::BitAnd => experimental::BinaryOp::BitAnd,
        legacy::BinaryOp::LogicalOr => experimental::BinaryOp::LogicalOr,
        legacy::BinaryOp::LogicalAnd => experimental::BinaryOp::LogicalAnd,
        legacy::BinaryOp::In => experimental::BinaryOp::In,
        legacy::BinaryOp::InstanceOf => experimental::BinaryOp::InstanceOf,
        legacy::BinaryOp::Exp => experimental::BinaryOp::Exp,
        legacy::BinaryOp::NullishCoalescing => experimental::BinaryOp::NullishCoalescing,
    }
}

fn convert_assign_op(op: legacy::AssignOp) -> experimental::AssignOp {
    match op {
        legacy::AssignOp::Assign => experimental::AssignOp::Assign,
        legacy::AssignOp::AddAssign => experimental::AssignOp::AddAssign,
        legacy::AssignOp::SubAssign => experimental::AssignOp::SubAssign,
        legacy::AssignOp::MulAssign => experimental::AssignOp::MulAssign,
        legacy::AssignOp::DivAssign => experimental::AssignOp::DivAssign,
        legacy::AssignOp::ModAssign => experimental::AssignOp::ModAssign,
        legacy::AssignOp::LShiftAssign => experimental::AssignOp::LShiftAssign,
        legacy::AssignOp::RShiftAssign => experimental::AssignOp::RShiftAssign,
        legacy::AssignOp::ZeroFillRShiftAssign => experimental::AssignOp::ZeroFillRShiftAssign,
        legacy::AssignOp::BitOrAssign => experimental::AssignOp::BitOrAssign,
        legacy::AssignOp::BitXorAssign => experimental::AssignOp::BitXorAssign,
        legacy::AssignOp::BitAndAssign => experimental::AssignOp::BitAndAssign,
        legacy::AssignOp::ExpAssign => experimental::AssignOp::ExpAssign,
        legacy::AssignOp::AndAssign => experimental::AssignOp::AndAssign,
        legacy::AssignOp::OrAssign => experimental::AssignOp::OrAssign,
        legacy::AssignOp::NullishAssign => experimental::AssignOp::NullishAssign,
    }
}

fn convert_meta_prop_kind(kind: legacy::MetaPropKind) -> experimental::MetaPropKind {
    match kind {
        legacy::MetaPropKind::NewTarget => experimental::MetaPropKind::NewTarget,
        legacy::MetaPropKind::ImportMeta => experimental::MetaPropKind::ImportMeta,
    }
}

fn convert_method_kind(kind: legacy::MethodKind) -> experimental::MethodKind {
    match kind {
        legacy::MethodKind::Method => experimental::MethodKind::Method,
        legacy::MethodKind::Getter => experimental::MethodKind::Getter,
        legacy::MethodKind::Setter => experimental::MethodKind::Setter,
    }
}
