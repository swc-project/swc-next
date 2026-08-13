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
    pub fn directives<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Directive> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn hashbang<'a>(self, ast: &crate::Ast<'a>) -> Option<Hashbang> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Hashbang(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Stmt> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_directives<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        directives: TypedSubRange<Directive>,
    ) {
        let field_val: u64 = unsafe { directives.to_extra_data(ast).other };
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
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TypedSubRange<Stmt>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.to_extra_data(ast);
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
        match ast.stmt_data(self) {
            StmtData::BlockStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_break_statement(self, ast: &Ast<'_>) -> Option<BreakStatement> {
        match ast.stmt_data(self) {
            StmtData::BreakStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_continue_statement(self, ast: &Ast<'_>) -> Option<ContinueStatement> {
        match ast.stmt_data(self) {
            StmtData::ContinueStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_debugger_statement(self, ast: &Ast<'_>) -> Option<DebuggerStatement> {
        match ast.stmt_data(self) {
            StmtData::DebuggerStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_do_while_statement(self, ast: &Ast<'_>) -> Option<DoWhileStatement> {
        match ast.stmt_data(self) {
            StmtData::DoWhileStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_empty_statement(self, ast: &Ast<'_>) -> Option<EmptyStatement> {
        match ast.stmt_data(self) {
            StmtData::EmptyStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expression_statement(self, ast: &Ast<'_>) -> Option<ExpressionStatement> {
        match ast.stmt_data(self) {
            StmtData::ExpressionStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for_in_statement(self, ast: &Ast<'_>) -> Option<ForInStatement> {
        match ast.stmt_data(self) {
            StmtData::ForInStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for_of_statement(self, ast: &Ast<'_>) -> Option<ForOfStatement> {
        match ast.stmt_data(self) {
            StmtData::ForOfStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_for_statement(self, ast: &Ast<'_>) -> Option<ForStatement> {
        match ast.stmt_data(self) {
            StmtData::ForStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_if_statement(self, ast: &Ast<'_>) -> Option<IfStatement> {
        match ast.stmt_data(self) {
            StmtData::IfStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_labeled_statement(self, ast: &Ast<'_>) -> Option<LabeledStatement> {
        match ast.stmt_data(self) {
            StmtData::LabeledStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_return_statement(self, ast: &Ast<'_>) -> Option<ReturnStatement> {
        match ast.stmt_data(self) {
            StmtData::ReturnStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_switch_statement(self, ast: &Ast<'_>) -> Option<SwitchStatement> {
        match ast.stmt_data(self) {
            StmtData::SwitchStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_throw_statement(self, ast: &Ast<'_>) -> Option<ThrowStatement> {
        match ast.stmt_data(self) {
            StmtData::ThrowStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_try_statement(self, ast: &Ast<'_>) -> Option<TryStatement> {
        match ast.stmt_data(self) {
            StmtData::TryStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_while_statement(self, ast: &Ast<'_>) -> Option<WhileStatement> {
        match ast.stmt_data(self) {
            StmtData::WhileStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_with_statement(self, ast: &Ast<'_>) -> Option<WithStatement> {
        match ast.stmt_data(self) {
            StmtData::WithStatement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_declaration(self, ast: &Ast<'_>) -> Option<Decl> {
        match ast.stmt_data(self) {
            StmtData::Declaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_import_declaration(self, ast: &Ast<'_>) -> Option<ImportDeclaration> {
        match ast.stmt_data(self) {
            StmtData::ImportDeclaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_named_declaration(self, ast: &Ast<'_>) -> Option<ExportNamedDeclaration> {
        match ast.stmt_data(self) {
            StmtData::ExportNamedDeclaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_default_declaration(self, ast: &Ast<'_>) -> Option<ExportDefaultDeclaration> {
        match ast.stmt_data(self) {
            StmtData::ExportDefaultDeclaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_export_all_declaration(self, ast: &Ast<'_>) -> Option<ExportAllDeclaration> {
        match ast.stmt_data(self) {
            StmtData::ExportAllDeclaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_export_assignment(self, ast: &Ast<'_>) -> Option<TsExportAssignment> {
        match ast.stmt_data(self) {
            StmtData::TsExportAssignment(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_namespace_export_declaration(
        self,
        ast: &Ast<'_>,
    ) -> Option<TsNamespaceExportDeclaration> {
        match ast.stmt_data(self) {
            StmtData::TsNamespaceExportDeclaration(it) => Some(it),
            _ => None,
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
        match ast.decl_data(self) {
            DeclData::VariableDeclaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_function(self, ast: &Ast<'_>) -> Option<Function> {
        match ast.decl_data(self) {
            DeclData::Function(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class(self, ast: &Ast<'_>) -> Option<Class> {
        match ast.decl_data(self) {
            DeclData::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_function(self, ast: &Ast<'_>) -> Option<TsFunction> {
        match ast.decl_data(self) {
            DeclData::TsFunction(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_type_alias_declaration(self, ast: &Ast<'_>) -> Option<TsTypeAliasDeclaration> {
        match ast.decl_data(self) {
            DeclData::TsTypeAliasDeclaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_interface_declaration(self, ast: &Ast<'_>) -> Option<TsInterfaceDeclaration> {
        match ast.decl_data(self) {
            DeclData::TsInterfaceDeclaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_enum_declaration(self, ast: &Ast<'_>) -> Option<TsEnumDeclaration> {
        match ast.decl_data(self) {
            DeclData::TsEnumDeclaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_module_declaration(self, ast: &Ast<'_>) -> Option<TsModuleDeclaration> {
        match ast.decl_data(self) {
            DeclData::TsModuleDeclaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_global_declaration(self, ast: &Ast<'_>) -> Option<TsGlobalDeclaration> {
        match ast.decl_data(self) {
            DeclData::TsGlobalDeclaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_import_equals_declaration(
        self,
        ast: &Ast<'_>,
    ) -> Option<TsImportEqualsDeclaration> {
        match ast.decl_data(self) {
            DeclData::TsImportEqualsDeclaration(it) => Some(it),
            _ => None,
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
        match ast.expr_data(self) {
            ExprData::IdentifierReference(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_this_expression(self, ast: &Ast<'_>) -> Option<ThisExpression> {
        match ast.expr_data(self) {
            ExprData::ThisExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_super(self, ast: &Ast<'_>) -> Option<Super> {
        match ast.expr_data(self) {
            ExprData::Super(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_meta_property(self, ast: &Ast<'_>) -> Option<MetaProperty> {
        match ast.expr_data(self) {
            ExprData::MetaProperty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_>) -> Option<StringLiteral> {
        match ast.expr_data(self) {
            ExprData::StringLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_numeric_literal(self, ast: &Ast<'_>) -> Option<NumericLiteral> {
        match ast.expr_data(self) {
            ExprData::NumericLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int_literal(self, ast: &Ast<'_>) -> Option<BigIntLiteral> {
        match ast.expr_data(self) {
            ExprData::BigIntLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_boolean_literal(self, ast: &Ast<'_>) -> Option<BooleanLiteral> {
        match ast.expr_data(self) {
            ExprData::BooleanLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_null_literal(self, ast: &Ast<'_>) -> Option<NullLiteral> {
        match ast.expr_data(self) {
            ExprData::NullLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_reg_exp_literal(self, ast: &Ast<'_>) -> Option<RegExpLiteral> {
        match ast.expr_data(self) {
            ExprData::RegExpLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_template_literal(self, ast: &Ast<'_>) -> Option<TemplateLiteral> {
        match ast.expr_data(self) {
            ExprData::TemplateLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_binary_expression(self, ast: &Ast<'_>) -> Option<BinaryExpression> {
        match ast.expr_data(self) {
            ExprData::BinaryExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_logical_expression(self, ast: &Ast<'_>) -> Option<LogicalExpression> {
        match ast.expr_data(self) {
            ExprData::LogicalExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_unary_expression(self, ast: &Ast<'_>) -> Option<UnaryExpression> {
        match ast.expr_data(self) {
            ExprData::UnaryExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_update_expression(self, ast: &Ast<'_>) -> Option<UpdateExpression> {
        match ast.expr_data(self) {
            ExprData::UpdateExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assignment_expression(self, ast: &Ast<'_>) -> Option<AssignmentExpression> {
        match ast.expr_data(self) {
            ExprData::AssignmentExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_conditional_expression(self, ast: &Ast<'_>) -> Option<ConditionalExpression> {
        match ast.expr_data(self) {
            ExprData::ConditionalExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_sequence_expression(self, ast: &Ast<'_>) -> Option<SequenceExpression> {
        match ast.expr_data(self) {
            ExprData::SequenceExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_member_expression(self, ast: &Ast<'_>) -> Option<MemberExpression> {
        match ast.expr_data(self) {
            ExprData::MemberExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_call_expression(self, ast: &Ast<'_>) -> Option<CallExpression> {
        match ast.expr_data(self) {
            ExprData::CallExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_chain_expression(self, ast: &Ast<'_>) -> Option<ChainExpression> {
        match ast.expr_data(self) {
            ExprData::ChainExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_new_expression(self, ast: &Ast<'_>) -> Option<NewExpression> {
        match ast.expr_data(self) {
            ExprData::NewExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_tagged_template_expression(self, ast: &Ast<'_>) -> Option<TaggedTemplateExpression> {
        match ast.expr_data(self) {
            ExprData::TaggedTemplateExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_arrow_function_expression(self, ast: &Ast<'_>) -> Option<ArrowFunctionExpression> {
        match ast.expr_data(self) {
            ExprData::ArrowFunctionExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array_expression(self, ast: &Ast<'_>) -> Option<ArrayExpression> {
        match ast.expr_data(self) {
            ExprData::ArrayExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object_expression(self, ast: &Ast<'_>) -> Option<ObjectExpression> {
        match ast.expr_data(self) {
            ExprData::ObjectExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_parenthesized_expression(self, ast: &Ast<'_>) -> Option<ParenthesizedExpression> {
        match ast.expr_data(self) {
            ExprData::ParenthesizedExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_function(self, ast: &Ast<'_>) -> Option<Function> {
        match ast.expr_data(self) {
            ExprData::Function(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class(self, ast: &Ast<'_>) -> Option<Class> {
        match ast.expr_data(self) {
            ExprData::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_import_expression(self, ast: &Ast<'_>) -> Option<ImportExpression> {
        match ast.expr_data(self) {
            ExprData::ImportExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_await_expression(self, ast: &Ast<'_>) -> Option<AwaitExpression> {
        match ast.expr_data(self) {
            ExprData::AwaitExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_yield_expression(self, ast: &Ast<'_>) -> Option<YieldExpression> {
        match ast.expr_data(self) {
            ExprData::YieldExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_identifier(self, ast: &Ast<'_>) -> Option<PrivateIdentifier> {
        match ast.expr_data(self) {
            ExprData::PrivateIdentifier(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_as_expression(self, ast: &Ast<'_>) -> Option<TsAsExpression> {
        match ast.expr_data(self) {
            ExprData::TsAsExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_satisfies_expression(self, ast: &Ast<'_>) -> Option<TsSatisfiesExpression> {
        match ast.expr_data(self) {
            ExprData::TsSatisfiesExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_type_assertion(self, ast: &Ast<'_>) -> Option<TsTypeAssertion> {
        match ast.expr_data(self) {
            ExprData::TsTypeAssertion(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_non_null_expression(self, ast: &Ast<'_>) -> Option<TsNonNullExpression> {
        match ast.expr_data(self) {
            ExprData::TsNonNullExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_instantiation_expression(
        self,
        ast: &Ast<'_>,
    ) -> Option<TsInstantiationExpression> {
        match ast.expr_data(self) {
            ExprData::TsInstantiationExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(self, ast: &Ast<'_>) -> Option<JsxElement> {
        match ast.expr_data(self) {
            ExprData::JsxElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self, ast: &Ast<'_>) -> Option<JsxFragment> {
        match ast.expr_data(self) {
            ExprData::JsxFragment(it) => Some(it),
            _ => None,
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
        match ast.binding_pattern_data(self) {
            BindingPatternData::BindingIdentifier(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_simple_assignment_target(self, ast: &Ast<'_>) -> Option<SimpleAssignmentTarget> {
        match ast.binding_pattern_data(self) {
            BindingPatternData::SimpleAssignmentTarget(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assignment_pattern(self, ast: &Ast<'_>) -> Option<AssignmentPattern> {
        match ast.binding_pattern_data(self) {
            BindingPatternData::AssignmentPattern(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_binding_rest_element(self, ast: &Ast<'_>) -> Option<BindingRestElement> {
        match ast.binding_pattern_data(self) {
            BindingPatternData::BindingRestElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array_pattern(self, ast: &Ast<'_>) -> Option<ArrayPattern> {
        match ast.binding_pattern_data(self) {
            BindingPatternData::ArrayPattern(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object_pattern(self, ast: &Ast<'_>) -> Option<ObjectPattern> {
        match ast.binding_pattern_data(self) {
            BindingPatternData::ObjectPattern(it) => Some(it),
            _ => None,
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
        match ast.assignment_target_data(self) {
            AssignmentTargetData::SimpleAssignmentTarget(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_array_assignment_target(self, ast: &Ast<'_>) -> Option<ArrayPattern> {
        match ast.assignment_target_data(self) {
            AssignmentTargetData::ArrayAssignmentTarget(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_object_assignment_target(self, ast: &Ast<'_>) -> Option<ObjectPattern> {
        match ast.assignment_target_data(self) {
            AssignmentTargetData::ObjectAssignmentTarget(it) => Some(it),
            _ => None,
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
        match ast.simple_assignment_target_data(self) {
            SimpleAssignmentTargetData::IdentifierReference(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_member_expression(self, ast: &Ast<'_>) -> Option<MemberExpression> {
        match ast.simple_assignment_target_data(self) {
            SimpleAssignmentTargetData::MemberExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_as_expression(self, ast: &Ast<'_>) -> Option<TsAsExpression> {
        match ast.simple_assignment_target_data(self) {
            SimpleAssignmentTargetData::TsAsExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_satisfies_expression(self, ast: &Ast<'_>) -> Option<TsSatisfiesExpression> {
        match ast.simple_assignment_target_data(self) {
            SimpleAssignmentTargetData::TsSatisfiesExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_type_assertion(self, ast: &Ast<'_>) -> Option<TsTypeAssertion> {
        match ast.simple_assignment_target_data(self) {
            SimpleAssignmentTargetData::TsTypeAssertion(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_non_null_expression(self, ast: &Ast<'_>) -> Option<TsNonNullExpression> {
        match ast.simple_assignment_target_data(self) {
            SimpleAssignmentTargetData::TsNonNullExpression(it) => Some(it),
            _ => None,
        }
    }
}
impl PropertyKey {
    #[inline]
    pub fn is_identifier_name(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierName
    }
    #[inline]
    pub fn is_private_identifier(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::PrivateIdentifier
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
        match ast.property_key_data(self) {
            PropertyKeyData::IdentifierName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_private_identifier(self, ast: &Ast<'_>) -> Option<PrivateIdentifier> {
        match ast.property_key_data(self) {
            PropertyKeyData::PrivateIdentifier(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_>) -> Option<StringLiteral> {
        match ast.property_key_data(self) {
            PropertyKeyData::StringLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_numeric_literal(self, ast: &Ast<'_>) -> Option<NumericLiteral> {
        match ast.property_key_data(self) {
            PropertyKeyData::NumericLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int_literal(self, ast: &Ast<'_>) -> Option<BigIntLiteral> {
        match ast.property_key_data(self) {
            PropertyKeyData::BigIntLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        match ast.property_key_data(self) {
            PropertyKeyData::Expr(it) => Some(it),
            _ => None,
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
        match ast.module_export_name_data(self) {
            ModuleExportNameData::IdentifierName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_>) -> Option<StringLiteral> {
        match ast.module_export_name_data(self) {
            ModuleExportNameData::StringLiteral(it) => Some(it),
            _ => None,
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
        match ast.argument_data(self) {
            ArgumentData::Expr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_spread_element(self, ast: &Ast<'_>) -> Option<SpreadElement> {
        match ast.argument_data(self) {
            ArgumentData::SpreadElement(it) => Some(it),
            _ => None,
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
        match ast.object_property_kind_data(self) {
            ObjectPropertyKindData::ObjectProperty(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_spread_element(self, ast: &Ast<'_>) -> Option<SpreadElement> {
        match ast.object_property_kind_data(self) {
            ObjectPropertyKindData::SpreadElement(it) => Some(it),
            _ => None,
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
        match ast.for_statement_init_data(self) {
            ForStatementInitData::VariableDeclaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        match ast.for_statement_init_data(self) {
            ForStatementInitData::Expr(it) => Some(it),
            _ => None,
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
        match ast.for_statement_left_data(self) {
            ForStatementLeftData::VariableDeclaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_assignment_target(self, ast: &Ast<'_>) -> Option<AssignmentTarget> {
        match ast.for_statement_left_data(self) {
            ForStatementLeftData::AssignmentTarget(it) => Some(it),
            _ => None,
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
        match ast.arrow_function_body_data(self) {
            ArrowFunctionBodyData::FunctionBody(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        match ast.arrow_function_body_data(self) {
            ArrowFunctionBodyData::Expr(it) => Some(it),
            _ => None,
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
        match ast.formal_parameter_item_data(self) {
            FormalParameterItemData::FormalParameter(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_parameter_property(self, ast: &Ast<'_>) -> Option<TsParameterProperty> {
        match ast.formal_parameter_item_data(self) {
            FormalParameterItemData::TsParameterProperty(it) => Some(it),
            _ => None,
        }
    }
}
impl FormalParameterPattern {
    #[inline]
    pub fn is_binding_pattern(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::BindingIdentifier
                | NodeKind::IdentifierReference
                | NodeKind::MemberExpression
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::AssignmentPattern
                | NodeKind::BindingRestElement
                | NodeKind::ArrayPattern
                | NodeKind::ObjectPattern
        )
    }
    #[inline]
    pub fn is_ts_this_parameter(self, ast: &Ast<'_>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsThisParameter
    }
    #[inline]
    pub fn as_binding_pattern(self, ast: &Ast<'_>) -> Option<BindingPattern> {
        match ast.formal_parameter_pattern_data(self) {
            FormalParameterPatternData::BindingPattern(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_this_parameter(self, ast: &Ast<'_>) -> Option<TsThisParameter> {
        match ast.formal_parameter_pattern_data(self) {
            FormalParameterPatternData::TsThisParameter(it) => Some(it),
            _ => None,
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
        match ast.class_element_data(self) {
            ClassElementData::MethodDefinition(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_property_definition(self, ast: &Ast<'_>) -> Option<PropertyDefinition> {
        match ast.class_element_data(self) {
            ClassElementData::PropertyDefinition(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_static_block(self, ast: &Ast<'_>) -> Option<StaticBlock> {
        match ast.class_element_data(self) {
            ClassElementData::StaticBlock(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_method_definition(self, ast: &Ast<'_>) -> Option<TsMethodDefinition> {
        match ast.class_element_data(self) {
            ClassElementData::TsMethodDefinition(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_index_signature(self, ast: &Ast<'_>) -> Option<TsIndexSignature> {
        match ast.class_element_data(self) {
            ClassElementData::TsIndexSignature(it) => Some(it),
            _ => None,
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
        match ast.import_declaration_specifier_data(self) {
            ImportDeclarationSpecifierData::ImportSpecifier(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_import_default_specifier(self, ast: &Ast<'_>) -> Option<ImportDefaultSpecifier> {
        match ast.import_declaration_specifier_data(self) {
            ImportDeclarationSpecifierData::ImportDefaultSpecifier(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_import_namespace_specifier(self, ast: &Ast<'_>) -> Option<ImportNamespaceSpecifier> {
        match ast.import_declaration_specifier_data(self) {
            ImportDeclarationSpecifierData::ImportNamespaceSpecifier(it) => Some(it),
            _ => None,
        }
    }
}
impl ExportDefaultDeclarationKind {
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.export_default_declaration_kind_data(self),
            ExportDefaultDeclarationKindData::Expr(_)
        )
    }
    #[inline]
    pub fn is_function(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.export_default_declaration_kind_data(self),
            ExportDefaultDeclarationKindData::Function(_)
        )
    }
    #[inline]
    pub fn is_class(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.export_default_declaration_kind_data(self),
            ExportDefaultDeclarationKindData::Class(_)
        )
    }
    #[inline]
    pub fn is_ts_function(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.export_default_declaration_kind_data(self),
            ExportDefaultDeclarationKindData::TsFunction(_)
        )
    }
    #[inline]
    pub fn is_ts_interface_declaration(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.export_default_declaration_kind_data(self),
            ExportDefaultDeclarationKindData::TsInterfaceDeclaration(_)
        )
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        match ast.export_default_declaration_kind_data(self) {
            ExportDefaultDeclarationKindData::Expr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_function(self, ast: &Ast<'_>) -> Option<Function> {
        match ast.export_default_declaration_kind_data(self) {
            ExportDefaultDeclarationKindData::Function(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_class(self, ast: &Ast<'_>) -> Option<Class> {
        match ast.export_default_declaration_kind_data(self) {
            ExportDefaultDeclarationKindData::Class(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_function(self, ast: &Ast<'_>) -> Option<TsFunction> {
        match ast.export_default_declaration_kind_data(self) {
            ExportDefaultDeclarationKindData::TsFunction(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_interface_declaration(self, ast: &Ast<'_>) -> Option<TsInterfaceDeclaration> {
        match ast.export_default_declaration_kind_data(self) {
            ExportDefaultDeclarationKindData::TsInterfaceDeclaration(it) => Some(it),
            _ => None,
        }
    }
}
impl Super {}
impl ThisExpression {}
impl StringLiteral {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> AstWtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: AstWtf8) {
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: AstUtf8) {
        let field_val: u64 = unsafe { raw.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl NumericLiteral {
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> NumericLiteralKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, NumericLiteralKind>(raw as u8) }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a>, kind: NumericLiteralKind) {
        let field_val: u32 = kind as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: AstUtf8) {
        let field_val: u64 = unsafe { raw.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl BigIntLiteral {
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a>, raw: AstUtf8) {
        let field_val: u64 = unsafe { raw.to_extra_data(ast).other };
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
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn flags<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_pattern<'a>(self, ast: &mut crate::Ast<'a>, pattern: AstUtf8) {
        let field_val: u64 = unsafe { pattern.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_flags<'a>(self, ast: &mut crate::Ast<'a>, flags: AstUtf8) {
        let field_val: u64 = unsafe { flags.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl TemplateLiteral {
    #[inline]
    pub fn quasis<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TemplateElement> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn expressions<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Expr> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_quasis<'a>(self, ast: &mut crate::Ast<'a>, quasis: TypedSubRange<TemplateElement>) {
        let field_val: u64 = unsafe { quasis.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_expressions<'a>(self, ast: &mut crate::Ast<'a>, expressions: TypedSubRange<Expr>) {
        let field_val: u64 = unsafe { expressions.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl TemplateElement {
    #[inline]
    pub fn cooked<'a>(self, ast: &crate::Ast<'a>) -> AstWtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 16usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
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
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_cooked<'a>(self, ast: &mut crate::Ast<'a>, cooked: AstWtf8) {
        let field_val: u64 = unsafe { cooked.to_extra_data(ast).other };
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
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = raw.to_extra_data(ast);
        };
    }
}
impl IdentifierReference {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        let field_val: u64 = unsafe { name.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl PrivateIdentifier {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        let field_val: u64 = unsafe { name.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl BindingIdentifier {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = name.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data(ast);
        };
    }
}
impl IdentifierName {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        let field_val: u64 = unsafe { name.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl LabelIdentifier {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a>) -> AstUtf8 {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        let field_val: u64 = unsafe { name.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl AssignmentPattern {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a>) -> BindingPattern {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        BindingPattern(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a>) -> Expr {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a>, left: BindingPattern) {
        let field_val: u32 = left.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374606332256583935u128)
            | (((field_val as u128) & 4294967295u128) << 8usize);
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a>, right: Expr) {
        let field_val: u32 = right.node_id().index() as u32;
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
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = type_annotation.to_extra_data(ast);
        };
    }
}
impl BindingRestElement {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a>) -> BindingPattern {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        BindingPattern(NodeId::from_raw_unchecked(raw))
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
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a>, argument: BindingPattern) {
        let field_val: u32 = argument.node_id().index() as u32;
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
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data(ast);
        };
    }
}
impl ArrayPattern {
    #[inline]
    pub fn rest<'a>(self, ast: &crate::Ast<'a>) -> Option<BindingRestElement> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| BindingRestElement(NodeId::from_raw_unchecked(raw)))
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
    pub fn elements<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Option<BindingPattern>> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_rest<'a>(self, ast: &mut crate::Ast<'a>, rest: Option<BindingRestElement>) {
        let field_val: u32 = rest.map_or(u32::MAX, |node| node.node_id().index() as u32);
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
    pub fn set_elements<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        elements: TypedSubRange<Option<BindingPattern>>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = elements.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data(ast);
        };
    }
}
impl ObjectPattern {
    #[inline]
    pub fn rest<'a>(self, ast: &crate::Ast<'a>) -> Option<BindingRestElement> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 8usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| BindingRestElement(NodeId::from_raw_unchecked(raw)))
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
    pub fn properties<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<BindingProperty> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_rest<'a>(self, ast: &mut crate::Ast<'a>, rest: Option<BindingRestElement>) {
        let field_val: u32 = rest.map_or(u32::MAX, |node| node.node_id().index() as u32);
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
    pub fn set_properties<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        properties: TypedSubRange<BindingProperty>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = properties.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data(ast);
        };
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
    pub fn declarators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<VariableDeclarator> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
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
        declarators: TypedSubRange<VariableDeclarator>,
    ) {
        let field_val: u64 = unsafe { declarators.to_extra_data(ast).other };
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
    pub fn init<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn definite<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 255u32;
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
    pub fn set_init<'a>(self, ast: &mut crate::Ast<'a>, init: Option<Expr>) {
        let field_val: u32 = init.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_definite<'a>(self, ast: &mut crate::Ast<'a>, definite: bool) {
        let field_val: u32 = definite as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938458759454868635832549375u128)
            | (((field_val as u128) & 255u128) << 64usize);
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
    pub fn cases<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<SwitchCase> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_discriminant<'a>(self, ast: &mut crate::Ast<'a>, discriminant: Expr) {
        let field_val: u32 = discriminant.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_cases<'a>(self, ast: &mut crate::Ast<'a>, cases: TypedSubRange<SwitchCase>) {
        let field_val: u64 = unsafe { cases.to_extra_data(ast).other };
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
    pub fn consequent<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Stmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a>, test: Option<Expr>) {
        let field_val: u32 = test.map_or(u32::MAX, |node| node.node_id().index() as u32);
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_consequent<'a>(self, ast: &mut crate::Ast<'a>, consequent: TypedSubRange<Stmt>) {
        let field_val: u64 = unsafe { consequent.to_extra_data(ast).other };
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
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> BlockStatement {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
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
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: BlockStatement) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
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
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
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
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
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
    pub fn super_class<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 56usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
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
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> ClassBody {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn super_type_arguments<'a>(
        self,
        ast: &crate::Ast<'a>,
    ) -> Option<TsTypeParameterInstantiation> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn implements<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TsClassImplements> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(4usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
    pub fn set_super_class<'a>(self, ast: &mut crate::Ast<'a>, super_class: Option<Expr>) {
        let field_val: u32 = super_class.map_or(u32::MAX, |node| node.node_id().index() as u32);
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
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: ClassBody) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = type_parameters.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_super_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        super_type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = super_type_arguments.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_implements<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        implements: TypedSubRange<TsClassImplements>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(4usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = implements.to_extra_data(ast);
        };
    }
}
impl ClassBody {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ClassElement> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TypedSubRange<ClassElement>) {
        let field_val: u64 = unsafe { body.to_extra_data(ast).other };
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
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Function {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Function) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = value.to_extra_data(ast);
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
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a>) -> PropertyKey {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a>, key: PropertyKey) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = key.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: Option<Expr>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = value.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsTypeAnnotation>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(3usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = type_annotation.to_extra_data(ast);
        };
    }
}
impl StaticBlock {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Stmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TypedSubRange<Stmt>) {
        let field_val: u64 = unsafe { body.to_extra_data(ast).other };
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
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 56usize) as u32) & 4294967295u32;
        FormalParameters(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> FunctionBody {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        let field_val: u32 = params.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920628978453625319957081358335u128)
            | (((field_val as u128) & 4294967295u128) << 56usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: FunctionBody) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = type_parameters.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        return_type: Option<TsTypeAnnotation>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = return_type.to_extra_data(ast);
        };
    }
}
impl FunctionBody {
    #[inline]
    pub fn directives<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Directive> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Stmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_directives<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        directives: TypedSubRange<Directive>,
    ) {
        let field_val: u64 = unsafe { directives.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TypedSubRange<Stmt>) {
        let field_val: u64 = unsafe { body.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 18446744073709551615u128)
            | (((field_val as u128) & 18446744073709551615u128) << 64usize);
    }
}
impl BlockStatement {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Stmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TypedSubRange<Stmt>) {
        let field_val: u64 = unsafe { body.to_extra_data(ast).other };
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
    pub fn items<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<FormalParameterItem> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn rest<'a>(self, ast: &crate::Ast<'a>) -> Option<BindingRestElement> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| BindingRestElement(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a>) -> FormalParameterKind {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 255u32;
        unsafe { std::mem::transmute::<u8, FormalParameterKind>(raw as u8) }
    }
    #[inline]
    pub fn set_items<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        items: TypedSubRange<FormalParameterItem>,
    ) {
        let field_val: u64 = unsafe { items.to_extra_data(ast).other };
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
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        FormalParameterPattern(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn set_pattern<'a>(self, ast: &mut crate::Ast<'a>, pattern: FormalParameterPattern) {
        let field_val: u32 = pattern.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
}
impl ArrayExpression {
    #[inline]
    pub fn elements<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Option<Argument>> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_elements<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        elements: TypedSubRange<Option<Argument>>,
    ) {
        let field_val: u64 = unsafe { elements.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl ObjectExpression {
    #[inline]
    pub fn properties<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ObjectPropertyKind> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_properties<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        properties: TypedSubRange<ObjectPropertyKind>,
    ) {
        let field_val: u64 = unsafe { properties.to_extra_data(ast).other };
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
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 16usize) as u32) & 4294967295u32;
        FormalParameters(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> ArrowFunctionBody {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 48usize) as u32) & 4294967295u32;
        ArrowFunctionBody(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        let field_val: u32 = params.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374325956791566335u128)
            | (((field_val as u128) & 4294967295u128) << 16usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: ArrowFunctionBody) {
        let field_val: u32 = body.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920937254537555274277570215935u128)
            | (((field_val as u128) & 4294967295u128) << 48usize);
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = type_parameters.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        return_type: Option<TsTypeAnnotation>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = return_type.to_extra_data(ast);
        };
    }
}
impl SequenceExpression {
    #[inline]
    pub fn expressions<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Expr> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_expressions<'a>(self, ast: &mut crate::Ast<'a>, expressions: TypedSubRange<Expr>) {
        let field_val: u64 = unsafe { expressions.to_extra_data(ast).other };
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
    pub fn optional<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn arguments<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Argument> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a>, optional: bool) {
        let field_val: u32 = optional as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
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
    pub fn set_arguments<'a>(self, ast: &mut crate::Ast<'a>, arguments: TypedSubRange<Argument>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = arguments.to_extra_data(ast);
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
    pub fn quasi<'a>(self, ast: &crate::Ast<'a>) -> TemplateLiteral {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        TemplateLiteral(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_tag<'a>(self, ast: &mut crate::Ast<'a>, tag: Expr) {
        let field_val: u32 = tag.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607427473244160u128)
            | (((field_val as u128) & 4294967295u128) << 0usize);
    }
    #[inline]
    pub fn set_quasi<'a>(self, ast: &mut crate::Ast<'a>, quasi: TemplateLiteral) {
        let field_val: u32 = quasi.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863362353627135u128)
            | (((field_val as u128) & 4294967295u128) << 32usize);
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_arguments: Option<TsTypeParameterInstantiation>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
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
    pub fn arguments<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Argument> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 96usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a>, callee: Expr) {
        let field_val: u32 = callee.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366841710300967557013911933812735u128)
            | (((field_val as u128) & 4294967295u128) << 64usize);
    }
    #[inline]
    pub fn set_arguments<'a>(self, ast: &mut crate::Ast<'a>, arguments: TypedSubRange<Argument>) {
        let field_val: u64 = unsafe { arguments.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
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
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 4294967295u32;
        Expr(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn options<'a>(self, ast: &crate::Ast<'a>) -> Option<Expr> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 32usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn phase<'a>(self, ast: &crate::Ast<'a>) -> Option<ImportPhase> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a>, source: Expr) {
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
    pub fn set_phase<'a>(self, ast: &mut crate::Ast<'a>, phase: Option<ImportPhase>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = phase.to_extra_data(ast);
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
    pub fn specifiers<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ImportDeclarationSpecifier> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ImportAttribute> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn phase<'a>(self, ast: &crate::Ast<'a>) -> Option<ImportPhase> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
        specifiers: TypedSubRange<ImportDeclarationSpecifier>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = specifiers.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_attributes<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        attributes: TypedSubRange<ImportAttribute>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = attributes.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_phase<'a>(self, ast: &mut crate::Ast<'a>, phase: Option<ImportPhase>) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(2usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = phase.to_extra_data(ast);
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
    pub fn specifiers<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ExportSpecifier> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ImportAttribute> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
        specifiers: TypedSubRange<ExportSpecifier>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = specifiers.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_attributes<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        attributes: TypedSubRange<ImportAttribute>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = attributes.to_extra_data(ast);
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
    pub fn attributes<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<ImportAttribute> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
        attributes: TypedSubRange<ImportAttribute>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = attributes.to_extra_data(ast);
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
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: AstUtf8) {
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
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
        match ast.ts_type_data(self) {
            TsTypeData::TsAnyKeyword(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_unknown_keyword(self, ast: &Ast<'_>) -> Option<TsUnknownKeyword> {
        match ast.ts_type_data(self) {
            TsTypeData::TsUnknownKeyword(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_never_keyword(self, ast: &Ast<'_>) -> Option<TsNeverKeyword> {
        match ast.ts_type_data(self) {
            TsTypeData::TsNeverKeyword(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_void_keyword(self, ast: &Ast<'_>) -> Option<TsVoidKeyword> {
        match ast.ts_type_data(self) {
            TsTypeData::TsVoidKeyword(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_null_keyword(self, ast: &Ast<'_>) -> Option<TsNullKeyword> {
        match ast.ts_type_data(self) {
            TsTypeData::TsNullKeyword(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_undefined_keyword(self, ast: &Ast<'_>) -> Option<TsUndefinedKeyword> {
        match ast.ts_type_data(self) {
            TsTypeData::TsUndefinedKeyword(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_string_keyword(self, ast: &Ast<'_>) -> Option<TsStringKeyword> {
        match ast.ts_type_data(self) {
            TsTypeData::TsStringKeyword(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_number_keyword(self, ast: &Ast<'_>) -> Option<TsNumberKeyword> {
        match ast.ts_type_data(self) {
            TsTypeData::TsNumberKeyword(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_big_int_keyword(self, ast: &Ast<'_>) -> Option<TsBigIntKeyword> {
        match ast.ts_type_data(self) {
            TsTypeData::TsBigIntKeyword(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_boolean_keyword(self, ast: &Ast<'_>) -> Option<TsBooleanKeyword> {
        match ast.ts_type_data(self) {
            TsTypeData::TsBooleanKeyword(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_symbol_keyword(self, ast: &Ast<'_>) -> Option<TsSymbolKeyword> {
        match ast.ts_type_data(self) {
            TsTypeData::TsSymbolKeyword(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_object_keyword(self, ast: &Ast<'_>) -> Option<TsObjectKeyword> {
        match ast.ts_type_data(self) {
            TsTypeData::TsObjectKeyword(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_intrinsic_keyword(self, ast: &Ast<'_>) -> Option<TsIntrinsicKeyword> {
        match ast.ts_type_data(self) {
            TsTypeData::TsIntrinsicKeyword(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_this_type(self, ast: &Ast<'_>) -> Option<TsThisType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsThisType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_type_reference(self, ast: &Ast<'_>) -> Option<TsTypeReference> {
        match ast.ts_type_data(self) {
            TsTypeData::TsTypeReference(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_type_query(self, ast: &Ast<'_>) -> Option<TsTypeQuery> {
        match ast.ts_type_data(self) {
            TsTypeData::TsTypeQuery(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_import_type(self, ast: &Ast<'_>) -> Option<TsImportType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsImportType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_literal_type(self, ast: &Ast<'_>) -> Option<TsLiteralType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsLiteralType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_template_literal_type(self, ast: &Ast<'_>) -> Option<TsTemplateLiteralType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsTemplateLiteralType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_array_type(self, ast: &Ast<'_>) -> Option<TsArrayType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsArrayType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_indexed_access_type(self, ast: &Ast<'_>) -> Option<TsIndexedAccessType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsIndexedAccessType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_tuple_type(self, ast: &Ast<'_>) -> Option<TsTupleType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsTupleType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_optional_type(self, ast: &Ast<'_>) -> Option<TsOptionalType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsOptionalType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_rest_type(self, ast: &Ast<'_>) -> Option<TsRestType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsRestType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_js_doc_nullable_type(self, ast: &Ast<'_>) -> Option<TsJsDocNullableType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsJsDocNullableType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_js_doc_non_nullable_type(self, ast: &Ast<'_>) -> Option<TsJsDocNonNullableType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsJsDocNonNullableType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_js_doc_unknown_type(self, ast: &Ast<'_>) -> Option<TsJsDocUnknownType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsJsDocUnknownType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_union_type(self, ast: &Ast<'_>) -> Option<TsUnionType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsUnionType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_intersection_type(self, ast: &Ast<'_>) -> Option<TsIntersectionType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsIntersectionType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_conditional_type(self, ast: &Ast<'_>) -> Option<TsConditionalType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsConditionalType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_infer_type(self, ast: &Ast<'_>) -> Option<TsInferType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsInferType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_type_operator(self, ast: &Ast<'_>) -> Option<TsTypeOperator> {
        match ast.ts_type_data(self) {
            TsTypeData::TsTypeOperator(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_parenthesized_type(self, ast: &Ast<'_>) -> Option<TsParenthesizedType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsParenthesizedType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_function_type(self, ast: &Ast<'_>) -> Option<TsFunctionType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsFunctionType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_constructor_type(self, ast: &Ast<'_>) -> Option<TsConstructorType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsConstructorType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_type_predicate(self, ast: &Ast<'_>) -> Option<TsTypePredicate> {
        match ast.ts_type_data(self) {
            TsTypeData::TsTypePredicate(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_type_literal(self, ast: &Ast<'_>) -> Option<TsTypeLiteral> {
        match ast.ts_type_data(self) {
            TsTypeData::TsTypeLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_mapped_type(self, ast: &Ast<'_>) -> Option<TsMappedType> {
        match ast.ts_type_data(self) {
            TsTypeData::TsMappedType(it) => Some(it),
            _ => None,
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
        match ast.ts_type_name_data(self) {
            TsTypeNameData::IdentifierReference(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_qualified_name(self, ast: &Ast<'_>) -> Option<TsQualifiedName> {
        match ast.ts_type_name_data(self) {
            TsTypeNameData::TsQualifiedName(it) => Some(it),
            _ => None,
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
        match ast.ts_qualified_name_left_data(self) {
            TsQualifiedNameLeftData::IdentifierReference(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_identifier_name(self, ast: &Ast<'_>) -> Option<IdentifierName> {
        match ast.ts_qualified_name_left_data(self) {
            TsQualifiedNameLeftData::IdentifierName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_binding_identifier(self, ast: &Ast<'_>) -> Option<BindingIdentifier> {
        match ast.ts_qualified_name_left_data(self) {
            TsQualifiedNameLeftData::BindingIdentifier(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_this_expression(self, ast: &Ast<'_>) -> Option<ThisExpression> {
        match ast.ts_qualified_name_left_data(self) {
            TsQualifiedNameLeftData::ThisExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_qualified_name(self, ast: &Ast<'_>) -> Option<TsQualifiedName> {
        match ast.ts_qualified_name_left_data(self) {
            TsQualifiedNameLeftData::TsQualifiedName(it) => Some(it),
            _ => None,
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
        match ast.ts_import_type_qualifier_data(self) {
            TsImportTypeQualifierData::IdentifierName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_qualified_name(self, ast: &Ast<'_>) -> Option<TsQualifiedName> {
        match ast.ts_import_type_qualifier_data(self) {
            TsImportTypeQualifierData::TsQualifiedName(it) => Some(it),
            _ => None,
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
        match ast.ts_type_query_expr_name_data(self) {
            TsTypeQueryExprNameData::TsTypeName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_this_expression(self, ast: &Ast<'_>) -> Option<ThisExpression> {
        match ast.ts_type_query_expr_name_data(self) {
            TsTypeQueryExprNameData::ThisExpression(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_import_type(self, ast: &Ast<'_>) -> Option<TsImportType> {
        match ast.ts_type_query_expr_name_data(self) {
            TsTypeQueryExprNameData::TsImportType(it) => Some(it),
            _ => None,
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
        match ast.ts_literal_data(self) {
            TsLiteralData::StringLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_numeric_literal(self, ast: &Ast<'_>) -> Option<NumericLiteral> {
        match ast.ts_literal_data(self) {
            TsLiteralData::NumericLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_big_int_literal(self, ast: &Ast<'_>) -> Option<BigIntLiteral> {
        match ast.ts_literal_data(self) {
            TsLiteralData::BigIntLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_boolean_literal(self, ast: &Ast<'_>) -> Option<BooleanLiteral> {
        match ast.ts_literal_data(self) {
            TsLiteralData::BooleanLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_template_literal(self, ast: &Ast<'_>) -> Option<TemplateLiteral> {
        match ast.ts_literal_data(self) {
            TsLiteralData::TemplateLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_unary_expression(self, ast: &Ast<'_>) -> Option<UnaryExpression> {
        match ast.ts_literal_data(self) {
            TsLiteralData::UnaryExpression(it) => Some(it),
            _ => None,
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
        match ast.ts_tuple_element_data(self) {
            TsTupleElementData::TsType(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_named_tuple_member(self, ast: &Ast<'_>) -> Option<TsNamedTupleMember> {
        match ast.ts_tuple_element_data(self) {
            TsTupleElementData::TsNamedTupleMember(it) => Some(it),
            _ => None,
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
        match ast.ts_type_element_data(self) {
            TsTypeElementData::TsPropertySignature(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_method_signature(self, ast: &Ast<'_>) -> Option<TsMethodSignature> {
        match ast.ts_type_element_data(self) {
            TsTypeElementData::TsMethodSignature(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_call_signature_declaration(
        self,
        ast: &Ast<'_>,
    ) -> Option<TsCallSignatureDeclaration> {
        match ast.ts_type_element_data(self) {
            TsTypeElementData::TsCallSignatureDeclaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_construct_signature_declaration(
        self,
        ast: &Ast<'_>,
    ) -> Option<TsConstructSignatureDeclaration> {
        match ast.ts_type_element_data(self) {
            TsTypeElementData::TsConstructSignatureDeclaration(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_index_signature(self, ast: &Ast<'_>) -> Option<TsIndexSignature> {
        match ast.ts_type_element_data(self) {
            TsTypeElementData::TsIndexSignature(it) => Some(it),
            _ => None,
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
        match ast.ts_type_predicate_name_data(self) {
            TsTypePredicateNameData::IdentifierName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_this_type(self, ast: &Ast<'_>) -> Option<TsThisType> {
        match ast.ts_type_predicate_name_data(self) {
            TsTypePredicateNameData::TsThisType(it) => Some(it),
            _ => None,
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
        match ast.ts_enum_member_name_data(self) {
            TsEnumMemberNameData::IdentifierName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_>) -> Option<StringLiteral> {
        match ast.ts_enum_member_name_data(self) {
            TsEnumMemberNameData::StringLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_template_literal(self, ast: &Ast<'_>) -> Option<TemplateLiteral> {
        match ast.ts_enum_member_name_data(self) {
            TsEnumMemberNameData::TemplateLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        match ast.ts_enum_member_name_data(self) {
            TsEnumMemberNameData::Expr(it) => Some(it),
            _ => None,
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
        match ast.ts_module_name_data(self) {
            TsModuleNameData::BindingIdentifier(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_>) -> Option<StringLiteral> {
        match ast.ts_module_name_data(self) {
            TsModuleNameData::StringLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_qualified_name(self, ast: &Ast<'_>) -> Option<TsQualifiedName> {
        match ast.ts_module_name_data(self) {
            TsModuleNameData::TsQualifiedName(it) => Some(it),
            _ => None,
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
        match ast.ts_module_body_data(self) {
            TsModuleBodyData::TsModuleBlock(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_module_declaration(self, ast: &Ast<'_>) -> Option<TsModuleDeclaration> {
        match ast.ts_module_body_data(self) {
            TsModuleBodyData::TsModuleDeclaration(it) => Some(it),
            _ => None,
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
        match ast.ts_module_reference_data(self) {
            TsModuleReferenceData::TsTypeName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_ts_external_module_reference(
        self,
        ast: &Ast<'_>,
    ) -> Option<TsExternalModuleReference> {
        match ast.ts_module_reference_data(self) {
            TsModuleReferenceData::TsExternalModuleReference(it) => Some(it),
            _ => None,
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
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 56usize) as u32) & 4294967295u32;
        FormalParameters(NodeId::from_raw_unchecked(raw))
    }
    #[inline]
    pub fn type_parameters<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterDeclaration> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        let field_val: u32 = params.node_id().index() as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920628978453625319957081358335u128)
            | (((field_val as u128) & 4294967295u128) << 56usize);
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_parameters: Option<TsTypeParameterDeclaration>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = type_parameters.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        return_type: Option<TsTypeAnnotation>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = return_type.to_extra_data(ast);
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
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a>) -> TsFunction {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: TsFunction) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = value.to_extra_data(ast);
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
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TsTypeParameter> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: TypedSubRange<TsTypeParameter>) {
        let field_val: u64 = unsafe { params.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl TsTypeParameterInstantiation {
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TsType> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: TypedSubRange<TsType>) {
        let field_val: u64 = unsafe { params.to_extra_data(ast).other };
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
    pub fn quasis<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TemplateElement> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn types<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TsType> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_quasis<'a>(self, ast: &mut crate::Ast<'a>, quasis: TypedSubRange<TemplateElement>) {
        let field_val: u64 = unsafe { quasis.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_types<'a>(self, ast: &mut crate::Ast<'a>, types: TypedSubRange<TsType>) {
        let field_val: u64 = unsafe { types.to_extra_data(ast).other };
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
    pub fn element_types<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TsTupleElement> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_element_types<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        element_types: TypedSubRange<TsTupleElement>,
    ) {
        let field_val: u64 = unsafe { element_types.to_extra_data(ast).other };
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
    pub fn types<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TsType> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_types<'a>(self, ast: &mut crate::Ast<'a>, types: TypedSubRange<TsType>) {
        let field_val: u64 = unsafe { types.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
}
impl TsIntersectionType {
    #[inline]
    pub fn types<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TsType> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_types<'a>(self, ast: &mut crate::Ast<'a>, types: TypedSubRange<TsType>) {
        let field_val: u64 = unsafe { types.to_extra_data(ast).other };
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
    pub fn members<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TsTypeElement> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_members<'a>(self, ast: &mut crate::Ast<'a>, members: TypedSubRange<TsTypeElement>) {
        let field_val: u64 = unsafe { members.to_extra_data(ast).other };
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
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a>) -> Option<TsType> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = name_type.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        type_annotation: Option<TsType>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = type_annotation.to_extra_data(ast);
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
    pub fn params<'a>(self, ast: &crate::Ast<'a>) -> FormalParameters {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeAnnotation> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a>, params: FormalParameters) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = params.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        return_type: Option<TsTypeAnnotation>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = return_type.to_extra_data(ast);
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
    pub fn parameters<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<BindingIdentifier> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
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
        parameters: TypedSubRange<BindingIdentifier>,
    ) {
        let field_val: u64 = unsafe { parameters.to_extra_data(ast).other };
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
    pub fn extends<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TsInterfaceHeritage> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TsInterfaceBody {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
    pub fn set_extends<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        extends: TypedSubRange<TsInterfaceHeritage>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = extends.to_extra_data(ast);
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TsInterfaceBody) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(1usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.to_extra_data(ast);
        };
    }
}
impl TsInterfaceBody {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TsTypeElement> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TypedSubRange<TsTypeElement>) {
        let field_val: u64 = unsafe { body.to_extra_data(ast).other };
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
    pub fn members<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<TsEnumMember> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_members<'a>(self, ast: &mut crate::Ast<'a>, members: TypedSubRange<TsEnumMember>) {
        let field_val: u64 = unsafe { members.to_extra_data(ast).other };
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
    pub fn directives<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Directive> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Stmt> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_directives<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        directives: TypedSubRange<Directive>,
    ) {
        let field_val: u64 = unsafe { directives.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a>, body: TypedSubRange<Stmt>) {
        let field_val: u64 = unsafe { body.to_extra_data(ast).other };
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
    pub fn decorators<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<Decorator> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn parameter<'a>(self, ast: &crate::Ast<'a>) -> BindingPattern {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 64usize) as u32) & 4294967295u32;
        BindingPattern(NodeId::from_raw_unchecked(raw))
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
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        decorators: TypedSubRange<Decorator>,
    ) {
        let field_val: u64 = unsafe { decorators.to_extra_data(ast).other };
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463444927863358058659840u128)
            | (((field_val as u128) & 18446744073709551615u128) << 0usize);
    }
    #[inline]
    pub fn set_parameter<'a>(self, ast: &mut crate::Ast<'a>, parameter: BindingPattern) {
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
        match ast.jsx_element_name_data(self) {
            JsxElementNameData::JsxIdentifier(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self, ast: &Ast<'_>) -> Option<JsxNamespacedName> {
        match ast.jsx_element_name_data(self) {
            JsxElementNameData::JsxNamespacedName(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_member_expression(self, ast: &Ast<'_>) -> Option<JsxMemberExpression> {
        match ast.jsx_element_name_data(self) {
            JsxElementNameData::JsxMemberExpression(it) => Some(it),
            _ => None,
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
        match ast.jsx_member_expression_object_data(self) {
            JsxMemberExpressionObjectData::JsxIdentifier(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_member_expression(self, ast: &Ast<'_>) -> Option<JsxMemberExpression> {
        match ast.jsx_member_expression_object_data(self) {
            JsxMemberExpressionObjectData::JsxMemberExpression(it) => Some(it),
            _ => None,
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
        match ast.jsx_attribute_name_data(self) {
            JsxAttributeNameData::JsxIdentifier(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self, ast: &Ast<'_>) -> Option<JsxNamespacedName> {
        match ast.jsx_attribute_name_data(self) {
            JsxAttributeNameData::JsxNamespacedName(it) => Some(it),
            _ => None,
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
        match ast.jsx_attribute_item_data(self) {
            JsxAttributeItemData::JsxAttribute(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_spread_attribute(self, ast: &Ast<'_>) -> Option<JsxSpreadAttribute> {
        match ast.jsx_attribute_item_data(self) {
            JsxAttributeItemData::JsxSpreadAttribute(it) => Some(it),
            _ => None,
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
        match ast.jsx_attribute_value_data(self) {
            JsxAttributeValueData::StringLiteral(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_expression_container(self, ast: &Ast<'_>) -> Option<JsxExpressionContainer> {
        match ast.jsx_attribute_value_data(self) {
            JsxAttributeValueData::JsxExpressionContainer(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(self, ast: &Ast<'_>) -> Option<JsxElement> {
        match ast.jsx_attribute_value_data(self) {
            JsxAttributeValueData::JsxElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self, ast: &Ast<'_>) -> Option<JsxFragment> {
        match ast.jsx_attribute_value_data(self) {
            JsxAttributeValueData::JsxFragment(it) => Some(it),
            _ => None,
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
        match ast.jsx_expression_data(self) {
            JsxExpressionData::Expr(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_empty_expression(self, ast: &Ast<'_>) -> Option<JsxEmptyExpression> {
        match ast.jsx_expression_data(self) {
            JsxExpressionData::JsxEmptyExpression(it) => Some(it),
            _ => None,
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
        match ast.jsx_child_data(self) {
            JsxChildData::JsxText(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_expression_container(self, ast: &Ast<'_>) -> Option<JsxExpressionContainer> {
        match ast.jsx_child_data(self) {
            JsxChildData::JsxExpressionContainer(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_spread_child(self, ast: &Ast<'_>) -> Option<JsxSpreadChild> {
        match ast.jsx_child_data(self) {
            JsxChildData::JsxSpreadChild(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_element(self, ast: &Ast<'_>) -> Option<JsxElement> {
        match ast.jsx_child_data(self) {
            JsxChildData::JsxElement(it) => Some(it),
            _ => None,
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self, ast: &Ast<'_>) -> Option<JsxFragment> {
        match ast.jsx_child_data(self) {
            JsxChildData::JsxFragment(it) => Some(it),
            _ => None,
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
    pub fn children<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<JsxChild> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
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
    pub fn set_children<'a>(self, ast: &mut crate::Ast<'a>, children: TypedSubRange<JsxChild>) {
        let field_val: u64 = unsafe { children.to_extra_data(ast).other };
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
    pub fn self_closing<'a>(self, ast: &crate::Ast<'a>) -> bool {
        let raw = ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u32) & 255u32;
        raw != 0
    }
    #[inline]
    pub fn type_arguments<'a>(self, ast: &crate::Ast<'a>) -> Option<TsTypeParameterInstantiation> {
        let raw =
            ((unsafe { ast.nodes.data_unchecked(self.0).0 } >> 40usize) as u32) & 4294967295u32;
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_unchecked(raw)))
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<JsxAttributeItem> {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            ExtraDataCompact::from_extra_data(
                *ast.extra_data.as_slice().get_unchecked(offset.index()),
                ast,
            )
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
    pub fn set_self_closing<'a>(self, ast: &mut crate::Ast<'a>, self_closing: bool) {
        let field_val: u32 = self_closing as u32;
        let payload = unsafe { ast.nodes.data_mut_unchecked(self.0) };
        payload.0 = (payload.0 & 340282366920938463463374607431768211200u128)
            | (((field_val as u128) & 255u128) << 0usize);
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
    pub fn set_attributes<'a>(
        self,
        ast: &mut crate::Ast<'a>,
        attributes: TypedSubRange<JsxAttributeItem>,
    ) {
        let offset = unsafe {
            ExtraDataId::from_usize_unchecked(
                ((ast.nodes.data_unchecked(self.0).0 >> 96usize) as u32 as usize)
                    .wrapping_add(0usize),
            )
        };
        debug_assert!(offset.index() < ast.extra_data.len());
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = attributes.to_extra_data(ast);
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
    pub fn children<'a>(self, ast: &crate::Ast<'a>) -> TypedSubRange<JsxChild> {
        let raw = (unsafe { ast.nodes.data_unchecked(self.0).0 } >> 0usize) as u64;
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
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
    pub fn set_children<'a>(self, ast: &mut crate::Ast<'a>, children: TypedSubRange<JsxChild>) {
        let field_val: u64 = unsafe { children.to_extra_data(ast).other };
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
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a>, name: AstUtf8) {
        let field_val: u64 = unsafe { name.to_extra_data(ast).other };
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
        unsafe { ExtraDataCompact::from_extra_data(ExtraData { other: raw }, ast) }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a>, value: AstUtf8) {
        let field_val: u64 = unsafe { value.to_extra_data(ast).other };
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
