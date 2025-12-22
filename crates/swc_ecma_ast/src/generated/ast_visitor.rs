#![allow(unused, clippy::useless_conversion, clippy::single_match)]
use crate::{Ast, NodeKind, ast::*, node_id::*};
use swc_core::common::Span;
pub trait Visit {
    fn ast(&self) -> &Ast;
    #[inline]
    fn enter_node(&mut self, node_id: NodeId) {}
    #[inline]
    fn leave_node(&mut self, node_id: NodeId) {}
    #[inline]
    fn visit_program(&mut self, node: Program) {
        <Program as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_module(&mut self, node: Module) {
        self.enter_node(node.node_id());
        <Module as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_script(&mut self, node: Script) {
        self.enter_node(node.node_id());
        <Script as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_module_item(&mut self, node: ModuleItem) {
        <ModuleItem as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_module_decl(&mut self, node: ModuleDecl) {
        <ModuleDecl as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_import_decl(&mut self, node: ImportDecl) {
        self.enter_node(node.node_id());
        <ImportDecl as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_import_specifier(&mut self, node: ImportSpecifier) {
        <ImportSpecifier as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_import_named_specifier(&mut self, node: ImportNamedSpecifier) {
        self.enter_node(node.node_id());
        <ImportNamedSpecifier as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_import_default_specifier(&mut self, node: ImportDefaultSpecifier) {
        self.enter_node(node.node_id());
        <ImportDefaultSpecifier as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_import_star_as_specifier(&mut self, node: ImportStarAsSpecifier) {
        self.enter_node(node.node_id());
        <ImportStarAsSpecifier as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_export_decl(&mut self, node: ExportDecl) {
        self.enter_node(node.node_id());
        <ExportDecl as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_named_export(&mut self, node: NamedExport) {
        self.enter_node(node.node_id());
        <NamedExport as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_export_specifier(&mut self, node: ExportSpecifier) {
        <ExportSpecifier as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_export_namespace_specifier(&mut self, node: ExportNamespaceSpecifier) {
        self.enter_node(node.node_id());
        <ExportNamespaceSpecifier as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_module_export_name(&mut self, node: ModuleExportName) {
        <ModuleExportName as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_export_default_specifier(&mut self, node: ExportDefaultSpecifier) {
        self.enter_node(node.node_id());
        <ExportDefaultSpecifier as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_export_named_specifier(&mut self, node: ExportNamedSpecifier) {
        self.enter_node(node.node_id());
        <ExportNamedSpecifier as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_export_default_decl(&mut self, node: ExportDefaultDecl) {
        self.enter_node(node.node_id());
        <ExportDefaultDecl as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_default_decl(&mut self, node: DefaultDecl) {
        <DefaultDecl as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_export_default_expr(&mut self, node: ExportDefaultExpr) {
        self.enter_node(node.node_id());
        <ExportDefaultExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_export_all(&mut self, node: ExportAll) {
        self.enter_node(node.node_id());
        <ExportAll as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_block_stmt(&mut self, node: BlockStmt) {
        self.enter_node(node.node_id());
        <BlockStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_stmt(&mut self, node: Stmt) {
        <Stmt as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_expr_stmt(&mut self, node: ExprStmt) {
        self.enter_node(node.node_id());
        <ExprStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_empty_stmt(&mut self, node: EmptyStmt) {
        self.enter_node(node.node_id());
        <EmptyStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_debugger_stmt(&mut self, node: DebuggerStmt) {
        self.enter_node(node.node_id());
        <DebuggerStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_with_stmt(&mut self, node: WithStmt) {
        self.enter_node(node.node_id());
        <WithStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_return_stmt(&mut self, node: ReturnStmt) {
        self.enter_node(node.node_id());
        <ReturnStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_labeled_stmt(&mut self, node: LabeledStmt) {
        self.enter_node(node.node_id());
        <LabeledStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_break_stmt(&mut self, node: BreakStmt) {
        self.enter_node(node.node_id());
        <BreakStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_continue_stmt(&mut self, node: ContinueStmt) {
        self.enter_node(node.node_id());
        <ContinueStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_if_stmt(&mut self, node: IfStmt) {
        self.enter_node(node.node_id());
        <IfStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_switch_stmt(&mut self, node: SwitchStmt) {
        self.enter_node(node.node_id());
        <SwitchStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_throw_stmt(&mut self, node: ThrowStmt) {
        self.enter_node(node.node_id());
        <ThrowStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_try_stmt(&mut self, node: TryStmt) {
        self.enter_node(node.node_id());
        <TryStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_while_stmt(&mut self, node: WhileStmt) {
        self.enter_node(node.node_id());
        <WhileStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_do_while_stmt(&mut self, node: DoWhileStmt) {
        self.enter_node(node.node_id());
        <DoWhileStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_for_stmt(&mut self, node: ForStmt) {
        self.enter_node(node.node_id());
        <ForStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_for_in_stmt(&mut self, node: ForInStmt) {
        self.enter_node(node.node_id());
        <ForInStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_for_of_stmt(&mut self, node: ForOfStmt) {
        self.enter_node(node.node_id());
        <ForOfStmt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_switch_case(&mut self, node: SwitchCase) {
        self.enter_node(node.node_id());
        <SwitchCase as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_catch_clause(&mut self, node: CatchClause) {
        self.enter_node(node.node_id());
        <CatchClause as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_for_head(&mut self, node: ForHead) {
        <ForHead as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_var_decl_or_expr(&mut self, node: VarDeclOrExpr) {
        <VarDeclOrExpr as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_decl(&mut self, node: Decl) {
        <Decl as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_fn_decl(&mut self, node: FnDecl) {
        self.enter_node(node.node_id());
        <FnDecl as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_class_decl(&mut self, node: ClassDecl) {
        self.enter_node(node.node_id());
        <ClassDecl as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_var_decl(&mut self, node: VarDecl) {
        self.enter_node(node.node_id());
        <VarDecl as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_var_declarator(&mut self, node: VarDeclarator) {
        self.enter_node(node.node_id());
        <VarDeclarator as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_using_decl(&mut self, node: UsingDecl) {
        self.enter_node(node.node_id());
        <UsingDecl as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_expr(&mut self, node: Expr) {
        <Expr as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_this_expr(&mut self, node: ThisExpr) {
        self.enter_node(node.node_id());
        <ThisExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_array_lit(&mut self, node: ArrayLit) {
        self.enter_node(node.node_id());
        <ArrayLit as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_object_lit(&mut self, node: ObjectLit) {
        self.enter_node(node.node_id());
        <ObjectLit as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_prop_or_spread(&mut self, node: PropOrSpread) {
        <PropOrSpread as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_spread_element(&mut self, node: SpreadElement) {
        self.enter_node(node.node_id());
        <SpreadElement as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_unary_expr(&mut self, node: UnaryExpr) {
        self.enter_node(node.node_id());
        <UnaryExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_update_expr(&mut self, node: UpdateExpr) {
        self.enter_node(node.node_id());
        <UpdateExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_bin_expr(&mut self, node: BinExpr) {
        self.enter_node(node.node_id());
        <BinExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_fn_expr(&mut self, node: FnExpr) {
        self.enter_node(node.node_id());
        <FnExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_class_expr(&mut self, node: ClassExpr) {
        self.enter_node(node.node_id());
        <ClassExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_assign_expr(&mut self, node: AssignExpr) {
        self.enter_node(node.node_id());
        <AssignExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_member_expr(&mut self, node: MemberExpr) {
        self.enter_node(node.node_id());
        <MemberExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_member_prop(&mut self, node: MemberProp) {
        <MemberProp as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_super_prop_expr(&mut self, node: SuperPropExpr) {
        self.enter_node(node.node_id());
        <SuperPropExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_super_prop(&mut self, node: SuperProp) {
        <SuperProp as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_cond_expr(&mut self, node: CondExpr) {
        self.enter_node(node.node_id());
        <CondExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_call_expr(&mut self, node: CallExpr) {
        self.enter_node(node.node_id());
        <CallExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_new_expr(&mut self, node: NewExpr) {
        self.enter_node(node.node_id());
        <NewExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_seq_expr(&mut self, node: SeqExpr) {
        self.enter_node(node.node_id());
        <SeqExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_arrow_expr(&mut self, node: ArrowExpr) {
        self.enter_node(node.node_id());
        <ArrowExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_yield_expr(&mut self, node: YieldExpr) {
        self.enter_node(node.node_id());
        <YieldExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_meta_prop_expr(&mut self, node: MetaPropExpr) {
        self.enter_node(node.node_id());
        <MetaPropExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_await_expr(&mut self, node: AwaitExpr) {
        self.enter_node(node.node_id());
        <AwaitExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_tpl(&mut self, node: Tpl) {
        self.enter_node(node.node_id());
        <Tpl as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_tagged_tpl(&mut self, node: TaggedTpl) {
        self.enter_node(node.node_id());
        <TaggedTpl as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_tpl_element(&mut self, node: TplElement) {
        self.enter_node(node.node_id());
        <TplElement as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_paren_expr(&mut self, node: ParenExpr) {
        self.enter_node(node.node_id());
        <ParenExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_callee(&mut self, node: Callee) {
        <Callee as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_super(&mut self, node: Super) {
        self.enter_node(node.node_id());
        <Super as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_import(&mut self, node: Import) {
        self.enter_node(node.node_id());
        <Import as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_expr_or_spread(&mut self, node: ExprOrSpread) {
        self.enter_node(node.node_id());
        <ExprOrSpread as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_spread_dot_3_token(&mut self, node: SpreadDot3Token) {
        self.enter_node(node.node_id());
        <SpreadDot3Token as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_block_stmt_or_expr(&mut self, node: BlockStmtOrExpr) {
        <BlockStmtOrExpr as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_assign_target(&mut self, node: AssignTarget) {
        <AssignTarget as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_assign_target_pat(&mut self, node: AssignTargetPat) {
        <AssignTargetPat as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_simple_assign_target(&mut self, node: SimpleAssignTarget) {
        <SimpleAssignTarget as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_chain_expr(&mut self, node: OptChainExpr) {
        self.enter_node(node.node_id());
        <OptChainExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_opt_chain_base(&mut self, node: OptChainBase) {
        <OptChainBase as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_call(&mut self, node: OptCall) {
        self.enter_node(node.node_id());
        <OptCall as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_invalid(&mut self, node: Invalid) {
        self.enter_node(node.node_id());
        <Invalid as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_function(&mut self, node: Function) {
        self.enter_node(node.node_id());
        <Function as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_param(&mut self, node: Param) {
        self.enter_node(node.node_id());
        <Param as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_param_or_ts_param_prop(&mut self, node: ParamOrTsParamProp) {
        <ParamOrTsParamProp as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_class(&mut self, node: Class) {
        self.enter_node(node.node_id());
        <Class as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_class_member(&mut self, node: ClassMember) {
        <ClassMember as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_class_prop(&mut self, node: ClassProp) {
        self.enter_node(node.node_id());
        <ClassProp as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_private_prop(&mut self, node: PrivateProp) {
        self.enter_node(node.node_id());
        <PrivateProp as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_class_method(&mut self, node: ClassMethod) {
        self.enter_node(node.node_id());
        <ClassMethod as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_private_method(&mut self, node: PrivateMethod) {
        self.enter_node(node.node_id());
        <PrivateMethod as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_constructor(&mut self, node: Constructor) {
        self.enter_node(node.node_id());
        <Constructor as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_decorator(&mut self, node: Decorator) {
        self.enter_node(node.node_id());
        <Decorator as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_static_block(&mut self, node: StaticBlock) {
        self.enter_node(node.node_id());
        <StaticBlock as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_key(&mut self, node: Key) {
        <Key as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_auto_accessor(&mut self, node: AutoAccessor) {
        self.enter_node(node.node_id());
        <AutoAccessor as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_prop(&mut self, node: Prop) {
        <Prop as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_key_value_prop(&mut self, node: KeyValueProp) {
        self.enter_node(node.node_id());
        <KeyValueProp as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_assign_prop(&mut self, node: AssignProp) {
        self.enter_node(node.node_id());
        <AssignProp as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_getter_prop(&mut self, node: GetterProp) {
        self.enter_node(node.node_id());
        <GetterProp as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_setter_prop(&mut self, node: SetterProp) {
        self.enter_node(node.node_id());
        <SetterProp as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_method_prop(&mut self, node: MethodProp) {
        self.enter_node(node.node_id());
        <MethodProp as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_prop_name(&mut self, node: PropName) {
        <PropName as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_computed_prop_name(&mut self, node: ComputedPropName) {
        self.enter_node(node.node_id());
        <ComputedPropName as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_pat(&mut self, node: Pat) {
        <Pat as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_array_pat(&mut self, node: ArrayPat) {
        self.enter_node(node.node_id());
        <ArrayPat as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_object_pat(&mut self, node: ObjectPat) {
        self.enter_node(node.node_id());
        <ObjectPat as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_assign_pat(&mut self, node: AssignPat) {
        self.enter_node(node.node_id());
        <AssignPat as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_rest_pat(&mut self, node: RestPat) {
        self.enter_node(node.node_id());
        <RestPat as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_object_pat_prop(&mut self, node: ObjectPatProp) {
        <ObjectPatProp as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_key_value_pat_prop(&mut self, node: KeyValuePatProp) {
        self.enter_node(node.node_id());
        <KeyValuePatProp as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_assign_pat_prop(&mut self, node: AssignPatProp) {
        self.enter_node(node.node_id());
        <AssignPatProp as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ident(&mut self, node: Ident) {
        self.enter_node(node.node_id());
        <Ident as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ident_name(&mut self, node: IdentName) {
        self.enter_node(node.node_id());
        <IdentName as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_private_name(&mut self, node: PrivateName) {
        self.enter_node(node.node_id());
        <PrivateName as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_binding_ident(&mut self, node: BindingIdent) {
        self.enter_node(node.node_id());
        <BindingIdent as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_lit(&mut self, node: Lit) {
        <Lit as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_str(&mut self, node: Str) {
        self.enter_node(node.node_id());
        <Str as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_bool(&mut self, node: Bool) {
        self.enter_node(node.node_id());
        <Bool as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_null(&mut self, node: Null) {
        self.enter_node(node.node_id());
        <Null as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_number(&mut self, node: Number) {
        self.enter_node(node.node_id());
        <Number as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_big_int(&mut self, node: BigInt) {
        self.enter_node(node.node_id());
        <BigInt as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_regex(&mut self, node: Regex) {
        self.enter_node(node.node_id());
        <Regex as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_object(&mut self, node: JSXObject) {
        <JSXObject as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_member_expr(&mut self, node: JSXMemberExpr) {
        self.enter_node(node.node_id());
        <JSXMemberExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_namespaced_name(&mut self, node: JSXNamespacedName) {
        self.enter_node(node.node_id());
        <JSXNamespacedName as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_empty_expr(&mut self, node: JSXEmptyExpr) {
        self.enter_node(node.node_id());
        <JSXEmptyExpr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_expr_container(&mut self, node: JSXExprContainer) {
        self.enter_node(node.node_id());
        <JSXExprContainer as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_expr(&mut self, node: JSXExpr) {
        <JSXExpr as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_spread_child(&mut self, node: JSXSpreadChild) {
        self.enter_node(node.node_id());
        <JSXSpreadChild as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_element_name(&mut self, node: JSXElementName) {
        <JSXElementName as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_opening_element(&mut self, node: JSXOpeningElement) {
        self.enter_node(node.node_id());
        <JSXOpeningElement as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_attr_or_spread(&mut self, node: JSXAttrOrSpread) {
        <JSXAttrOrSpread as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_closing_element(&mut self, node: JSXClosingElement) {
        self.enter_node(node.node_id());
        <JSXClosingElement as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_attr(&mut self, node: JSXAttr) {
        self.enter_node(node.node_id());
        <JSXAttr as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_attr_name(&mut self, node: JSXAttrName) {
        <JSXAttrName as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_attr_value(&mut self, node: JSXAttrValue) {
        <JSXAttrValue as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_text(&mut self, node: JSXText) {
        self.enter_node(node.node_id());
        <JSXText as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_element(&mut self, node: JSXElement) {
        self.enter_node(node.node_id());
        <JSXElement as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_element_child(&mut self, node: JSXElementChild) {
        <JSXElementChild as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_fragment(&mut self, node: JSXFragment) {
        self.enter_node(node.node_id());
        <JSXFragment as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_opening_fragment(&mut self, node: JSXOpeningFragment) {
        self.enter_node(node.node_id());
        <JSXOpeningFragment as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_closing_fragment(&mut self, node: JSXClosingFragment) {
        self.enter_node(node.node_id());
        <JSXClosingFragment as VisitWith<Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_module_items(&mut self, node: TypedSubRange<ModuleItem>) {
        <TypedSubRange<ModuleItem> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_stmts(&mut self, node: TypedSubRange<Stmt>) {
        <TypedSubRange<Stmt> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_import_specifiers(&mut self, node: TypedSubRange<ImportSpecifier>) {
        <TypedSubRange<ImportSpecifier> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_object_lit(&mut self, node: Option<ObjectLit>) {
        <Option<ObjectLit> as VisitWith<Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_module_export_name(&mut self, node: Option<ModuleExportName>) {
        <Option<ModuleExportName> as VisitWith<Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_export_specifiers(&mut self, node: TypedSubRange<ExportSpecifier>) {
        <TypedSubRange<ExportSpecifier> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_str(&mut self, node: Option<Str>) {
        <Option<Str> as VisitWith<Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_expr(&mut self, node: Option<Expr>) {
        <Option<Expr> as VisitWith<Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_ident(&mut self, node: Option<Ident>) {
        <Option<Ident> as VisitWith<Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_stmt(&mut self, node: Option<Stmt>) {
        <Option<Stmt> as VisitWith<Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_switch_cases(&mut self, node: TypedSubRange<SwitchCase>) {
        <TypedSubRange<SwitchCase> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_catch_clause(&mut self, node: Option<CatchClause>) {
        <Option<CatchClause> as VisitWith<Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_block_stmt(&mut self, node: Option<BlockStmt>) {
        <Option<BlockStmt> as VisitWith<Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_var_decl_or_expr(&mut self, node: Option<VarDeclOrExpr>) {
        <Option<VarDeclOrExpr> as VisitWith<Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_pat(&mut self, node: Option<Pat>) {
        <Option<Pat> as VisitWith<Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_var_declarators(&mut self, node: TypedSubRange<VarDeclarator>) {
        <TypedSubRange<VarDeclarator> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_expr_or_spread(&mut self, node: Option<ExprOrSpread>) {
        <Option<ExprOrSpread> as VisitWith<Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_vec_expr_or_spreads(&mut self, node: TypedSubRange<Option<ExprOrSpread>>) {
        <TypedSubRange<Option<ExprOrSpread>> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_prop_or_spreads(&mut self, node: TypedSubRange<PropOrSpread>) {
        <TypedSubRange<PropOrSpread> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_expr_or_spreads(&mut self, node: TypedSubRange<ExprOrSpread>) {
        <TypedSubRange<ExprOrSpread> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_expr_or_spreads(&mut self, node: Option<TypedSubRange<ExprOrSpread>>) {
        <Option<TypedSubRange<ExprOrSpread>> as VisitWith<Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_exprs(&mut self, node: TypedSubRange<Expr>) {
        <TypedSubRange<Expr> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_pats(&mut self, node: TypedSubRange<Pat>) {
        <TypedSubRange<Pat> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_tpl_elements(&mut self, node: TypedSubRange<TplElement>) {
        <TypedSubRange<TplElement> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_spread_dot_3_token(&mut self, node: Option<SpreadDot3Token>) {
        <Option<SpreadDot3Token> as VisitWith<Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_params(&mut self, node: TypedSubRange<Param>) {
        <TypedSubRange<Param> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_decorators(&mut self, node: TypedSubRange<Decorator>) {
        <TypedSubRange<Decorator> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_class_members(&mut self, node: TypedSubRange<ClassMember>) {
        <TypedSubRange<ClassMember> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_param_or_ts_param_props(&mut self, node: TypedSubRange<ParamOrTsParamProp>) {
        <TypedSubRange<ParamOrTsParamProp> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_vec_pats(&mut self, node: TypedSubRange<Option<Pat>>) {
        <TypedSubRange<Option<Pat>> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_object_pat_props(&mut self, node: TypedSubRange<ObjectPatProp>) {
        <TypedSubRange<ObjectPatProp> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_attr_or_spreads(&mut self, node: TypedSubRange<JSXAttrOrSpread>) {
        <TypedSubRange<JSXAttrOrSpread> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_jsx_attr_value(&mut self, node: Option<JSXAttrValue>) {
        <Option<JSXAttrValue> as VisitWith<Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_jsx_element_childs(&mut self, node: TypedSubRange<JSXElementChild>) {
        <TypedSubRange<JSXElementChild> as VisitWith<Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_jsx_closing_element(&mut self, node: Option<JSXClosingElement>) {
        <Option<JSXClosingElement> as VisitWith<Self>>::visit_children_with(node, self);
    }
}
pub trait VisitWith<V: ?Sized + Visit> {
    fn visit_with(self, visitor: &mut V);
    fn visit_children_with(self, visitor: &mut V);
}
impl<V: ?Sized + Visit> VisitWith<V> for Program {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_program(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Module(it) => <Module as VisitWith<V>>::visit_with(it, visitor),
            Self::Script(it) => <Script as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Module {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_module(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.body(visitor.ast());
        <TypedSubRange<ModuleItem> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.shebang(visitor.ast());
        <OptionalUtf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Script {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_script(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.body(visitor.ast());
        <TypedSubRange<Stmt> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.shebang(visitor.ast());
        <OptionalUtf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ModuleItem {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_module_item(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::ModuleDecl(it) => <ModuleDecl as VisitWith<V>>::visit_with(it, visitor),
            Self::Stmt(it) => <Stmt as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ModuleDecl {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_module_decl(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Import(it) => <ImportDecl as VisitWith<V>>::visit_with(it, visitor),
            Self::ExportDecl(it) => <ExportDecl as VisitWith<V>>::visit_with(it, visitor),
            Self::ExportNamed(it) => <NamedExport as VisitWith<V>>::visit_with(it, visitor),
            Self::ExportDefaultDecl(it) => {
                <ExportDefaultDecl as VisitWith<V>>::visit_with(it, visitor)
            }
            Self::ExportDefaultExpr(it) => {
                <ExportDefaultExpr as VisitWith<V>>::visit_with(it, visitor)
            }
            Self::ExportAll(it) => <ExportAll as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ImportDecl {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_import_decl(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.specifiers(visitor.ast());
        <TypedSubRange<ImportSpecifier> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.src(visitor.ast());
        <Str as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.type_only(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.with(visitor.ast());
        <Option<ObjectLit> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.phase(visitor.ast());
        <ImportPhase as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ImportSpecifier {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_import_specifier(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Named(it) => <ImportNamedSpecifier as VisitWith<V>>::visit_with(it, visitor),
            Self::Default(it) => <ImportDefaultSpecifier as VisitWith<V>>::visit_with(it, visitor),
            Self::Namespace(it) => <ImportStarAsSpecifier as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ImportNamedSpecifier {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_import_named_specifier(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.local(visitor.ast());
        <Ident as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.imported(visitor.ast());
        <Option<ModuleExportName> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.is_type_only(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ImportDefaultSpecifier {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_import_default_specifier(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.local(visitor.ast());
        <Ident as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ImportStarAsSpecifier {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_import_star_as_specifier(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.local(visitor.ast());
        <Ident as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ExportDecl {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_export_decl(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.decl(visitor.ast());
        <Decl as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for NamedExport {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_named_export(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.specifiers(visitor.ast());
        <TypedSubRange<ExportSpecifier> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.src(visitor.ast());
        <Option<Str> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.type_only(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.with(visitor.ast());
        <Option<ObjectLit> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ExportSpecifier {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_export_specifier(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Namespace(it) => {
                <ExportNamespaceSpecifier as VisitWith<V>>::visit_with(it, visitor)
            }
            Self::Default(it) => <ExportDefaultSpecifier as VisitWith<V>>::visit_with(it, visitor),
            Self::Named(it) => <ExportNamedSpecifier as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ExportNamespaceSpecifier {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_export_namespace_specifier(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <ModuleExportName as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ModuleExportName {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_module_export_name(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Ident(it) => <Ident as VisitWith<V>>::visit_with(it, visitor),
            Self::Str(it) => <Str as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ExportDefaultSpecifier {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_export_default_specifier(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.exported(visitor.ast());
        <Ident as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ExportNamedSpecifier {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_export_named_specifier(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.orig(visitor.ast());
        <ModuleExportName as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.exported(visitor.ast());
        <Option<ModuleExportName> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.is_type_only(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ExportDefaultDecl {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_export_default_decl(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.decl(visitor.ast());
        <DefaultDecl as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for DefaultDecl {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_default_decl(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Class(it) => <ClassExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Fn(it) => <FnExpr as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ExportDefaultExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_export_default_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ExportAll {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_export_all(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.src(visitor.ast());
        <Str as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.type_only(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.with(visitor.ast());
        <Option<ObjectLit> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for BlockStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_block_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.stmts(visitor.ast());
        <TypedSubRange<Stmt> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Stmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Block(it) => <BlockStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::Empty(it) => <EmptyStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::Debugger(it) => <DebuggerStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::With(it) => <WithStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::Return(it) => <ReturnStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::Labeled(it) => <LabeledStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::Break(it) => <BreakStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::Continue(it) => <ContinueStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::If(it) => <IfStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::Switch(it) => <SwitchStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::Throw(it) => <ThrowStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::Try(it) => <TryStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::While(it) => <WhileStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::DoWhile(it) => <DoWhileStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::For(it) => <ForStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::ForIn(it) => <ForInStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::ForOf(it) => <ForOfStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::Decl(it) => <Decl as VisitWith<V>>::visit_with(it, visitor),
            Self::Expr(it) => <ExprStmt as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ExprStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_expr_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for EmptyStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_empty_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<V: ?Sized + Visit> VisitWith<V> for DebuggerStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_debugger_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<V: ?Sized + Visit> VisitWith<V> for WithStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_with_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.obj(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ReturnStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_return_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.arg(visitor.ast());
        <Option<Expr> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for LabeledStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_labeled_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.label(visitor.ast());
        <Ident as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for BreakStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_break_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.label(visitor.ast());
        <Option<Ident> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ContinueStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_continue_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.label(visitor.ast());
        <Option<Ident> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IfStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_if_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.test(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.cons(visitor.ast());
        <Stmt as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.alt(visitor.ast());
        <Option<Stmt> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for SwitchStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_switch_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.discriminant(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.cases(visitor.ast());
        <TypedSubRange<SwitchCase> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ThrowStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_throw_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.arg(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TryStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_try_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.block(visitor.ast());
        <BlockStmt as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.handler(visitor.ast());
        <Option<CatchClause> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.finalizer(visitor.ast());
        <Option<BlockStmt> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for WhileStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_while_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.test(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for DoWhileStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_do_while_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.test(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ForStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_for_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.init(visitor.ast());
        <Option<VarDeclOrExpr> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.test(visitor.ast());
        <Option<Expr> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.update(visitor.ast());
        <Option<Expr> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ForInStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_for_in_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.left(visitor.ast());
        <ForHead as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ForOfStmt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_for_of_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.is_await(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.left(visitor.ast());
        <ForHead as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for SwitchCase {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_switch_case(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.test(visitor.ast());
        <Option<Expr> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.cons(visitor.ast());
        <TypedSubRange<Stmt> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for CatchClause {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_catch_clause(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.param(visitor.ast());
        <Option<Pat> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <BlockStmt as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ForHead {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_for_head(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::VarDecl(it) => <VarDecl as VisitWith<V>>::visit_with(it, visitor),
            Self::UsingDecl(it) => <UsingDecl as VisitWith<V>>::visit_with(it, visitor),
            Self::Pat(it) => <Pat as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for VarDeclOrExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_var_decl_or_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::VarDecl(it) => <VarDecl as VisitWith<V>>::visit_with(it, visitor),
            Self::Expr(it) => <Expr as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Decl {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_decl(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Class(it) => <ClassDecl as VisitWith<V>>::visit_with(it, visitor),
            Self::Fn(it) => <FnDecl as VisitWith<V>>::visit_with(it, visitor),
            Self::Var(it) => <VarDecl as VisitWith<V>>::visit_with(it, visitor),
            Self::Using(it) => <UsingDecl as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for FnDecl {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_fn_decl(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.ident(visitor.ast());
        <Ident as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.declare(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.function(visitor.ast());
        <Function as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ClassDecl {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_class_decl(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.ident(visitor.ast());
        <Ident as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.declare(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.class(visitor.ast());
        <Class as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for VarDecl {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_var_decl(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.kind(visitor.ast());
        <VarDeclKind as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.declare(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.decls(visitor.ast());
        <TypedSubRange<VarDeclarator> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for VarDeclarator {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_var_declarator(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <Pat as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.init(visitor.ast());
        <Option<Expr> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for UsingDecl {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_using_decl(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.is_await(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.decls(visitor.ast());
        <TypedSubRange<VarDeclarator> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Expr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::This(it) => <ThisExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Array(it) => <ArrayLit as VisitWith<V>>::visit_with(it, visitor),
            Self::Object(it) => <ObjectLit as VisitWith<V>>::visit_with(it, visitor),
            Self::Fn(it) => <FnExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Unary(it) => <UnaryExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Update(it) => <UpdateExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Bin(it) => <BinExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Assign(it) => <AssignExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Member(it) => <MemberExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::SuperProp(it) => <SuperPropExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Cond(it) => <CondExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Call(it) => <CallExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::New(it) => <NewExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Seq(it) => <SeqExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Ident(it) => <Ident as VisitWith<V>>::visit_with(it, visitor),
            Self::Lit(it) => <Lit as VisitWith<V>>::visit_with(it, visitor),
            Self::Tpl(it) => <Tpl as VisitWith<V>>::visit_with(it, visitor),
            Self::TaggedTpl(it) => <TaggedTpl as VisitWith<V>>::visit_with(it, visitor),
            Self::Arrow(it) => <ArrowExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Class(it) => <ClassExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Yield(it) => <YieldExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::MetaProp(it) => <MetaPropExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Await(it) => <AwaitExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Paren(it) => <ParenExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::JSXMember(it) => <JSXMemberExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::JSXNamespacedName(it) => {
                <JSXNamespacedName as VisitWith<V>>::visit_with(it, visitor)
            }
            Self::JSXEmpty(it) => <JSXEmptyExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::JSXElement(it) => <JSXElement as VisitWith<V>>::visit_with(it, visitor),
            Self::JSXFragment(it) => <JSXFragment as VisitWith<V>>::visit_with(it, visitor),
            Self::PrivateName(it) => <PrivateName as VisitWith<V>>::visit_with(it, visitor),
            Self::OptChain(it) => <OptChainExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Invalid(it) => <Invalid as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ThisExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_this_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<V: ?Sized + Visit> VisitWith<V> for ArrayLit {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_array_lit(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.elems(visitor.ast());
        <TypedSubRange<Option<ExprOrSpread>> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ObjectLit {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_object_lit(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.props(visitor.ast());
        <TypedSubRange<PropOrSpread> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for PropOrSpread {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_prop_or_spread(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::SpreadElement(it) => <SpreadElement as VisitWith<V>>::visit_with(it, visitor),
            Self::Prop(it) => <Prop as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for SpreadElement {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_spread_element(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.dot_3_token(visitor.ast());
        <Span as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.expr(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for UnaryExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_unary_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.op(visitor.ast());
        <UnaryOp as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.arg(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for UpdateExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_update_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.op(visitor.ast());
        <UpdateOp as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.prefix(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.arg(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for BinExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_bin_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.op(visitor.ast());
        <BinaryOp as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.left(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for FnExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_fn_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.ident(visitor.ast());
        <Option<Ident> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.function(visitor.ast());
        <Function as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ClassExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_class_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.ident(visitor.ast());
        <Option<Ident> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.class(visitor.ast());
        <Class as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AssignExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_assign_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.op(visitor.ast());
        <AssignOp as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.left(visitor.ast());
        <AssignTarget as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for MemberExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_member_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.obj(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.prop(visitor.ast());
        <MemberProp as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for MemberProp {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_member_prop(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Ident(it) => <IdentName as VisitWith<V>>::visit_with(it, visitor),
            Self::PrivateName(it) => <PrivateName as VisitWith<V>>::visit_with(it, visitor),
            Self::Computed(it) => <ComputedPropName as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for SuperPropExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_super_prop_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.obj(visitor.ast());
        <Super as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.prop(visitor.ast());
        <SuperProp as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for SuperProp {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_super_prop(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Ident(it) => <IdentName as VisitWith<V>>::visit_with(it, visitor),
            Self::Computed(it) => <ComputedPropName as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for CondExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_cond_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.test(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.cons(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.alt(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for CallExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_call_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.callee(visitor.ast());
        <Callee as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.args(visitor.ast());
        <TypedSubRange<ExprOrSpread> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for NewExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_new_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.callee(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.args(visitor.ast());
        <Option<TypedSubRange<ExprOrSpread>> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for SeqExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_seq_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.exprs(visitor.ast());
        <TypedSubRange<Expr> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ArrowExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_arrow_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.params(visitor.ast());
        <TypedSubRange<Pat> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <BlockStmtOrExpr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.is_async(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.is_generator(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for YieldExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_yield_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.arg(visitor.ast());
        <Option<Expr> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.delegate(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for MetaPropExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_meta_prop_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.kind(visitor.ast());
        <MetaPropKind as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AwaitExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_await_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.arg(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Tpl {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_tpl(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.exprs(visitor.ast());
        <TypedSubRange<Expr> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.quasis(visitor.ast());
        <TypedSubRange<TplElement> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TaggedTpl {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_tagged_tpl(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.tag(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.tpl(visitor.ast());
        <Tpl as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TplElement {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_tpl_element(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.tail(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.cooked(visitor.ast());
        <OptionalWtf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.raw(visitor.ast());
        <Utf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ParenExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_paren_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Callee {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_callee(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Super(it) => <Super as VisitWith<V>>::visit_with(it, visitor),
            Self::Import(it) => <Import as VisitWith<V>>::visit_with(it, visitor),
            Self::Expr(it) => <Expr as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Super {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_super(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<V: ?Sized + Visit> VisitWith<V> for Import {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_import(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.phase(visitor.ast());
        <ImportPhase as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ExprOrSpread {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_expr_or_spread(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.spread(visitor.ast());
        <Option<SpreadDot3Token> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.expr(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for SpreadDot3Token {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_spread_dot_3_token(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<V: ?Sized + Visit> VisitWith<V> for BlockStmtOrExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_block_stmt_or_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::BlockStmt(it) => <BlockStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::Expr(it) => <Expr as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AssignTarget {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_assign_target(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Simple(it) => <SimpleAssignTarget as VisitWith<V>>::visit_with(it, visitor),
            Self::Pat(it) => <AssignTargetPat as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AssignTargetPat {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_assign_target_pat(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Array(it) => <ArrayPat as VisitWith<V>>::visit_with(it, visitor),
            Self::Object(it) => <ObjectPat as VisitWith<V>>::visit_with(it, visitor),
            Self::Invalid(it) => <Invalid as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for SimpleAssignTarget {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_simple_assign_target(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Ident(it) => <BindingIdent as VisitWith<V>>::visit_with(it, visitor),
            Self::Member(it) => <MemberExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::SuperProp(it) => <SuperPropExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Paren(it) => <ParenExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::OptChain(it) => <OptChainExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Invalid(it) => <Invalid as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for OptChainExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_chain_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.optional(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.base(visitor.ast());
        <OptChainBase as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for OptChainBase {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_chain_base(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Member(it) => <MemberExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Call(it) => <OptCall as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for OptCall {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_call(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.callee(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.args(visitor.ast());
        <TypedSubRange<ExprOrSpread> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Invalid {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_invalid(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<V: ?Sized + Visit> VisitWith<V> for Function {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_function(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.params(visitor.ast());
        <TypedSubRange<Param> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Option<BlockStmt> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.is_generator(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.is_async(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Param {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_param(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.pat(visitor.ast());
        <Pat as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ParamOrTsParamProp {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_param_or_ts_param_prop(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Param(it) => <Param as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Class {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_class(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <TypedSubRange<ClassMember> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.super_class(visitor.ast());
        <Option<Expr> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.is_abstract(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ClassMember {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_class_member(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Constructor(it) => <Constructor as VisitWith<V>>::visit_with(it, visitor),
            Self::Method(it) => <ClassMethod as VisitWith<V>>::visit_with(it, visitor),
            Self::PrivateMethod(it) => <PrivateMethod as VisitWith<V>>::visit_with(it, visitor),
            Self::ClassProp(it) => <ClassProp as VisitWith<V>>::visit_with(it, visitor),
            Self::PrivateProp(it) => <PrivateProp as VisitWith<V>>::visit_with(it, visitor),
            Self::Empty(it) => <EmptyStmt as VisitWith<V>>::visit_with(it, visitor),
            Self::StaticBlock(it) => <StaticBlock as VisitWith<V>>::visit_with(it, visitor),
            Self::AutoAccessor(it) => <AutoAccessor as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ClassProp {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_class_prop(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Option<Expr> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.is_static(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for PrivateProp {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_private_prop(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PrivateName as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Option<Expr> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.is_static(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ClassMethod {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_class_method(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.function(visitor.ast());
        <Function as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.kind(visitor.ast());
        <MethodKind as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.is_static(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for PrivateMethod {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_private_method(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PrivateName as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.function(visitor.ast());
        <Function as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.kind(visitor.ast());
        <MethodKind as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.is_static(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Constructor {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_constructor(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.params(visitor.ast());
        <TypedSubRange<ParamOrTsParamProp> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Option<BlockStmt> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Decorator {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_decorator(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for StaticBlock {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_static_block(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.body(visitor.ast());
        <BlockStmt as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Key {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_key(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Private(it) => <PrivateName as VisitWith<V>>::visit_with(it, visitor),
            Self::Public(it) => <PropName as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AutoAccessor {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_auto_accessor(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <Key as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Option<Expr> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.is_static(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Prop {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_prop(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Shorthand(it) => <Ident as VisitWith<V>>::visit_with(it, visitor),
            Self::KeyValue(it) => <KeyValueProp as VisitWith<V>>::visit_with(it, visitor),
            Self::Assign(it) => <AssignProp as VisitWith<V>>::visit_with(it, visitor),
            Self::Getter(it) => <GetterProp as VisitWith<V>>::visit_with(it, visitor),
            Self::Setter(it) => <SetterProp as VisitWith<V>>::visit_with(it, visitor),
            Self::Method(it) => <MethodProp as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for KeyValueProp {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_key_value_prop(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AssignProp {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_assign_prop(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <Ident as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for GetterProp {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_getter_prop(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Option<BlockStmt> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for SetterProp {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_setter_prop(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.this_param(visitor.ast());
        <Option<Pat> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.param(visitor.ast());
        <Pat as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Option<BlockStmt> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for MethodProp {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_method_prop(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.function(visitor.ast());
        <Function as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for PropName {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_prop_name(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Ident(it) => <IdentName as VisitWith<V>>::visit_with(it, visitor),
            Self::Str(it) => <Str as VisitWith<V>>::visit_with(it, visitor),
            Self::Num(it) => <Number as VisitWith<V>>::visit_with(it, visitor),
            Self::Computed(it) => <ComputedPropName as VisitWith<V>>::visit_with(it, visitor),
            Self::BigInt(it) => <BigInt as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ComputedPropName {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_computed_prop_name(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Pat {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_pat(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Ident(it) => <BindingIdent as VisitWith<V>>::visit_with(it, visitor),
            Self::Array(it) => <ArrayPat as VisitWith<V>>::visit_with(it, visitor),
            Self::Rest(it) => <RestPat as VisitWith<V>>::visit_with(it, visitor),
            Self::Object(it) => <ObjectPat as VisitWith<V>>::visit_with(it, visitor),
            Self::Assign(it) => <AssignPat as VisitWith<V>>::visit_with(it, visitor),
            Self::Invalid(it) => <Invalid as VisitWith<V>>::visit_with(it, visitor),
            Self::Expr(it) => <Expr as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ArrayPat {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_array_pat(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.elems(visitor.ast());
        <TypedSubRange<Option<Pat>> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.optional(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ObjectPat {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_object_pat(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.props(visitor.ast());
        <TypedSubRange<ObjectPatProp> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.optional(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AssignPat {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_assign_pat(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.left(visitor.ast());
        <Pat as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for RestPat {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_rest_pat(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.dot_3_token(visitor.ast());
        <Span as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.arg(visitor.ast());
        <Pat as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ObjectPatProp {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_object_pat_prop(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::KeyValue(it) => <KeyValuePatProp as VisitWith<V>>::visit_with(it, visitor),
            Self::Assign(it) => <AssignPatProp as VisitWith<V>>::visit_with(it, visitor),
            Self::Rest(it) => <RestPat as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for KeyValuePatProp {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_key_value_pat_prop(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Pat as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AssignPatProp {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_assign_pat_prop(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <BindingIdent as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Option<Expr> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Ident {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_ident(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.sym(visitor.ast());
        <Utf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.optional(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IdentName {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_ident_name(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.sym(visitor.ast());
        <Utf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for PrivateName {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_private_name(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <Utf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for BindingIdent {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_binding_ident(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.id(visitor.ast());
        <Ident as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Lit {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_lit(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Str(it) => <Str as VisitWith<V>>::visit_with(it, visitor),
            Self::Bool(it) => <Bool as VisitWith<V>>::visit_with(it, visitor),
            Self::Null(it) => <Null as VisitWith<V>>::visit_with(it, visitor),
            Self::Num(it) => <Number as VisitWith<V>>::visit_with(it, visitor),
            Self::BigInt(it) => <BigInt as VisitWith<V>>::visit_with(it, visitor),
            Self::Regex(it) => <Regex as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Str {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_str(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.value(visitor.ast());
        <Wtf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.raw(visitor.ast());
        <OptionalUtf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Bool {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_bool(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.value(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Null {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_null(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<V: ?Sized + Visit> VisitWith<V> for Number {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_number(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.value(visitor.ast());
        <f64 as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.raw(visitor.ast());
        <OptionalUtf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for BigInt {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_big_int(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.value(visitor.ast());
        <BigIntId as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.raw(visitor.ast());
        <OptionalUtf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Regex {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_regex(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.exp(visitor.ast());
        <Utf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.flags(visitor.ast());
        <Utf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXObject {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_object(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::JSXMemberExpr(it) => <JSXMemberExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Ident(it) => <Ident as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXMemberExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_member_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.obj(visitor.ast());
        <JSXObject as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.prop(visitor.ast());
        <IdentName as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXNamespacedName {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_namespaced_name(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.ns(visitor.ast());
        <IdentName as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.name(visitor.ast());
        <IdentName as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXEmptyExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_empty_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXExprContainer {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_expr_container(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <JSXExpr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXExpr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::JSXEmptyExpr(it) => <JSXEmptyExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::Expr(it) => <Expr as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXSpreadChild {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_spread_child(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <Expr as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXElementName {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_element_name(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Ident(it) => <Ident as VisitWith<V>>::visit_with(it, visitor),
            Self::JSXMemberExpr(it) => <JSXMemberExpr as VisitWith<V>>::visit_with(it, visitor),
            Self::JSXNamespacedName(it) => {
                <JSXNamespacedName as VisitWith<V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXOpeningElement {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_opening_element(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <JSXElementName as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.attrs(visitor.ast());
        <TypedSubRange<JSXAttrOrSpread> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.self_closing(visitor.ast());
        <bool as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXAttrOrSpread {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_attr_or_spread(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::JSXAttr(it) => <JSXAttr as VisitWith<V>>::visit_with(it, visitor),
            Self::SpreadElement(it) => <SpreadElement as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXClosingElement {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_closing_element(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <JSXElementName as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXAttr {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_attr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <JSXAttrName as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Option<JSXAttrValue> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXAttrName {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_attr_name(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Ident(it) => <IdentName as VisitWith<V>>::visit_with(it, visitor),
            Self::JSXNamespacedName(it) => {
                <JSXNamespacedName as VisitWith<V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXAttrValue {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_attr_value(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::Str(it) => <Str as VisitWith<V>>::visit_with(it, visitor),
            Self::JSXExprContainer(it) => {
                <JSXExprContainer as VisitWith<V>>::visit_with(it, visitor)
            }
            Self::JSXElement(it) => <JSXElement as VisitWith<V>>::visit_with(it, visitor),
            Self::JSXFragment(it) => <JSXFragment as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXText {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_text(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.value(visitor.ast());
        <Utf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.raw(visitor.ast());
        <Utf8Ref as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXElement {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_element(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.opening(visitor.ast());
        <JSXOpeningElement as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.children(visitor.ast());
        <TypedSubRange<JSXElementChild> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.closing(visitor.ast());
        <Option<JSXClosingElement> as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXElementChild {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_element_child(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Self::JSXText(it) => <JSXText as VisitWith<V>>::visit_with(it, visitor),
            Self::JSXExprContainer(it) => {
                <JSXExprContainer as VisitWith<V>>::visit_with(it, visitor)
            }
            Self::JSXSpreadChild(it) => <JSXSpreadChild as VisitWith<V>>::visit_with(it, visitor),
            Self::JSXElement(it) => <JSXElement as VisitWith<V>>::visit_with(it, visitor),
            Self::JSXFragment(it) => <JSXFragment as VisitWith<V>>::visit_with(it, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXFragment {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_fragment(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        let field_value = self.opening(visitor.ast());
        <JSXOpeningFragment as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.children(visitor.ast());
        <TypedSubRange<JSXElementChild> as VisitWith<V>>::visit_with(field_value, visitor);
        let field_value = self.closing(visitor.ast());
        <JSXClosingFragment as VisitWith<V>>::visit_with(field_value, visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXOpeningFragment {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_opening_fragment(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<V: ?Sized + Visit> VisitWith<V> for JSXClosingFragment {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_closing_fragment(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<ModuleItem> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_module_items(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<Stmt> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_stmts(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<ImportSpecifier> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_import_specifiers(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<ObjectLit> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_object_lit(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<ModuleExportName> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_module_export_name(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<ExportSpecifier> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_export_specifiers(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<Str> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_str(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<Expr> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<Ident> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_ident(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<Stmt> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<SwitchCase> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_switch_cases(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<CatchClause> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_catch_clause(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<BlockStmt> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_block_stmt(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<VarDeclOrExpr> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_var_decl_or_expr(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<Pat> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_pat(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<VarDeclarator> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_var_declarators(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<ExprOrSpread> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_expr_or_spread(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<Option<ExprOrSpread>> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_vec_expr_or_spreads(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_opt_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<PropOrSpread> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_prop_or_spreads(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<ExprOrSpread> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_expr_or_spreads(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<TypedSubRange<ExprOrSpread>> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_expr_or_spreads(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<Expr> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_exprs(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<Pat> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_pats(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<TplElement> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_tpl_elements(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<SpreadDot3Token> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_spread_dot_3_token(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<Param> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_params(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<Decorator> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_decorators(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<ClassMember> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_class_members(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<ParamOrTsParamProp> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_param_or_ts_param_props(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<Option<Pat>> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_vec_pats(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_opt_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<ObjectPatProp> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_object_pat_props(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<JSXAttrOrSpread> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_attr_or_spreads(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<JSXAttrValue> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_jsx_attr_value(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TypedSubRange<JSXElementChild> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_jsx_element_childs(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Option<JSXClosingElement> {
    fn visit_with(self, visitor: &mut V) {
        <V as Visit>::visit_opt_jsx_closing_element(visitor, self)
    }
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
pub trait VisitMut {
    fn ast(&mut self) -> &mut Ast;
    #[inline]
    fn enter_node(&mut self, node_kind: NodeKind) {}
    #[inline]
    fn leave_node(&mut self, node_kind: NodeKind) {}
    #[inline]
    fn visit_mut_program(&mut self, node: Program) -> Program {
        <Program as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_module(&mut self, node: Module) -> Module {
        self.enter_node(NodeKind::Module);
        let node = <Module as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Module);
        node
    }
    #[inline]
    fn visit_mut_script(&mut self, node: Script) -> Script {
        self.enter_node(NodeKind::Script);
        let node = <Script as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Script);
        node
    }
    #[inline]
    fn visit_mut_module_item(&mut self, node: ModuleItem) -> ModuleItem {
        <ModuleItem as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_module_decl(&mut self, node: ModuleDecl) -> ModuleDecl {
        <ModuleDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_decl(&mut self, node: ImportDecl) -> ImportDecl {
        self.enter_node(NodeKind::ImportDecl);
        let node = <ImportDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ImportDecl);
        node
    }
    #[inline]
    fn visit_mut_import_specifier(&mut self, node: ImportSpecifier) -> ImportSpecifier {
        <ImportSpecifier as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_named_specifier(
        &mut self,
        node: ImportNamedSpecifier,
    ) -> ImportNamedSpecifier {
        self.enter_node(NodeKind::ImportNamedSpecifier);
        let node =
            <ImportNamedSpecifier as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ImportNamedSpecifier);
        node
    }
    #[inline]
    fn visit_mut_import_default_specifier(
        &mut self,
        node: ImportDefaultSpecifier,
    ) -> ImportDefaultSpecifier {
        self.enter_node(NodeKind::ImportDefaultSpecifier);
        let node =
            <ImportDefaultSpecifier as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ImportDefaultSpecifier);
        node
    }
    #[inline]
    fn visit_mut_import_star_as_specifier(
        &mut self,
        node: ImportStarAsSpecifier,
    ) -> ImportStarAsSpecifier {
        self.enter_node(NodeKind::ImportStarAsSpecifier);
        let node =
            <ImportStarAsSpecifier as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ImportStarAsSpecifier);
        node
    }
    #[inline]
    fn visit_mut_export_decl(&mut self, node: ExportDecl) -> ExportDecl {
        self.enter_node(NodeKind::ExportDecl);
        let node = <ExportDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExportDecl);
        node
    }
    #[inline]
    fn visit_mut_named_export(&mut self, node: NamedExport) -> NamedExport {
        self.enter_node(NodeKind::NamedExport);
        let node = <NamedExport as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::NamedExport);
        node
    }
    #[inline]
    fn visit_mut_export_specifier(&mut self, node: ExportSpecifier) -> ExportSpecifier {
        <ExportSpecifier as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_namespace_specifier(
        &mut self,
        node: ExportNamespaceSpecifier,
    ) -> ExportNamespaceSpecifier {
        self.enter_node(NodeKind::ExportNamespaceSpecifier);
        let node =
            <ExportNamespaceSpecifier as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExportNamespaceSpecifier);
        node
    }
    #[inline]
    fn visit_mut_module_export_name(&mut self, node: ModuleExportName) -> ModuleExportName {
        <ModuleExportName as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_default_specifier(
        &mut self,
        node: ExportDefaultSpecifier,
    ) -> ExportDefaultSpecifier {
        self.enter_node(NodeKind::ExportDefaultSpecifier);
        let node =
            <ExportDefaultSpecifier as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExportDefaultSpecifier);
        node
    }
    #[inline]
    fn visit_mut_export_named_specifier(
        &mut self,
        node: ExportNamedSpecifier,
    ) -> ExportNamedSpecifier {
        self.enter_node(NodeKind::ExportNamedSpecifier);
        let node =
            <ExportNamedSpecifier as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExportNamedSpecifier);
        node
    }
    #[inline]
    fn visit_mut_export_default_decl(&mut self, node: ExportDefaultDecl) -> ExportDefaultDecl {
        self.enter_node(NodeKind::ExportDefaultDecl);
        let node = <ExportDefaultDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExportDefaultDecl);
        node
    }
    #[inline]
    fn visit_mut_default_decl(&mut self, node: DefaultDecl) -> DefaultDecl {
        <DefaultDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_default_expr(&mut self, node: ExportDefaultExpr) -> ExportDefaultExpr {
        self.enter_node(NodeKind::ExportDefaultExpr);
        let node = <ExportDefaultExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExportDefaultExpr);
        node
    }
    #[inline]
    fn visit_mut_export_all(&mut self, node: ExportAll) -> ExportAll {
        self.enter_node(NodeKind::ExportAll);
        let node = <ExportAll as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExportAll);
        node
    }
    #[inline]
    fn visit_mut_block_stmt(&mut self, node: BlockStmt) -> BlockStmt {
        self.enter_node(NodeKind::BlockStmt);
        let node = <BlockStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::BlockStmt);
        node
    }
    #[inline]
    fn visit_mut_stmt(&mut self, node: Stmt) -> Stmt {
        <Stmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_expr_stmt(&mut self, node: ExprStmt) -> ExprStmt {
        self.enter_node(NodeKind::ExprStmt);
        let node = <ExprStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExprStmt);
        node
    }
    #[inline]
    fn visit_mut_empty_stmt(&mut self, node: EmptyStmt) -> EmptyStmt {
        self.enter_node(NodeKind::EmptyStmt);
        let node = <EmptyStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::EmptyStmt);
        node
    }
    #[inline]
    fn visit_mut_debugger_stmt(&mut self, node: DebuggerStmt) -> DebuggerStmt {
        self.enter_node(NodeKind::DebuggerStmt);
        let node = <DebuggerStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::DebuggerStmt);
        node
    }
    #[inline]
    fn visit_mut_with_stmt(&mut self, node: WithStmt) -> WithStmt {
        self.enter_node(NodeKind::WithStmt);
        let node = <WithStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::WithStmt);
        node
    }
    #[inline]
    fn visit_mut_return_stmt(&mut self, node: ReturnStmt) -> ReturnStmt {
        self.enter_node(NodeKind::ReturnStmt);
        let node = <ReturnStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ReturnStmt);
        node
    }
    #[inline]
    fn visit_mut_labeled_stmt(&mut self, node: LabeledStmt) -> LabeledStmt {
        self.enter_node(NodeKind::LabeledStmt);
        let node = <LabeledStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::LabeledStmt);
        node
    }
    #[inline]
    fn visit_mut_break_stmt(&mut self, node: BreakStmt) -> BreakStmt {
        self.enter_node(NodeKind::BreakStmt);
        let node = <BreakStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::BreakStmt);
        node
    }
    #[inline]
    fn visit_mut_continue_stmt(&mut self, node: ContinueStmt) -> ContinueStmt {
        self.enter_node(NodeKind::ContinueStmt);
        let node = <ContinueStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ContinueStmt);
        node
    }
    #[inline]
    fn visit_mut_if_stmt(&mut self, node: IfStmt) -> IfStmt {
        self.enter_node(NodeKind::IfStmt);
        let node = <IfStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::IfStmt);
        node
    }
    #[inline]
    fn visit_mut_switch_stmt(&mut self, node: SwitchStmt) -> SwitchStmt {
        self.enter_node(NodeKind::SwitchStmt);
        let node = <SwitchStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::SwitchStmt);
        node
    }
    #[inline]
    fn visit_mut_throw_stmt(&mut self, node: ThrowStmt) -> ThrowStmt {
        self.enter_node(NodeKind::ThrowStmt);
        let node = <ThrowStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ThrowStmt);
        node
    }
    #[inline]
    fn visit_mut_try_stmt(&mut self, node: TryStmt) -> TryStmt {
        self.enter_node(NodeKind::TryStmt);
        let node = <TryStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::TryStmt);
        node
    }
    #[inline]
    fn visit_mut_while_stmt(&mut self, node: WhileStmt) -> WhileStmt {
        self.enter_node(NodeKind::WhileStmt);
        let node = <WhileStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::WhileStmt);
        node
    }
    #[inline]
    fn visit_mut_do_while_stmt(&mut self, node: DoWhileStmt) -> DoWhileStmt {
        self.enter_node(NodeKind::DoWhileStmt);
        let node = <DoWhileStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::DoWhileStmt);
        node
    }
    #[inline]
    fn visit_mut_for_stmt(&mut self, node: ForStmt) -> ForStmt {
        self.enter_node(NodeKind::ForStmt);
        let node = <ForStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ForStmt);
        node
    }
    #[inline]
    fn visit_mut_for_in_stmt(&mut self, node: ForInStmt) -> ForInStmt {
        self.enter_node(NodeKind::ForInStmt);
        let node = <ForInStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ForInStmt);
        node
    }
    #[inline]
    fn visit_mut_for_of_stmt(&mut self, node: ForOfStmt) -> ForOfStmt {
        self.enter_node(NodeKind::ForOfStmt);
        let node = <ForOfStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ForOfStmt);
        node
    }
    #[inline]
    fn visit_mut_switch_case(&mut self, node: SwitchCase) -> SwitchCase {
        self.enter_node(NodeKind::SwitchCase);
        let node = <SwitchCase as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::SwitchCase);
        node
    }
    #[inline]
    fn visit_mut_catch_clause(&mut self, node: CatchClause) -> CatchClause {
        self.enter_node(NodeKind::CatchClause);
        let node = <CatchClause as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::CatchClause);
        node
    }
    #[inline]
    fn visit_mut_for_head(&mut self, node: ForHead) -> ForHead {
        <ForHead as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_var_decl_or_expr(&mut self, node: VarDeclOrExpr) -> VarDeclOrExpr {
        <VarDeclOrExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_decl(&mut self, node: Decl) -> Decl {
        <Decl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_fn_decl(&mut self, node: FnDecl) -> FnDecl {
        self.enter_node(NodeKind::FnDecl);
        let node = <FnDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::FnDecl);
        node
    }
    #[inline]
    fn visit_mut_class_decl(&mut self, node: ClassDecl) -> ClassDecl {
        self.enter_node(NodeKind::ClassDecl);
        let node = <ClassDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ClassDecl);
        node
    }
    #[inline]
    fn visit_mut_var_decl(&mut self, node: VarDecl) -> VarDecl {
        self.enter_node(NodeKind::VarDecl);
        let node = <VarDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::VarDecl);
        node
    }
    #[inline]
    fn visit_mut_var_declarator(&mut self, node: VarDeclarator) -> VarDeclarator {
        self.enter_node(NodeKind::VarDeclarator);
        let node = <VarDeclarator as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::VarDeclarator);
        node
    }
    #[inline]
    fn visit_mut_using_decl(&mut self, node: UsingDecl) -> UsingDecl {
        self.enter_node(NodeKind::UsingDecl);
        let node = <UsingDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::UsingDecl);
        node
    }
    #[inline]
    fn visit_mut_expr(&mut self, node: Expr) -> Expr {
        <Expr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_this_expr(&mut self, node: ThisExpr) -> ThisExpr {
        self.enter_node(NodeKind::ThisExpr);
        let node = <ThisExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ThisExpr);
        node
    }
    #[inline]
    fn visit_mut_array_lit(&mut self, node: ArrayLit) -> ArrayLit {
        self.enter_node(NodeKind::ArrayLit);
        let node = <ArrayLit as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ArrayLit);
        node
    }
    #[inline]
    fn visit_mut_object_lit(&mut self, node: ObjectLit) -> ObjectLit {
        self.enter_node(NodeKind::ObjectLit);
        let node = <ObjectLit as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ObjectLit);
        node
    }
    #[inline]
    fn visit_mut_prop_or_spread(&mut self, node: PropOrSpread) -> PropOrSpread {
        <PropOrSpread as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_spread_element(&mut self, node: SpreadElement) -> SpreadElement {
        self.enter_node(NodeKind::SpreadElement);
        let node = <SpreadElement as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::SpreadElement);
        node
    }
    #[inline]
    fn visit_mut_unary_expr(&mut self, node: UnaryExpr) -> UnaryExpr {
        self.enter_node(NodeKind::UnaryExpr);
        let node = <UnaryExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::UnaryExpr);
        node
    }
    #[inline]
    fn visit_mut_update_expr(&mut self, node: UpdateExpr) -> UpdateExpr {
        self.enter_node(NodeKind::UpdateExpr);
        let node = <UpdateExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::UpdateExpr);
        node
    }
    #[inline]
    fn visit_mut_bin_expr(&mut self, node: BinExpr) -> BinExpr {
        self.enter_node(NodeKind::BinExpr);
        let node = <BinExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::BinExpr);
        node
    }
    #[inline]
    fn visit_mut_fn_expr(&mut self, node: FnExpr) -> FnExpr {
        self.enter_node(NodeKind::FnExpr);
        let node = <FnExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::FnExpr);
        node
    }
    #[inline]
    fn visit_mut_class_expr(&mut self, node: ClassExpr) -> ClassExpr {
        self.enter_node(NodeKind::ClassExpr);
        let node = <ClassExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ClassExpr);
        node
    }
    #[inline]
    fn visit_mut_assign_expr(&mut self, node: AssignExpr) -> AssignExpr {
        self.enter_node(NodeKind::AssignExpr);
        let node = <AssignExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::AssignExpr);
        node
    }
    #[inline]
    fn visit_mut_member_expr(&mut self, node: MemberExpr) -> MemberExpr {
        self.enter_node(NodeKind::MemberExpr);
        let node = <MemberExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::MemberExpr);
        node
    }
    #[inline]
    fn visit_mut_member_prop(&mut self, node: MemberProp) -> MemberProp {
        <MemberProp as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_super_prop_expr(&mut self, node: SuperPropExpr) -> SuperPropExpr {
        self.enter_node(NodeKind::SuperPropExpr);
        let node = <SuperPropExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::SuperPropExpr);
        node
    }
    #[inline]
    fn visit_mut_super_prop(&mut self, node: SuperProp) -> SuperProp {
        <SuperProp as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_cond_expr(&mut self, node: CondExpr) -> CondExpr {
        self.enter_node(NodeKind::CondExpr);
        let node = <CondExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::CondExpr);
        node
    }
    #[inline]
    fn visit_mut_call_expr(&mut self, node: CallExpr) -> CallExpr {
        self.enter_node(NodeKind::CallExpr);
        let node = <CallExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::CallExpr);
        node
    }
    #[inline]
    fn visit_mut_new_expr(&mut self, node: NewExpr) -> NewExpr {
        self.enter_node(NodeKind::NewExpr);
        let node = <NewExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::NewExpr);
        node
    }
    #[inline]
    fn visit_mut_seq_expr(&mut self, node: SeqExpr) -> SeqExpr {
        self.enter_node(NodeKind::SeqExpr);
        let node = <SeqExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::SeqExpr);
        node
    }
    #[inline]
    fn visit_mut_arrow_expr(&mut self, node: ArrowExpr) -> ArrowExpr {
        self.enter_node(NodeKind::ArrowExpr);
        let node = <ArrowExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ArrowExpr);
        node
    }
    #[inline]
    fn visit_mut_yield_expr(&mut self, node: YieldExpr) -> YieldExpr {
        self.enter_node(NodeKind::YieldExpr);
        let node = <YieldExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::YieldExpr);
        node
    }
    #[inline]
    fn visit_mut_meta_prop_expr(&mut self, node: MetaPropExpr) -> MetaPropExpr {
        self.enter_node(NodeKind::MetaPropExpr);
        let node = <MetaPropExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::MetaPropExpr);
        node
    }
    #[inline]
    fn visit_mut_await_expr(&mut self, node: AwaitExpr) -> AwaitExpr {
        self.enter_node(NodeKind::AwaitExpr);
        let node = <AwaitExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::AwaitExpr);
        node
    }
    #[inline]
    fn visit_mut_tpl(&mut self, node: Tpl) -> Tpl {
        self.enter_node(NodeKind::Tpl);
        let node = <Tpl as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Tpl);
        node
    }
    #[inline]
    fn visit_mut_tagged_tpl(&mut self, node: TaggedTpl) -> TaggedTpl {
        self.enter_node(NodeKind::TaggedTpl);
        let node = <TaggedTpl as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::TaggedTpl);
        node
    }
    #[inline]
    fn visit_mut_tpl_element(&mut self, node: TplElement) -> TplElement {
        self.enter_node(NodeKind::TplElement);
        let node = <TplElement as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::TplElement);
        node
    }
    #[inline]
    fn visit_mut_paren_expr(&mut self, node: ParenExpr) -> ParenExpr {
        self.enter_node(NodeKind::ParenExpr);
        let node = <ParenExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ParenExpr);
        node
    }
    #[inline]
    fn visit_mut_callee(&mut self, node: Callee) -> Callee {
        <Callee as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_super(&mut self, node: Super) -> Super {
        self.enter_node(NodeKind::Super);
        let node = <Super as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Super);
        node
    }
    #[inline]
    fn visit_mut_import(&mut self, node: Import) -> Import {
        self.enter_node(NodeKind::Import);
        let node = <Import as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Import);
        node
    }
    #[inline]
    fn visit_mut_expr_or_spread(&mut self, node: ExprOrSpread) -> ExprOrSpread {
        self.enter_node(NodeKind::ExprOrSpread);
        let node = <ExprOrSpread as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExprOrSpread);
        node
    }
    #[inline]
    fn visit_mut_spread_dot_3_token(&mut self, node: SpreadDot3Token) -> SpreadDot3Token {
        self.enter_node(NodeKind::SpreadDot3Token);
        let node = <SpreadDot3Token as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::SpreadDot3Token);
        node
    }
    #[inline]
    fn visit_mut_block_stmt_or_expr(&mut self, node: BlockStmtOrExpr) -> BlockStmtOrExpr {
        <BlockStmtOrExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_assign_target(&mut self, node: AssignTarget) -> AssignTarget {
        <AssignTarget as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_assign_target_pat(&mut self, node: AssignTargetPat) -> AssignTargetPat {
        <AssignTargetPat as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_simple_assign_target(&mut self, node: SimpleAssignTarget) -> SimpleAssignTarget {
        <SimpleAssignTarget as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_chain_expr(&mut self, node: OptChainExpr) -> OptChainExpr {
        self.enter_node(NodeKind::OptChainExpr);
        let node = <OptChainExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::OptChainExpr);
        node
    }
    #[inline]
    fn visit_mut_opt_chain_base(&mut self, node: OptChainBase) -> OptChainBase {
        <OptChainBase as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_call(&mut self, node: OptCall) -> OptCall {
        self.enter_node(NodeKind::OptCall);
        let node = <OptCall as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::OptCall);
        node
    }
    #[inline]
    fn visit_mut_invalid(&mut self, node: Invalid) -> Invalid {
        self.enter_node(NodeKind::Invalid);
        let node = <Invalid as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Invalid);
        node
    }
    #[inline]
    fn visit_mut_function(&mut self, node: Function) -> Function {
        self.enter_node(NodeKind::Function);
        let node = <Function as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Function);
        node
    }
    #[inline]
    fn visit_mut_param(&mut self, node: Param) -> Param {
        self.enter_node(NodeKind::Param);
        let node = <Param as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Param);
        node
    }
    #[inline]
    fn visit_mut_param_or_ts_param_prop(&mut self, node: ParamOrTsParamProp) -> ParamOrTsParamProp {
        <ParamOrTsParamProp as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_class(&mut self, node: Class) -> Class {
        self.enter_node(NodeKind::Class);
        let node = <Class as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Class);
        node
    }
    #[inline]
    fn visit_mut_class_member(&mut self, node: ClassMember) -> ClassMember {
        <ClassMember as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_class_prop(&mut self, node: ClassProp) -> ClassProp {
        self.enter_node(NodeKind::ClassProp);
        let node = <ClassProp as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ClassProp);
        node
    }
    #[inline]
    fn visit_mut_private_prop(&mut self, node: PrivateProp) -> PrivateProp {
        self.enter_node(NodeKind::PrivateProp);
        let node = <PrivateProp as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::PrivateProp);
        node
    }
    #[inline]
    fn visit_mut_class_method(&mut self, node: ClassMethod) -> ClassMethod {
        self.enter_node(NodeKind::ClassMethod);
        let node = <ClassMethod as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ClassMethod);
        node
    }
    #[inline]
    fn visit_mut_private_method(&mut self, node: PrivateMethod) -> PrivateMethod {
        self.enter_node(NodeKind::PrivateMethod);
        let node = <PrivateMethod as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::PrivateMethod);
        node
    }
    #[inline]
    fn visit_mut_constructor(&mut self, node: Constructor) -> Constructor {
        self.enter_node(NodeKind::Constructor);
        let node = <Constructor as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Constructor);
        node
    }
    #[inline]
    fn visit_mut_decorator(&mut self, node: Decorator) -> Decorator {
        self.enter_node(NodeKind::Decorator);
        let node = <Decorator as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Decorator);
        node
    }
    #[inline]
    fn visit_mut_static_block(&mut self, node: StaticBlock) -> StaticBlock {
        self.enter_node(NodeKind::StaticBlock);
        let node = <StaticBlock as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::StaticBlock);
        node
    }
    #[inline]
    fn visit_mut_key(&mut self, node: Key) -> Key {
        <Key as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_auto_accessor(&mut self, node: AutoAccessor) -> AutoAccessor {
        self.enter_node(NodeKind::AutoAccessor);
        let node = <AutoAccessor as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::AutoAccessor);
        node
    }
    #[inline]
    fn visit_mut_prop(&mut self, node: Prop) -> Prop {
        <Prop as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_key_value_prop(&mut self, node: KeyValueProp) -> KeyValueProp {
        self.enter_node(NodeKind::KeyValueProp);
        let node = <KeyValueProp as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::KeyValueProp);
        node
    }
    #[inline]
    fn visit_mut_assign_prop(&mut self, node: AssignProp) -> AssignProp {
        self.enter_node(NodeKind::AssignProp);
        let node = <AssignProp as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::AssignProp);
        node
    }
    #[inline]
    fn visit_mut_getter_prop(&mut self, node: GetterProp) -> GetterProp {
        self.enter_node(NodeKind::GetterProp);
        let node = <GetterProp as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::GetterProp);
        node
    }
    #[inline]
    fn visit_mut_setter_prop(&mut self, node: SetterProp) -> SetterProp {
        self.enter_node(NodeKind::SetterProp);
        let node = <SetterProp as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::SetterProp);
        node
    }
    #[inline]
    fn visit_mut_method_prop(&mut self, node: MethodProp) -> MethodProp {
        self.enter_node(NodeKind::MethodProp);
        let node = <MethodProp as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::MethodProp);
        node
    }
    #[inline]
    fn visit_mut_prop_name(&mut self, node: PropName) -> PropName {
        <PropName as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_computed_prop_name(&mut self, node: ComputedPropName) -> ComputedPropName {
        self.enter_node(NodeKind::ComputedPropName);
        let node = <ComputedPropName as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ComputedPropName);
        node
    }
    #[inline]
    fn visit_mut_pat(&mut self, node: Pat) -> Pat {
        <Pat as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_array_pat(&mut self, node: ArrayPat) -> ArrayPat {
        self.enter_node(NodeKind::ArrayPat);
        let node = <ArrayPat as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ArrayPat);
        node
    }
    #[inline]
    fn visit_mut_object_pat(&mut self, node: ObjectPat) -> ObjectPat {
        self.enter_node(NodeKind::ObjectPat);
        let node = <ObjectPat as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ObjectPat);
        node
    }
    #[inline]
    fn visit_mut_assign_pat(&mut self, node: AssignPat) -> AssignPat {
        self.enter_node(NodeKind::AssignPat);
        let node = <AssignPat as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::AssignPat);
        node
    }
    #[inline]
    fn visit_mut_rest_pat(&mut self, node: RestPat) -> RestPat {
        self.enter_node(NodeKind::RestPat);
        let node = <RestPat as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::RestPat);
        node
    }
    #[inline]
    fn visit_mut_object_pat_prop(&mut self, node: ObjectPatProp) -> ObjectPatProp {
        <ObjectPatProp as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_key_value_pat_prop(&mut self, node: KeyValuePatProp) -> KeyValuePatProp {
        self.enter_node(NodeKind::KeyValuePatProp);
        let node = <KeyValuePatProp as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::KeyValuePatProp);
        node
    }
    #[inline]
    fn visit_mut_assign_pat_prop(&mut self, node: AssignPatProp) -> AssignPatProp {
        self.enter_node(NodeKind::AssignPatProp);
        let node = <AssignPatProp as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::AssignPatProp);
        node
    }
    #[inline]
    fn visit_mut_ident(&mut self, node: Ident) -> Ident {
        self.enter_node(NodeKind::Ident);
        let node = <Ident as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Ident);
        node
    }
    #[inline]
    fn visit_mut_ident_name(&mut self, node: IdentName) -> IdentName {
        self.enter_node(NodeKind::IdentName);
        let node = <IdentName as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::IdentName);
        node
    }
    #[inline]
    fn visit_mut_private_name(&mut self, node: PrivateName) -> PrivateName {
        self.enter_node(NodeKind::PrivateName);
        let node = <PrivateName as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::PrivateName);
        node
    }
    #[inline]
    fn visit_mut_binding_ident(&mut self, node: BindingIdent) -> BindingIdent {
        self.enter_node(NodeKind::BindingIdent);
        let node = <BindingIdent as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::BindingIdent);
        node
    }
    #[inline]
    fn visit_mut_lit(&mut self, node: Lit) -> Lit {
        <Lit as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_str(&mut self, node: Str) -> Str {
        self.enter_node(NodeKind::Str);
        let node = <Str as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Str);
        node
    }
    #[inline]
    fn visit_mut_bool(&mut self, node: Bool) -> Bool {
        self.enter_node(NodeKind::Bool);
        let node = <Bool as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Bool);
        node
    }
    #[inline]
    fn visit_mut_null(&mut self, node: Null) -> Null {
        self.enter_node(NodeKind::Null);
        let node = <Null as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Null);
        node
    }
    #[inline]
    fn visit_mut_number(&mut self, node: Number) -> Number {
        self.enter_node(NodeKind::Number);
        let node = <Number as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Number);
        node
    }
    #[inline]
    fn visit_mut_big_int(&mut self, node: BigInt) -> BigInt {
        self.enter_node(NodeKind::BigInt);
        let node = <BigInt as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::BigInt);
        node
    }
    #[inline]
    fn visit_mut_regex(&mut self, node: Regex) -> Regex {
        self.enter_node(NodeKind::Regex);
        let node = <Regex as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Regex);
        node
    }
    #[inline]
    fn visit_mut_jsx_object(&mut self, node: JSXObject) -> JSXObject {
        <JSXObject as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_member_expr(&mut self, node: JSXMemberExpr) -> JSXMemberExpr {
        self.enter_node(NodeKind::JSXMemberExpr);
        let node = <JSXMemberExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXMemberExpr);
        node
    }
    #[inline]
    fn visit_mut_jsx_namespaced_name(&mut self, node: JSXNamespacedName) -> JSXNamespacedName {
        self.enter_node(NodeKind::JSXNamespacedName);
        let node = <JSXNamespacedName as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXNamespacedName);
        node
    }
    #[inline]
    fn visit_mut_jsx_empty_expr(&mut self, node: JSXEmptyExpr) -> JSXEmptyExpr {
        self.enter_node(NodeKind::JSXEmptyExpr);
        let node = <JSXEmptyExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXEmptyExpr);
        node
    }
    #[inline]
    fn visit_mut_jsx_expr_container(&mut self, node: JSXExprContainer) -> JSXExprContainer {
        self.enter_node(NodeKind::JSXExprContainer);
        let node = <JSXExprContainer as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXExprContainer);
        node
    }
    #[inline]
    fn visit_mut_jsx_expr(&mut self, node: JSXExpr) -> JSXExpr {
        <JSXExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_spread_child(&mut self, node: JSXSpreadChild) -> JSXSpreadChild {
        self.enter_node(NodeKind::JSXSpreadChild);
        let node = <JSXSpreadChild as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXSpreadChild);
        node
    }
    #[inline]
    fn visit_mut_jsx_element_name(&mut self, node: JSXElementName) -> JSXElementName {
        <JSXElementName as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_opening_element(&mut self, node: JSXOpeningElement) -> JSXOpeningElement {
        self.enter_node(NodeKind::JSXOpeningElement);
        let node = <JSXOpeningElement as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXOpeningElement);
        node
    }
    #[inline]
    fn visit_mut_jsx_attr_or_spread(&mut self, node: JSXAttrOrSpread) -> JSXAttrOrSpread {
        <JSXAttrOrSpread as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_closing_element(&mut self, node: JSXClosingElement) -> JSXClosingElement {
        self.enter_node(NodeKind::JSXClosingElement);
        let node = <JSXClosingElement as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXClosingElement);
        node
    }
    #[inline]
    fn visit_mut_jsx_attr(&mut self, node: JSXAttr) -> JSXAttr {
        self.enter_node(NodeKind::JSXAttr);
        let node = <JSXAttr as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXAttr);
        node
    }
    #[inline]
    fn visit_mut_jsx_attr_name(&mut self, node: JSXAttrName) -> JSXAttrName {
        <JSXAttrName as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_attr_value(&mut self, node: JSXAttrValue) -> JSXAttrValue {
        <JSXAttrValue as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_text(&mut self, node: JSXText) -> JSXText {
        self.enter_node(NodeKind::JSXText);
        let node = <JSXText as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXText);
        node
    }
    #[inline]
    fn visit_mut_jsx_element(&mut self, node: JSXElement) -> JSXElement {
        self.enter_node(NodeKind::JSXElement);
        let node = <JSXElement as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXElement);
        node
    }
    #[inline]
    fn visit_mut_jsx_element_child(&mut self, node: JSXElementChild) -> JSXElementChild {
        <JSXElementChild as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_fragment(&mut self, node: JSXFragment) -> JSXFragment {
        self.enter_node(NodeKind::JSXFragment);
        let node = <JSXFragment as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXFragment);
        node
    }
    #[inline]
    fn visit_mut_jsx_opening_fragment(&mut self, node: JSXOpeningFragment) -> JSXOpeningFragment {
        self.enter_node(NodeKind::JSXOpeningFragment);
        let node = <JSXOpeningFragment as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXOpeningFragment);
        node
    }
    #[inline]
    fn visit_mut_jsx_closing_fragment(&mut self, node: JSXClosingFragment) -> JSXClosingFragment {
        self.enter_node(NodeKind::JSXClosingFragment);
        let node = <JSXClosingFragment as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXClosingFragment);
        node
    }
    #[inline]
    fn visit_mut_module_items(
        &mut self,
        node: TypedSubRange<ModuleItem>,
    ) -> TypedSubRange<ModuleItem> {
        <TypedSubRange<ModuleItem> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_stmts(&mut self, node: TypedSubRange<Stmt>) -> TypedSubRange<Stmt> {
        <TypedSubRange<Stmt> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_import_specifiers(
        &mut self,
        node: TypedSubRange<ImportSpecifier>,
    ) -> TypedSubRange<ImportSpecifier> {
        <TypedSubRange<ImportSpecifier> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_opt_object_lit(&mut self, node: Option<ObjectLit>) -> Option<ObjectLit> {
        <Option<ObjectLit> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_module_export_name(
        &mut self,
        node: Option<ModuleExportName>,
    ) -> Option<ModuleExportName> {
        <Option<ModuleExportName> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_specifiers(
        &mut self,
        node: TypedSubRange<ExportSpecifier>,
    ) -> TypedSubRange<ExportSpecifier> {
        <TypedSubRange<ExportSpecifier> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_opt_str(&mut self, node: Option<Str>) -> Option<Str> {
        <Option<Str> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_expr(&mut self, node: Option<Expr>) -> Option<Expr> {
        <Option<Expr> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_ident(&mut self, node: Option<Ident>) -> Option<Ident> {
        <Option<Ident> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_stmt(&mut self, node: Option<Stmt>) -> Option<Stmt> {
        <Option<Stmt> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_switch_cases(
        &mut self,
        node: TypedSubRange<SwitchCase>,
    ) -> TypedSubRange<SwitchCase> {
        <TypedSubRange<SwitchCase> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_opt_catch_clause(&mut self, node: Option<CatchClause>) -> Option<CatchClause> {
        <Option<CatchClause> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_block_stmt(&mut self, node: Option<BlockStmt>) -> Option<BlockStmt> {
        <Option<BlockStmt> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_var_decl_or_expr(
        &mut self,
        node: Option<VarDeclOrExpr>,
    ) -> Option<VarDeclOrExpr> {
        <Option<VarDeclOrExpr> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_pat(&mut self, node: Option<Pat>) -> Option<Pat> {
        <Option<Pat> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_var_declarators(
        &mut self,
        node: TypedSubRange<VarDeclarator>,
    ) -> TypedSubRange<VarDeclarator> {
        <TypedSubRange<VarDeclarator> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_opt_expr_or_spread(&mut self, node: Option<ExprOrSpread>) -> Option<ExprOrSpread> {
        <Option<ExprOrSpread> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_vec_expr_or_spreads(
        &mut self,
        node: TypedSubRange<Option<ExprOrSpread>>,
    ) -> TypedSubRange<Option<ExprOrSpread>> {
        <TypedSubRange<Option<ExprOrSpread>> as VisitMutWith<Self>>::visit_mut_children_with(
            node, self,
        );
        node
    }
    #[inline]
    fn visit_mut_prop_or_spreads(
        &mut self,
        node: TypedSubRange<PropOrSpread>,
    ) -> TypedSubRange<PropOrSpread> {
        <TypedSubRange<PropOrSpread> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_expr_or_spreads(
        &mut self,
        node: TypedSubRange<ExprOrSpread>,
    ) -> TypedSubRange<ExprOrSpread> {
        <TypedSubRange<ExprOrSpread> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_opt_expr_or_spreads(
        &mut self,
        node: Option<TypedSubRange<ExprOrSpread>>,
    ) -> Option<TypedSubRange<ExprOrSpread>> {
        <Option<TypedSubRange<ExprOrSpread>> as VisitMutWith<Self>>::visit_mut_children_with(
            node, self,
        )
    }
    #[inline]
    fn visit_mut_exprs(&mut self, node: TypedSubRange<Expr>) -> TypedSubRange<Expr> {
        <TypedSubRange<Expr> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_pats(&mut self, node: TypedSubRange<Pat>) -> TypedSubRange<Pat> {
        <TypedSubRange<Pat> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_tpl_elements(
        &mut self,
        node: TypedSubRange<TplElement>,
    ) -> TypedSubRange<TplElement> {
        <TypedSubRange<TplElement> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_opt_spread_dot_3_token(
        &mut self,
        node: Option<SpreadDot3Token>,
    ) -> Option<SpreadDot3Token> {
        <Option<SpreadDot3Token> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_params(&mut self, node: TypedSubRange<Param>) -> TypedSubRange<Param> {
        <TypedSubRange<Param> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_decorators(&mut self, node: TypedSubRange<Decorator>) -> TypedSubRange<Decorator> {
        <TypedSubRange<Decorator> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_class_members(
        &mut self,
        node: TypedSubRange<ClassMember>,
    ) -> TypedSubRange<ClassMember> {
        <TypedSubRange<ClassMember> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_param_or_ts_param_props(
        &mut self,
        node: TypedSubRange<ParamOrTsParamProp>,
    ) -> TypedSubRange<ParamOrTsParamProp> {
        <TypedSubRange<ParamOrTsParamProp> as VisitMutWith<Self>>::visit_mut_children_with(
            node, self,
        );
        node
    }
    #[inline]
    fn visit_mut_opt_vec_pats(
        &mut self,
        node: TypedSubRange<Option<Pat>>,
    ) -> TypedSubRange<Option<Pat>> {
        <TypedSubRange<Option<Pat>> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_object_pat_props(
        &mut self,
        node: TypedSubRange<ObjectPatProp>,
    ) -> TypedSubRange<ObjectPatProp> {
        <TypedSubRange<ObjectPatProp> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_jsx_attr_or_spreads(
        &mut self,
        node: TypedSubRange<JSXAttrOrSpread>,
    ) -> TypedSubRange<JSXAttrOrSpread> {
        <TypedSubRange<JSXAttrOrSpread> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_opt_jsx_attr_value(&mut self, node: Option<JSXAttrValue>) -> Option<JSXAttrValue> {
        <Option<JSXAttrValue> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_element_childs(
        &mut self,
        node: TypedSubRange<JSXElementChild>,
    ) -> TypedSubRange<JSXElementChild> {
        <TypedSubRange<JSXElementChild> as VisitMutWith<Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_opt_jsx_closing_element(
        &mut self,
        node: Option<JSXClosingElement>,
    ) -> Option<JSXClosingElement> {
        <Option<JSXClosingElement> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
    }
}
pub trait VisitMutWith<V: ?Sized + VisitMut> {
    fn visit_mut_with(self, visitor: &mut V) -> Self;
    fn visit_mut_children_with(self, visitor: &mut V) -> Self;
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Program {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_program(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Module(it) => {
                Self::Module(<Module as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Script(it) => {
                Self::Script(<Script as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Module {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_module(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.body(visitor.ast());
        let new_node =
            <TypedSubRange<ModuleItem> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        let field_value = self.shebang(visitor.ast());
        let new_node = <OptionalUtf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_shebang(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Script {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_script(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.body(visitor.ast());
        let new_node =
            <TypedSubRange<Stmt> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        let field_value = self.shebang(visitor.ast());
        let new_node = <OptionalUtf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_shebang(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ModuleItem {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_module_item(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::ModuleDecl(it) => {
                Self::ModuleDecl(<ModuleDecl as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Stmt(it) => Self::Stmt(<Stmt as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ModuleDecl {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_module_decl(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Import(it) => {
                Self::Import(<ImportDecl as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::ExportDecl(it) => {
                Self::ExportDecl(<ExportDecl as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::ExportNamed(it) => Self::ExportNamed(
                <NamedExport as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::ExportDefaultDecl(it) => Self::ExportDefaultDecl(
                <ExportDefaultDecl as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::ExportDefaultExpr(it) => Self::ExportDefaultExpr(
                <ExportDefaultExpr as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::ExportAll(it) => {
                Self::ExportAll(<ExportAll as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ImportDecl {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_import_decl(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.specifiers(visitor.ast());
        let new_node = <TypedSubRange<ImportSpecifier> as VisitMutWith<V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_specifiers(visitor.ast(), new_node);
        let field_value = self.src(visitor.ast());
        let new_node = <Str as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_src(visitor.ast(), new_node);
        let field_value = self.type_only(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_type_only(visitor.ast(), new_node);
        let field_value = self.with(visitor.ast());
        let new_node = <Option<ObjectLit> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_with(visitor.ast(), new_node);
        let field_value = self.phase(visitor.ast());
        let new_node = <ImportPhase as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_phase(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ImportSpecifier {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_import_specifier(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Named(it) => Self::Named(
                <ImportNamedSpecifier as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Default(it) => Self::Default(
                <ImportDefaultSpecifier as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Namespace(it) => Self::Namespace(
                <ImportStarAsSpecifier as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ImportNamedSpecifier {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_import_named_specifier(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.local(visitor.ast());
        let new_node = <Ident as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_local(visitor.ast(), new_node);
        let field_value = self.imported(visitor.ast());
        let new_node =
            <Option<ModuleExportName> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_imported(visitor.ast(), new_node);
        let field_value = self.is_type_only(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_is_type_only(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ImportDefaultSpecifier {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_import_default_specifier(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.local(visitor.ast());
        let new_node = <Ident as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_local(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ImportStarAsSpecifier {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_import_star_as_specifier(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.local(visitor.ast());
        let new_node = <Ident as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_local(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ExportDecl {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_export_decl(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.decl(visitor.ast());
        let new_node = <Decl as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_decl(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for NamedExport {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_named_export(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.specifiers(visitor.ast());
        let new_node = <TypedSubRange<ExportSpecifier> as VisitMutWith<V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_specifiers(visitor.ast(), new_node);
        let field_value = self.src(visitor.ast());
        let new_node = <Option<Str> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_src(visitor.ast(), new_node);
        let field_value = self.type_only(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_type_only(visitor.ast(), new_node);
        let field_value = self.with(visitor.ast());
        let new_node = <Option<ObjectLit> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_with(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ExportSpecifier {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_export_specifier(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Namespace(it) => Self::Namespace(
                <ExportNamespaceSpecifier as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Default(it) => Self::Default(
                <ExportDefaultSpecifier as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Named(it) => Self::Named(
                <ExportNamedSpecifier as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ExportNamespaceSpecifier {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_export_namespace_specifier(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.name(visitor.ast());
        let new_node = <ModuleExportName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_name(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ModuleExportName {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_module_export_name(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => Self::Ident(<Ident as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Str(it) => Self::Str(<Str as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ExportDefaultSpecifier {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_export_default_specifier(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.exported(visitor.ast());
        let new_node = <Ident as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_exported(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ExportNamedSpecifier {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_export_named_specifier(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.orig(visitor.ast());
        let new_node = <ModuleExportName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_orig(visitor.ast(), new_node);
        let field_value = self.exported(visitor.ast());
        let new_node =
            <Option<ModuleExportName> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_exported(visitor.ast(), new_node);
        let field_value = self.is_type_only(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_is_type_only(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ExportDefaultDecl {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_export_default_decl(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.decl(visitor.ast());
        let new_node = <DefaultDecl as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_decl(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for DefaultDecl {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_default_decl(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Class(it) => {
                Self::Class(<ClassExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Fn(it) => Self::Fn(<FnExpr as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ExportDefaultExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_export_default_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ExportAll {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_export_all(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.src(visitor.ast());
        let new_node = <Str as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_src(visitor.ast(), new_node);
        let field_value = self.type_only(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_type_only(visitor.ast(), new_node);
        let field_value = self.with(visitor.ast());
        let new_node = <Option<ObjectLit> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_with(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for BlockStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_block_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.stmts(visitor.ast());
        let new_node =
            <TypedSubRange<Stmt> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_stmts(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Stmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Block(it) => {
                Self::Block(<BlockStmt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Empty(it) => {
                Self::Empty(<EmptyStmt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Debugger(it) => Self::Debugger(
                <DebuggerStmt as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::With(it) => {
                Self::With(<WithStmt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Return(it) => {
                Self::Return(<ReturnStmt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Labeled(it) => Self::Labeled(<LabeledStmt as VisitMutWith<V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Break(it) => {
                Self::Break(<BreakStmt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Continue(it) => Self::Continue(
                <ContinueStmt as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::If(it) => Self::If(<IfStmt as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Switch(it) => {
                Self::Switch(<SwitchStmt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Throw(it) => {
                Self::Throw(<ThrowStmt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Try(it) => Self::Try(<TryStmt as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::While(it) => {
                Self::While(<WhileStmt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::DoWhile(it) => Self::DoWhile(<DoWhileStmt as VisitMutWith<V>>::visit_mut_with(
                it, visitor,
            )),
            Self::For(it) => Self::For(<ForStmt as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::ForIn(it) => {
                Self::ForIn(<ForInStmt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::ForOf(it) => {
                Self::ForOf(<ForOfStmt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Decl(it) => Self::Decl(<Decl as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Expr(it) => {
                Self::Expr(<ExprStmt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ExprStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_expr_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for EmptyStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_empty_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for DebuggerStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_debugger_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for WithStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_with_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.obj(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_obj(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Stmt as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ReturnStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_return_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.arg(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_arg(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for LabeledStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_labeled_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.label(visitor.ast());
        let new_node = <Ident as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_label(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Stmt as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for BreakStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_break_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.label(visitor.ast());
        let new_node = <Option<Ident> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_label(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ContinueStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_continue_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.label(visitor.ast());
        let new_node = <Option<Ident> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_label(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for IfStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_if_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.test(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_test(visitor.ast(), new_node);
        let field_value = self.cons(visitor.ast());
        let new_node = <Stmt as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_cons(visitor.ast(), new_node);
        let field_value = self.alt(visitor.ast());
        let new_node = <Option<Stmt> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_alt(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for SwitchStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_switch_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.discriminant(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_discriminant(visitor.ast(), new_node);
        let field_value = self.cases(visitor.ast());
        let new_node =
            <TypedSubRange<SwitchCase> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_cases(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ThrowStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_throw_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.arg(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_arg(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TryStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_try_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.block(visitor.ast());
        let new_node = <BlockStmt as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_block(visitor.ast(), new_node);
        let field_value = self.handler(visitor.ast());
        let new_node =
            <Option<CatchClause> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_handler(visitor.ast(), new_node);
        let field_value = self.finalizer(visitor.ast());
        let new_node = <Option<BlockStmt> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_finalizer(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for WhileStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_while_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.test(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_test(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Stmt as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for DoWhileStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_do_while_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.test(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_test(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Stmt as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ForStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_for_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.init(visitor.ast());
        let new_node =
            <Option<VarDeclOrExpr> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_init(visitor.ast(), new_node);
        let field_value = self.test(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_test(visitor.ast(), new_node);
        let field_value = self.update(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_update(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Stmt as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ForInStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_for_in_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.left(visitor.ast());
        let new_node = <ForHead as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_left(visitor.ast(), new_node);
        let field_value = self.right(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_right(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Stmt as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ForOfStmt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_for_of_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.is_await(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_is_await(visitor.ast(), new_node);
        let field_value = self.left(visitor.ast());
        let new_node = <ForHead as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_left(visitor.ast(), new_node);
        let field_value = self.right(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_right(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Stmt as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for SwitchCase {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_switch_case(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.test(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_test(visitor.ast(), new_node);
        let field_value = self.cons(visitor.ast());
        let new_node =
            <TypedSubRange<Stmt> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_cons(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for CatchClause {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_catch_clause(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.param(visitor.ast());
        let new_node = <Option<Pat> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_param(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <BlockStmt as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ForHead {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_for_head(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::VarDecl(it) => {
                Self::VarDecl(<VarDecl as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::UsingDecl(it) => {
                Self::UsingDecl(<UsingDecl as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Pat(it) => Self::Pat(<Pat as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for VarDeclOrExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_var_decl_or_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::VarDecl(it) => {
                Self::VarDecl(<VarDecl as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Expr(it) => Self::Expr(<Expr as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Decl {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_decl(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Class(it) => {
                Self::Class(<ClassDecl as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Fn(it) => Self::Fn(<FnDecl as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Var(it) => Self::Var(<VarDecl as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Using(it) => {
                Self::Using(<UsingDecl as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for FnDecl {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_fn_decl(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.ident(visitor.ast());
        let new_node = <Ident as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_ident(visitor.ast(), new_node);
        let field_value = self.declare(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_declare(visitor.ast(), new_node);
        let field_value = self.function(visitor.ast());
        let new_node = <Function as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_function(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ClassDecl {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_class_decl(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.ident(visitor.ast());
        let new_node = <Ident as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_ident(visitor.ast(), new_node);
        let field_value = self.declare(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_declare(visitor.ast(), new_node);
        let field_value = self.class(visitor.ast());
        let new_node = <Class as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_class(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for VarDecl {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_var_decl(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.kind(visitor.ast());
        let new_node = <VarDeclKind as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_kind(visitor.ast(), new_node);
        let field_value = self.declare(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_declare(visitor.ast(), new_node);
        let field_value = self.decls(visitor.ast());
        let new_node =
            <TypedSubRange<VarDeclarator> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_decls(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for VarDeclarator {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_var_declarator(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.name(visitor.ast());
        let new_node = <Pat as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_name(visitor.ast(), new_node);
        let field_value = self.init(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_init(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for UsingDecl {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_using_decl(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.is_await(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_is_await(visitor.ast(), new_node);
        let field_value = self.decls(visitor.ast());
        let new_node =
            <TypedSubRange<VarDeclarator> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_decls(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Expr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::This(it) => {
                Self::This(<ThisExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Array(it) => {
                Self::Array(<ArrayLit as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Object(it) => {
                Self::Object(<ObjectLit as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Fn(it) => Self::Fn(<FnExpr as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Unary(it) => {
                Self::Unary(<UnaryExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Update(it) => {
                Self::Update(<UpdateExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Bin(it) => Self::Bin(<BinExpr as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Assign(it) => {
                Self::Assign(<AssignExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Member(it) => {
                Self::Member(<MemberExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::SuperProp(it) => Self::SuperProp(
                <SuperPropExpr as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Cond(it) => {
                Self::Cond(<CondExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Call(it) => {
                Self::Call(<CallExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::New(it) => Self::New(<NewExpr as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Seq(it) => Self::Seq(<SeqExpr as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Ident(it) => Self::Ident(<Ident as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Lit(it) => Self::Lit(<Lit as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Tpl(it) => Self::Tpl(<Tpl as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::TaggedTpl(it) => {
                Self::TaggedTpl(<TaggedTpl as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Arrow(it) => {
                Self::Arrow(<ArrowExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Class(it) => {
                Self::Class(<ClassExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Yield(it) => {
                Self::Yield(<YieldExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::MetaProp(it) => Self::MetaProp(
                <MetaPropExpr as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Await(it) => {
                Self::Await(<AwaitExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Paren(it) => {
                Self::Paren(<ParenExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::JSXMember(it) => Self::JSXMember(
                <JSXMemberExpr as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXNamespacedName(it) => Self::JSXNamespacedName(
                <JSXNamespacedName as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXEmpty(it) => Self::JSXEmpty(
                <JSXEmptyExpr as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXElement(it) => {
                Self::JSXElement(<JSXElement as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::JSXFragment(it) => Self::JSXFragment(
                <JSXFragment as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::PrivateName(it) => Self::PrivateName(
                <PrivateName as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::OptChain(it) => Self::OptChain(
                <OptChainExpr as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Invalid(it) => {
                Self::Invalid(<Invalid as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ThisExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_this_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ArrayLit {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_array_lit(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.elems(visitor.ast());
        let new_node = <TypedSubRange<Option<ExprOrSpread>> as VisitMutWith<V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_elems(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ObjectLit {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_object_lit(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.props(visitor.ast());
        let new_node =
            <TypedSubRange<PropOrSpread> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_props(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for PropOrSpread {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_prop_or_spread(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::SpreadElement(it) => Self::SpreadElement(
                <SpreadElement as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Prop(it) => Self::Prop(<Prop as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for SpreadElement {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_spread_element(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.dot_3_token(visitor.ast());
        let new_node = <Span as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_dot3_token(visitor.ast(), new_node);
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for UnaryExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_unary_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.op(visitor.ast());
        let new_node = <UnaryOp as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_op(visitor.ast(), new_node);
        let field_value = self.arg(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_arg(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for UpdateExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_update_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.op(visitor.ast());
        let new_node = <UpdateOp as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_op(visitor.ast(), new_node);
        let field_value = self.prefix(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_prefix(visitor.ast(), new_node);
        let field_value = self.arg(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_arg(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for BinExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_bin_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.op(visitor.ast());
        let new_node = <BinaryOp as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_op(visitor.ast(), new_node);
        let field_value = self.left(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_left(visitor.ast(), new_node);
        let field_value = self.right(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_right(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for FnExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_fn_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.ident(visitor.ast());
        let new_node = <Option<Ident> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_ident(visitor.ast(), new_node);
        let field_value = self.function(visitor.ast());
        let new_node = <Function as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_function(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ClassExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_class_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.ident(visitor.ast());
        let new_node = <Option<Ident> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_ident(visitor.ast(), new_node);
        let field_value = self.class(visitor.ast());
        let new_node = <Class as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_class(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for AssignExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_assign_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.op(visitor.ast());
        let new_node = <AssignOp as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_op(visitor.ast(), new_node);
        let field_value = self.left(visitor.ast());
        let new_node = <AssignTarget as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_left(visitor.ast(), new_node);
        let field_value = self.right(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_right(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for MemberExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_member_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.obj(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_obj(visitor.ast(), new_node);
        let field_value = self.prop(visitor.ast());
        let new_node = <MemberProp as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_prop(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for MemberProp {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_member_prop(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => {
                Self::Ident(<IdentName as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::PrivateName(it) => Self::PrivateName(
                <PrivateName as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Computed(it) => Self::Computed(
                <ComputedPropName as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for SuperPropExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_super_prop_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.obj(visitor.ast());
        let new_node = <Super as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_obj(visitor.ast(), new_node);
        let field_value = self.prop(visitor.ast());
        let new_node = <SuperProp as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_prop(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for SuperProp {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_super_prop(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => {
                Self::Ident(<IdentName as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Computed(it) => Self::Computed(
                <ComputedPropName as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for CondExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_cond_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.test(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_test(visitor.ast(), new_node);
        let field_value = self.cons(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_cons(visitor.ast(), new_node);
        let field_value = self.alt(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_alt(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for CallExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_call_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.callee(visitor.ast());
        let new_node = <Callee as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_callee(visitor.ast(), new_node);
        let field_value = self.args(visitor.ast());
        let new_node =
            <TypedSubRange<ExprOrSpread> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_args(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for NewExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_new_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.callee(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_callee(visitor.ast(), new_node);
        let field_value = self.args(visitor.ast());
        let new_node = <Option<TypedSubRange<ExprOrSpread>> as VisitMutWith<V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_args(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for SeqExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_seq_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.exprs(visitor.ast());
        let new_node =
            <TypedSubRange<Expr> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_exprs(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ArrowExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_arrow_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.params(visitor.ast());
        let new_node =
            <TypedSubRange<Pat> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_params(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <BlockStmtOrExpr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        let field_value = self.is_async(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_is_async(visitor.ast(), new_node);
        let field_value = self.is_generator(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_is_generator(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for YieldExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_yield_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.arg(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_arg(visitor.ast(), new_node);
        let field_value = self.delegate(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_delegate(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for MetaPropExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_meta_prop_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.kind(visitor.ast());
        let new_node = <MetaPropKind as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_kind(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for AwaitExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_await_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.arg(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_arg(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Tpl {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_tpl(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.exprs(visitor.ast());
        let new_node =
            <TypedSubRange<Expr> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_exprs(visitor.ast(), new_node);
        let field_value = self.quasis(visitor.ast());
        let new_node =
            <TypedSubRange<TplElement> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_quasis(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TaggedTpl {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_tagged_tpl(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.tag(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_tag(visitor.ast(), new_node);
        let field_value = self.tpl(visitor.ast());
        let new_node = <Tpl as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_tpl(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TplElement {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_tpl_element(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.tail(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_tail(visitor.ast(), new_node);
        let field_value = self.cooked(visitor.ast());
        let new_node = <OptionalWtf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_cooked(visitor.ast(), new_node);
        let field_value = self.raw(visitor.ast());
        let new_node = <Utf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_raw(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ParenExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_paren_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Callee {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_callee(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Super(it) => Self::Super(<Super as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Import(it) => {
                Self::Import(<Import as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Expr(it) => Self::Expr(<Expr as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Super {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_super(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Import {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_import(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.phase(visitor.ast());
        let new_node = <ImportPhase as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_phase(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ExprOrSpread {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_expr_or_spread(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.spread(visitor.ast());
        let new_node =
            <Option<SpreadDot3Token> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_spread(visitor.ast(), new_node);
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for SpreadDot3Token {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_spread_dot_3_token(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for BlockStmtOrExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_block_stmt_or_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::BlockStmt(it) => {
                Self::BlockStmt(<BlockStmt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Expr(it) => Self::Expr(<Expr as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for AssignTarget {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_assign_target(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Simple(it) => Self::Simple(
                <SimpleAssignTarget as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Pat(it) => Self::Pat(<AssignTargetPat as VisitMutWith<V>>::visit_mut_with(
                it, visitor,
            )),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for AssignTargetPat {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_assign_target_pat(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Array(it) => {
                Self::Array(<ArrayPat as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Object(it) => {
                Self::Object(<ObjectPat as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Invalid(it) => {
                Self::Invalid(<Invalid as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for SimpleAssignTarget {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_simple_assign_target(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => Self::Ident(<BindingIdent as VisitMutWith<V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Member(it) => {
                Self::Member(<MemberExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::SuperProp(it) => Self::SuperProp(
                <SuperPropExpr as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Paren(it) => {
                Self::Paren(<ParenExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::OptChain(it) => Self::OptChain(
                <OptChainExpr as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Invalid(it) => {
                Self::Invalid(<Invalid as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for OptChainExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_chain_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.optional(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_optional(visitor.ast(), new_node);
        let field_value = self.base(visitor.ast());
        let new_node = <OptChainBase as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_base(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for OptChainBase {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_chain_base(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Member(it) => {
                Self::Member(<MemberExpr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Call(it) => Self::Call(<OptCall as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for OptCall {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_call(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.callee(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_callee(visitor.ast(), new_node);
        let field_value = self.args(visitor.ast());
        let new_node =
            <TypedSubRange<ExprOrSpread> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_args(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Invalid {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_invalid(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Function {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_function(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.params(visitor.ast());
        let new_node =
            <TypedSubRange<Param> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_params(visitor.ast(), new_node);
        let field_value = self.decorators(visitor.ast());
        let new_node =
            <TypedSubRange<Decorator> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_decorators(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Option<BlockStmt> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        let field_value = self.is_generator(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_is_generator(visitor.ast(), new_node);
        let field_value = self.is_async(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_is_async(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Param {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_param(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.decorators(visitor.ast());
        let new_node =
            <TypedSubRange<Decorator> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_decorators(visitor.ast(), new_node);
        let field_value = self.pat(visitor.ast());
        let new_node = <Pat as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_pat(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ParamOrTsParamProp {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_param_or_ts_param_prop(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Param(it) => Self::Param(<Param as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Class {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_class(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.decorators(visitor.ast());
        let new_node =
            <TypedSubRange<Decorator> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_decorators(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node =
            <TypedSubRange<ClassMember> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        let field_value = self.super_class(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_super_class(visitor.ast(), new_node);
        let field_value = self.is_abstract(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_is_abstract(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ClassMember {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_class_member(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Constructor(it) => Self::Constructor(
                <Constructor as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Method(it) => Self::Method(<ClassMethod as VisitMutWith<V>>::visit_mut_with(
                it, visitor,
            )),
            Self::PrivateMethod(it) => Self::PrivateMethod(
                <PrivateMethod as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::ClassProp(it) => {
                Self::ClassProp(<ClassProp as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::PrivateProp(it) => Self::PrivateProp(
                <PrivateProp as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Empty(it) => {
                Self::Empty(<EmptyStmt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::StaticBlock(it) => Self::StaticBlock(
                <StaticBlock as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::AutoAccessor(it) => Self::AutoAccessor(
                <AutoAccessor as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ClassProp {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_class_prop(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        let field_value = self.is_static(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_is_static(visitor.ast(), new_node);
        let field_value = self.decorators(visitor.ast());
        let new_node =
            <TypedSubRange<Decorator> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_decorators(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for PrivateProp {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_private_prop(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PrivateName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        let field_value = self.is_static(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_is_static(visitor.ast(), new_node);
        let field_value = self.decorators(visitor.ast());
        let new_node =
            <TypedSubRange<Decorator> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_decorators(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ClassMethod {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_class_method(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.function(visitor.ast());
        let new_node = <Function as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_function(visitor.ast(), new_node);
        let field_value = self.kind(visitor.ast());
        let new_node = <MethodKind as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_kind(visitor.ast(), new_node);
        let field_value = self.is_static(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_is_static(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for PrivateMethod {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_private_method(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PrivateName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.function(visitor.ast());
        let new_node = <Function as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_function(visitor.ast(), new_node);
        let field_value = self.kind(visitor.ast());
        let new_node = <MethodKind as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_kind(visitor.ast(), new_node);
        let field_value = self.is_static(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_is_static(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Constructor {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_constructor(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.params(visitor.ast());
        let new_node = <TypedSubRange<ParamOrTsParamProp> as VisitMutWith<V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_params(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Option<BlockStmt> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Decorator {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_decorator(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for StaticBlock {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_static_block(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.body(visitor.ast());
        let new_node = <BlockStmt as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Key {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_key(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Private(it) => Self::Private(<PrivateName as VisitMutWith<V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Public(it) => {
                Self::Public(<PropName as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for AutoAccessor {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_auto_accessor(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <Key as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        let field_value = self.is_static(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_is_static(visitor.ast(), new_node);
        let field_value = self.decorators(visitor.ast());
        let new_node =
            <TypedSubRange<Decorator> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_decorators(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Prop {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_prop(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Shorthand(it) => {
                Self::Shorthand(<Ident as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::KeyValue(it) => Self::KeyValue(
                <KeyValueProp as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Assign(it) => {
                Self::Assign(<AssignProp as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Getter(it) => {
                Self::Getter(<GetterProp as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Setter(it) => {
                Self::Setter(<SetterProp as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Method(it) => {
                Self::Method(<MethodProp as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for KeyValueProp {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_key_value_prop(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for AssignProp {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_assign_prop(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <Ident as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for GetterProp {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_getter_prop(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Option<BlockStmt> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for SetterProp {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_setter_prop(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.this_param(visitor.ast());
        let new_node = <Option<Pat> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_this_param(visitor.ast(), new_node);
        let field_value = self.param(visitor.ast());
        let new_node = <Pat as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_param(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Option<BlockStmt> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for MethodProp {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_method_prop(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.function(visitor.ast());
        let new_node = <Function as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_function(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for PropName {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_prop_name(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => {
                Self::Ident(<IdentName as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Str(it) => Self::Str(<Str as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Num(it) => Self::Num(<Number as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Computed(it) => Self::Computed(
                <ComputedPropName as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::BigInt(it) => {
                Self::BigInt(<BigInt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ComputedPropName {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_computed_prop_name(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Pat {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_pat(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => Self::Ident(<BindingIdent as VisitMutWith<V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Array(it) => {
                Self::Array(<ArrayPat as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Rest(it) => Self::Rest(<RestPat as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Object(it) => {
                Self::Object(<ObjectPat as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Assign(it) => {
                Self::Assign(<AssignPat as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Invalid(it) => {
                Self::Invalid(<Invalid as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Expr(it) => Self::Expr(<Expr as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ArrayPat {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_array_pat(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.elems(visitor.ast());
        let new_node =
            <TypedSubRange<Option<Pat>> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_elems(visitor.ast(), new_node);
        let field_value = self.optional(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_optional(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ObjectPat {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_object_pat(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.props(visitor.ast());
        let new_node =
            <TypedSubRange<ObjectPatProp> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_props(visitor.ast(), new_node);
        let field_value = self.optional(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_optional(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for AssignPat {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_assign_pat(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.left(visitor.ast());
        let new_node = <Pat as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_left(visitor.ast(), new_node);
        let field_value = self.right(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_right(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for RestPat {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_rest_pat(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.dot_3_token(visitor.ast());
        let new_node = <Span as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_dot3_token(visitor.ast(), new_node);
        let field_value = self.arg(visitor.ast());
        let new_node = <Pat as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_arg(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for ObjectPatProp {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_object_pat_prop(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::KeyValue(it) => Self::KeyValue(
                <KeyValuePatProp as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Assign(it) => Self::Assign(<AssignPatProp as VisitMutWith<V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Rest(it) => Self::Rest(<RestPat as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for KeyValuePatProp {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_key_value_pat_prop(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node = <Pat as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for AssignPatProp {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_assign_pat_prop(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <BindingIdent as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Ident {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_ident(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.sym(visitor.ast());
        let new_node = <Utf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_sym(visitor.ast(), new_node);
        let field_value = self.optional(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_optional(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for IdentName {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_ident_name(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.sym(visitor.ast());
        let new_node = <Utf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_sym(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for PrivateName {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_private_name(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.name(visitor.ast());
        let new_node = <Utf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_name(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for BindingIdent {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_binding_ident(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.id(visitor.ast());
        let new_node = <Ident as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_id(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Lit {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_lit(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Str(it) => Self::Str(<Str as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Bool(it) => Self::Bool(<Bool as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Null(it) => Self::Null(<Null as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::Num(it) => Self::Num(<Number as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::BigInt(it) => {
                Self::BigInt(<BigInt as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::Regex(it) => Self::Regex(<Regex as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Str {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_str(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.value(visitor.ast());
        let new_node = <Wtf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        let field_value = self.raw(visitor.ast());
        let new_node = <OptionalUtf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_raw(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Bool {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_bool(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.value(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Null {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_null(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Number {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_number(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.value(visitor.ast());
        let new_node = <f64 as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        let field_value = self.raw(visitor.ast());
        let new_node = <OptionalUtf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_raw(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for BigInt {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_big_int(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.value(visitor.ast());
        let new_node = <BigIntId as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        let field_value = self.raw(visitor.ast());
        let new_node = <OptionalUtf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_raw(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Regex {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_regex(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.exp(visitor.ast());
        let new_node = <Utf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_exp(visitor.ast(), new_node);
        let field_value = self.flags(visitor.ast());
        let new_node = <Utf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_flags(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXObject {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_object(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::JSXMemberExpr(it) => Self::JSXMemberExpr(
                <JSXMemberExpr as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Ident(it) => Self::Ident(<Ident as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXMemberExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_member_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.obj(visitor.ast());
        let new_node = <JSXObject as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_obj(visitor.ast(), new_node);
        let field_value = self.prop(visitor.ast());
        let new_node = <IdentName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_prop(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXNamespacedName {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_namespaced_name(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.ns(visitor.ast());
        let new_node = <IdentName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_ns(visitor.ast(), new_node);
        let field_value = self.name(visitor.ast());
        let new_node = <IdentName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_name(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXEmptyExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_empty_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXExprContainer {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_expr_container(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <JSXExpr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXExpr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::JSXEmptyExpr(it) => Self::JSXEmptyExpr(
                <JSXEmptyExpr as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::Expr(it) => Self::Expr(<Expr as VisitMutWith<V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXSpreadChild {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_spread_child(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXElementName {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_element_name(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => Self::Ident(<Ident as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::JSXMemberExpr(it) => Self::JSXMemberExpr(
                <JSXMemberExpr as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXNamespacedName(it) => Self::JSXNamespacedName(
                <JSXNamespacedName as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXOpeningElement {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_opening_element(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.name(visitor.ast());
        let new_node = <JSXElementName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_name(visitor.ast(), new_node);
        let field_value = self.attrs(visitor.ast());
        let new_node = <TypedSubRange<JSXAttrOrSpread> as VisitMutWith<V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_attrs(visitor.ast(), new_node);
        let field_value = self.self_closing(visitor.ast());
        let new_node = <bool as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_self_closing(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXAttrOrSpread {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_attr_or_spread(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::JSXAttr(it) => {
                Self::JSXAttr(<JSXAttr as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::SpreadElement(it) => Self::SpreadElement(
                <SpreadElement as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXClosingElement {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_closing_element(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.name(visitor.ast());
        let new_node = <JSXElementName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_name(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXAttr {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_attr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.name(visitor.ast());
        let new_node = <JSXAttrName as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_name(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node =
            <Option<JSXAttrValue> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXAttrName {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_attr_name(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => {
                Self::Ident(<IdentName as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::JSXNamespacedName(it) => Self::JSXNamespacedName(
                <JSXNamespacedName as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXAttrValue {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_attr_value(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Str(it) => Self::Str(<Str as VisitMutWith<V>>::visit_mut_with(it, visitor)),
            Self::JSXExprContainer(it) => Self::JSXExprContainer(
                <JSXExprContainer as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXElement(it) => {
                Self::JSXElement(<JSXElement as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::JSXFragment(it) => Self::JSXFragment(
                <JSXFragment as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXText {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_text(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.value(visitor.ast());
        let new_node = <Utf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        let field_value = self.raw(visitor.ast());
        let new_node = <Utf8Ref as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_raw(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXElement {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_element(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.opening(visitor.ast());
        let new_node = <JSXOpeningElement as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_opening(visitor.ast(), new_node);
        let field_value = self.children(visitor.ast());
        let new_node = <TypedSubRange<JSXElementChild> as VisitMutWith<V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_children(visitor.ast(), new_node);
        let field_value = self.closing(visitor.ast());
        let new_node =
            <Option<JSXClosingElement> as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_closing(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXElementChild {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_element_child(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::JSXText(it) => {
                Self::JSXText(<JSXText as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::JSXExprContainer(it) => Self::JSXExprContainer(
                <JSXExprContainer as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXSpreadChild(it) => Self::JSXSpreadChild(
                <JSXSpreadChild as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXElement(it) => {
                Self::JSXElement(<JSXElement as VisitMutWith<V>>::visit_mut_with(it, visitor))
            }
            Self::JSXFragment(it) => Self::JSXFragment(
                <JSXFragment as VisitMutWith<V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXFragment {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_fragment(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.opening(visitor.ast());
        let new_node =
            <JSXOpeningFragment as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_opening(visitor.ast(), new_node);
        let field_value = self.children(visitor.ast());
        let new_node = <TypedSubRange<JSXElementChild> as VisitMutWith<V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_children(visitor.ast(), new_node);
        let field_value = self.closing(visitor.ast());
        let new_node =
            <JSXClosingFragment as VisitMutWith<V>>::visit_mut_with(field_value, visitor);
        self.set_closing(visitor.ast(), new_node);
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXOpeningFragment {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_opening_fragment(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for JSXClosingFragment {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_closing_fragment(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<ModuleItem> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_module_items(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<Stmt> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_stmts(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<ImportSpecifier> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_import_specifiers(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<ObjectLit> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_object_lit(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<ModuleExportName> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_module_export_name(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<ExportSpecifier> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_export_specifiers(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<Str> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_str(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<Expr> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<Ident> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_ident(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<Stmt> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<SwitchCase> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_switch_cases(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<CatchClause> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_catch_clause(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<BlockStmt> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_block_stmt(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<VarDeclOrExpr> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_var_decl_or_expr(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<Pat> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_pat(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<VarDeclarator> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_var_declarators(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<ExprOrSpread> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_expr_or_spread(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<Option<ExprOrSpread>> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_vec_expr_or_spreads(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_opt_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<PropOrSpread> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_prop_or_spreads(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<ExprOrSpread> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_expr_or_spreads(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<TypedSubRange<ExprOrSpread>> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_expr_or_spreads(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<Expr> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_exprs(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<Pat> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_pats(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<TplElement> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_tpl_elements(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<SpreadDot3Token> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_spread_dot_3_token(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<Param> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_params(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<Decorator> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_decorators(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<ClassMember> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_class_members(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<ParamOrTsParamProp> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_param_or_ts_param_props(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<Option<Pat>> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_vec_pats(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_opt_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<ObjectPatProp> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_object_pat_props(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<JSXAttrOrSpread> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_attr_or_spreads(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<JSXAttrValue> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_jsx_attr_value(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for TypedSubRange<JSXElementChild> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_jsx_element_childs(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child);
            child.visit_mut_with(visitor);
        }
        self
    }
}
impl<V: ?Sized + VisitMut> VisitMutWith<V> for Option<JSXClosingElement> {
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut>::visit_mut_opt_jsx_closing_element(visitor, self)
    }
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
