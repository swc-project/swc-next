//! Conversions and match macros for manually expanded enum variant groups.
use crate::{Ast, ast::*};
impl Stmt {
    ///Returns whether this node is a [`Decl`].
    #[inline]
    pub fn is_decl(self, ast: &Ast<'_>) -> bool {
        matches!(ast.stmt_data(self), match_decl!(StmtData))
    }
    ///Converts this [`Stmt`] to a [`Decl`] when it belongs to that variant group.
    #[inline]
    pub fn as_decl(self, ast: &Ast<'_>) -> Option<Decl> {
        self.is_decl(ast).then_some(Decl(self.0))
    }
    ///Converts this [`Stmt`] to a [`Decl`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_decl(self, ast: &Ast<'_>) -> Decl {
        self.as_decl(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl Decl {
    ///Converts this [`Decl`] to a [`Stmt`].
    #[inline]
    pub fn as_stmt(self) -> Stmt {
        Stmt(self.0)
    }
}
impl From<Decl> for Stmt {
    #[inline]
    fn from(value: Decl) -> Self {
        value.as_stmt()
    }
}
impl BindingPattern {
    ///Returns whether this node is a [`SimpleAssignmentTarget`].
    #[inline]
    pub fn is_simple_assignment_target(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.binding_pattern_data(self),
            match_simple_assignment_target!(BindingPatternData)
        )
    }
    ///Converts this [`BindingPattern`] to a [`SimpleAssignmentTarget`] when it belongs to that variant group.
    #[inline]
    pub fn as_simple_assignment_target(self, ast: &Ast<'_>) -> Option<SimpleAssignmentTarget> {
        self.is_simple_assignment_target(ast)
            .then_some(SimpleAssignmentTarget(self.0))
    }
    ///Converts this [`BindingPattern`] to a [`SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_simple_assignment_target(self, ast: &Ast<'_>) -> SimpleAssignmentTarget {
        self.as_simple_assignment_target(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl SimpleAssignmentTarget {
    ///Converts this [`SimpleAssignmentTarget`] to a [`BindingPattern`].
    #[inline]
    pub fn as_binding_pattern(self) -> BindingPattern {
        BindingPattern(self.0)
    }
}
impl From<SimpleAssignmentTarget> for BindingPattern {
    #[inline]
    fn from(value: SimpleAssignmentTarget) -> Self {
        value.as_binding_pattern()
    }
}
impl AssignmentTarget {
    ///Returns whether this node is a [`SimpleAssignmentTarget`].
    #[inline]
    pub fn is_simple_assignment_target(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.assignment_target_data(self),
            match_simple_assignment_target!(AssignmentTargetData)
        )
    }
    ///Converts this [`AssignmentTarget`] to a [`SimpleAssignmentTarget`] when it belongs to that variant group.
    #[inline]
    pub fn as_simple_assignment_target(self, ast: &Ast<'_>) -> Option<SimpleAssignmentTarget> {
        self.is_simple_assignment_target(ast)
            .then_some(SimpleAssignmentTarget(self.0))
    }
    ///Converts this [`AssignmentTarget`] to a [`SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_simple_assignment_target(self, ast: &Ast<'_>) -> SimpleAssignmentTarget {
        self.as_simple_assignment_target(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl SimpleAssignmentTarget {
    ///Converts this [`SimpleAssignmentTarget`] to a [`AssignmentTarget`].
    #[inline]
    pub fn as_assignment_target(self) -> AssignmentTarget {
        AssignmentTarget(self.0)
    }
}
impl From<SimpleAssignmentTarget> for AssignmentTarget {
    #[inline]
    fn from(value: SimpleAssignmentTarget) -> Self {
        value.as_assignment_target()
    }
}
impl PropertyKey {
    ///Returns whether this node is a [`Expr`].
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(ast.property_key_data(self), match_expr!(PropertyKeyData))
    }
    ///Converts this [`PropertyKey`] to a [`Expr`] when it belongs to that variant group.
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        self.is_expr(ast).then_some(Expr(self.0))
    }
    ///Converts this [`PropertyKey`] to a [`Expr`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_expr(self, ast: &Ast<'_>) -> Expr {
        self.as_expr(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl Expr {
    ///Converts this [`Expr`] to a [`PropertyKey`].
    #[inline]
    pub fn as_property_key(self) -> PropertyKey {
        PropertyKey(self.0)
    }
}
impl From<Expr> for PropertyKey {
    #[inline]
    fn from(value: Expr) -> Self {
        value.as_property_key()
    }
}
impl Argument {
    ///Returns whether this node is a [`Expr`].
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(ast.argument_data(self), match_expr!(ArgumentData))
    }
    ///Converts this [`Argument`] to a [`Expr`] when it belongs to that variant group.
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        self.is_expr(ast).then_some(Expr(self.0))
    }
    ///Converts this [`Argument`] to a [`Expr`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_expr(self, ast: &Ast<'_>) -> Expr {
        self.as_expr(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl Expr {
    ///Converts this [`Expr`] to a [`Argument`].
    #[inline]
    pub fn as_argument(self) -> Argument {
        Argument(self.0)
    }
}
impl From<Expr> for Argument {
    #[inline]
    fn from(value: Expr) -> Self {
        value.as_argument()
    }
}
impl ForStatementInit {
    ///Returns whether this node is a [`Expr`].
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.for_statement_init_data(self),
            match_expr!(ForStatementInitData)
        )
    }
    ///Converts this [`ForStatementInit`] to a [`Expr`] when it belongs to that variant group.
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        self.is_expr(ast).then_some(Expr(self.0))
    }
    ///Converts this [`ForStatementInit`] to a [`Expr`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_expr(self, ast: &Ast<'_>) -> Expr {
        self.as_expr(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl Expr {
    ///Converts this [`Expr`] to a [`ForStatementInit`].
    #[inline]
    pub fn as_for_statement_init(self) -> ForStatementInit {
        ForStatementInit(self.0)
    }
}
impl From<Expr> for ForStatementInit {
    #[inline]
    fn from(value: Expr) -> Self {
        value.as_for_statement_init()
    }
}
impl ForStatementLeft {
    ///Returns whether this node is a [`AssignmentTarget`].
    #[inline]
    pub fn is_assignment_target(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.for_statement_left_data(self),
            match_assignment_target!(ForStatementLeftData)
        )
    }
    ///Converts this [`ForStatementLeft`] to a [`AssignmentTarget`] when it belongs to that variant group.
    #[inline]
    pub fn as_assignment_target(self, ast: &Ast<'_>) -> Option<AssignmentTarget> {
        self.is_assignment_target(ast)
            .then_some(AssignmentTarget(self.0))
    }
    ///Converts this [`ForStatementLeft`] to a [`AssignmentTarget`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_assignment_target(self, ast: &Ast<'_>) -> AssignmentTarget {
        self.as_assignment_target(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl AssignmentTarget {
    ///Converts this [`AssignmentTarget`] to a [`ForStatementLeft`].
    #[inline]
    pub fn as_for_statement_left(self) -> ForStatementLeft {
        ForStatementLeft(self.0)
    }
}
impl From<AssignmentTarget> for ForStatementLeft {
    #[inline]
    fn from(value: AssignmentTarget) -> Self {
        value.as_for_statement_left()
    }
}
impl ForStatementLeft {
    ///Returns whether this node is a [`SimpleAssignmentTarget`].
    #[inline]
    pub fn is_simple_assignment_target(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.for_statement_left_data(self),
            match_simple_assignment_target!(ForStatementLeftData)
        )
    }
    ///Converts this [`ForStatementLeft`] to a [`SimpleAssignmentTarget`] when it belongs to that variant group.
    #[inline]
    pub fn as_simple_assignment_target(self, ast: &Ast<'_>) -> Option<SimpleAssignmentTarget> {
        self.is_simple_assignment_target(ast)
            .then_some(SimpleAssignmentTarget(self.0))
    }
    ///Converts this [`ForStatementLeft`] to a [`SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_simple_assignment_target(self, ast: &Ast<'_>) -> SimpleAssignmentTarget {
        self.as_simple_assignment_target(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl SimpleAssignmentTarget {
    ///Converts this [`SimpleAssignmentTarget`] to a [`ForStatementLeft`].
    #[inline]
    pub fn as_for_statement_left(self) -> ForStatementLeft {
        ForStatementLeft(self.0)
    }
}
impl From<SimpleAssignmentTarget> for ForStatementLeft {
    #[inline]
    fn from(value: SimpleAssignmentTarget) -> Self {
        value.as_for_statement_left()
    }
}
impl ArrowFunctionBody {
    ///Returns whether this node is a [`Expr`].
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.arrow_function_body_data(self),
            match_expr!(ArrowFunctionBodyData)
        )
    }
    ///Converts this [`ArrowFunctionBody`] to a [`Expr`] when it belongs to that variant group.
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        self.is_expr(ast).then_some(Expr(self.0))
    }
    ///Converts this [`ArrowFunctionBody`] to a [`Expr`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_expr(self, ast: &Ast<'_>) -> Expr {
        self.as_expr(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl Expr {
    ///Converts this [`Expr`] to a [`ArrowFunctionBody`].
    #[inline]
    pub fn as_arrow_function_body(self) -> ArrowFunctionBody {
        ArrowFunctionBody(self.0)
    }
}
impl From<Expr> for ArrowFunctionBody {
    #[inline]
    fn from(value: Expr) -> Self {
        value.as_arrow_function_body()
    }
}
impl FormalParameterPattern {
    ///Returns whether this node is a [`BindingPattern`].
    #[inline]
    pub fn is_binding_pattern(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.formal_parameter_pattern_data(self),
            match_binding_pattern!(FormalParameterPatternData)
        )
    }
    ///Converts this [`FormalParameterPattern`] to a [`BindingPattern`] when it belongs to that variant group.
    #[inline]
    pub fn as_binding_pattern(self, ast: &Ast<'_>) -> Option<BindingPattern> {
        self.is_binding_pattern(ast)
            .then_some(BindingPattern(self.0))
    }
    ///Converts this [`FormalParameterPattern`] to a [`BindingPattern`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_binding_pattern(self, ast: &Ast<'_>) -> BindingPattern {
        self.as_binding_pattern(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl BindingPattern {
    ///Converts this [`BindingPattern`] to a [`FormalParameterPattern`].
    #[inline]
    pub fn as_formal_parameter_pattern(self) -> FormalParameterPattern {
        FormalParameterPattern(self.0)
    }
}
impl From<BindingPattern> for FormalParameterPattern {
    #[inline]
    fn from(value: BindingPattern) -> Self {
        value.as_formal_parameter_pattern()
    }
}
impl FormalParameterPattern {
    ///Returns whether this node is a [`SimpleAssignmentTarget`].
    #[inline]
    pub fn is_simple_assignment_target(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.formal_parameter_pattern_data(self),
            match_simple_assignment_target!(FormalParameterPatternData)
        )
    }
    ///Converts this [`FormalParameterPattern`] to a [`SimpleAssignmentTarget`] when it belongs to that variant group.
    #[inline]
    pub fn as_simple_assignment_target(self, ast: &Ast<'_>) -> Option<SimpleAssignmentTarget> {
        self.is_simple_assignment_target(ast)
            .then_some(SimpleAssignmentTarget(self.0))
    }
    ///Converts this [`FormalParameterPattern`] to a [`SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_simple_assignment_target(self, ast: &Ast<'_>) -> SimpleAssignmentTarget {
        self.as_simple_assignment_target(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl SimpleAssignmentTarget {
    ///Converts this [`SimpleAssignmentTarget`] to a [`FormalParameterPattern`].
    #[inline]
    pub fn as_formal_parameter_pattern(self) -> FormalParameterPattern {
        FormalParameterPattern(self.0)
    }
}
impl From<SimpleAssignmentTarget> for FormalParameterPattern {
    #[inline]
    fn from(value: SimpleAssignmentTarget) -> Self {
        value.as_formal_parameter_pattern()
    }
}
impl ExportDefaultDeclarationKind {
    ///Returns whether this node is a [`Expr`].
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.export_default_declaration_kind_data(self),
            match_expr!(ExportDefaultDeclarationKindData)
        )
    }
    ///Converts this [`ExportDefaultDeclarationKind`] to a [`Expr`] when it belongs to that variant group.
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        self.is_expr(ast).then_some(Expr(self.0))
    }
    ///Converts this [`ExportDefaultDeclarationKind`] to a [`Expr`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_expr(self, ast: &Ast<'_>) -> Expr {
        self.as_expr(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl Expr {
    ///Converts this [`Expr`] to a [`ExportDefaultDeclarationKind`].
    #[inline]
    pub fn as_export_default_declaration_kind(self) -> ExportDefaultDeclarationKind {
        ExportDefaultDeclarationKind(self.0)
    }
}
impl From<Expr> for ExportDefaultDeclarationKind {
    #[inline]
    fn from(value: Expr) -> Self {
        value.as_export_default_declaration_kind()
    }
}
impl TsTypeQueryExprName {
    ///Returns whether this node is a [`TsTypeName`].
    #[inline]
    pub fn is_ts_type_name(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.ts_type_query_expr_name_data(self),
            match_ts_type_name!(TsTypeQueryExprNameData)
        )
    }
    ///Converts this [`TsTypeQueryExprName`] to a [`TsTypeName`] when it belongs to that variant group.
    #[inline]
    pub fn as_ts_type_name(self, ast: &Ast<'_>) -> Option<TsTypeName> {
        self.is_ts_type_name(ast).then_some(TsTypeName(self.0))
    }
    ///Converts this [`TsTypeQueryExprName`] to a [`TsTypeName`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_ts_type_name(self, ast: &Ast<'_>) -> TsTypeName {
        self.as_ts_type_name(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl TsTypeName {
    ///Converts this [`TsTypeName`] to a [`TsTypeQueryExprName`].
    #[inline]
    pub fn as_ts_type_query_expr_name(self) -> TsTypeQueryExprName {
        TsTypeQueryExprName(self.0)
    }
}
impl From<TsTypeName> for TsTypeQueryExprName {
    #[inline]
    fn from(value: TsTypeName) -> Self {
        value.as_ts_type_query_expr_name()
    }
}
impl TsTupleElement {
    ///Returns whether this node is a [`TsType`].
    #[inline]
    pub fn is_ts_type(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.ts_tuple_element_data(self),
            match_ts_type!(TsTupleElementData)
        )
    }
    ///Converts this [`TsTupleElement`] to a [`TsType`] when it belongs to that variant group.
    #[inline]
    pub fn as_ts_type(self, ast: &Ast<'_>) -> Option<TsType> {
        self.is_ts_type(ast).then_some(TsType(self.0))
    }
    ///Converts this [`TsTupleElement`] to a [`TsType`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_ts_type(self, ast: &Ast<'_>) -> TsType {
        self.as_ts_type(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl TsType {
    ///Converts this [`TsType`] to a [`TsTupleElement`].
    #[inline]
    pub fn as_ts_tuple_element(self) -> TsTupleElement {
        TsTupleElement(self.0)
    }
}
impl From<TsType> for TsTupleElement {
    #[inline]
    fn from(value: TsType) -> Self {
        value.as_ts_tuple_element()
    }
}
impl TsEnumMemberName {
    ///Returns whether this node is a [`Expr`].
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.ts_enum_member_name_data(self),
            match_expr!(TsEnumMemberNameData)
        )
    }
    ///Converts this [`TsEnumMemberName`] to a [`Expr`] when it belongs to that variant group.
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        self.is_expr(ast).then_some(Expr(self.0))
    }
    ///Converts this [`TsEnumMemberName`] to a [`Expr`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_expr(self, ast: &Ast<'_>) -> Expr {
        self.as_expr(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl Expr {
    ///Converts this [`Expr`] to a [`TsEnumMemberName`].
    #[inline]
    pub fn as_ts_enum_member_name(self) -> TsEnumMemberName {
        TsEnumMemberName(self.0)
    }
}
impl From<Expr> for TsEnumMemberName {
    #[inline]
    fn from(value: Expr) -> Self {
        value.as_ts_enum_member_name()
    }
}
impl TsModuleReference {
    ///Returns whether this node is a [`TsTypeName`].
    #[inline]
    pub fn is_ts_type_name(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.ts_module_reference_data(self),
            match_ts_type_name!(TsModuleReferenceData)
        )
    }
    ///Converts this [`TsModuleReference`] to a [`TsTypeName`] when it belongs to that variant group.
    #[inline]
    pub fn as_ts_type_name(self, ast: &Ast<'_>) -> Option<TsTypeName> {
        self.is_ts_type_name(ast).then_some(TsTypeName(self.0))
    }
    ///Converts this [`TsModuleReference`] to a [`TsTypeName`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_ts_type_name(self, ast: &Ast<'_>) -> TsTypeName {
        self.as_ts_type_name(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl TsTypeName {
    ///Converts this [`TsTypeName`] to a [`TsModuleReference`].
    #[inline]
    pub fn as_ts_module_reference(self) -> TsModuleReference {
        TsModuleReference(self.0)
    }
}
impl From<TsTypeName> for TsModuleReference {
    #[inline]
    fn from(value: TsTypeName) -> Self {
        value.as_ts_module_reference()
    }
}
impl JsxExpression {
    ///Returns whether this node is a [`Expr`].
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_>) -> bool {
        matches!(
            ast.jsx_expression_data(self),
            match_expr!(JsxExpressionData)
        )
    }
    ///Converts this [`JsxExpression`] to a [`Expr`] when it belongs to that variant group.
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_>) -> Option<Expr> {
        self.is_expr(ast).then_some(Expr(self.0))
    }
    ///Converts this [`JsxExpression`] to a [`Expr`].
    ///
    /// # Panics
    /// Panics when this node is not part of the target variant group.
    #[inline]
    pub fn to_expr(self, ast: &Ast<'_>) -> Expr {
        self.as_expr(ast)
            .expect("AST enum variant is not part of the target group")
    }
}
impl Expr {
    ///Converts this [`Expr`] to a [`JsxExpression`].
    #[inline]
    pub fn as_jsx_expression(self) -> JsxExpression {
        JsxExpression(self.0)
    }
}
impl From<Expr> for JsxExpression {
    #[inline]
    fn from(value: Expr) -> Self {
        value.as_jsx_expression()
    }
}
/// Matches all variants of the [`Decl`] group.
#[macro_export]
macro_rules! match_decl {
    ($ty:ident) => {
        $ty::VariableDeclaration(_)
            | $ty::FunctionDeclaration(_)
            | $ty::ClassDeclaration(_)
            | $ty::TsFunction(_)
            | $ty::TsTypeAliasDeclaration(_)
            | $ty::TsInterfaceDeclaration(_)
            | $ty::TsEnumDeclaration(_)
            | $ty::TsModuleDeclaration(_)
            | $ty::TsGlobalDeclaration(_)
            | $ty::TsImportEqualsDeclaration(_)
    };
}
pub use match_decl;
/// Matches all variants of the [`Expr`] group.
#[macro_export]
macro_rules! match_expr {
    ($ty:ident) => {
        $ty::IdentifierReference(_)
            | $ty::ThisExpression(_)
            | $ty::Super(_)
            | $ty::MetaProperty(_)
            | $ty::StringLiteral(_)
            | $ty::NumericLiteral(_)
            | $ty::BigIntLiteral(_)
            | $ty::BooleanLiteral(_)
            | $ty::NullLiteral(_)
            | $ty::RegExpLiteral(_)
            | $ty::TemplateLiteral(_)
            | $ty::BinaryExpression(_)
            | $ty::LogicalExpression(_)
            | $ty::UnaryExpression(_)
            | $ty::UpdateExpression(_)
            | $ty::AssignmentExpression(_)
            | $ty::ConditionalExpression(_)
            | $ty::SequenceExpression(_)
            | $ty::MemberExpression(_)
            | $ty::CallExpression(_)
            | $ty::ChainExpression(_)
            | $ty::NewExpression(_)
            | $ty::TaggedTemplateExpression(_)
            | $ty::ArrowFunctionExpression(_)
            | $ty::ArrayExpression(_)
            | $ty::ObjectExpression(_)
            | $ty::ParenthesizedExpression(_)
            | $ty::FunctionExpression(_)
            | $ty::ClassExpression(_)
            | $ty::ImportExpression(_)
            | $ty::AwaitExpression(_)
            | $ty::YieldExpression(_)
            | $ty::PrivateIdentifier(_)
            | $ty::TsAsExpression(_)
            | $ty::TsSatisfiesExpression(_)
            | $ty::TsTypeAssertion(_)
            | $ty::TsNonNullExpression(_)
            | $ty::TsInstantiationExpression(_)
            | $ty::JsxElement(_)
            | $ty::JsxFragment(_)
    };
}
pub use match_expr;
/// Matches all variants of the [`BindingPattern`] group.
#[macro_export]
macro_rules! match_binding_pattern {
    ($ty:ident) => {
        $ty::BindingIdentifier(_)
            | $ty::AssignmentPattern(_)
            | $ty::BindingRestElement(_)
            | $ty::ArrayPattern(_)
            | $ty::ObjectPattern(_)
            | $ty::IdentifierReference(_)
            | $ty::MemberExpression(_)
            | $ty::TsAsExpression(_)
            | $ty::TsSatisfiesExpression(_)
            | $ty::TsTypeAssertion(_)
            | $ty::TsNonNullExpression(_)
    };
}
pub use match_binding_pattern;
/// Matches all variants of the [`AssignmentTarget`] group.
#[macro_export]
macro_rules! match_assignment_target {
    ($ty:ident) => {
        $ty::ArrayAssignmentTarget(_)
            | $ty::ObjectAssignmentTarget(_)
            | $ty::IdentifierReference(_)
            | $ty::MemberExpression(_)
            | $ty::TsAsExpression(_)
            | $ty::TsSatisfiesExpression(_)
            | $ty::TsTypeAssertion(_)
            | $ty::TsNonNullExpression(_)
    };
}
pub use match_assignment_target;
/// Matches all variants of the [`SimpleAssignmentTarget`] group.
#[macro_export]
macro_rules! match_simple_assignment_target {
    ($ty:ident) => {
        $ty::IdentifierReference(_)
            | $ty::MemberExpression(_)
            | $ty::TsAsExpression(_)
            | $ty::TsSatisfiesExpression(_)
            | $ty::TsTypeAssertion(_)
            | $ty::TsNonNullExpression(_)
    };
}
pub use match_simple_assignment_target;
/// Matches all variants of the [`TsType`] group.
#[macro_export]
macro_rules! match_ts_type {
    ($ty:ident) => {
        $ty::TsAnyKeyword(_)
            | $ty::TsUnknownKeyword(_)
            | $ty::TsNeverKeyword(_)
            | $ty::TsVoidKeyword(_)
            | $ty::TsNullKeyword(_)
            | $ty::TsUndefinedKeyword(_)
            | $ty::TsStringKeyword(_)
            | $ty::TsNumberKeyword(_)
            | $ty::TsBigIntKeyword(_)
            | $ty::TsBooleanKeyword(_)
            | $ty::TsSymbolKeyword(_)
            | $ty::TsObjectKeyword(_)
            | $ty::TsIntrinsicKeyword(_)
            | $ty::TsThisType(_)
            | $ty::TsTypeReference(_)
            | $ty::TsTypeQuery(_)
            | $ty::TsImportType(_)
            | $ty::TsLiteralType(_)
            | $ty::TsTemplateLiteralType(_)
            | $ty::TsArrayType(_)
            | $ty::TsIndexedAccessType(_)
            | $ty::TsTupleType(_)
            | $ty::TsOptionalType(_)
            | $ty::TsRestType(_)
            | $ty::TsJsDocNullableType(_)
            | $ty::TsJsDocNonNullableType(_)
            | $ty::TsJsDocUnknownType(_)
            | $ty::TsUnionType(_)
            | $ty::TsIntersectionType(_)
            | $ty::TsConditionalType(_)
            | $ty::TsInferType(_)
            | $ty::TsTypeOperator(_)
            | $ty::TsParenthesizedType(_)
            | $ty::TsFunctionType(_)
            | $ty::TsConstructorType(_)
            | $ty::TsTypePredicate(_)
            | $ty::TsTypeLiteral(_)
            | $ty::TsMappedType(_)
    };
}
pub use match_ts_type;
/// Matches all variants of the [`TsTypeName`] group.
#[macro_export]
macro_rules! match_ts_type_name {
    ($ty:ident) => {
        $ty::IdentifierReference(_) | $ty::TsQualifiedName(_)
    };
}
pub use match_ts_type_name;
