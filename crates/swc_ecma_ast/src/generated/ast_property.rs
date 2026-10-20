#![allow(
    unused,
    clippy::useless_conversion,
    clippy::identity_op,
    clippy::erasing_op,
    clippy::let_and_return
)]
use crate::*;
use oxc_index::Idx;
impl Program {
    #[inline]
    pub fn directives<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Directive> {
        unsafe { ast.nodes.data_unchecked(self.0).program.directives }
    }
    #[inline]
    pub fn hashbang<'a>(self, ast: &crate::Ast<'a>) -> Option<Hashbang> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).program.hashbang };
        (raw != u32::MAX).then(|| Hashbang(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Stmt> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .program
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_directives<'a>(self, ast: &mut crate::Ast<'a>, directives: SubRange<Directive>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).program.directives = directives;
        }
    }
    #[inline]
    pub fn set_hashbang<'a>(self, ast: &mut crate::Ast<'a>, hashbang: Option<Hashbang>) {
        let field_val: u32 = hashbang.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).program.hashbang = field_val;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: SubRange<Stmt>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .program
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.into_extra_data();
        };
    }
}
impl Stmt {
    #[inline]
    pub fn is_block_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::BlockStatement
    }
    #[inline]
    pub fn is_break_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::BreakStatement
    }
    #[inline]
    pub fn is_continue_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ContinueStatement
    }
    #[inline]
    pub fn is_debugger_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::DebuggerStatement
    }
    #[inline]
    pub fn is_do_while_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::DoWhileStatement
    }
    #[inline]
    pub fn is_empty_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::EmptyStatement
    }
    #[inline]
    pub fn is_expression_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ExpressionStatement
    }
    #[inline]
    pub fn is_for_in_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ForInStatement
    }
    #[inline]
    pub fn is_for_of_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ForOfStatement
    }
    #[inline]
    pub fn is_for_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ForStatement
    }
    #[inline]
    pub fn is_if_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::IfStatement
    }
    #[inline]
    pub fn is_labeled_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::LabeledStatement
    }
    #[inline]
    pub fn is_return_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ReturnStatement
    }
    #[inline]
    pub fn is_switch_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::SwitchStatement
    }
    #[inline]
    pub fn is_throw_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ThrowStatement
    }
    #[inline]
    pub fn is_try_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TryStatement
    }
    #[inline]
    pub fn is_while_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::WhileStatement
    }
    #[inline]
    pub fn is_with_statement(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::WithStatement
    }
    #[inline]
    pub fn is_declaration(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::VariableDeclaration
                | NodeKind::Function
                | NodeKind::Class
                | NodeKind::TsFunction
                | NodeKind::TsTypeAliasDeclaration
                | NodeKind::TsInterfaceDeclaration
                | NodeKind::TsEnumDeclaration
                | NodeKind::TsModuleDeclaration
                | NodeKind::TsGlobalDeclaration
                | NodeKind::TsImportEqualsDeclaration
        )
    }
    #[inline]
    pub fn is_import_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ImportDeclaration
    }
    #[inline]
    pub fn is_export_named_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ExportNamedDeclaration
    }
    #[inline]
    pub fn is_export_default_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ExportDefaultDeclaration
    }
    #[inline]
    pub fn is_export_all_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ExportAllDeclaration
    }
    #[inline]
    pub fn is_ts_export_assignment(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsExportAssignment
    }
    #[inline]
    pub fn is_ts_namespace_export_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsNamespaceExportDeclaration
    }
    #[inline]
    pub fn as_block_statement(self, ast: &Ast<'_>) -> Option<BlockStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::BlockStatement) {
            Some(BlockStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_break_statement(self, ast: &Ast<'_>) -> Option<BreakStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::BreakStatement) {
            Some(BreakStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_continue_statement(self, ast: &Ast<'_>) -> Option<ContinueStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ContinueStatement) {
            Some(ContinueStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_debugger_statement(self, ast: &Ast<'_>) -> Option<DebuggerStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::DebuggerStatement) {
            Some(DebuggerStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_do_while_statement(self, ast: &Ast<'_>) -> Option<DoWhileStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::DoWhileStatement) {
            Some(DoWhileStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_empty_statement(self, ast: &Ast<'_>) -> Option<EmptyStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::EmptyStatement) {
            Some(EmptyStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_expression_statement(self, ast: &Ast<'_>) -> Option<ExpressionStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ExpressionStatement) {
            Some(ExpressionStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_for_in_statement(self, ast: &Ast<'_>) -> Option<ForInStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ForInStatement) {
            Some(ForInStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_for_of_statement(self, ast: &Ast<'_>) -> Option<ForOfStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ForOfStatement) {
            Some(ForOfStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_for_statement(self, ast: &Ast<'_>) -> Option<ForStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ForStatement) {
            Some(ForStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_if_statement(self, ast: &Ast<'_>) -> Option<IfStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::IfStatement) {
            Some(IfStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_labeled_statement(self, ast: &Ast<'_>) -> Option<LabeledStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::LabeledStatement) {
            Some(LabeledStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_return_statement(self, ast: &Ast<'_>) -> Option<ReturnStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ReturnStatement) {
            Some(ReturnStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_switch_statement(self, ast: &Ast<'_>) -> Option<SwitchStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::SwitchStatement) {
            Some(SwitchStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_throw_statement(self, ast: &Ast<'_>) -> Option<ThrowStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ThrowStatement) {
            Some(ThrowStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_try_statement(self, ast: &Ast<'_>) -> Option<TryStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TryStatement) {
            Some(TryStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_while_statement(self, ast: &Ast<'_>) -> Option<WhileStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::WhileStatement) {
            Some(WhileStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_with_statement(self, ast: &Ast<'_>) -> Option<WithStatement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::WithStatement) {
            Some(WithStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_declaration(self, ast: &Ast<'_>) -> Option<Decl> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(
            kind,
            NodeKind::VariableDeclaration
                | NodeKind::Function
                | NodeKind::Class
                | NodeKind::TsFunction
                | NodeKind::TsTypeAliasDeclaration
                | NodeKind::TsInterfaceDeclaration
                | NodeKind::TsEnumDeclaration
                | NodeKind::TsModuleDeclaration
                | NodeKind::TsGlobalDeclaration
                | NodeKind::TsImportEqualsDeclaration
        ) {
            Some(Decl(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_import_declaration(self, ast: &Ast<'_>) -> Option<ImportDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ImportDeclaration) {
            Some(ImportDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_export_named_declaration(self, ast: &Ast<'_>) -> Option<ExportNamedDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ExportNamedDeclaration) {
            Some(ExportNamedDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_export_default_declaration(self, ast: &Ast<'_>) -> Option<ExportDefaultDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ExportDefaultDeclaration) {
            Some(ExportDefaultDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_export_all_declaration(self, ast: &Ast<'_>) -> Option<ExportAllDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ExportAllDeclaration) {
            Some(ExportAllDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_export_assignment(self, ast: &Ast<'_>) -> Option<TsExportAssignment> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsExportAssignment) {
            Some(TsExportAssignment(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_namespace_export_declaration(
        self,
        ast: &Ast<'_>,
    ) -> Option<TsNamespaceExportDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsNamespaceExportDeclaration) {
            Some(TsNamespaceExportDeclaration(self.0))
        } else {
            None
        }
    }
}
impl Decl {
    #[inline]
    pub fn is_variable_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::VariableDeclaration
    }
    #[inline]
    pub fn is_function(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::Function
    }
    #[inline]
    pub fn is_class(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::Class
    }
    #[inline]
    pub fn is_ts_function(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsFunction
    }
    #[inline]
    pub fn is_ts_type_alias_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypeAliasDeclaration
    }
    #[inline]
    pub fn is_ts_interface_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsInterfaceDeclaration
    }
    #[inline]
    pub fn is_ts_enum_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsEnumDeclaration
    }
    #[inline]
    pub fn is_ts_module_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsModuleDeclaration
    }
    #[inline]
    pub fn is_ts_global_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsGlobalDeclaration
    }
    #[inline]
    pub fn is_ts_import_equals_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsImportEqualsDeclaration
    }
    #[inline]
    pub fn as_variable_declaration(self, ast: &Ast<'_>) -> Option<VariableDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::VariableDeclaration) {
            Some(VariableDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_function(self, ast: &Ast<'_>) -> Option<Function> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::Function) {
            Some(Function(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_class(self, ast: &Ast<'_>) -> Option<Class> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::Class) {
            Some(Class(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_function(self, ast: &Ast<'_>) -> Option<TsFunction> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsFunction) {
            Some(TsFunction(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_alias_declaration(self, ast: &Ast<'_>) -> Option<TsTypeAliasDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsTypeAliasDeclaration) {
            Some(TsTypeAliasDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_interface_declaration(self, ast: &Ast<'_>) -> Option<TsInterfaceDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsInterfaceDeclaration) {
            Some(TsInterfaceDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_enum_declaration(self, ast: &Ast<'_>) -> Option<TsEnumDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsEnumDeclaration) {
            Some(TsEnumDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_module_declaration(self, ast: &Ast<'_>) -> Option<TsModuleDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsModuleDeclaration) {
            Some(TsModuleDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_global_declaration(self, ast: &Ast<'_>) -> Option<TsGlobalDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsGlobalDeclaration) {
            Some(TsGlobalDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_import_equals_declaration(
        self,
        ast: &Ast<'_>,
    ) -> Option<TsImportEqualsDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsImportEqualsDeclaration) {
            Some(TsImportEqualsDeclaration(self.0))
        } else {
            None
        }
    }
}
impl Expr {
    #[inline]
    pub fn is_identifier_reference(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierReference
    }
    #[inline]
    pub fn is_this_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ThisExpression
    }
    #[inline]
    pub fn is_super(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::Super
    }
    #[inline]
    pub fn is_meta_property(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::MetaProperty
    }
    #[inline]
    pub fn is_string_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::StringLiteral
    }
    #[inline]
    pub fn is_numeric_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::NumericLiteral
    }
    #[inline]
    pub fn is_big_int_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::BigIntLiteral
    }
    #[inline]
    pub fn is_boolean_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::BooleanLiteral
    }
    #[inline]
    pub fn is_null_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::NullLiteral
    }
    #[inline]
    pub fn is_reg_exp_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::RegExpLiteral
    }
    #[inline]
    pub fn is_template_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TemplateLiteral
    }
    #[inline]
    pub fn is_binary_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::BinaryExpression
    }
    #[inline]
    pub fn is_logical_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::LogicalExpression
    }
    #[inline]
    pub fn is_unary_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::UnaryExpression
    }
    #[inline]
    pub fn is_update_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::UpdateExpression
    }
    #[inline]
    pub fn is_assignment_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::AssignmentExpression
    }
    #[inline]
    pub fn is_conditional_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ConditionalExpression
    }
    #[inline]
    pub fn is_sequence_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::SequenceExpression
    }
    #[inline]
    pub fn is_member_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::MemberExpression
    }
    #[inline]
    pub fn is_call_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::CallExpression
    }
    #[inline]
    pub fn is_chain_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ChainExpression
    }
    #[inline]
    pub fn is_new_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::NewExpression
    }
    #[inline]
    pub fn is_tagged_template_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TaggedTemplateExpression
    }
    #[inline]
    pub fn is_arrow_function_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ArrowFunctionExpression
    }
    #[inline]
    pub fn is_array_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ArrayExpression
    }
    #[inline]
    pub fn is_object_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ObjectExpression
    }
    #[inline]
    pub fn is_parenthesized_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ParenthesizedExpression
    }
    #[inline]
    pub fn is_function(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::Function
    }
    #[inline]
    pub fn is_class(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::Class
    }
    #[inline]
    pub fn is_import_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ImportExpression
    }
    #[inline]
    pub fn is_await_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::AwaitExpression
    }
    #[inline]
    pub fn is_yield_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::YieldExpression
    }
    #[inline]
    pub fn is_private_identifier(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::PrivateIdentifier
    }
    #[inline]
    pub fn is_ts_as_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsAsExpression
    }
    #[inline]
    pub fn is_ts_satisfies_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsSatisfiesExpression
    }
    #[inline]
    pub fn is_ts_type_assertion(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypeAssertion
    }
    #[inline]
    pub fn is_ts_non_null_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsNonNullExpression
    }
    #[inline]
    pub fn is_ts_instantiation_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsInstantiationExpression
    }
    #[inline]
    pub fn is_jsx_element(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxElement
    }
    #[inline]
    pub fn is_jsx_fragment(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxFragment
    }
    #[inline]
    pub fn as_identifier_reference(self, ast: &Ast<'_>) -> Option<IdentifierReference> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::IdentifierReference) {
            Some(IdentifierReference(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_this_expression(self, ast: &Ast<'_>) -> Option<ThisExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ThisExpression) {
            Some(ThisExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_super(self, ast: &Ast<'_>) -> Option<Super> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::Super) {
            Some(Super(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_meta_property(self, ast: &Ast<'_>) -> Option<MetaProperty> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::MetaProperty) {
            Some(MetaProperty(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_>) -> Option<StringLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::StringLiteral) {
            Some(StringLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_numeric_literal(self, ast: &Ast<'_>) -> Option<NumericLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::NumericLiteral) {
            Some(NumericLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_big_int_literal(self, ast: &Ast<'_>) -> Option<BigIntLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::BigIntLiteral) {
            Some(BigIntLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_boolean_literal(self, ast: &Ast<'_>) -> Option<BooleanLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::BooleanLiteral) {
            Some(BooleanLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_null_literal(self, ast: &Ast<'_>) -> Option<NullLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::NullLiteral) {
            Some(NullLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_reg_exp_literal(self, ast: &Ast<'_>) -> Option<RegExpLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::RegExpLiteral) {
            Some(RegExpLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_template_literal(self, ast: &Ast<'_>) -> Option<TemplateLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TemplateLiteral) {
            Some(TemplateLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_binary_expression(self, ast: &Ast<'_>) -> Option<BinaryExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::BinaryExpression) {
            Some(BinaryExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_logical_expression(self, ast: &Ast<'_>) -> Option<LogicalExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::LogicalExpression) {
            Some(LogicalExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_unary_expression(self, ast: &Ast<'_>) -> Option<UnaryExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::UnaryExpression) {
            Some(UnaryExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_update_expression(self, ast: &Ast<'_>) -> Option<UpdateExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::UpdateExpression) {
            Some(UpdateExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_assignment_expression(self, ast: &Ast<'_>) -> Option<AssignmentExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::AssignmentExpression) {
            Some(AssignmentExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_conditional_expression(self, ast: &Ast<'_>) -> Option<ConditionalExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ConditionalExpression) {
            Some(ConditionalExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_sequence_expression(self, ast: &Ast<'_>) -> Option<SequenceExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::SequenceExpression) {
            Some(SequenceExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_member_expression(self, ast: &Ast<'_>) -> Option<MemberExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::MemberExpression) {
            Some(MemberExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_call_expression(self, ast: &Ast<'_>) -> Option<CallExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::CallExpression) {
            Some(CallExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_chain_expression(self, ast: &Ast<'_>) -> Option<ChainExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ChainExpression) {
            Some(ChainExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_new_expression(self, ast: &Ast<'_>) -> Option<NewExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::NewExpression) {
            Some(NewExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_tagged_template_expression(self, ast: &Ast<'_>) -> Option<TaggedTemplateExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TaggedTemplateExpression) {
            Some(TaggedTemplateExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_arrow_function_expression(self, ast: &Ast<'_>) -> Option<ArrowFunctionExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ArrowFunctionExpression) {
            Some(ArrowFunctionExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_array_expression(self, ast: &Ast<'_>) -> Option<ArrayExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ArrayExpression) {
            Some(ArrayExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_object_expression(self, ast: &Ast<'_>) -> Option<ObjectExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ObjectExpression) {
            Some(ObjectExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_parenthesized_expression(self, ast: &Ast<'_>) -> Option<ParenthesizedExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ParenthesizedExpression) {
            Some(ParenthesizedExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_function(self, ast: &Ast<'_>) -> Option<Function> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::Function) {
            Some(Function(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_class(self, ast: &Ast<'_>) -> Option<Class> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::Class) {
            Some(Class(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_import_expression(self, ast: &Ast<'_>) -> Option<ImportExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ImportExpression) {
            Some(ImportExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_await_expression(self, ast: &Ast<'_>) -> Option<AwaitExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::AwaitExpression) {
            Some(AwaitExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_yield_expression(self, ast: &Ast<'_>) -> Option<YieldExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::YieldExpression) {
            Some(YieldExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_private_identifier(self, ast: &Ast<'_>) -> Option<PrivateIdentifier> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::PrivateIdentifier) {
            Some(PrivateIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_as_expression(self, ast: &Ast<'_>) -> Option<TsAsExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsAsExpression) {
            Some(TsAsExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_satisfies_expression(self, ast: &Ast<'_>) -> Option<TsSatisfiesExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsSatisfiesExpression) {
            Some(TsSatisfiesExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_assertion(self, ast: &Ast<'_>) -> Option<TsTypeAssertion> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsTypeAssertion) {
            Some(TsTypeAssertion(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_non_null_expression(self, ast: &Ast<'_>) -> Option<TsNonNullExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsNonNullExpression) {
            Some(TsNonNullExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_instantiation_expression(
        self,
        ast: &Ast<'_>,
    ) -> Option<TsInstantiationExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsInstantiationExpression) {
            Some(TsInstantiationExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_element(self, ast: &Ast<'_>) -> Option<JsxElement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxElement) {
            Some(JsxElement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self, ast: &Ast<'_>) -> Option<JsxFragment> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxFragment) {
            Some(JsxFragment(self.0))
        } else {
            None
        }
    }
}
impl BindingPattern {
    #[inline]
    pub fn is_binding_identifier(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::BindingIdentifier
    }
    #[inline]
    pub fn is_simple_assignment_target(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference
                | NodeKind::MemberExpression
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
        )
    }
    #[inline]
    pub fn is_assignment_pattern(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::AssignmentPattern
    }
    #[inline]
    pub fn is_binding_rest_element(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::BindingRestElement
    }
    #[inline]
    pub fn is_array_pattern(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ArrayPattern
    }
    #[inline]
    pub fn is_object_pattern(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ObjectPattern
    }
    #[inline]
    pub fn as_binding_identifier(self, ast: &Ast<'_>) -> Option<BindingIdentifier> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::BindingIdentifier) {
            Some(BindingIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_simple_assignment_target(self, ast: &Ast<'_>) -> Option<SimpleAssignmentTarget> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(
            kind,
            NodeKind::IdentifierReference
                | NodeKind::MemberExpression
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
        ) {
            Some(SimpleAssignmentTarget(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_assignment_pattern(self, ast: &Ast<'_>) -> Option<AssignmentPattern> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::AssignmentPattern) {
            Some(AssignmentPattern(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_binding_rest_element(self, ast: &Ast<'_>) -> Option<BindingRestElement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::BindingRestElement) {
            Some(BindingRestElement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_array_pattern(self, ast: &Ast<'_>) -> Option<ArrayPattern> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ArrayPattern) {
            Some(ArrayPattern(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_object_pattern(self, ast: &Ast<'_>) -> Option<ObjectPattern> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ObjectPattern) {
            Some(ObjectPattern(self.0))
        } else {
            None
        }
    }
}
impl AssignmentTarget {
    #[inline]
    pub fn is_simple_assignment_target(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference
                | NodeKind::MemberExpression
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
        )
    }
    #[inline]
    pub fn is_array_assignment_target(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ArrayPattern
    }
    #[inline]
    pub fn is_object_assignment_target(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ObjectPattern
    }
    #[inline]
    pub fn as_simple_assignment_target(self, ast: &Ast<'_>) -> Option<SimpleAssignmentTarget> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(
            kind,
            NodeKind::IdentifierReference
                | NodeKind::MemberExpression
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
        ) {
            Some(SimpleAssignmentTarget(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_array_assignment_target(self, ast: &Ast<'_>) -> Option<ArrayPattern> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ArrayPattern) {
            Some(ArrayPattern(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_object_assignment_target(self, ast: &Ast<'_>) -> Option<ObjectPattern> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ObjectPattern) {
            Some(ObjectPattern(self.0))
        } else {
            None
        }
    }
}
impl SimpleAssignmentTarget {
    #[inline]
    pub fn is_identifier_reference(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierReference
    }
    #[inline]
    pub fn is_member_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::MemberExpression
    }
    #[inline]
    pub fn is_ts_as_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsAsExpression
    }
    #[inline]
    pub fn is_ts_satisfies_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsSatisfiesExpression
    }
    #[inline]
    pub fn is_ts_type_assertion(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypeAssertion
    }
    #[inline]
    pub fn is_ts_non_null_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsNonNullExpression
    }
    #[inline]
    pub fn as_identifier_reference(self, ast: &Ast<'_>) -> Option<IdentifierReference> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::IdentifierReference) {
            Some(IdentifierReference(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_member_expression(self, ast: &Ast<'_>) -> Option<MemberExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::MemberExpression) {
            Some(MemberExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_as_expression(self, ast: &Ast<'_>) -> Option<TsAsExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsAsExpression) {
            Some(TsAsExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_satisfies_expression(self, ast: &Ast<'_>) -> Option<TsSatisfiesExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsSatisfiesExpression) {
            Some(TsSatisfiesExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_assertion(self, ast: &Ast<'_>) -> Option<TsTypeAssertion> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsTypeAssertion) {
            Some(TsTypeAssertion(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_non_null_expression(self, ast: &Ast<'_>) -> Option<TsNonNullExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsNonNullExpression) {
            Some(TsNonNullExpression(self.0))
        } else {
            None
        }
    }
}
impl PropertyKey {
    #[inline]
    pub fn is_identifier_name(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierName
    }
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference
                | NodeKind::ThisExpression
                | NodeKind::Super
                | NodeKind::MetaProperty
                | NodeKind::StringLiteral
                | NodeKind::NumericLiteral
                | NodeKind::BigIntLiteral
                | NodeKind::BooleanLiteral
                | NodeKind::NullLiteral
                | NodeKind::RegExpLiteral
                | NodeKind::TemplateLiteral
                | NodeKind::BinaryExpression
                | NodeKind::LogicalExpression
                | NodeKind::UnaryExpression
                | NodeKind::UpdateExpression
                | NodeKind::AssignmentExpression
                | NodeKind::ConditionalExpression
                | NodeKind::SequenceExpression
                | NodeKind::MemberExpression
                | NodeKind::CallExpression
                | NodeKind::ChainExpression
                | NodeKind::NewExpression
                | NodeKind::TaggedTemplateExpression
                | NodeKind::ArrowFunctionExpression
                | NodeKind::ArrayExpression
                | NodeKind::ObjectExpression
                | NodeKind::ParenthesizedExpression
                | NodeKind::Function
                | NodeKind::Class
                | NodeKind::ImportExpression
                | NodeKind::AwaitExpression
                | NodeKind::YieldExpression
                | NodeKind::PrivateIdentifier
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::TsInstantiationExpression
                | NodeKind::JsxElement
                | NodeKind::JsxFragment
        )
    }
    #[inline]
    pub fn as_identifier_name(self, ast: &Ast<'_>) -> Option<IdentifierName> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::IdentifierName) {
            Some(IdentifierName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(
            kind,
            NodeKind::IdentifierReference
                | NodeKind::ThisExpression
                | NodeKind::Super
                | NodeKind::MetaProperty
                | NodeKind::StringLiteral
                | NodeKind::NumericLiteral
                | NodeKind::BigIntLiteral
                | NodeKind::BooleanLiteral
                | NodeKind::NullLiteral
                | NodeKind::RegExpLiteral
                | NodeKind::TemplateLiteral
                | NodeKind::BinaryExpression
                | NodeKind::LogicalExpression
                | NodeKind::UnaryExpression
                | NodeKind::UpdateExpression
                | NodeKind::AssignmentExpression
                | NodeKind::ConditionalExpression
                | NodeKind::SequenceExpression
                | NodeKind::MemberExpression
                | NodeKind::CallExpression
                | NodeKind::ChainExpression
                | NodeKind::NewExpression
                | NodeKind::TaggedTemplateExpression
                | NodeKind::ArrowFunctionExpression
                | NodeKind::ArrayExpression
                | NodeKind::ObjectExpression
                | NodeKind::ParenthesizedExpression
                | NodeKind::Function
                | NodeKind::Class
                | NodeKind::ImportExpression
                | NodeKind::AwaitExpression
                | NodeKind::YieldExpression
                | NodeKind::PrivateIdentifier
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::TsInstantiationExpression
                | NodeKind::JsxElement
                | NodeKind::JsxFragment
        ) {
            Some(Expr(self.0))
        } else {
            None
        }
    }
}
impl ModuleExportName {
    #[inline]
    pub fn is_identifier_name(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierName
    }
    #[inline]
    pub fn is_string_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::StringLiteral
    }
    #[inline]
    pub fn as_identifier_name(self, ast: &Ast<'_>) -> Option<IdentifierName> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::IdentifierName) {
            Some(IdentifierName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_>) -> Option<StringLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::StringLiteral) {
            Some(StringLiteral(self.0))
        } else {
            None
        }
    }
}
impl Argument {
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference
                | NodeKind::ThisExpression
                | NodeKind::Super
                | NodeKind::MetaProperty
                | NodeKind::StringLiteral
                | NodeKind::NumericLiteral
                | NodeKind::BigIntLiteral
                | NodeKind::BooleanLiteral
                | NodeKind::NullLiteral
                | NodeKind::RegExpLiteral
                | NodeKind::TemplateLiteral
                | NodeKind::BinaryExpression
                | NodeKind::LogicalExpression
                | NodeKind::UnaryExpression
                | NodeKind::UpdateExpression
                | NodeKind::AssignmentExpression
                | NodeKind::ConditionalExpression
                | NodeKind::SequenceExpression
                | NodeKind::MemberExpression
                | NodeKind::CallExpression
                | NodeKind::ChainExpression
                | NodeKind::NewExpression
                | NodeKind::TaggedTemplateExpression
                | NodeKind::ArrowFunctionExpression
                | NodeKind::ArrayExpression
                | NodeKind::ObjectExpression
                | NodeKind::ParenthesizedExpression
                | NodeKind::Function
                | NodeKind::Class
                | NodeKind::ImportExpression
                | NodeKind::AwaitExpression
                | NodeKind::YieldExpression
                | NodeKind::PrivateIdentifier
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::TsInstantiationExpression
                | NodeKind::JsxElement
                | NodeKind::JsxFragment
        )
    }
    #[inline]
    pub fn is_spread_element(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::SpreadElement
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(
            kind,
            NodeKind::IdentifierReference
                | NodeKind::ThisExpression
                | NodeKind::Super
                | NodeKind::MetaProperty
                | NodeKind::StringLiteral
                | NodeKind::NumericLiteral
                | NodeKind::BigIntLiteral
                | NodeKind::BooleanLiteral
                | NodeKind::NullLiteral
                | NodeKind::RegExpLiteral
                | NodeKind::TemplateLiteral
                | NodeKind::BinaryExpression
                | NodeKind::LogicalExpression
                | NodeKind::UnaryExpression
                | NodeKind::UpdateExpression
                | NodeKind::AssignmentExpression
                | NodeKind::ConditionalExpression
                | NodeKind::SequenceExpression
                | NodeKind::MemberExpression
                | NodeKind::CallExpression
                | NodeKind::ChainExpression
                | NodeKind::NewExpression
                | NodeKind::TaggedTemplateExpression
                | NodeKind::ArrowFunctionExpression
                | NodeKind::ArrayExpression
                | NodeKind::ObjectExpression
                | NodeKind::ParenthesizedExpression
                | NodeKind::Function
                | NodeKind::Class
                | NodeKind::ImportExpression
                | NodeKind::AwaitExpression
                | NodeKind::YieldExpression
                | NodeKind::PrivateIdentifier
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::TsInstantiationExpression
                | NodeKind::JsxElement
                | NodeKind::JsxFragment
        ) {
            Some(Expr(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_spread_element(self, ast: &Ast<'_>) -> Option<SpreadElement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::SpreadElement) {
            Some(SpreadElement(self.0))
        } else {
            None
        }
    }
}
impl ObjectPropertyKind {
    #[inline]
    pub fn is_object_property(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ObjectProperty
    }
    #[inline]
    pub fn is_spread_element(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::SpreadElement
    }
    #[inline]
    pub fn as_object_property(self, ast: &Ast<'_>) -> Option<ObjectProperty> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ObjectProperty) {
            Some(ObjectProperty(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_spread_element(self, ast: &Ast<'_>) -> Option<SpreadElement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::SpreadElement) {
            Some(SpreadElement(self.0))
        } else {
            None
        }
    }
}
impl ForStatementInit {
    #[inline]
    pub fn is_variable_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::VariableDeclaration
    }
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference
                | NodeKind::ThisExpression
                | NodeKind::Super
                | NodeKind::MetaProperty
                | NodeKind::StringLiteral
                | NodeKind::NumericLiteral
                | NodeKind::BigIntLiteral
                | NodeKind::BooleanLiteral
                | NodeKind::NullLiteral
                | NodeKind::RegExpLiteral
                | NodeKind::TemplateLiteral
                | NodeKind::BinaryExpression
                | NodeKind::LogicalExpression
                | NodeKind::UnaryExpression
                | NodeKind::UpdateExpression
                | NodeKind::AssignmentExpression
                | NodeKind::ConditionalExpression
                | NodeKind::SequenceExpression
                | NodeKind::MemberExpression
                | NodeKind::CallExpression
                | NodeKind::ChainExpression
                | NodeKind::NewExpression
                | NodeKind::TaggedTemplateExpression
                | NodeKind::ArrowFunctionExpression
                | NodeKind::ArrayExpression
                | NodeKind::ObjectExpression
                | NodeKind::ParenthesizedExpression
                | NodeKind::Function
                | NodeKind::Class
                | NodeKind::ImportExpression
                | NodeKind::AwaitExpression
                | NodeKind::YieldExpression
                | NodeKind::PrivateIdentifier
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::TsInstantiationExpression
                | NodeKind::JsxElement
                | NodeKind::JsxFragment
        )
    }
    #[inline]
    pub fn as_variable_declaration(self, ast: &Ast<'_>) -> Option<VariableDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::VariableDeclaration) {
            Some(VariableDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(
            kind,
            NodeKind::IdentifierReference
                | NodeKind::ThisExpression
                | NodeKind::Super
                | NodeKind::MetaProperty
                | NodeKind::StringLiteral
                | NodeKind::NumericLiteral
                | NodeKind::BigIntLiteral
                | NodeKind::BooleanLiteral
                | NodeKind::NullLiteral
                | NodeKind::RegExpLiteral
                | NodeKind::TemplateLiteral
                | NodeKind::BinaryExpression
                | NodeKind::LogicalExpression
                | NodeKind::UnaryExpression
                | NodeKind::UpdateExpression
                | NodeKind::AssignmentExpression
                | NodeKind::ConditionalExpression
                | NodeKind::SequenceExpression
                | NodeKind::MemberExpression
                | NodeKind::CallExpression
                | NodeKind::ChainExpression
                | NodeKind::NewExpression
                | NodeKind::TaggedTemplateExpression
                | NodeKind::ArrowFunctionExpression
                | NodeKind::ArrayExpression
                | NodeKind::ObjectExpression
                | NodeKind::ParenthesizedExpression
                | NodeKind::Function
                | NodeKind::Class
                | NodeKind::ImportExpression
                | NodeKind::AwaitExpression
                | NodeKind::YieldExpression
                | NodeKind::PrivateIdentifier
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::TsInstantiationExpression
                | NodeKind::JsxElement
                | NodeKind::JsxFragment
        ) {
            Some(Expr(self.0))
        } else {
            None
        }
    }
}
impl ForStatementLeft {
    #[inline]
    pub fn is_variable_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::VariableDeclaration
    }
    #[inline]
    pub fn is_assignment_target(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference
                | NodeKind::MemberExpression
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::ArrayPattern
                | NodeKind::ObjectPattern
        )
    }
    #[inline]
    pub fn as_variable_declaration(self, ast: &Ast<'_>) -> Option<VariableDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::VariableDeclaration) {
            Some(VariableDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_assignment_target(self, ast: &Ast<'_>) -> Option<AssignmentTarget> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(
            kind,
            NodeKind::IdentifierReference
                | NodeKind::MemberExpression
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::ArrayPattern
                | NodeKind::ObjectPattern
        ) {
            Some(AssignmentTarget(self.0))
        } else {
            None
        }
    }
}
impl ArrowFunctionBody {
    #[inline]
    pub fn is_function_body(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::FunctionBody
    }
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference
                | NodeKind::ThisExpression
                | NodeKind::Super
                | NodeKind::MetaProperty
                | NodeKind::StringLiteral
                | NodeKind::NumericLiteral
                | NodeKind::BigIntLiteral
                | NodeKind::BooleanLiteral
                | NodeKind::NullLiteral
                | NodeKind::RegExpLiteral
                | NodeKind::TemplateLiteral
                | NodeKind::BinaryExpression
                | NodeKind::LogicalExpression
                | NodeKind::UnaryExpression
                | NodeKind::UpdateExpression
                | NodeKind::AssignmentExpression
                | NodeKind::ConditionalExpression
                | NodeKind::SequenceExpression
                | NodeKind::MemberExpression
                | NodeKind::CallExpression
                | NodeKind::ChainExpression
                | NodeKind::NewExpression
                | NodeKind::TaggedTemplateExpression
                | NodeKind::ArrowFunctionExpression
                | NodeKind::ArrayExpression
                | NodeKind::ObjectExpression
                | NodeKind::ParenthesizedExpression
                | NodeKind::Function
                | NodeKind::Class
                | NodeKind::ImportExpression
                | NodeKind::AwaitExpression
                | NodeKind::YieldExpression
                | NodeKind::PrivateIdentifier
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::TsInstantiationExpression
                | NodeKind::JsxElement
                | NodeKind::JsxFragment
        )
    }
    #[inline]
    pub fn as_function_body(self, ast: &Ast<'_>) -> Option<FunctionBody> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::FunctionBody) {
            Some(FunctionBody(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(
            kind,
            NodeKind::IdentifierReference
                | NodeKind::ThisExpression
                | NodeKind::Super
                | NodeKind::MetaProperty
                | NodeKind::StringLiteral
                | NodeKind::NumericLiteral
                | NodeKind::BigIntLiteral
                | NodeKind::BooleanLiteral
                | NodeKind::NullLiteral
                | NodeKind::RegExpLiteral
                | NodeKind::TemplateLiteral
                | NodeKind::BinaryExpression
                | NodeKind::LogicalExpression
                | NodeKind::UnaryExpression
                | NodeKind::UpdateExpression
                | NodeKind::AssignmentExpression
                | NodeKind::ConditionalExpression
                | NodeKind::SequenceExpression
                | NodeKind::MemberExpression
                | NodeKind::CallExpression
                | NodeKind::ChainExpression
                | NodeKind::NewExpression
                | NodeKind::TaggedTemplateExpression
                | NodeKind::ArrowFunctionExpression
                | NodeKind::ArrayExpression
                | NodeKind::ObjectExpression
                | NodeKind::ParenthesizedExpression
                | NodeKind::Function
                | NodeKind::Class
                | NodeKind::ImportExpression
                | NodeKind::AwaitExpression
                | NodeKind::YieldExpression
                | NodeKind::PrivateIdentifier
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::TsInstantiationExpression
                | NodeKind::JsxElement
                | NodeKind::JsxFragment
        ) {
            Some(Expr(self.0))
        } else {
            None
        }
    }
}
impl FormalParameterItem {
    #[inline]
    pub fn is_formal_parameter(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::FormalParameter
    }
    #[inline]
    pub fn is_ts_parameter_property(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsParameterProperty
    }
    #[inline]
    pub fn as_formal_parameter(self, ast: &Ast<'_>) -> Option<FormalParameter> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::FormalParameter) {
            Some(FormalParameter(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_parameter_property(self, ast: &Ast<'_>) -> Option<TsParameterProperty> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsParameterProperty) {
            Some(TsParameterProperty(self.0))
        } else {
            None
        }
    }
}
impl FormalParameterPattern {
    #[inline]
    pub fn is_binding_identifier(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::BindingIdentifier
    }
    #[inline]
    pub fn is_array_pattern(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ArrayPattern
    }
    #[inline]
    pub fn is_object_pattern(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ObjectPattern
    }
    #[inline]
    pub fn as_binding_identifier(self, ast: &Ast<'_>) -> Option<BindingIdentifier> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::BindingIdentifier) {
            Some(BindingIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_array_pattern(self, ast: &Ast<'_>) -> Option<ArrayPattern> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ArrayPattern) {
            Some(ArrayPattern(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_object_pattern(self, ast: &Ast<'_>) -> Option<ObjectPattern> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ObjectPattern) {
            Some(ObjectPattern(self.0))
        } else {
            None
        }
    }
}
impl ClassElement {
    #[inline]
    pub fn is_method_definition(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::MethodDefinition
    }
    #[inline]
    pub fn is_property_definition(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::PropertyDefinition
    }
    #[inline]
    pub fn is_static_block(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::StaticBlock
    }
    #[inline]
    pub fn is_ts_method_definition(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsMethodDefinition
    }
    #[inline]
    pub fn is_ts_index_signature(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsIndexSignature
    }
    #[inline]
    pub fn as_method_definition(self, ast: &Ast<'_>) -> Option<MethodDefinition> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::MethodDefinition) {
            Some(MethodDefinition(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_property_definition(self, ast: &Ast<'_>) -> Option<PropertyDefinition> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::PropertyDefinition) {
            Some(PropertyDefinition(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_static_block(self, ast: &Ast<'_>) -> Option<StaticBlock> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::StaticBlock) {
            Some(StaticBlock(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_method_definition(self, ast: &Ast<'_>) -> Option<TsMethodDefinition> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsMethodDefinition) {
            Some(TsMethodDefinition(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_index_signature(self, ast: &Ast<'_>) -> Option<TsIndexSignature> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsIndexSignature) {
            Some(TsIndexSignature(self.0))
        } else {
            None
        }
    }
}
impl ImportDeclarationSpecifier {
    #[inline]
    pub fn is_import_specifier(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ImportSpecifier
    }
    #[inline]
    pub fn is_import_default_specifier(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ImportDefaultSpecifier
    }
    #[inline]
    pub fn is_import_namespace_specifier(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ImportNamespaceSpecifier
    }
    #[inline]
    pub fn as_import_specifier(self, ast: &Ast<'_>) -> Option<ImportSpecifier> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ImportSpecifier) {
            Some(ImportSpecifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_import_default_specifier(self, ast: &Ast<'_>) -> Option<ImportDefaultSpecifier> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ImportDefaultSpecifier) {
            Some(ImportDefaultSpecifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_import_namespace_specifier(self, ast: &Ast<'_>) -> Option<ImportNamespaceSpecifier> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ImportNamespaceSpecifier) {
            Some(ImportNamespaceSpecifier(self.0))
        } else {
            None
        }
    }
}
impl ExportDefaultDeclarationKind {
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        match kind {
            NodeKind::IdentifierReference => true,
            NodeKind::ThisExpression => true,
            NodeKind::Super => true,
            NodeKind::MetaProperty => true,
            NodeKind::StringLiteral => true,
            NodeKind::NumericLiteral => true,
            NodeKind::BigIntLiteral => true,
            NodeKind::BooleanLiteral => true,
            NodeKind::NullLiteral => true,
            NodeKind::RegExpLiteral => true,
            NodeKind::TemplateLiteral => true,
            NodeKind::BinaryExpression => true,
            NodeKind::LogicalExpression => true,
            NodeKind::UnaryExpression => true,
            NodeKind::UpdateExpression => true,
            NodeKind::AssignmentExpression => true,
            NodeKind::ConditionalExpression => true,
            NodeKind::SequenceExpression => true,
            NodeKind::MemberExpression => true,
            NodeKind::CallExpression => true,
            NodeKind::ChainExpression => true,
            NodeKind::NewExpression => true,
            NodeKind::TaggedTemplateExpression => true,
            NodeKind::ArrowFunctionExpression => true,
            NodeKind::ArrayExpression => true,
            NodeKind::ObjectExpression => true,
            NodeKind::ParenthesizedExpression => true,
            NodeKind::ImportExpression => true,
            NodeKind::AwaitExpression => true,
            NodeKind::YieldExpression => true,
            NodeKind::PrivateIdentifier => true,
            NodeKind::TsAsExpression => true,
            NodeKind::TsSatisfiesExpression => true,
            NodeKind::TsTypeAssertion => true,
            NodeKind::TsNonNullExpression => true,
            NodeKind::TsInstantiationExpression => true,
            NodeKind::JsxElement => true,
            NodeKind::JsxFragment => true,
            NodeKind::Function => {
                let function = Function(self.0);
                function.function_type(ast) != FunctionType::FunctionDeclaration
            }
            NodeKind::Class => {
                let class = Class(self.0);
                class.class_type(ast) != ClassType::ClassDeclaration
            }
            _ => false,
        }
    }
    #[inline]
    pub fn is_function(self, ast: &Ast<'_>) -> bool {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if kind == NodeKind::Function {
            let function = Function(self.0);
            function.function_type(ast) == FunctionType::FunctionDeclaration
        } else {
            false
        }
    }
    #[inline]
    pub fn is_class(self, ast: &Ast<'_>) -> bool {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if kind == NodeKind::Class {
            let class = Class(self.0);
            class.class_type(ast) == ClassType::ClassDeclaration
        } else {
            false
        }
    }
    #[inline]
    pub fn is_ts_function(self, ast: &Ast<'_>) -> bool {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        matches!(kind, NodeKind::TsFunction)
    }
    #[inline]
    pub fn is_ts_interface_declaration(self, ast: &Ast<'_>) -> bool {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        matches!(kind, NodeKind::TsInterfaceDeclaration)
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if match kind {
            NodeKind::IdentifierReference => true,
            NodeKind::ThisExpression => true,
            NodeKind::Super => true,
            NodeKind::MetaProperty => true,
            NodeKind::StringLiteral => true,
            NodeKind::NumericLiteral => true,
            NodeKind::BigIntLiteral => true,
            NodeKind::BooleanLiteral => true,
            NodeKind::NullLiteral => true,
            NodeKind::RegExpLiteral => true,
            NodeKind::TemplateLiteral => true,
            NodeKind::BinaryExpression => true,
            NodeKind::LogicalExpression => true,
            NodeKind::UnaryExpression => true,
            NodeKind::UpdateExpression => true,
            NodeKind::AssignmentExpression => true,
            NodeKind::ConditionalExpression => true,
            NodeKind::SequenceExpression => true,
            NodeKind::MemberExpression => true,
            NodeKind::CallExpression => true,
            NodeKind::ChainExpression => true,
            NodeKind::NewExpression => true,
            NodeKind::TaggedTemplateExpression => true,
            NodeKind::ArrowFunctionExpression => true,
            NodeKind::ArrayExpression => true,
            NodeKind::ObjectExpression => true,
            NodeKind::ParenthesizedExpression => true,
            NodeKind::ImportExpression => true,
            NodeKind::AwaitExpression => true,
            NodeKind::YieldExpression => true,
            NodeKind::PrivateIdentifier => true,
            NodeKind::TsAsExpression => true,
            NodeKind::TsSatisfiesExpression => true,
            NodeKind::TsTypeAssertion => true,
            NodeKind::TsNonNullExpression => true,
            NodeKind::TsInstantiationExpression => true,
            NodeKind::JsxElement => true,
            NodeKind::JsxFragment => true,
            NodeKind::Function => {
                let function = Function(self.0);
                function.function_type(ast) != FunctionType::FunctionDeclaration
            }
            NodeKind::Class => {
                let class = Class(self.0);
                class.class_type(ast) != ClassType::ClassDeclaration
            }
            _ => false,
        } {
            Some(Expr(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_function(self, ast: &Ast<'_>) -> Option<Function> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if if kind == NodeKind::Function {
            let function = Function(self.0);
            function.function_type(ast) == FunctionType::FunctionDeclaration
        } else {
            false
        } {
            Some(Function(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_class(self, ast: &Ast<'_>) -> Option<Class> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if if kind == NodeKind::Class {
            let class = Class(self.0);
            class.class_type(ast) == ClassType::ClassDeclaration
        } else {
            false
        } {
            Some(Class(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_function(self, ast: &Ast<'_>) -> Option<TsFunction> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsFunction) {
            Some(TsFunction(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_interface_declaration(self, ast: &Ast<'_>) -> Option<TsInterfaceDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsInterfaceDeclaration) {
            Some(TsInterfaceDeclaration(self.0))
        } else {
            None
        }
    }
}
impl Super {}
impl ThisExpression {}
impl StringLiteral {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> AstWtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).string_literal.value }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).string_literal.raw }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: AstWtf8) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).string_literal.value = value;
        }
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: AstUtf8) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).string_literal.raw = raw;
        }
    }
}
impl NumericLiteral {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> f64 {
        unsafe { ast.nodes.data_unchecked(self.0).numeric_literal.value }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> Option<AstUtf8> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).numeric_literal.raw };
        (raw != u64::from(u32::MAX)).then(|| AstUtf8::from_raw(raw))
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: f64) {
        if unsafe { ast.nodes.data_unchecked(self.0).numeric_literal.value }.to_bits()
            == value.to_bits()
        {
            return;
        }
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).numeric_literal.value = value;
        }
        self.set_raw(ast, None);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: Option<AstUtf8>) {
        let field_val: u64 = raw.map_or(u64::from(u32::MAX), AstUtf8::raw);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).numeric_literal.raw = field_val;
        }
    }
}
impl BigIntLiteral {
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).big_int_literal.raw }
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: AstUtf8) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).big_int_literal.raw = raw;
        }
    }
}
impl BooleanLiteral {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).boolean_literal.value }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).boolean_literal.value = value;
        }
    }
}
impl NullLiteral {}
impl RegExpLiteral {
    #[inline]
    pub fn pattern<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).reg_exp_literal.pattern }
    }
    #[inline]
    pub fn flags<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).reg_exp_literal.flags }
    }
    #[inline]
    pub fn set_pattern<'a>(self, ast: &mut crate::Ast<'a>, pattern: AstUtf8) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).reg_exp_literal.pattern = pattern;
        }
    }
    #[inline]
    pub fn set_flags<'a>(self, ast: &mut crate::Ast<'a>, flags: AstUtf8) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).reg_exp_literal.flags = flags;
        }
    }
}
impl TemplateLiteral {
    #[inline]
    pub fn quasis<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TemplateElement> {
        unsafe { ast.nodes.data_unchecked(self.0).template_literal.quasis }
    }
    #[inline]
    pub fn expressions<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Expr> {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .template_literal
                .expressions
        }
    }
    #[inline]
    pub fn set_quasis<'a>(self, ast: &mut crate::Ast<'a>, quasis: SubRange<TemplateElement>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).template_literal.quasis = quasis;
        }
    }
    #[inline]
    pub fn set_expressions<'a>(self, ast: &mut crate::Ast<'a>, expressions: SubRange<Expr>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .template_literal
                .expressions = expressions;
        }
    }
}
impl TemplateElement {
    #[inline]
    pub fn cooked<'a>(self, ast: &crate::Ast<'a>) -> AstWtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).template_element.cooked }
    }
    #[inline]
    pub fn tail<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).template_element.tail }
    }
    #[inline]
    pub fn is_cooked_undefined<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .template_element
                .is_cooked_undefined
        }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .template_element
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_cooked<'a>(self, ast: &mut crate::Ast<'a>, cooked: AstWtf8) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).template_element.cooked = cooked;
        }
    }
    #[inline]
    pub fn set_tail<'a>(self, ast: &mut crate::Ast<'a>, tail: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).template_element.tail = tail;
        }
    }
    #[inline]
    pub fn set_is_cooked_undefined<'a>(self, ast: &mut crate::Ast<'a>, is_cooked_undefined: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .template_element
                .is_cooked_undefined = is_cooked_undefined;
        }
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: AstUtf8) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .template_element
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = raw.into_extra_data();
        };
    }
}
impl IdentifierReference {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).identifier_reference.name }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .identifier_reference
                .name = name;
        }
    }
}
impl PrivateIdentifier {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).private_identifier.name }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).private_identifier.name = name;
        }
    }
}
impl BindingIdentifier {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).binding_identifier.name }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).binding_identifier.name = name;
        }
    }
}
impl IdentifierName {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).identifier_name.name }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).identifier_name.name = name;
        }
    }
}
impl LabelIdentifier {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).label_identifier.name }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).label_identifier.name = name;
        }
    }
}
impl AssignmentPattern {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> BindingPattern {
        unsafe { ast.nodes.data_unchecked(self.0).assignment_pattern.left }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).assignment_pattern.right }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: BindingPattern) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).assignment_pattern.left = left;
        }
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .assignment_pattern
                .right = right;
        }
    }
}
impl BindingRestElement {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> BindingPattern {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .binding_rest_element
                .argument
        }
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: BindingPattern) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .binding_rest_element
                .argument = argument;
        }
    }
}
impl ArrayPattern {
    #[inline]
    pub fn elements<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Option<BindingPattern>> {
        unsafe { ast.nodes.data_unchecked(self.0).array_pattern.elements }
    }
    #[inline]
    pub fn rest<'a>(self, ast: &crate::Ast<'a>) -> Option<BindingRestElement> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).array_pattern.rest };
        (raw != u32::MAX).then(|| BindingRestElement(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_elements<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        elements: SubRange<Option<BindingPattern>>,
    ) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).array_pattern.elements = elements;
        }
    }
    #[inline]
    pub fn set_rest<'a>(self, ast: &mut crate::Ast<'a>, rest: Option<BindingRestElement>) {
        let field_val: u32 = rest.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).array_pattern.rest = field_val;
        }
    }
}
impl ObjectPattern {
    #[inline]
    pub fn properties<'a>(self, ast: &crate::Ast<'a>) -> SubRange<BindingProperty> {
        unsafe { ast.nodes.data_unchecked(self.0).object_pattern.properties }
    }
    #[inline]
    pub fn rest<'a>(self, ast: &crate::Ast<'a>) -> Option<BindingRestElement> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).object_pattern.rest };
        (raw != u32::MAX).then(|| BindingRestElement(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_properties<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        properties: SubRange<BindingProperty>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .object_pattern
                .properties = properties;
        }
    }
    #[inline]
    pub fn set_rest<'a>(self, ast: &mut crate::Ast<'a>, rest: Option<BindingRestElement>) {
        let field_val: u32 = rest.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).object_pattern.rest = field_val;
        }
    }
}
impl BindingProperty {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        unsafe { ast.nodes.data_unchecked(self.0).binding_property.key }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> BindingPattern {
        unsafe { ast.nodes.data_unchecked(self.0).binding_property.value }
    }
    #[inline]
    pub fn shorthand<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).binding_property.shorthand }
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).binding_property.computed }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropertyKey) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).binding_property.key = key;
        }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: BindingPattern) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).binding_property.value = value;
        }
    }
    #[inline]
    pub fn set_shorthand<'a>(self, ast: &mut crate::Ast<'a>, shorthand: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .binding_property
                .shorthand = shorthand;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .binding_property
                .computed = computed;
        }
    }
}
impl BinaryExpression {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).binary_expression.left }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).binary_expression.right }
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a>) -> BinaryOperator {
        unsafe { ast.nodes.data_unchecked(self.0).binary_expression.operator }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: Expr) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).binary_expression.left = left;
        }
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).binary_expression.right = right;
        }
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a>, operator: BinaryOperator) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .binary_expression
                .operator = operator;
        }
    }
}
impl LogicalExpression {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).logical_expression.left }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).logical_expression.right }
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a>) -> LogicalOperator {
        unsafe { ast.nodes.data_unchecked(self.0).logical_expression.operator }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: Expr) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).logical_expression.left = left;
        }
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .logical_expression
                .right = right;
        }
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a>, operator: LogicalOperator) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .logical_expression
                .operator = operator;
        }
    }
}
impl ConditionalExpression {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).conditional_expression.test }
    }
    #[inline]
    pub fn consequent<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .conditional_expression
                .consequent
        }
    }
    #[inline]
    pub fn alternate<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .conditional_expression
                .alternate
        }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .conditional_expression
                .test = test;
        }
    }
    #[inline]
    pub fn set_consequent<'a>(self, ast: &mut crate::Ast<'a>, consequent: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .conditional_expression
                .consequent = consequent;
        }
    }
    #[inline]
    pub fn set_alternate<'a>(self, ast: &mut crate::Ast<'a>, alternate: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .conditional_expression
                .alternate = alternate;
        }
    }
}
impl UnaryExpression {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).unary_expression.argument }
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a>) -> UnaryOperator {
        unsafe { ast.nodes.data_unchecked(self.0).unary_expression.operator }
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .unary_expression
                .argument = argument;
        }
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a>, operator: UnaryOperator) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .unary_expression
                .operator = operator;
        }
    }
}
impl UpdateExpression {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> SimpleAssignmentTarget {
        unsafe { ast.nodes.data_unchecked(self.0).update_expression.argument }
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a>) -> UpdateOperator {
        unsafe { ast.nodes.data_unchecked(self.0).update_expression.operator }
    }
    #[inline]
    pub fn prefix<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).update_expression.prefix }
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: SimpleAssignmentTarget) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .update_expression
                .argument = argument;
        }
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a>, operator: UpdateOperator) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .update_expression
                .operator = operator;
        }
    }
    #[inline]
    pub fn set_prefix<'a>(self, ast: &mut crate::Ast<'a>, prefix: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .update_expression
                .prefix = prefix;
        }
    }
}
impl AssignmentExpression {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> AssignmentTarget {
        unsafe { ast.nodes.data_unchecked(self.0).assignment_expression.left }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).assignment_expression.right }
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a>) -> AssignmentOperator {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .assignment_expression
                .operator
        }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: AssignmentTarget) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .assignment_expression
                .left = left;
        }
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .assignment_expression
                .right = right;
        }
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a>, operator: AssignmentOperator) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .assignment_expression
                .operator = operator;
        }
    }
}
impl VariableDeclaration {
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> VariableKind {
        unsafe { ast.nodes.data_unchecked(self.0).variable_declaration.kind }
    }
    #[inline]
    pub fn declarators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<VariableDeclarator> {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .variable_declaration
                .declarators
        }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .variable_declaration
                .declare
        }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: VariableKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .variable_declaration
                .kind = kind;
        }
    }
    #[inline]
    pub fn set_declarators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        declarators: SubRange<VariableDeclarator>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .variable_declaration
                .declarators = declarators;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .variable_declaration
                .declare = declare;
        }
    }
}
impl VariableDeclarator {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> BindingPattern {
        unsafe { ast.nodes.data_unchecked(self.0).variable_declarator.id }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .variable_declarator
                .type_annotation
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn init<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).variable_declarator.init };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn definite<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .variable_declarator
                .definite
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: BindingPattern) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).variable_declarator.id = id;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .variable_declarator
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_init<'a>(self, ast: &mut crate::Ast<'a>, init: Option<Expr>) {
        let field_val: u32 = init.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .variable_declarator
                .init = field_val;
        }
    }
    #[inline]
    pub fn set_definite<'a>(self, ast: &mut crate::Ast<'a>, definite: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .variable_declarator
                .definite = definite;
        }
    }
}
impl IfStatement {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).if_statement.test }
    }
    #[inline]
    pub fn consequent<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        unsafe { ast.nodes.data_unchecked(self.0).if_statement.consequent }
    }
    #[inline]
    pub fn alternate<'a>(self, ast: &crate::Ast<'a>) -> Option<Stmt> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).if_statement.alternate };
        (raw != u32::MAX).then(|| Stmt(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).if_statement.test = test;
        }
    }
    #[inline]
    pub fn set_consequent<'a>(self, ast: &mut crate::Ast<'a>, consequent: Stmt) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).if_statement.consequent = consequent;
        }
    }
    #[inline]
    pub fn set_alternate<'a>(self, ast: &mut crate::Ast<'a>, alternate: Option<Stmt>) {
        let field_val: u32 = alternate.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).if_statement.alternate = field_val;
        }
    }
}
impl SwitchStatement {
    #[inline]
    pub fn discriminant<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .switch_statement
                .discriminant
        }
    }
    #[inline]
    pub fn cases<'a>(self, ast: &crate::Ast<'a>) -> SubRange<SwitchCase> {
        unsafe { ast.nodes.data_unchecked(self.0).switch_statement.cases }
    }
    #[inline]
    pub fn set_discriminant<'a>(self, ast: &mut crate::Ast<'a>, discriminant: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .switch_statement
                .discriminant = discriminant;
        }
    }
    #[inline]
    pub fn set_cases<'a>(self, ast: &mut crate::Ast<'a>, cases: SubRange<SwitchCase>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).switch_statement.cases = cases;
        }
    }
}
impl SwitchCase {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).switch_case.test };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn consequent<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Stmt> {
        unsafe { ast.nodes.data_unchecked(self.0).switch_case.consequent }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Option<Expr>) {
        let field_val: u32 = test.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).switch_case.test = field_val;
        }
    }
    #[inline]
    pub fn set_consequent<'a>(self, ast: &mut crate::Ast<'a>, consequent: SubRange<Stmt>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).switch_case.consequent = consequent;
        }
    }
}
impl ForStatement {
    #[inline]
    pub fn init<'a>(self, ast: &crate::Ast<'a>) -> Option<ForStatementInit> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).for_statement.init };
        (raw != u32::MAX).then(|| ForStatementInit(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).for_statement.test };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn update<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).for_statement.update };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        unsafe { ast.nodes.data_unchecked(self.0).for_statement.body }
    }
    #[inline]
    pub fn set_init<'a>(self, ast: &mut crate::Ast<'a>, init: Option<ForStatementInit>) {
        let field_val: u32 = init.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).for_statement.init = field_val;
        }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Option<Expr>) {
        let field_val: u32 = test.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).for_statement.test = field_val;
        }
    }
    #[inline]
    pub fn set_update<'a>(self, ast: &mut crate::Ast<'a>, update: Option<Expr>) {
        let field_val: u32 = update.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).for_statement.update = field_val;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).for_statement.body = body;
        }
    }
}
impl ForInStatement {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> ForStatementLeft {
        unsafe { ast.nodes.data_unchecked(self.0).for_in_statement.left }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).for_in_statement.right }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        unsafe { ast.nodes.data_unchecked(self.0).for_in_statement.body }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: ForStatementLeft) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).for_in_statement.left = left;
        }
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).for_in_statement.right = right;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).for_in_statement.body = body;
        }
    }
}
impl ForOfStatement {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> ForStatementLeft {
        unsafe { ast.nodes.data_unchecked(self.0).for_of_statement.left }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).for_of_statement.right }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        unsafe { ast.nodes.data_unchecked(self.0).for_of_statement.body }
    }
    #[inline]
    pub fn r#await<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).for_of_statement.r#await }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: ForStatementLeft) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).for_of_statement.left = left;
        }
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).for_of_statement.right = right;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).for_of_statement.body = body;
        }
    }
    #[inline]
    pub fn set_await<'a>(self, ast: &mut crate::Ast<'a>, r#await: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .for_of_statement
                .r#await = r#await;
        }
    }
}
impl TryStatement {
    #[inline]
    pub fn block<'a>(self, ast: &crate::Ast<'a>) -> BlockStatement {
        unsafe { ast.nodes.data_unchecked(self.0).try_statement.block }
    }
    #[inline]
    pub fn handler<'a>(self, ast: &crate::Ast<'a>) -> Option<CatchClause> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).try_statement.handler };
        (raw != u32::MAX).then(|| CatchClause(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn finalizer<'a>(self, ast: &crate::Ast<'a>) -> Option<BlockStatement> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).try_statement.finalizer };
        (raw != u32::MAX).then(|| BlockStatement(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_block<'a>(self, ast: &mut crate::Ast<'a>, block: BlockStatement) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).try_statement.block = block;
        }
    }
    #[inline]
    pub fn set_handler<'a>(self, ast: &mut crate::Ast<'a>, handler: Option<CatchClause>) {
        let field_val: u32 = handler.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).try_statement.handler = field_val;
        }
    }
    #[inline]
    pub fn set_finalizer<'a>(self, ast: &mut crate::Ast<'a>, finalizer: Option<BlockStatement>) {
        let field_val: u32 = finalizer.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).try_statement.finalizer = field_val;
        }
    }
}
impl CatchClause {
    #[inline]
    pub fn param<'a>(self, ast: &crate::Ast<'a>) -> Option<BindingPattern> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).catch_clause.param };
        (raw != u32::MAX).then(|| BindingPattern(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .catch_clause
                .type_annotation
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> BlockStatement {
        unsafe { ast.nodes.data_unchecked(self.0).catch_clause.body }
    }
    #[inline]
    pub fn set_param<'a>(self, ast: &mut crate::Ast<'a>, param: Option<BindingPattern>) {
        let field_val: u32 = param.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).catch_clause.param = field_val;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .catch_clause
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: BlockStatement) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).catch_clause.body = body;
        }
    }
}
impl BreakStatement {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> Option<LabelIdentifier> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).break_statement.label };
        (raw != u32::MAX).then(|| LabelIdentifier(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: Option<LabelIdentifier>) {
        let field_val: u32 = label.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).break_statement.label = field_val;
        }
    }
}
impl ContinueStatement {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> Option<LabelIdentifier> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).continue_statement.label };
        (raw != u32::MAX).then(|| LabelIdentifier(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: Option<LabelIdentifier>) {
        let field_val: u32 = label.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .continue_statement
                .label = field_val;
        }
    }
}
impl LabeledStatement {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> LabelIdentifier {
        unsafe { ast.nodes.data_unchecked(self.0).labeled_statement.label }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        unsafe { ast.nodes.data_unchecked(self.0).labeled_statement.body }
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: LabelIdentifier) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).labeled_statement.label = label;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).labeled_statement.body = body;
        }
    }
}
impl ExpressionStatement {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .expression_statement
                .expression
        }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .expression_statement
                .expression = expression;
        }
    }
}
impl ReturnStatement {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).return_statement.argument };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: Option<Expr>) {
        let field_val: u32 = argument.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .return_statement
                .argument = field_val;
        }
    }
}
impl ThrowStatement {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).throw_statement.argument }
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .throw_statement
                .argument = argument;
        }
    }
}
impl DebuggerStatement {}
impl EmptyStatement {}
impl Directive {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> StringLiteral {
        unsafe { ast.nodes.data_unchecked(self.0).directive.expression }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).directive.value }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: StringLiteral) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).directive.expression = expression;
        }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: AstUtf8) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).directive.value = value;
        }
    }
}
impl Decorator {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).decorator.expression }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).decorator.expression = expression;
        }
    }
}
impl Class {
    #[inline]
    pub fn class_type<'a>(self, ast: &crate::Ast<'a>) -> ClassType {
        unsafe { ast.nodes.data_unchecked(self.0).class.class_type }
    }
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> Option<BindingIdentifier> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).class.id };
        (raw != u32::MAX).then(|| BindingIdentifier(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).class.type_parameters };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).class.r#abstract }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).class.declare }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Decorator> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn super_class<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn super_type_arguments<'a>(
        self,
        ast: &crate::Ast<'a>,
    ) -> Option<TsTypeParameterInstantiation> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn implements<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsClassImplements> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> ClassBody {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(4usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_class_type<'a>(self, ast: &mut crate::Ast<'a>, class_type: ClassType) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).class.class_type = class_type;
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: Option<BindingIdentifier>) {
        let field_val: u32 = id.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).class.id = field_val;
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).class.type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a>, r#abstract: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).class.r#abstract = r#abstract;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).class.declare = declare;
        }
    }
    #[inline]
    pub fn set_decorators<'a>(self, ast: &mut crate::Ast<'a>, decorators: SubRange<Decorator>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.into_extra_data();
        };
    }
    #[inline]
    pub fn set_super_class<'a>(self, ast: &mut crate::Ast<'a>, super_class: Option<Expr>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = super_class.into_extra_data();
        };
    }
    #[inline]
    pub fn set_super_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = super_type_arguments.into_extra_data();
        };
    }
    #[inline]
    pub fn set_implements<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        implements: SubRange<TsClassImplements>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = implements.into_extra_data();
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: ClassBody) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(4usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.into_extra_data();
        };
    }
}
impl ClassBody {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> SubRange<ClassElement> {
        unsafe { ast.nodes.data_unchecked(self.0).class_body.body }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: SubRange<ClassElement>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).class_body.body = body;
        }
    }
}
impl MethodDefinition {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        unsafe { ast.nodes.data_unchecked(self.0).method_definition.key }
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> MethodDefinitionKind {
        unsafe { ast.nodes.data_unchecked(self.0).method_definition.kind }
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).method_definition.computed }
    }
    #[inline]
    pub fn r#static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).method_definition.r#static }
    }
    #[inline]
    pub fn r#override<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .method_definition
                .r#override
        }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).method_definition.optional }
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .method_definition
                .r#abstract
        }
    }
    #[inline]
    pub fn accessibility<'a>(self, ast: &crate::Ast<'a>) -> Accessibility {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .method_definition
                .accessibility
        }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Decorator> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropertyKey) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).method_definition.key = key;
        }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: MethodDefinitionKind) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).method_definition.kind = kind;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .method_definition
                .computed = computed;
        }
    }
    #[inline]
    pub fn set_static<'a>(self, ast: &mut crate::Ast<'a>, r#static: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .method_definition
                .r#static = r#static;
        }
    }
    #[inline]
    pub fn set_override<'a>(self, ast: &mut crate::Ast<'a>, r#override: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .method_definition
                .r#override = r#override;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .method_definition
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a>, r#abstract: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .method_definition
                .r#abstract = r#abstract;
        }
    }
    #[inline]
    pub fn set_accessibility<'a>(self, ast: &mut crate::Ast<'a>, accessibility: Accessibility) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .method_definition
                .accessibility = accessibility;
        }
    }
    #[inline]
    pub fn set_decorators<'a>(self, ast: &mut crate::Ast<'a>, decorators: SubRange<Decorator>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.into_extra_data();
        };
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Function) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = value.into_extra_data();
        };
    }
}
impl PropertyDefinition {
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .computed
        }
    }
    #[inline]
    pub fn r#static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .r#static
        }
    }
    #[inline]
    pub fn accessor<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .accessor
        }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).property_definition.declare }
    }
    #[inline]
    pub fn r#override<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .r#override
        }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .optional
        }
    }
    #[inline]
    pub fn definite<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .definite
        }
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .readonly
        }
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .r#abstract
        }
    }
    #[inline]
    pub fn accessibility<'a>(self, ast: &crate::Ast<'a>) -> Accessibility {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .accessibility
        }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Decorator> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .property_definition
                .computed = computed;
        }
    }
    #[inline]
    pub fn set_static<'a>(self, ast: &mut crate::Ast<'a>, r#static: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .property_definition
                .r#static = r#static;
        }
    }
    #[inline]
    pub fn set_accessor<'a>(self, ast: &mut crate::Ast<'a>, accessor: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .property_definition
                .accessor = accessor;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .property_definition
                .declare = declare;
        }
    }
    #[inline]
    pub fn set_override<'a>(self, ast: &mut crate::Ast<'a>, r#override: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .property_definition
                .r#override = r#override;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .property_definition
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_definite<'a>(self, ast: &mut crate::Ast<'a>, definite: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .property_definition
                .definite = definite;
        }
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a>, readonly: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .property_definition
                .readonly = readonly;
        }
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a>, r#abstract: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .property_definition
                .r#abstract = r#abstract;
        }
    }
    #[inline]
    pub fn set_accessibility<'a>(self, ast: &mut crate::Ast<'a>, accessibility: Accessibility) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .property_definition
                .accessibility = accessibility;
        }
    }
    #[inline]
    pub fn set_decorators<'a>(self, ast: &mut crate::Ast<'a>, decorators: SubRange<Decorator>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.into_extra_data();
        };
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropertyKey) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = key.into_extra_data();
        };
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = type_annotation.into_extra_data();
        };
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<Expr>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = value.into_extra_data();
        };
    }
}
impl StaticBlock {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Stmt> {
        unsafe { ast.nodes.data_unchecked(self.0).static_block.body }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: SubRange<Stmt>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).static_block.body = body;
        }
    }
}
impl Function {
    #[inline]
    pub fn function_type<'a>(self, ast: &crate::Ast<'a>) -> FunctionType {
        unsafe { ast.nodes.data_unchecked(self.0).function.function_type }
    }
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> Option<BindingIdentifier> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).function.id };
        (raw != u32::MAX).then(|| BindingIdentifier(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).function.type_parameters };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn generator<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).function.generator }
    }
    #[inline]
    pub fn r#async<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).function.r#async }
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a>) -> Option<TsThisParameter> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> FunctionBody {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_function_type<'a>(self, ast: &mut crate::Ast<'a>, function_type: FunctionType) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).function.function_type = function_type;
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: Option<BindingIdentifier>) {
        let field_val: u32 = id.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).function.id = field_val;
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .function
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_generator<'a>(self, ast: &mut crate::Ast<'a>, generator: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).function.generator = generator;
        }
    }
    #[inline]
    pub fn set_async<'a>(self, ast: &mut crate::Ast<'a>, r#async: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).function.r#async = r#async;
        }
    }
    #[inline]
    pub fn set_this_param<'a>(self, ast: &mut crate::Ast<'a>, this_param: Option<TsThisParameter>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = this_param.into_extra_data();
        };
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = params.into_extra_data();
        };
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        return_type: Option<TsTypeAnnotation>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = return_type.into_extra_data();
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: FunctionBody) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.into_extra_data();
        };
    }
}
impl FunctionBody {
    #[inline]
    pub fn directives<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Directive> {
        unsafe { ast.nodes.data_unchecked(self.0).function_body.directives }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Stmt> {
        unsafe { ast.nodes.data_unchecked(self.0).function_body.body }
    }
    #[inline]
    pub fn set_directives<'a>(self, ast: &mut crate::Ast<'a>, directives: SubRange<Directive>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .function_body
                .directives = directives;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: SubRange<Stmt>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).function_body.body = body;
        }
    }
}
impl BlockStatement {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Stmt> {
        unsafe { ast.nodes.data_unchecked(self.0).block_statement.body }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: SubRange<Stmt>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).block_statement.body = body;
        }
    }
}
impl WhileStatement {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).while_statement.test }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        unsafe { ast.nodes.data_unchecked(self.0).while_statement.body }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).while_statement.test = test;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).while_statement.body = body;
        }
    }
}
impl DoWhileStatement {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        unsafe { ast.nodes.data_unchecked(self.0).do_while_statement.body }
    }
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).do_while_statement.test }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).do_while_statement.body = body;
        }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).do_while_statement.test = test;
        }
    }
}
impl WithStatement {
    #[inline]
    pub fn object<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).with_statement.object }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        unsafe { ast.nodes.data_unchecked(self.0).with_statement.body }
    }
    #[inline]
    pub fn set_object<'a>(self, ast: &mut crate::Ast<'a>, object: Expr) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).with_statement.object = object;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).with_statement.body = body;
        }
    }
}
impl FormalParameters {
    #[inline]
    pub fn items<'a>(self, ast: &crate::Ast<'a>) -> SubRange<FormalParameterItem> {
        unsafe { ast.nodes.data_unchecked(self.0).formal_parameters.items }
    }
    #[inline]
    pub fn rest<'a>(self, ast: &crate::Ast<'a>) -> Option<FormalParameterRest> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).formal_parameters.rest };
        (raw != u32::MAX).then(|| FormalParameterRest(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> FormalParameterKind {
        unsafe { ast.nodes.data_unchecked(self.0).formal_parameters.kind }
    }
    #[inline]
    pub fn set_items<'a>(self, ast: &mut crate::Ast<'a>, items: SubRange<FormalParameterItem>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).formal_parameters.items = items;
        }
    }
    #[inline]
    pub fn set_rest<'a>(self, ast: &mut crate::Ast<'a>, rest: Option<FormalParameterRest>) {
        let field_val: u32 = rest.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).formal_parameters.rest = field_val;
        }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: FormalParameterKind) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).formal_parameters.kind = kind;
        }
    }
}
impl FormalParameter {
    #[inline]
    pub fn pattern<'a>(self, ast: &crate::Ast<'a>) -> FormalParameterPattern {
        unsafe { ast.nodes.data_unchecked(self.0).formal_parameter.pattern }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .formal_parameter
                .type_annotation
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).formal_parameter.optional }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Decorator> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .formal_parameter
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn initializer<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .formal_parameter
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_pattern<'a>(self, ast: &mut crate::Ast<'a>, pattern: FormalParameterPattern) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .formal_parameter
                .pattern = pattern;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .formal_parameter
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .formal_parameter
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_decorators<'a>(self, ast: &mut crate::Ast<'a>, decorators: SubRange<Decorator>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .formal_parameter
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.into_extra_data();
        };
    }
    #[inline]
    pub fn set_initializer<'a>(self, ast: &mut crate::Ast<'a>, initializer: Option<Expr>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .formal_parameter
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = initializer.into_extra_data();
        };
    }
}
impl FormalParameterRest {
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Decorator> {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .formal_parameter_rest
                .decorators
        }
    }
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> FormalParameterPattern {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .formal_parameter_rest
                .argument
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .formal_parameter_rest
                .type_annotation
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_decorators<'a>(self, ast: &mut crate::Ast<'a>, decorators: SubRange<Decorator>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .formal_parameter_rest
                .decorators = decorators;
        }
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: FormalParameterPattern) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .formal_parameter_rest
                .argument = argument;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .formal_parameter_rest
                .type_annotation = field_val;
        }
    }
}
impl ArrayExpression {
    #[inline]
    pub fn elements<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Option<Argument>> {
        unsafe { ast.nodes.data_unchecked(self.0).array_expression.elements }
    }
    #[inline]
    pub fn set_elements<'a>(self, ast: &mut crate::Ast<'a>, elements: SubRange<Option<Argument>>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .array_expression
                .elements = elements;
        }
    }
}
impl ObjectExpression {
    #[inline]
    pub fn properties<'a>(self, ast: &crate::Ast<'a>) -> SubRange<ObjectPropertyKind> {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .object_expression
                .properties
        }
    }
    #[inline]
    pub fn set_properties<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        properties: SubRange<ObjectPropertyKind>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .object_expression
                .properties = properties;
        }
    }
}
impl SpreadElement {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).spread_element.argument }
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: Expr) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).spread_element.argument = argument;
        }
    }
}
impl ObjectProperty {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        unsafe { ast.nodes.data_unchecked(self.0).object_property.key }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).object_property.value }
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> PropertyKind {
        unsafe { ast.nodes.data_unchecked(self.0).object_property.kind }
    }
    #[inline]
    pub fn method<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).object_property.method }
    }
    #[inline]
    pub fn shorthand<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).object_property.shorthand }
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).object_property.computed }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropertyKey) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).object_property.key = key;
        }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Expr) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).object_property.value = value;
        }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: PropertyKind) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).object_property.kind = kind;
        }
    }
    #[inline]
    pub fn set_method<'a>(self, ast: &mut crate::Ast<'a>, method: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).object_property.method = method;
        }
    }
    #[inline]
    pub fn set_shorthand<'a>(self, ast: &mut crate::Ast<'a>, shorthand: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .object_property
                .shorthand = shorthand;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .object_property
                .computed = computed;
        }
    }
}
impl ParenthesizedExpression {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .parenthesized_expression
                .expression
        }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .parenthesized_expression
                .expression = expression;
        }
    }
}
impl ArrowFunctionExpression {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .arrow_function_expression
                .expression
        }
    }
    #[inline]
    pub fn r#async<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .arrow_function_expression
                .r#async
        }
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .arrow_function_expression
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .arrow_function_expression
                .params
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .arrow_function_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> ArrowFunctionBody {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .arrow_function_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .arrow_function_expression
                .expression = expression;
        }
    }
    #[inline]
    pub fn set_async<'a>(self, ast: &mut crate::Ast<'a>, r#async: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .arrow_function_expression
                .r#async = r#async;
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .arrow_function_expression
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .arrow_function_expression
                .params = params;
        }
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        return_type: Option<TsTypeAnnotation>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .arrow_function_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = return_type.into_extra_data();
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: ArrowFunctionBody) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .arrow_function_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.into_extra_data();
        };
    }
}
impl SequenceExpression {
    #[inline]
    pub fn expressions<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Expr> {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .sequence_expression
                .expressions
        }
    }
    #[inline]
    pub fn set_expressions<'a>(self, ast: &mut crate::Ast<'a>, expressions: SubRange<Expr>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .sequence_expression
                .expressions = expressions;
        }
    }
}
impl MemberExpression {
    #[inline]
    pub fn object<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).member_expression.object }
    }
    #[inline]
    pub fn property<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        unsafe { ast.nodes.data_unchecked(self.0).member_expression.property }
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).member_expression.computed }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).member_expression.optional }
    }
    #[inline]
    pub fn set_object<'a>(self, ast: &mut crate::Ast<'a>, object: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .member_expression
                .object = object;
        }
    }
    #[inline]
    pub fn set_property<'a>(self, ast: &mut crate::Ast<'a>, property: PropertyKey) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .member_expression
                .property = property;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .member_expression
                .computed = computed;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .member_expression
                .optional = optional;
        }
    }
}
impl CallExpression {
    #[inline]
    pub fn callee<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).call_expression.callee }
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .call_expression
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).call_expression.optional }
    }
    #[inline]
    pub fn pure<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).call_expression.pure }
    }
    #[inline]
    pub fn arguments<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Argument> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .call_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a>, callee: Expr) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).call_expression.callee = callee;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .call_expression
                .type_arguments = field_val;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .call_expression
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_pure<'a>(self, ast: &mut crate::Ast<'a>, pure: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).call_expression.pure = pure;
        }
    }
    #[inline]
    pub fn set_arguments<'a>(self, ast: &mut crate::Ast<'a>, arguments: SubRange<Argument>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .call_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = arguments.into_extra_data();
        };
    }
}
impl ChainExpression {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).chain_expression.expression }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .chain_expression
                .expression = expression;
        }
    }
}
impl TaggedTemplateExpression {
    #[inline]
    pub fn tag<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .tagged_template_expression
                .tag
        }
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .tagged_template_expression
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn quasi<'a>(self, ast: &crate::Ast<'a>) -> TemplateLiteral {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .tagged_template_expression
                .quasi
        }
    }
    #[inline]
    pub fn set_tag<'a>(self, ast: &mut crate::Ast<'a>, tag: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .tagged_template_expression
                .tag = tag;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .tagged_template_expression
                .type_arguments = field_val;
        }
    }
    #[inline]
    pub fn set_quasi<'a>(self, ast: &mut crate::Ast<'a>, quasi: TemplateLiteral) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .tagged_template_expression
                .quasi = quasi;
        }
    }
}
impl NewExpression {
    #[inline]
    pub fn callee<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).new_expression.callee }
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .new_expression
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn arguments<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Argument> {
        unsafe { ast.nodes.data_unchecked(self.0).new_expression.arguments }
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a>, callee: Expr) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).new_expression.callee = callee;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .new_expression
                .type_arguments = field_val;
        }
    }
    #[inline]
    pub fn set_arguments<'a>(self, ast: &mut crate::Ast<'a>, arguments: SubRange<Argument>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .new_expression
                .arguments = arguments;
        }
    }
}
impl AwaitExpression {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).await_expression.argument }
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .await_expression
                .argument = argument;
        }
    }
}
impl YieldExpression {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).yield_expression.argument };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn delegate<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).yield_expression.delegate }
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: Option<Expr>) {
        let field_val: u32 = argument.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .yield_expression
                .argument = field_val;
        }
    }
    #[inline]
    pub fn set_delegate<'a>(self, ast: &mut crate::Ast<'a>, delegate: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .yield_expression
                .delegate = delegate;
        }
    }
}
impl MetaProperty {
    #[inline]
    pub fn meta<'a>(self, ast: &crate::Ast<'a>) -> IdentifierName {
        unsafe { ast.nodes.data_unchecked(self.0).meta_property.meta }
    }
    #[inline]
    pub fn property<'a>(self, ast: &crate::Ast<'a>) -> IdentifierName {
        unsafe { ast.nodes.data_unchecked(self.0).meta_property.property }
    }
    #[inline]
    pub fn set_meta<'a>(self, ast: &mut crate::Ast<'a>, meta: IdentifierName) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).meta_property.meta = meta;
        }
    }
    #[inline]
    pub fn set_property<'a>(self, ast: &mut crate::Ast<'a>, property: IdentifierName) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).meta_property.property = property;
        }
    }
}
impl ImportExpression {
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).import_expression.source }
    }
    #[inline]
    pub fn options<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).import_expression.options };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn sync<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).import_expression.sync }
    }
    #[inline]
    pub fn phase<'a>(self, ast: &crate::Ast<'a>) -> Option<ImportPhase> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .import_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a>, source: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .import_expression
                .source = source;
        }
    }
    #[inline]
    pub fn set_options<'a>(self, ast: &mut crate::Ast<'a>, options: Option<Expr>) {
        let field_val: u32 = options.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .import_expression
                .options = field_val;
        }
    }
    #[inline]
    pub fn set_sync<'a>(self, ast: &mut crate::Ast<'a>, sync: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).import_expression.sync = sync;
        }
    }
    #[inline]
    pub fn set_phase<'a>(self, ast: &mut crate::Ast<'a>, phase: Option<ImportPhase>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .import_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = phase.into_extra_data();
        };
    }
}
impl ImportDeclaration {
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a>) -> StringLiteral {
        unsafe { ast.nodes.data_unchecked(self.0).import_declaration.source }
    }
    #[inline]
    pub fn import_kind<'a>(self, ast: &crate::Ast<'a>) -> ImportOrExportKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .import_declaration
                .import_kind
        }
    }
    #[inline]
    pub fn specifiers<'a>(self, ast: &crate::Ast<'a>) -> SubRange<ImportDeclarationSpecifier> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .import_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a>) -> SubRange<ImportAttribute> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .import_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn phase<'a>(self, ast: &crate::Ast<'a>) -> Option<ImportPhase> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .import_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a>, source: StringLiteral) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .import_declaration
                .source = source;
        }
    }
    #[inline]
    pub fn set_import_kind<'a>(self, ast: &mut crate::Ast<'a>, import_kind: ImportOrExportKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .import_declaration
                .import_kind = import_kind;
        }
    }
    #[inline]
    pub fn set_specifiers<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        specifiers: SubRange<ImportDeclarationSpecifier>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .import_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = specifiers.into_extra_data();
        };
    }
    #[inline]
    pub fn set_attributes<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        attributes: SubRange<ImportAttribute>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .import_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = attributes.into_extra_data();
        };
    }
    #[inline]
    pub fn set_phase<'a>(self, ast: &mut crate::Ast<'a>, phase: Option<ImportPhase>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .import_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = phase.into_extra_data();
        };
    }
}
impl ImportSpecifier {
    #[inline]
    pub fn imported<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        unsafe { ast.nodes.data_unchecked(self.0).import_specifier.imported }
    }
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        unsafe { ast.nodes.data_unchecked(self.0).import_specifier.local }
    }
    #[inline]
    pub fn import_kind<'a>(self, ast: &crate::Ast<'a>) -> ImportOrExportKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .import_specifier
                .import_kind
        }
    }
    #[inline]
    pub fn set_imported<'a>(self, ast: &mut crate::Ast<'a>, imported: ModuleExportName) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .import_specifier
                .imported = imported;
        }
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: BindingIdentifier) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).import_specifier.local = local;
        }
    }
    #[inline]
    pub fn set_import_kind<'a>(self, ast: &mut crate::Ast<'a>, import_kind: ImportOrExportKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .import_specifier
                .import_kind = import_kind;
        }
    }
}
impl ImportDefaultSpecifier {
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .import_default_specifier
                .local
        }
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: BindingIdentifier) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .import_default_specifier
                .local = local;
        }
    }
}
impl ImportNamespaceSpecifier {
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .import_namespace_specifier
                .local
        }
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: BindingIdentifier) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .import_namespace_specifier
                .local = local;
        }
    }
}
impl ImportAttribute {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        unsafe { ast.nodes.data_unchecked(self.0).import_attribute.key }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> StringLiteral {
        unsafe { ast.nodes.data_unchecked(self.0).import_attribute.value }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: ModuleExportName) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).import_attribute.key = key;
        }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: StringLiteral) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).import_attribute.value = value;
        }
    }
}
impl ExportNamedDeclaration {
    #[inline]
    pub fn declaration<'a>(self, ast: &crate::Ast<'a>) -> Option<Decl> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .export_named_declaration
                .declaration
        };
        (raw != u32::MAX).then(|| Decl(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a>) -> Option<StringLiteral> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .export_named_declaration
                .source
        };
        (raw != u32::MAX).then(|| StringLiteral(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn export_kind<'a>(self, ast: &crate::Ast<'a>) -> ImportOrExportKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .export_named_declaration
                .export_kind
        }
    }
    #[inline]
    pub fn specifiers<'a>(self, ast: &crate::Ast<'a>) -> SubRange<ExportSpecifier> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .export_named_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a>) -> SubRange<ImportAttribute> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .export_named_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_declaration<'a>(self, ast: &mut crate::Ast<'a>, declaration: Option<Decl>) {
        let field_val: u32 = declaration.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .export_named_declaration
                .declaration = field_val;
        }
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a>, source: Option<StringLiteral>) {
        let field_val: u32 = source.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .export_named_declaration
                .source = field_val;
        }
    }
    #[inline]
    pub fn set_export_kind<'a>(self, ast: &mut crate::Ast<'a>, export_kind: ImportOrExportKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .export_named_declaration
                .export_kind = export_kind;
        }
    }
    #[inline]
    pub fn set_specifiers<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        specifiers: SubRange<ExportSpecifier>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .export_named_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = specifiers.into_extra_data();
        };
    }
    #[inline]
    pub fn set_attributes<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        attributes: SubRange<ImportAttribute>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .export_named_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = attributes.into_extra_data();
        };
    }
}
impl ExportDefaultDeclaration {
    #[inline]
    pub fn declaration<'a>(self, ast: &crate::Ast<'a>) -> ExportDefaultDeclarationKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .export_default_declaration
                .declaration
        }
    }
    #[inline]
    pub fn set_declaration<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        declaration: ExportDefaultDeclarationKind,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .export_default_declaration
                .declaration = declaration;
        }
    }
}
impl ExportAllDeclaration {
    #[inline]
    pub fn exported<'a>(self, ast: &crate::Ast<'a>) -> Option<ModuleExportName> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .export_all_declaration
                .exported
        };
        (raw != u32::MAX).then(|| ModuleExportName(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a>) -> StringLiteral {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .export_all_declaration
                .source
        }
    }
    #[inline]
    pub fn export_kind<'a>(self, ast: &crate::Ast<'a>) -> ImportOrExportKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .export_all_declaration
                .export_kind
        }
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a>) -> SubRange<ImportAttribute> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .export_all_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_exported<'a>(self, ast: &mut crate::Ast<'a>, exported: Option<ModuleExportName>) {
        let field_val: u32 = exported.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .export_all_declaration
                .exported = field_val;
        }
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a>, source: StringLiteral) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .export_all_declaration
                .source = source;
        }
    }
    #[inline]
    pub fn set_export_kind<'a>(self, ast: &mut crate::Ast<'a>, export_kind: ImportOrExportKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .export_all_declaration
                .export_kind = export_kind;
        }
    }
    #[inline]
    pub fn set_attributes<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        attributes: SubRange<ImportAttribute>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .export_all_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = attributes.into_extra_data();
        };
    }
}
impl ExportSpecifier {
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        unsafe { ast.nodes.data_unchecked(self.0).export_specifier.local }
    }
    #[inline]
    pub fn exported<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        unsafe { ast.nodes.data_unchecked(self.0).export_specifier.exported }
    }
    #[inline]
    pub fn export_kind<'a>(self, ast: &crate::Ast<'a>) -> ImportOrExportKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .export_specifier
                .export_kind
        }
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: ModuleExportName) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).export_specifier.local = local;
        }
    }
    #[inline]
    pub fn set_exported<'a>(self, ast: &mut crate::Ast<'a>, exported: ModuleExportName) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .export_specifier
                .exported = exported;
        }
    }
    #[inline]
    pub fn set_export_kind<'a>(self, ast: &mut crate::Ast<'a>, export_kind: ImportOrExportKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .export_specifier
                .export_kind = export_kind;
        }
    }
}
impl Hashbang {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).hashbang.value }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: AstUtf8) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).hashbang.value = value;
        }
    }
}
impl TsType {
    #[inline]
    pub fn is_ts_any_keyword(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsAnyKeyword
    }
    #[inline]
    pub fn is_ts_unknown_keyword(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsUnknownKeyword
    }
    #[inline]
    pub fn is_ts_never_keyword(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsNeverKeyword
    }
    #[inline]
    pub fn is_ts_void_keyword(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsVoidKeyword
    }
    #[inline]
    pub fn is_ts_null_keyword(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsNullKeyword
    }
    #[inline]
    pub fn is_ts_undefined_keyword(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsUndefinedKeyword
    }
    #[inline]
    pub fn is_ts_string_keyword(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsStringKeyword
    }
    #[inline]
    pub fn is_ts_number_keyword(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsNumberKeyword
    }
    #[inline]
    pub fn is_ts_big_int_keyword(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsBigIntKeyword
    }
    #[inline]
    pub fn is_ts_boolean_keyword(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsBooleanKeyword
    }
    #[inline]
    pub fn is_ts_symbol_keyword(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsSymbolKeyword
    }
    #[inline]
    pub fn is_ts_object_keyword(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsObjectKeyword
    }
    #[inline]
    pub fn is_ts_intrinsic_keyword(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsIntrinsicKeyword
    }
    #[inline]
    pub fn is_ts_this_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsThisType
    }
    #[inline]
    pub fn is_ts_type_reference(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypeReference
    }
    #[inline]
    pub fn is_ts_type_query(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypeQuery
    }
    #[inline]
    pub fn is_ts_import_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsImportType
    }
    #[inline]
    pub fn is_ts_literal_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsLiteralType
    }
    #[inline]
    pub fn is_ts_template_literal_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTemplateLiteralType
    }
    #[inline]
    pub fn is_ts_array_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsArrayType
    }
    #[inline]
    pub fn is_ts_indexed_access_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsIndexedAccessType
    }
    #[inline]
    pub fn is_ts_tuple_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTupleType
    }
    #[inline]
    pub fn is_ts_optional_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsOptionalType
    }
    #[inline]
    pub fn is_ts_rest_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsRestType
    }
    #[inline]
    pub fn is_ts_js_doc_nullable_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsJsDocNullableType
    }
    #[inline]
    pub fn is_ts_js_doc_non_nullable_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsJsDocNonNullableType
    }
    #[inline]
    pub fn is_ts_js_doc_unknown_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsJsDocUnknownType
    }
    #[inline]
    pub fn is_ts_union_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsUnionType
    }
    #[inline]
    pub fn is_ts_intersection_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsIntersectionType
    }
    #[inline]
    pub fn is_ts_conditional_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsConditionalType
    }
    #[inline]
    pub fn is_ts_infer_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsInferType
    }
    #[inline]
    pub fn is_ts_type_operator(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypeOperator
    }
    #[inline]
    pub fn is_ts_parenthesized_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsParenthesizedType
    }
    #[inline]
    pub fn is_ts_function_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsFunctionType
    }
    #[inline]
    pub fn is_ts_constructor_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsConstructorType
    }
    #[inline]
    pub fn is_ts_type_predicate(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypePredicate
    }
    #[inline]
    pub fn is_ts_type_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypeLiteral
    }
    #[inline]
    pub fn is_ts_mapped_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsMappedType
    }
    #[inline]
    pub fn as_ts_any_keyword(self, ast: &Ast<'_>) -> Option<TsAnyKeyword> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsAnyKeyword) {
            Some(TsAnyKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_unknown_keyword(self, ast: &Ast<'_>) -> Option<TsUnknownKeyword> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsUnknownKeyword) {
            Some(TsUnknownKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_never_keyword(self, ast: &Ast<'_>) -> Option<TsNeverKeyword> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsNeverKeyword) {
            Some(TsNeverKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_void_keyword(self, ast: &Ast<'_>) -> Option<TsVoidKeyword> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsVoidKeyword) {
            Some(TsVoidKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_null_keyword(self, ast: &Ast<'_>) -> Option<TsNullKeyword> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsNullKeyword) {
            Some(TsNullKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_undefined_keyword(self, ast: &Ast<'_>) -> Option<TsUndefinedKeyword> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsUndefinedKeyword) {
            Some(TsUndefinedKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_string_keyword(self, ast: &Ast<'_>) -> Option<TsStringKeyword> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsStringKeyword) {
            Some(TsStringKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_number_keyword(self, ast: &Ast<'_>) -> Option<TsNumberKeyword> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsNumberKeyword) {
            Some(TsNumberKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_big_int_keyword(self, ast: &Ast<'_>) -> Option<TsBigIntKeyword> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsBigIntKeyword) {
            Some(TsBigIntKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_boolean_keyword(self, ast: &Ast<'_>) -> Option<TsBooleanKeyword> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsBooleanKeyword) {
            Some(TsBooleanKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_symbol_keyword(self, ast: &Ast<'_>) -> Option<TsSymbolKeyword> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsSymbolKeyword) {
            Some(TsSymbolKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_object_keyword(self, ast: &Ast<'_>) -> Option<TsObjectKeyword> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsObjectKeyword) {
            Some(TsObjectKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_intrinsic_keyword(self, ast: &Ast<'_>) -> Option<TsIntrinsicKeyword> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsIntrinsicKeyword) {
            Some(TsIntrinsicKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_this_type(self, ast: &Ast<'_>) -> Option<TsThisType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsThisType) {
            Some(TsThisType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_reference(self, ast: &Ast<'_>) -> Option<TsTypeReference> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsTypeReference) {
            Some(TsTypeReference(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_query(self, ast: &Ast<'_>) -> Option<TsTypeQuery> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsTypeQuery) {
            Some(TsTypeQuery(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_import_type(self, ast: &Ast<'_>) -> Option<TsImportType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsImportType) {
            Some(TsImportType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_literal_type(self, ast: &Ast<'_>) -> Option<TsLiteralType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsLiteralType) {
            Some(TsLiteralType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_template_literal_type(self, ast: &Ast<'_>) -> Option<TsTemplateLiteralType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsTemplateLiteralType) {
            Some(TsTemplateLiteralType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_array_type(self, ast: &Ast<'_>) -> Option<TsArrayType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsArrayType) {
            Some(TsArrayType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_indexed_access_type(self, ast: &Ast<'_>) -> Option<TsIndexedAccessType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsIndexedAccessType) {
            Some(TsIndexedAccessType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_tuple_type(self, ast: &Ast<'_>) -> Option<TsTupleType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsTupleType) {
            Some(TsTupleType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_optional_type(self, ast: &Ast<'_>) -> Option<TsOptionalType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsOptionalType) {
            Some(TsOptionalType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_rest_type(self, ast: &Ast<'_>) -> Option<TsRestType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsRestType) {
            Some(TsRestType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_js_doc_nullable_type(self, ast: &Ast<'_>) -> Option<TsJsDocNullableType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsJsDocNullableType) {
            Some(TsJsDocNullableType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_js_doc_non_nullable_type(self, ast: &Ast<'_>) -> Option<TsJsDocNonNullableType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsJsDocNonNullableType) {
            Some(TsJsDocNonNullableType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_js_doc_unknown_type(self, ast: &Ast<'_>) -> Option<TsJsDocUnknownType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsJsDocUnknownType) {
            Some(TsJsDocUnknownType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_union_type(self, ast: &Ast<'_>) -> Option<TsUnionType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsUnionType) {
            Some(TsUnionType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_intersection_type(self, ast: &Ast<'_>) -> Option<TsIntersectionType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsIntersectionType) {
            Some(TsIntersectionType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_conditional_type(self, ast: &Ast<'_>) -> Option<TsConditionalType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsConditionalType) {
            Some(TsConditionalType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_infer_type(self, ast: &Ast<'_>) -> Option<TsInferType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsInferType) {
            Some(TsInferType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_operator(self, ast: &Ast<'_>) -> Option<TsTypeOperator> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsTypeOperator) {
            Some(TsTypeOperator(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_parenthesized_type(self, ast: &Ast<'_>) -> Option<TsParenthesizedType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsParenthesizedType) {
            Some(TsParenthesizedType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_function_type(self, ast: &Ast<'_>) -> Option<TsFunctionType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsFunctionType) {
            Some(TsFunctionType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_constructor_type(self, ast: &Ast<'_>) -> Option<TsConstructorType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsConstructorType) {
            Some(TsConstructorType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_predicate(self, ast: &Ast<'_>) -> Option<TsTypePredicate> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsTypePredicate) {
            Some(TsTypePredicate(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_literal(self, ast: &Ast<'_>) -> Option<TsTypeLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsTypeLiteral) {
            Some(TsTypeLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_mapped_type(self, ast: &Ast<'_>) -> Option<TsMappedType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsMappedType) {
            Some(TsMappedType(self.0))
        } else {
            None
        }
    }
}
impl TsTypeName {
    #[inline]
    pub fn is_identifier_reference(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierReference
    }
    #[inline]
    pub fn is_ts_qualified_name(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsQualifiedName
    }
    #[inline]
    pub fn as_identifier_reference(self, ast: &Ast<'_>) -> Option<IdentifierReference> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::IdentifierReference) {
            Some(IdentifierReference(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_qualified_name(self, ast: &Ast<'_>) -> Option<TsQualifiedName> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsQualifiedName) {
            Some(TsQualifiedName(self.0))
        } else {
            None
        }
    }
}
impl TsQualifiedNameLeft {
    #[inline]
    pub fn is_identifier_reference(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierReference
    }
    #[inline]
    pub fn is_identifier_name(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierName
    }
    #[inline]
    pub fn is_binding_identifier(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::BindingIdentifier
    }
    #[inline]
    pub fn is_this_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ThisExpression
    }
    #[inline]
    pub fn is_ts_qualified_name(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsQualifiedName
    }
    #[inline]
    pub fn as_identifier_reference(self, ast: &Ast<'_>) -> Option<IdentifierReference> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::IdentifierReference) {
            Some(IdentifierReference(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_identifier_name(self, ast: &Ast<'_>) -> Option<IdentifierName> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::IdentifierName) {
            Some(IdentifierName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_binding_identifier(self, ast: &Ast<'_>) -> Option<BindingIdentifier> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::BindingIdentifier) {
            Some(BindingIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_this_expression(self, ast: &Ast<'_>) -> Option<ThisExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ThisExpression) {
            Some(ThisExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_qualified_name(self, ast: &Ast<'_>) -> Option<TsQualifiedName> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsQualifiedName) {
            Some(TsQualifiedName(self.0))
        } else {
            None
        }
    }
}
impl TsImportTypeQualifier {
    #[inline]
    pub fn is_identifier_name(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierName
    }
    #[inline]
    pub fn is_ts_qualified_name(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsQualifiedName
    }
    #[inline]
    pub fn as_identifier_name(self, ast: &Ast<'_>) -> Option<IdentifierName> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::IdentifierName) {
            Some(IdentifierName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_qualified_name(self, ast: &Ast<'_>) -> Option<TsQualifiedName> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsQualifiedName) {
            Some(TsQualifiedName(self.0))
        } else {
            None
        }
    }
}
impl TsTypeQueryExprName {
    #[inline]
    pub fn is_ts_type_name(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference | NodeKind::TsQualifiedName
        )
    }
    #[inline]
    pub fn is_this_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::ThisExpression
    }
    #[inline]
    pub fn is_ts_import_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsImportType
    }
    #[inline]
    pub fn as_ts_type_name(self, ast: &Ast<'_>) -> Option<TsTypeName> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(
            kind,
            NodeKind::IdentifierReference | NodeKind::TsQualifiedName
        ) {
            Some(TsTypeName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_this_expression(self, ast: &Ast<'_>) -> Option<ThisExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::ThisExpression) {
            Some(ThisExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_import_type(self, ast: &Ast<'_>) -> Option<TsImportType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsImportType) {
            Some(TsImportType(self.0))
        } else {
            None
        }
    }
}
impl TsLiteral {
    #[inline]
    pub fn is_string_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::StringLiteral
    }
    #[inline]
    pub fn is_numeric_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::NumericLiteral
    }
    #[inline]
    pub fn is_big_int_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::BigIntLiteral
    }
    #[inline]
    pub fn is_boolean_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::BooleanLiteral
    }
    #[inline]
    pub fn is_template_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TemplateLiteral
    }
    #[inline]
    pub fn is_unary_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::UnaryExpression
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_>) -> Option<StringLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::StringLiteral) {
            Some(StringLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_numeric_literal(self, ast: &Ast<'_>) -> Option<NumericLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::NumericLiteral) {
            Some(NumericLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_big_int_literal(self, ast: &Ast<'_>) -> Option<BigIntLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::BigIntLiteral) {
            Some(BigIntLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_boolean_literal(self, ast: &Ast<'_>) -> Option<BooleanLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::BooleanLiteral) {
            Some(BooleanLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_template_literal(self, ast: &Ast<'_>) -> Option<TemplateLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TemplateLiteral) {
            Some(TemplateLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_unary_expression(self, ast: &Ast<'_>) -> Option<UnaryExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::UnaryExpression) {
            Some(UnaryExpression(self.0))
        } else {
            None
        }
    }
}
impl TsTupleElement {
    #[inline]
    pub fn is_ts_type(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::TsAnyKeyword
                | NodeKind::TsUnknownKeyword
                | NodeKind::TsNeverKeyword
                | NodeKind::TsVoidKeyword
                | NodeKind::TsNullKeyword
                | NodeKind::TsUndefinedKeyword
                | NodeKind::TsStringKeyword
                | NodeKind::TsNumberKeyword
                | NodeKind::TsBigIntKeyword
                | NodeKind::TsBooleanKeyword
                | NodeKind::TsSymbolKeyword
                | NodeKind::TsObjectKeyword
                | NodeKind::TsIntrinsicKeyword
                | NodeKind::TsThisType
                | NodeKind::TsTypeReference
                | NodeKind::TsTypeQuery
                | NodeKind::TsImportType
                | NodeKind::TsLiteralType
                | NodeKind::TsTemplateLiteralType
                | NodeKind::TsArrayType
                | NodeKind::TsIndexedAccessType
                | NodeKind::TsTupleType
                | NodeKind::TsOptionalType
                | NodeKind::TsRestType
                | NodeKind::TsJsDocNullableType
                | NodeKind::TsJsDocNonNullableType
                | NodeKind::TsJsDocUnknownType
                | NodeKind::TsUnionType
                | NodeKind::TsIntersectionType
                | NodeKind::TsConditionalType
                | NodeKind::TsInferType
                | NodeKind::TsTypeOperator
                | NodeKind::TsParenthesizedType
                | NodeKind::TsFunctionType
                | NodeKind::TsConstructorType
                | NodeKind::TsTypePredicate
                | NodeKind::TsTypeLiteral
                | NodeKind::TsMappedType
        )
    }
    #[inline]
    pub fn is_ts_named_tuple_member(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsNamedTupleMember
    }
    #[inline]
    pub fn as_ts_type(self, ast: &Ast<'_>) -> Option<TsType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(
            kind,
            NodeKind::TsAnyKeyword
                | NodeKind::TsUnknownKeyword
                | NodeKind::TsNeverKeyword
                | NodeKind::TsVoidKeyword
                | NodeKind::TsNullKeyword
                | NodeKind::TsUndefinedKeyword
                | NodeKind::TsStringKeyword
                | NodeKind::TsNumberKeyword
                | NodeKind::TsBigIntKeyword
                | NodeKind::TsBooleanKeyword
                | NodeKind::TsSymbolKeyword
                | NodeKind::TsObjectKeyword
                | NodeKind::TsIntrinsicKeyword
                | NodeKind::TsThisType
                | NodeKind::TsTypeReference
                | NodeKind::TsTypeQuery
                | NodeKind::TsImportType
                | NodeKind::TsLiteralType
                | NodeKind::TsTemplateLiteralType
                | NodeKind::TsArrayType
                | NodeKind::TsIndexedAccessType
                | NodeKind::TsTupleType
                | NodeKind::TsOptionalType
                | NodeKind::TsRestType
                | NodeKind::TsJsDocNullableType
                | NodeKind::TsJsDocNonNullableType
                | NodeKind::TsJsDocUnknownType
                | NodeKind::TsUnionType
                | NodeKind::TsIntersectionType
                | NodeKind::TsConditionalType
                | NodeKind::TsInferType
                | NodeKind::TsTypeOperator
                | NodeKind::TsParenthesizedType
                | NodeKind::TsFunctionType
                | NodeKind::TsConstructorType
                | NodeKind::TsTypePredicate
                | NodeKind::TsTypeLiteral
                | NodeKind::TsMappedType
        ) {
            Some(TsType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_named_tuple_member(self, ast: &Ast<'_>) -> Option<TsNamedTupleMember> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsNamedTupleMember) {
            Some(TsNamedTupleMember(self.0))
        } else {
            None
        }
    }
}
impl TsTypeElement {
    #[inline]
    pub fn is_ts_property_signature(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsPropertySignature
    }
    #[inline]
    pub fn is_ts_method_signature(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsMethodSignature
    }
    #[inline]
    pub fn is_ts_call_signature_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsCallSignatureDeclaration
    }
    #[inline]
    pub fn is_ts_construct_signature_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsConstructSignatureDeclaration
    }
    #[inline]
    pub fn is_ts_index_signature(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsIndexSignature
    }
    #[inline]
    pub fn as_ts_property_signature(self, ast: &Ast<'_>) -> Option<TsPropertySignature> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsPropertySignature) {
            Some(TsPropertySignature(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_method_signature(self, ast: &Ast<'_>) -> Option<TsMethodSignature> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsMethodSignature) {
            Some(TsMethodSignature(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_call_signature_declaration(
        self,
        ast: &Ast<'_>,
    ) -> Option<TsCallSignatureDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsCallSignatureDeclaration) {
            Some(TsCallSignatureDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_construct_signature_declaration(
        self,
        ast: &Ast<'_>,
    ) -> Option<TsConstructSignatureDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsConstructSignatureDeclaration) {
            Some(TsConstructSignatureDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_index_signature(self, ast: &Ast<'_>) -> Option<TsIndexSignature> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsIndexSignature) {
            Some(TsIndexSignature(self.0))
        } else {
            None
        }
    }
}
impl TsTypePredicateName {
    #[inline]
    pub fn is_identifier_name(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierName
    }
    #[inline]
    pub fn is_ts_this_type(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsThisType
    }
    #[inline]
    pub fn as_identifier_name(self, ast: &Ast<'_>) -> Option<IdentifierName> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::IdentifierName) {
            Some(IdentifierName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_this_type(self, ast: &Ast<'_>) -> Option<TsThisType> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsThisType) {
            Some(TsThisType(self.0))
        } else {
            None
        }
    }
}
impl TsEnumMemberName {
    #[inline]
    pub fn is_identifier_name(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierName
    }
    #[inline]
    pub fn is_string_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::StringLiteral
    }
    #[inline]
    pub fn is_template_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TemplateLiteral
    }
    #[inline]
    pub fn as_identifier_name(self, ast: &Ast<'_>) -> Option<IdentifierName> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::IdentifierName) {
            Some(IdentifierName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_>) -> Option<StringLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::StringLiteral) {
            Some(StringLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_template_literal(self, ast: &Ast<'_>) -> Option<TemplateLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TemplateLiteral) {
            Some(TemplateLiteral(self.0))
        } else {
            None
        }
    }
}
impl TsModuleName {
    #[inline]
    pub fn is_binding_identifier(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::BindingIdentifier
    }
    #[inline]
    pub fn is_string_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::StringLiteral
    }
    #[inline]
    pub fn is_ts_qualified_name(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsQualifiedName
    }
    #[inline]
    pub fn as_binding_identifier(self, ast: &Ast<'_>) -> Option<BindingIdentifier> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::BindingIdentifier) {
            Some(BindingIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_>) -> Option<StringLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::StringLiteral) {
            Some(StringLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_qualified_name(self, ast: &Ast<'_>) -> Option<TsQualifiedName> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsQualifiedName) {
            Some(TsQualifiedName(self.0))
        } else {
            None
        }
    }
}
impl TsModuleBody {
    #[inline]
    pub fn is_ts_module_block(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsModuleBlock
    }
    #[inline]
    pub fn is_ts_module_declaration(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsModuleDeclaration
    }
    #[inline]
    pub fn as_ts_module_block(self, ast: &Ast<'_>) -> Option<TsModuleBlock> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsModuleBlock) {
            Some(TsModuleBlock(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_module_declaration(self, ast: &Ast<'_>) -> Option<TsModuleDeclaration> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsModuleDeclaration) {
            Some(TsModuleDeclaration(self.0))
        } else {
            None
        }
    }
}
impl TsModuleReference {
    #[inline]
    pub fn is_ts_type_name(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference | NodeKind::TsQualifiedName
        )
    }
    #[inline]
    pub fn is_ts_external_module_reference(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsExternalModuleReference
    }
    #[inline]
    pub fn as_ts_type_name(self, ast: &Ast<'_>) -> Option<TsTypeName> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(
            kind,
            NodeKind::IdentifierReference | NodeKind::TsQualifiedName
        ) {
            Some(TsTypeName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_external_module_reference(
        self,
        ast: &Ast<'_>,
    ) -> Option<TsExternalModuleReference> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::TsExternalModuleReference) {
            Some(TsExternalModuleReference(self.0))
        } else {
            None
        }
    }
}
impl TsFunction {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> Option<BindingIdentifier> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).ts_function.id };
        (raw != u32::MAX).then(|| BindingIdentifier(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).ts_function.type_parameters };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn generator<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).ts_function.generator }
    }
    #[inline]
    pub fn r#async<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).ts_function.r#async }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).ts_function.declare }
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a>) -> Option<TsThisParameter> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: Option<BindingIdentifier>) {
        let field_val: u32 = id.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_function.id = field_val;
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_function
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_generator<'a>(self, ast: &mut crate::Ast<'a>, generator: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_function.generator = generator;
        }
    }
    #[inline]
    pub fn set_async<'a>(self, ast: &mut crate::Ast<'a>, r#async: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_function.r#async = r#async;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_function.declare = declare;
        }
    }
    #[inline]
    pub fn set_this_param<'a>(self, ast: &mut crate::Ast<'a>, this_param: Option<TsThisParameter>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = this_param.into_extra_data();
        };
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = params.into_extra_data();
        };
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        return_type: Option<TsTypeAnnotation>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = return_type.into_extra_data();
        };
    }
}
impl TsMethodDefinition {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        unsafe { ast.nodes.data_unchecked(self.0).ts_method_definition.key }
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> MethodDefinitionKind {
        unsafe { ast.nodes.data_unchecked(self.0).ts_method_definition.kind }
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_definition
                .computed
        }
    }
    #[inline]
    pub fn r#static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_definition
                .r#static
        }
    }
    #[inline]
    pub fn r#override<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_definition
                .r#override
        }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_definition
                .optional
        }
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_definition
                .r#abstract
        }
    }
    #[inline]
    pub fn accessibility<'a>(self, ast: &crate::Ast<'a>) -> Accessibility {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_definition
                .accessibility
        }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Decorator> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> TsFunction {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropertyKey) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_method_definition
                .key = key;
        }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: MethodDefinitionKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_method_definition
                .kind = kind;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_method_definition
                .computed = computed;
        }
    }
    #[inline]
    pub fn set_static<'a>(self, ast: &mut crate::Ast<'a>, r#static: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_method_definition
                .r#static = r#static;
        }
    }
    #[inline]
    pub fn set_override<'a>(self, ast: &mut crate::Ast<'a>, r#override: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_method_definition
                .r#override = r#override;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_method_definition
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a>, r#abstract: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_method_definition
                .r#abstract = r#abstract;
        }
    }
    #[inline]
    pub fn set_accessibility<'a>(self, ast: &mut crate::Ast<'a>, accessibility: Accessibility) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_method_definition
                .accessibility = accessibility;
        }
    }
    #[inline]
    pub fn set_decorators<'a>(self, ast: &mut crate::Ast<'a>, decorators: SubRange<Decorator>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.into_extra_data();
        };
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: TsFunction) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = value.into_extra_data();
        };
    }
}
impl TsTypeAnnotation {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_annotation
                .type_annotation
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_annotation
                .type_annotation = type_annotation;
        }
    }
}
impl TsAnyKeyword {}
impl TsUnknownKeyword {}
impl TsNeverKeyword {}
impl TsVoidKeyword {}
impl TsNullKeyword {}
impl TsUndefinedKeyword {}
impl TsStringKeyword {}
impl TsNumberKeyword {}
impl TsBigIntKeyword {}
impl TsBooleanKeyword {}
impl TsSymbolKeyword {}
impl TsObjectKeyword {}
impl TsIntrinsicKeyword {}
impl TsThisType {}
impl TsTypeReference {
    #[inline]
    pub fn type_name<'a>(self, ast: &crate::Ast<'a>) -> TsTypeName {
        unsafe { ast.nodes.data_unchecked(self.0).ts_type_reference.type_name }
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_reference
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_type_name<'a>(self, ast: &mut crate::Ast<'a>, type_name: TsTypeName) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_reference
                .type_name = type_name;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_reference
                .type_arguments = field_val;
        }
    }
}
impl TsQualifiedName {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> TsQualifiedNameLeft {
        unsafe { ast.nodes.data_unchecked(self.0).ts_qualified_name.left }
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> IdentifierName {
        unsafe { ast.nodes.data_unchecked(self.0).ts_qualified_name.right }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: TsQualifiedNameLeft) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_qualified_name.left = left;
        }
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: IdentifierName) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_qualified_name.right = right;
        }
    }
}
impl TsTypeQuery {
    #[inline]
    pub fn expr_name<'a>(self, ast: &crate::Ast<'a>) -> TsTypeQueryExprName {
        unsafe { ast.nodes.data_unchecked(self.0).ts_type_query.expr_name }
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_query
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_expr_name<'a>(self, ast: &mut crate::Ast<'a>, expr_name: TsTypeQueryExprName) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_type_query.expr_name = expr_name;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_query
                .type_arguments = field_val;
        }
    }
}
impl TsImportType {
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a>) -> StringLiteral {
        unsafe { ast.nodes.data_unchecked(self.0).ts_import_type.source }
    }
    #[inline]
    pub fn options<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).ts_import_type.options };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn qualifier<'a>(self, ast: &crate::Ast<'a>) -> Option<TsImportTypeQualifier> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).ts_import_type.qualifier };
        (raw != u32::MAX).then(|| TsImportTypeQualifier(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_import_type
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a>, source: StringLiteral) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_import_type.source = source;
        }
    }
    #[inline]
    pub fn set_options<'a>(self, ast: &mut crate::Ast<'a>, options: Option<Expr>) {
        let field_val: u32 = options.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_import_type.options = field_val;
        }
    }
    #[inline]
    pub fn set_qualifier<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        qualifier: Option<TsImportTypeQualifier>,
    ) {
        let field_val: u32 = qualifier.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_import_type
                .qualifier = field_val;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_import_type
                .type_arguments = field_val;
        }
    }
}
impl TsTypeParameter {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        unsafe { ast.nodes.data_unchecked(self.0).ts_type_parameter.name }
    }
    #[inline]
    pub fn constraint<'a>(self, ast: &crate::Ast<'a>) -> Option<TsType> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_parameter
                .constraint
        };
        (raw != u32::MAX).then(|| TsType(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn default<'a>(self, ast: &crate::Ast<'a>) -> Option<TsType> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).ts_type_parameter.default };
        (raw != u32::MAX).then(|| TsType(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn r#in<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).ts_type_parameter.r#in }
    }
    #[inline]
    pub fn out<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).ts_type_parameter.out }
    }
    #[inline]
    pub fn r#const<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).ts_type_parameter.r#const }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: BindingIdentifier) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_type_parameter.name = name;
        }
    }
    #[inline]
    pub fn set_constraint<'a>(self, ast: &mut crate::Ast<'a>, constraint: Option<TsType>) {
        let field_val: u32 = constraint.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_parameter
                .constraint = field_val;
        }
    }
    #[inline]
    pub fn set_default<'a>(self, ast: &mut crate::Ast<'a>, default: Option<TsType>) {
        let field_val: u32 = default.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_parameter
                .default = field_val;
        }
    }
    #[inline]
    pub fn set_in<'a>(self, ast: &mut crate::Ast<'a>, r#in: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_type_parameter.r#in = r#in;
        }
    }
    #[inline]
    pub fn set_out<'a>(self, ast: &mut crate::Ast<'a>, out: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_type_parameter.out = out;
        }
    }
    #[inline]
    pub fn set_const<'a>(self, ast: &mut crate::Ast<'a>, r#const: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_parameter
                .r#const = r#const;
        }
    }
}
impl TsTypeParameterDeclaration {
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsTypeParameter> {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_parameter_declaration
                .params
        }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: SubRange<TsTypeParameter>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_parameter_declaration
                .params = params;
        }
    }
}
impl TsTypeParameterInstantiation {
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsType> {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_parameter_instantiation
                .params
        }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: SubRange<TsType>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_parameter_instantiation
                .params = params;
        }
    }
}
impl TsLiteralType {
    #[inline]
    pub fn literal<'a>(self, ast: &crate::Ast<'a>) -> TsLiteral {
        unsafe { ast.nodes.data_unchecked(self.0).ts_literal_type.literal }
    }
    #[inline]
    pub fn set_literal<'a>(self, ast: &mut crate::Ast<'a>, literal: TsLiteral) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_literal_type.literal = literal;
        }
    }
}
impl TsTemplateLiteralType {
    #[inline]
    pub fn quasis<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TemplateElement> {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_template_literal_type
                .quasis
        }
    }
    #[inline]
    pub fn types<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsType> {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_template_literal_type
                .types
        }
    }
    #[inline]
    pub fn set_quasis<'a>(self, ast: &mut crate::Ast<'a>, quasis: SubRange<TemplateElement>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_template_literal_type
                .quasis = quasis;
        }
    }
    #[inline]
    pub fn set_types<'a>(self, ast: &mut crate::Ast<'a>, types: SubRange<TsType>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_template_literal_type
                .types = types;
        }
    }
}
impl TsArrayType {
    #[inline]
    pub fn element_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe { ast.nodes.data_unchecked(self.0).ts_array_type.element_type }
    }
    #[inline]
    pub fn set_element_type<'a>(self, ast: &mut crate::Ast<'a>, element_type: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_array_type
                .element_type = element_type;
        }
    }
}
impl TsIndexedAccessType {
    #[inline]
    pub fn object_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_indexed_access_type
                .object_type
        }
    }
    #[inline]
    pub fn index_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_indexed_access_type
                .index_type
        }
    }
    #[inline]
    pub fn set_object_type<'a>(self, ast: &mut crate::Ast<'a>, object_type: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_indexed_access_type
                .object_type = object_type;
        }
    }
    #[inline]
    pub fn set_index_type<'a>(self, ast: &mut crate::Ast<'a>, index_type: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_indexed_access_type
                .index_type = index_type;
        }
    }
}
impl TsTupleType {
    #[inline]
    pub fn element_types<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsTupleElement> {
        unsafe { ast.nodes.data_unchecked(self.0).ts_tuple_type.element_types }
    }
    #[inline]
    pub fn set_element_types<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        element_types: SubRange<TsTupleElement>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_tuple_type
                .element_types = element_types;
        }
    }
}
impl TsNamedTupleMember {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> IdentifierName {
        unsafe { ast.nodes.data_unchecked(self.0).ts_named_tuple_member.label }
    }
    #[inline]
    pub fn element_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_named_tuple_member
                .element_type
        }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_named_tuple_member
                .optional
        }
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: IdentifierName) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_named_tuple_member
                .label = label;
        }
    }
    #[inline]
    pub fn set_element_type<'a>(self, ast: &mut crate::Ast<'a>, element_type: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_named_tuple_member
                .element_type = element_type;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_named_tuple_member
                .optional = optional;
        }
    }
}
impl TsOptionalType {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_optional_type
                .type_annotation
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_optional_type
                .type_annotation = type_annotation;
        }
    }
}
impl TsRestType {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsTupleElement {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_rest_type
                .type_annotation
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: TsTupleElement,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_rest_type
                .type_annotation = type_annotation;
        }
    }
}
impl TsJsDocNullableType {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_js_doc_nullable_type
                .type_annotation
        }
    }
    #[inline]
    pub fn postfix<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_js_doc_nullable_type
                .postfix
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_js_doc_nullable_type
                .type_annotation = type_annotation;
        }
    }
    #[inline]
    pub fn set_postfix<'a>(self, ast: &mut crate::Ast<'a>, postfix: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_js_doc_nullable_type
                .postfix = postfix;
        }
    }
}
impl TsJsDocNonNullableType {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_js_doc_non_nullable_type
                .type_annotation
        }
    }
    #[inline]
    pub fn postfix<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_js_doc_non_nullable_type
                .postfix
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_js_doc_non_nullable_type
                .type_annotation = type_annotation;
        }
    }
    #[inline]
    pub fn set_postfix<'a>(self, ast: &mut crate::Ast<'a>, postfix: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_js_doc_non_nullable_type
                .postfix = postfix;
        }
    }
}
impl TsJsDocUnknownType {}
impl TsUnionType {
    #[inline]
    pub fn types<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsType> {
        unsafe { ast.nodes.data_unchecked(self.0).ts_union_type.types }
    }
    #[inline]
    pub fn set_types<'a>(self, ast: &mut crate::Ast<'a>, types: SubRange<TsType>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_union_type.types = types;
        }
    }
}
impl TsIntersectionType {
    #[inline]
    pub fn types<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsType> {
        unsafe { ast.nodes.data_unchecked(self.0).ts_intersection_type.types }
    }
    #[inline]
    pub fn set_types<'a>(self, ast: &mut crate::Ast<'a>, types: SubRange<TsType>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_intersection_type
                .types = types;
        }
    }
}
impl TsConditionalType {
    #[inline]
    pub fn check_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_conditional_type
                .check_type
        }
    }
    #[inline]
    pub fn extends_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_conditional_type
                .extends_type
        }
    }
    #[inline]
    pub fn true_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_conditional_type
                .true_type
        }
    }
    #[inline]
    pub fn false_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_conditional_type
                .false_type
        }
    }
    #[inline]
    pub fn set_check_type<'a>(self, ast: &mut crate::Ast<'a>, check_type: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_conditional_type
                .check_type = check_type;
        }
    }
    #[inline]
    pub fn set_extends_type<'a>(self, ast: &mut crate::Ast<'a>, extends_type: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_conditional_type
                .extends_type = extends_type;
        }
    }
    #[inline]
    pub fn set_true_type<'a>(self, ast: &mut crate::Ast<'a>, true_type: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_conditional_type
                .true_type = true_type;
        }
    }
    #[inline]
    pub fn set_false_type<'a>(self, ast: &mut crate::Ast<'a>, false_type: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_conditional_type
                .false_type = false_type;
        }
    }
}
impl TsInferType {
    #[inline]
    pub fn type_parameter<'a>(self, ast: &crate::Ast<'a>) -> TsTypeParameter {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_infer_type
                .type_parameter
        }
    }
    #[inline]
    pub fn set_type_parameter<'a>(self, ast: &mut crate::Ast<'a>, type_parameter: TsTypeParameter) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_infer_type
                .type_parameter = type_parameter;
        }
    }
}
impl TsTypeOperator {
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a>) -> TsTypeOperatorKind {
        unsafe { ast.nodes.data_unchecked(self.0).ts_type_operator.operator }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_operator
                .type_annotation
        }
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a>, operator: TsTypeOperatorKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_operator
                .operator = operator;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_operator
                .type_annotation = type_annotation;
        }
    }
}
impl TsParenthesizedType {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_parenthesized_type
                .type_annotation
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_parenthesized_type
                .type_annotation = type_annotation;
        }
    }
}
impl TsFunctionType {
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_function_type
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a>) -> Option<TsThisParameter> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).ts_function_type.this_param };
        (raw != u32::MAX).then(|| TsThisParameter(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        unsafe { ast.nodes.data_unchecked(self.0).ts_function_type.params }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> TsTypeAnnotation {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_function_type
                .return_type
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_function_type
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_this_param<'a>(self, ast: &mut crate::Ast<'a>, this_param: Option<TsThisParameter>) {
        let field_val: u32 = this_param.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_function_type
                .this_param = field_val;
        }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_function_type.params = params;
        }
    }
    #[inline]
    pub fn set_return_type<'a>(self, ast: &mut crate::Ast<'a>, return_type: TsTypeAnnotation) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_function_type
                .return_type = return_type;
        }
    }
}
impl TsConstructorType {
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_constructor_type
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        unsafe { ast.nodes.data_unchecked(self.0).ts_constructor_type.params }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> TsTypeAnnotation {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_constructor_type
                .return_type
        }
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_constructor_type
                .r#abstract
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_constructor_type
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_constructor_type
                .params = params;
        }
    }
    #[inline]
    pub fn set_return_type<'a>(self, ast: &mut crate::Ast<'a>, return_type: TsTypeAnnotation) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_constructor_type
                .return_type = return_type;
        }
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a>, r#abstract: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_constructor_type
                .r#abstract = r#abstract;
        }
    }
}
impl TsTypePredicate {
    #[inline]
    pub fn parameter_name<'a>(self, ast: &crate::Ast<'a>) -> TsTypePredicateName {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_predicate
                .parameter_name
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_predicate
                .type_annotation
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn asserts<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).ts_type_predicate.asserts }
    }
    #[inline]
    pub fn set_parameter_name<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        parameter_name: TsTypePredicateName,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_predicate
                .parameter_name = parameter_name;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_predicate
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_asserts<'a>(self, ast: &mut crate::Ast<'a>, asserts: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_predicate
                .asserts = asserts;
        }
    }
}
impl TsTypeLiteral {
    #[inline]
    pub fn members<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsTypeElement> {
        unsafe { ast.nodes.data_unchecked(self.0).ts_type_literal.members }
    }
    #[inline]
    pub fn set_members<'a>(self, ast: &mut crate::Ast<'a>, members: SubRange<TsTypeElement>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_type_literal.members = members;
        }
    }
}
impl TsMappedType {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        unsafe { ast.nodes.data_unchecked(self.0).ts_mapped_type.key }
    }
    #[inline]
    pub fn constraint<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe { ast.nodes.data_unchecked(self.0).ts_mapped_type.constraint }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> TsMappedTypeModifier {
        unsafe { ast.nodes.data_unchecked(self.0).ts_mapped_type.optional }
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a>) -> TsMappedTypeModifier {
        unsafe { ast.nodes.data_unchecked(self.0).ts_mapped_type.readonly }
    }
    #[inline]
    pub fn name_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsType> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_mapped_type
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsType> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_mapped_type
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: BindingIdentifier) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_mapped_type.key = key;
        }
    }
    #[inline]
    pub fn set_constraint<'a>(self, ast: &mut crate::Ast<'a>, constraint: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_mapped_type
                .constraint = constraint;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: TsMappedTypeModifier) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_mapped_type.optional = optional;
        }
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a>, readonly: TsMappedTypeModifier) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_mapped_type.readonly = readonly;
        }
    }
    #[inline]
    pub fn set_name_type<'a>(self, ast: &mut crate::Ast<'a>, name_type: Option<TsType>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_mapped_type
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = name_type.into_extra_data();
        };
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsType>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_mapped_type
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = type_annotation.into_extra_data();
        };
    }
}
impl TsPropertySignature {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        unsafe { ast.nodes.data_unchecked(self.0).ts_property_signature.key }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_property_signature
                .type_annotation
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_property_signature
                .computed
        }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_property_signature
                .optional
        }
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_property_signature
                .readonly
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropertyKey) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_property_signature
                .key = key;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_property_signature
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_property_signature
                .computed = computed;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_property_signature
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a>, readonly: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_property_signature
                .readonly = readonly;
        }
    }
}
impl TsMethodSignature {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        unsafe { ast.nodes.data_unchecked(self.0).ts_method_signature.key }
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_signature
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> TsMethodSignatureKind {
        unsafe { ast.nodes.data_unchecked(self.0).ts_method_signature.kind }
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_signature
                .computed
        }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_signature
                .optional
        }
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a>) -> Option<TsThisParameter> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_signature
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_signature
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_signature
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropertyKey) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_method_signature.key = key;
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_method_signature
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: TsMethodSignatureKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_method_signature
                .kind = kind;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_method_signature
                .computed = computed;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_method_signature
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_this_param<'a>(self, ast: &mut crate::Ast<'a>, this_param: Option<TsThisParameter>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_signature
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = this_param.into_extra_data();
        };
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_signature
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = params.into_extra_data();
        };
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        return_type: Option<TsTypeAnnotation>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_method_signature
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = return_type.into_extra_data();
        };
    }
}
impl TsCallSignatureDeclaration {
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_call_signature_declaration
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a>) -> Option<TsThisParameter> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_call_signature_declaration
                .this_param
        };
        (raw != u32::MAX).then(|| TsThisParameter(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_call_signature_declaration
                .params
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_call_signature_declaration
                .return_type
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_call_signature_declaration
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_this_param<'a>(self, ast: &mut crate::Ast<'a>, this_param: Option<TsThisParameter>) {
        let field_val: u32 = this_param.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_call_signature_declaration
                .this_param = field_val;
        }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_call_signature_declaration
                .params = params;
        }
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        return_type: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = return_type.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_call_signature_declaration
                .return_type = field_val;
        }
    }
}
impl TsConstructSignatureDeclaration {
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_construct_signature_declaration
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_construct_signature_declaration
                .params
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_construct_signature_declaration
                .return_type
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_construct_signature_declaration
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_construct_signature_declaration
                .params = params;
        }
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        return_type: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = return_type.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_construct_signature_declaration
                .return_type = field_val;
        }
    }
}
impl TsIndexSignature {
    #[inline]
    pub fn parameters<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsIndexSignatureParameter> {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_index_signature
                .parameters
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsTypeAnnotation {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_index_signature
                .type_annotation
        }
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).ts_index_signature.readonly }
    }
    #[inline]
    pub fn r#static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).ts_index_signature.r#static }
    }
    #[inline]
    pub fn set_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        parameters: SubRange<TsIndexSignatureParameter>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_index_signature
                .parameters = parameters;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: TsTypeAnnotation,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_index_signature
                .type_annotation = type_annotation;
        }
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a>, readonly: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_index_signature
                .readonly = readonly;
        }
    }
    #[inline]
    pub fn set_static<'a>(self, ast: &mut crate::Ast<'a>, r#static: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_index_signature
                .r#static = r#static;
        }
    }
}
impl TsTypeAliasDeclaration {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_alias_declaration
                .id
        }
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_alias_declaration
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_alias_declaration
                .type_annotation
        }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_alias_declaration
                .declare
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: BindingIdentifier) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_alias_declaration
                .id = id;
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_alias_declaration
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_alias_declaration
                .type_annotation = type_annotation;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_alias_declaration
                .declare = declare;
        }
    }
}
impl TsInterfaceDeclaration {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        unsafe { ast.nodes.data_unchecked(self.0).ts_interface_declaration.id }
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_interface_declaration
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_interface_declaration
                .declare
        }
    }
    #[inline]
    pub fn extends<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsInterfaceHeritage> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_interface_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TsInterfaceBody {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_interface_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: BindingIdentifier) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_interface_declaration
                .id = id;
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_interface_declaration
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_interface_declaration
                .declare = declare;
        }
    }
    #[inline]
    pub fn set_extends<'a>(self, ast: &mut crate::Ast<'a>, extends: SubRange<TsInterfaceHeritage>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_interface_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = extends.into_extra_data();
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TsInterfaceBody) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_interface_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.into_extra_data();
        };
    }
}
impl TsInterfaceBody {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsTypeElement> {
        unsafe { ast.nodes.data_unchecked(self.0).ts_interface_body.body }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: SubRange<TsTypeElement>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_interface_body.body = body;
        }
    }
}
impl TsInterfaceHeritage {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_interface_heritage
                .expression
        }
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_interface_heritage
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_interface_heritage
                .expression = expression;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_interface_heritage
                .type_arguments = field_val;
        }
    }
}
impl TsClassImplements {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_class_implements
                .expression
        }
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_class_implements
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_class_implements
                .expression = expression;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_class_implements
                .type_arguments = field_val;
        }
    }
}
impl TsEnumDeclaration {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        unsafe { ast.nodes.data_unchecked(self.0).ts_enum_declaration.id }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TsEnumBody {
        unsafe { ast.nodes.data_unchecked(self.0).ts_enum_declaration.body }
    }
    #[inline]
    pub fn is_const<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_enum_declaration
                .is_const
        }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).ts_enum_declaration.declare }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: BindingIdentifier) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_enum_declaration.id = id;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TsEnumBody) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_enum_declaration
                .body = body;
        }
    }
    #[inline]
    pub fn set_is_const<'a>(self, ast: &mut crate::Ast<'a>, is_const: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_enum_declaration
                .is_const = is_const;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_enum_declaration
                .declare = declare;
        }
    }
}
impl TsEnumBody {
    #[inline]
    pub fn members<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsEnumMember> {
        unsafe { ast.nodes.data_unchecked(self.0).ts_enum_body.members }
    }
    #[inline]
    pub fn set_members<'a>(self, ast: &mut crate::Ast<'a>, members: SubRange<TsEnumMember>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_enum_body.members = members;
        }
    }
}
impl TsEnumMember {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> TsEnumMemberName {
        unsafe { ast.nodes.data_unchecked(self.0).ts_enum_member.id }
    }
    #[inline]
    pub fn initializer<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).ts_enum_member.initializer };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0).ts_enum_member.computed }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: TsEnumMemberName) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_enum_member.id = id;
        }
    }
    #[inline]
    pub fn set_initializer<'a>(self, ast: &mut crate::Ast<'a>, initializer: Option<Expr>) {
        let field_val: u32 = initializer.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_enum_member
                .initializer = field_val;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_enum_member.computed = computed;
        }
    }
}
impl TsModuleDeclaration {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> TsModuleName {
        unsafe { ast.nodes.data_unchecked(self.0).ts_module_declaration.id }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Option<TsModuleBody> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).ts_module_declaration.body };
        (raw != u32::MAX).then(|| TsModuleBody(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> TsModuleDeclarationKind {
        unsafe { ast.nodes.data_unchecked(self.0).ts_module_declaration.kind }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_module_declaration
                .declare
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: TsModuleName) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_module_declaration
                .id = id;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Option<TsModuleBody>) {
        let field_val: u32 = body.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_module_declaration
                .body = field_val;
        }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: TsModuleDeclarationKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_module_declaration
                .kind = kind;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_module_declaration
                .declare = declare;
        }
    }
}
impl TsModuleBlock {
    #[inline]
    pub fn directives<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Directive> {
        unsafe { ast.nodes.data_unchecked(self.0).ts_module_block.directives }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Stmt> {
        unsafe { ast.nodes.data_unchecked(self.0).ts_module_block.body }
    }
    #[inline]
    pub fn set_directives<'a>(self, ast: &mut crate::Ast<'a>, directives: SubRange<Directive>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_module_block
                .directives = directives;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: SubRange<Stmt>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).ts_module_block.body = body;
        }
    }
}
impl TsGlobalDeclaration {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> IdentifierName {
        unsafe { ast.nodes.data_unchecked(self.0).ts_global_declaration.id }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TsModuleBlock {
        unsafe { ast.nodes.data_unchecked(self.0).ts_global_declaration.body }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_global_declaration
                .declare
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: IdentifierName) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_global_declaration
                .id = id;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TsModuleBlock) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_global_declaration
                .body = body;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_global_declaration
                .declare = declare;
        }
    }
}
impl TsParameterProperty {
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Decorator> {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_parameter_property
                .decorators
        }
    }
    #[inline]
    pub fn parameter<'a>(self, ast: &crate::Ast<'a>) -> FormalParameter {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_parameter_property
                .parameter
        }
    }
    #[inline]
    pub fn r#override<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_parameter_property
                .r#override
        }
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_parameter_property
                .readonly
        }
    }
    #[inline]
    pub fn accessibility<'a>(self, ast: &crate::Ast<'a>) -> Accessibility {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_parameter_property
                .accessibility
        }
    }
    #[inline]
    pub fn set_decorators<'a>(self, ast: &mut crate::Ast<'a>, decorators: SubRange<Decorator>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_parameter_property
                .decorators = decorators;
        }
    }
    #[inline]
    pub fn set_parameter<'a>(self, ast: &mut crate::Ast<'a>, parameter: FormalParameter) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_parameter_property
                .parameter = parameter;
        }
    }
    #[inline]
    pub fn set_override<'a>(self, ast: &mut crate::Ast<'a>, r#override: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_parameter_property
                .r#override = r#override;
        }
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a>, readonly: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_parameter_property
                .readonly = readonly;
        }
    }
    #[inline]
    pub fn set_accessibility<'a>(self, ast: &mut crate::Ast<'a>, accessibility: Accessibility) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_parameter_property
                .accessibility = accessibility;
        }
    }
}
impl TsThisParameter {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_this_parameter
                .type_annotation
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_this_parameter
                .type_annotation = field_val;
        }
    }
}
impl TsAsExpression {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).ts_as_expression.expression }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_as_expression
                .type_annotation
        }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_as_expression
                .expression = expression;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_as_expression
                .type_annotation = type_annotation;
        }
    }
}
impl TsSatisfiesExpression {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_satisfies_expression
                .expression
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_satisfies_expression
                .type_annotation
        }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_satisfies_expression
                .expression = expression;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_satisfies_expression
                .type_annotation = type_annotation;
        }
    }
}
impl TsTypeAssertion {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_assertion
                .type_annotation
        }
    }
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_type_assertion
                .expression
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_assertion
                .type_annotation = type_annotation;
        }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_type_assertion
                .expression = expression;
        }
    }
}
impl TsNonNullExpression {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_non_null_expression
                .expression
        }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_non_null_expression
                .expression = expression;
        }
    }
}
impl TsInstantiationExpression {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_instantiation_expression
                .expression
        }
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> TsTypeParameterInstantiation {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_instantiation_expression
                .type_arguments
        }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_instantiation_expression
                .expression = expression;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: TsTypeParameterInstantiation,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_instantiation_expression
                .type_arguments = type_arguments;
        }
    }
}
impl TsExportAssignment {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_export_assignment
                .expression
        }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_export_assignment
                .expression = expression;
        }
    }
}
impl TsNamespaceExportDeclaration {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> IdentifierName {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_namespace_export_declaration
                .id
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: IdentifierName) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_namespace_export_declaration
                .id = id;
        }
    }
}
impl TsImportEqualsDeclaration {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_import_equals_declaration
                .id
        }
    }
    #[inline]
    pub fn module_reference<'a>(self, ast: &crate::Ast<'a>) -> TsModuleReference {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_import_equals_declaration
                .module_reference
        }
    }
    #[inline]
    pub fn import_kind<'a>(self, ast: &crate::Ast<'a>) -> ImportOrExportKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_import_equals_declaration
                .import_kind
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: BindingIdentifier) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_import_equals_declaration
                .id = id;
        }
    }
    #[inline]
    pub fn set_module_reference<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        module_reference: TsModuleReference,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_import_equals_declaration
                .module_reference = module_reference;
        }
    }
    #[inline]
    pub fn set_import_kind<'a>(self, ast: &mut crate::Ast<'a>, import_kind: ImportOrExportKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_import_equals_declaration
                .import_kind = import_kind;
        }
    }
}
impl TsExternalModuleReference {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> StringLiteral {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_external_module_reference
                .expression
        }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: StringLiteral) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_external_module_reference
                .expression = expression;
        }
    }
}
impl TsIndexSignatureParameter {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_index_signature_parameter
                .name
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsTypeAnnotation {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .ts_index_signature_parameter
                .type_annotation
        }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: BindingIdentifier) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_index_signature_parameter
                .name = name;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: TsTypeAnnotation,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .ts_index_signature_parameter
                .type_annotation = type_annotation;
        }
    }
}
impl JsxElementName {
    #[inline]
    pub fn is_jsx_identifier(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxIdentifier
    }
    #[inline]
    pub fn is_jsx_namespaced_name(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxNamespacedName
    }
    #[inline]
    pub fn is_jsx_member_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxMemberExpression
    }
    #[inline]
    pub fn as_jsx_identifier(self, ast: &Ast<'_>) -> Option<JsxIdentifier> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxIdentifier) {
            Some(JsxIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self, ast: &Ast<'_>) -> Option<JsxNamespacedName> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxNamespacedName) {
            Some(JsxNamespacedName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_member_expression(self, ast: &Ast<'_>) -> Option<JsxMemberExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxMemberExpression) {
            Some(JsxMemberExpression(self.0))
        } else {
            None
        }
    }
}
impl JsxMemberExpressionObject {
    #[inline]
    pub fn is_jsx_identifier(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxIdentifier
    }
    #[inline]
    pub fn is_jsx_member_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxMemberExpression
    }
    #[inline]
    pub fn as_jsx_identifier(self, ast: &Ast<'_>) -> Option<JsxIdentifier> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxIdentifier) {
            Some(JsxIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_member_expression(self, ast: &Ast<'_>) -> Option<JsxMemberExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxMemberExpression) {
            Some(JsxMemberExpression(self.0))
        } else {
            None
        }
    }
}
impl JsxAttributeName {
    #[inline]
    pub fn is_jsx_identifier(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxIdentifier
    }
    #[inline]
    pub fn is_jsx_namespaced_name(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxNamespacedName
    }
    #[inline]
    pub fn as_jsx_identifier(self, ast: &Ast<'_>) -> Option<JsxIdentifier> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxIdentifier) {
            Some(JsxIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self, ast: &Ast<'_>) -> Option<JsxNamespacedName> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxNamespacedName) {
            Some(JsxNamespacedName(self.0))
        } else {
            None
        }
    }
}
impl JsxAttributeItem {
    #[inline]
    pub fn is_jsx_attribute(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxAttribute
    }
    #[inline]
    pub fn is_jsx_spread_attribute(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxSpreadAttribute
    }
    #[inline]
    pub fn as_jsx_attribute(self, ast: &Ast<'_>) -> Option<JsxAttribute> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxAttribute) {
            Some(JsxAttribute(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_spread_attribute(self, ast: &Ast<'_>) -> Option<JsxSpreadAttribute> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxSpreadAttribute) {
            Some(JsxSpreadAttribute(self.0))
        } else {
            None
        }
    }
}
impl JsxAttributeValue {
    #[inline]
    pub fn is_string_literal(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::StringLiteral
    }
    #[inline]
    pub fn is_jsx_expression_container(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxExpressionContainer
    }
    #[inline]
    pub fn is_jsx_element(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxElement
    }
    #[inline]
    pub fn is_jsx_fragment(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxFragment
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_>) -> Option<StringLiteral> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::StringLiteral) {
            Some(StringLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_expression_container(self, ast: &Ast<'_>) -> Option<JsxExpressionContainer> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxExpressionContainer) {
            Some(JsxExpressionContainer(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_element(self, ast: &Ast<'_>) -> Option<JsxElement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxElement) {
            Some(JsxElement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self, ast: &Ast<'_>) -> Option<JsxFragment> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxFragment) {
            Some(JsxFragment(self.0))
        } else {
            None
        }
    }
}
impl JsxExpression {
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference
                | NodeKind::ThisExpression
                | NodeKind::Super
                | NodeKind::MetaProperty
                | NodeKind::StringLiteral
                | NodeKind::NumericLiteral
                | NodeKind::BigIntLiteral
                | NodeKind::BooleanLiteral
                | NodeKind::NullLiteral
                | NodeKind::RegExpLiteral
                | NodeKind::TemplateLiteral
                | NodeKind::BinaryExpression
                | NodeKind::LogicalExpression
                | NodeKind::UnaryExpression
                | NodeKind::UpdateExpression
                | NodeKind::AssignmentExpression
                | NodeKind::ConditionalExpression
                | NodeKind::SequenceExpression
                | NodeKind::MemberExpression
                | NodeKind::CallExpression
                | NodeKind::ChainExpression
                | NodeKind::NewExpression
                | NodeKind::TaggedTemplateExpression
                | NodeKind::ArrowFunctionExpression
                | NodeKind::ArrayExpression
                | NodeKind::ObjectExpression
                | NodeKind::ParenthesizedExpression
                | NodeKind::Function
                | NodeKind::Class
                | NodeKind::ImportExpression
                | NodeKind::AwaitExpression
                | NodeKind::YieldExpression
                | NodeKind::PrivateIdentifier
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::TsInstantiationExpression
                | NodeKind::JsxElement
                | NodeKind::JsxFragment
        )
    }
    #[inline]
    pub fn is_jsx_empty_expression(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxEmptyExpression
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(
            kind,
            NodeKind::IdentifierReference
                | NodeKind::ThisExpression
                | NodeKind::Super
                | NodeKind::MetaProperty
                | NodeKind::StringLiteral
                | NodeKind::NumericLiteral
                | NodeKind::BigIntLiteral
                | NodeKind::BooleanLiteral
                | NodeKind::NullLiteral
                | NodeKind::RegExpLiteral
                | NodeKind::TemplateLiteral
                | NodeKind::BinaryExpression
                | NodeKind::LogicalExpression
                | NodeKind::UnaryExpression
                | NodeKind::UpdateExpression
                | NodeKind::AssignmentExpression
                | NodeKind::ConditionalExpression
                | NodeKind::SequenceExpression
                | NodeKind::MemberExpression
                | NodeKind::CallExpression
                | NodeKind::ChainExpression
                | NodeKind::NewExpression
                | NodeKind::TaggedTemplateExpression
                | NodeKind::ArrowFunctionExpression
                | NodeKind::ArrayExpression
                | NodeKind::ObjectExpression
                | NodeKind::ParenthesizedExpression
                | NodeKind::Function
                | NodeKind::Class
                | NodeKind::ImportExpression
                | NodeKind::AwaitExpression
                | NodeKind::YieldExpression
                | NodeKind::PrivateIdentifier
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::TsInstantiationExpression
                | NodeKind::JsxElement
                | NodeKind::JsxFragment
        ) {
            Some(Expr(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_empty_expression(self, ast: &Ast<'_>) -> Option<JsxEmptyExpression> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxEmptyExpression) {
            Some(JsxEmptyExpression(self.0))
        } else {
            None
        }
    }
}
impl JsxChild {
    #[inline]
    pub fn is_jsx_text(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxText
    }
    #[inline]
    pub fn is_jsx_expression_container(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxExpressionContainer
    }
    #[inline]
    pub fn is_jsx_spread_child(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxSpreadChild
    }
    #[inline]
    pub fn is_jsx_element(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxElement
    }
    #[inline]
    pub fn is_jsx_fragment(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxFragment
    }
    #[inline]
    pub fn as_jsx_text(self, ast: &Ast<'_>) -> Option<JsxText> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxText) {
            Some(JsxText(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_expression_container(self, ast: &Ast<'_>) -> Option<JsxExpressionContainer> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxExpressionContainer) {
            Some(JsxExpressionContainer(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_spread_child(self, ast: &Ast<'_>) -> Option<JsxSpreadChild> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxSpreadChild) {
            Some(JsxSpreadChild(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_element(self, ast: &Ast<'_>) -> Option<JsxElement> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxElement) {
            Some(JsxElement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self, ast: &Ast<'_>) -> Option<JsxFragment> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0) };
        if matches!(kind, NodeKind::JsxFragment) {
            Some(JsxFragment(self.0))
        } else {
            None
        }
    }
}
impl JsxElement {
    #[inline]
    pub fn opening_element<'a>(self, ast: &crate::Ast<'a>) -> JsxOpeningElement {
        unsafe { ast.nodes.data_unchecked(self.0).jsx_element.opening_element }
    }
    #[inline]
    pub fn children<'a>(self, ast: &crate::Ast<'a>) -> SubRange<JsxChild> {
        unsafe { ast.nodes.data_unchecked(self.0).jsx_element.children }
    }
    #[inline]
    pub fn closing_element<'a>(self, ast: &crate::Ast<'a>) -> Option<JsxClosingElement> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).jsx_element.closing_element };
        (raw != u32::MAX).then(|| JsxClosingElement(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_opening_element<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        opening_element: JsxOpeningElement,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_element
                .opening_element = opening_element;
        }
    }
    #[inline]
    pub fn set_children<'a>(self, ast: &mut crate::Ast<'a>, children: SubRange<JsxChild>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).jsx_element.children = children;
        }
    }
    #[inline]
    pub fn set_closing_element<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        closing_element: Option<JsxClosingElement>,
    ) {
        let field_val: u32 = closing_element.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_element
                .closing_element = field_val;
        }
    }
}
impl JsxOpeningElement {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JsxElementName {
        unsafe { ast.nodes.data_unchecked(self.0).jsx_opening_element.name }
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .jsx_opening_element
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn self_closing<'a>(self, ast: &crate::Ast<'a>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .jsx_opening_element
                .self_closing
        }
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a>) -> SubRange<JsxAttributeItem> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .jsx_opening_element
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JsxElementName) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_opening_element
                .name = name;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_opening_element
                .type_arguments = field_val;
        }
    }
    #[inline]
    pub fn set_self_closing<'a>(self, ast: &mut crate::Ast<'a>, self_closing: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_opening_element
                .self_closing = self_closing;
        }
    }
    #[inline]
    pub fn set_attributes<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        attributes: SubRange<JsxAttributeItem>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .jsx_opening_element
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = attributes.into_extra_data();
        };
    }
}
impl JsxClosingElement {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JsxElementName {
        unsafe { ast.nodes.data_unchecked(self.0).jsx_closing_element.name }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JsxElementName) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_closing_element
                .name = name;
        }
    }
}
impl JsxFragment {
    #[inline]
    pub fn opening_fragment<'a>(self, ast: &crate::Ast<'a>) -> JsxOpeningFragment {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .jsx_fragment
                .opening_fragment
        }
    }
    #[inline]
    pub fn children<'a>(self, ast: &crate::Ast<'a>) -> SubRange<JsxChild> {
        unsafe { ast.nodes.data_unchecked(self.0).jsx_fragment.children }
    }
    #[inline]
    pub fn closing_fragment<'a>(self, ast: &crate::Ast<'a>) -> JsxClosingFragment {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .jsx_fragment
                .closing_fragment
        }
    }
    #[inline]
    pub fn set_opening_fragment<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        opening_fragment: JsxOpeningFragment,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_fragment
                .opening_fragment = opening_fragment;
        }
    }
    #[inline]
    pub fn set_children<'a>(self, ast: &mut crate::Ast<'a>, children: SubRange<JsxChild>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).jsx_fragment.children = children;
        }
    }
    #[inline]
    pub fn set_closing_fragment<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        closing_fragment: JsxClosingFragment,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_fragment
                .closing_fragment = closing_fragment;
        }
    }
}
impl JsxOpeningFragment {}
impl JsxClosingFragment {}
impl JsxIdentifier {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).jsx_identifier.name }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).jsx_identifier.name = name;
        }
    }
}
impl JsxNamespacedName {
    #[inline]
    pub fn namespace<'a>(self, ast: &crate::Ast<'a>) -> JsxIdentifier {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .jsx_namespaced_name
                .namespace
        }
    }
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JsxIdentifier {
        unsafe { ast.nodes.data_unchecked(self.0).jsx_namespaced_name.name }
    }
    #[inline]
    pub fn set_namespace<'a>(self, ast: &mut crate::Ast<'a>, namespace: JsxIdentifier) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_namespaced_name
                .namespace = namespace;
        }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JsxIdentifier) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_namespaced_name
                .name = name;
        }
    }
}
impl JsxMemberExpression {
    #[inline]
    pub fn object<'a>(self, ast: &crate::Ast<'a>) -> JsxMemberExpressionObject {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .jsx_member_expression
                .object
        }
    }
    #[inline]
    pub fn property<'a>(self, ast: &crate::Ast<'a>) -> JsxIdentifier {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .jsx_member_expression
                .property
        }
    }
    #[inline]
    pub fn set_object<'a>(self, ast: &mut crate::Ast<'a>, object: JsxMemberExpressionObject) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_member_expression
                .object = object;
        }
    }
    #[inline]
    pub fn set_property<'a>(self, ast: &mut crate::Ast<'a>, property: JsxIdentifier) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_member_expression
                .property = property;
        }
    }
}
impl JsxAttribute {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JsxAttributeName {
        unsafe { ast.nodes.data_unchecked(self.0).jsx_attribute.name }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<JsxAttributeValue> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0).jsx_attribute.value };
        (raw != u32::MAX).then(|| JsxAttributeValue(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JsxAttributeName) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).jsx_attribute.name = name;
        }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<JsxAttributeValue>) {
        let field_val: u32 = value.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).jsx_attribute.value = field_val;
        }
    }
}
impl JsxSpreadAttribute {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .jsx_spread_attribute
                .argument
        }
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_spread_attribute
                .argument = argument;
        }
    }
}
impl JsxExpressionContainer {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> JsxExpression {
        unsafe {
            ast.nodes
                .data_unchecked(self.0)
                .jsx_expression_container
                .expression
        }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: JsxExpression) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_expression_container
                .expression = expression;
        }
    }
}
impl JsxEmptyExpression {}
impl JsxText {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        unsafe { ast.nodes.data_unchecked(self.0).jsx_text.value }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: AstUtf8) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0).jsx_text.value = value;
        }
    }
}
impl JsxSpreadChild {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        unsafe { ast.nodes.data_unchecked(self.0).jsx_spread_child.expression }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0)
                .jsx_spread_child
                .expression = expression;
        }
    }
}
