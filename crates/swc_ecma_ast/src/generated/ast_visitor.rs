#![allow(unused, clippy::useless_conversion, clippy::single_match)]
use crate::*;
use swc_experimental_allocator::atom::{Atom, Wtf8Atom};
use swc_experimental_allocator::boxed::Box;
use swc_experimental_allocator::vec::Vec;
pub trait Visit<'a> {
    #[inline]
    fn visit_program(&mut self, node: &Program<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_module(&mut self, node: &Module<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_script(&mut self, node: &Script<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_module_item(&mut self, node: &ModuleItem<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_module_decl(&mut self, node: &ModuleDecl<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_import_decl(&mut self, node: &ImportDecl<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_import_specifier(&mut self, node: &ImportSpecifier<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_import_named_specifier(&mut self, node: &ImportNamedSpecifier<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_import_default_specifier(&mut self, node: &ImportDefaultSpecifier<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_import_star_as_specifier(&mut self, node: &ImportStarAsSpecifier<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_export_decl(&mut self, node: &ExportDecl<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_named_export(&mut self, node: &NamedExport<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_export_specifier(&mut self, node: &ExportSpecifier<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_export_namespace_specifier(&mut self, node: &ExportNamespaceSpecifier<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_module_export_name(&mut self, node: &ModuleExportName<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_export_default_specifier(&mut self, node: &ExportDefaultSpecifier<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_export_named_specifier(&mut self, node: &ExportNamedSpecifier<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_export_default_decl(&mut self, node: &ExportDefaultDecl<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_default_decl(&mut self, node: &DefaultDecl<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_export_default_expr(&mut self, node: &ExportDefaultExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_export_all(&mut self, node: &ExportAll<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_block_stmt(&mut self, node: &BlockStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_stmt(&mut self, node: &Stmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_expr_stmt(&mut self, node: &ExprStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_empty_stmt(&mut self, node: &EmptyStmt) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_debugger_stmt(&mut self, node: &DebuggerStmt) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_with_stmt(&mut self, node: &WithStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_return_stmt(&mut self, node: &ReturnStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_labeled_stmt(&mut self, node: &LabeledStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_break_stmt(&mut self, node: &BreakStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_continue_stmt(&mut self, node: &ContinueStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_if_stmt(&mut self, node: &IfStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_switch_stmt(&mut self, node: &SwitchStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_throw_stmt(&mut self, node: &ThrowStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_try_stmt(&mut self, node: &TryStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_while_stmt(&mut self, node: &WhileStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_do_while_stmt(&mut self, node: &DoWhileStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_for_stmt(&mut self, node: &ForStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_for_in_stmt(&mut self, node: &ForInStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_for_of_stmt(&mut self, node: &ForOfStmt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_switch_case(&mut self, node: &SwitchCase<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_catch_clause(&mut self, node: &CatchClause<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_for_head(&mut self, node: &ForHead<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_var_decl_or_expr(&mut self, node: &VarDeclOrExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_decl(&mut self, node: &Decl<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_fn_decl(&mut self, node: &FnDecl<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_class_decl(&mut self, node: &ClassDecl<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_var_decl(&mut self, node: &VarDecl<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_var_declarator(&mut self, node: &VarDeclarator<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_using_decl(&mut self, node: &UsingDecl<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_expr(&mut self, node: &Expr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_this_expr(&mut self, node: &ThisExpr) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_array_lit(&mut self, node: &ArrayLit<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_object_lit(&mut self, node: &ObjectLit<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_prop_or_spread(&mut self, node: &PropOrSpread<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_spread_element(&mut self, node: &SpreadElement<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_unary_expr(&mut self, node: &UnaryExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_update_expr(&mut self, node: &UpdateExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_bin_expr(&mut self, node: &BinExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_fn_expr(&mut self, node: &FnExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_class_expr(&mut self, node: &ClassExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_assign_expr(&mut self, node: &AssignExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_member_expr(&mut self, node: &MemberExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_member_prop(&mut self, node: &MemberProp<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_super_prop_expr(&mut self, node: &SuperPropExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_super_prop(&mut self, node: &SuperProp<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_cond_expr(&mut self, node: &CondExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_call_expr(&mut self, node: &CallExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_new_expr(&mut self, node: &NewExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_seq_expr(&mut self, node: &SeqExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_arrow_expr(&mut self, node: &ArrowExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_yield_expr(&mut self, node: &YieldExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_meta_prop_expr(&mut self, node: &MetaPropExpr) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_await_expr(&mut self, node: &AwaitExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_tpl(&mut self, node: &Tpl<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_tagged_tpl(&mut self, node: &TaggedTpl<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_tpl_element(&mut self, node: &TplElement<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_paren_expr(&mut self, node: &ParenExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_callee(&mut self, node: &Callee<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_super(&mut self, node: &Super) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_import(&mut self, node: &Import) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_expr_or_spread(&mut self, node: &ExprOrSpread<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_block_stmt_or_expr(&mut self, node: &BlockStmtOrExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_assign_target(&mut self, node: &AssignTarget<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_assign_target_pat(&mut self, node: &AssignTargetPat<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_simple_assign_target(&mut self, node: &SimpleAssignTarget<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_chain_expr(&mut self, node: &OptChainExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_chain_base(&mut self, node: &OptChainBase<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_call(&mut self, node: &OptCall<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_invalid(&mut self, node: &Invalid) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_function(&mut self, node: &Function<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_param(&mut self, node: &Param<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_param_or_ts_param_prop(&mut self, node: &ParamOrTsParamProp<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_class(&mut self, node: &Class<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_class_member(&mut self, node: &ClassMember<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_class_prop(&mut self, node: &ClassProp<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_private_prop(&mut self, node: &PrivateProp<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_class_method(&mut self, node: &ClassMethod<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_private_method(&mut self, node: &PrivateMethod<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_constructor(&mut self, node: &Constructor<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_decorator(&mut self, node: &Decorator<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_static_block(&mut self, node: &StaticBlock<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_key(&mut self, node: &Key<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_auto_accessor(&mut self, node: &AutoAccessor<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_prop(&mut self, node: &Prop<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_key_value_prop(&mut self, node: &KeyValueProp<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_assign_prop(&mut self, node: &AssignProp<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_getter_prop(&mut self, node: &GetterProp<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_setter_prop(&mut self, node: &SetterProp<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_method_prop(&mut self, node: &MethodProp<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_prop_name(&mut self, node: &PropName<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_computed_prop_name(&mut self, node: &ComputedPropName<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_pat(&mut self, node: &Pat<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_array_pat(&mut self, node: &ArrayPat<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_object_pat(&mut self, node: &ObjectPat<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_assign_pat(&mut self, node: &AssignPat<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_rest_pat(&mut self, node: &RestPat<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_object_pat_prop(&mut self, node: &ObjectPatProp<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_key_value_pat_prop(&mut self, node: &KeyValuePatProp<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_assign_pat_prop(&mut self, node: &AssignPatProp<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_ident(&mut self, node: &Ident<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_ident_name(&mut self, node: &IdentName<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_private_name(&mut self, node: &PrivateName<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_binding_ident(&mut self, node: &BindingIdent<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_lit(&mut self, node: &Lit<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_str(&mut self, node: &Str<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_bool(&mut self, node: &Bool) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_null(&mut self, node: &Null) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_number(&mut self, node: &Number<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_big_int(&mut self, node: &BigInt<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_regex(&mut self, node: &Regex<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_object(&mut self, node: &JSXObject<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_member_expr(&mut self, node: &JSXMemberExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_namespaced_name(&mut self, node: &JSXNamespacedName<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_empty_expr(&mut self, node: &JSXEmptyExpr) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_expr_container(&mut self, node: &JSXExprContainer<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_expr(&mut self, node: &JSXExpr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_spread_child(&mut self, node: &JSXSpreadChild<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_element_name(&mut self, node: &JSXElementName<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_opening_element(&mut self, node: &JSXOpeningElement<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_attr_or_spread(&mut self, node: &JSXAttrOrSpread<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_closing_element(&mut self, node: &JSXClosingElement<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_attr(&mut self, node: &JSXAttr<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_attr_name(&mut self, node: &JSXAttrName<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_attr_value(&mut self, node: &JSXAttrValue<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_text(&mut self, node: &JSXText<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_element(&mut self, node: &JSXElement<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_element_child(&mut self, node: &JSXElementChild<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_fragment(&mut self, node: &JSXFragment<'a>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_opening_fragment(&mut self, node: &JSXOpeningFragment) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_closing_fragment(&mut self, node: &JSXClosingFragment) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_module_items(&mut self, node: &Vec<'a, ModuleItem<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_stmts(&mut self, node: &Vec<'a, Stmt<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_import_specifiers(&mut self, node: &Vec<'a, ImportSpecifier<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_object_lit(&mut self, node: &Option<Box<'a, ObjectLit<'a>>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_module_export_name(&mut self, node: &Option<ModuleExportName<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_export_specifiers(&mut self, node: &Vec<'a, ExportSpecifier<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_str(&mut self, node: &Option<Box<'a, Str<'a>>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_expr(&mut self, node: &Option<Expr<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_ident(&mut self, node: &Option<Box<'a, Ident<'a>>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_stmt(&mut self, node: &Option<Stmt<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_switch_cases(&mut self, node: &Vec<'a, SwitchCase<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_catch_clause(&mut self, node: &Option<Box<'a, CatchClause<'a>>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_block_stmt(&mut self, node: &Option<Box<'a, BlockStmt<'a>>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_var_decl_or_expr(&mut self, node: &Option<VarDeclOrExpr<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_pat(&mut self, node: &Option<Pat<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_var_declarators(&mut self, node: &Vec<'a, VarDeclarator<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_expr_or_spread(&mut self, node: &Option<Box<'a, ExprOrSpread<'a>>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_vec_expr_or_spreads(&mut self, node: &Vec<'a, Option<Box<'a, ExprOrSpread<'a>>>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_prop_or_spreads(&mut self, node: &Vec<'a, PropOrSpread<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_expr_or_spreads(&mut self, node: &Vec<'a, ExprOrSpread<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_expr_or_spreads(&mut self, node: &Option<Vec<'a, ExprOrSpread<'a>>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_exprs(&mut self, node: &Vec<'a, Expr<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_pats(&mut self, node: &Vec<'a, Pat<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_tpl_elements(&mut self, node: &Vec<'a, TplElement<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_params(&mut self, node: &Vec<'a, Param<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_decorators(&mut self, node: &Vec<'a, Decorator<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_class_members(&mut self, node: &Vec<'a, ClassMember<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_param_or_ts_param_props(&mut self, node: &Vec<'a, ParamOrTsParamProp<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_vec_pats(&mut self, node: &Vec<'a, Option<Pat<'a>>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_object_pat_props(&mut self, node: &Vec<'a, ObjectPatProp<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_attr_or_spreads(&mut self, node: &Vec<'a, JSXAttrOrSpread<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_jsx_attr_value(&mut self, node: &Option<JSXAttrValue<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_jsx_element_childs(&mut self, node: &Vec<'a, JSXElementChild<'a>>) {
        node.visit_children_with(self);
    }
    #[inline]
    fn visit_opt_jsx_closing_element(&mut self, node: &Option<Box<'a, JSXClosingElement<'a>>>) {
        node.visit_children_with(self);
    }
}
pub trait VisitWith<'a, V: ?Sized + Visit<'a>> {
    fn visit_with(&self, visitor: &mut V);
    fn visit_children_with(&self, visitor: &mut V);
}
impl<'a, T, V> VisitWith<'a, V> for Box<'a, T>
where
    T: VisitWith<'a, V>,
    V: ?Sized + Visit<'a>,
{
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        (**self).visit_with(visitor)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        (**self).visit_children_with(visitor)
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Program<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_program(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Module(it) => it.visit_with(visitor),
            Self::Script(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Module<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_module(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Script<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_script(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ModuleItem<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_module_item(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::ModuleDecl(it) => it.visit_with(visitor),
            Self::Stmt(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ModuleDecl<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_module_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Import(it) => it.visit_with(visitor),
            Self::ExportDecl(it) => it.visit_with(visitor),
            Self::ExportNamed(it) => it.visit_with(visitor),
            Self::ExportDefaultDecl(it) => it.visit_with(visitor),
            Self::ExportDefaultExpr(it) => it.visit_with(visitor),
            Self::ExportAll(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportDecl<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.specifiers.visit_with(visitor);
        self.src.visit_with(visitor);
        self.with.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportSpecifier<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Named(it) => it.visit_with(visitor),
            Self::Default(it) => it.visit_with(visitor),
            Self::Namespace(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportNamedSpecifier<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_named_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.local.visit_with(visitor);
        self.imported.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportDefaultSpecifier<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_default_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.local.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ImportStarAsSpecifier<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_star_as_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.local.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportDecl<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.decl.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for NamedExport<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_named_export(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.specifiers.visit_with(visitor);
        self.src.visit_with(visitor);
        self.with.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportSpecifier<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Namespace(it) => it.visit_with(visitor),
            Self::Default(it) => it.visit_with(visitor),
            Self::Named(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportNamespaceSpecifier<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_namespace_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.name.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ModuleExportName<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_module_export_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_with(visitor),
            Self::Str(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportDefaultSpecifier<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_default_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.exported.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportNamedSpecifier<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_named_specifier(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.orig.visit_with(visitor);
        self.exported.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportDefaultDecl<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_default_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.decl.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for DefaultDecl<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_default_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Class(it) => it.visit_with(visitor),
            Self::Fn(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportDefaultExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_default_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.expr.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExportAll<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_all(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.src.visit_with(visitor);
        self.with.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BlockStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_block_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.stmts.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Stmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Block(it) => it.visit_with(visitor),
            Self::Empty(it) => it.visit_with(visitor),
            Self::Debugger(it) => it.visit_with(visitor),
            Self::With(it) => it.visit_with(visitor),
            Self::Return(it) => it.visit_with(visitor),
            Self::Labeled(it) => it.visit_with(visitor),
            Self::Break(it) => it.visit_with(visitor),
            Self::Continue(it) => it.visit_with(visitor),
            Self::If(it) => it.visit_with(visitor),
            Self::Switch(it) => it.visit_with(visitor),
            Self::Throw(it) => it.visit_with(visitor),
            Self::Try(it) => it.visit_with(visitor),
            Self::While(it) => it.visit_with(visitor),
            Self::DoWhile(it) => it.visit_with(visitor),
            Self::For(it) => it.visit_with(visitor),
            Self::ForIn(it) => it.visit_with(visitor),
            Self::ForOf(it) => it.visit_with(visitor),
            Self::Decl(it) => it.visit_with(visitor),
            Self::Expr(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExprStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_expr_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.expr.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for EmptyStmt {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_empty_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for DebuggerStmt {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_debugger_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for WithStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_with_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.obj.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ReturnStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_return_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.arg.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for LabeledStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_labeled_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.label.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BreakStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_break_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.label.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ContinueStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_continue_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.label.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for IfStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_if_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.test.visit_with(visitor);
        self.cons.visit_with(visitor);
        self.alt.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SwitchStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_switch_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.discriminant.visit_with(visitor);
        self.cases.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ThrowStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_throw_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.arg.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TryStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_try_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.block.visit_with(visitor);
        self.handler.visit_with(visitor);
        self.finalizer.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for WhileStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_while_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.test.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for DoWhileStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_do_while_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.test.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ForStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_for_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.init.visit_with(visitor);
        self.test.visit_with(visitor);
        self.update.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ForInStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_for_in_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.left.visit_with(visitor);
        self.right.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ForOfStmt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_for_of_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.left.visit_with(visitor);
        self.right.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SwitchCase<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_switch_case(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.test.visit_with(visitor);
        self.cons.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for CatchClause<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_catch_clause(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.param.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ForHead<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_for_head(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::VarDecl(it) => it.visit_with(visitor),
            Self::UsingDecl(it) => it.visit_with(visitor),
            Self::Pat(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for VarDeclOrExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_var_decl_or_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::VarDecl(it) => it.visit_with(visitor),
            Self::Expr(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Decl<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Class(it) => it.visit_with(visitor),
            Self::Fn(it) => it.visit_with(visitor),
            Self::Var(it) => it.visit_with(visitor),
            Self::Using(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for FnDecl<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_fn_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.ident.visit_with(visitor);
        self.function.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ClassDecl<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.ident.visit_with(visitor);
        self.class.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for VarDecl<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_var_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.decls.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for VarDeclarator<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_var_declarator(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.name.visit_with(visitor);
        self.init.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for UsingDecl<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_using_decl(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.decls.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Expr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::This(it) => it.visit_with(visitor),
            Self::Array(it) => it.visit_with(visitor),
            Self::Object(it) => it.visit_with(visitor),
            Self::Fn(it) => it.visit_with(visitor),
            Self::Unary(it) => it.visit_with(visitor),
            Self::Update(it) => it.visit_with(visitor),
            Self::Bin(it) => it.visit_with(visitor),
            Self::Assign(it) => it.visit_with(visitor),
            Self::Member(it) => it.visit_with(visitor),
            Self::SuperProp(it) => it.visit_with(visitor),
            Self::Cond(it) => it.visit_with(visitor),
            Self::Call(it) => it.visit_with(visitor),
            Self::New(it) => it.visit_with(visitor),
            Self::Seq(it) => it.visit_with(visitor),
            Self::Ident(it) => it.visit_with(visitor),
            Self::Lit(it) => it.visit_with(visitor),
            Self::Tpl(it) => it.visit_with(visitor),
            Self::TaggedTpl(it) => it.visit_with(visitor),
            Self::Arrow(it) => it.visit_with(visitor),
            Self::Class(it) => it.visit_with(visitor),
            Self::Yield(it) => it.visit_with(visitor),
            Self::MetaProp(it) => it.visit_with(visitor),
            Self::Await(it) => it.visit_with(visitor),
            Self::Paren(it) => it.visit_with(visitor),
            Self::JSXMember(it) => it.visit_with(visitor),
            Self::JSXNamespacedName(it) => it.visit_with(visitor),
            Self::JSXEmpty(it) => it.visit_with(visitor),
            Self::JSXElement(it) => it.visit_with(visitor),
            Self::JSXFragment(it) => it.visit_with(visitor),
            Self::PrivateName(it) => it.visit_with(visitor),
            Self::OptChain(it) => it.visit_with(visitor),
            Self::Invalid(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ThisExpr {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_this_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ArrayLit<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_array_lit(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.elems.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ObjectLit<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_object_lit(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.props.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for PropOrSpread<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_prop_or_spread(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Spread(it) => it.visit_with(visitor),
            Self::Prop(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SpreadElement<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_spread_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.expr.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for UnaryExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_unary_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.arg.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for UpdateExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_update_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.arg.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BinExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_bin_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.left.visit_with(visitor);
        self.right.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for FnExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_fn_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.ident.visit_with(visitor);
        self.function.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ClassExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.ident.visit_with(visitor);
        self.class.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assign_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.left.visit_with(visitor);
        self.right.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for MemberExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_member_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.obj.visit_with(visitor);
        self.prop.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for MemberProp<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_member_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_with(visitor),
            Self::PrivateName(it) => it.visit_with(visitor),
            Self::Computed(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SuperPropExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_super_prop_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.obj.visit_with(visitor);
        self.prop.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SuperProp<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_super_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_with(visitor),
            Self::Computed(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for CondExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_cond_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.test.visit_with(visitor);
        self.cons.visit_with(visitor);
        self.alt.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for CallExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_call_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.callee.visit_with(visitor);
        self.args.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for NewExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_new_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.callee.visit_with(visitor);
        self.args.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SeqExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_seq_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.exprs.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ArrowExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_arrow_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.params.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for YieldExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_yield_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.arg.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for MetaPropExpr {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_meta_prop_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AwaitExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_await_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.arg.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Tpl<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_tpl(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.exprs.visit_with(visitor);
        self.quasis.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TaggedTpl<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_tagged_tpl(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.tag.visit_with(visitor);
        self.tpl.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for TplElement<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_tpl_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ParenExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_paren_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.expr.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Callee<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_callee(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Super(it) => it.visit_with(visitor),
            Self::Import(it) => it.visit_with(visitor),
            Self::Expr(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Super {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_super(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Import {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ExprOrSpread<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_expr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.expr.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BlockStmtOrExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_block_stmt_or_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::BlockStmt(it) => it.visit_with(visitor),
            Self::Expr(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignTarget<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assign_target(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Simple(it) => it.visit_with(visitor),
            Self::Pat(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignTargetPat<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assign_target_pat(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Array(it) => it.visit_with(visitor),
            Self::Object(it) => it.visit_with(visitor),
            Self::Invalid(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SimpleAssignTarget<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_simple_assign_target(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_with(visitor),
            Self::Member(it) => it.visit_with(visitor),
            Self::SuperProp(it) => it.visit_with(visitor),
            Self::Paren(it) => it.visit_with(visitor),
            Self::OptChain(it) => it.visit_with(visitor),
            Self::Invalid(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for OptChainExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_chain_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.base.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for OptChainBase<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_chain_base(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Member(it) => it.visit_with(visitor),
            Self::Call(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for OptCall<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_call(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.callee.visit_with(visitor);
        self.args.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Invalid {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_invalid(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Function<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_function(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.params.visit_with(visitor);
        self.decorators.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Param<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_param(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.decorators.visit_with(visitor);
        self.pat.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ParamOrTsParamProp<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_param_or_ts_param_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Param(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Class<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.decorators.visit_with(visitor);
        self.body.visit_with(visitor);
        self.super_class.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ClassMember<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_member(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Constructor(it) => it.visit_with(visitor),
            Self::Method(it) => it.visit_with(visitor),
            Self::PrivateMethod(it) => it.visit_with(visitor),
            Self::ClassProp(it) => it.visit_with(visitor),
            Self::PrivateProp(it) => it.visit_with(visitor),
            Self::Empty(it) => it.visit_with(visitor),
            Self::StaticBlock(it) => it.visit_with(visitor),
            Self::AutoAccessor(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ClassProp<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.key.visit_with(visitor);
        self.value.visit_with(visitor);
        self.decorators.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for PrivateProp<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_private_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.key.visit_with(visitor);
        self.value.visit_with(visitor);
        self.decorators.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ClassMethod<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_method(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.key.visit_with(visitor);
        self.function.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for PrivateMethod<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_private_method(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.key.visit_with(visitor);
        self.function.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Constructor<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_constructor(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.key.visit_with(visitor);
        self.params.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Decorator<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_decorator(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.expr.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for StaticBlock<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_static_block(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Key<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_key(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Private(it) => it.visit_with(visitor),
            Self::Public(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AutoAccessor<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_auto_accessor(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.key.visit_with(visitor);
        self.value.visit_with(visitor);
        self.decorators.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Prop<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Shorthand(it) => it.visit_with(visitor),
            Self::KeyValue(it) => it.visit_with(visitor),
            Self::Assign(it) => it.visit_with(visitor),
            Self::Getter(it) => it.visit_with(visitor),
            Self::Setter(it) => it.visit_with(visitor),
            Self::Method(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for KeyValueProp<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_key_value_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.key.visit_with(visitor);
        self.value.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignProp<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assign_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.key.visit_with(visitor);
        self.value.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for GetterProp<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_getter_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.key.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for SetterProp<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_setter_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.key.visit_with(visitor);
        self.this_param.visit_with(visitor);
        self.param.visit_with(visitor);
        self.body.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for MethodProp<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_method_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.key.visit_with(visitor);
        self.function.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for PropName<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_prop_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_with(visitor),
            Self::Str(it) => it.visit_with(visitor),
            Self::Num(it) => it.visit_with(visitor),
            Self::Computed(it) => it.visit_with(visitor),
            Self::BigInt(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ComputedPropName<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_computed_prop_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.expr.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Pat<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_pat(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_with(visitor),
            Self::Array(it) => it.visit_with(visitor),
            Self::Rest(it) => it.visit_with(visitor),
            Self::Object(it) => it.visit_with(visitor),
            Self::Assign(it) => it.visit_with(visitor),
            Self::Invalid(it) => it.visit_with(visitor),
            Self::Expr(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ArrayPat<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_array_pat(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.elems.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ObjectPat<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_object_pat(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.props.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignPat<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assign_pat(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.left.visit_with(visitor);
        self.right.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for RestPat<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_rest_pat(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.arg.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for ObjectPatProp<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_object_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::KeyValue(it) => it.visit_with(visitor),
            Self::Assign(it) => it.visit_with(visitor),
            Self::Rest(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for KeyValuePatProp<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_key_value_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.key.visit_with(visitor);
        self.value.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for AssignPatProp<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_assign_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.key.visit_with(visitor);
        self.value.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Ident<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ident(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for IdentName<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_ident_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for PrivateName<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_private_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BindingIdent<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_binding_ident(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.id.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Lit<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_lit(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Str(it) => it.visit_with(visitor),
            Self::Bool(it) => it.visit_with(visitor),
            Self::Null(it) => it.visit_with(visitor),
            Self::Num(it) => it.visit_with(visitor),
            Self::BigInt(it) => it.visit_with(visitor),
            Self::Regex(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Str<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_str(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Bool {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_bool(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Null {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_null(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Number<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_number(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for BigInt<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_big_int(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Regex<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_regex(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXObject<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_object(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::JSXMemberExpr(it) => it.visit_with(visitor),
            Self::Ident(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXMemberExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_member_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.obj.visit_with(visitor);
        self.prop.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXNamespacedName<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_namespaced_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.ns.visit_with(visitor);
        self.name.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXEmptyExpr {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_empty_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXExprContainer<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_expr_container(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.expr.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXExpr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::JSXEmptyExpr(it) => it.visit_with(visitor),
            Self::Expr(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXSpreadChild<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_spread_child(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.expr.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXElementName<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_element_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_with(visitor),
            Self::JSXMemberExpr(it) => it.visit_with(visitor),
            Self::JSXNamespacedName(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXOpeningElement<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_opening_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.name.visit_with(visitor);
        self.attrs.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXAttrOrSpread<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::JSXAttr(it) => it.visit_with(visitor),
            Self::SpreadElement(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXClosingElement<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_closing_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.name.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXAttr<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.name.visit_with(visitor);
        self.value.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXAttrName<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attr_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_with(visitor),
            Self::JSXNamespacedName(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXAttrValue<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attr_value(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Str(it) => it.visit_with(visitor),
            Self::JSXExprContainer(it) => it.visit_with(visitor),
            Self::JSXElement(it) => it.visit_with(visitor),
            Self::JSXFragment(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXText<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_text(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXElement<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.opening.visit_with(visitor);
        self.children.visit_with(visitor);
        self.closing.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXElementChild<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_element_child(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::JSXText(it) => it.visit_with(visitor),
            Self::JSXExprContainer(it) => it.visit_with(visitor),
            Self::JSXSpreadChild(it) => it.visit_with(visitor),
            Self::JSXElement(it) => it.visit_with(visitor),
            Self::JSXFragment(it) => it.visit_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXFragment<'a> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_fragment(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        self.opening.visit_with(visitor);
        self.children.visit_with(visitor);
        self.closing.visit_with(visitor);
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXOpeningFragment {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_opening_fragment(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for JSXClosingFragment {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_closing_fragment(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, ModuleItem<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_module_items(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, Stmt<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_stmts(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, ImportSpecifier<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_import_specifiers(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Box<'a, ObjectLit<'a>>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_object_lit(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<ModuleExportName<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_module_export_name(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, ExportSpecifier<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_export_specifiers(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Box<'a, Str<'a>>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_str(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Expr<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Box<'a, Ident<'a>>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_ident(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Stmt<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, SwitchCase<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_switch_cases(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Box<'a, CatchClause<'a>>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_catch_clause(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Box<'a, BlockStmt<'a>>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_block_stmt(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<VarDeclOrExpr<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_var_decl_or_expr(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Pat<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_pat(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, VarDeclarator<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_var_declarators(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Box<'a, ExprOrSpread<'a>>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_expr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, Option<Box<'a, ExprOrSpread<'a>>>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_vec_expr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, PropOrSpread<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_prop_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, ExprOrSpread<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_expr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Vec<'a, ExprOrSpread<'a>>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_expr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, Expr<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_exprs(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, Pat<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_pats(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, TplElement<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_tpl_elements(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, Param<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_params(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, Decorator<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_decorators(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, ClassMember<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_class_members(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, ParamOrTsParamProp<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_param_or_ts_param_props(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, Option<Pat<'a>>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_vec_pats(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, ObjectPatProp<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_object_pat_props(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, JSXAttrOrSpread<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_attr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<JSXAttrValue<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_jsx_attr_value(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Vec<'a, JSXElementChild<'a>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_jsx_element_childs(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        for node in self {
            node.visit_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for Option<Box<'a, JSXClosingElement<'a>>> {
    #[inline]
    fn visit_with(&self, visitor: &mut V) {
        <V as Visit<'a>>::visit_opt_jsx_closing_element(visitor, self)
    }
    #[inline]
    fn visit_children_with(&self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_with(visitor);
        }
    }
}
pub trait VisitMut<'a> {
    #[inline]
    fn visit_mut_program(&mut self, node: &mut Program<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_module(&mut self, node: &mut Module<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_script(&mut self, node: &mut Script<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_module_item(&mut self, node: &mut ModuleItem<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_module_decl(&mut self, node: &mut ModuleDecl<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_import_decl(&mut self, node: &mut ImportDecl<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_import_specifier(&mut self, node: &mut ImportSpecifier<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_import_named_specifier(&mut self, node: &mut ImportNamedSpecifier<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_import_default_specifier(&mut self, node: &mut ImportDefaultSpecifier<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_import_star_as_specifier(&mut self, node: &mut ImportStarAsSpecifier<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_export_decl(&mut self, node: &mut ExportDecl<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_named_export(&mut self, node: &mut NamedExport<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_export_specifier(&mut self, node: &mut ExportSpecifier<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_export_namespace_specifier(&mut self, node: &mut ExportNamespaceSpecifier<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_module_export_name(&mut self, node: &mut ModuleExportName<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_export_default_specifier(&mut self, node: &mut ExportDefaultSpecifier<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_export_named_specifier(&mut self, node: &mut ExportNamedSpecifier<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_export_default_decl(&mut self, node: &mut ExportDefaultDecl<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_default_decl(&mut self, node: &mut DefaultDecl<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_export_default_expr(&mut self, node: &mut ExportDefaultExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_export_all(&mut self, node: &mut ExportAll<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_block_stmt(&mut self, node: &mut BlockStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_stmt(&mut self, node: &mut Stmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_expr_stmt(&mut self, node: &mut ExprStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_empty_stmt(&mut self, node: &mut EmptyStmt) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_debugger_stmt(&mut self, node: &mut DebuggerStmt) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_with_stmt(&mut self, node: &mut WithStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_return_stmt(&mut self, node: &mut ReturnStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_labeled_stmt(&mut self, node: &mut LabeledStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_break_stmt(&mut self, node: &mut BreakStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_continue_stmt(&mut self, node: &mut ContinueStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_if_stmt(&mut self, node: &mut IfStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_switch_stmt(&mut self, node: &mut SwitchStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_throw_stmt(&mut self, node: &mut ThrowStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_try_stmt(&mut self, node: &mut TryStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_while_stmt(&mut self, node: &mut WhileStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_do_while_stmt(&mut self, node: &mut DoWhileStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_for_stmt(&mut self, node: &mut ForStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_for_in_stmt(&mut self, node: &mut ForInStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_for_of_stmt(&mut self, node: &mut ForOfStmt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_switch_case(&mut self, node: &mut SwitchCase<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_catch_clause(&mut self, node: &mut CatchClause<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_for_head(&mut self, node: &mut ForHead<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_var_decl_or_expr(&mut self, node: &mut VarDeclOrExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_decl(&mut self, node: &mut Decl<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_fn_decl(&mut self, node: &mut FnDecl<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_class_decl(&mut self, node: &mut ClassDecl<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_var_decl(&mut self, node: &mut VarDecl<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_var_declarator(&mut self, node: &mut VarDeclarator<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_using_decl(&mut self, node: &mut UsingDecl<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_expr(&mut self, node: &mut Expr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_this_expr(&mut self, node: &mut ThisExpr) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_array_lit(&mut self, node: &mut ArrayLit<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_object_lit(&mut self, node: &mut ObjectLit<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_prop_or_spread(&mut self, node: &mut PropOrSpread<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_spread_element(&mut self, node: &mut SpreadElement<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_unary_expr(&mut self, node: &mut UnaryExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_update_expr(&mut self, node: &mut UpdateExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_bin_expr(&mut self, node: &mut BinExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_fn_expr(&mut self, node: &mut FnExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_class_expr(&mut self, node: &mut ClassExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_assign_expr(&mut self, node: &mut AssignExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_member_expr(&mut self, node: &mut MemberExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_member_prop(&mut self, node: &mut MemberProp<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_super_prop_expr(&mut self, node: &mut SuperPropExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_super_prop(&mut self, node: &mut SuperProp<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_cond_expr(&mut self, node: &mut CondExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_call_expr(&mut self, node: &mut CallExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_new_expr(&mut self, node: &mut NewExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_seq_expr(&mut self, node: &mut SeqExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_arrow_expr(&mut self, node: &mut ArrowExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_yield_expr(&mut self, node: &mut YieldExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_meta_prop_expr(&mut self, node: &mut MetaPropExpr) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_await_expr(&mut self, node: &mut AwaitExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_tpl(&mut self, node: &mut Tpl<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_tagged_tpl(&mut self, node: &mut TaggedTpl<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_tpl_element(&mut self, node: &mut TplElement<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_paren_expr(&mut self, node: &mut ParenExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_callee(&mut self, node: &mut Callee<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_super(&mut self, node: &mut Super) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_import(&mut self, node: &mut Import) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_expr_or_spread(&mut self, node: &mut ExprOrSpread<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_block_stmt_or_expr(&mut self, node: &mut BlockStmtOrExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_assign_target(&mut self, node: &mut AssignTarget<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_assign_target_pat(&mut self, node: &mut AssignTargetPat<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_simple_assign_target(&mut self, node: &mut SimpleAssignTarget<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_chain_expr(&mut self, node: &mut OptChainExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_chain_base(&mut self, node: &mut OptChainBase<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_call(&mut self, node: &mut OptCall<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_invalid(&mut self, node: &mut Invalid) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_function(&mut self, node: &mut Function<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_param(&mut self, node: &mut Param<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_param_or_ts_param_prop(&mut self, node: &mut ParamOrTsParamProp<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_class(&mut self, node: &mut Class<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_class_member(&mut self, node: &mut ClassMember<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_class_prop(&mut self, node: &mut ClassProp<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_private_prop(&mut self, node: &mut PrivateProp<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_class_method(&mut self, node: &mut ClassMethod<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_private_method(&mut self, node: &mut PrivateMethod<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_constructor(&mut self, node: &mut Constructor<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_decorator(&mut self, node: &mut Decorator<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_static_block(&mut self, node: &mut StaticBlock<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_key(&mut self, node: &mut Key<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_auto_accessor(&mut self, node: &mut AutoAccessor<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_prop(&mut self, node: &mut Prop<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_key_value_prop(&mut self, node: &mut KeyValueProp<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_assign_prop(&mut self, node: &mut AssignProp<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_getter_prop(&mut self, node: &mut GetterProp<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_setter_prop(&mut self, node: &mut SetterProp<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_method_prop(&mut self, node: &mut MethodProp<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_prop_name(&mut self, node: &mut PropName<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_computed_prop_name(&mut self, node: &mut ComputedPropName<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_pat(&mut self, node: &mut Pat<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_array_pat(&mut self, node: &mut ArrayPat<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_object_pat(&mut self, node: &mut ObjectPat<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_assign_pat(&mut self, node: &mut AssignPat<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_rest_pat(&mut self, node: &mut RestPat<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_object_pat_prop(&mut self, node: &mut ObjectPatProp<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_key_value_pat_prop(&mut self, node: &mut KeyValuePatProp<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_assign_pat_prop(&mut self, node: &mut AssignPatProp<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_ident(&mut self, node: &mut Ident<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_ident_name(&mut self, node: &mut IdentName<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_private_name(&mut self, node: &mut PrivateName<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_binding_ident(&mut self, node: &mut BindingIdent<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_lit(&mut self, node: &mut Lit<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_str(&mut self, node: &mut Str<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_bool(&mut self, node: &mut Bool) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_null(&mut self, node: &mut Null) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_number(&mut self, node: &mut Number<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_big_int(&mut self, node: &mut BigInt<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_regex(&mut self, node: &mut Regex<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_object(&mut self, node: &mut JSXObject<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_member_expr(&mut self, node: &mut JSXMemberExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_namespaced_name(&mut self, node: &mut JSXNamespacedName<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_empty_expr(&mut self, node: &mut JSXEmptyExpr) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_expr_container(&mut self, node: &mut JSXExprContainer<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_expr(&mut self, node: &mut JSXExpr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_spread_child(&mut self, node: &mut JSXSpreadChild<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_element_name(&mut self, node: &mut JSXElementName<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_opening_element(&mut self, node: &mut JSXOpeningElement<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_attr_or_spread(&mut self, node: &mut JSXAttrOrSpread<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_closing_element(&mut self, node: &mut JSXClosingElement<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_attr(&mut self, node: &mut JSXAttr<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_attr_name(&mut self, node: &mut JSXAttrName<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_attr_value(&mut self, node: &mut JSXAttrValue<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_text(&mut self, node: &mut JSXText<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_element(&mut self, node: &mut JSXElement<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_element_child(&mut self, node: &mut JSXElementChild<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_fragment(&mut self, node: &mut JSXFragment<'a>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_opening_fragment(&mut self, node: &mut JSXOpeningFragment) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_closing_fragment(&mut self, node: &mut JSXClosingFragment) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_module_items(&mut self, node: &mut Vec<'a, ModuleItem<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_stmts(&mut self, node: &mut Vec<'a, Stmt<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_import_specifiers(&mut self, node: &mut Vec<'a, ImportSpecifier<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_object_lit(&mut self, node: &mut Option<Box<'a, ObjectLit<'a>>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_module_export_name(&mut self, node: &mut Option<ModuleExportName<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_export_specifiers(&mut self, node: &mut Vec<'a, ExportSpecifier<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_str(&mut self, node: &mut Option<Box<'a, Str<'a>>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_expr(&mut self, node: &mut Option<Expr<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_ident(&mut self, node: &mut Option<Box<'a, Ident<'a>>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_stmt(&mut self, node: &mut Option<Stmt<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_switch_cases(&mut self, node: &mut Vec<'a, SwitchCase<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_catch_clause(&mut self, node: &mut Option<Box<'a, CatchClause<'a>>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_block_stmt(&mut self, node: &mut Option<Box<'a, BlockStmt<'a>>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_var_decl_or_expr(&mut self, node: &mut Option<VarDeclOrExpr<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_pat(&mut self, node: &mut Option<Pat<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_var_declarators(&mut self, node: &mut Vec<'a, VarDeclarator<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_expr_or_spread(&mut self, node: &mut Option<Box<'a, ExprOrSpread<'a>>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_vec_expr_or_spreads(
        &mut self,
        node: &mut Vec<'a, Option<Box<'a, ExprOrSpread<'a>>>>,
    ) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_prop_or_spreads(&mut self, node: &mut Vec<'a, PropOrSpread<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_expr_or_spreads(&mut self, node: &mut Vec<'a, ExprOrSpread<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_expr_or_spreads(&mut self, node: &mut Option<Vec<'a, ExprOrSpread<'a>>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_exprs(&mut self, node: &mut Vec<'a, Expr<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_pats(&mut self, node: &mut Vec<'a, Pat<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_tpl_elements(&mut self, node: &mut Vec<'a, TplElement<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_params(&mut self, node: &mut Vec<'a, Param<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_decorators(&mut self, node: &mut Vec<'a, Decorator<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_class_members(&mut self, node: &mut Vec<'a, ClassMember<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_param_or_ts_param_props(&mut self, node: &mut Vec<'a, ParamOrTsParamProp<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_vec_pats(&mut self, node: &mut Vec<'a, Option<Pat<'a>>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_object_pat_props(&mut self, node: &mut Vec<'a, ObjectPatProp<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_attr_or_spreads(&mut self, node: &mut Vec<'a, JSXAttrOrSpread<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_jsx_attr_value(&mut self, node: &mut Option<JSXAttrValue<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_jsx_element_childs(&mut self, node: &mut Vec<'a, JSXElementChild<'a>>) {
        node.visit_mut_children_with(self);
    }
    #[inline]
    fn visit_mut_opt_jsx_closing_element(
        &mut self,
        node: &mut Option<Box<'a, JSXClosingElement<'a>>>,
    ) {
        node.visit_mut_children_with(self);
    }
}
pub trait VisitMutWith<'a, V: ?Sized + VisitMut<'a>> {
    fn visit_mut_with(&mut self, visitor: &mut V);
    fn visit_mut_children_with(&mut self, visitor: &mut V);
}
impl<'a, T, V> VisitMutWith<'a, V> for Box<'a, T>
where
    T: VisitMutWith<'a, V>,
    V: ?Sized + VisitMut<'a>,
{
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        (**self).visit_mut_with(visitor)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        (**self).visit_mut_children_with(visitor)
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Program<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_program(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Module(it) => it.visit_mut_with(visitor),
            Self::Script(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Module<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_module(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Script<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_script(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ModuleItem<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_module_item(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::ModuleDecl(it) => it.visit_mut_with(visitor),
            Self::Stmt(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ModuleDecl<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_module_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Import(it) => it.visit_mut_with(visitor),
            Self::ExportDecl(it) => it.visit_mut_with(visitor),
            Self::ExportNamed(it) => it.visit_mut_with(visitor),
            Self::ExportDefaultDecl(it) => it.visit_mut_with(visitor),
            Self::ExportDefaultExpr(it) => it.visit_mut_with(visitor),
            Self::ExportAll(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportDecl<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.specifiers.visit_mut_with(visitor);
        self.src.visit_mut_with(visitor);
        self.with.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportSpecifier<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Named(it) => it.visit_mut_with(visitor),
            Self::Default(it) => it.visit_mut_with(visitor),
            Self::Namespace(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportNamedSpecifier<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_named_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.local.visit_mut_with(visitor);
        self.imported.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportDefaultSpecifier<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_default_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.local.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ImportStarAsSpecifier<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_star_as_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.local.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportDecl<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.decl.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for NamedExport<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_named_export(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.specifiers.visit_mut_with(visitor);
        self.src.visit_mut_with(visitor);
        self.with.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportSpecifier<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Namespace(it) => it.visit_mut_with(visitor),
            Self::Default(it) => it.visit_mut_with(visitor),
            Self::Named(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportNamespaceSpecifier<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_namespace_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.name.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ModuleExportName<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_module_export_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_mut_with(visitor),
            Self::Str(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportDefaultSpecifier<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_default_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.exported.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportNamedSpecifier<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_named_specifier(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.orig.visit_mut_with(visitor);
        self.exported.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportDefaultDecl<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_default_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.decl.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for DefaultDecl<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_default_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Class(it) => it.visit_mut_with(visitor),
            Self::Fn(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportDefaultExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_default_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.expr.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExportAll<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_all(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.src.visit_mut_with(visitor);
        self.with.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BlockStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_block_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.stmts.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Stmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Block(it) => it.visit_mut_with(visitor),
            Self::Empty(it) => it.visit_mut_with(visitor),
            Self::Debugger(it) => it.visit_mut_with(visitor),
            Self::With(it) => it.visit_mut_with(visitor),
            Self::Return(it) => it.visit_mut_with(visitor),
            Self::Labeled(it) => it.visit_mut_with(visitor),
            Self::Break(it) => it.visit_mut_with(visitor),
            Self::Continue(it) => it.visit_mut_with(visitor),
            Self::If(it) => it.visit_mut_with(visitor),
            Self::Switch(it) => it.visit_mut_with(visitor),
            Self::Throw(it) => it.visit_mut_with(visitor),
            Self::Try(it) => it.visit_mut_with(visitor),
            Self::While(it) => it.visit_mut_with(visitor),
            Self::DoWhile(it) => it.visit_mut_with(visitor),
            Self::For(it) => it.visit_mut_with(visitor),
            Self::ForIn(it) => it.visit_mut_with(visitor),
            Self::ForOf(it) => it.visit_mut_with(visitor),
            Self::Decl(it) => it.visit_mut_with(visitor),
            Self::Expr(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExprStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_expr_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.expr.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for EmptyStmt {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_empty_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for DebuggerStmt {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_debugger_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for WithStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_with_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.obj.visit_mut_with(visitor);
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ReturnStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_return_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.arg.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for LabeledStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_labeled_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.label.visit_mut_with(visitor);
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BreakStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_break_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.label.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ContinueStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_continue_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.label.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for IfStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_if_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.test.visit_mut_with(visitor);
        self.cons.visit_mut_with(visitor);
        self.alt.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SwitchStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_switch_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.discriminant.visit_mut_with(visitor);
        self.cases.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ThrowStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_throw_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.arg.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TryStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_try_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.block.visit_mut_with(visitor);
        self.handler.visit_mut_with(visitor);
        self.finalizer.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for WhileStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_while_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.test.visit_mut_with(visitor);
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for DoWhileStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_do_while_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.test.visit_mut_with(visitor);
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_for_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.init.visit_mut_with(visitor);
        self.test.visit_mut_with(visitor);
        self.update.visit_mut_with(visitor);
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForInStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_for_in_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.left.visit_mut_with(visitor);
        self.right.visit_mut_with(visitor);
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForOfStmt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_for_of_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.left.visit_mut_with(visitor);
        self.right.visit_mut_with(visitor);
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SwitchCase<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_switch_case(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.test.visit_mut_with(visitor);
        self.cons.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for CatchClause<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_catch_clause(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.param.visit_mut_with(visitor);
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ForHead<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_for_head(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::VarDecl(it) => it.visit_mut_with(visitor),
            Self::UsingDecl(it) => it.visit_mut_with(visitor),
            Self::Pat(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for VarDeclOrExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_var_decl_or_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::VarDecl(it) => it.visit_mut_with(visitor),
            Self::Expr(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Decl<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Class(it) => it.visit_mut_with(visitor),
            Self::Fn(it) => it.visit_mut_with(visitor),
            Self::Var(it) => it.visit_mut_with(visitor),
            Self::Using(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for FnDecl<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_fn_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.ident.visit_mut_with(visitor);
        self.function.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassDecl<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.ident.visit_mut_with(visitor);
        self.class.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for VarDecl<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_var_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.decls.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for VarDeclarator<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_var_declarator(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.name.visit_mut_with(visitor);
        self.init.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for UsingDecl<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_using_decl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.decls.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Expr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::This(it) => it.visit_mut_with(visitor),
            Self::Array(it) => it.visit_mut_with(visitor),
            Self::Object(it) => it.visit_mut_with(visitor),
            Self::Fn(it) => it.visit_mut_with(visitor),
            Self::Unary(it) => it.visit_mut_with(visitor),
            Self::Update(it) => it.visit_mut_with(visitor),
            Self::Bin(it) => it.visit_mut_with(visitor),
            Self::Assign(it) => it.visit_mut_with(visitor),
            Self::Member(it) => it.visit_mut_with(visitor),
            Self::SuperProp(it) => it.visit_mut_with(visitor),
            Self::Cond(it) => it.visit_mut_with(visitor),
            Self::Call(it) => it.visit_mut_with(visitor),
            Self::New(it) => it.visit_mut_with(visitor),
            Self::Seq(it) => it.visit_mut_with(visitor),
            Self::Ident(it) => it.visit_mut_with(visitor),
            Self::Lit(it) => it.visit_mut_with(visitor),
            Self::Tpl(it) => it.visit_mut_with(visitor),
            Self::TaggedTpl(it) => it.visit_mut_with(visitor),
            Self::Arrow(it) => it.visit_mut_with(visitor),
            Self::Class(it) => it.visit_mut_with(visitor),
            Self::Yield(it) => it.visit_mut_with(visitor),
            Self::MetaProp(it) => it.visit_mut_with(visitor),
            Self::Await(it) => it.visit_mut_with(visitor),
            Self::Paren(it) => it.visit_mut_with(visitor),
            Self::JSXMember(it) => it.visit_mut_with(visitor),
            Self::JSXNamespacedName(it) => it.visit_mut_with(visitor),
            Self::JSXEmpty(it) => it.visit_mut_with(visitor),
            Self::JSXElement(it) => it.visit_mut_with(visitor),
            Self::JSXFragment(it) => it.visit_mut_with(visitor),
            Self::PrivateName(it) => it.visit_mut_with(visitor),
            Self::OptChain(it) => it.visit_mut_with(visitor),
            Self::Invalid(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ThisExpr {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_this_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ArrayLit<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_array_lit(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.elems.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ObjectLit<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_object_lit(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.props.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PropOrSpread<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_prop_or_spread(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Spread(it) => it.visit_mut_with(visitor),
            Self::Prop(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SpreadElement<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_spread_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.expr.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for UnaryExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_unary_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.arg.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for UpdateExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_update_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.arg.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BinExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_bin_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.left.visit_mut_with(visitor);
        self.right.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for FnExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_fn_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.ident.visit_mut_with(visitor);
        self.function.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.ident.visit_mut_with(visitor);
        self.class.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assign_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.left.visit_mut_with(visitor);
        self.right.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MemberExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_member_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.obj.visit_mut_with(visitor);
        self.prop.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MemberProp<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_member_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_mut_with(visitor),
            Self::PrivateName(it) => it.visit_mut_with(visitor),
            Self::Computed(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SuperPropExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_super_prop_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.obj.visit_mut_with(visitor);
        self.prop.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SuperProp<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_super_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_mut_with(visitor),
            Self::Computed(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for CondExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_cond_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.test.visit_mut_with(visitor);
        self.cons.visit_mut_with(visitor);
        self.alt.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for CallExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_call_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.callee.visit_mut_with(visitor);
        self.args.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for NewExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_new_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.callee.visit_mut_with(visitor);
        self.args.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SeqExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_seq_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.exprs.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ArrowExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_arrow_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.params.visit_mut_with(visitor);
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for YieldExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_yield_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.arg.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MetaPropExpr {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_meta_prop_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AwaitExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_await_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.arg.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Tpl<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_tpl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.exprs.visit_mut_with(visitor);
        self.quasis.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TaggedTpl<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_tagged_tpl(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.tag.visit_mut_with(visitor);
        self.tpl.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for TplElement<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_tpl_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ParenExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_paren_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.expr.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Callee<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_callee(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Super(it) => it.visit_mut_with(visitor),
            Self::Import(it) => it.visit_mut_with(visitor),
            Self::Expr(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Super {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_super(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Import {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ExprOrSpread<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_expr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.expr.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BlockStmtOrExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_block_stmt_or_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::BlockStmt(it) => it.visit_mut_with(visitor),
            Self::Expr(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignTarget<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assign_target(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Simple(it) => it.visit_mut_with(visitor),
            Self::Pat(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignTargetPat<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assign_target_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Array(it) => it.visit_mut_with(visitor),
            Self::Object(it) => it.visit_mut_with(visitor),
            Self::Invalid(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SimpleAssignTarget<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_simple_assign_target(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_mut_with(visitor),
            Self::Member(it) => it.visit_mut_with(visitor),
            Self::SuperProp(it) => it.visit_mut_with(visitor),
            Self::Paren(it) => it.visit_mut_with(visitor),
            Self::OptChain(it) => it.visit_mut_with(visitor),
            Self::Invalid(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for OptChainExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_chain_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.base.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for OptChainBase<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_chain_base(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Member(it) => it.visit_mut_with(visitor),
            Self::Call(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for OptCall<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_call(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.callee.visit_mut_with(visitor);
        self.args.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Invalid {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_invalid(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Function<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_function(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.params.visit_mut_with(visitor);
        self.decorators.visit_mut_with(visitor);
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Param<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_param(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.decorators.visit_mut_with(visitor);
        self.pat.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ParamOrTsParamProp<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_param_or_ts_param_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Param(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Class<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.decorators.visit_mut_with(visitor);
        self.body.visit_mut_with(visitor);
        self.super_class.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassMember<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_member(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Constructor(it) => it.visit_mut_with(visitor),
            Self::Method(it) => it.visit_mut_with(visitor),
            Self::PrivateMethod(it) => it.visit_mut_with(visitor),
            Self::ClassProp(it) => it.visit_mut_with(visitor),
            Self::PrivateProp(it) => it.visit_mut_with(visitor),
            Self::Empty(it) => it.visit_mut_with(visitor),
            Self::StaticBlock(it) => it.visit_mut_with(visitor),
            Self::AutoAccessor(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassProp<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.key.visit_mut_with(visitor);
        self.value.visit_mut_with(visitor);
        self.decorators.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PrivateProp<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_private_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.key.visit_mut_with(visitor);
        self.value.visit_mut_with(visitor);
        self.decorators.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ClassMethod<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_method(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.key.visit_mut_with(visitor);
        self.function.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PrivateMethod<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_private_method(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.key.visit_mut_with(visitor);
        self.function.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Constructor<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_constructor(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.key.visit_mut_with(visitor);
        self.params.visit_mut_with(visitor);
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Decorator<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_decorator(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.expr.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for StaticBlock<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_static_block(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Key<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_key(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Private(it) => it.visit_mut_with(visitor),
            Self::Public(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AutoAccessor<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_auto_accessor(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.key.visit_mut_with(visitor);
        self.value.visit_mut_with(visitor);
        self.decorators.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Prop<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Shorthand(it) => it.visit_mut_with(visitor),
            Self::KeyValue(it) => it.visit_mut_with(visitor),
            Self::Assign(it) => it.visit_mut_with(visitor),
            Self::Getter(it) => it.visit_mut_with(visitor),
            Self::Setter(it) => it.visit_mut_with(visitor),
            Self::Method(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for KeyValueProp<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_key_value_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.key.visit_mut_with(visitor);
        self.value.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignProp<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assign_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.key.visit_mut_with(visitor);
        self.value.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for GetterProp<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_getter_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.key.visit_mut_with(visitor);
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for SetterProp<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_setter_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.key.visit_mut_with(visitor);
        self.this_param.visit_mut_with(visitor);
        self.param.visit_mut_with(visitor);
        self.body.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for MethodProp<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_method_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.key.visit_mut_with(visitor);
        self.function.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PropName<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_prop_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_mut_with(visitor),
            Self::Str(it) => it.visit_mut_with(visitor),
            Self::Num(it) => it.visit_mut_with(visitor),
            Self::Computed(it) => it.visit_mut_with(visitor),
            Self::BigInt(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ComputedPropName<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_computed_prop_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.expr.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Pat<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_mut_with(visitor),
            Self::Array(it) => it.visit_mut_with(visitor),
            Self::Rest(it) => it.visit_mut_with(visitor),
            Self::Object(it) => it.visit_mut_with(visitor),
            Self::Assign(it) => it.visit_mut_with(visitor),
            Self::Invalid(it) => it.visit_mut_with(visitor),
            Self::Expr(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ArrayPat<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_array_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.elems.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ObjectPat<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_object_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.props.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignPat<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assign_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.left.visit_mut_with(visitor);
        self.right.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for RestPat<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_rest_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.arg.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for ObjectPatProp<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_object_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::KeyValue(it) => it.visit_mut_with(visitor),
            Self::Assign(it) => it.visit_mut_with(visitor),
            Self::Rest(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for KeyValuePatProp<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_key_value_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.key.visit_mut_with(visitor);
        self.value.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for AssignPatProp<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_assign_pat_prop(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.key.visit_mut_with(visitor);
        self.value.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Ident<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ident(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for IdentName<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_ident_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for PrivateName<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_private_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BindingIdent<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_binding_ident(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.id.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Lit<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_lit(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Str(it) => it.visit_mut_with(visitor),
            Self::Bool(it) => it.visit_mut_with(visitor),
            Self::Null(it) => it.visit_mut_with(visitor),
            Self::Num(it) => it.visit_mut_with(visitor),
            Self::BigInt(it) => it.visit_mut_with(visitor),
            Self::Regex(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Str<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_str(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Bool {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_bool(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Null {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_null(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Number<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_number(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for BigInt<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_big_int(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Regex<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_regex(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXObject<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_object(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::JSXMemberExpr(it) => it.visit_mut_with(visitor),
            Self::Ident(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXMemberExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_member_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.obj.visit_mut_with(visitor);
        self.prop.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXNamespacedName<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_namespaced_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.ns.visit_mut_with(visitor);
        self.name.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXEmptyExpr {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_empty_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXExprContainer<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_expr_container(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.expr.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXExpr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::JSXEmptyExpr(it) => it.visit_mut_with(visitor),
            Self::Expr(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXSpreadChild<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_spread_child(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.expr.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXElementName<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_element_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_mut_with(visitor),
            Self::JSXMemberExpr(it) => it.visit_mut_with(visitor),
            Self::JSXNamespacedName(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXOpeningElement<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_opening_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.name.visit_mut_with(visitor);
        self.attrs.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXAttrOrSpread<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::JSXAttr(it) => it.visit_mut_with(visitor),
            Self::SpreadElement(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXClosingElement<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_closing_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.name.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXAttr<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.name.visit_mut_with(visitor);
        self.value.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXAttrName<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attr_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Ident(it) => it.visit_mut_with(visitor),
            Self::JSXNamespacedName(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXAttrValue<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attr_value(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::Str(it) => it.visit_mut_with(visitor),
            Self::JSXExprContainer(it) => it.visit_mut_with(visitor),
            Self::JSXElement(it) => it.visit_mut_with(visitor),
            Self::JSXFragment(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXText<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_text(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXElement<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.opening.visit_mut_with(visitor);
        self.children.visit_mut_with(visitor);
        self.closing.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXElementChild<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_element_child(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        match self {
            Self::JSXText(it) => it.visit_mut_with(visitor),
            Self::JSXExprContainer(it) => it.visit_mut_with(visitor),
            Self::JSXSpreadChild(it) => it.visit_mut_with(visitor),
            Self::JSXElement(it) => it.visit_mut_with(visitor),
            Self::JSXFragment(it) => it.visit_mut_with(visitor),
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXFragment<'a> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_fragment(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        self.opening.visit_mut_with(visitor);
        self.children.visit_mut_with(visitor);
        self.closing.visit_mut_with(visitor);
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXOpeningFragment {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_opening_fragment(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for JSXClosingFragment {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_closing_fragment(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {}
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, ModuleItem<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_module_items(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, Stmt<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_stmts(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, ImportSpecifier<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_import_specifiers(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Box<'a, ObjectLit<'a>>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_object_lit(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<ModuleExportName<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_module_export_name(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, ExportSpecifier<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_export_specifiers(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Box<'a, Str<'a>>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_str(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Expr<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Box<'a, Ident<'a>>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_ident(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Stmt<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, SwitchCase<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_switch_cases(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Box<'a, CatchClause<'a>>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_catch_clause(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Box<'a, BlockStmt<'a>>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_block_stmt(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<VarDeclOrExpr<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_var_decl_or_expr(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Pat<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_pat(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, VarDeclarator<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_var_declarators(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Box<'a, ExprOrSpread<'a>>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_expr_or_spread(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V>
    for Vec<'a, Option<Box<'a, ExprOrSpread<'a>>>>
{
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_vec_expr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, PropOrSpread<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_prop_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, ExprOrSpread<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_expr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Vec<'a, ExprOrSpread<'a>>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_expr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, Expr<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_exprs(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, Pat<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_pats(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, TplElement<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_tpl_elements(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, Param<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_params(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, Decorator<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_decorators(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, ClassMember<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_class_members(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, ParamOrTsParamProp<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_param_or_ts_param_props(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, Option<Pat<'a>>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_vec_pats(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, ObjectPatProp<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_object_pat_props(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, JSXAttrOrSpread<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_attr_or_spreads(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<JSXAttrValue<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_jsx_attr_value(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Vec<'a, JSXElementChild<'a>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_jsx_element_childs(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        for node in self {
            node.visit_mut_with(visitor);
        }
    }
}
impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for Option<Box<'a, JSXClosingElement<'a>>> {
    #[inline]
    fn visit_mut_with(&mut self, visitor: &mut V) {
        <V as VisitMut<'a>>::visit_mut_opt_jsx_closing_element(visitor, self)
    }
    #[inline]
    fn visit_mut_children_with(&mut self, visitor: &mut V) {
        if let Some(node) = self {
            node.visit_mut_with(visitor);
        }
    }
}
