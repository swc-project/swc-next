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
        match self {
            Self::Module(it) => <Module as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Script(it) => <Script as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::ModuleDeclImport(it) => <ImportDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ModuleDeclExportDecl(it) => {
                <ExportDecl as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::ModuleDeclExportNamed(it) => {
                <NamedExport as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::ModuleDeclExportDefaultDecl(it) => {
                <ExportDefaultDecl as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::ModuleDeclExportDefaultExpr(it) => {
                <ExportDefaultExpr as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::ModuleDeclExportAll(it) => {
                <ExportAll as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::StmtBlock(it) => <BlockStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtEmpty(it) => <EmptyStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtDebugger(it) => <DebuggerStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtWith(it) => <WithStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtReturn(it) => <ReturnStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtLabeled(it) => <LabeledStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtBreak(it) => <BreakStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtContinue(it) => <ContinueStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtIf(it) => <IfStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtSwitch(it) => <SwitchStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtThrow(it) => <ThrowStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtTry(it) => <TryStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtWhile(it) => <WhileStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtDoWhile(it) => <DoWhileStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtFor(it) => <ForStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtForIn(it) => <ForInStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtForOf(it) => <ForOfStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtDeclClass(it) => <ClassDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtDeclFn(it) => <FnDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtDeclVar(it) => <VarDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtDeclUsing(it) => <UsingDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StmtExpr(it) => <ExprStmt as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Import(it) => <ImportDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExportDecl(it) => <ExportDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExportNamed(it) => <NamedExport as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExportDefaultDecl(it) => {
                <ExportDefaultDecl as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::ExportDefaultExpr(it) => {
                <ExportDefaultExpr as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::ExportAll(it) => <ExportAll as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Named(it) => <ImportNamedSpecifier as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Default(it) => {
                <ImportDefaultSpecifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::Namespace(it) => {
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
        match self {
            Self::Namespace(it) => {
                <ExportNamespaceSpecifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::Default(it) => {
                <ExportDefaultSpecifier as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::Named(it) => <ExportNamedSpecifier as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Ident(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Str(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Class(it) => <ClassExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Fn(it) => <FnExpr as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Class(it) => <ClassDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Fn(it) => <FnDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Var(it) => <VarDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Using(it) => <UsingDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Block(it) => <BlockStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Empty(it) => <EmptyStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Debugger(it) => <DebuggerStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::With(it) => <WithStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Return(it) => <ReturnStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Labeled(it) => <LabeledStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Break(it) => <BreakStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Continue(it) => <ContinueStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::If(it) => <IfStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Switch(it) => <SwitchStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Throw(it) => <ThrowStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Try(it) => <TryStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::While(it) => <WhileStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::DoWhile(it) => <DoWhileStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::For(it) => <ForStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ForIn(it) => <ForInStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ForOf(it) => <ForOfStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Expr(it) => <ExprStmt as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::ExprThis(it) => <ThisExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprArray(it) => <ArrayLit as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprObject(it) => <ObjectLit as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprFn(it) => <FnExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprUnary(it) => <UnaryExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprUpdate(it) => <UpdateExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprBin(it) => <BinExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprAssign(it) => <AssignExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprMember(it) => <MemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprSuperProp(it) => <SuperPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprCond(it) => <CondExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprImport(it) => <ImportExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprCall(it) => <CallExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprNew(it) => <NewExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprSeq(it) => <SeqExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprIdent(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprStr(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprBool(it) => <Bool as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprNull(it) => <Null as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprNum(it) => <Number as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprBigInt(it) => <BigInt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprRegex(it) => <Regex as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprTpl(it) => <Tpl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprTaggedTpl(it) => <TaggedTpl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprArrow(it) => <ArrowExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprClass(it) => <ClassExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprYield(it) => <YieldExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprMetaProp(it) => <MetaPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprAwait(it) => <AwaitExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprParen(it) => <ParenExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprJSXMember(it) => <JSXMemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprJSXNamespacedName(it) => {
                <JSXNamespacedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::ExprJSXEmpty(it) => <JSXEmptyExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprJSXElement(it) => <JSXElement as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprJSXFragment(it) => <JSXFragment as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprPrivateName(it) => <PrivateName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprOptChain(it) => <OptChainExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprInvalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Ident(it) => <BindingIdent as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Array(it) => <ArrayPat as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Rest(it) => <RestPat as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Object(it) => <ObjectPat as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Assign(it) => <AssignPat as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Invalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::VarDecl(it) => <VarDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::UsingDecl(it) => <UsingDecl as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::This(it) => <ThisExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Array(it) => <ArrayLit as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Object(it) => <ObjectLit as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Fn(it) => <FnExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Unary(it) => <UnaryExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Update(it) => <UpdateExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Bin(it) => <BinExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Assign(it) => <AssignExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Member(it) => <MemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::SuperProp(it) => <SuperPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Cond(it) => <CondExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Import(it) => <ImportExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Call(it) => <CallExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::New(it) => <NewExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Seq(it) => <SeqExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Ident(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Str(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Bool(it) => <Bool as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Null(it) => <Null as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Num(it) => <Number as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::BigInt(it) => <BigInt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Regex(it) => <Regex as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Tpl(it) => <Tpl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::TaggedTpl(it) => <TaggedTpl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Arrow(it) => <ArrowExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Class(it) => <ClassExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Yield(it) => <YieldExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::MetaProp(it) => <MetaPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Await(it) => <AwaitExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Paren(it) => <ParenExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXMember(it) => <JSXMemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXNamespacedName(it) => {
                <JSXNamespacedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::JSXEmpty(it) => <JSXEmptyExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXElement(it) => <JSXElement as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXFragment(it) => <JSXFragment as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::PrivateName(it) => <PrivateName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::OptChain(it) => <OptChainExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Invalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::VarDecl(it) => <VarDecl as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Class(it) => <ClassDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Fn(it) => <FnDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Var(it) => <VarDecl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Using(it) => <UsingDecl as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::This(it) => <ThisExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Array(it) => <ArrayLit as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Object(it) => <ObjectLit as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Fn(it) => <FnExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Unary(it) => <UnaryExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Update(it) => <UpdateExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Bin(it) => <BinExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Assign(it) => <AssignExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Member(it) => <MemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::SuperProp(it) => <SuperPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Cond(it) => <CondExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Import(it) => <ImportExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Call(it) => <CallExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::New(it) => <NewExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Seq(it) => <SeqExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Ident(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Str(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Bool(it) => <Bool as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Null(it) => <Null as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Num(it) => <Number as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::BigInt(it) => <BigInt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Regex(it) => <Regex as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Tpl(it) => <Tpl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::TaggedTpl(it) => <TaggedTpl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Arrow(it) => <ArrowExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Class(it) => <ClassExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Yield(it) => <YieldExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::MetaProp(it) => <MetaPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Await(it) => <AwaitExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Paren(it) => <ParenExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXMember(it) => <JSXMemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXNamespacedName(it) => {
                <JSXNamespacedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::JSXEmpty(it) => <JSXEmptyExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXElement(it) => <JSXElement as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXFragment(it) => <JSXFragment as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::PrivateName(it) => <PrivateName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::OptChain(it) => <OptChainExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Invalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Shorthand(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::KeyValue(it) => <KeyValueProp as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Assign(it) => <AssignProp as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Getter(it) => <GetterProp as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Setter(it) => <SetterProp as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Method(it) => <MethodProp as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Spread(it) => <SpreadElement as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Ident(it) => <IdentName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::PrivateName(it) => <PrivateName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Computed(it) => <ComputedPropName as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Ident(it) => <IdentName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Computed(it) => <ComputedPropName as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::This(it) => <ThisExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Array(it) => <ArrayLit as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Object(it) => <ObjectLit as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Fn(it) => <FnExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Unary(it) => <UnaryExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Update(it) => <UpdateExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Bin(it) => <BinExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Assign(it) => <AssignExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Member(it) => <MemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::SuperProp(it) => <SuperPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Cond(it) => <CondExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Import(it) => <ImportExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Call(it) => <CallExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::New(it) => <NewExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Seq(it) => <SeqExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Ident(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Str(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Bool(it) => <Bool as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Null(it) => <Null as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Num(it) => <Number as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::BigInt(it) => <BigInt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Regex(it) => <Regex as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Tpl(it) => <Tpl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::TaggedTpl(it) => <TaggedTpl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Arrow(it) => <ArrowExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Class(it) => <ClassExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Yield(it) => <YieldExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::MetaProp(it) => <MetaPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Await(it) => <AwaitExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Paren(it) => <ParenExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXMember(it) => <JSXMemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXNamespacedName(it) => {
                <JSXNamespacedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::JSXEmpty(it) => <JSXEmptyExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXElement(it) => <JSXElement as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXFragment(it) => <JSXFragment as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::PrivateName(it) => <PrivateName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::OptChain(it) => <OptChainExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Invalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Super(it) => <Super as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::This(it) => <ThisExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Array(it) => <ArrayLit as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Object(it) => <ObjectLit as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Fn(it) => <FnExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Unary(it) => <UnaryExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Update(it) => <UpdateExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Bin(it) => <BinExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Assign(it) => <AssignExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Member(it) => <MemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::SuperProp(it) => <SuperPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Cond(it) => <CondExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Import(it) => <ImportExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Call(it) => <CallExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::New(it) => <NewExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Seq(it) => <SeqExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Ident(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Str(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Bool(it) => <Bool as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Null(it) => <Null as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Num(it) => <Number as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::BigInt(it) => <BigInt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Regex(it) => <Regex as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Tpl(it) => <Tpl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::TaggedTpl(it) => <TaggedTpl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Arrow(it) => <ArrowExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Class(it) => <ClassExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Yield(it) => <YieldExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::MetaProp(it) => <MetaPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Await(it) => <AwaitExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Paren(it) => <ParenExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXMember(it) => <JSXMemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXNamespacedName(it) => {
                <JSXNamespacedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::JSXEmpty(it) => <JSXEmptyExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXElement(it) => <JSXElement as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXFragment(it) => <JSXFragment as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::PrivateName(it) => <PrivateName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::OptChain(it) => <OptChainExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Invalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::BlockStmt(it) => <BlockStmt as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::SimpleIdent(it) => <BindingIdent as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::SimpleMember(it) => <MemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::SimpleSuperProp(it) => {
                <SuperPropExpr as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::SimpleParen(it) => <ParenExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::SimpleOptChain(it) => <OptChainExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::SimpleInvalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::PatArray(it) => <ArrayPat as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::PatObject(it) => <ObjectPat as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::PatternInvalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Array(it) => <ArrayPat as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Object(it) => <ObjectPat as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Invalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Ident(it) => <BindingIdent as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Member(it) => <MemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::SuperProp(it) => <SuperPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Paren(it) => <ParenExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::OptChain(it) => <OptChainExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Invalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Member(it) => <MemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Call(it) => <OptCall as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Param(it) => <Param as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Constructor(it) => <Constructor as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Method(it) => <ClassMethod as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::PrivateMethod(it) => <PrivateMethod as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ClassProp(it) => <ClassProp as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::PrivateProp(it) => <PrivateProp as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Empty(it) => <EmptyStmt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::StaticBlock(it) => <StaticBlock as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::AutoAccessor(it) => <AutoAccessor as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Ident(it) => <IdentName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Str(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Num(it) => <Number as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Computed(it) => <ComputedPropName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::BigInt(it) => <BigInt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Private(it) => <PrivateName as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Shorthand(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::KeyValue(it) => <KeyValueProp as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Assign(it) => <AssignProp as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Getter(it) => <GetterProp as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Setter(it) => <SetterProp as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Method(it) => <MethodProp as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Ident(it) => <IdentName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Str(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Num(it) => <Number as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Computed(it) => <ComputedPropName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::BigInt(it) => <BigInt as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::ExprThis(it) => <ThisExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprArray(it) => <ArrayLit as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprObject(it) => <ObjectLit as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprFn(it) => <FnExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprUnary(it) => <UnaryExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprUpdate(it) => <UpdateExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprBin(it) => <BinExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprAssign(it) => <AssignExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprMember(it) => <MemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprSuperProp(it) => <SuperPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprCond(it) => <CondExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprImport(it) => <ImportExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprCall(it) => <CallExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprNew(it) => <NewExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprSeq(it) => <SeqExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprIdent(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprStr(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprBool(it) => <Bool as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprNull(it) => <Null as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprNum(it) => <Number as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprBigInt(it) => <BigInt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprRegex(it) => <Regex as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprTpl(it) => <Tpl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprTaggedTpl(it) => <TaggedTpl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprArrow(it) => <ArrowExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprClass(it) => <ClassExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprYield(it) => <YieldExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprMetaProp(it) => <MetaPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprAwait(it) => <AwaitExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprParen(it) => <ParenExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprJSXMember(it) => <JSXMemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprJSXNamespacedName(it) => {
                <JSXNamespacedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::ExprJSXEmpty(it) => <JSXEmptyExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprJSXElement(it) => <JSXElement as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprJSXFragment(it) => <JSXFragment as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprPrivateName(it) => <PrivateName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprOptChain(it) => <OptChainExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::ExprInvalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Ident(it) => <BindingIdent as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Array(it) => <ArrayPat as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Rest(it) => <RestPat as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Object(it) => <ObjectPat as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Assign(it) => <AssignPat as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Invalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::KeyValue(it) => <KeyValuePatProp as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Assign(it) => <AssignPatProp as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Rest(it) => <RestPat as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Str(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Bool(it) => <Bool as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Null(it) => <Null as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Num(it) => <Number as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::BigInt(it) => <BigInt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Regex(it) => <Regex as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::JSXMemberExpr(it) => <JSXMemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Ident(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::This(it) => <ThisExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Array(it) => <ArrayLit as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Object(it) => <ObjectLit as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Fn(it) => <FnExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Unary(it) => <UnaryExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Update(it) => <UpdateExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Bin(it) => <BinExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Assign(it) => <AssignExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Member(it) => <MemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::SuperProp(it) => <SuperPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Cond(it) => <CondExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Import(it) => <ImportExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Call(it) => <CallExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::New(it) => <NewExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Seq(it) => <SeqExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Ident(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Str(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Bool(it) => <Bool as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Null(it) => <Null as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Num(it) => <Number as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::BigInt(it) => <BigInt as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Regex(it) => <Regex as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Tpl(it) => <Tpl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::TaggedTpl(it) => <TaggedTpl as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Arrow(it) => <ArrowExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Class(it) => <ClassExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Yield(it) => <YieldExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::MetaProp(it) => <MetaPropExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Await(it) => <AwaitExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Paren(it) => <ParenExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXMember(it) => <JSXMemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXNamespacedName(it) => {
                <JSXNamespacedName as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::JSXEmpty(it) => <JSXEmptyExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXElement(it) => <JSXElement as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXFragment(it) => <JSXFragment as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::PrivateName(it) => <PrivateName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::OptChain(it) => <OptChainExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::Invalid(it) => <Invalid as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXEmptyExpr(it) => <JSXEmptyExpr as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Ident(it) => <Ident as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXMemberExpr(it) => <JSXMemberExpr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXNamespacedName(it) => {
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
        match self {
            Self::JSXAttr(it) => <JSXAttr as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::SpreadElement(it) => <SpreadElement as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::Ident(it) => <IdentName as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXNamespacedName(it) => {
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
        match self {
            Self::Str(it) => <Str as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXExprContainer(it) => {
                <JSXExprContainer as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::JSXElement(it) => <JSXElement as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXFragment(it) => <JSXFragment as VisitWith<'a, V>>::visit_with(it, visitor),
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
        match self {
            Self::JSXText(it) => <JSXText as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXExprContainer(it) => {
                <JSXExprContainer as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::JSXSpreadChild(it) => {
                <JSXSpreadChild as VisitWith<'a, V>>::visit_with(it, visitor)
            }
            Self::JSXElement(it) => <JSXElement as VisitWith<'a, V>>::visit_with(it, visitor),
            Self::JSXFragment(it) => <JSXFragment as VisitWith<'a, V>>::visit_with(it, visitor),
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
    fn enter_node(&mut self, node_kind: NodeKind) {}
    #[inline]
    fn leave_node(&mut self, node_kind: NodeKind) {}
    #[inline]
    fn visit_mut_program(&mut self, node: Program) -> Program {
        <Program as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_module(&mut self, node: Module) -> Module {
        self.enter_node(NodeKind::Module);
        let node = <Module as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Module);
        node
    }
    #[inline]
    fn visit_mut_script(&mut self, node: Script) -> Script {
        self.enter_node(NodeKind::Script);
        let node = <Script as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Script);
        node
    }
    #[inline]
    fn visit_mut_module_item(&mut self, node: ModuleItem) -> ModuleItem {
        <ModuleItem as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_module_decl(&mut self, node: ModuleDecl) -> ModuleDecl {
        <ModuleDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_decl(&mut self, node: ImportDecl) -> ImportDecl {
        self.enter_node(NodeKind::ImportDecl);
        let node = <ImportDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ImportDecl);
        node
    }
    #[inline]
    fn visit_mut_import_specifier(&mut self, node: ImportSpecifier) -> ImportSpecifier {
        <ImportSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_import_named_specifier(
        &mut self,
        node: ImportNamedSpecifier,
    ) -> ImportNamedSpecifier {
        self.enter_node(NodeKind::ImportNamedSpecifier);
        let node =
            <ImportNamedSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
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
            <ImportDefaultSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
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
            <ImportStarAsSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ImportStarAsSpecifier);
        node
    }
    #[inline]
    fn visit_mut_export_decl(&mut self, node: ExportDecl) -> ExportDecl {
        self.enter_node(NodeKind::ExportDecl);
        let node = <ExportDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExportDecl);
        node
    }
    #[inline]
    fn visit_mut_named_export(&mut self, node: NamedExport) -> NamedExport {
        self.enter_node(NodeKind::NamedExport);
        let node = <NamedExport as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::NamedExport);
        node
    }
    #[inline]
    fn visit_mut_export_specifier(&mut self, node: ExportSpecifier) -> ExportSpecifier {
        <ExportSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_namespace_specifier(
        &mut self,
        node: ExportNamespaceSpecifier,
    ) -> ExportNamespaceSpecifier {
        self.enter_node(NodeKind::ExportNamespaceSpecifier);
        let node = <ExportNamespaceSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
        self.leave_node(NodeKind::ExportNamespaceSpecifier);
        node
    }
    #[inline]
    fn visit_mut_module_export_name(&mut self, node: ModuleExportName) -> ModuleExportName {
        <ModuleExportName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_default_specifier(
        &mut self,
        node: ExportDefaultSpecifier,
    ) -> ExportDefaultSpecifier {
        self.enter_node(NodeKind::ExportDefaultSpecifier);
        let node =
            <ExportDefaultSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
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
            <ExportNamedSpecifier as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExportNamedSpecifier);
        node
    }
    #[inline]
    fn visit_mut_export_default_decl(&mut self, node: ExportDefaultDecl) -> ExportDefaultDecl {
        self.enter_node(NodeKind::ExportDefaultDecl);
        let node =
            <ExportDefaultDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExportDefaultDecl);
        node
    }
    #[inline]
    fn visit_mut_default_decl(&mut self, node: DefaultDecl) -> DefaultDecl {
        <DefaultDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_default_expr(&mut self, node: ExportDefaultExpr) -> ExportDefaultExpr {
        self.enter_node(NodeKind::ExportDefaultExpr);
        let node =
            <ExportDefaultExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExportDefaultExpr);
        node
    }
    #[inline]
    fn visit_mut_export_all(&mut self, node: ExportAll) -> ExportAll {
        self.enter_node(NodeKind::ExportAll);
        let node = <ExportAll as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExportAll);
        node
    }
    #[inline]
    fn visit_mut_block_stmt(&mut self, node: BlockStmt) -> BlockStmt {
        self.enter_node(NodeKind::BlockStmt);
        let node = <BlockStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::BlockStmt);
        node
    }
    #[inline]
    fn visit_mut_stmt(&mut self, node: Stmt) -> Stmt {
        <Stmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_expr_stmt(&mut self, node: ExprStmt) -> ExprStmt {
        self.enter_node(NodeKind::ExprStmt);
        let node = <ExprStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExprStmt);
        node
    }
    #[inline]
    fn visit_mut_empty_stmt(&mut self, node: EmptyStmt) -> EmptyStmt {
        self.enter_node(NodeKind::EmptyStmt);
        let node = <EmptyStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::EmptyStmt);
        node
    }
    #[inline]
    fn visit_mut_debugger_stmt(&mut self, node: DebuggerStmt) -> DebuggerStmt {
        self.enter_node(NodeKind::DebuggerStmt);
        let node = <DebuggerStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::DebuggerStmt);
        node
    }
    #[inline]
    fn visit_mut_with_stmt(&mut self, node: WithStmt) -> WithStmt {
        self.enter_node(NodeKind::WithStmt);
        let node = <WithStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::WithStmt);
        node
    }
    #[inline]
    fn visit_mut_return_stmt(&mut self, node: ReturnStmt) -> ReturnStmt {
        self.enter_node(NodeKind::ReturnStmt);
        let node = <ReturnStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ReturnStmt);
        node
    }
    #[inline]
    fn visit_mut_labeled_stmt(&mut self, node: LabeledStmt) -> LabeledStmt {
        self.enter_node(NodeKind::LabeledStmt);
        let node = <LabeledStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::LabeledStmt);
        node
    }
    #[inline]
    fn visit_mut_break_stmt(&mut self, node: BreakStmt) -> BreakStmt {
        self.enter_node(NodeKind::BreakStmt);
        let node = <BreakStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::BreakStmt);
        node
    }
    #[inline]
    fn visit_mut_continue_stmt(&mut self, node: ContinueStmt) -> ContinueStmt {
        self.enter_node(NodeKind::ContinueStmt);
        let node = <ContinueStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ContinueStmt);
        node
    }
    #[inline]
    fn visit_mut_if_stmt(&mut self, node: IfStmt) -> IfStmt {
        self.enter_node(NodeKind::IfStmt);
        let node = <IfStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::IfStmt);
        node
    }
    #[inline]
    fn visit_mut_switch_stmt(&mut self, node: SwitchStmt) -> SwitchStmt {
        self.enter_node(NodeKind::SwitchStmt);
        let node = <SwitchStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::SwitchStmt);
        node
    }
    #[inline]
    fn visit_mut_throw_stmt(&mut self, node: ThrowStmt) -> ThrowStmt {
        self.enter_node(NodeKind::ThrowStmt);
        let node = <ThrowStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ThrowStmt);
        node
    }
    #[inline]
    fn visit_mut_try_stmt(&mut self, node: TryStmt) -> TryStmt {
        self.enter_node(NodeKind::TryStmt);
        let node = <TryStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::TryStmt);
        node
    }
    #[inline]
    fn visit_mut_while_stmt(&mut self, node: WhileStmt) -> WhileStmt {
        self.enter_node(NodeKind::WhileStmt);
        let node = <WhileStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::WhileStmt);
        node
    }
    #[inline]
    fn visit_mut_do_while_stmt(&mut self, node: DoWhileStmt) -> DoWhileStmt {
        self.enter_node(NodeKind::DoWhileStmt);
        let node = <DoWhileStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::DoWhileStmt);
        node
    }
    #[inline]
    fn visit_mut_for_stmt(&mut self, node: ForStmt) -> ForStmt {
        self.enter_node(NodeKind::ForStmt);
        let node = <ForStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ForStmt);
        node
    }
    #[inline]
    fn visit_mut_for_in_stmt(&mut self, node: ForInStmt) -> ForInStmt {
        self.enter_node(NodeKind::ForInStmt);
        let node = <ForInStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ForInStmt);
        node
    }
    #[inline]
    fn visit_mut_for_of_stmt(&mut self, node: ForOfStmt) -> ForOfStmt {
        self.enter_node(NodeKind::ForOfStmt);
        let node = <ForOfStmt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ForOfStmt);
        node
    }
    #[inline]
    fn visit_mut_switch_case(&mut self, node: SwitchCase) -> SwitchCase {
        self.enter_node(NodeKind::SwitchCase);
        let node = <SwitchCase as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::SwitchCase);
        node
    }
    #[inline]
    fn visit_mut_catch_clause(&mut self, node: CatchClause) -> CatchClause {
        self.enter_node(NodeKind::CatchClause);
        let node = <CatchClause as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::CatchClause);
        node
    }
    #[inline]
    fn visit_mut_for_head(&mut self, node: ForHead) -> ForHead {
        <ForHead as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_var_decl_or_expr(&mut self, node: VarDeclOrExpr) -> VarDeclOrExpr {
        <VarDeclOrExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_decl(&mut self, node: Decl) -> Decl {
        <Decl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_fn_decl(&mut self, node: FnDecl) -> FnDecl {
        self.enter_node(NodeKind::FnDecl);
        let node = <FnDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::FnDecl);
        node
    }
    #[inline]
    fn visit_mut_class_decl(&mut self, node: ClassDecl) -> ClassDecl {
        self.enter_node(NodeKind::ClassDecl);
        let node = <ClassDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ClassDecl);
        node
    }
    #[inline]
    fn visit_mut_var_decl(&mut self, node: VarDecl) -> VarDecl {
        self.enter_node(NodeKind::VarDecl);
        let node = <VarDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::VarDecl);
        node
    }
    #[inline]
    fn visit_mut_var_declarator(&mut self, node: VarDeclarator) -> VarDeclarator {
        self.enter_node(NodeKind::VarDeclarator);
        let node = <VarDeclarator as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::VarDeclarator);
        node
    }
    #[inline]
    fn visit_mut_using_decl(&mut self, node: UsingDecl) -> UsingDecl {
        self.enter_node(NodeKind::UsingDecl);
        let node = <UsingDecl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::UsingDecl);
        node
    }
    #[inline]
    fn visit_mut_expr(&mut self, node: Expr) -> Expr {
        <Expr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_this_expr(&mut self, node: ThisExpr) -> ThisExpr {
        self.enter_node(NodeKind::ThisExpr);
        let node = <ThisExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ThisExpr);
        node
    }
    #[inline]
    fn visit_mut_array_lit(&mut self, node: ArrayLit) -> ArrayLit {
        self.enter_node(NodeKind::ArrayLit);
        let node = <ArrayLit as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ArrayLit);
        node
    }
    #[inline]
    fn visit_mut_object_lit(&mut self, node: ObjectLit) -> ObjectLit {
        self.enter_node(NodeKind::ObjectLit);
        let node = <ObjectLit as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ObjectLit);
        node
    }
    #[inline]
    fn visit_mut_prop_or_spread(&mut self, node: PropOrSpread) -> PropOrSpread {
        <PropOrSpread as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_spread_element(&mut self, node: SpreadElement) -> SpreadElement {
        self.enter_node(NodeKind::SpreadElement);
        let node = <SpreadElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::SpreadElement);
        node
    }
    #[inline]
    fn visit_mut_unary_expr(&mut self, node: UnaryExpr) -> UnaryExpr {
        self.enter_node(NodeKind::UnaryExpr);
        let node = <UnaryExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::UnaryExpr);
        node
    }
    #[inline]
    fn visit_mut_update_expr(&mut self, node: UpdateExpr) -> UpdateExpr {
        self.enter_node(NodeKind::UpdateExpr);
        let node = <UpdateExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::UpdateExpr);
        node
    }
    #[inline]
    fn visit_mut_bin_expr(&mut self, node: BinExpr) -> BinExpr {
        self.enter_node(NodeKind::BinExpr);
        let node = <BinExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::BinExpr);
        node
    }
    #[inline]
    fn visit_mut_fn_expr(&mut self, node: FnExpr) -> FnExpr {
        self.enter_node(NodeKind::FnExpr);
        let node = <FnExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::FnExpr);
        node
    }
    #[inline]
    fn visit_mut_class_expr(&mut self, node: ClassExpr) -> ClassExpr {
        self.enter_node(NodeKind::ClassExpr);
        let node = <ClassExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ClassExpr);
        node
    }
    #[inline]
    fn visit_mut_assign_expr(&mut self, node: AssignExpr) -> AssignExpr {
        self.enter_node(NodeKind::AssignExpr);
        let node = <AssignExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::AssignExpr);
        node
    }
    #[inline]
    fn visit_mut_member_expr(&mut self, node: MemberExpr) -> MemberExpr {
        self.enter_node(NodeKind::MemberExpr);
        let node = <MemberExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::MemberExpr);
        node
    }
    #[inline]
    fn visit_mut_member_prop(&mut self, node: MemberProp) -> MemberProp {
        <MemberProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_super_prop_expr(&mut self, node: SuperPropExpr) -> SuperPropExpr {
        self.enter_node(NodeKind::SuperPropExpr);
        let node = <SuperPropExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::SuperPropExpr);
        node
    }
    #[inline]
    fn visit_mut_super_prop(&mut self, node: SuperProp) -> SuperProp {
        <SuperProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_cond_expr(&mut self, node: CondExpr) -> CondExpr {
        self.enter_node(NodeKind::CondExpr);
        let node = <CondExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::CondExpr);
        node
    }
    #[inline]
    fn visit_mut_import_expr(&mut self, node: ImportExpr) -> ImportExpr {
        self.enter_node(NodeKind::ImportExpr);
        let node = <ImportExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ImportExpr);
        node
    }
    #[inline]
    fn visit_mut_call_expr(&mut self, node: CallExpr) -> CallExpr {
        self.enter_node(NodeKind::CallExpr);
        let node = <CallExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::CallExpr);
        node
    }
    #[inline]
    fn visit_mut_new_expr(&mut self, node: NewExpr) -> NewExpr {
        self.enter_node(NodeKind::NewExpr);
        let node = <NewExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::NewExpr);
        node
    }
    #[inline]
    fn visit_mut_seq_expr(&mut self, node: SeqExpr) -> SeqExpr {
        self.enter_node(NodeKind::SeqExpr);
        let node = <SeqExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::SeqExpr);
        node
    }
    #[inline]
    fn visit_mut_arrow_expr(&mut self, node: ArrowExpr) -> ArrowExpr {
        self.enter_node(NodeKind::ArrowExpr);
        let node = <ArrowExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ArrowExpr);
        node
    }
    #[inline]
    fn visit_mut_yield_expr(&mut self, node: YieldExpr) -> YieldExpr {
        self.enter_node(NodeKind::YieldExpr);
        let node = <YieldExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::YieldExpr);
        node
    }
    #[inline]
    fn visit_mut_meta_prop_expr(&mut self, node: MetaPropExpr) -> MetaPropExpr {
        self.enter_node(NodeKind::MetaPropExpr);
        let node = <MetaPropExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::MetaPropExpr);
        node
    }
    #[inline]
    fn visit_mut_await_expr(&mut self, node: AwaitExpr) -> AwaitExpr {
        self.enter_node(NodeKind::AwaitExpr);
        let node = <AwaitExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::AwaitExpr);
        node
    }
    #[inline]
    fn visit_mut_tpl(&mut self, node: Tpl) -> Tpl {
        self.enter_node(NodeKind::Tpl);
        let node = <Tpl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Tpl);
        node
    }
    #[inline]
    fn visit_mut_tagged_tpl(&mut self, node: TaggedTpl) -> TaggedTpl {
        self.enter_node(NodeKind::TaggedTpl);
        let node = <TaggedTpl as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::TaggedTpl);
        node
    }
    #[inline]
    fn visit_mut_tpl_element(&mut self, node: TplElement) -> TplElement {
        self.enter_node(NodeKind::TplElement);
        let node = <TplElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::TplElement);
        node
    }
    #[inline]
    fn visit_mut_paren_expr(&mut self, node: ParenExpr) -> ParenExpr {
        self.enter_node(NodeKind::ParenExpr);
        let node = <ParenExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ParenExpr);
        node
    }
    #[inline]
    fn visit_mut_callee(&mut self, node: Callee) -> Callee {
        <Callee as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_super(&mut self, node: Super) -> Super {
        self.enter_node(NodeKind::Super);
        let node = <Super as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Super);
        node
    }
    #[inline]
    fn visit_mut_expr_or_spread(&mut self, node: ExprOrSpread) -> ExprOrSpread {
        self.enter_node(NodeKind::ExprOrSpread);
        let node = <ExprOrSpread as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ExprOrSpread);
        node
    }
    #[inline]
    fn visit_mut_block_stmt_or_expr(&mut self, node: BlockStmtOrExpr) -> BlockStmtOrExpr {
        <BlockStmtOrExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_assign_target(&mut self, node: AssignTarget) -> AssignTarget {
        <AssignTarget as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_assign_target_pat(&mut self, node: AssignTargetPat) -> AssignTargetPat {
        <AssignTargetPat as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_simple_assign_target(&mut self, node: SimpleAssignTarget) -> SimpleAssignTarget {
        <SimpleAssignTarget as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_chain_expr(&mut self, node: OptChainExpr) -> OptChainExpr {
        self.enter_node(NodeKind::OptChainExpr);
        let node = <OptChainExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::OptChainExpr);
        node
    }
    #[inline]
    fn visit_mut_opt_chain_base(&mut self, node: OptChainBase) -> OptChainBase {
        <OptChainBase as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_call(&mut self, node: OptCall) -> OptCall {
        self.enter_node(NodeKind::OptCall);
        let node = <OptCall as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::OptCall);
        node
    }
    #[inline]
    fn visit_mut_invalid(&mut self, node: Invalid) -> Invalid {
        self.enter_node(NodeKind::Invalid);
        let node = <Invalid as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Invalid);
        node
    }
    #[inline]
    fn visit_mut_function(&mut self, node: Function) -> Function {
        self.enter_node(NodeKind::Function);
        let node = <Function as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Function);
        node
    }
    #[inline]
    fn visit_mut_param(&mut self, node: Param) -> Param {
        self.enter_node(NodeKind::Param);
        let node = <Param as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Param);
        node
    }
    #[inline]
    fn visit_mut_param_or_ts_param_prop(&mut self, node: ParamOrTsParamProp) -> ParamOrTsParamProp {
        <ParamOrTsParamProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_class(&mut self, node: Class) -> Class {
        self.enter_node(NodeKind::Class);
        let node = <Class as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Class);
        node
    }
    #[inline]
    fn visit_mut_class_member(&mut self, node: ClassMember) -> ClassMember {
        <ClassMember as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_class_prop(&mut self, node: ClassProp) -> ClassProp {
        self.enter_node(NodeKind::ClassProp);
        let node = <ClassProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ClassProp);
        node
    }
    #[inline]
    fn visit_mut_private_prop(&mut self, node: PrivateProp) -> PrivateProp {
        self.enter_node(NodeKind::PrivateProp);
        let node = <PrivateProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::PrivateProp);
        node
    }
    #[inline]
    fn visit_mut_class_method(&mut self, node: ClassMethod) -> ClassMethod {
        self.enter_node(NodeKind::ClassMethod);
        let node = <ClassMethod as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ClassMethod);
        node
    }
    #[inline]
    fn visit_mut_private_method(&mut self, node: PrivateMethod) -> PrivateMethod {
        self.enter_node(NodeKind::PrivateMethod);
        let node = <PrivateMethod as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::PrivateMethod);
        node
    }
    #[inline]
    fn visit_mut_constructor(&mut self, node: Constructor) -> Constructor {
        self.enter_node(NodeKind::Constructor);
        let node = <Constructor as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Constructor);
        node
    }
    #[inline]
    fn visit_mut_decorator(&mut self, node: Decorator) -> Decorator {
        self.enter_node(NodeKind::Decorator);
        let node = <Decorator as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Decorator);
        node
    }
    #[inline]
    fn visit_mut_static_block(&mut self, node: StaticBlock) -> StaticBlock {
        self.enter_node(NodeKind::StaticBlock);
        let node = <StaticBlock as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::StaticBlock);
        node
    }
    #[inline]
    fn visit_mut_key(&mut self, node: Key) -> Key {
        <Key as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_auto_accessor(&mut self, node: AutoAccessor) -> AutoAccessor {
        self.enter_node(NodeKind::AutoAccessor);
        let node = <AutoAccessor as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::AutoAccessor);
        node
    }
    #[inline]
    fn visit_mut_prop(&mut self, node: Prop) -> Prop {
        <Prop as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_key_value_prop(&mut self, node: KeyValueProp) -> KeyValueProp {
        self.enter_node(NodeKind::KeyValueProp);
        let node = <KeyValueProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::KeyValueProp);
        node
    }
    #[inline]
    fn visit_mut_assign_prop(&mut self, node: AssignProp) -> AssignProp {
        self.enter_node(NodeKind::AssignProp);
        let node = <AssignProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::AssignProp);
        node
    }
    #[inline]
    fn visit_mut_getter_prop(&mut self, node: GetterProp) -> GetterProp {
        self.enter_node(NodeKind::GetterProp);
        let node = <GetterProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::GetterProp);
        node
    }
    #[inline]
    fn visit_mut_setter_prop(&mut self, node: SetterProp) -> SetterProp {
        self.enter_node(NodeKind::SetterProp);
        let node = <SetterProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::SetterProp);
        node
    }
    #[inline]
    fn visit_mut_method_prop(&mut self, node: MethodProp) -> MethodProp {
        self.enter_node(NodeKind::MethodProp);
        let node = <MethodProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::MethodProp);
        node
    }
    #[inline]
    fn visit_mut_prop_name(&mut self, node: PropName) -> PropName {
        <PropName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_computed_prop_name(&mut self, node: ComputedPropName) -> ComputedPropName {
        self.enter_node(NodeKind::ComputedPropName);
        let node =
            <ComputedPropName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ComputedPropName);
        node
    }
    #[inline]
    fn visit_mut_pat(&mut self, node: Pat) -> Pat {
        <Pat as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_array_pat(&mut self, node: ArrayPat) -> ArrayPat {
        self.enter_node(NodeKind::ArrayPat);
        let node = <ArrayPat as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ArrayPat);
        node
    }
    #[inline]
    fn visit_mut_object_pat(&mut self, node: ObjectPat) -> ObjectPat {
        self.enter_node(NodeKind::ObjectPat);
        let node = <ObjectPat as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::ObjectPat);
        node
    }
    #[inline]
    fn visit_mut_assign_pat(&mut self, node: AssignPat) -> AssignPat {
        self.enter_node(NodeKind::AssignPat);
        let node = <AssignPat as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::AssignPat);
        node
    }
    #[inline]
    fn visit_mut_rest_pat(&mut self, node: RestPat) -> RestPat {
        self.enter_node(NodeKind::RestPat);
        let node = <RestPat as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::RestPat);
        node
    }
    #[inline]
    fn visit_mut_object_pat_prop(&mut self, node: ObjectPatProp) -> ObjectPatProp {
        <ObjectPatProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_key_value_pat_prop(&mut self, node: KeyValuePatProp) -> KeyValuePatProp {
        self.enter_node(NodeKind::KeyValuePatProp);
        let node = <KeyValuePatProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::KeyValuePatProp);
        node
    }
    #[inline]
    fn visit_mut_assign_pat_prop(&mut self, node: AssignPatProp) -> AssignPatProp {
        self.enter_node(NodeKind::AssignPatProp);
        let node = <AssignPatProp as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::AssignPatProp);
        node
    }
    #[inline]
    fn visit_mut_ident(&mut self, node: Ident) -> Ident {
        self.enter_node(NodeKind::Ident);
        let node = <Ident as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Ident);
        node
    }
    #[inline]
    fn visit_mut_ident_name(&mut self, node: IdentName) -> IdentName {
        self.enter_node(NodeKind::IdentName);
        let node = <IdentName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::IdentName);
        node
    }
    #[inline]
    fn visit_mut_private_name(&mut self, node: PrivateName) -> PrivateName {
        self.enter_node(NodeKind::PrivateName);
        let node = <PrivateName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::PrivateName);
        node
    }
    #[inline]
    fn visit_mut_binding_ident(&mut self, node: BindingIdent) -> BindingIdent {
        self.enter_node(NodeKind::BindingIdent);
        let node = <BindingIdent as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::BindingIdent);
        node
    }
    #[inline]
    fn visit_mut_lit(&mut self, node: Lit) -> Lit {
        <Lit as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_str(&mut self, node: Str) -> Str {
        self.enter_node(NodeKind::Str);
        let node = <Str as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Str);
        node
    }
    #[inline]
    fn visit_mut_bool(&mut self, node: Bool) -> Bool {
        self.enter_node(NodeKind::Bool);
        let node = <Bool as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Bool);
        node
    }
    #[inline]
    fn visit_mut_null(&mut self, node: Null) -> Null {
        self.enter_node(NodeKind::Null);
        let node = <Null as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Null);
        node
    }
    #[inline]
    fn visit_mut_number(&mut self, node: Number) -> Number {
        self.enter_node(NodeKind::Number);
        let node = <Number as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Number);
        node
    }
    #[inline]
    fn visit_mut_big_int(&mut self, node: BigInt) -> BigInt {
        self.enter_node(NodeKind::BigInt);
        let node = <BigInt as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::BigInt);
        node
    }
    #[inline]
    fn visit_mut_regex(&mut self, node: Regex) -> Regex {
        self.enter_node(NodeKind::Regex);
        let node = <Regex as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::Regex);
        node
    }
    #[inline]
    fn visit_mut_jsx_object(&mut self, node: JSXObject) -> JSXObject {
        <JSXObject as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_member_expr(&mut self, node: JSXMemberExpr) -> JSXMemberExpr {
        self.enter_node(NodeKind::JSXMemberExpr);
        let node = <JSXMemberExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXMemberExpr);
        node
    }
    #[inline]
    fn visit_mut_jsx_namespaced_name(&mut self, node: JSXNamespacedName) -> JSXNamespacedName {
        self.enter_node(NodeKind::JSXNamespacedName);
        let node =
            <JSXNamespacedName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXNamespacedName);
        node
    }
    #[inline]
    fn visit_mut_jsx_empty_expr(&mut self, node: JSXEmptyExpr) -> JSXEmptyExpr {
        self.enter_node(NodeKind::JSXEmptyExpr);
        let node = <JSXEmptyExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXEmptyExpr);
        node
    }
    #[inline]
    fn visit_mut_jsx_expr_container(&mut self, node: JSXExprContainer) -> JSXExprContainer {
        self.enter_node(NodeKind::JSXExprContainer);
        let node =
            <JSXExprContainer as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXExprContainer);
        node
    }
    #[inline]
    fn visit_mut_jsx_expr(&mut self, node: JSXExpr) -> JSXExpr {
        <JSXExpr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_spread_child(&mut self, node: JSXSpreadChild) -> JSXSpreadChild {
        self.enter_node(NodeKind::JSXSpreadChild);
        let node = <JSXSpreadChild as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXSpreadChild);
        node
    }
    #[inline]
    fn visit_mut_jsx_element_name(&mut self, node: JSXElementName) -> JSXElementName {
        <JSXElementName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_opening_element(&mut self, node: JSXOpeningElement) -> JSXOpeningElement {
        self.enter_node(NodeKind::JSXOpeningElement);
        let node =
            <JSXOpeningElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXOpeningElement);
        node
    }
    #[inline]
    fn visit_mut_jsx_attr_or_spread(&mut self, node: JSXAttrOrSpread) -> JSXAttrOrSpread {
        <JSXAttrOrSpread as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_closing_element(&mut self, node: JSXClosingElement) -> JSXClosingElement {
        self.enter_node(NodeKind::JSXClosingElement);
        let node =
            <JSXClosingElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXClosingElement);
        node
    }
    #[inline]
    fn visit_mut_jsx_attr(&mut self, node: JSXAttr) -> JSXAttr {
        self.enter_node(NodeKind::JSXAttr);
        let node = <JSXAttr as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXAttr);
        node
    }
    #[inline]
    fn visit_mut_jsx_attr_name(&mut self, node: JSXAttrName) -> JSXAttrName {
        <JSXAttrName as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_attr_value(&mut self, node: JSXAttrValue) -> JSXAttrValue {
        <JSXAttrValue as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_text(&mut self, node: JSXText) -> JSXText {
        self.enter_node(NodeKind::JSXText);
        let node = <JSXText as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXText);
        node
    }
    #[inline]
    fn visit_mut_jsx_element(&mut self, node: JSXElement) -> JSXElement {
        self.enter_node(NodeKind::JSXElement);
        let node = <JSXElement as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXElement);
        node
    }
    #[inline]
    fn visit_mut_jsx_element_child(&mut self, node: JSXElementChild) -> JSXElementChild {
        <JSXElementChild as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_fragment(&mut self, node: JSXFragment) -> JSXFragment {
        self.enter_node(NodeKind::JSXFragment);
        let node = <JSXFragment as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXFragment);
        node
    }
    #[inline]
    fn visit_mut_jsx_opening_fragment(&mut self, node: JSXOpeningFragment) -> JSXOpeningFragment {
        self.enter_node(NodeKind::JSXOpeningFragment);
        let node =
            <JSXOpeningFragment as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXOpeningFragment);
        node
    }
    #[inline]
    fn visit_mut_jsx_closing_fragment(&mut self, node: JSXClosingFragment) -> JSXClosingFragment {
        self.enter_node(NodeKind::JSXClosingFragment);
        let node =
            <JSXClosingFragment as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        self.leave_node(NodeKind::JSXClosingFragment);
        node
    }
    #[inline]
    fn visit_mut_module_items(
        &mut self,
        node: TypedSubRange<ModuleItem>,
    ) -> TypedSubRange<ModuleItem> {
        <TypedSubRange<ModuleItem> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_stmts(&mut self, node: TypedSubRange<Stmt>) -> TypedSubRange<Stmt> {
        <TypedSubRange<Stmt> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_import_specifiers(
        &mut self,
        node: TypedSubRange<ImportSpecifier>,
    ) -> TypedSubRange<ImportSpecifier> {
        <TypedSubRange<ImportSpecifier> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
        node
    }
    #[inline]
    fn visit_mut_opt_object_lit(&mut self, node: Option<ObjectLit>) -> Option<ObjectLit> {
        <Option<ObjectLit> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_export_specifiers(
        &mut self,
        node: TypedSubRange<ExportSpecifier>,
    ) -> TypedSubRange<ExportSpecifier> {
        <TypedSubRange<ExportSpecifier> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
        node
    }
    #[inline]
    fn visit_mut_opt_str(&mut self, node: Option<Str>) -> Option<Str> {
        <Option<Str> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_expr(&mut self, node: Option<Expr>) -> Option<Expr> {
        <Option<Expr> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_ident(&mut self, node: Option<Ident>) -> Option<Ident> {
        <Option<Ident> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_stmt(&mut self, node: Option<Stmt>) -> Option<Stmt> {
        <Option<Stmt> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_switch_cases(
        &mut self,
        node: TypedSubRange<SwitchCase>,
    ) -> TypedSubRange<SwitchCase> {
        <TypedSubRange<SwitchCase> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_opt_catch_clause(&mut self, node: Option<CatchClause>) -> Option<CatchClause> {
        <Option<CatchClause> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_block_stmt(&mut self, node: Option<BlockStmt>) -> Option<BlockStmt> {
        <Option<BlockStmt> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_var_decl_or_expr(
        &mut self,
        node: Option<VarDeclOrExpr>,
    ) -> Option<VarDeclOrExpr> {
        <Option<VarDeclOrExpr> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_pat(&mut self, node: Option<Pat>) -> Option<Pat> {
        <Option<Pat> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_var_declarators(
        &mut self,
        node: TypedSubRange<VarDeclarator>,
    ) -> TypedSubRange<VarDeclarator> {
        <TypedSubRange<VarDeclarator> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
        node
    }
    #[inline]
    fn visit_mut_opt_expr_or_spread(&mut self, node: Option<ExprOrSpread>) -> Option<ExprOrSpread> {
        <Option<ExprOrSpread> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_opt_vec_expr_or_spreads(
        &mut self,
        node: TypedSubRange<Option<ExprOrSpread>>,
    ) -> TypedSubRange<Option<ExprOrSpread>> {
        <TypedSubRange<Option<ExprOrSpread>> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
        node
    }
    #[inline]
    fn visit_mut_prop_or_spreads(
        &mut self,
        node: TypedSubRange<PropOrSpread>,
    ) -> TypedSubRange<PropOrSpread> {
        <TypedSubRange<PropOrSpread> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
        node
    }
    #[inline]
    fn visit_mut_expr_or_spreads(
        &mut self,
        node: TypedSubRange<ExprOrSpread>,
    ) -> TypedSubRange<ExprOrSpread> {
        <TypedSubRange<ExprOrSpread> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
        node
    }
    #[inline]
    fn visit_mut_exprs(&mut self, node: TypedSubRange<Expr>) -> TypedSubRange<Expr> {
        <TypedSubRange<Expr> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_pats(&mut self, node: TypedSubRange<Pat>) -> TypedSubRange<Pat> {
        <TypedSubRange<Pat> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_tpl_elements(
        &mut self,
        node: TypedSubRange<TplElement>,
    ) -> TypedSubRange<TplElement> {
        <TypedSubRange<TplElement> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_params(&mut self, node: TypedSubRange<Param>) -> TypedSubRange<Param> {
        <TypedSubRange<Param> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_decorators(&mut self, node: TypedSubRange<Decorator>) -> TypedSubRange<Decorator> {
        <TypedSubRange<Decorator> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_class_members(
        &mut self,
        node: TypedSubRange<ClassMember>,
    ) -> TypedSubRange<ClassMember> {
        <TypedSubRange<ClassMember> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_param_or_ts_param_props(
        &mut self,
        node: TypedSubRange<ParamOrTsParamProp>,
    ) -> TypedSubRange<ParamOrTsParamProp> {
        <TypedSubRange<ParamOrTsParamProp> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
        node
    }
    #[inline]
    fn visit_mut_opt_vec_pats(
        &mut self,
        node: TypedSubRange<Option<Pat>>,
    ) -> TypedSubRange<Option<Pat>> {
        <TypedSubRange<Option<Pat>> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self);
        node
    }
    #[inline]
    fn visit_mut_object_pat_props(
        &mut self,
        node: TypedSubRange<ObjectPatProp>,
    ) -> TypedSubRange<ObjectPatProp> {
        <TypedSubRange<ObjectPatProp> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
        node
    }
    #[inline]
    fn visit_mut_jsx_attr_or_spreads(
        &mut self,
        node: TypedSubRange<JSXAttrOrSpread>,
    ) -> TypedSubRange<JSXAttrOrSpread> {
        <TypedSubRange<JSXAttrOrSpread> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
        node
    }
    #[inline]
    fn visit_mut_opt_jsx_attr_value(&mut self, node: Option<JSXAttrValue>) -> Option<JSXAttrValue> {
        <Option<JSXAttrValue> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
    #[inline]
    fn visit_mut_jsx_element_childs(
        &mut self,
        node: TypedSubRange<JSXElementChild>,
    ) -> TypedSubRange<JSXElementChild> {
        <TypedSubRange<JSXElementChild> as VisitMutWith<'a, Self>>::visit_mut_children_with(
            node, self,
        );
        node
    }
    #[inline]
    fn visit_mut_opt_jsx_closing_element(
        &mut self,
        node: Option<JSXClosingElement>,
    ) -> Option<JSXClosingElement> {
        <Option<JSXClosingElement> as VisitMutWith<'a, Self>>::visit_mut_children_with(node, self)
    }
}
pub trait VisitMutWith<'a, V: ?Sized + VisitMut<'a>> {
    fn visit_mut_with(self, visitor: &mut V) -> Self;
    fn visit_mut_children_with(self, visitor: &mut V) -> Self;
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Program {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_program(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Module(it) => {
                Self::Module(<Module as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Script(it) => {
                Self::Script(<Script as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Module {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_module(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.body(visitor.ast());
        let new_node = <TypedSubRange<ModuleItem> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Script {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_script(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.body(visitor.ast());
        let new_node =
            <TypedSubRange<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ModuleItem {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_module_item(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::ModuleDeclImport(it) => {
                Self::ModuleDeclImport(<ImportDecl as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::ModuleDeclExportDecl(it) => Self::ModuleDeclExportDecl(
                <ExportDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ModuleDeclExportNamed(it) => Self::ModuleDeclExportNamed(
                <NamedExport as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ModuleDeclExportDefaultDecl(it) => Self::ModuleDeclExportDefaultDecl(
                <ExportDefaultDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ModuleDeclExportDefaultExpr(it) => Self::ModuleDeclExportDefaultExpr(
                <ExportDefaultExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ModuleDeclExportAll(it) => Self::ModuleDeclExportAll(
                <ExportAll as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtBlock(it) => Self::StmtBlock(
                <BlockStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtEmpty(it) => Self::StmtEmpty(
                <EmptyStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtDebugger(it) => Self::StmtDebugger(
                <DebuggerStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtWith(it) => Self::StmtWith(
                <WithStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtReturn(it) => Self::StmtReturn(
                <ReturnStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtLabeled(it) => Self::StmtLabeled(
                <LabeledStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtBreak(it) => Self::StmtBreak(
                <BreakStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtContinue(it) => Self::StmtContinue(
                <ContinueStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtIf(it) => {
                Self::StmtIf(<IfStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::StmtSwitch(it) => Self::StmtSwitch(
                <SwitchStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtThrow(it) => Self::StmtThrow(
                <ThrowStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtTry(it) => Self::StmtTry(<TryStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::StmtWhile(it) => Self::StmtWhile(
                <WhileStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtDoWhile(it) => Self::StmtDoWhile(
                <DoWhileStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtFor(it) => Self::StmtFor(<ForStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::StmtForIn(it) => Self::StmtForIn(
                <ForInStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtForOf(it) => Self::StmtForOf(
                <ForOfStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtDeclClass(it) => Self::StmtDeclClass(
                <ClassDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtDeclFn(it) => {
                Self::StmtDeclFn(<FnDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::StmtDeclVar(it) => Self::StmtDeclVar(
                <VarDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtDeclUsing(it) => Self::StmtDeclUsing(
                <UsingDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::StmtExpr(it) => Self::StmtExpr(
                <ExprStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ModuleDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_module_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Import(it) => Self::Import(<ImportDecl as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::ExportDecl(it) => Self::ExportDecl(
                <ExportDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExportNamed(it) => Self::ExportNamed(
                <NamedExport as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExportDefaultDecl(it) => Self::ExportDefaultDecl(
                <ExportDefaultDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExportDefaultExpr(it) => Self::ExportDefaultExpr(
                <ExportDefaultExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExportAll(it) => Self::ExportAll(
                <ExportAll as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_import_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.specifiers(visitor.ast());
        let new_node = <TypedSubRange<ImportSpecifier> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_specifiers(visitor.ast(), new_node);
        let field_value = self.src(visitor.ast());
        let new_node = <Str as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_src(visitor.ast(), new_node);
        let field_value = self.with(visitor.ast());
        let new_node =
            <Option<ObjectLit> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_with(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_import_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Named(it) => Self::Named(
                <ImportNamedSpecifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Default(it) => Self::Default(
                <ImportDefaultSpecifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Namespace(it) => Self::Namespace(<ImportStarAsSpecifier as VisitMutWith<
                'a,
                V,
            >>::visit_mut_with(it, visitor)),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportNamedSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_import_named_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.local(visitor.ast());
        let new_node = <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_local(visitor.ast(), new_node);
        let field_value = self.imported(visitor.ast());
        let new_node =
            <ModuleExportName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_imported(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportDefaultSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_import_default_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.local(visitor.ast());
        let new_node = <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_local(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportStarAsSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_import_star_as_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.local(visitor.ast());
        let new_node = <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_local(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_export_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.decl(visitor.ast());
        let new_node = <Decl as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_decl(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for NamedExport {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_named_export(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.specifiers(visitor.ast());
        let new_node = <TypedSubRange<ExportSpecifier> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_specifiers(visitor.ast(), new_node);
        let field_value = self.src(visitor.ast());
        let new_node = <Option<Str> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_src(visitor.ast(), new_node);
        let field_value = self.with(visitor.ast());
        let new_node =
            <Option<ObjectLit> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_with(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_export_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Namespace(it) => Self::Namespace(<ExportNamespaceSpecifier as VisitMutWith<
                'a,
                V,
            >>::visit_mut_with(it, visitor)),
            Self::Default(it) => Self::Default(
                <ExportDefaultSpecifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Named(it) => Self::Named(
                <ExportNamedSpecifier as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportNamespaceSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_export_namespace_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.name(visitor.ast());
        let new_node =
            <ModuleExportName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_name(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ModuleExportName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_module_export_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => {
                Self::Ident(<Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Str(it) => Self::Str(<Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportDefaultSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_export_default_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.exported(visitor.ast());
        let new_node = <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_exported(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportNamedSpecifier {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_export_named_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.orig(visitor.ast());
        let new_node =
            <ModuleExportName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_orig(visitor.ast(), new_node);
        let field_value = self.exported(visitor.ast());
        let new_node =
            <ModuleExportName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_exported(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportDefaultDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_export_default_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.decl(visitor.ast());
        let new_node = <DefaultDecl as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_decl(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for DefaultDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_default_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Class(it) => Self::Class(<ClassExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Fn(it) => Self::Fn(<FnExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportDefaultExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_export_default_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportAll {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_export_all(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.src(visitor.ast());
        let new_node = <Str as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_src(visitor.ast(), new_node);
        let field_value = self.with(visitor.ast());
        let new_node =
            <Option<ObjectLit> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_with(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BlockStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_block_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.stmts(visitor.ast());
        let new_node =
            <TypedSubRange<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_stmts(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Stmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Class(it) => Self::Class(<ClassDecl as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Fn(it) => Self::Fn(<FnDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Var(it) => Self::Var(<VarDecl as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Using(it) => Self::Using(<UsingDecl as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Block(it) => Self::Block(<BlockStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Empty(it) => Self::Empty(<EmptyStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Debugger(it) => Self::Debugger(
                <DebuggerStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::With(it) => Self::With(<WithStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Return(it) => Self::Return(<ReturnStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Labeled(it) => Self::Labeled(
                <LabeledStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Break(it) => Self::Break(<BreakStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Continue(it) => Self::Continue(
                <ContinueStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::If(it) => Self::If(<IfStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Switch(it) => Self::Switch(<SwitchStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Throw(it) => Self::Throw(<ThrowStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Try(it) => Self::Try(<TryStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::While(it) => Self::While(<WhileStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::DoWhile(it) => Self::DoWhile(
                <DoWhileStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::For(it) => Self::For(<ForStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::ForIn(it) => Self::ForIn(<ForInStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::ForOf(it) => Self::ForOf(<ForOfStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Expr(it) => Self::Expr(<ExprStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExprStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_expr_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for EmptyStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_empty_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for DebuggerStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_debugger_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for WithStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_with_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.obj(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_obj(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ReturnStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_return_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.arg(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_arg(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for LabeledStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_labeled_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.label(visitor.ast());
        let new_node = <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_label(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BreakStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_break_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.label(visitor.ast());
        let new_node = <Option<Ident> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_label(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ContinueStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_continue_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.label(visitor.ast());
        let new_node = <Option<Ident> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_label(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for IfStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_if_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.test(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_test(visitor.ast(), new_node);
        let field_value = self.cons(visitor.ast());
        let new_node = <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_cons(visitor.ast(), new_node);
        let field_value = self.alt(visitor.ast());
        let new_node = <Option<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_alt(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SwitchStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_switch_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.discriminant(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_discriminant(visitor.ast(), new_node);
        let field_value = self.cases(visitor.ast());
        let new_node = <TypedSubRange<SwitchCase> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_cases(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ThrowStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_throw_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.arg(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_arg(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TryStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_try_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.block(visitor.ast());
        let new_node = <BlockStmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_block(visitor.ast(), new_node);
        let field_value = self.handler(visitor.ast());
        let new_node =
            <Option<CatchClause> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_handler(visitor.ast(), new_node);
        let field_value = self.finalizer(visitor.ast());
        let new_node =
            <Option<BlockStmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_finalizer(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for WhileStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_while_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.test(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_test(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for DoWhileStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_do_while_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.test(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_test(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_for_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.init(visitor.ast());
        let new_node =
            <Option<VarDeclOrExpr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_init(visitor.ast(), new_node);
        let field_value = self.test(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_test(visitor.ast(), new_node);
        let field_value = self.update(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_update(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForInStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_for_in_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.left(visitor.ast());
        let new_node = <ForHead as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_left(visitor.ast(), new_node);
        let field_value = self.right(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_right(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForOfStmt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_for_of_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.left(visitor.ast());
        let new_node = <ForHead as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_left(visitor.ast(), new_node);
        let field_value = self.right(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_right(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <Stmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SwitchCase {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_switch_case(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.test(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_test(visitor.ast(), new_node);
        let field_value = self.cons(visitor.ast());
        let new_node =
            <TypedSubRange<Stmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_cons(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for CatchClause {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_catch_clause(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.param(visitor.ast());
        let new_node = <Option<Pat> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_param(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <BlockStmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForHead {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_for_head(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::ExprThis(it) => Self::ExprThis(
                <ThisExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprArray(it) => Self::ExprArray(
                <ArrayLit as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprObject(it) => Self::ExprObject(
                <ObjectLit as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprFn(it) => {
                Self::ExprFn(<FnExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprUnary(it) => Self::ExprUnary(
                <UnaryExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprUpdate(it) => Self::ExprUpdate(
                <UpdateExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprBin(it) => Self::ExprBin(<BinExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::ExprAssign(it) => Self::ExprAssign(
                <AssignExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprMember(it) => Self::ExprMember(
                <MemberExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprSuperProp(it) => {
                Self::ExprSuperProp(<SuperPropExpr as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::ExprCond(it) => Self::ExprCond(
                <CondExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprImport(it) => Self::ExprImport(
                <ImportExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprCall(it) => Self::ExprCall(
                <CallExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprNew(it) => Self::ExprNew(<NewExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::ExprSeq(it) => Self::ExprSeq(<SeqExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::ExprIdent(it) => {
                Self::ExprIdent(<Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprStr(it) => {
                Self::ExprStr(<Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprBool(it) => {
                Self::ExprBool(<Bool as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprNull(it) => {
                Self::ExprNull(<Null as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprNum(it) => {
                Self::ExprNum(<Number as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprBigInt(it) => {
                Self::ExprBigInt(<BigInt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprRegex(it) => {
                Self::ExprRegex(<Regex as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprTpl(it) => {
                Self::ExprTpl(<Tpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprTaggedTpl(it) => Self::ExprTaggedTpl(
                <TaggedTpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprArrow(it) => Self::ExprArrow(
                <ArrowExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprClass(it) => Self::ExprClass(
                <ClassExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprYield(it) => Self::ExprYield(
                <YieldExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprMetaProp(it) => Self::ExprMetaProp(
                <MetaPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprAwait(it) => Self::ExprAwait(
                <AwaitExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprParen(it) => Self::ExprParen(
                <ParenExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprJSXMember(it) => {
                Self::ExprJSXMember(<JSXMemberExpr as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::ExprJSXNamespacedName(it) => Self::ExprJSXNamespacedName(
                <JSXNamespacedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprJSXEmpty(it) => Self::ExprJSXEmpty(
                <JSXEmptyExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprJSXElement(it) => Self::ExprJSXElement(
                <JSXElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprJSXFragment(it) => {
                Self::ExprJSXFragment(<JSXFragment as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::ExprPrivateName(it) => {
                Self::ExprPrivateName(<PrivateName as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::ExprOptChain(it) => Self::ExprOptChain(
                <OptChainExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprInvalid(it) => Self::ExprInvalid(
                <Invalid as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Ident(it) => Self::Ident(<BindingIdent as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Array(it) => Self::Array(<ArrayPat as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Rest(it) => Self::Rest(<RestPat as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Object(it) => Self::Object(<ObjectPat as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Assign(it) => Self::Assign(<AssignPat as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Invalid(it) => Self::Invalid(<Invalid as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::VarDecl(it) => Self::VarDecl(<VarDecl as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::UsingDecl(it) => Self::UsingDecl(
                <UsingDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for VarDeclOrExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_var_decl_or_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::This(it) => Self::This(<ThisExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Array(it) => Self::Array(<ArrayLit as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Object(it) => Self::Object(<ObjectLit as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Fn(it) => Self::Fn(<FnExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Unary(it) => Self::Unary(<UnaryExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Update(it) => Self::Update(<UpdateExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Bin(it) => Self::Bin(<BinExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Assign(it) => Self::Assign(<AssignExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Member(it) => Self::Member(<MemberExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::SuperProp(it) => Self::SuperProp(
                <SuperPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Cond(it) => Self::Cond(<CondExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Import(it) => Self::Import(<ImportExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Call(it) => Self::Call(<CallExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::New(it) => Self::New(<NewExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Seq(it) => Self::Seq(<SeqExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Ident(it) => {
                Self::Ident(<Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Str(it) => Self::Str(<Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Bool(it) => {
                Self::Bool(<Bool as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Null(it) => {
                Self::Null(<Null as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Num(it) => {
                Self::Num(<Number as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::BigInt(it) => {
                Self::BigInt(<BigInt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Regex(it) => {
                Self::Regex(<Regex as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Tpl(it) => Self::Tpl(<Tpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::TaggedTpl(it) => Self::TaggedTpl(
                <TaggedTpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Arrow(it) => Self::Arrow(<ArrowExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Class(it) => Self::Class(<ClassExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Yield(it) => Self::Yield(<YieldExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::MetaProp(it) => Self::MetaProp(
                <MetaPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Await(it) => Self::Await(<AwaitExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Paren(it) => Self::Paren(<ParenExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::JSXMember(it) => Self::JSXMember(
                <JSXMemberExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXNamespacedName(it) => Self::JSXNamespacedName(
                <JSXNamespacedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXEmpty(it) => Self::JSXEmpty(
                <JSXEmptyExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXElement(it) => Self::JSXElement(
                <JSXElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXFragment(it) => Self::JSXFragment(
                <JSXFragment as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::PrivateName(it) => Self::PrivateName(
                <PrivateName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::OptChain(it) => Self::OptChain(
                <OptChainExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Invalid(it) => Self::Invalid(<Invalid as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::VarDecl(it) => Self::VarDecl(<VarDecl as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Decl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Class(it) => Self::Class(<ClassDecl as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Fn(it) => Self::Fn(<FnDecl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Var(it) => Self::Var(<VarDecl as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Using(it) => Self::Using(<UsingDecl as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for FnDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_fn_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.ident(visitor.ast());
        let new_node = <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_ident(visitor.ast(), new_node);
        let field_value = self.function(visitor.ast());
        let new_node = <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_function(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_class_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.ident(visitor.ast());
        let new_node = <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_ident(visitor.ast(), new_node);
        let field_value = self.class(visitor.ast());
        let new_node = <Class as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_class(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for VarDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_var_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.decls(visitor.ast());
        let new_node = <TypedSubRange<VarDeclarator> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_decls(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for VarDeclarator {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_var_declarator(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.name(visitor.ast());
        let new_node = <Pat as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_name(visitor.ast(), new_node);
        let field_value = self.init(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_init(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for UsingDecl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_using_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.decls(visitor.ast());
        let new_node = <TypedSubRange<VarDeclarator> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_decls(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Expr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::This(it) => Self::This(<ThisExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Array(it) => Self::Array(<ArrayLit as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Object(it) => Self::Object(<ObjectLit as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Fn(it) => Self::Fn(<FnExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Unary(it) => Self::Unary(<UnaryExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Update(it) => Self::Update(<UpdateExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Bin(it) => Self::Bin(<BinExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Assign(it) => Self::Assign(<AssignExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Member(it) => Self::Member(<MemberExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::SuperProp(it) => Self::SuperProp(
                <SuperPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Cond(it) => Self::Cond(<CondExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Import(it) => Self::Import(<ImportExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Call(it) => Self::Call(<CallExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::New(it) => Self::New(<NewExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Seq(it) => Self::Seq(<SeqExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Ident(it) => {
                Self::Ident(<Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Str(it) => Self::Str(<Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Bool(it) => {
                Self::Bool(<Bool as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Null(it) => {
                Self::Null(<Null as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Num(it) => {
                Self::Num(<Number as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::BigInt(it) => {
                Self::BigInt(<BigInt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Regex(it) => {
                Self::Regex(<Regex as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Tpl(it) => Self::Tpl(<Tpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::TaggedTpl(it) => Self::TaggedTpl(
                <TaggedTpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Arrow(it) => Self::Arrow(<ArrowExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Class(it) => Self::Class(<ClassExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Yield(it) => Self::Yield(<YieldExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::MetaProp(it) => Self::MetaProp(
                <MetaPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Await(it) => Self::Await(<AwaitExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Paren(it) => Self::Paren(<ParenExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::JSXMember(it) => Self::JSXMember(
                <JSXMemberExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXNamespacedName(it) => Self::JSXNamespacedName(
                <JSXNamespacedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXEmpty(it) => Self::JSXEmpty(
                <JSXEmptyExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXElement(it) => Self::JSXElement(
                <JSXElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXFragment(it) => Self::JSXFragment(
                <JSXFragment as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::PrivateName(it) => Self::PrivateName(
                <PrivateName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::OptChain(it) => Self::OptChain(
                <OptChainExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Invalid(it) => Self::Invalid(<Invalid as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ThisExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_this_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ArrayLit {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_array_lit(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.elems(visitor.ast());
        let new_node = <TypedSubRange<Option<ExprOrSpread>> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_elems(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ObjectLit {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_object_lit(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.props(visitor.ast());
        let new_node = <TypedSubRange<PropOrSpread> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_props(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PropOrSpread {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_prop_or_spread(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Shorthand(it) => {
                Self::Shorthand(<Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::KeyValue(it) => Self::KeyValue(
                <KeyValueProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Assign(it) => Self::Assign(<AssignProp as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Getter(it) => Self::Getter(<GetterProp as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Setter(it) => Self::Setter(<SetterProp as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Method(it) => Self::Method(<MethodProp as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Spread(it) => Self::Spread(
                <SpreadElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SpreadElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_spread_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for UnaryExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_unary_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.arg(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_arg(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for UpdateExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_update_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.arg(visitor.ast());
        let new_node =
            <SimpleAssignTarget as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_arg(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BinExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_bin_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.left(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_left(visitor.ast(), new_node);
        let field_value = self.right(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_right(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for FnExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_fn_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.ident(visitor.ast());
        let new_node = <Option<Ident> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_ident(visitor.ast(), new_node);
        let field_value = self.function(visitor.ast());
        let new_node = <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_function(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_class_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.ident(visitor.ast());
        let new_node = <Option<Ident> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_ident(visitor.ast(), new_node);
        let field_value = self.class(visitor.ast());
        let new_node = <Class as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_class(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_assign_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.left(visitor.ast());
        let new_node = <AssignTarget as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_left(visitor.ast(), new_node);
        let field_value = self.right(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_right(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MemberExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_member_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.obj(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_obj(visitor.ast(), new_node);
        let field_value = self.prop(visitor.ast());
        let new_node = <MemberProp as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_prop(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MemberProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_member_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => Self::Ident(<IdentName as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::PrivateName(it) => Self::PrivateName(
                <PrivateName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Computed(it) => Self::Computed(
                <ComputedPropName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SuperPropExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_super_prop_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.obj(visitor.ast());
        let new_node = <Super as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_obj(visitor.ast(), new_node);
        let field_value = self.prop(visitor.ast());
        let new_node = <SuperProp as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_prop(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SuperProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_super_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => Self::Ident(<IdentName as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Computed(it) => Self::Computed(
                <ComputedPropName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for CondExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_cond_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.test(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_test(visitor.ast(), new_node);
        let field_value = self.cons(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_cons(visitor.ast(), new_node);
        let field_value = self.alt(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_alt(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_import_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.source(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_source(visitor.ast(), new_node);
        let field_value = self.options(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_options(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for CallExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_call_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.callee(visitor.ast());
        let new_node = <Callee as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_callee(visitor.ast(), new_node);
        let field_value = self.args(visitor.ast());
        let new_node = <TypedSubRange<ExprOrSpread> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_args(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for NewExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_new_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.callee(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_callee(visitor.ast(), new_node);
        let field_value = self.args(visitor.ast());
        let new_node = <TypedSubRange<ExprOrSpread> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_args(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SeqExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_seq_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.exprs(visitor.ast());
        let new_node =
            <TypedSubRange<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_exprs(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ArrowExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_arrow_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.params(visitor.ast());
        let new_node =
            <TypedSubRange<Pat> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_params(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node =
            <BlockStmtOrExpr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for YieldExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_yield_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.arg(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_arg(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MetaPropExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_meta_prop_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AwaitExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_await_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.arg(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_arg(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Tpl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_tpl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.exprs(visitor.ast());
        let new_node =
            <TypedSubRange<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_exprs(visitor.ast(), new_node);
        let field_value = self.quasis(visitor.ast());
        let new_node = <TypedSubRange<TplElement> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_quasis(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TaggedTpl {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_tagged_tpl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.tag(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_tag(visitor.ast(), new_node);
        let field_value = self.tpl(visitor.ast());
        let new_node = <Tpl as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_tpl(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TplElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_tpl_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ParenExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_paren_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Callee {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_callee(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::This(it) => Self::This(<ThisExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Array(it) => Self::Array(<ArrayLit as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Object(it) => Self::Object(<ObjectLit as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Fn(it) => Self::Fn(<FnExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Unary(it) => Self::Unary(<UnaryExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Update(it) => Self::Update(<UpdateExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Bin(it) => Self::Bin(<BinExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Assign(it) => Self::Assign(<AssignExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Member(it) => Self::Member(<MemberExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::SuperProp(it) => Self::SuperProp(
                <SuperPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Cond(it) => Self::Cond(<CondExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Import(it) => Self::Import(<ImportExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Call(it) => Self::Call(<CallExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::New(it) => Self::New(<NewExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Seq(it) => Self::Seq(<SeqExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Ident(it) => {
                Self::Ident(<Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Str(it) => Self::Str(<Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Bool(it) => {
                Self::Bool(<Bool as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Null(it) => {
                Self::Null(<Null as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Num(it) => {
                Self::Num(<Number as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::BigInt(it) => {
                Self::BigInt(<BigInt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Regex(it) => {
                Self::Regex(<Regex as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Tpl(it) => Self::Tpl(<Tpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::TaggedTpl(it) => Self::TaggedTpl(
                <TaggedTpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Arrow(it) => Self::Arrow(<ArrowExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Class(it) => Self::Class(<ClassExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Yield(it) => Self::Yield(<YieldExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::MetaProp(it) => Self::MetaProp(
                <MetaPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Await(it) => Self::Await(<AwaitExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Paren(it) => Self::Paren(<ParenExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::JSXMember(it) => Self::JSXMember(
                <JSXMemberExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXNamespacedName(it) => Self::JSXNamespacedName(
                <JSXNamespacedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXEmpty(it) => Self::JSXEmpty(
                <JSXEmptyExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXElement(it) => Self::JSXElement(
                <JSXElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXFragment(it) => Self::JSXFragment(
                <JSXFragment as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::PrivateName(it) => Self::PrivateName(
                <PrivateName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::OptChain(it) => Self::OptChain(
                <OptChainExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Invalid(it) => Self::Invalid(<Invalid as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Super(it) => {
                Self::Super(<Super as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Super {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_super(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExprOrSpread {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_expr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BlockStmtOrExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_block_stmt_or_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::This(it) => Self::This(<ThisExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Array(it) => Self::Array(<ArrayLit as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Object(it) => Self::Object(<ObjectLit as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Fn(it) => Self::Fn(<FnExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Unary(it) => Self::Unary(<UnaryExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Update(it) => Self::Update(<UpdateExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Bin(it) => Self::Bin(<BinExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Assign(it) => Self::Assign(<AssignExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Member(it) => Self::Member(<MemberExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::SuperProp(it) => Self::SuperProp(
                <SuperPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Cond(it) => Self::Cond(<CondExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Import(it) => Self::Import(<ImportExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Call(it) => Self::Call(<CallExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::New(it) => Self::New(<NewExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Seq(it) => Self::Seq(<SeqExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Ident(it) => {
                Self::Ident(<Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Str(it) => Self::Str(<Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Bool(it) => {
                Self::Bool(<Bool as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Null(it) => {
                Self::Null(<Null as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Num(it) => {
                Self::Num(<Number as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::BigInt(it) => {
                Self::BigInt(<BigInt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Regex(it) => {
                Self::Regex(<Regex as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Tpl(it) => Self::Tpl(<Tpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::TaggedTpl(it) => Self::TaggedTpl(
                <TaggedTpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Arrow(it) => Self::Arrow(<ArrowExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Class(it) => Self::Class(<ClassExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Yield(it) => Self::Yield(<YieldExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::MetaProp(it) => Self::MetaProp(
                <MetaPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Await(it) => Self::Await(<AwaitExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Paren(it) => Self::Paren(<ParenExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::JSXMember(it) => Self::JSXMember(
                <JSXMemberExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXNamespacedName(it) => Self::JSXNamespacedName(
                <JSXNamespacedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXEmpty(it) => Self::JSXEmpty(
                <JSXEmptyExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXElement(it) => Self::JSXElement(
                <JSXElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXFragment(it) => Self::JSXFragment(
                <JSXFragment as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::PrivateName(it) => Self::PrivateName(
                <PrivateName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::OptChain(it) => Self::OptChain(
                <OptChainExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Invalid(it) => Self::Invalid(<Invalid as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::BlockStmt(it) => Self::BlockStmt(
                <BlockStmt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignTarget {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_assign_target(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::SimpleIdent(it) => Self::SimpleIdent(
                <BindingIdent as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::SimpleMember(it) => Self::SimpleMember(
                <MemberExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::SimpleSuperProp(it) => {
                Self::SimpleSuperProp(<SuperPropExpr as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::SimpleParen(it) => Self::SimpleParen(
                <ParenExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::SimpleOptChain(it) => {
                Self::SimpleOptChain(<OptChainExpr as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::SimpleInvalid(it) => Self::SimpleInvalid(
                <Invalid as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::PatArray(it) => Self::PatArray(
                <ArrayPat as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::PatObject(it) => Self::PatObject(
                <ObjectPat as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::PatternInvalid(it) => Self::PatternInvalid(
                <Invalid as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignTargetPat {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_assign_target_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Array(it) => Self::Array(<ArrayPat as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Object(it) => Self::Object(<ObjectPat as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Invalid(it) => Self::Invalid(<Invalid as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SimpleAssignTarget {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_simple_assign_target(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => Self::Ident(<BindingIdent as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Member(it) => Self::Member(<MemberExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::SuperProp(it) => Self::SuperProp(
                <SuperPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Paren(it) => Self::Paren(<ParenExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::OptChain(it) => Self::OptChain(
                <OptChainExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Invalid(it) => Self::Invalid(<Invalid as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for OptChainExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_chain_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.base(visitor.ast());
        let new_node = <OptChainBase as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_base(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for OptChainBase {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_chain_base(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Member(it) => Self::Member(<MemberExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Call(it) => Self::Call(<OptCall as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for OptCall {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_call(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.callee(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_callee(visitor.ast(), new_node);
        let field_value = self.args(visitor.ast());
        let new_node = <TypedSubRange<ExprOrSpread> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_args(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Invalid {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_invalid(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Function {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_function(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.params(visitor.ast());
        let new_node =
            <TypedSubRange<Param> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_params(visitor.ast(), new_node);
        let field_value = self.decorators(visitor.ast());
        let new_node =
            <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_decorators(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <BlockStmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Param {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_param(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.decorators(visitor.ast());
        let new_node =
            <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_decorators(visitor.ast(), new_node);
        let field_value = self.pat(visitor.ast());
        let new_node = <Pat as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_pat(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ParamOrTsParamProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_param_or_ts_param_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Param(it) => {
                Self::Param(<Param as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Class {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_class(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.decorators(visitor.ast());
        let new_node =
            <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_decorators(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node = <TypedSubRange<ClassMember> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_body(visitor.ast(), new_node);
        let field_value = self.super_class(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_super_class(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassMember {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_class_member(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Constructor(it) => Self::Constructor(
                <Constructor as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Method(it) => Self::Method(<ClassMethod as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::PrivateMethod(it) => {
                Self::PrivateMethod(<PrivateMethod as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::ClassProp(it) => Self::ClassProp(
                <ClassProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::PrivateProp(it) => Self::PrivateProp(
                <PrivateProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Empty(it) => Self::Empty(<EmptyStmt as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::StaticBlock(it) => Self::StaticBlock(
                <StaticBlock as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::AutoAccessor(it) => Self::AutoAccessor(
                <AutoAccessor as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_class_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        let field_value = self.decorators(visitor.ast());
        let new_node =
            <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_decorators(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PrivateProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_private_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PrivateName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        let field_value = self.decorators(visitor.ast());
        let new_node =
            <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_decorators(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassMethod {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_class_method(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.function(visitor.ast());
        let new_node = <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_function(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PrivateMethod {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_private_method(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PrivateName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.function(visitor.ast());
        let new_node = <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_function(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Constructor {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_constructor(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.params(visitor.ast());
        let new_node = <TypedSubRange<ParamOrTsParamProp> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_params(visitor.ast(), new_node);
        let field_value = self.body(visitor.ast());
        let new_node =
            <Option<BlockStmt> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Decorator {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_decorator(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for StaticBlock {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_static_block(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.body(visitor.ast());
        let new_node = <BlockStmt as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_body(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Key {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_key(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => Self::Ident(<IdentName as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Str(it) => Self::Str(<Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Num(it) => {
                Self::Num(<Number as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Computed(it) => Self::Computed(
                <ComputedPropName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::BigInt(it) => {
                Self::BigInt(<BigInt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Private(it) => Self::Private(
                <PrivateName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AutoAccessor {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_auto_accessor(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <Key as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        let field_value = self.decorators(visitor.ast());
        let new_node =
            <TypedSubRange<Decorator> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_decorators(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Prop {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Shorthand(it) => {
                Self::Shorthand(<Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::KeyValue(it) => Self::KeyValue(
                <KeyValueProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Assign(it) => Self::Assign(<AssignProp as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Getter(it) => Self::Getter(<GetterProp as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Setter(it) => Self::Setter(<SetterProp as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Method(it) => Self::Method(<MethodProp as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for KeyValueProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_key_value_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_assign_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for GetterProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_getter_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.function(visitor.ast());
        let new_node = <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_function(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SetterProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_setter_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.function(visitor.ast());
        let new_node = <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_function(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MethodProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_method_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.function(visitor.ast());
        let new_node = <Function as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_function(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PropName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_prop_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => Self::Ident(<IdentName as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Str(it) => Self::Str(<Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Num(it) => {
                Self::Num(<Number as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Computed(it) => Self::Computed(
                <ComputedPropName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::BigInt(it) => {
                Self::BigInt(<BigInt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ComputedPropName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_computed_prop_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Pat {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::ExprThis(it) => Self::ExprThis(
                <ThisExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprArray(it) => Self::ExprArray(
                <ArrayLit as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprObject(it) => Self::ExprObject(
                <ObjectLit as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprFn(it) => {
                Self::ExprFn(<FnExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprUnary(it) => Self::ExprUnary(
                <UnaryExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprUpdate(it) => Self::ExprUpdate(
                <UpdateExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprBin(it) => Self::ExprBin(<BinExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::ExprAssign(it) => Self::ExprAssign(
                <AssignExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprMember(it) => Self::ExprMember(
                <MemberExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprSuperProp(it) => {
                Self::ExprSuperProp(<SuperPropExpr as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::ExprCond(it) => Self::ExprCond(
                <CondExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprImport(it) => Self::ExprImport(
                <ImportExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprCall(it) => Self::ExprCall(
                <CallExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprNew(it) => Self::ExprNew(<NewExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::ExprSeq(it) => Self::ExprSeq(<SeqExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::ExprIdent(it) => {
                Self::ExprIdent(<Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprStr(it) => {
                Self::ExprStr(<Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprBool(it) => {
                Self::ExprBool(<Bool as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprNull(it) => {
                Self::ExprNull(<Null as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprNum(it) => {
                Self::ExprNum(<Number as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprBigInt(it) => {
                Self::ExprBigInt(<BigInt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprRegex(it) => {
                Self::ExprRegex(<Regex as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprTpl(it) => {
                Self::ExprTpl(<Tpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::ExprTaggedTpl(it) => Self::ExprTaggedTpl(
                <TaggedTpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprArrow(it) => Self::ExprArrow(
                <ArrowExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprClass(it) => Self::ExprClass(
                <ClassExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprYield(it) => Self::ExprYield(
                <YieldExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprMetaProp(it) => Self::ExprMetaProp(
                <MetaPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprAwait(it) => Self::ExprAwait(
                <AwaitExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprParen(it) => Self::ExprParen(
                <ParenExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprJSXMember(it) => {
                Self::ExprJSXMember(<JSXMemberExpr as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::ExprJSXNamespacedName(it) => Self::ExprJSXNamespacedName(
                <JSXNamespacedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprJSXEmpty(it) => Self::ExprJSXEmpty(
                <JSXEmptyExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprJSXElement(it) => Self::ExprJSXElement(
                <JSXElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprJSXFragment(it) => {
                Self::ExprJSXFragment(<JSXFragment as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::ExprPrivateName(it) => {
                Self::ExprPrivateName(<PrivateName as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::ExprOptChain(it) => Self::ExprOptChain(
                <OptChainExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::ExprInvalid(it) => Self::ExprInvalid(
                <Invalid as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Ident(it) => Self::Ident(<BindingIdent as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Array(it) => Self::Array(<ArrayPat as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Rest(it) => Self::Rest(<RestPat as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Object(it) => Self::Object(<ObjectPat as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Assign(it) => Self::Assign(<AssignPat as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Invalid(it) => Self::Invalid(<Invalid as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ArrayPat {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_array_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.elems(visitor.ast());
        let new_node = <TypedSubRange<Option<Pat>> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_elems(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ObjectPat {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_object_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.props(visitor.ast());
        let new_node = <TypedSubRange<ObjectPatProp> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_props(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignPat {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_assign_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.left(visitor.ast());
        let new_node = <Pat as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_left(visitor.ast(), new_node);
        let field_value = self.right(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_right(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for RestPat {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_rest_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.arg(visitor.ast());
        let new_node = <Pat as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_arg(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ObjectPatProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_object_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::KeyValue(it) => Self::KeyValue(
                <KeyValuePatProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Assign(it) => Self::Assign(
                <AssignPatProp as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Rest(it) => Self::Rest(<RestPat as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for KeyValuePatProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_key_value_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <PropName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node = <Pat as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignPatProp {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_assign_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.key(visitor.ast());
        let new_node = <BindingIdent as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_key(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node = <Option<Expr> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Ident {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_ident(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for IdentName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_ident_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PrivateName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_private_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BindingIdent {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_binding_ident(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.id(visitor.ast());
        let new_node = <Ident as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_id(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Lit {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_lit(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Str(it) => Self::Str(<Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Bool(it) => {
                Self::Bool(<Bool as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Null(it) => {
                Self::Null(<Null as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Num(it) => {
                Self::Num(<Number as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::BigInt(it) => {
                Self::BigInt(<BigInt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Regex(it) => {
                Self::Regex(<Regex as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Str {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_str(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Bool {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_bool(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Null {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_null(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Number {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_number(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BigInt {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_big_int(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Regex {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_regex(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXObject {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_object(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::JSXMemberExpr(it) => {
                Self::JSXMemberExpr(<JSXMemberExpr as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::Ident(it) => {
                Self::Ident(<Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXMemberExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_member_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.obj(visitor.ast());
        let new_node = <JSXObject as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_obj(visitor.ast(), new_node);
        let field_value = self.prop(visitor.ast());
        let new_node = <IdentName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_prop(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXNamespacedName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_namespaced_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.ns(visitor.ast());
        let new_node = <IdentName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_ns(visitor.ast(), new_node);
        let field_value = self.name(visitor.ast());
        let new_node = <IdentName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_name(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXEmptyExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_empty_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXExprContainer {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_expr_container(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <JSXExpr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXExpr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::This(it) => Self::This(<ThisExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Array(it) => Self::Array(<ArrayLit as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Object(it) => Self::Object(<ObjectLit as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Fn(it) => Self::Fn(<FnExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Unary(it) => Self::Unary(<UnaryExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Update(it) => Self::Update(<UpdateExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Bin(it) => Self::Bin(<BinExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Assign(it) => Self::Assign(<AssignExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Member(it) => Self::Member(<MemberExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::SuperProp(it) => Self::SuperProp(
                <SuperPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Cond(it) => Self::Cond(<CondExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Import(it) => Self::Import(<ImportExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Call(it) => Self::Call(<CallExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::New(it) => Self::New(<NewExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Seq(it) => Self::Seq(<SeqExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Ident(it) => {
                Self::Ident(<Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Str(it) => Self::Str(<Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::Bool(it) => {
                Self::Bool(<Bool as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Null(it) => {
                Self::Null(<Null as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Num(it) => {
                Self::Num(<Number as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::BigInt(it) => {
                Self::BigInt(<BigInt as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Regex(it) => {
                Self::Regex(<Regex as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::Tpl(it) => Self::Tpl(<Tpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::TaggedTpl(it) => Self::TaggedTpl(
                <TaggedTpl as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Arrow(it) => Self::Arrow(<ArrowExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Class(it) => Self::Class(<ClassExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Yield(it) => Self::Yield(<YieldExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::MetaProp(it) => Self::MetaProp(
                <MetaPropExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Await(it) => Self::Await(<AwaitExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::Paren(it) => Self::Paren(<ParenExpr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::JSXMember(it) => Self::JSXMember(
                <JSXMemberExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXNamespacedName(it) => Self::JSXNamespacedName(
                <JSXNamespacedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXEmpty(it) => Self::JSXEmpty(
                <JSXEmptyExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXElement(it) => Self::JSXElement(
                <JSXElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXFragment(it) => Self::JSXFragment(
                <JSXFragment as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::PrivateName(it) => Self::PrivateName(
                <PrivateName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::OptChain(it) => Self::OptChain(
                <OptChainExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::Invalid(it) => Self::Invalid(<Invalid as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::JSXEmptyExpr(it) => Self::JSXEmptyExpr(
                <JSXEmptyExpr as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXSpreadChild {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_spread_child(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.expr(visitor.ast());
        let new_node = <Expr as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_expr(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXElementName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_element_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => {
                Self::Ident(<Ident as VisitMutWith<'a, V>>::visit_mut_with(it, visitor))
            }
            Self::JSXMemberExpr(it) => {
                Self::JSXMemberExpr(<JSXMemberExpr as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::JSXNamespacedName(it) => Self::JSXNamespacedName(
                <JSXNamespacedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXOpeningElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_opening_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.name(visitor.ast());
        let new_node =
            <JSXElementName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_name(visitor.ast(), new_node);
        let field_value = self.attrs(visitor.ast());
        let new_node = <TypedSubRange<JSXAttrOrSpread> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_attrs(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXAttrOrSpread {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_attr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::JSXAttr(it) => Self::JSXAttr(<JSXAttr as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::SpreadElement(it) => {
                Self::SpreadElement(<SpreadElement as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXClosingElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_closing_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.name(visitor.ast());
        let new_node =
            <JSXElementName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_name(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXAttr {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_attr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.name(visitor.ast());
        let new_node = <JSXAttrName as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_name(visitor.ast(), new_node);
        let field_value = self.value(visitor.ast());
        let new_node =
            <Option<JSXAttrValue> as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_value(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXAttrName {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_attr_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Ident(it) => Self::Ident(<IdentName as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::JSXNamespacedName(it) => Self::JSXNamespacedName(
                <JSXNamespacedName as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXAttrValue {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_attr_value(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::Str(it) => Self::Str(<Str as VisitMutWith<'a, V>>::visit_mut_with(it, visitor)),
            Self::JSXExprContainer(it) => Self::JSXExprContainer(
                <JSXExprContainer as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXElement(it) => Self::JSXElement(
                <JSXElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXFragment(it) => Self::JSXFragment(
                <JSXFragment as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXText {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_text(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXElement {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.opening(visitor.ast());
        let new_node =
            <JSXOpeningElement as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_opening(visitor.ast(), new_node);
        let field_value = self.children(visitor.ast());
        let new_node = <TypedSubRange<JSXElementChild> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_children(visitor.ast(), new_node);
        let field_value = self.closing(visitor.ast());
        let new_node = <Option<JSXClosingElement> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_closing(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXElementChild {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_element_child(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Self::JSXText(it) => Self::JSXText(<JSXText as VisitMutWith<'a, V>>::visit_mut_with(
                it, visitor,
            )),
            Self::JSXExprContainer(it) => Self::JSXExprContainer(
                <JSXExprContainer as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXSpreadChild(it) => {
                Self::JSXSpreadChild(<JSXSpreadChild as VisitMutWith<'a, V>>::visit_mut_with(
                    it, visitor,
                ))
            }
            Self::JSXElement(it) => Self::JSXElement(
                <JSXElement as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
            Self::JSXFragment(it) => Self::JSXFragment(
                <JSXFragment as VisitMutWith<'a, V>>::visit_mut_with(it, visitor),
            ),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXFragment {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_fragment(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        let field_value = self.opening(visitor.ast());
        let new_node =
            <JSXOpeningFragment as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_opening(visitor.ast(), new_node);
        let field_value = self.children(visitor.ast());
        let new_node = <TypedSubRange<JSXElementChild> as VisitMutWith<'a, V>>::visit_mut_with(
            field_value,
            visitor,
        );
        self.set_children(visitor.ast(), new_node);
        let field_value = self.closing(visitor.ast());
        let new_node =
            <JSXClosingFragment as VisitMutWith<'a, V>>::visit_mut_with(field_value, visitor);
        self.set_closing(visitor.ast(), new_node);
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXOpeningFragment {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_opening_fragment(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXClosingFragment {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_closing_fragment(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ModuleItem> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_module_items(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Stmt> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_stmts(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ImportSpecifier> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_import_specifiers(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<ObjectLit> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_object_lit(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ExportSpecifier> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_export_specifiers(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Str> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_str(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Expr> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Ident> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_ident(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Stmt> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<SwitchCase> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_switch_cases(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<CatchClause> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_catch_clause(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<BlockStmt> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_block_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<VarDeclOrExpr> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_var_decl_or_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Pat> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<VarDeclarator> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_var_declarators(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<ExprOrSpread> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_expr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Option<ExprOrSpread>> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_vec_expr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<PropOrSpread> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_prop_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ExprOrSpread> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_expr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Expr> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_exprs(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Pat> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_pats(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<TplElement> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_tpl_elements(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Param> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_params(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Decorator> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_decorators(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ClassMember> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_class_members(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ParamOrTsParamProp> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_param_or_ts_param_props(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<Option<Pat>> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_vec_pats(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<ObjectPatProp> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_object_pat_props(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<JSXAttrOrSpread> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_attr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<JSXAttrValue> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_jsx_attr_value(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TypedSubRange<JSXElementChild> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_jsx_element_childs(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        for child_idx in self.iter() {
            let child = visitor.ast().get_node_in_sub_range(child_idx);
            let new_child = child.visit_mut_with(visitor);
            self.replace_slot(visitor.ast(), child_idx, new_child);
        }
        self
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<JSXClosingElement> {
    #[inline]
    fn visit_mut_with(self, visitor: &mut V) -> Self {
        <V as VisitMut<'a>>::visit_mut_opt_jsx_closing_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(self, visitor: &mut V) -> Self {
        match self {
            Some(it) => Some(it.visit_mut_with(visitor)),
            None => None,
        }
    }
}
