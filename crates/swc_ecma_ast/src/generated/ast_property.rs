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
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn hashbang<'a>(self, ast: &crate::Ast<'a>) -> Option<Hashbang> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Hashbang(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Stmt> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_directives<'a>(self, ast: &mut crate::Ast<'a>, directives: SubRange<Directive>) {
        let field_val: u64 = directives.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300949110269842519228415u128)
            | (((field_val as u128) & 18446744073709551615u128) << 32usize);
    }
    #[inline]
    pub fn set_hashbang<'a>(self, ast: &mut crate::Ast<'a>, hashbang: Option<Hashbang>) {
        let field_val: u32 = hashbang.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: SubRange<Stmt>) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: AstWtf8) {
        let field_val: u64 = value.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: AstUtf8) {
        let field_val: u64 = raw.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl NumericLiteral {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> f64 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> Option<AstUtf8> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: f64) {
        let field_val: u64 = value.into_extra_data().raw();
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        if raw == field_val {
            return;
        }
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
        self.set_raw(ast, None);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: Option<AstUtf8>) {
        let field_val: u64 = raw.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl BigIntLiteral {
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: AstUtf8) {
        let field_val: u64 = raw.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl BooleanLiteral {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: bool) {
        let field_val: u32 = value as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
}
impl NullLiteral {}
impl RegExpLiteral {
    #[inline]
    pub fn pattern<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn flags<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_pattern<'a>(self, ast: &mut crate::Ast<'a>, pattern: AstUtf8) {
        let field_val: u64 = pattern.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_flags<'a>(self, ast: &mut crate::Ast<'a>, flags: AstUtf8) {
        let field_val: u64 = flags.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl TemplateLiteral {
    #[inline]
    pub fn quasis<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TemplateElement> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn expressions<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Expr> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_quasis<'a>(self, ast: &mut crate::Ast<'a>, quasis: SubRange<TemplateElement>) {
        let field_val: u64 = quasis.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_expressions<'a>(self, ast: &mut crate::Ast<'a>, expressions: SubRange<Expr>) {
        let field_val: u64 = expressions.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl TemplateElement {
    #[inline]
    pub fn cooked<'a>(self, ast: &crate::Ast<'a>) -> AstWtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 16usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn tail<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn is_cooked_undefined<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_cooked<'a>(self, ast: &mut crate::Ast<'a>, cooked: AstWtf8) {
        let field_val: u64 = cooked.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937254537554992802593570815u128)
            | (((field_val as u128) & 18446744073709551615u128) << 16usize);
    }
    #[inline]
    pub fn set_tail<'a>(self, ast: &mut crate::Ast<'a>, tail: bool) {
        let field_val: u32 = tail as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_is_cooked_undefined<'a>(self, ast: &mut crate::Ast<'a>, is_cooked_undefined: bool) {
        let field_val: u32 = is_cooked_undefined as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768146175u128)
            | (((field_val as u128) & 255u128) << 8usize);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: AstUtf8) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        let field_val: u64 = name.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl PrivateIdentifier {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        let field_val: u64 = name.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl BindingIdentifier {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        let field_val: u64 = name.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl IdentifierName {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        let field_val: u64 = name.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl LabelIdentifier {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        let field_val: u64 = name.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl AssignmentPattern {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> BindingPattern {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        BindingPattern(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: BindingPattern) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl BindingRestElement {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> BindingPattern {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        BindingPattern(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: BindingPattern) {
        let field_val: u32 = argument.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ArrayPattern {
    #[inline]
    pub fn elements<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Option<BindingPattern>> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn rest<'a>(self, ast: &crate::Ast<'a>) -> Option<BindingRestElement> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| BindingRestElement(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_elements<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        elements: SubRange<Option<BindingPattern>>,
    ) {
        let field_val: u64 = elements.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_rest<'a>(self, ast: &mut crate::Ast<'a>, rest: Option<BindingRestElement>) {
        let field_val: u32 = rest.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl ObjectPattern {
    #[inline]
    pub fn properties<'a>(self, ast: &crate::Ast<'a>) -> SubRange<BindingProperty> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn rest<'a>(self, ast: &crate::Ast<'a>) -> Option<BindingRestElement> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| BindingRestElement(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_properties<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        properties: SubRange<BindingProperty>,
    ) {
        let field_val: u64 = properties.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_rest<'a>(self, ast: &mut crate::Ast<'a>, rest: Option<BindingRestElement>) {
        let field_val: u32 = rest.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl BindingProperty {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        PropertyKey(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> BindingPattern {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        BindingPattern(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn shorthand<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 72usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropertyKey) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: BindingPattern) {
        let field_val: u32 = value.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_shorthand<'a>(self, ast: &mut crate::Ast<'a>, shorthand: bool) {
        let field_val: u32 = shorthand as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        let field_val: u32 = computed as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937259259921475672238718975u128)
            | (((field_val as u128) & 255u128) << 72usize);
    }
}
impl BinaryExpression {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a>) -> BinaryOperator {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, BinaryOperator>(raw as u8) }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: Expr) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a>, operator: BinaryOperator) {
        let field_val: u32 = operator as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
}
impl LogicalExpression {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a>) -> LogicalOperator {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, LogicalOperator>(raw as u8) }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: Expr) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a>, operator: LogicalOperator) {
        let field_val: u32 = operator as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
}
impl ConditionalExpression {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn consequent<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn alternate<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u32 = test.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_consequent<'a>(self, ast: &mut crate::Ast<'a>, consequent: Expr) {
        let field_val: u32 = consequent.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_alternate<'a>(self, ast: &mut crate::Ast<'a>, alternate: Expr) {
        let field_val: u32 = alternate.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl UnaryExpression {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a>) -> UnaryOperator {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, UnaryOperator>(raw as u8) }
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: Expr) {
        let field_val: u32 = argument.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a>, operator: UnaryOperator) {
        let field_val: u32 = operator as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606336551550975u128)
            | (((field_val as u128) & 255u128) << 32usize);
    }
}
impl UpdateExpression {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> SimpleAssignmentTarget {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        SimpleAssignmentTarget(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a>) -> UpdateOperator {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, UpdateOperator>(raw as u8) }
    }
    #[inline]
    pub fn prefix<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: SimpleAssignmentTarget) {
        let field_val: u32 = argument.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a>, operator: UpdateOperator) {
        let field_val: u32 = operator as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606336551550975u128)
            | (((field_val as u128) & 255u128) << 32usize);
    }
    #[inline]
    pub fn set_prefix<'a>(self, ast: &mut crate::Ast<'a>, prefix: bool) {
        let field_val: u32 = prefix as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374327056303128575u128)
            | (((field_val as u128) & 255u128) << 40usize);
    }
}
impl AssignmentExpression {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> AssignmentTarget {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        AssignmentTarget(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a>) -> AssignmentOperator {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, AssignmentOperator>(raw as u8) }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: AssignmentTarget) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a>, operator: AssignmentOperator) {
        let field_val: u32 = operator as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
}
impl VariableDeclaration {
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> VariableKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, VariableKind>(raw as u8) }
    }
    #[inline]
    pub fn declarators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<VariableDeclarator> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 72usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: VariableKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_declarators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        declarators: SubRange<VariableDeclarator>,
    ) {
        let field_val: u64 = declarators.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937259259921475672238718975u128)
            | (((field_val as u128) & 255u128) << 72usize);
    }
}
impl VariableDeclarator {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> BindingPattern {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        BindingPattern(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn init<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn definite<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: BindingPattern) {
        let field_val: u32 = id.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_init<'a>(self, ast: &mut crate::Ast<'a>, init: Option<Expr>) {
        let field_val: u32 = init.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_definite<'a>(self, ast: &mut crate::Ast<'a>, definite: bool) {
        let field_val: u32 = definite as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282346717757022325968521078060875775u128)
            | (((field_val as u128) & 255u128) << 96usize);
    }
}
impl IfStatement {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn consequent<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn alternate<'a>(self, ast: &crate::Ast<'a>) -> Option<Stmt> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Stmt(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u32 = test.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_consequent<'a>(self, ast: &mut crate::Ast<'a>, consequent: Stmt) {
        let field_val: u32 = consequent.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_alternate<'a>(self, ast: &mut crate::Ast<'a>, alternate: Option<Stmt>) {
        let field_val: u32 = alternate.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl SwitchStatement {
    #[inline]
    pub fn discriminant<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn cases<'a>(self, ast: &crate::Ast<'a>) -> SubRange<SwitchCase> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_discriminant<'a>(self, ast: &mut crate::Ast<'a>, discriminant: Expr) {
        let field_val: u32 = discriminant.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_cases<'a>(self, ast: &mut crate::Ast<'a>, cases: SubRange<SwitchCase>) {
        let field_val: u64 = cases.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl SwitchCase {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn consequent<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Stmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Option<Expr>) {
        let field_val: u32 = test.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_consequent<'a>(self, ast: &mut crate::Ast<'a>, consequent: SubRange<Stmt>) {
        let field_val: u64 = consequent.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl ForStatement {
    #[inline]
    pub fn init<'a>(self, ast: &crate::Ast<'a>) -> Option<ForStatementInit> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| ForStatementInit(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn update<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_init<'a>(self, ast: &mut crate::Ast<'a>, init: Option<ForStatementInit>) {
        let field_val: u32 = init.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Option<Expr>) {
        let field_val: u32 = test.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_update<'a>(self, ast: &mut crate::Ast<'a>, update: Option<Expr>) {
        let field_val: u32 = update.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 79228162514264337593543950335u128)
            | (((field_val as u128) & 4294967295u128) << 96usize);
    }
}
impl ForInStatement {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> ForStatementLeft {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        ForStatementLeft(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: ForStatementLeft) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl ForOfStatement {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> ForStatementLeft {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        ForStatementLeft(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn r#await<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: ForStatementLeft) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_await<'a>(self, ast: &mut crate::Ast<'a>, r#await: bool) {
        let field_val: u32 = r#await as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282346717757022325968521078060875775u128)
            | (((field_val as u128) & 255u128) << 96usize);
    }
}
impl TryStatement {
    #[inline]
    pub fn block<'a>(self, ast: &crate::Ast<'a>) -> BlockStatement {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        BlockStatement(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn handler<'a>(self, ast: &crate::Ast<'a>) -> Option<CatchClause> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| CatchClause(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn finalizer<'a>(self, ast: &crate::Ast<'a>) -> Option<BlockStatement> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| BlockStatement(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_block<'a>(self, ast: &mut crate::Ast<'a>, block: BlockStatement) {
        let field_val: u32 = block.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_handler<'a>(self, ast: &mut crate::Ast<'a>, handler: Option<CatchClause>) {
        let field_val: u32 = handler.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_finalizer<'a>(self, ast: &mut crate::Ast<'a>, finalizer: Option<BlockStatement>) {
        let field_val: u32 = finalizer.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl CatchClause {
    #[inline]
    pub fn param<'a>(self, ast: &crate::Ast<'a>) -> Option<BindingPattern> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| BindingPattern(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> BlockStatement {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        BlockStatement(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_param<'a>(self, ast: &mut crate::Ast<'a>, param: Option<BindingPattern>) {
        let field_val: u32 = param.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: BlockStatement) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl BreakStatement {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> Option<LabelIdentifier> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| LabelIdentifier(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: Option<LabelIdentifier>) {
        let field_val: u32 = label.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ContinueStatement {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> Option<LabelIdentifier> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| LabelIdentifier(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: Option<LabelIdentifier>) {
        let field_val: u32 = label.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl LabeledStatement {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> LabelIdentifier {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        LabelIdentifier(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: LabelIdentifier) {
        let field_val: u32 = label.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl ExpressionStatement {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ReturnStatement {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: Option<Expr>) {
        let field_val: u32 = argument.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ThrowStatement {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: Expr) {
        let field_val: u32 = argument.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl DebuggerStatement {}
impl EmptyStatement {}
impl Directive {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> StringLiteral {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        StringLiteral(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: StringLiteral) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: AstUtf8) {
        let field_val: u64 = value.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl Decorator {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl Class {
    #[inline]
    pub fn class_type<'a>(self, ast: &crate::Ast<'a>) -> ClassType {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, ClassType>(raw as u8) }
    }
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> Option<BindingIdentifier> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 24usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| BindingIdentifier(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 56usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 16usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Decorator> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn super_class<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn implements<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsClassImplements> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> ClassBody {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(4usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_class_type<'a>(self, ast: &mut crate::Ast<'a>, class_type: ClassType) {
        let field_val: u32 = class_type as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: Option<BindingIdentifier>) {
        let field_val: u32 = id.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463302549837747060735u128)
            | (((field_val as u128) & 4294967295u128) << 24usize);
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920628978453625319957081358335u128)
            | (((field_val as u128) & 4294967295u128) << 56usize);
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a>, r#abstract: bool) {
        let field_val: u32 = r#abstract as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768146175u128)
            | (((field_val as u128) & 255u128) << 8usize);
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431751499775u128)
            | (((field_val as u128) & 255u128) << 16usize);
    }
    #[inline]
    pub fn set_decorators<'a>(self, ast: &mut crate::Ast<'a>, decorators: SubRange<Decorator>) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: SubRange<ClassElement>) {
        let field_val: u64 = body.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl MethodDefinition {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 56usize) as u32) & 4294967295u32;
        PropertyKey(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> MethodDefinitionKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, MethodDefinitionKind>(raw as u8) }
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn r#static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 16usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn r#override<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 24usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn accessibility<'a>(self, ast: &crate::Ast<'a>) -> Accessibility {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 48usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, Accessibility>(raw as u8) }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Decorator> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropertyKey) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920628978453625319957081358335u128)
            | (((field_val as u128) & 4294967295u128) << 56usize);
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: MethodDefinitionKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        let field_val: u32 = computed as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768146175u128)
            | (((field_val as u128) & 255u128) << 8usize);
    }
    #[inline]
    pub fn set_static<'a>(self, ast: &mut crate::Ast<'a>, r#static: bool) {
        let field_val: u32 = r#static as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431751499775u128)
            | (((field_val as u128) & 255u128) << 16usize);
    }
    #[inline]
    pub fn set_override<'a>(self, ast: &mut crate::Ast<'a>, r#override: bool) {
        let field_val: u32 = r#override as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427490021375u128)
            | (((field_val as u128) & 255u128) << 24usize);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606336551550975u128)
            | (((field_val as u128) & 255u128) << 32usize);
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a>, r#abstract: bool) {
        let field_val: u32 = r#abstract as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374327056303128575u128)
            | (((field_val as u128) & 255u128) << 40usize);
    }
    #[inline]
    pub fn set_accessibility<'a>(self, ast: &mut crate::Ast<'a>, accessibility: Accessibility) {
        let field_val: u32 = accessibility as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463302831312706994175u128)
            | (((field_val as u128) & 255u128) << 48usize);
    }
    #[inline]
    pub fn set_decorators<'a>(self, ast: &mut crate::Ast<'a>, decorators: SubRange<Decorator>) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn r#static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn accessor<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 16usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 24usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn r#override<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn definite<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 48usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 56usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn accessibility<'a>(self, ast: &crate::Ast<'a>) -> Accessibility {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 72usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, Accessibility>(raw as u8) }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Decorator> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        let field_val: u32 = computed as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_static<'a>(self, ast: &mut crate::Ast<'a>, r#static: bool) {
        let field_val: u32 = r#static as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768146175u128)
            | (((field_val as u128) & 255u128) << 8usize);
    }
    #[inline]
    pub fn set_accessor<'a>(self, ast: &mut crate::Ast<'a>, accessor: bool) {
        let field_val: u32 = accessor as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431751499775u128)
            | (((field_val as u128) & 255u128) << 16usize);
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427490021375u128)
            | (((field_val as u128) & 255u128) << 24usize);
    }
    #[inline]
    pub fn set_override<'a>(self, ast: &mut crate::Ast<'a>, r#override: bool) {
        let field_val: u32 = r#override as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606336551550975u128)
            | (((field_val as u128) & 255u128) << 32usize);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374327056303128575u128)
            | (((field_val as u128) & 255u128) << 40usize);
    }
    #[inline]
    pub fn set_definite<'a>(self, ast: &mut crate::Ast<'a>, definite: bool) {
        let field_val: u32 = definite as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463302831312706994175u128)
            | (((field_val as u128) & 255u128) << 48usize);
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a>, readonly: bool) {
        let field_val: u32 = readonly as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444999920952096587775u128)
            | (((field_val as u128) & 255u128) << 56usize);
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a>, r#abstract: bool) {
        let field_val: u32 = r#abstract as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_accessibility<'a>(self, ast: &mut crate::Ast<'a>, accessibility: Accessibility) {
        let field_val: u32 = accessibility as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937259259921475672238718975u128)
            | (((field_val as u128) & 255u128) << 72usize);
    }
    #[inline]
    pub fn set_decorators<'a>(self, ast: &mut crate::Ast<'a>, decorators: SubRange<Decorator>) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: SubRange<Stmt>) {
        let field_val: u64 = body.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl Function {
    #[inline]
    pub fn function_type<'a>(self, ast: &crate::Ast<'a>) -> FunctionType {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, FunctionType>(raw as u8) }
    }
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> Option<BindingIdentifier> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 24usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| BindingIdentifier(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 56usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn generator<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn r#async<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 16usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a>) -> Option<TsThisParameter> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> FunctionBody {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_function_type<'a>(self, ast: &mut crate::Ast<'a>, function_type: FunctionType) {
        let field_val: u32 = function_type as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: Option<BindingIdentifier>) {
        let field_val: u32 = id.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463302549837747060735u128)
            | (((field_val as u128) & 4294967295u128) << 24usize);
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920628978453625319957081358335u128)
            | (((field_val as u128) & 4294967295u128) << 56usize);
    }
    #[inline]
    pub fn set_generator<'a>(self, ast: &mut crate::Ast<'a>, generator: bool) {
        let field_val: u32 = generator as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768146175u128)
            | (((field_val as u128) & 255u128) << 8usize);
    }
    #[inline]
    pub fn set_async<'a>(self, ast: &mut crate::Ast<'a>, r#async: bool) {
        let field_val: u32 = r#async as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431751499775u128)
            | (((field_val as u128) & 255u128) << 16usize);
    }
    #[inline]
    pub fn set_this_param<'a>(self, ast: &mut crate::Ast<'a>, this_param: Option<TsThisParameter>) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Stmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_directives<'a>(self, ast: &mut crate::Ast<'a>, directives: SubRange<Directive>) {
        let field_val: u64 = directives.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: SubRange<Stmt>) {
        let field_val: u64 = body.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl BlockStatement {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Stmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: SubRange<Stmt>) {
        let field_val: u64 = body.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl WhileStatement {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u32 = test.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl DoWhileStatement {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Expr) {
        let field_val: u32 = test.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl WithStatement {
    #[inline]
    pub fn object<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Stmt {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Stmt(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_object<'a>(self, ast: &mut crate::Ast<'a>, object: Expr) {
        let field_val: u32 = object.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Stmt) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl FormalParameters {
    #[inline]
    pub fn items<'a>(self, ast: &crate::Ast<'a>) -> SubRange<FormalParameterItem> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn rest<'a>(self, ast: &crate::Ast<'a>) -> Option<FormalParameterRest> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| FormalParameterRest(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> FormalParameterKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, FormalParameterKind>(raw as u8) }
    }
    #[inline]
    pub fn set_items<'a>(self, ast: &mut crate::Ast<'a>, items: SubRange<FormalParameterItem>) {
        let field_val: u64 = items.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_rest<'a>(self, ast: &mut crate::Ast<'a>, rest: Option<FormalParameterRest>) {
        let field_val: u32 = rest.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: FormalParameterKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282346717757022325968521078060875775u128)
            | (((field_val as u128) & 255u128) << 96usize);
    }
}
impl FormalParameter {
    #[inline]
    pub fn pattern<'a>(self, ast: &crate::Ast<'a>) -> FormalParameterPattern {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        FormalParameterPattern(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Decorator> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn initializer<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_pattern<'a>(self, ast: &mut crate::Ast<'a>, pattern: FormalParameterPattern) {
        let field_val: u32 = pattern.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458741008125661634625535u128)
            | (((field_val as u128) & 4294967295u128) << 40usize);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_decorators<'a>(self, ast: &mut crate::Ast<'a>, decorators: SubRange<Decorator>) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> FormalParameterPattern {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        FormalParameterPattern(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_decorators<'a>(self, ast: &mut crate::Ast<'a>, decorators: SubRange<Decorator>) {
        let field_val: u64 = decorators.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: FormalParameterPattern) {
        let field_val: u32 = argument.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 79228162514264337593543950335u128)
            | (((field_val as u128) & 4294967295u128) << 96usize);
    }
}
impl ArrayExpression {
    #[inline]
    pub fn elements<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Option<Argument>> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_elements<'a>(self, ast: &mut crate::Ast<'a>, elements: SubRange<Option<Argument>>) {
        let field_val: u64 = elements.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl ObjectExpression {
    #[inline]
    pub fn properties<'a>(self, ast: &crate::Ast<'a>) -> SubRange<ObjectPropertyKind> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_properties<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        properties: SubRange<ObjectPropertyKind>,
    ) {
        let field_val: u64 = properties.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl SpreadElement {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: Expr) {
        let field_val: u32 = argument.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ObjectProperty {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        PropertyKey(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> PropertyKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, PropertyKind>(raw as u8) }
    }
    #[inline]
    pub fn method<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 72usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn shorthand<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 80usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 88usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropertyKey) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Expr) {
        let field_val: u32 = value.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: PropertyKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_method<'a>(self, ast: &mut crate::Ast<'a>, method: bool) {
        let field_val: u32 = method as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937259259921475672238718975u128)
            | (((field_val as u128) & 255u128) << 72usize);
    }
    #[inline]
    pub fn set_shorthand<'a>(self, ast: &mut crate::Ast<'a>, shorthand: bool) {
        let field_val: u32 = shorthand as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920630187379372876992218136575u128)
            | (((field_val as u128) & 255u128) << 80usize);
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        let field_val: u32 = computed as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366842019785958931614906949042175u128)
            | (((field_val as u128) & 255u128) << 88usize);
    }
}
impl ParenthesizedExpression {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ArrowFunctionExpression {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn r#async<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 16usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 48usize) as u32) & 4294967295u32;
        FormalParameters(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> ArrowFunctionBody {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: bool) {
        let field_val: u32 = expression as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_async<'a>(self, ast: &mut crate::Ast<'a>, r#async: bool) {
        let field_val: u32 = r#async as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768146175u128)
            | (((field_val as u128) & 255u128) << 8usize);
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374325956791566335u128)
            | (((field_val as u128) & 4294967295u128) << 16usize);
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        let field_val: u32 = params.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937254537555274277570215935u128)
            | (((field_val as u128) & 4294967295u128) << 48usize);
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        return_type: Option<TsTypeAnnotation>,
    ) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_expressions<'a>(self, ast: &mut crate::Ast<'a>, expressions: SubRange<Expr>) {
        let field_val: u64 = expressions.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl MemberExpression {
    #[inline]
    pub fn object<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn property<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        PropertyKey(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 72usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_object<'a>(self, ast: &mut crate::Ast<'a>, object: Expr) {
        let field_val: u32 = object.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_property<'a>(self, ast: &mut crate::Ast<'a>, property: PropertyKey) {
        let field_val: u32 = property.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        let field_val: u32 = computed as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937259259921475672238718975u128)
            | (((field_val as u128) & 255u128) << 72usize);
    }
}
impl CallExpression {
    #[inline]
    pub fn callee<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn arguments<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Argument> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a>, callee: Expr) {
        let field_val: u32 = callee.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458741008125661634625535u128)
            | (((field_val as u128) & 4294967295u128) << 40usize);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_arguments<'a>(self, ast: &mut crate::Ast<'a>, arguments: SubRange<Argument>) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl TaggedTemplateExpression {
    #[inline]
    pub fn tag<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn quasi<'a>(self, ast: &crate::Ast<'a>) -> TemplateLiteral {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        TemplateLiteral(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_tag<'a>(self, ast: &mut crate::Ast<'a>, tag: Expr) {
        let field_val: u32 = tag.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_quasi<'a>(self, ast: &mut crate::Ast<'a>, quasi: TemplateLiteral) {
        let field_val: u32 = quasi.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl NewExpression {
    #[inline]
    pub fn callee<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn arguments<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Argument> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a>, callee: Expr) {
        let field_val: u32 = callee.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 79228162514264337593543950335u128)
            | (((field_val as u128) & 4294967295u128) << 96usize);
    }
    #[inline]
    pub fn set_arguments<'a>(self, ast: &mut crate::Ast<'a>, arguments: SubRange<Argument>) {
        let field_val: u64 = arguments.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl AwaitExpression {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: Expr) {
        let field_val: u32 = argument.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl YieldExpression {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn delegate<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: Option<Expr>) {
        let field_val: u32 = argument.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_delegate<'a>(self, ast: &mut crate::Ast<'a>, delegate: bool) {
        let field_val: u32 = delegate as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606336551550975u128)
            | (((field_val as u128) & 255u128) << 32usize);
    }
}
impl MetaProperty {
    #[inline]
    pub fn meta<'a>(self, ast: &crate::Ast<'a>) -> IdentifierName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        IdentifierName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn property<'a>(self, ast: &crate::Ast<'a>) -> IdentifierName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        IdentifierName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_meta<'a>(self, ast: &mut crate::Ast<'a>, meta: IdentifierName) {
        let field_val: u32 = meta.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_property<'a>(self, ast: &mut crate::Ast<'a>, property: IdentifierName) {
        let field_val: u32 = property.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl ImportExpression {
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn options<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn sync<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn phase<'a>(self, ast: &crate::Ast<'a>) -> Option<ImportPhase> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a>, source: Expr) {
        let field_val: u32 = source.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_options<'a>(self, ast: &mut crate::Ast<'a>, options: Option<Expr>) {
        let field_val: u32 = options.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458741008125661634625535u128)
            | (((field_val as u128) & 4294967295u128) << 40usize);
    }
    #[inline]
    pub fn set_sync<'a>(self, ast: &mut crate::Ast<'a>, sync: bool) {
        let field_val: u32 = sync as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_phase<'a>(self, ast: &mut crate::Ast<'a>, phase: Option<ImportPhase>) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        StringLiteral(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn import_kind<'a>(self, ast: &crate::Ast<'a>) -> ImportOrExportKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, ImportOrExportKind>(raw as u8) }
    }
    #[inline]
    pub fn specifiers<'a>(self, ast: &crate::Ast<'a>) -> SubRange<ImportDeclarationSpecifier> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a>) -> SubRange<ImportAttribute> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn phase<'a>(self, ast: &crate::Ast<'a>) -> Option<ImportPhase> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a>, source: StringLiteral) {
        let field_val: u32 = source.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_import_kind<'a>(self, ast: &mut crate::Ast<'a>, import_kind: ImportOrExportKind) {
        let field_val: u32 = import_kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_specifiers<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        specifiers: SubRange<ImportDeclarationSpecifier>,
    ) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        ModuleExportName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        BindingIdentifier(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn import_kind<'a>(self, ast: &crate::Ast<'a>) -> ImportOrExportKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, ImportOrExportKind>(raw as u8) }
    }
    #[inline]
    pub fn set_imported<'a>(self, ast: &mut crate::Ast<'a>, imported: ModuleExportName) {
        let field_val: u32 = imported.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: BindingIdentifier) {
        let field_val: u32 = local.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_import_kind<'a>(self, ast: &mut crate::Ast<'a>, import_kind: ImportOrExportKind) {
        let field_val: u32 = import_kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
}
impl ImportDefaultSpecifier {
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        BindingIdentifier(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: BindingIdentifier) {
        let field_val: u32 = local.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ImportNamespaceSpecifier {
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        BindingIdentifier(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: BindingIdentifier) {
        let field_val: u32 = local.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ImportAttribute {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        ModuleExportName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> StringLiteral {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        StringLiteral(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: ModuleExportName) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: StringLiteral) {
        let field_val: u32 = value.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl ExportNamedDeclaration {
    #[inline]
    pub fn declaration<'a>(self, ast: &crate::Ast<'a>) -> Option<Decl> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Decl(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a>) -> Option<StringLiteral> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| StringLiteral(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn export_kind<'a>(self, ast: &crate::Ast<'a>) -> ImportOrExportKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, ImportOrExportKind>(raw as u8) }
    }
    #[inline]
    pub fn specifiers<'a>(self, ast: &crate::Ast<'a>) -> SubRange<ExportSpecifier> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a>) -> SubRange<ImportAttribute> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a>, source: Option<StringLiteral>) {
        let field_val: u32 = source.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458741008125661634625535u128)
            | (((field_val as u128) & 4294967295u128) << 40usize);
    }
    #[inline]
    pub fn set_export_kind<'a>(self, ast: &mut crate::Ast<'a>, export_kind: ImportOrExportKind) {
        let field_val: u32 = export_kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_specifiers<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        specifiers: SubRange<ExportSpecifier>,
    ) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        ExportDefaultDeclarationKind(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_declaration<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        declaration: ExportDefaultDeclarationKind,
    ) {
        let field_val: u32 = declaration.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ExportAllDeclaration {
    #[inline]
    pub fn exported<'a>(self, ast: &crate::Ast<'a>) -> Option<ModuleExportName> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| ModuleExportName(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a>) -> StringLiteral {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        StringLiteral(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn export_kind<'a>(self, ast: &crate::Ast<'a>) -> ImportOrExportKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, ImportOrExportKind>(raw as u8) }
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a>) -> SubRange<ImportAttribute> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a>, source: StringLiteral) {
        let field_val: u32 = source.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458741008125661634625535u128)
            | (((field_val as u128) & 4294967295u128) << 40usize);
    }
    #[inline]
    pub fn set_export_kind<'a>(self, ast: &mut crate::Ast<'a>, export_kind: ImportOrExportKind) {
        let field_val: u32 = export_kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_attributes<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        attributes: SubRange<ImportAttribute>,
    ) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        ModuleExportName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn exported<'a>(self, ast: &crate::Ast<'a>) -> ModuleExportName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        ModuleExportName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn export_kind<'a>(self, ast: &crate::Ast<'a>) -> ImportOrExportKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, ImportOrExportKind>(raw as u8) }
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a>, local: ModuleExportName) {
        let field_val: u32 = local.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_exported<'a>(self, ast: &mut crate::Ast<'a>, exported: ModuleExportName) {
        let field_val: u32 = exported.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_export_kind<'a>(self, ast: &mut crate::Ast<'a>, export_kind: ImportOrExportKind) {
        let field_val: u32 = export_kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
}
impl Hashbang {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: AstUtf8) {
        let field_val: u64 = value.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 24usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| BindingIdentifier(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 56usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn generator<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn r#async<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 16usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a>) -> Option<TsThisParameter> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463302549837747060735u128)
            | (((field_val as u128) & 4294967295u128) << 24usize);
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920628978453625319957081358335u128)
            | (((field_val as u128) & 4294967295u128) << 56usize);
    }
    #[inline]
    pub fn set_generator<'a>(self, ast: &mut crate::Ast<'a>, generator: bool) {
        let field_val: u32 = generator as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_async<'a>(self, ast: &mut crate::Ast<'a>, r#async: bool) {
        let field_val: u32 = r#async as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768146175u128)
            | (((field_val as u128) & 255u128) << 8usize);
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431751499775u128)
            | (((field_val as u128) & 255u128) << 16usize);
    }
    #[inline]
    pub fn set_this_param<'a>(self, ast: &mut crate::Ast<'a>, this_param: Option<TsThisParameter>) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 56usize) as u32) & 4294967295u32;
        PropertyKey(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> MethodDefinitionKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, MethodDefinitionKind>(raw as u8) }
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn r#static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 16usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn r#override<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 24usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn accessibility<'a>(self, ast: &crate::Ast<'a>) -> Accessibility {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 48usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, Accessibility>(raw as u8) }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Decorator> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> TsFunction {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropertyKey) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920628978453625319957081358335u128)
            | (((field_val as u128) & 4294967295u128) << 56usize);
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: MethodDefinitionKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        let field_val: u32 = computed as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768146175u128)
            | (((field_val as u128) & 255u128) << 8usize);
    }
    #[inline]
    pub fn set_static<'a>(self, ast: &mut crate::Ast<'a>, r#static: bool) {
        let field_val: u32 = r#static as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431751499775u128)
            | (((field_val as u128) & 255u128) << 16usize);
    }
    #[inline]
    pub fn set_override<'a>(self, ast: &mut crate::Ast<'a>, r#override: bool) {
        let field_val: u32 = r#override as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427490021375u128)
            | (((field_val as u128) & 255u128) << 24usize);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606336551550975u128)
            | (((field_val as u128) & 255u128) << 32usize);
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a>, r#abstract: bool) {
        let field_val: u32 = r#abstract as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374327056303128575u128)
            | (((field_val as u128) & 255u128) << 40usize);
    }
    #[inline]
    pub fn set_accessibility<'a>(self, ast: &mut crate::Ast<'a>, accessibility: Accessibility) {
        let field_val: u32 = accessibility as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463302831312706994175u128)
            | (((field_val as u128) & 255u128) << 48usize);
    }
    #[inline]
    pub fn set_decorators<'a>(self, ast: &mut crate::Ast<'a>, decorators: SubRange<Decorator>) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsTypeName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_type_name<'a>(self, ast: &mut crate::Ast<'a>, type_name: TsTypeName) {
        let field_val: u32 = type_name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl TsQualifiedName {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> TsQualifiedNameLeft {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsQualifiedNameLeft(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> IdentifierName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        IdentifierName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: TsQualifiedNameLeft) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: IdentifierName) {
        let field_val: u32 = right.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl TsTypeQuery {
    #[inline]
    pub fn expr_name<'a>(self, ast: &crate::Ast<'a>) -> TsTypeQueryExprName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsTypeQueryExprName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_expr_name<'a>(self, ast: &mut crate::Ast<'a>, expr_name: TsTypeQueryExprName) {
        let field_val: u32 = expr_name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl TsImportType {
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a>) -> StringLiteral {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        StringLiteral(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn options<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn qualifier<'a>(self, ast: &crate::Ast<'a>) -> Option<TsImportTypeQualifier> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsImportTypeQualifier(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a>, source: StringLiteral) {
        let field_val: u32 = source.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_options<'a>(self, ast: &mut crate::Ast<'a>, options: Option<Expr>) {
        let field_val: u32 = options.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_qualifier<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        qualifier: Option<TsImportTypeQualifier>,
    ) {
        let field_val: u32 = qualifier.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 79228162514264337593543950335u128)
            | (((field_val as u128) & 4294967295u128) << 96usize);
    }
}
impl TsTypeParameter {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        BindingIdentifier(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn constraint<'a>(self, ast: &crate::Ast<'a>) -> Option<TsType> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsType(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn default<'a>(self, ast: &crate::Ast<'a>) -> Option<TsType> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsType(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn r#in<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn out<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 104usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn r#const<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 112usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: BindingIdentifier) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_constraint<'a>(self, ast: &mut crate::Ast<'a>, constraint: Option<TsType>) {
        let field_val: u32 = constraint.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_default<'a>(self, ast: &mut crate::Ast<'a>, default: Option<TsType>) {
        let field_val: u32 = default.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_in<'a>(self, ast: &mut crate::Ast<'a>, r#in: bool) {
        let field_val: u32 = r#in as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282346717757022325968521078060875775u128)
            | (((field_val as u128) & 255u128) << 96usize);
    }
    #[inline]
    pub fn set_out<'a>(self, ast: &mut crate::Ast<'a>, out: bool) {
        let field_val: u32 = out as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340277194906489532287416500882690277375u128)
            | (((field_val as u128) & 255u128) << 104usize);
    }
    #[inline]
    pub fn set_const<'a>(self, ast: &mut crate::Ast<'a>, r#const: bool) {
        let field_val: u32 = r#const as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 338958331222012082418099330867817086975u128)
            | (((field_val as u128) & 255u128) << 112usize);
    }
}
impl TsTypeParameterDeclaration {
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsTypeParameter> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: SubRange<TsTypeParameter>) {
        let field_val: u64 = params.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl TsTypeParameterInstantiation {
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsType> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: SubRange<TsType>) {
        let field_val: u64 = params.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl TsLiteralType {
    #[inline]
    pub fn literal<'a>(self, ast: &crate::Ast<'a>) -> TsLiteral {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsLiteral(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_literal<'a>(self, ast: &mut crate::Ast<'a>, literal: TsLiteral) {
        let field_val: u32 = literal.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl TsTemplateLiteralType {
    #[inline]
    pub fn quasis<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TemplateElement> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn types<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsType> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_quasis<'a>(self, ast: &mut crate::Ast<'a>, quasis: SubRange<TemplateElement>) {
        let field_val: u64 = quasis.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_types<'a>(self, ast: &mut crate::Ast<'a>, types: SubRange<TsType>) {
        let field_val: u64 = types.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl TsArrayType {
    #[inline]
    pub fn element_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_element_type<'a>(self, ast: &mut crate::Ast<'a>, element_type: TsType) {
        let field_val: u32 = element_type.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl TsIndexedAccessType {
    #[inline]
    pub fn object_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn index_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_object_type<'a>(self, ast: &mut crate::Ast<'a>, object_type: TsType) {
        let field_val: u32 = object_type.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_index_type<'a>(self, ast: &mut crate::Ast<'a>, index_type: TsType) {
        let field_val: u32 = index_type.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl TsTupleType {
    #[inline]
    pub fn element_types<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsTupleElement> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_element_types<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        element_types: SubRange<TsTupleElement>,
    ) {
        let field_val: u64 = element_types.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl TsNamedTupleMember {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a>) -> IdentifierName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        IdentifierName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn element_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a>, label: IdentifierName) {
        let field_val: u32 = label.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_element_type<'a>(self, ast: &mut crate::Ast<'a>, element_type: TsType) {
        let field_val: u32 = element_type.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
}
impl TsOptionalType {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl TsRestType {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl TsJsDocNullableType {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn postfix<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_postfix<'a>(self, ast: &mut crate::Ast<'a>, postfix: bool) {
        let field_val: u32 = postfix as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606336551550975u128)
            | (((field_val as u128) & 255u128) << 32usize);
    }
}
impl TsJsDocNonNullableType {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn postfix<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_postfix<'a>(self, ast: &mut crate::Ast<'a>, postfix: bool) {
        let field_val: u32 = postfix as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606336551550975u128)
            | (((field_val as u128) & 255u128) << 32usize);
    }
}
impl TsJsDocUnknownType {}
impl TsUnionType {
    #[inline]
    pub fn types<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsType> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_types<'a>(self, ast: &mut crate::Ast<'a>, types: SubRange<TsType>) {
        let field_val: u64 = types.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl TsIntersectionType {
    #[inline]
    pub fn types<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsType> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_types<'a>(self, ast: &mut crate::Ast<'a>, types: SubRange<TsType>) {
        let field_val: u64 = types.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl TsConditionalType {
    #[inline]
    pub fn check_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn extends_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn true_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn false_type<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_check_type<'a>(self, ast: &mut crate::Ast<'a>, check_type: TsType) {
        let field_val: u32 = check_type.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_extends_type<'a>(self, ast: &mut crate::Ast<'a>, extends_type: TsType) {
        let field_val: u32 = extends_type.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_true_type<'a>(self, ast: &mut crate::Ast<'a>, true_type: TsType) {
        let field_val: u32 = true_type.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_false_type<'a>(self, ast: &mut crate::Ast<'a>, false_type: TsType) {
        let field_val: u32 = false_type.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 79228162514264337593543950335u128)
            | (((field_val as u128) & 4294967295u128) << 96usize);
    }
}
impl TsInferType {
    #[inline]
    pub fn type_parameter<'a>(self, ast: &crate::Ast<'a>) -> TsTypeParameter {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsTypeParameter(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_type_parameter<'a>(self, ast: &mut crate::Ast<'a>, type_parameter: TsTypeParameter) {
        let field_val: u32 = type_parameter.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl TsTypeOperator {
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a>) -> TsTypeOperatorKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, TsTypeOperatorKind>(raw as u8) }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a>, operator: TsTypeOperatorKind) {
        let field_val: u32 = operator as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606336551550975u128)
            | (((field_val as u128) & 255u128) << 32usize);
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl TsParenthesizedType {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl TsFunctionType {
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a>) -> Option<TsThisParameter> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsThisParameter(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        FormalParameters(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> TsTypeAnnotation {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 4294967295u32;
        TsTypeAnnotation(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_this_param<'a>(self, ast: &mut crate::Ast<'a>, this_param: Option<TsThisParameter>) {
        let field_val: u32 = this_param.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        let field_val: u32 = params.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_return_type<'a>(self, ast: &mut crate::Ast<'a>, return_type: TsTypeAnnotation) {
        let field_val: u32 = return_type.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 79228162514264337593543950335u128)
            | (((field_val as u128) & 4294967295u128) << 96usize);
    }
}
impl TsConstructorType {
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        FormalParameters(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> TsTypeAnnotation {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        TsTypeAnnotation(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        let field_val: u32 = params.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_return_type<'a>(self, ast: &mut crate::Ast<'a>, return_type: TsTypeAnnotation) {
        let field_val: u32 = return_type.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a>, r#abstract: bool) {
        let field_val: u32 = r#abstract as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282346717757022325968521078060875775u128)
            | (((field_val as u128) & 255u128) << 96usize);
    }
}
impl TsTypePredicate {
    #[inline]
    pub fn parameter_name<'a>(self, ast: &crate::Ast<'a>) -> TsTypePredicateName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsTypePredicateName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn asserts<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_parameter_name<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        parameter_name: TsTypePredicateName,
    ) {
        let field_val: u32 = parameter_name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_asserts<'a>(self, ast: &mut crate::Ast<'a>, asserts: bool) {
        let field_val: u32 = asserts as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
}
impl TsTypeLiteral {
    #[inline]
    pub fn members<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsTypeElement> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_members<'a>(self, ast: &mut crate::Ast<'a>, members: SubRange<TsTypeElement>) {
        let field_val: u64 = members.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl TsMappedType {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 16usize) as u32) & 4294967295u32;
        BindingIdentifier(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn constraint<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 48usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> TsMappedTypeModifier {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, TsMappedTypeModifier>(raw as u8) }
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a>) -> TsMappedTypeModifier {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, TsMappedTypeModifier>(raw as u8) }
    }
    #[inline]
    pub fn name_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsType> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsType> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: BindingIdentifier) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374325956791566335u128)
            | (((field_val as u128) & 4294967295u128) << 16usize);
    }
    #[inline]
    pub fn set_constraint<'a>(self, ast: &mut crate::Ast<'a>, constraint: TsType) {
        let field_val: u32 = constraint.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937254537555274277570215935u128)
            | (((field_val as u128) & 4294967295u128) << 48usize);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: TsMappedTypeModifier) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a>, readonly: TsMappedTypeModifier) {
        let field_val: u32 = readonly as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768146175u128)
            | (((field_val as u128) & 255u128) << 8usize);
    }
    #[inline]
    pub fn set_name_type<'a>(self, ast: &mut crate::Ast<'a>, name_type: Option<TsType>) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        PropertyKey(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 72usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 80usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropertyKey) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        let field_val: u32 = computed as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937259259921475672238718975u128)
            | (((field_val as u128) & 255u128) << 72usize);
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a>, readonly: bool) {
        let field_val: u32 = readonly as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920630187379372876992218136575u128)
            | (((field_val as u128) & 255u128) << 80usize);
    }
}
impl TsMethodSignature {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 24usize) as u32) & 4294967295u32;
        PropertyKey(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 56usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> TsMethodSignatureKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, TsMethodSignatureKind>(raw as u8) }
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 16usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a>) -> Option<TsThisParameter> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropertyKey) {
        let field_val: u32 = key.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463302549837747060735u128)
            | (((field_val as u128) & 4294967295u128) << 24usize);
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920628978453625319957081358335u128)
            | (((field_val as u128) & 4294967295u128) << 56usize);
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: TsMethodSignatureKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        let field_val: u32 = computed as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768146175u128)
            | (((field_val as u128) & 255u128) << 8usize);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431751499775u128)
            | (((field_val as u128) & 255u128) << 16usize);
    }
    #[inline]
    pub fn set_this_param<'a>(self, ast: &mut crate::Ast<'a>, this_param: Option<TsThisParameter>) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a>) -> Option<TsThisParameter> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsThisParameter(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        FormalParameters(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_this_param<'a>(self, ast: &mut crate::Ast<'a>, this_param: Option<TsThisParameter>) {
        let field_val: u32 = this_param.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        let field_val: u32 = params.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        return_type: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = return_type.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 79228162514264337593543950335u128)
            | (((field_val as u128) & 4294967295u128) << 96usize);
    }
}
impl TsConstructSignatureDeclaration {
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        FormalParameters(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        let field_val: u32 = params.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        return_type: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = return_type.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
}
impl TsIndexSignature {
    #[inline]
    pub fn parameters<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsIndexSignatureParameter> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsTypeAnnotation {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        TsTypeAnnotation(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn r#static<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 104usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        parameters: SubRange<TsIndexSignatureParameter>,
    ) {
        let field_val: u64 = parameters.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: TsTypeAnnotation,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a>, readonly: bool) {
        let field_val: u32 = readonly as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282346717757022325968521078060875775u128)
            | (((field_val as u128) & 255u128) << 96usize);
    }
    #[inline]
    pub fn set_static<'a>(self, ast: &mut crate::Ast<'a>, r#static: bool) {
        let field_val: u32 = r#static as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340277194906489532287416500882690277375u128)
            | (((field_val as u128) & 255u128) << 104usize);
    }
}
impl TsTypeAliasDeclaration {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        BindingIdentifier(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: BindingIdentifier) {
        let field_val: u32 = id.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282346717757022325968521078060875775u128)
            | (((field_val as u128) & 255u128) << 96usize);
    }
}
impl TsInterfaceDeclaration {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        BindingIdentifier(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn extends<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsInterfaceHeritage> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TsInterfaceBody {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: BindingIdentifier) {
        let field_val: u32 = id.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458741008125661634625535u128)
            | (((field_val as u128) & 4294967295u128) << 40usize);
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_extends<'a>(self, ast: &mut crate::Ast<'a>, extends: SubRange<TsInterfaceHeritage>) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: SubRange<TsTypeElement>) {
        let field_val: u64 = body.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl TsInterfaceHeritage {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl TsClassImplements {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl TsEnumDeclaration {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        BindingIdentifier(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TsEnumBody {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        TsEnumBody(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn is_const<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 72usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: BindingIdentifier) {
        let field_val: u32 = id.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TsEnumBody) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_is_const<'a>(self, ast: &mut crate::Ast<'a>, is_const: bool) {
        let field_val: u32 = is_const as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937259259921475672238718975u128)
            | (((field_val as u128) & 255u128) << 72usize);
    }
}
impl TsEnumBody {
    #[inline]
    pub fn members<'a>(self, ast: &crate::Ast<'a>) -> SubRange<TsEnumMember> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_members<'a>(self, ast: &mut crate::Ast<'a>, members: SubRange<TsEnumMember>) {
        let field_val: u64 = members.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl TsEnumMember {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> TsEnumMemberName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsEnumMemberName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn initializer<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: TsEnumMemberName) {
        let field_val: u32 = id.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_initializer<'a>(self, ast: &mut crate::Ast<'a>, initializer: Option<Expr>) {
        let field_val: u32 = initializer.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a>, computed: bool) {
        let field_val: u32 = computed as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
}
impl TsModuleDeclaration {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> TsModuleName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsModuleName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> Option<TsModuleBody> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsModuleBody(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> TsModuleDeclarationKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, TsModuleDeclarationKind>(raw as u8) }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 72usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: TsModuleName) {
        let field_val: u32 = id.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: Option<TsModuleBody>) {
        let field_val: u32 = body.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: TsModuleDeclarationKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937259259921475672238718975u128)
            | (((field_val as u128) & 255u128) << 72usize);
    }
}
impl TsModuleBlock {
    #[inline]
    pub fn directives<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Directive> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Stmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_directives<'a>(self, ast: &mut crate::Ast<'a>, directives: SubRange<Directive>) {
        let field_val: u64 = directives.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: SubRange<Stmt>) {
        let field_val: u64 = body.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl TsGlobalDeclaration {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> IdentifierName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        IdentifierName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TsModuleBlock {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        TsModuleBlock(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: IdentifierName) {
        let field_val: u32 = id.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TsModuleBlock) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a>, declare: bool) {
        let field_val: u32 = declare as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
}
impl TsParameterProperty {
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> SubRange<Decorator> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn parameter<'a>(self, ast: &crate::Ast<'a>) -> FormalParameter {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        FormalParameter(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn r#override<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 104usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn accessibility<'a>(self, ast: &crate::Ast<'a>) -> Accessibility {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 112usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, Accessibility>(raw as u8) }
    }
    #[inline]
    pub fn set_decorators<'a>(self, ast: &mut crate::Ast<'a>, decorators: SubRange<Decorator>) {
        let field_val: u64 = decorators.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_parameter<'a>(self, ast: &mut crate::Ast<'a>, parameter: FormalParameter) {
        let field_val: u32 = parameter.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_override<'a>(self, ast: &mut crate::Ast<'a>, r#override: bool) {
        let field_val: u32 = r#override as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282346717757022325968521078060875775u128)
            | (((field_val as u128) & 255u128) << 96usize);
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a>, readonly: bool) {
        let field_val: u32 = readonly as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340277194906489532287416500882690277375u128)
            | (((field_val as u128) & 255u128) << 104usize);
    }
    #[inline]
    pub fn set_accessibility<'a>(self, ast: &mut crate::Ast<'a>, accessibility: Accessibility) {
        let field_val: u32 = accessibility as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 338958331222012082418099330867817086975u128)
            | (((field_val as u128) & 255u128) << 112usize);
    }
}
impl TsThisParameter {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl TsAsExpression {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl TsSatisfiesExpression {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl TsTypeAssertion {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsType {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        TsType(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_type_annotation<'a>(self, ast: &mut crate::Ast<'a>, type_annotation: TsType) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl TsNonNullExpression {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl TsInstantiationExpression {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> TsTypeParameterInstantiation {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: TsTypeParameterInstantiation,
    ) {
        let field_val: u32 = type_arguments.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl TsExportAssignment {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl TsNamespaceExportDeclaration {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> IdentifierName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        IdentifierName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: IdentifierName) {
        let field_val: u32 = id.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl TsImportEqualsDeclaration {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        BindingIdentifier(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn module_reference<'a>(self, ast: &crate::Ast<'a>) -> TsModuleReference {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        TsModuleReference(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn import_kind<'a>(self, ast: &crate::Ast<'a>) -> ImportOrExportKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, ImportOrExportKind>(raw as u8) }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a>, id: BindingIdentifier) {
        let field_val: u32 = id.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_module_reference<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        module_reference: TsModuleReference,
    ) {
        let field_val: u32 = module_reference.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_import_kind<'a>(self, ast: &mut crate::Ast<'a>, import_kind: ImportOrExportKind) {
        let field_val: u32 = import_kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
}
impl TsExternalModuleReference {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> StringLiteral {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        StringLiteral(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: StringLiteral) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl TsIndexSignatureParameter {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> BindingIdentifier {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        BindingIdentifier(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> TsTypeAnnotation {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        TsTypeAnnotation(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: BindingIdentifier) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: TsTypeAnnotation,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        JsxOpeningElement(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn children<'a>(self, ast: &crate::Ast<'a>) -> SubRange<JsxChild> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn closing_element<'a>(self, ast: &crate::Ast<'a>) -> Option<JsxClosingElement> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| JsxClosingElement(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_opening_element<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        opening_element: JsxOpeningElement,
    ) {
        let field_val: u32 = opening_element.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_children<'a>(self, ast: &mut crate::Ast<'a>, children: SubRange<JsxChild>) {
        let field_val: u64 = children.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_closing_element<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        closing_element: Option<JsxClosingElement>,
    ) {
        let field_val: u32 = closing_element.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 79228162514264337593543950335u128)
            | (((field_val as u128) & 4294967295u128) << 96usize);
    }
}
impl JsxOpeningElement {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JsxElementName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        JsxElementName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn self_closing<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a>) -> SubRange<JsxAttributeItem> {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JsxElementName) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458741008125661634625535u128)
            | (((field_val as u128) & 4294967295u128) << 40usize);
    }
    #[inline]
    pub fn set_self_closing<'a>(self, ast: &mut crate::Ast<'a>, self_closing: bool) {
        let field_val: u32 = self_closing as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_attributes<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        attributes: SubRange<JsxAttributeItem>,
    ) {
        let start = unsafe { ast.nodes.data_unchecked(self.0).0 >> 96 } as u32 as usize;
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
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        JsxElementName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JsxElementName) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl JsxFragment {
    #[inline]
    pub fn opening_fragment<'a>(self, ast: &crate::Ast<'a>) -> JsxOpeningFragment {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        JsxOpeningFragment(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn children<'a>(self, ast: &crate::Ast<'a>) -> SubRange<JsxChild> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn closing_fragment<'a>(self, ast: &crate::Ast<'a>) -> JsxClosingFragment {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 4294967295u32;
        JsxClosingFragment(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_opening_fragment<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        opening_fragment: JsxOpeningFragment,
    ) {
        let field_val: u32 = opening_fragment.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_children<'a>(self, ast: &mut crate::Ast<'a>, children: SubRange<JsxChild>) {
        let field_val: u64 = children.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_closing_fragment<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        closing_fragment: JsxClosingFragment,
    ) {
        let field_val: u32 = closing_fragment.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 79228162514264337593543950335u128)
            | (((field_val as u128) & 4294967295u128) << 96usize);
    }
}
impl JsxOpeningFragment {}
impl JsxClosingFragment {}
impl JsxIdentifier {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        let field_val: u64 = name.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl JsxNamespacedName {
    #[inline]
    pub fn namespace<'a>(self, ast: &crate::Ast<'a>) -> JsxIdentifier {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        JsxIdentifier(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JsxIdentifier {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        JsxIdentifier(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_namespace<'a>(self, ast: &mut crate::Ast<'a>, namespace: JsxIdentifier) {
        let field_val: u32 = namespace.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JsxIdentifier) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl JsxMemberExpression {
    #[inline]
    pub fn object<'a>(self, ast: &crate::Ast<'a>) -> JsxMemberExpressionObject {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        JsxMemberExpressionObject(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn property<'a>(self, ast: &crate::Ast<'a>) -> JsxIdentifier {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        JsxIdentifier(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_object<'a>(self, ast: &mut crate::Ast<'a>, object: JsxMemberExpressionObject) {
        let field_val: u32 = object.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_property<'a>(self, ast: &mut crate::Ast<'a>, property: JsxIdentifier) {
        let field_val: u32 = property.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl JsxAttribute {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> JsxAttributeName {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        JsxAttributeName(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<JsxAttributeValue> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| JsxAttributeValue(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: JsxAttributeName) {
        let field_val: u32 = name.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<JsxAttributeValue>) {
        let field_val: u32 = value.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
}
impl JsxSpreadAttribute {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: Expr) {
        let field_val: u32 = argument.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl JsxExpressionContainer {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> JsxExpression {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        JsxExpression(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: JsxExpression) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl JsxEmptyExpression {}
impl JsxText {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataValue::from_extra_data(&ExtraData::from_raw(raw)) }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: AstUtf8) {
        let field_val: u64 = value.into_extra_data().raw();
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl JsxSpreadChild {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a>, expression: Expr) {
        let field_val: u32 = expression.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
