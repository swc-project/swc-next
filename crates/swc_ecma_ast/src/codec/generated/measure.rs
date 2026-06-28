#![allow(unused)]
use crate::{
    codec::{AstMeasure, Header},
    *,
};
use swc_experimental_allocator::boxed::Box;
impl<'a, M: AstMeasure> AstMeasure for Box<'a, M> {
    fn measure(&self, header: &mut Header) {
        (**self).measure(header)
    }
}
impl<M: AstMeasure> AstMeasure for Option<M> {
    fn measure(&self, header: &mut Header) {
        if let Some(it) = self {
            it.measure(header);
        }
    }
}
impl<'a> AstMeasure for Program<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Module(it) => it.measure(header),
            Self::Script(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for Module<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.body);
        header.opt_utf8(self.shebang);
    }
}
impl<'a> AstMeasure for Script<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.body);
        header.opt_utf8(self.shebang);
    }
}
impl<'a> AstMeasure for ModuleItem<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::ModuleDecl(it) => it.measure(header),
            Self::Stmt(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for ModuleDecl<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Import(it) => it.measure(header),
            Self::ExportDecl(it) => it.measure(header),
            Self::ExportNamed(it) => it.measure(header),
            Self::ExportDefaultDecl(it) => it.measure(header),
            Self::ExportDefaultExpr(it) => it.measure(header),
            Self::ExportAll(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for ImportDecl<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.specifiers);
        self.src.measure(header);
        self.with.measure(header);
    }
}
impl<'a> AstMeasure for ImportSpecifier<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Named(it) => it.measure(header),
            Self::Default(it) => it.measure(header),
            Self::Namespace(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for ImportNamedSpecifier<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.local.measure(header);
        self.imported.measure(header);
    }
}
impl<'a> AstMeasure for ImportDefaultSpecifier<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.local.measure(header);
    }
}
impl<'a> AstMeasure for ImportStarAsSpecifier<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.local.measure(header);
    }
}
impl<'a> AstMeasure for ExportDecl<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.decl.measure(header);
    }
}
impl<'a> AstMeasure for NamedExport<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.specifiers);
        self.src.measure(header);
        self.with.measure(header);
    }
}
impl<'a> AstMeasure for ExportSpecifier<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Namespace(it) => it.measure(header),
            Self::Default(it) => it.measure(header),
            Self::Named(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for ExportNamespaceSpecifier<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.name.measure(header);
    }
}
impl<'a> AstMeasure for ModuleExportName<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Ident(it) => it.measure(header),
            Self::Str(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for ExportDefaultSpecifier<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.exported.measure(header);
    }
}
impl<'a> AstMeasure for ExportNamedSpecifier<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.orig.measure(header);
        self.exported.measure(header);
    }
}
impl<'a> AstMeasure for ExportDefaultDecl<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.decl.measure(header);
    }
}
impl<'a> AstMeasure for DefaultDecl<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Class(it) => it.measure(header),
            Self::Fn(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for ExportDefaultExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.expr.measure(header);
    }
}
impl<'a> AstMeasure for ExportAll<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.src.measure(header);
        self.with.measure(header);
    }
}
impl<'a> AstMeasure for BlockStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.stmts);
    }
}
impl<'a> AstMeasure for Stmt<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Block(it) => it.measure(header),
            Self::Empty(it) => it.measure(header),
            Self::Debugger(it) => it.measure(header),
            Self::With(it) => it.measure(header),
            Self::Return(it) => it.measure(header),
            Self::Labeled(it) => it.measure(header),
            Self::Break(it) => it.measure(header),
            Self::Continue(it) => it.measure(header),
            Self::If(it) => it.measure(header),
            Self::Switch(it) => it.measure(header),
            Self::Throw(it) => it.measure(header),
            Self::Try(it) => it.measure(header),
            Self::While(it) => it.measure(header),
            Self::DoWhile(it) => it.measure(header),
            Self::For(it) => it.measure(header),
            Self::ForIn(it) => it.measure(header),
            Self::ForOf(it) => it.measure(header),
            Self::Decl(it) => it.measure(header),
            Self::Expr(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for ExprStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.expr.measure(header);
    }
}
impl AstMeasure for EmptyStmt {
    fn measure(&self, header: &mut Header) {
        header.node();
    }
}
impl AstMeasure for DebuggerStmt {
    fn measure(&self, header: &mut Header) {
        header.node();
    }
}
impl<'a> AstMeasure for WithStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.obj.measure(header);
        self.body.measure(header);
    }
}
impl<'a> AstMeasure for ReturnStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.arg.measure(header);
    }
}
impl<'a> AstMeasure for LabeledStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.label.measure(header);
        self.body.measure(header);
    }
}
impl<'a> AstMeasure for BreakStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.label.measure(header);
    }
}
impl<'a> AstMeasure for ContinueStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.label.measure(header);
    }
}
impl<'a> AstMeasure for IfStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.test.measure(header);
        self.cons.measure(header);
        self.alt.measure(header);
    }
}
impl<'a> AstMeasure for SwitchStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.discriminant.measure(header);
        header.nodes(&self.cases);
    }
}
impl<'a> AstMeasure for ThrowStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.arg.measure(header);
    }
}
impl<'a> AstMeasure for TryStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.block.measure(header);
        self.handler.measure(header);
        self.finalizer.measure(header);
    }
}
impl<'a> AstMeasure for WhileStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.test.measure(header);
        self.body.measure(header);
    }
}
impl<'a> AstMeasure for DoWhileStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.test.measure(header);
        self.body.measure(header);
    }
}
impl<'a> AstMeasure for ForStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.init.measure(header);
        self.test.measure(header);
        self.update.measure(header);
        self.body.measure(header);
    }
}
impl<'a> AstMeasure for ForInStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.left.measure(header);
        self.right.measure(header);
        self.body.measure(header);
    }
}
impl<'a> AstMeasure for ForOfStmt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.left.measure(header);
        self.right.measure(header);
        self.body.measure(header);
    }
}
impl<'a> AstMeasure for SwitchCase<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.test.measure(header);
        header.nodes(&self.cons);
    }
}
impl<'a> AstMeasure for CatchClause<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.param.measure(header);
        self.body.measure(header);
    }
}
impl<'a> AstMeasure for ForHead<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::VarDecl(it) => it.measure(header),
            Self::UsingDecl(it) => it.measure(header),
            Self::Pat(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for VarDeclOrExpr<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::VarDecl(it) => it.measure(header),
            Self::Expr(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for Decl<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Class(it) => it.measure(header),
            Self::Fn(it) => it.measure(header),
            Self::Var(it) => it.measure(header),
            Self::Using(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for FnDecl<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.ident.measure(header);
        self.function.measure(header);
    }
}
impl<'a> AstMeasure for ClassDecl<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.ident.measure(header);
        self.class.measure(header);
    }
}
impl<'a> AstMeasure for VarDecl<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.decls);
    }
}
impl<'a> AstMeasure for VarDeclarator<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.name.measure(header);
        self.init.measure(header);
    }
}
impl<'a> AstMeasure for UsingDecl<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.decls);
    }
}
impl<'a> AstMeasure for Expr<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::This(it) => it.measure(header),
            Self::Array(it) => it.measure(header),
            Self::Object(it) => it.measure(header),
            Self::Fn(it) => it.measure(header),
            Self::Unary(it) => it.measure(header),
            Self::Update(it) => it.measure(header),
            Self::Bin(it) => it.measure(header),
            Self::Assign(it) => it.measure(header),
            Self::Member(it) => it.measure(header),
            Self::SuperProp(it) => it.measure(header),
            Self::Cond(it) => it.measure(header),
            Self::Import(it) => it.measure(header),
            Self::Call(it) => it.measure(header),
            Self::New(it) => it.measure(header),
            Self::Seq(it) => it.measure(header),
            Self::Ident(it) => it.measure(header),
            Self::Lit(it) => it.measure(header),
            Self::Tpl(it) => it.measure(header),
            Self::TaggedTpl(it) => it.measure(header),
            Self::Arrow(it) => it.measure(header),
            Self::Class(it) => it.measure(header),
            Self::Yield(it) => it.measure(header),
            Self::MetaProp(it) => it.measure(header),
            Self::Await(it) => it.measure(header),
            Self::Paren(it) => it.measure(header),
            Self::JSXMember(it) => it.measure(header),
            Self::JSXNamespacedName(it) => it.measure(header),
            Self::JSXEmpty(it) => it.measure(header),
            Self::JSXElement(it) => it.measure(header),
            Self::JSXFragment(it) => it.measure(header),
            Self::PrivateName(it) => it.measure(header),
            Self::OptChain(it) => it.measure(header),
            Self::Invalid(it) => it.measure(header),
        }
    }
}
impl AstMeasure for ThisExpr {
    fn measure(&self, header: &mut Header) {
        header.node();
    }
}
impl<'a> AstMeasure for ArrayLit<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.elems);
    }
}
impl<'a> AstMeasure for ObjectLit<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.props);
    }
}
impl<'a> AstMeasure for PropOrSpread<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Spread(it) => it.measure(header),
            Self::Prop(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for SpreadElement<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.expr.measure(header);
    }
}
impl<'a> AstMeasure for UnaryExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.arg.measure(header);
    }
}
impl<'a> AstMeasure for UpdateExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.arg.measure(header);
    }
}
impl<'a> AstMeasure for BinExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.left.measure(header);
        self.right.measure(header);
    }
}
impl<'a> AstMeasure for FnExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.ident.measure(header);
        self.function.measure(header);
    }
}
impl<'a> AstMeasure for ClassExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.ident.measure(header);
        self.class.measure(header);
    }
}
impl<'a> AstMeasure for AssignExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.left.measure(header);
        self.right.measure(header);
    }
}
impl<'a> AstMeasure for MemberExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.obj.measure(header);
        self.prop.measure(header);
    }
}
impl<'a> AstMeasure for MemberProp<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Ident(it) => it.measure(header),
            Self::PrivateName(it) => it.measure(header),
            Self::Computed(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for SuperPropExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.obj.measure(header);
        self.prop.measure(header);
    }
}
impl<'a> AstMeasure for SuperProp<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Ident(it) => it.measure(header),
            Self::Computed(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for CondExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.test.measure(header);
        self.cons.measure(header);
        self.alt.measure(header);
    }
}
impl<'a> AstMeasure for ImportExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.source.measure(header);
        self.options.measure(header);
    }
}
impl<'a> AstMeasure for CallExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.callee.measure(header);
        header.nodes(&self.args);
    }
}
impl<'a> AstMeasure for NewExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.callee.measure(header);
        header.nodes(&self.args);
    }
}
impl<'a> AstMeasure for SeqExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.exprs);
    }
}
impl<'a> AstMeasure for ArrowExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.params);
        self.body.measure(header);
    }
}
impl<'a> AstMeasure for YieldExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.arg.measure(header);
    }
}
impl AstMeasure for MetaPropExpr {
    fn measure(&self, header: &mut Header) {
        header.node();
    }
}
impl<'a> AstMeasure for AwaitExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.arg.measure(header);
    }
}
impl<'a> AstMeasure for Tpl<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.exprs);
        header.nodes(&self.quasis);
    }
}
impl<'a> AstMeasure for TaggedTpl<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.tag.measure(header);
        self.tpl.measure(header);
    }
}
impl<'a> AstMeasure for TplElement<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.opt_wtf8(self.cooked);
        header.utf8(self.raw);
    }
}
impl<'a> AstMeasure for ParenExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.expr.measure(header);
    }
}
impl<'a> AstMeasure for Callee<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Super(it) => it.measure(header),
            Self::Expr(it) => it.measure(header),
        }
    }
}
impl AstMeasure for Super {
    fn measure(&self, header: &mut Header) {
        header.node();
    }
}
impl<'a> AstMeasure for ExprOrSpread<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.expr.measure(header);
    }
}
impl<'a> AstMeasure for BlockStmtOrExpr<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::BlockStmt(it) => it.measure(header),
            Self::Expr(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for AssignTarget<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Simple(it) => it.measure(header),
            Self::Pat(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for AssignTargetPat<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Array(it) => it.measure(header),
            Self::Object(it) => it.measure(header),
            Self::Invalid(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for SimpleAssignTarget<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Ident(it) => it.measure(header),
            Self::Member(it) => it.measure(header),
            Self::SuperProp(it) => it.measure(header),
            Self::Paren(it) => it.measure(header),
            Self::OptChain(it) => it.measure(header),
            Self::Invalid(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for OptChainExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.base.measure(header);
    }
}
impl<'a> AstMeasure for OptChainBase<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Member(it) => it.measure(header),
            Self::Call(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for OptCall<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.callee.measure(header);
        header.nodes(&self.args);
    }
}
impl AstMeasure for Invalid {
    fn measure(&self, header: &mut Header) {
        header.node();
    }
}
impl<'a> AstMeasure for Function<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.params);
        header.nodes(&self.decorators);
        self.body.measure(header);
    }
}
impl<'a> AstMeasure for Param<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.decorators);
        self.pat.measure(header);
    }
}
impl<'a> AstMeasure for ParamOrTsParamProp<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Param(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for Class<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.decorators);
        header.nodes(&self.body);
        self.super_class.measure(header);
    }
}
impl<'a> AstMeasure for ClassMember<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Constructor(it) => it.measure(header),
            Self::Method(it) => it.measure(header),
            Self::PrivateMethod(it) => it.measure(header),
            Self::ClassProp(it) => it.measure(header),
            Self::PrivateProp(it) => it.measure(header),
            Self::Empty(it) => it.measure(header),
            Self::StaticBlock(it) => it.measure(header),
            Self::AutoAccessor(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for ClassProp<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.key.measure(header);
        self.value.measure(header);
        header.nodes(&self.decorators);
    }
}
impl<'a> AstMeasure for PrivateProp<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.key.measure(header);
        self.value.measure(header);
        header.nodes(&self.decorators);
    }
}
impl<'a> AstMeasure for ClassMethod<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.key.measure(header);
        self.function.measure(header);
    }
}
impl<'a> AstMeasure for PrivateMethod<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.key.measure(header);
        self.function.measure(header);
    }
}
impl<'a> AstMeasure for Constructor<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.key.measure(header);
        header.nodes(&self.params);
        self.body.measure(header);
    }
}
impl<'a> AstMeasure for Decorator<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.expr.measure(header);
    }
}
impl<'a> AstMeasure for StaticBlock<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.body.measure(header);
    }
}
impl<'a> AstMeasure for Key<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Private(it) => it.measure(header),
            Self::Public(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for AutoAccessor<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.key.measure(header);
        self.value.measure(header);
        header.nodes(&self.decorators);
    }
}
impl<'a> AstMeasure for Prop<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Shorthand(it) => it.measure(header),
            Self::KeyValue(it) => it.measure(header),
            Self::Assign(it) => it.measure(header),
            Self::Getter(it) => it.measure(header),
            Self::Setter(it) => it.measure(header),
            Self::Method(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for KeyValueProp<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.key.measure(header);
        self.value.measure(header);
    }
}
impl<'a> AstMeasure for AssignProp<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.key.measure(header);
        self.value.measure(header);
    }
}
impl<'a> AstMeasure for GetterProp<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.key.measure(header);
        self.body.measure(header);
    }
}
impl<'a> AstMeasure for SetterProp<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.key.measure(header);
        self.this_param.measure(header);
        self.param.measure(header);
        self.body.measure(header);
    }
}
impl<'a> AstMeasure for MethodProp<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.key.measure(header);
        self.function.measure(header);
    }
}
impl<'a> AstMeasure for PropName<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Ident(it) => it.measure(header),
            Self::Str(it) => it.measure(header),
            Self::Num(it) => it.measure(header),
            Self::Computed(it) => it.measure(header),
            Self::BigInt(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for ComputedPropName<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.expr.measure(header);
    }
}
impl<'a> AstMeasure for Pat<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Ident(it) => it.measure(header),
            Self::Array(it) => it.measure(header),
            Self::Rest(it) => it.measure(header),
            Self::Object(it) => it.measure(header),
            Self::Assign(it) => it.measure(header),
            Self::Invalid(it) => it.measure(header),
            Self::Expr(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for ArrayPat<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.elems);
    }
}
impl<'a> AstMeasure for ObjectPat<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.nodes(&self.props);
    }
}
impl<'a> AstMeasure for AssignPat<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.left.measure(header);
        self.right.measure(header);
    }
}
impl<'a> AstMeasure for RestPat<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.arg.measure(header);
    }
}
impl<'a> AstMeasure for ObjectPatProp<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::KeyValue(it) => it.measure(header),
            Self::Assign(it) => it.measure(header),
            Self::Rest(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for KeyValuePatProp<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.key.measure(header);
        self.value.measure(header);
    }
}
impl<'a> AstMeasure for AssignPatProp<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.key.measure(header);
        self.value.measure(header);
    }
}
impl<'a> AstMeasure for Ident<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.utf8(self.sym);
    }
}
impl<'a> AstMeasure for IdentName<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.utf8(self.sym);
    }
}
impl<'a> AstMeasure for PrivateName<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.utf8(self.name);
    }
}
impl<'a> AstMeasure for BindingIdent<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.id.measure(header);
    }
}
impl<'a> AstMeasure for Lit<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Str(it) => it.measure(header),
            Self::Bool(it) => it.measure(header),
            Self::Null(it) => it.measure(header),
            Self::Num(it) => it.measure(header),
            Self::BigInt(it) => it.measure(header),
            Self::Regex(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for Str<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.wtf8(self.value);
        header.opt_utf8(self.raw);
    }
}
impl AstMeasure for Bool {
    fn measure(&self, header: &mut Header) {
        header.node();
    }
}
impl AstMeasure for Null {
    fn measure(&self, header: &mut Header) {
        header.node();
    }
}
impl<'a> AstMeasure for Number<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.opt_utf8(self.raw);
    }
}
impl<'a> AstMeasure for BigInt<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.utf8(self.value);
        header.opt_utf8(self.raw);
    }
}
impl<'a> AstMeasure for Regex<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.utf8(self.exp);
        header.utf8(self.flags);
    }
}
impl<'a> AstMeasure for JSXObject<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::JSXMemberExpr(it) => it.measure(header),
            Self::Ident(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for JSXMemberExpr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.obj.measure(header);
        self.prop.measure(header);
    }
}
impl<'a> AstMeasure for JSXNamespacedName<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.ns.measure(header);
        self.name.measure(header);
    }
}
impl AstMeasure for JSXEmptyExpr {
    fn measure(&self, header: &mut Header) {
        header.node();
    }
}
impl<'a> AstMeasure for JSXExprContainer<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.expr.measure(header);
    }
}
impl<'a> AstMeasure for JSXExpr<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::JSXEmptyExpr(it) => it.measure(header),
            Self::Expr(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for JSXSpreadChild<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.expr.measure(header);
    }
}
impl<'a> AstMeasure for JSXElementName<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Ident(it) => it.measure(header),
            Self::JSXMemberExpr(it) => it.measure(header),
            Self::JSXNamespacedName(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for JSXOpeningElement<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.name.measure(header);
        header.nodes(&self.attrs);
    }
}
impl<'a> AstMeasure for JSXAttrOrSpread<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::JSXAttr(it) => it.measure(header),
            Self::SpreadElement(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for JSXClosingElement<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.name.measure(header);
    }
}
impl<'a> AstMeasure for JSXAttr<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.name.measure(header);
        self.value.measure(header);
    }
}
impl<'a> AstMeasure for JSXAttrName<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Ident(it) => it.measure(header),
            Self::JSXNamespacedName(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for JSXAttrValue<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::Str(it) => it.measure(header),
            Self::JSXExprContainer(it) => it.measure(header),
            Self::JSXElement(it) => it.measure(header),
            Self::JSXFragment(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for JSXText<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        header.utf8(self.value);
        header.utf8(self.raw);
    }
}
impl<'a> AstMeasure for JSXElement<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.opening.measure(header);
        header.nodes(&self.children);
        self.closing.measure(header);
    }
}
impl<'a> AstMeasure for JSXElementChild<'a> {
    fn measure(&self, header: &mut Header) {
        match self {
            Self::JSXText(it) => it.measure(header),
            Self::JSXExprContainer(it) => it.measure(header),
            Self::JSXSpreadChild(it) => it.measure(header),
            Self::JSXElement(it) => it.measure(header),
            Self::JSXFragment(it) => it.measure(header),
        }
    }
}
impl<'a> AstMeasure for JSXFragment<'a> {
    fn measure(&self, header: &mut Header) {
        header.node();
        self.opening.measure(header);
        header.nodes(&self.children);
        self.closing.measure(header);
    }
}
impl AstMeasure for JSXOpeningFragment {
    fn measure(&self, header: &mut Header) {
        header.node();
    }
}
impl AstMeasure for JSXClosingFragment {
    fn measure(&self, header: &mut Header) {
        header.node();
    }
}
