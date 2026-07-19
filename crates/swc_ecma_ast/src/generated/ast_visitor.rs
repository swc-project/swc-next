#![allow(unused, clippy::useless_conversion, clippy::single_match)]
use crate::*;
pub trait Visit<'a> {
    fn ast(&self) -> &Ast<'a>;
    #[inline]
    fn enter_node(&mut self, node_id: NodeId) {}
    #[inline]
    fn leave_node(&mut self, node_id: NodeId) {}
    #[inline]
    fn visit_program(&mut self, node: Program) {
        <Program as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_module(&mut self, node: Module) {
        self.enter_node(node.node_id());
        <Module as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_script(&mut self, node: Script) {
        self.enter_node(node.node_id());
        <Script as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_module_item(&mut self, node: ModuleItem) {
        <ModuleItem as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_module_decl(&mut self, node: ModuleDecl) {
        <ModuleDecl as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_import_decl(&mut self, node: ImportDecl) {
        self.enter_node(node.node_id());
        <ImportDecl as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_import_specifier(&mut self, node: ImportSpecifier) {
        <ImportSpecifier as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_import_named_specifier(&mut self, node: ImportNamedSpecifier) {
        self.enter_node(node.node_id());
        <ImportNamedSpecifier as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_import_default_specifier(&mut self, node: ImportDefaultSpecifier) {
        self.enter_node(node.node_id());
        <ImportDefaultSpecifier as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_import_star_as_specifier(&mut self, node: ImportStarAsSpecifier) {
        self.enter_node(node.node_id());
        <ImportStarAsSpecifier as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_export_decl(&mut self, node: ExportDecl) {
        self.enter_node(node.node_id());
        <ExportDecl as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_named_export(&mut self, node: NamedExport) {
        self.enter_node(node.node_id());
        <NamedExport as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_export_specifier(&mut self, node: ExportSpecifier) {
        <ExportSpecifier as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_export_namespace_specifier(&mut self, node: ExportNamespaceSpecifier) {
        self.enter_node(node.node_id());
        <ExportNamespaceSpecifier as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_module_export_name(&mut self, node: ModuleExportName) {
        <ModuleExportName as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_export_default_specifier(&mut self, node: ExportDefaultSpecifier) {
        self.enter_node(node.node_id());
        <ExportDefaultSpecifier as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_export_named_specifier(&mut self, node: ExportNamedSpecifier) {
        self.enter_node(node.node_id());
        <ExportNamedSpecifier as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_export_default_decl(&mut self, node: ExportDefaultDecl) {
        self.enter_node(node.node_id());
        <ExportDefaultDecl as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_default_decl(&mut self, node: DefaultDecl) {
        <DefaultDecl as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_export_default_expr(&mut self, node: ExportDefaultExpr) {
        self.enter_node(node.node_id());
        <ExportDefaultExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_export_all(&mut self, node: ExportAll) {
        self.enter_node(node.node_id());
        <ExportAll as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_block_stmt(&mut self, node: BlockStmt) {
        self.enter_node(node.node_id());
        <BlockStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_stmt(&mut self, node: Stmt) {
        <Stmt as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_expr_stmt(&mut self, node: ExprStmt) {
        self.enter_node(node.node_id());
        <ExprStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_empty_stmt(&mut self, node: EmptyStmt) {
        self.enter_node(node.node_id());
        <EmptyStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_debugger_stmt(&mut self, node: DebuggerStmt) {
        self.enter_node(node.node_id());
        <DebuggerStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_with_stmt(&mut self, node: WithStmt) {
        self.enter_node(node.node_id());
        <WithStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_return_stmt(&mut self, node: ReturnStmt) {
        self.enter_node(node.node_id());
        <ReturnStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_labeled_stmt(&mut self, node: LabeledStmt) {
        self.enter_node(node.node_id());
        <LabeledStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_break_stmt(&mut self, node: BreakStmt) {
        self.enter_node(node.node_id());
        <BreakStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_continue_stmt(&mut self, node: ContinueStmt) {
        self.enter_node(node.node_id());
        <ContinueStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_if_stmt(&mut self, node: IfStmt) {
        self.enter_node(node.node_id());
        <IfStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_switch_stmt(&mut self, node: SwitchStmt) {
        self.enter_node(node.node_id());
        <SwitchStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_throw_stmt(&mut self, node: ThrowStmt) {
        self.enter_node(node.node_id());
        <ThrowStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_try_stmt(&mut self, node: TryStmt) {
        self.enter_node(node.node_id());
        <TryStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_while_stmt(&mut self, node: WhileStmt) {
        self.enter_node(node.node_id());
        <WhileStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_do_while_stmt(&mut self, node: DoWhileStmt) {
        self.enter_node(node.node_id());
        <DoWhileStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_for_stmt(&mut self, node: ForStmt) {
        self.enter_node(node.node_id());
        <ForStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_for_in_stmt(&mut self, node: ForInStmt) {
        self.enter_node(node.node_id());
        <ForInStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_for_of_stmt(&mut self, node: ForOfStmt) {
        self.enter_node(node.node_id());
        <ForOfStmt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_switch_case(&mut self, node: SwitchCase) {
        self.enter_node(node.node_id());
        <SwitchCase as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_catch_clause(&mut self, node: CatchClause) {
        self.enter_node(node.node_id());
        <CatchClause as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_for_head(&mut self, node: ForHead) {
        <ForHead as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_var_decl_or_expr(&mut self, node: VarDeclOrExpr) {
        <VarDeclOrExpr as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_decl(&mut self, node: Decl) {
        <Decl as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_fn_decl(&mut self, node: FnDecl) {
        self.enter_node(node.node_id());
        <FnDecl as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_class_decl(&mut self, node: ClassDecl) {
        self.enter_node(node.node_id());
        <ClassDecl as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_var_decl(&mut self, node: VarDecl) {
        self.enter_node(node.node_id());
        <VarDecl as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_var_declarator(&mut self, node: VarDeclarator) {
        self.enter_node(node.node_id());
        <VarDeclarator as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_using_decl(&mut self, node: UsingDecl) {
        self.enter_node(node.node_id());
        <UsingDecl as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_expr(&mut self, node: Expr) {
        <Expr as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_this_expr(&mut self, node: ThisExpr) {
        self.enter_node(node.node_id());
        <ThisExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_array_lit(&mut self, node: ArrayLit) {
        self.enter_node(node.node_id());
        <ArrayLit as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_object_lit(&mut self, node: ObjectLit) {
        self.enter_node(node.node_id());
        <ObjectLit as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_prop_or_spread(&mut self, node: PropOrSpread) {
        <PropOrSpread as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_spread_element(&mut self, node: SpreadElement) {
        self.enter_node(node.node_id());
        <SpreadElement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_unary_expr(&mut self, node: UnaryExpr) {
        self.enter_node(node.node_id());
        <UnaryExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_update_expr(&mut self, node: UpdateExpr) {
        self.enter_node(node.node_id());
        <UpdateExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_bin_expr(&mut self, node: BinExpr) {
        self.enter_node(node.node_id());
        <BinExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_fn_expr(&mut self, node: FnExpr) {
        self.enter_node(node.node_id());
        <FnExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_class_expr(&mut self, node: ClassExpr) {
        self.enter_node(node.node_id());
        <ClassExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_assign_expr(&mut self, node: AssignExpr) {
        self.enter_node(node.node_id());
        <AssignExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_member_expr(&mut self, node: MemberExpr) {
        self.enter_node(node.node_id());
        <MemberExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_member_prop(&mut self, node: MemberProp) {
        <MemberProp as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_super_prop_expr(&mut self, node: SuperPropExpr) {
        self.enter_node(node.node_id());
        <SuperPropExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_super_prop(&mut self, node: SuperProp) {
        <SuperProp as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_cond_expr(&mut self, node: CondExpr) {
        self.enter_node(node.node_id());
        <CondExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_import_expr(&mut self, node: ImportExpr) {
        self.enter_node(node.node_id());
        <ImportExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_call_expr(&mut self, node: CallExpr) {
        self.enter_node(node.node_id());
        <CallExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_new_expr(&mut self, node: NewExpr) {
        self.enter_node(node.node_id());
        <NewExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_seq_expr(&mut self, node: SeqExpr) {
        self.enter_node(node.node_id());
        <SeqExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_arrow_expr(&mut self, node: ArrowExpr) {
        self.enter_node(node.node_id());
        <ArrowExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_yield_expr(&mut self, node: YieldExpr) {
        self.enter_node(node.node_id());
        <YieldExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_meta_prop_expr(&mut self, node: MetaPropExpr) {
        self.enter_node(node.node_id());
        <MetaPropExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_await_expr(&mut self, node: AwaitExpr) {
        self.enter_node(node.node_id());
        <AwaitExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_tpl(&mut self, node: Tpl) {
        self.enter_node(node.node_id());
        <Tpl as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_tagged_tpl(&mut self, node: TaggedTpl) {
        self.enter_node(node.node_id());
        <TaggedTpl as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_tpl_element(&mut self, node: TplElement) {
        self.enter_node(node.node_id());
        <TplElement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_paren_expr(&mut self, node: ParenExpr) {
        self.enter_node(node.node_id());
        <ParenExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_callee(&mut self, node: Callee) {
        <Callee as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_super(&mut self, node: Super) {
        self.enter_node(node.node_id());
        <Super as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_expr_or_spread(&mut self, node: ExprOrSpread) {
        self.enter_node(node.node_id());
        <ExprOrSpread as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_block_stmt_or_expr(&mut self, node: BlockStmtOrExpr) {
        <BlockStmtOrExpr as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_assign_target(&mut self, node: AssignTarget) {
        <AssignTarget as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_assign_target_pat(&mut self, node: AssignTargetPat) {
        <AssignTargetPat as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_simple_assign_target(&mut self, node: SimpleAssignTarget) {
        <SimpleAssignTarget as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_chain_expr(&mut self, node: OptChainExpr) {
        self.enter_node(node.node_id());
        <OptChainExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_opt_chain_base(&mut self, node: OptChainBase) {
        <OptChainBase as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_call(&mut self, node: OptCall) {
        self.enter_node(node.node_id());
        <OptCall as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_invalid(&mut self, node: Invalid) {
        self.enter_node(node.node_id());
        <Invalid as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_function(&mut self, node: Function) {
        self.enter_node(node.node_id());
        <Function as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_param(&mut self, node: Param) {
        self.enter_node(node.node_id());
        <Param as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_param_or_ts_param_prop(&mut self, node: ParamOrTsParamProp) {
        <ParamOrTsParamProp as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_class(&mut self, node: Class) {
        self.enter_node(node.node_id());
        <Class as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_class_member(&mut self, node: ClassMember) {
        <ClassMember as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_class_prop(&mut self, node: ClassProp) {
        self.enter_node(node.node_id());
        <ClassProp as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_private_prop(&mut self, node: PrivateProp) {
        self.enter_node(node.node_id());
        <PrivateProp as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_class_method(&mut self, node: ClassMethod) {
        self.enter_node(node.node_id());
        <ClassMethod as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_private_method(&mut self, node: PrivateMethod) {
        self.enter_node(node.node_id());
        <PrivateMethod as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_constructor(&mut self, node: Constructor) {
        self.enter_node(node.node_id());
        <Constructor as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_decorator(&mut self, node: Decorator) {
        self.enter_node(node.node_id());
        <Decorator as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_static_block(&mut self, node: StaticBlock) {
        self.enter_node(node.node_id());
        <StaticBlock as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_key(&mut self, node: Key) {
        <Key as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_auto_accessor(&mut self, node: AutoAccessor) {
        self.enter_node(node.node_id());
        <AutoAccessor as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_prop(&mut self, node: Prop) {
        <Prop as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_key_value_prop(&mut self, node: KeyValueProp) {
        self.enter_node(node.node_id());
        <KeyValueProp as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_assign_prop(&mut self, node: AssignProp) {
        self.enter_node(node.node_id());
        <AssignProp as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_getter_prop(&mut self, node: GetterProp) {
        self.enter_node(node.node_id());
        <GetterProp as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_setter_prop(&mut self, node: SetterProp) {
        self.enter_node(node.node_id());
        <SetterProp as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_method_prop(&mut self, node: MethodProp) {
        self.enter_node(node.node_id());
        <MethodProp as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_prop_name(&mut self, node: PropName) {
        <PropName as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_computed_prop_name(&mut self, node: ComputedPropName) {
        self.enter_node(node.node_id());
        <ComputedPropName as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_pat(&mut self, node: Pat) {
        <Pat as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_array_pat(&mut self, node: ArrayPat) {
        self.enter_node(node.node_id());
        <ArrayPat as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_object_pat(&mut self, node: ObjectPat) {
        self.enter_node(node.node_id());
        <ObjectPat as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_assign_pat(&mut self, node: AssignPat) {
        self.enter_node(node.node_id());
        <AssignPat as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_rest_pat(&mut self, node: RestPat) {
        self.enter_node(node.node_id());
        <RestPat as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_object_pat_prop(&mut self, node: ObjectPatProp) {
        <ObjectPatProp as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_key_value_pat_prop(&mut self, node: KeyValuePatProp) {
        self.enter_node(node.node_id());
        <KeyValuePatProp as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_assign_pat_prop(&mut self, node: AssignPatProp) {
        self.enter_node(node.node_id());
        <AssignPatProp as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ident(&mut self, node: Ident) {
        self.enter_node(node.node_id());
        <Ident as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_ident_name(&mut self, node: IdentName) {
        self.enter_node(node.node_id());
        <IdentName as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_private_name(&mut self, node: PrivateName) {
        self.enter_node(node.node_id());
        <PrivateName as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_binding_ident(&mut self, node: BindingIdent) {
        self.enter_node(node.node_id());
        <BindingIdent as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_lit(&mut self, node: Lit) {
        <Lit as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_str(&mut self, node: Str) {
        self.enter_node(node.node_id());
        <Str as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_bool(&mut self, node: Bool) {
        self.enter_node(node.node_id());
        <Bool as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_null(&mut self, node: Null) {
        self.enter_node(node.node_id());
        <Null as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_number(&mut self, node: Number) {
        self.enter_node(node.node_id());
        <Number as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_big_int(&mut self, node: BigInt) {
        self.enter_node(node.node_id());
        <BigInt as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_regex(&mut self, node: Regex) {
        self.enter_node(node.node_id());
        <Regex as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_object(&mut self, node: JSXObject) {
        <JSXObject as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_member_expr(&mut self, node: JSXMemberExpr) {
        self.enter_node(node.node_id());
        <JSXMemberExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_namespaced_name(&mut self, node: JSXNamespacedName) {
        self.enter_node(node.node_id());
        <JSXNamespacedName as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_empty_expr(&mut self, node: JSXEmptyExpr) {
        self.enter_node(node.node_id());
        <JSXEmptyExpr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_expr_container(&mut self, node: JSXExprContainer) {
        self.enter_node(node.node_id());
        <JSXExprContainer as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_expr(&mut self, node: JSXExpr) {
        <JSXExpr as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_spread_child(&mut self, node: JSXSpreadChild) {
        self.enter_node(node.node_id());
        <JSXSpreadChild as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_element_name(&mut self, node: JSXElementName) {
        <JSXElementName as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_opening_element(&mut self, node: JSXOpeningElement) {
        self.enter_node(node.node_id());
        <JSXOpeningElement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_attr_or_spread(&mut self, node: JSXAttrOrSpread) {
        <JSXAttrOrSpread as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_closing_element(&mut self, node: JSXClosingElement) {
        self.enter_node(node.node_id());
        <JSXClosingElement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_attr(&mut self, node: JSXAttr) {
        self.enter_node(node.node_id());
        <JSXAttr as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_attr_name(&mut self, node: JSXAttrName) {
        <JSXAttrName as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_attr_value(&mut self, node: JSXAttrValue) {
        <JSXAttrValue as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_text(&mut self, node: JSXText) {
        self.enter_node(node.node_id());
        <JSXText as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_element(&mut self, node: JSXElement) {
        self.enter_node(node.node_id());
        <JSXElement as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_element_child(&mut self, node: JSXElementChild) {
        <JSXElementChild as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_fragment(&mut self, node: JSXFragment) {
        self.enter_node(node.node_id());
        <JSXFragment as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_opening_fragment(&mut self, node: JSXOpeningFragment) {
        self.enter_node(node.node_id());
        <JSXOpeningFragment as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_jsx_closing_fragment(&mut self, node: JSXClosingFragment) {
        self.enter_node(node.node_id());
        <JSXClosingFragment as VisitWith<'a, Self>>::visit_children_with(node, self);
        self.leave_node(node.node_id());
    }
    #[inline]
    fn visit_module_items(&mut self, node: TypedSubRange<ModuleItem>) {
        <TypedSubRange<ModuleItem> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_stmts(&mut self, node: TypedSubRange<Stmt>) {
        <TypedSubRange<Stmt> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_import_specifiers(&mut self, node: TypedSubRange<ImportSpecifier>) {
        <TypedSubRange<ImportSpecifier> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_object_lit(&mut self, node: Option<ObjectLit>) {
        <Option<ObjectLit> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_export_specifiers(&mut self, node: TypedSubRange<ExportSpecifier>) {
        <TypedSubRange<ExportSpecifier> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_str(&mut self, node: Option<Str>) {
        <Option<Str> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_expr(&mut self, node: Option<Expr>) {
        <Option<Expr> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_ident(&mut self, node: Option<Ident>) {
        <Option<Ident> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_stmt(&mut self, node: Option<Stmt>) {
        <Option<Stmt> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_switch_cases(&mut self, node: TypedSubRange<SwitchCase>) {
        <TypedSubRange<SwitchCase> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_catch_clause(&mut self, node: Option<CatchClause>) {
        <Option<CatchClause> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_block_stmt(&mut self, node: Option<BlockStmt>) {
        <Option<BlockStmt> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_var_decl_or_expr(&mut self, node: Option<VarDeclOrExpr>) {
        <Option<VarDeclOrExpr> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_pat(&mut self, node: Option<Pat>) {
        <Option<Pat> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_var_declarators(&mut self, node: TypedSubRange<VarDeclarator>) {
        <TypedSubRange<VarDeclarator> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_expr_or_spread(&mut self, node: Option<ExprOrSpread>) {
        <Option<ExprOrSpread> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_opt_vec_expr_or_spreads(&mut self, node: TypedSubRange<Option<ExprOrSpread>>) {
        <TypedSubRange<Option<ExprOrSpread>> as VisitWith<'a, Self>>::visit_children_with(
            node, self,
        )
    }
    #[inline]
    fn visit_prop_or_spreads(&mut self, node: TypedSubRange<PropOrSpread>) {
        <TypedSubRange<PropOrSpread> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_expr_or_spreads(&mut self, node: TypedSubRange<ExprOrSpread>) {
        <TypedSubRange<ExprOrSpread> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_exprs(&mut self, node: TypedSubRange<Expr>) {
        <TypedSubRange<Expr> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_pats(&mut self, node: TypedSubRange<Pat>) {
        <TypedSubRange<Pat> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_tpl_elements(&mut self, node: TypedSubRange<TplElement>) {
        <TypedSubRange<TplElement> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_params(&mut self, node: TypedSubRange<Param>) {
        <TypedSubRange<Param> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_decorators(&mut self, node: TypedSubRange<Decorator>) {
        <TypedSubRange<Decorator> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_class_members(&mut self, node: TypedSubRange<ClassMember>) {
        <TypedSubRange<ClassMember> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_param_or_ts_param_props(&mut self, node: TypedSubRange<ParamOrTsParamProp>) {
        <TypedSubRange<ParamOrTsParamProp> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_vec_pats(&mut self, node: TypedSubRange<Option<Pat>>) {
        <TypedSubRange<Option<Pat>> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_object_pat_props(&mut self, node: TypedSubRange<ObjectPatProp>) {
        <TypedSubRange<ObjectPatProp> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_jsx_attr_or_spreads(&mut self, node: TypedSubRange<JSXAttrOrSpread>) {
        <TypedSubRange<JSXAttrOrSpread> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_jsx_attr_value(&mut self, node: Option<JSXAttrValue>) {
        <Option<JSXAttrValue> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
    #[inline]
    fn visit_jsx_element_childs(&mut self, node: TypedSubRange<JSXElementChild>) {
        <TypedSubRange<JSXElementChild> as VisitWith<'a, Self>>::visit_children_with(node, self)
    }
    #[inline]
    fn visit_opt_jsx_closing_element(&mut self, node: Option<JSXClosingElement>) {
        <Option<JSXClosingElement> as VisitWith<'a, Self>>::visit_children_with(node, self);
    }
}
pub trait VisitWith<'a, V: ?Sized + Visit<'a>> {
    fn visit_with(self, visitor: &mut V);
    fn visit_children_with(self, visitor: &mut V);
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Program {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_program(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().program_data(self) {
            ProgramData::Module(it) => <Module as VisitWith<'a, V>>::visit_with(it, visitor),
            ProgramData::Script(it) => <Script as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Module {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_module(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.body(visitor.ast());
        <TypedSubRange<ModuleItem> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Script {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_script(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.body(visitor.ast());
        <TypedSubRange<Stmt> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ModuleItem {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_module_item(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().module_item_data(self) {
            ModuleItemData::ModuleDecl(it) => {
                <ModuleDecl as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ModuleItemData::Stmt(it) => <Stmt as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ModuleDecl {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_module_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().module_decl_data(self) {
            ModuleDeclData::Import(it) => <ImportDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            ModuleDeclData::ExportDecl(it) => {
                <ExportDecl as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ModuleDeclData::ExportNamed(it) => {
                <NamedExport as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ModuleDeclData::ExportDefaultDecl(it) => {
                <ExportDefaultDecl as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ModuleDeclData::ExportDefaultExpr(it) => {
                <ExportDefaultExpr as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ModuleDeclData::ExportAll(it) => {
                <ExportAll as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportDecl {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.specifiers(visitor.ast());
        let field_value_1 = self.src(visitor.ast());
        let field_value_3 = self.with(visitor.ast());
        <TypedSubRange<ImportSpecifier> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Str as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<ObjectLit> as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportSpecifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().import_specifier_data(self) {
            ImportSpecifierData::Named(it) => {
                <ImportNamedSpecifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ImportSpecifierData::Default(it) => {
                <ImportDefaultSpecifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ImportSpecifierData::Namespace(it) => {
                <ImportStarAsSpecifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportNamedSpecifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_named_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.local(visitor.ast());
        let field_value_1 = self.imported(visitor.ast());
        <Ident as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <ModuleExportName as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportDefaultSpecifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_default_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.local(visitor.ast());
        <Ident as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportStarAsSpecifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_star_as_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.local(visitor.ast());
        <Ident as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportDecl {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.decl(visitor.ast());
        <Decl as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for NamedExport {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_named_export(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.specifiers(visitor.ast());
        let field_value_1 = self.src(visitor.ast());
        let field_value_3 = self.with(visitor.ast());
        <TypedSubRange<ExportSpecifier> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<Str> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<ObjectLit> as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportSpecifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().export_specifier_data(self) {
            ExportSpecifierData::Namespace(it) => {
                <ExportNamespaceSpecifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExportSpecifierData::Default(it) => {
                <ExportDefaultSpecifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExportSpecifierData::Named(it) => {
                <ExportNamedSpecifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportNamespaceSpecifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_namespace_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.name(visitor.ast());
        <ModuleExportName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ModuleExportName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_module_export_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().module_export_name_data(self) {
            ModuleExportNameData::Ident(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            ModuleExportNameData::Str(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportDefaultSpecifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_default_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.exported(visitor.ast());
        <Ident as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportNamedSpecifier {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_named_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.orig(visitor.ast());
        let field_value_1 = self.exported(visitor.ast());
        <ModuleExportName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <ModuleExportName as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportDefaultDecl {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_default_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.decl(visitor.ast());
        <DefaultDecl as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for DefaultDecl {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_default_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().default_decl_data(self) {
            DefaultDeclData::Class(it) => <ClassExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            DefaultDeclData::Fn(it) => <FnExpr as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportDefaultExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_default_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expr(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportAll {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_all(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.src(visitor.ast());
        let field_value_2 = self.with(visitor.ast());
        <Str as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<ObjectLit> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BlockStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_block_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.stmts(visitor.ast());
        <TypedSubRange<Stmt> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Stmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().stmt_data(self) {
            StmtData::Block(it) => <BlockStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::Empty(it) => <EmptyStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::Debugger(it) => <DebuggerStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::With(it) => <WithStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::Return(it) => <ReturnStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::Labeled(it) => <LabeledStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::Break(it) => <BreakStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::Continue(it) => <ContinueStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::If(it) => <IfStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::Switch(it) => <SwitchStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::Throw(it) => <ThrowStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::Try(it) => <TryStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::While(it) => <WhileStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::DoWhile(it) => <DoWhileStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::For(it) => <ForStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::ForIn(it) => <ForInStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::ForOf(it) => <ForOfStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::Decl(it) => <Decl as VisitWith<'a, V>>::visit_with(it, visitor),
            StmtData::Expr(it) => <ExprStmt as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExprStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_expr_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expr(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for EmptyStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_empty_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for DebuggerStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_debugger_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for WithStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_with_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.obj(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ReturnStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_return_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.arg(visitor.ast());
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for LabeledStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_labeled_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.label(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <Ident as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BreakStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_break_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.label(visitor.ast());
        <Option<Ident> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ContinueStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_continue_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.label(visitor.ast());
        <Option<Ident> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for IfStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_if_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.test(visitor.ast());
        let field_value_1 = self.cons(visitor.ast());
        let field_value_2 = self.alt(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<Stmt> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SwitchStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_switch_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.discriminant(visitor.ast());
        let field_value_1 = self.cases(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<SwitchCase> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ThrowStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_throw_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.arg(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TryStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_try_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.block(visitor.ast());
        let field_value_1 = self.handler(visitor.ast());
        let field_value_2 = self.finalizer(visitor.ast());
        <BlockStmt as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<CatchClause> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<BlockStmt> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for WhileStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_while_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.test(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for DoWhileStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_do_while_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.test(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ForStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_for_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.init(visitor.ast());
        let field_value_1 = self.test(visitor.ast());
        let field_value_2 = self.update(visitor.ast());
        let field_value_3 = self.body(visitor.ast());
        <Option<VarDeclOrExpr> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ForInStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_for_in_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.left(visitor.ast());
        let field_value_1 = self.right(visitor.ast());
        let field_value_2 = self.body(visitor.ast());
        <ForHead as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ForOfStmt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_for_of_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.left(visitor.ast());
        let field_value_2 = self.right(visitor.ast());
        let field_value_3 = self.body(visitor.ast());
        <ForHead as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
        <Stmt as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SwitchCase {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_switch_case(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.test(visitor.ast());
        let field_value_1 = self.cons(visitor.ast());
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<Stmt> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for CatchClause {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_catch_clause(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.param(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <Option<Pat> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <BlockStmt as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ForHead {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_for_head(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().for_head_data(self) {
            ForHeadData::VarDecl(it) => <VarDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            ForHeadData::UsingDecl(it) => <UsingDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            ForHeadData::Pat(it) => <Pat as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for VarDeclOrExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_var_decl_or_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().var_decl_or_expr_data(self) {
            VarDeclOrExprData::VarDecl(it) => {
                <VarDecl as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            VarDeclOrExprData::Expr(it) => <Expr as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Decl {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().decl_data(self) {
            DeclData::Class(it) => <ClassDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            DeclData::Fn(it) => <FnDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            DeclData::Var(it) => <VarDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            DeclData::Using(it) => <UsingDecl as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for FnDecl {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_fn_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.ident(visitor.ast());
        let field_value_2 = self.function(visitor.ast());
        <Ident as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Function as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ClassDecl {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.ident(visitor.ast());
        let field_value_2 = self.class(visitor.ast());
        <Ident as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Class as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for VarDecl {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_var_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_2 = self.decls(visitor.ast());
        <TypedSubRange<VarDeclarator> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for VarDeclarator {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_var_declarator(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.name(visitor.ast());
        let field_value_1 = self.init(visitor.ast());
        <Pat as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for UsingDecl {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_using_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.decls(visitor.ast());
        <TypedSubRange<VarDeclarator> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Expr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().expr_data(self) {
            ExprData::This(it) => <ThisExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Array(it) => <ArrayLit as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Object(it) => <ObjectLit as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Fn(it) => <FnExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Unary(it) => <UnaryExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Update(it) => <UpdateExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Bin(it) => <BinExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Assign(it) => <AssignExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Member(it) => <MemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::SuperProp(it) => <SuperPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Cond(it) => <CondExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Import(it) => <ImportExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Call(it) => <CallExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::New(it) => <NewExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Seq(it) => <SeqExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Ident(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Lit(it) => <Lit as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Tpl(it) => <Tpl as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::TaggedTpl(it) => <TaggedTpl as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Arrow(it) => <ArrowExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Class(it) => <ClassExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Yield(it) => <YieldExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::MetaProp(it) => <MetaPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Await(it) => <AwaitExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Paren(it) => <ParenExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::JSXMember(it) => <JSXMemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::JSXNamespacedName(it) => {
                <JSXNamespacedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ExprData::JSXEmpty(it) => <JSXEmptyExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::JSXElement(it) => <JSXElement as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::JSXFragment(it) => <JSXFragment as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::PrivateName(it) => <PrivateName as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::OptChain(it) => <OptChainExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            ExprData::Invalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ThisExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_this_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ArrayLit {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_array_lit(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.elems(visitor.ast());
        <TypedSubRange<Option<ExprOrSpread>> as VisitWith<'a, V>>::visit_with(
            field_value_0,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ObjectLit {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_object_lit(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.props(visitor.ast());
        <TypedSubRange<PropOrSpread> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for PropOrSpread {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_prop_or_spread(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().prop_or_spread_data(self) {
            PropOrSpreadData::Spread(it) => {
                <SpreadElement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            PropOrSpreadData::Prop(it) => <Prop as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SpreadElement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_spread_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.expr(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for UnaryExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_unary_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.arg(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for UpdateExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_update_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_2 = self.arg(visitor.ast());
        <SimpleAssignTarget as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BinExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_bin_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.left(visitor.ast());
        let field_value_2 = self.right(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for FnExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_fn_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.ident(visitor.ast());
        let field_value_1 = self.function(visitor.ast());
        <Option<Ident> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Function as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ClassExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.ident(visitor.ast());
        let field_value_1 = self.class(visitor.ast());
        <Option<Ident> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Class as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assign_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.left(visitor.ast());
        let field_value_2 = self.right(visitor.ast());
        <AssignTarget as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for MemberExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_member_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.obj(visitor.ast());
        let field_value_1 = self.prop(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <MemberProp as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for MemberProp {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_member_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().member_prop_data(self) {
            MemberPropData::Ident(it) => <IdentName as VisitWith<'a, V>>::visit_with(it, visitor),
            MemberPropData::PrivateName(it) => {
                <PrivateName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            MemberPropData::Computed(it) => {
                <ComputedPropName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SuperPropExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_super_prop_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.obj(visitor.ast());
        let field_value_1 = self.prop(visitor.ast());
        <Super as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <SuperProp as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SuperProp {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_super_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().super_prop_data(self) {
            SuperPropData::Ident(it) => <IdentName as VisitWith<'a, V>>::visit_with(it, visitor),
            SuperPropData::Computed(it) => {
                <ComputedPropName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for CondExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_cond_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.test(visitor.ast());
        let field_value_1 = self.cons(visitor.ast());
        let field_value_2 = self.alt(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.source(visitor.ast());
        let field_value_2 = self.options(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for CallExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_call_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.callee(visitor.ast());
        let field_value_1 = self.args(visitor.ast());
        <Callee as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<ExprOrSpread> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for NewExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_new_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.callee(visitor.ast());
        let field_value_1 = self.args(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<ExprOrSpread> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SeqExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_seq_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.exprs(visitor.ast());
        <TypedSubRange<Expr> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ArrowExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_arrow_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.params(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        <TypedSubRange<Pat> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <BlockStmtOrExpr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for YieldExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_yield_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.arg(visitor.ast());
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for MetaPropExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_meta_prop_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AwaitExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_await_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.arg(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Tpl {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_tpl(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.exprs(visitor.ast());
        let field_value_1 = self.quasis(visitor.ast());
        <TypedSubRange<Expr> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<TplElement> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TaggedTpl {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_tagged_tpl(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.tag(visitor.ast());
        let field_value_1 = self.tpl(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Tpl as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TplElement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_tpl_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ParenExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_paren_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expr(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Callee {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_callee(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().callee_data(self) {
            CalleeData::Super(it) => <Super as VisitWith<'a, V>>::visit_with(it, visitor),
            CalleeData::Expr(it) => <Expr as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Super {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_super(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExprOrSpread {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_expr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.expr(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BlockStmtOrExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_block_stmt_or_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().block_stmt_or_expr_data(self) {
            BlockStmtOrExprData::BlockStmt(it) => {
                <BlockStmt as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            BlockStmtOrExprData::Expr(it) => <Expr as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignTarget {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assign_target(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().assign_target_data(self) {
            AssignTargetData::Simple(it) => {
                <SimpleAssignTarget as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            AssignTargetData::Pat(it) => {
                <AssignTargetPat as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignTargetPat {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assign_target_pat(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().assign_target_pat_data(self) {
            AssignTargetPatData::Array(it) => {
                <ArrayPat as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            AssignTargetPatData::Object(it) => {
                <ObjectPat as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            AssignTargetPatData::Invalid(it) => {
                <Invalid as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SimpleAssignTarget {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_simple_assign_target(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().simple_assign_target_data(self) {
            SimpleAssignTargetData::Ident(it) => {
                <BindingIdent as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            SimpleAssignTargetData::Member(it) => {
                <MemberExpr as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            SimpleAssignTargetData::SuperProp(it) => {
                <SuperPropExpr as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            SimpleAssignTargetData::Paren(it) => {
                <ParenExpr as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            SimpleAssignTargetData::OptChain(it) => {
                <OptChainExpr as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            SimpleAssignTargetData::Invalid(it) => {
                <Invalid as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for OptChainExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_chain_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.base(visitor.ast());
        <OptChainBase as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for OptChainBase {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_chain_base(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().opt_chain_base_data(self) {
            OptChainBaseData::Member(it) => {
                <MemberExpr as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            OptChainBaseData::Call(it) => <OptCall as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for OptCall {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_call(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.callee(visitor.ast());
        let field_value_1 = self.args(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<ExprOrSpread> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Invalid {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_invalid(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Function {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_function(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.params(visitor.ast());
        let field_value_1 = self.decorators(visitor.ast());
        let field_value_2 = self.body(visitor.ast());
        <TypedSubRange<Param> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <BlockStmt as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Param {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_param(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.decorators(visitor.ast());
        let field_value_1 = self.pat(visitor.ast());
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Pat as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ParamOrTsParamProp {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_param_or_ts_param_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().param_or_ts_param_prop_data(self) {
            ParamOrTsParamPropData::Param(it) => {
                <Param as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Class {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.decorators(visitor.ast());
        let field_value_1 = self.body(visitor.ast());
        let field_value_2 = self.super_class(visitor.ast());
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<ClassMember> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ClassMember {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_member(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().class_member_data(self) {
            ClassMemberData::Constructor(it) => {
                <Constructor as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ClassMemberData::Method(it) => {
                <ClassMethod as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ClassMemberData::PrivateMethod(it) => {
                <PrivateMethod as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ClassMemberData::ClassProp(it) => {
                <ClassProp as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ClassMemberData::PrivateProp(it) => {
                <PrivateProp as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ClassMemberData::Empty(it) => <EmptyStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            ClassMemberData::StaticBlock(it) => {
                <StaticBlock as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ClassMemberData::AutoAccessor(it) => {
                <AutoAccessor as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ClassProp {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.value(visitor.ast());
        let field_value_3 = self.decorators(visitor.ast());
        <PropName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for PrivateProp {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_private_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.value(visitor.ast());
        let field_value_3 = self.decorators(visitor.ast());
        <PrivateName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ClassMethod {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_method(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.function(visitor.ast());
        <PropName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Function as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for PrivateMethod {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_private_method(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.function(visitor.ast());
        <PrivateName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Function as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Constructor {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_constructor(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.params(visitor.ast());
        let field_value_2 = self.body(visitor.ast());
        <PropName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<ParamOrTsParamProp> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<BlockStmt> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Decorator {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_decorator(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expr(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for StaticBlock {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_static_block(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.body(visitor.ast());
        <BlockStmt as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Key {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_key(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().key_data(self) {
            KeyData::Private(it) => <PrivateName as VisitWith<'a, V>>::visit_with(it, visitor),
            KeyData::Public(it) => <PropName as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AutoAccessor {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_auto_accessor(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.value(visitor.ast());
        let field_value_3 = self.decorators(visitor.ast());
        <Key as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <TypedSubRange<Decorator> as VisitWith<'a, V>>::visit_with(field_value_3, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Prop {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().prop_data(self) {
            PropData::Shorthand(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            PropData::KeyValue(it) => <KeyValueProp as VisitWith<'a, V>>::visit_with(it, visitor),
            PropData::Assign(it) => <AssignProp as VisitWith<'a, V>>::visit_with(it, visitor),
            PropData::Getter(it) => <GetterProp as VisitWith<'a, V>>::visit_with(it, visitor),
            PropData::Setter(it) => <SetterProp as VisitWith<'a, V>>::visit_with(it, visitor),
            PropData::Method(it) => <MethodProp as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for KeyValueProp {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_key_value_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.value(visitor.ast());
        <PropName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignProp {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assign_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.value(visitor.ast());
        <Ident as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for GetterProp {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_getter_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.function(visitor.ast());
        <PropName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Function as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SetterProp {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_setter_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.function(visitor.ast());
        <PropName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Function as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for MethodProp {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_method_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.function(visitor.ast());
        <PropName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Function as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for PropName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_prop_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().prop_name_data(self) {
            PropNameData::Ident(it) => <IdentName as VisitWith<'a, V>>::visit_with(it, visitor),
            PropNameData::Str(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
            PropNameData::Num(it) => <Number as VisitWith<'a, V>>::visit_with(it, visitor),
            PropNameData::Computed(it) => {
                <ComputedPropName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            PropNameData::BigInt(it) => <BigInt as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ComputedPropName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_computed_prop_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expr(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Pat {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_pat(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().pat_data(self) {
            PatData::Ident(it) => <BindingIdent as VisitWith<'a, V>>::visit_with(it, visitor),
            PatData::Array(it) => <ArrayPat as VisitWith<'a, V>>::visit_with(it, visitor),
            PatData::Rest(it) => <RestPat as VisitWith<'a, V>>::visit_with(it, visitor),
            PatData::Object(it) => <ObjectPat as VisitWith<'a, V>>::visit_with(it, visitor),
            PatData::Assign(it) => <AssignPat as VisitWith<'a, V>>::visit_with(it, visitor),
            PatData::Invalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
            PatData::Expr(it) => <Expr as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ArrayPat {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_array_pat(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.elems(visitor.ast());
        <TypedSubRange<Option<Pat>> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ObjectPat {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_object_pat(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.props(visitor.ast());
        <TypedSubRange<ObjectPatProp> as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignPat {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assign_pat(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.left(visitor.ast());
        let field_value_1 = self.right(visitor.ast());
        <Pat as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Expr as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for RestPat {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_rest_pat(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_1 = self.arg(visitor.ast());
        <Pat as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ObjectPatProp {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_object_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().object_pat_prop_data(self) {
            ObjectPatPropData::KeyValue(it) => {
                <KeyValuePatProp as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ObjectPatPropData::Assign(it) => {
                <AssignPatProp as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            ObjectPatPropData::Rest(it) => <RestPat as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for KeyValuePatProp {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_key_value_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.value(visitor.ast());
        <PropName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Pat as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignPatProp {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assign_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.key(visitor.ast());
        let field_value_1 = self.value(visitor.ast());
        <BindingIdent as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<Expr> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Ident {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ident(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for IdentName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ident_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for PrivateName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_private_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BindingIdent {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_binding_ident(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.id(visitor.ast());
        <Ident as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Lit {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_lit(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().lit_data(self) {
            LitData::Str(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
            LitData::Bool(it) => <Bool as VisitWith<'a, V>>::visit_with(it, visitor),
            LitData::Null(it) => <Null as VisitWith<'a, V>>::visit_with(it, visitor),
            LitData::Num(it) => <Number as VisitWith<'a, V>>::visit_with(it, visitor),
            LitData::BigInt(it) => <BigInt as VisitWith<'a, V>>::visit_with(it, visitor),
            LitData::Regex(it) => <Regex as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Str {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_str(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Bool {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_bool(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Null {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_null(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Number {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_number(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BigInt {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_big_int(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Regex {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_regex(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXObject {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_object(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().jsx_object_data(self) {
            JSXObjectData::JSXMemberExpr(it) => {
                <JSXMemberExpr as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JSXObjectData::Ident(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXMemberExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_member_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.obj(visitor.ast());
        let field_value_1 = self.prop(visitor.ast());
        <JSXObject as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <IdentName as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXNamespacedName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_namespaced_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.ns(visitor.ast());
        let field_value_1 = self.name(visitor.ast());
        <IdentName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <IdentName as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXEmptyExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_empty_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXExprContainer {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_expr_container(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expr(visitor.ast());
        <JSXExpr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXExpr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().jsx_expr_data(self) {
            JSXExprData::JSXEmptyExpr(it) => {
                <JSXEmptyExpr as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JSXExprData::Expr(it) => <Expr as VisitWith<'a, V>>::visit_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXSpreadChild {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_spread_child(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.expr(visitor.ast());
        <Expr as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXElementName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_element_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().jsx_element_name_data(self) {
            JSXElementNameData::Ident(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            JSXElementNameData::JSXMemberExpr(it) => {
                <JSXMemberExpr as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JSXElementNameData::JSXNamespacedName(it) => {
                <JSXNamespacedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXOpeningElement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_opening_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.name(visitor.ast());
        let field_value_1 = self.attrs(visitor.ast());
        <JSXElementName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<JSXAttrOrSpread> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXAttrOrSpread {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().jsx_attr_or_spread_data(self) {
            JSXAttrOrSpreadData::JSXAttr(it) => {
                <JSXAttr as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JSXAttrOrSpreadData::SpreadElement(it) => {
                <SpreadElement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXClosingElement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_closing_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.name(visitor.ast());
        <JSXElementName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXAttr {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.name(visitor.ast());
        let field_value_1 = self.value(visitor.ast());
        <JSXAttrName as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <Option<JSXAttrValue> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXAttrName {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attr_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().jsx_attr_name_data(self) {
            JSXAttrNameData::Ident(it) => <IdentName as VisitWith<'a, V>>::visit_with(it, visitor),
            JSXAttrNameData::JSXNamespacedName(it) => {
                <JSXNamespacedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXAttrValue {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attr_value(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().jsx_attr_value_data(self) {
            JSXAttrValueData::Str(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
            JSXAttrValueData::JSXExprContainer(it) => {
                <JSXExprContainer as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JSXAttrValueData::JSXElement(it) => {
                <JSXElement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JSXAttrValueData::JSXFragment(it) => {
                <JSXFragment as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXText {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_text(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXElement {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.opening(visitor.ast());
        let field_value_1 = self.children(visitor.ast());
        let field_value_2 = self.closing(visitor.ast());
        <JSXOpeningElement as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<JSXElementChild> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <Option<JSXClosingElement> as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXElementChild {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_element_child(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match visitor.ast().jsx_element_child_data(self) {
            JSXElementChildData::JSXText(it) => {
                <JSXText as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JSXElementChildData::JSXExprContainer(it) => {
                <JSXExprContainer as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JSXElementChildData::JSXSpreadChild(it) => {
                <JSXSpreadChild as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JSXElementChildData::JSXElement(it) => {
                <JSXElement as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            JSXElementChildData::JSXFragment(it) => {
                <JSXFragment as VisitWith<'a, V>>::visit_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXFragment {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_fragment(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        let field_value_0 = self.opening(visitor.ast());
        let field_value_1 = self.children(visitor.ast());
        let field_value_2 = self.closing(visitor.ast());
        <JSXOpeningFragment as VisitWith<'a, V>>::visit_with(field_value_0, visitor);
        <TypedSubRange<JSXElementChild> as VisitWith<'a, V>>::visit_with(field_value_1, visitor);
        <JSXClosingFragment as VisitWith<'a, V>>::visit_with(field_value_2, visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXOpeningFragment {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_opening_fragment(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXClosingFragment {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_closing_fragment(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<ModuleItem> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_module_items(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<Stmt> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_stmts(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<ImportSpecifier> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_specifiers(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<ObjectLit> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_object_lit(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<ExportSpecifier> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_specifiers(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Str> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_str(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Expr> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Ident> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_ident(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Stmt> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<SwitchCase> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_switch_cases(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<CatchClause> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_catch_clause(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<BlockStmt> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_block_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<VarDeclOrExpr> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_var_decl_or_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Pat> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_pat(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<VarDeclarator> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_var_declarators(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<ExprOrSpread> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_expr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<Option<ExprOrSpread>> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_vec_expr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<PropOrSpread> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_prop_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<ExprOrSpread> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_expr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<Expr> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_exprs(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<Pat> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_pats(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<TplElement> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_tpl_elements(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<Param> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_params(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<Decorator> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_decorators(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<ClassMember> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_members(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<ParamOrTsParamProp> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_param_or_ts_param_props(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<Option<Pat>> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_vec_pats(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<ObjectPatProp> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_object_pat_props(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<JSXAttrOrSpread> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<JSXAttrValue> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_jsx_attr_value(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TypedSubRange<JSXElementChild> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_element_childs(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<JSXClosingElement> {
    #[inline]
    fn visit_with(self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_jsx_closing_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(self, visitor: &mut V) {
        match self {
            Some(it) => it.visit_with(visitor),
            None => {}
        }
    }
}
pub trait VisitMut<'a> {
    fn ast(&mut self) -> &mut Ast<'a>;
    #[inline]
    fn visit_mut_program(&mut self, node: Program) {
        <Program as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_module(&mut self, node: Module) {
        <Module as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_script(&mut self, node: Script) {
        <Script as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_module_item(&mut self, node: ModuleItem) {
        <ModuleItem as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_module_decl(&mut self, node: ModuleDecl) {
        <ModuleDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_decl(&mut self, node: ImportDecl) {
        <ImportDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_specifier(&mut self, node: ImportSpecifier) {
        <ImportSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_named_specifier(&mut self, node: ImportNamedSpecifier) {
        <ImportNamedSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_default_specifier(&mut self, node: ImportDefaultSpecifier) {
        <ImportDefaultSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_star_as_specifier(&mut self, node: ImportStarAsSpecifier) {
        <ImportStarAsSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_decl(&mut self, node: ExportDecl) {
        <ExportDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_named_export(&mut self, node: NamedExport) {
        <NamedExport as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_specifier(&mut self, node: ExportSpecifier) {
        <ExportSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_namespace_specifier(&mut self, node: ExportNamespaceSpecifier) {
        <ExportNamespaceSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_module_export_name(&mut self, node: ModuleExportName) {
        <ModuleExportName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_default_specifier(&mut self, node: ExportDefaultSpecifier) {
        <ExportDefaultSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_named_specifier(&mut self, node: ExportNamedSpecifier) {
        <ExportNamedSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_default_decl(&mut self, node: ExportDefaultDecl) {
        <ExportDefaultDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_default_decl(&mut self, node: DefaultDecl) {
        <DefaultDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_default_expr(&mut self, node: ExportDefaultExpr) {
        <ExportDefaultExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_all(&mut self, node: ExportAll) {
        <ExportAll as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_block_stmt(&mut self, node: BlockStmt) {
        <BlockStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_stmt(&mut self, node: Stmt) {
        <Stmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_expr_stmt(&mut self, node: ExprStmt) {
        <ExprStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_empty_stmt(&mut self, node: EmptyStmt) {
        <EmptyStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_debugger_stmt(&mut self, node: DebuggerStmt) {
        <DebuggerStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_with_stmt(&mut self, node: WithStmt) {
        <WithStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_return_stmt(&mut self, node: ReturnStmt) {
        <ReturnStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_labeled_stmt(&mut self, node: LabeledStmt) {
        <LabeledStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_break_stmt(&mut self, node: BreakStmt) {
        <BreakStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_continue_stmt(&mut self, node: ContinueStmt) {
        <ContinueStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_if_stmt(&mut self, node: IfStmt) {
        <IfStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_switch_stmt(&mut self, node: SwitchStmt) {
        <SwitchStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_throw_stmt(&mut self, node: ThrowStmt) {
        <ThrowStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_try_stmt(&mut self, node: TryStmt) {
        <TryStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_while_stmt(&mut self, node: WhileStmt) {
        <WhileStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_do_while_stmt(&mut self, node: DoWhileStmt) {
        <DoWhileStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_for_stmt(&mut self, node: ForStmt) {
        <ForStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_for_in_stmt(&mut self, node: ForInStmt) {
        <ForInStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_for_of_stmt(&mut self, node: ForOfStmt) {
        <ForOfStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_switch_case(&mut self, node: SwitchCase) {
        <SwitchCase as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_catch_clause(&mut self, node: CatchClause) {
        <CatchClause as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_for_head(&mut self, node: ForHead) {
        <ForHead as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_var_decl_or_expr(&mut self, node: VarDeclOrExpr) {
        <VarDeclOrExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_decl(&mut self, node: Decl) {
        <Decl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_fn_decl(&mut self, node: FnDecl) {
        <FnDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_class_decl(&mut self, node: ClassDecl) {
        <ClassDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_var_decl(&mut self, node: VarDecl) {
        <VarDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_var_declarator(&mut self, node: VarDeclarator) {
        <VarDeclarator as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_using_decl(&mut self, node: UsingDecl) {
        <UsingDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_expr(&mut self, node: Expr) {
        <Expr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_this_expr(&mut self, node: ThisExpr) {
        <ThisExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_array_lit(&mut self, node: ArrayLit) {
        <ArrayLit as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_object_lit(&mut self, node: ObjectLit) {
        <ObjectLit as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_prop_or_spread(&mut self, node: PropOrSpread) {
        <PropOrSpread as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_spread_element(&mut self, node: SpreadElement) {
        <SpreadElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_unary_expr(&mut self, node: UnaryExpr) {
        <UnaryExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_update_expr(&mut self, node: UpdateExpr) {
        <UpdateExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_bin_expr(&mut self, node: BinExpr) {
        <BinExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_fn_expr(&mut self, node: FnExpr) {
        <FnExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_class_expr(&mut self, node: ClassExpr) {
        <ClassExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_assign_expr(&mut self, node: AssignExpr) {
        <AssignExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_member_expr(&mut self, node: MemberExpr) {
        <MemberExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_member_prop(&mut self, node: MemberProp) {
        <MemberProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_super_prop_expr(&mut self, node: SuperPropExpr) {
        <SuperPropExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_super_prop(&mut self, node: SuperProp) {
        <SuperProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_cond_expr(&mut self, node: CondExpr) {
        <CondExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_expr(&mut self, node: ImportExpr) {
        <ImportExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_call_expr(&mut self, node: CallExpr) {
        <CallExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_new_expr(&mut self, node: NewExpr) {
        <NewExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_seq_expr(&mut self, node: SeqExpr) {
        <SeqExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_arrow_expr(&mut self, node: ArrowExpr) {
        <ArrowExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_yield_expr(&mut self, node: YieldExpr) {
        <YieldExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_meta_prop_expr(&mut self, node: MetaPropExpr) {
        <MetaPropExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_await_expr(&mut self, node: AwaitExpr) {
        <AwaitExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_tpl(&mut self, node: Tpl) {
        <Tpl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_tagged_tpl(&mut self, node: TaggedTpl) {
        <TaggedTpl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_tpl_element(&mut self, node: TplElement) {
        <TplElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_paren_expr(&mut self, node: ParenExpr) {
        <ParenExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_callee(&mut self, node: Callee) {
        <Callee as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_super(&mut self, node: Super) {
        <Super as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_expr_or_spread(&mut self, node: ExprOrSpread) {
        <ExprOrSpread as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_block_stmt_or_expr(&mut self, node: BlockStmtOrExpr) {
        <BlockStmtOrExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_assign_target(&mut self, node: AssignTarget) {
        <AssignTarget as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_assign_target_pat(&mut self, node: AssignTargetPat) {
        <AssignTargetPat as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_simple_assign_target(&mut self, node: SimpleAssignTarget) {
        <SimpleAssignTarget as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_chain_expr(&mut self, node: OptChainExpr) {
        <OptChainExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_chain_base(&mut self, node: OptChainBase) {
        <OptChainBase as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_call(&mut self, node: OptCall) {
        <OptCall as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_invalid(&mut self, node: Invalid) {
        <Invalid as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_function(&mut self, node: Function) {
        <Function as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_param(&mut self, node: Param) {
        <Param as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_param_or_ts_param_prop(&mut self, node: ParamOrTsParamProp) {
        <ParamOrTsParamProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_class(&mut self, node: Class) {
        <Class as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_class_member(&mut self, node: ClassMember) {
        <ClassMember as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_class_prop(&mut self, node: ClassProp) {
        <ClassProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_private_prop(&mut self, node: PrivateProp) {
        <PrivateProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_class_method(&mut self, node: ClassMethod) {
        <ClassMethod as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_private_method(&mut self, node: PrivateMethod) {
        <PrivateMethod as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_constructor(&mut self, node: Constructor) {
        <Constructor as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_decorator(&mut self, node: Decorator) {
        <Decorator as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_static_block(&mut self, node: StaticBlock) {
        <StaticBlock as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_key(&mut self, node: Key) {
        <Key as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_auto_accessor(&mut self, node: AutoAccessor) {
        <AutoAccessor as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_prop(&mut self, node: Prop) {
        <Prop as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_key_value_prop(&mut self, node: KeyValueProp) {
        <KeyValueProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_assign_prop(&mut self, node: AssignProp) {
        <AssignProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_getter_prop(&mut self, node: GetterProp) {
        <GetterProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_setter_prop(&mut self, node: SetterProp) {
        <SetterProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_method_prop(&mut self, node: MethodProp) {
        <MethodProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_prop_name(&mut self, node: PropName) {
        <PropName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_computed_prop_name(&mut self, node: ComputedPropName) {
        <ComputedPropName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_pat(&mut self, node: Pat) {
        <Pat as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_array_pat(&mut self, node: ArrayPat) {
        <ArrayPat as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_object_pat(&mut self, node: ObjectPat) {
        <ObjectPat as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_assign_pat(&mut self, node: AssignPat) {
        <AssignPat as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_rest_pat(&mut self, node: RestPat) {
        <RestPat as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_object_pat_prop(&mut self, node: ObjectPatProp) {
        <ObjectPatProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_key_value_pat_prop(&mut self, node: KeyValuePatProp) {
        <KeyValuePatProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_assign_pat_prop(&mut self, node: AssignPatProp) {
        <AssignPatProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ident(&mut self, node: Ident) {
        <Ident as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_ident_name(&mut self, node: IdentName) {
        <IdentName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_private_name(&mut self, node: PrivateName) {
        <PrivateName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_binding_ident(&mut self, node: BindingIdent) {
        <BindingIdent as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_lit(&mut self, node: Lit) {
        <Lit as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_str(&mut self, node: Str) {
        <Str as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_bool(&mut self, node: Bool) {
        <Bool as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_null(&mut self, node: Null) {
        <Null as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_number(&mut self, node: Number) {
        <Number as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_big_int(&mut self, node: BigInt) {
        <BigInt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_regex(&mut self, node: Regex) {
        <Regex as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_object(&mut self, node: JSXObject) {
        <JSXObject as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_member_expr(&mut self, node: JSXMemberExpr) {
        <JSXMemberExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_namespaced_name(&mut self, node: JSXNamespacedName) {
        <JSXNamespacedName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_empty_expr(&mut self, node: JSXEmptyExpr) {
        <JSXEmptyExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_expr_container(&mut self, node: JSXExprContainer) {
        <JSXExprContainer as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_expr(&mut self, node: JSXExpr) {
        <JSXExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_spread_child(&mut self, node: JSXSpreadChild) {
        <JSXSpreadChild as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_element_name(&mut self, node: JSXElementName) {
        <JSXElementName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_opening_element(&mut self, node: JSXOpeningElement) {
        <JSXOpeningElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_attr_or_spread(&mut self, node: JSXAttrOrSpread) {
        <JSXAttrOrSpread as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_closing_element(&mut self, node: JSXClosingElement) {
        <JSXClosingElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_attr(&mut self, node: JSXAttr) {
        <JSXAttr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_attr_name(&mut self, node: JSXAttrName) {
        <JSXAttrName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_attr_value(&mut self, node: JSXAttrValue) {
        <JSXAttrValue as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_text(&mut self, node: JSXText) {
        <JSXText as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_element(&mut self, node: JSXElement) {
        <JSXElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_element_child(&mut self, node: JSXElementChild) {
        <JSXElementChild as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_fragment(&mut self, node: JSXFragment) {
        <JSXFragment as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_opening_fragment(&mut self, node: JSXOpeningFragment) {
        <JSXOpeningFragment as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_closing_fragment(&mut self, node: JSXClosingFragment) {
        <JSXClosingFragment as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_module_items(&mut self, node: TypedSubRange<ModuleItem>) {
        <TypedSubRange<ModuleItem> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_stmts(&mut self, node: TypedSubRange<Stmt>) {
        <TypedSubRange<Stmt> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_import_specifiers(&mut self, node: TypedSubRange<ImportSpecifier>) {
        <TypedSubRange<ImportSpecifier> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_object_lit(&mut self, node: Option<ObjectLit>) {
        <Option<ObjectLit> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_specifiers(&mut self, node: TypedSubRange<ExportSpecifier>) {
        <TypedSubRange<ExportSpecifier> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_str(&mut self, node: Option<Str>) {
        <Option<Str> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_expr(&mut self, node: Option<Expr>) {
        <Option<Expr> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_ident(&mut self, node: Option<Ident>) {
        <Option<Ident> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_stmt(&mut self, node: Option<Stmt>) {
        <Option<Stmt> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_switch_cases(&mut self, node: TypedSubRange<SwitchCase>) {
        <TypedSubRange<SwitchCase> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_opt_catch_clause(&mut self, node: Option<CatchClause>) {
        <Option<CatchClause> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_block_stmt(&mut self, node: Option<BlockStmt>) {
        <Option<BlockStmt> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_var_decl_or_expr(&mut self, node: Option<VarDeclOrExpr>) {
        <Option<VarDeclOrExpr> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_pat(&mut self, node: Option<Pat>) {
        <Option<Pat> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_var_declarators(&mut self, node: TypedSubRange<VarDeclarator>) {
        <TypedSubRange<VarDeclarator> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_expr_or_spread(&mut self, node: Option<ExprOrSpread>) {
        <Option<ExprOrSpread> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_vec_expr_or_spreads(&mut self, node: TypedSubRange<Option<ExprOrSpread>>) {
        <TypedSubRange<Option<ExprOrSpread>> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_prop_or_spreads(&mut self, node: TypedSubRange<PropOrSpread>) {
        <TypedSubRange<PropOrSpread> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_expr_or_spreads(&mut self, node: TypedSubRange<ExprOrSpread>) {
        <TypedSubRange<ExprOrSpread> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_exprs(&mut self, node: TypedSubRange<Expr>) {
        <TypedSubRange<Expr> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_pats(&mut self, node: TypedSubRange<Pat>) {
        <TypedSubRange<Pat> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_tpl_elements(&mut self, node: TypedSubRange<TplElement>) {
        <TypedSubRange<TplElement> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_params(&mut self, node: TypedSubRange<Param>) {
        <TypedSubRange<Param> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_decorators(&mut self, node: TypedSubRange<Decorator>) {
        <TypedSubRange<Decorator> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_class_members(&mut self, node: TypedSubRange<ClassMember>) {
        <TypedSubRange<ClassMember> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_param_or_ts_param_props(&mut self, node: TypedSubRange<ParamOrTsParamProp>) {
        <TypedSubRange<ParamOrTsParamProp> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_vec_pats(&mut self, node: TypedSubRange<Option<Pat>>) {
        <TypedSubRange<Option<Pat>> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
    }
    #[inline]
    fn visit_mut_object_pat_props(&mut self, node: TypedSubRange<ObjectPatProp>) {
        <TypedSubRange<ObjectPatProp> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_jsx_attr_or_spreads(&mut self, node: TypedSubRange<JSXAttrOrSpread>) {
        <TypedSubRange<JSXAttrOrSpread> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_jsx_attr_value(&mut self, node: Option<JSXAttrValue>) {
        <Option<JSXAttrValue> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_element_childs(&mut self, node: TypedSubRange<JSXElementChild>) {
        <TypedSubRange<JSXElementChild> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
    }
    #[inline]
    fn visit_mut_opt_jsx_closing_element(&mut self, node: Option<JSXClosingElement>) {
        <Option<JSXClosingElement> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
}
pub trait VisitMutWith<'a, V: ?Sized + VisitMut<'a>> {
    fn visit_mut_with(self, visitor: &mut V);
    fn visit_mut_children_with(self, visitor: &mut V);
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Program {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_program(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().program_data(self);
        match data {
            ProgramData::Module(it) => <Module as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ProgramData::Script(it) => <Script as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Module {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_module(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.body(visitor.ast());
        <TypedSubRange<ModuleItem> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Script {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_script(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.body(visitor.ast());
        <TypedSubRange<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ModuleItem {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_module_item(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().module_item_data(self);
        match data {
            ModuleItemData::ModuleDecl(it) => {
                <ModuleDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ModuleItemData::Stmt(it) => <Stmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ModuleDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_module_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().module_decl_data(self);
        match data {
            ModuleDeclData::Import(it) => {
                <ImportDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ModuleDeclData::ExportDecl(it) => {
                <ExportDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ModuleDeclData::ExportNamed(it) => {
                <NamedExport as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ModuleDeclData::ExportDefaultDecl(it) => {
                <ExportDefaultDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ModuleDeclData::ExportDefaultExpr(it) => {
                <ExportDefaultExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ModuleDeclData::ExportAll(it) => {
                <ExportAll as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.specifiers(visitor.ast());
        <TypedSubRange<ImportSpecifier> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.src(visitor.ast());
        <Str as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.with(visitor.ast());
        <Option<ObjectLit> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().import_specifier_data(self);
        match data {
            ImportSpecifierData::Named(it) => {
                <ImportNamedSpecifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ImportSpecifierData::Default(it) => {
                <ImportDefaultSpecifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ImportSpecifierData::Namespace(it) => {
                <ImportStarAsSpecifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportNamedSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_named_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.local(visitor.ast());
        <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.imported(visitor.ast());
        <ModuleExportName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportDefaultSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_default_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.local(visitor.ast());
        <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportStarAsSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_star_as_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.local(visitor.ast());
        <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.decl(visitor.ast());
        <Decl as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for NamedExport {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_named_export(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.specifiers(visitor.ast());
        <TypedSubRange<ExportSpecifier> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.src(visitor.ast());
        <Option<Str> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.with(visitor.ast());
        <Option<ObjectLit> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().export_specifier_data(self);
        match data {
            ExportSpecifierData::Namespace(it) => {
                <ExportNamespaceSpecifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExportSpecifierData::Default(it) => {
                <ExportDefaultSpecifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExportSpecifierData::Named(it) => {
                <ExportNamedSpecifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportNamespaceSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_namespace_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <ModuleExportName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ModuleExportName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_module_export_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().module_export_name_data(self);
        match data {
            ModuleExportNameData::Ident(it) => {
                <Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ModuleExportNameData::Str(it) => {
                <Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportDefaultSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_default_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.exported(visitor.ast());
        <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportNamedSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_named_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.orig(visitor.ast());
        <ModuleExportName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.exported(visitor.ast());
        <ModuleExportName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportDefaultDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_default_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.decl(visitor.ast());
        <DefaultDecl as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for DefaultDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_default_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().default_decl_data(self);
        match data {
            DefaultDeclData::Class(it) => {
                <ClassExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            DefaultDeclData::Fn(it) => <FnExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportDefaultExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_default_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportAll {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_all(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.src(visitor.ast());
        <Str as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.with(visitor.ast());
        <Option<ObjectLit> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BlockStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_block_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.stmts(visitor.ast());
        <TypedSubRange<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Stmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().stmt_data(self);
        match data {
            StmtData::Block(it) => <BlockStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            StmtData::Empty(it) => <EmptyStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            StmtData::Debugger(it) => {
                <DebuggerStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::With(it) => <WithStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            StmtData::Return(it) => {
                <ReturnStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::Labeled(it) => {
                <LabeledStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::Break(it) => <BreakStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            StmtData::Continue(it) => {
                <ContinueStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::If(it) => <IfStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            StmtData::Switch(it) => {
                <SwitchStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::Throw(it) => <ThrowStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            StmtData::Try(it) => <TryStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            StmtData::While(it) => <WhileStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            StmtData::DoWhile(it) => {
                <DoWhileStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            StmtData::For(it) => <ForStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            StmtData::ForIn(it) => <ForInStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            StmtData::ForOf(it) => <ForOfStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            StmtData::Decl(it) => <Decl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            StmtData::Expr(it) => <ExprStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExprStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_expr_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for EmptyStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_empty_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for DebuggerStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_debugger_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for WithStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_with_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.obj(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ReturnStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_return_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.arg(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for LabeledStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_labeled_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.label(visitor.ast());
        <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BreakStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_break_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.label(visitor.ast());
        <Option<Ident> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ContinueStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_continue_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.label(visitor.ast());
        <Option<Ident> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for IfStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_if_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.test(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.cons(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.alt(visitor.ast());
        <Option<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SwitchStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_switch_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.discriminant(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.cases(visitor.ast());
        <TypedSubRange<SwitchCase> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ThrowStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_throw_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.arg(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TryStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_try_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.block(visitor.ast());
        <BlockStmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.handler(visitor.ast());
        <Option<CatchClause> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.finalizer(visitor.ast());
        <Option<BlockStmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for WhileStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_while_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.test(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for DoWhileStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_do_while_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.test(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_for_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.init(visitor.ast());
        <Option<VarDeclOrExpr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.test(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.update(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForInStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_for_in_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.left(visitor.ast());
        <ForHead as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForOfStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_for_of_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.left(visitor.ast());
        <ForHead as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SwitchCase {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_switch_case(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.test(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.cons(visitor.ast());
        <TypedSubRange<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for CatchClause {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_catch_clause(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.param(visitor.ast());
        <Option<Pat> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <BlockStmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForHead {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_for_head(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().for_head_data(self);
        match data {
            ForHeadData::VarDecl(it) => {
                <VarDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ForHeadData::UsingDecl(it) => {
                <UsingDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ForHeadData::Pat(it) => <Pat as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for VarDeclOrExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_var_decl_or_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().var_decl_or_expr_data(self);
        match data {
            VarDeclOrExprData::VarDecl(it) => {
                <VarDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            VarDeclOrExprData::Expr(it) => {
                <Expr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Decl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().decl_data(self);
        match data {
            DeclData::Class(it) => <ClassDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            DeclData::Fn(it) => <FnDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            DeclData::Var(it) => <VarDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            DeclData::Using(it) => <UsingDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for FnDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_fn_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.ident(visitor.ast());
        <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.function(visitor.ast());
        <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.ident(visitor.ast());
        <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.class(visitor.ast());
        <Class as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for VarDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_var_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.decls(visitor.ast());
        <TypedSubRange<VarDeclarator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for VarDeclarator {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_var_declarator(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <Pat as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.init(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for UsingDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_using_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.decls(visitor.ast());
        <TypedSubRange<VarDeclarator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Expr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().expr_data(self);
        match data {
            ExprData::This(it) => <ThisExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::Array(it) => <ArrayLit as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::Object(it) => <ObjectLit as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::Fn(it) => <FnExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::Unary(it) => <UnaryExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::Update(it) => {
                <UpdateExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::Bin(it) => <BinExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::Assign(it) => {
                <AssignExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::Member(it) => {
                <MemberExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::SuperProp(it) => {
                <SuperPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::Cond(it) => <CondExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::Import(it) => {
                <ImportExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::Call(it) => <CallExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::New(it) => <NewExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::Seq(it) => <SeqExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::Ident(it) => <Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::Lit(it) => <Lit as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::Tpl(it) => <Tpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::TaggedTpl(it) => {
                <TaggedTpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::Arrow(it) => <ArrowExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::Class(it) => <ClassExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::Yield(it) => <YieldExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::MetaProp(it) => {
                <MetaPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::Await(it) => <AwaitExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::Paren(it) => <ParenExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ExprData::JSXMember(it) => {
                <JSXMemberExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::JSXNamespacedName(it) => {
                <JSXNamespacedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::JSXEmpty(it) => {
                <JSXEmptyExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::JSXElement(it) => {
                <JSXElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::JSXFragment(it) => {
                <JSXFragment as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::PrivateName(it) => {
                <PrivateName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::OptChain(it) => {
                <OptChainExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ExprData::Invalid(it) => <Invalid as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ThisExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_this_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ArrayLit {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_array_lit(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.elems(visitor.ast());
        <TypedSubRange<Option<ExprOrSpread>> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ObjectLit {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_object_lit(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.props(visitor.ast());
        <TypedSubRange<PropOrSpread> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PropOrSpread {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_prop_or_spread(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().prop_or_spread_data(self);
        match data {
            PropOrSpreadData::Spread(it) => {
                <SpreadElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            PropOrSpreadData::Prop(it) => {
                <Prop as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SpreadElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_spread_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for UnaryExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_unary_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.arg(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for UpdateExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_update_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.arg(visitor.ast());
        <SimpleAssignTarget as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BinExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_bin_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.left(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for FnExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_fn_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.ident(visitor.ast());
        <Option<Ident> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.function(visitor.ast());
        <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.ident(visitor.ast());
        <Option<Ident> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.class(visitor.ast());
        <Class as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assign_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.left(visitor.ast());
        <AssignTarget as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MemberExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_member_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.obj(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.prop(visitor.ast());
        <MemberProp as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MemberProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_member_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().member_prop_data(self);
        match data {
            MemberPropData::Ident(it) => {
                <IdentName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            MemberPropData::PrivateName(it) => {
                <PrivateName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            MemberPropData::Computed(it) => {
                <ComputedPropName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SuperPropExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_super_prop_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.obj(visitor.ast());
        <Super as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.prop(visitor.ast());
        <SuperProp as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SuperProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_super_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().super_prop_data(self);
        match data {
            SuperPropData::Ident(it) => {
                <IdentName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            SuperPropData::Computed(it) => {
                <ComputedPropName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for CondExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_cond_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.test(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.cons(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.alt(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.source(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.options(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for CallExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_call_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.callee(visitor.ast());
        <Callee as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.args(visitor.ast());
        <TypedSubRange<ExprOrSpread> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for NewExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_new_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.callee(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.args(visitor.ast());
        <TypedSubRange<ExprOrSpread> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SeqExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_seq_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.exprs(visitor.ast());
        <TypedSubRange<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ArrowExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_arrow_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.params(visitor.ast());
        <TypedSubRange<Pat> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <BlockStmtOrExpr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for YieldExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_yield_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.arg(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MetaPropExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_meta_prop_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AwaitExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_await_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.arg(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Tpl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_tpl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.exprs(visitor.ast());
        <TypedSubRange<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.quasis(visitor.ast());
        <TypedSubRange<TplElement> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TaggedTpl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_tagged_tpl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.tag(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.tpl(visitor.ast());
        <Tpl as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TplElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_tpl_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ParenExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_paren_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Callee {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_callee(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().callee_data(self);
        match data {
            CalleeData::Super(it) => <Super as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            CalleeData::Expr(it) => <Expr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Super {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_super(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExprOrSpread {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_expr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BlockStmtOrExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_block_stmt_or_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().block_stmt_or_expr_data(self);
        match data {
            BlockStmtOrExprData::BlockStmt(it) => {
                <BlockStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            BlockStmtOrExprData::Expr(it) => {
                <Expr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignTarget {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assign_target(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().assign_target_data(self);
        match data {
            AssignTargetData::Simple(it) => {
                <SimpleAssignTarget as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            AssignTargetData::Pat(it) => {
                <AssignTargetPat as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignTargetPat {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assign_target_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().assign_target_pat_data(self);
        match data {
            AssignTargetPatData::Array(it) => {
                <ArrayPat as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            AssignTargetPatData::Object(it) => {
                <ObjectPat as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            AssignTargetPatData::Invalid(it) => {
                <Invalid as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SimpleAssignTarget {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_simple_assign_target(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().simple_assign_target_data(self);
        match data {
            SimpleAssignTargetData::Ident(it) => {
                <BindingIdent as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            SimpleAssignTargetData::Member(it) => {
                <MemberExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            SimpleAssignTargetData::SuperProp(it) => {
                <SuperPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            SimpleAssignTargetData::Paren(it) => {
                <ParenExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            SimpleAssignTargetData::OptChain(it) => {
                <OptChainExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            SimpleAssignTargetData::Invalid(it) => {
                <Invalid as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for OptChainExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_chain_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.base(visitor.ast());
        <OptChainBase as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for OptChainBase {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_chain_base(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().opt_chain_base_data(self);
        match data {
            OptChainBaseData::Member(it) => {
                <MemberExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            OptChainBaseData::Call(it) => {
                <OptCall as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for OptCall {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_call(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.callee(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.args(visitor.ast());
        <TypedSubRange<ExprOrSpread> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Invalid {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_invalid(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Function {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_function(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.params(visitor.ast());
        <TypedSubRange<Param> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <BlockStmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Param {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_param(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.pat(visitor.ast());
        <Pat as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ParamOrTsParamProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_param_or_ts_param_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().param_or_ts_param_prop_data(self);
        match data {
            ParamOrTsParamPropData::Param(it) => {
                <Param as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Class {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.body(visitor.ast());
        <TypedSubRange<ClassMember> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.super_class(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassMember {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_member(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().class_member_data(self);
        match data {
            ClassMemberData::Constructor(it) => {
                <Constructor as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ClassMemberData::Method(it) => {
                <ClassMethod as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ClassMemberData::PrivateMethod(it) => {
                <PrivateMethod as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ClassMemberData::ClassProp(it) => {
                <ClassProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ClassMemberData::PrivateProp(it) => {
                <PrivateProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ClassMemberData::Empty(it) => {
                <EmptyStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ClassMemberData::StaticBlock(it) => {
                <StaticBlock as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ClassMemberData::AutoAccessor(it) => {
                <AutoAccessor as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PrivateProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_private_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PrivateName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassMethod {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_method(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.function(visitor.ast());
        <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PrivateMethod {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_private_method(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PrivateName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.function(visitor.ast());
        <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Constructor {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_constructor(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.params(visitor.ast());
        <TypedSubRange<ParamOrTsParamProp> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.body(visitor.ast());
        <Option<BlockStmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Decorator {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_decorator(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for StaticBlock {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_static_block(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.body(visitor.ast());
        <BlockStmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Key {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_key(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().key_data(self);
        match data {
            KeyData::Private(it) => {
                <PrivateName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            KeyData::Public(it) => <PropName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AutoAccessor {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_auto_accessor(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <Key as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.decorators(visitor.ast());
        <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Prop {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().prop_data(self);
        match data {
            PropData::Shorthand(it) => <Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            PropData::KeyValue(it) => {
                <KeyValueProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            PropData::Assign(it) => {
                <AssignProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            PropData::Getter(it) => {
                <GetterProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            PropData::Setter(it) => {
                <SetterProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            PropData::Method(it) => {
                <MethodProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for KeyValueProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_key_value_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assign_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for GetterProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_getter_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.function(visitor.ast());
        <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SetterProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_setter_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.function(visitor.ast());
        <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MethodProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_method_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.function(visitor.ast());
        <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PropName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_prop_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().prop_name_data(self);
        match data {
            PropNameData::Ident(it) => {
                <IdentName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            PropNameData::Str(it) => <Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            PropNameData::Num(it) => <Number as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            PropNameData::Computed(it) => {
                <ComputedPropName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            PropNameData::BigInt(it) => {
                <BigInt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ComputedPropName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_computed_prop_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Pat {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().pat_data(self);
        match data {
            PatData::Ident(it) => {
                <BindingIdent as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            PatData::Array(it) => <ArrayPat as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            PatData::Rest(it) => <RestPat as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            PatData::Object(it) => <ObjectPat as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            PatData::Assign(it) => <AssignPat as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            PatData::Invalid(it) => <Invalid as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            PatData::Expr(it) => <Expr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ArrayPat {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_array_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.elems(visitor.ast());
        <TypedSubRange<Option<Pat>> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ObjectPat {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_object_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.props(visitor.ast());
        <TypedSubRange<ObjectPatProp> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignPat {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assign_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.left(visitor.ast());
        <Pat as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.right(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for RestPat {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_rest_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.arg(visitor.ast());
        <Pat as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ObjectPatProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_object_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().object_pat_prop_data(self);
        match data {
            ObjectPatPropData::KeyValue(it) => {
                <KeyValuePatProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ObjectPatPropData::Assign(it) => {
                <AssignPatProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            ObjectPatPropData::Rest(it) => {
                <RestPat as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for KeyValuePatProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_key_value_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Pat as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignPatProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assign_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.key(visitor.ast());
        <BindingIdent as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Ident {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ident(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for IdentName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ident_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PrivateName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_private_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BindingIdent {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_binding_ident(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.id(visitor.ast());
        <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Lit {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_lit(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().lit_data(self);
        match data {
            LitData::Str(it) => <Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            LitData::Bool(it) => <Bool as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            LitData::Null(it) => <Null as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            LitData::Num(it) => <Number as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            LitData::BigInt(it) => <BigInt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            LitData::Regex(it) => <Regex as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Str {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_str(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Bool {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_bool(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Null {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_null(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Number {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_number(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BigInt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_big_int(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Regex {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_regex(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXObject {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_object(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().jsx_object_data(self);
        match data {
            JSXObjectData::JSXMemberExpr(it) => {
                <JSXMemberExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JSXObjectData::Ident(it) => <Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXMemberExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_member_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.obj(visitor.ast());
        <JSXObject as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.prop(visitor.ast());
        <IdentName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXNamespacedName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_namespaced_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.ns(visitor.ast());
        <IdentName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.name(visitor.ast());
        <IdentName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXEmptyExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_empty_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXExprContainer {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_expr_container(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <JSXExpr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().jsx_expr_data(self);
        match data {
            JSXExprData::JSXEmptyExpr(it) => {
                <JSXEmptyExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JSXExprData::Expr(it) => <Expr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXSpreadChild {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_spread_child(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.expr(visitor.ast());
        <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXElementName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_element_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().jsx_element_name_data(self);
        match data {
            JSXElementNameData::Ident(it) => {
                <Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JSXElementNameData::JSXMemberExpr(it) => {
                <JSXMemberExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JSXElementNameData::JSXNamespacedName(it) => {
                <JSXNamespacedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXOpeningElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_opening_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <JSXElementName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.attrs(visitor.ast());
        <TypedSubRange<JSXAttrOrSpread> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXAttrOrSpread {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().jsx_attr_or_spread_data(self);
        match data {
            JSXAttrOrSpreadData::JSXAttr(it) => {
                <JSXAttr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JSXAttrOrSpreadData::SpreadElement(it) => {
                <SpreadElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXClosingElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_closing_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <JSXElementName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXAttr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.name(visitor.ast());
        <JSXAttrName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.value(visitor.ast());
        <Option<JSXAttrValue> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXAttrName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attr_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().jsx_attr_name_data(self);
        match data {
            JSXAttrNameData::Ident(it) => {
                <IdentName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JSXAttrNameData::JSXNamespacedName(it) => {
                <JSXNamespacedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXAttrValue {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attr_value(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().jsx_attr_value_data(self);
        match data {
            JSXAttrValueData::Str(it) => <Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            JSXAttrValueData::JSXExprContainer(it) => {
                <JSXExprContainer as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JSXAttrValueData::JSXElement(it) => {
                <JSXElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JSXAttrValueData::JSXFragment(it) => {
                <JSXFragment as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXText {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_text(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.opening(visitor.ast());
        <JSXOpeningElement as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.children(visitor.ast());
        <TypedSubRange<JSXElementChild> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.closing(visitor.ast());
        <Option<JSXClosingElement> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXElementChild {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_element_child(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let data = visitor.ast().jsx_element_child_data(self);
        match data {
            JSXElementChildData::JSXText(it) => {
                <JSXText as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JSXElementChildData::JSXExprContainer(it) => {
                <JSXExprContainer as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JSXElementChildData::JSXSpreadChild(it) => {
                <JSXSpreadChild as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JSXElementChildData::JSXElement(it) => {
                <JSXElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
            JSXElementChildData::JSXFragment(it) => {
                <JSXFragment as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXFragment {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_fragment(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        let field_value = self.opening(visitor.ast());
        <JSXOpeningFragment as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        let field_value = self.children(visitor.ast());
        <TypedSubRange<JSXElementChild> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        let field_value = self.closing(visitor.ast());
        <JSXClosingFragment as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXOpeningFragment {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_opening_fragment(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXClosingFragment {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_closing_fragment(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ModuleItem> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_module_items(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Stmt> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_stmts(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ImportSpecifier> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_specifiers(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<ObjectLit> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_object_lit(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ExportSpecifier> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_specifiers(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Str> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_str(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Expr> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Ident> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_ident(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Stmt> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<SwitchCase> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_switch_cases(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<CatchClause> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_catch_clause(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<BlockStmt> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_block_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<VarDeclOrExpr> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_var_decl_or_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Pat> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<VarDeclarator> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_var_declarators(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<ExprOrSpread> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_expr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Option<ExprOrSpread>> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_vec_expr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<PropOrSpread> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_prop_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ExprOrSpread> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_expr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Expr> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_exprs(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Pat> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_pats(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<TplElement> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_tpl_elements(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Param> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_params(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Decorator> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_decorators(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ClassMember> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_members(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ParamOrTsParamProp> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_param_or_ts_param_props(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Option<Pat>> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_vec_pats(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ObjectPatProp> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_object_pat_props(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<JSXAttrOrSpread> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<JSXAttrValue> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_jsx_attr_value(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<JSXElementChild> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_element_childs(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            child.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<JSXClosingElement> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_jsx_closing_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) {
        if let Some(it) = self {
            it.visit_mut_with(visitor);
        }
    }
}
