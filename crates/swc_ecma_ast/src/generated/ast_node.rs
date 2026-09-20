#![allow(unused)]
use crate::node_id::sealed::AstNode as SealedAstNode;
use crate::{Ast, GetSpan, SetSpan};
use crate::{ast::*, node_id::*};
/// The concrete type stored at a node ID.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Program,
    Super,
    ThisExpression,
    StringLiteral,
    NumericLiteral,
    BigIntLiteral,
    BooleanLiteral,
    NullLiteral,
    RegExpLiteral,
    TemplateLiteral,
    TemplateElement,
    IdentifierReference,
    PrivateIdentifier,
    BindingIdentifier,
    IdentifierName,
    LabelIdentifier,
    AssignmentPattern,
    BindingRestElement,
    ArrayPattern,
    ObjectPattern,
    BindingProperty,
    BinaryExpression,
    LogicalExpression,
    ConditionalExpression,
    UnaryExpression,
    UpdateExpression,
    AssignmentExpression,
    VariableDeclaration,
    VariableDeclarator,
    IfStatement,
    SwitchStatement,
    SwitchCase,
    ForStatement,
    ForInStatement,
    ForOfStatement,
    TryStatement,
    CatchClause,
    BreakStatement,
    ContinueStatement,
    LabeledStatement,
    ExpressionStatement,
    ReturnStatement,
    ThrowStatement,
    DebuggerStatement,
    EmptyStatement,
    Directive,
    Decorator,
    Class,
    ClassBody,
    MethodDefinition,
    PropertyDefinition,
    StaticBlock,
    Function,
    FunctionBody,
    BlockStatement,
    WhileStatement,
    DoWhileStatement,
    WithStatement,
    FormalParameters,
    FormalParameter,
    ArrayExpression,
    ObjectExpression,
    SpreadElement,
    ObjectProperty,
    ParenthesizedExpression,
    ArrowFunctionExpression,
    SequenceExpression,
    MemberExpression,
    CallExpression,
    ChainExpression,
    TaggedTemplateExpression,
    NewExpression,
    AwaitExpression,
    YieldExpression,
    MetaProperty,
    ImportExpression,
    ImportDeclaration,
    ImportSpecifier,
    ImportDefaultSpecifier,
    ImportNamespaceSpecifier,
    ImportAttribute,
    ExportNamedDeclaration,
    ExportDefaultDeclaration,
    ExportAllDeclaration,
    ExportSpecifier,
    Hashbang,
    TsFunction,
    TsMethodDefinition,
    TsTypeAnnotation,
    TsAnyKeyword,
    TsUnknownKeyword,
    TsNeverKeyword,
    TsVoidKeyword,
    TsNullKeyword,
    TsUndefinedKeyword,
    TsStringKeyword,
    TsNumberKeyword,
    TsBigIntKeyword,
    TsBooleanKeyword,
    TsSymbolKeyword,
    TsObjectKeyword,
    TsIntrinsicKeyword,
    TsThisType,
    TsTypeReference,
    TsQualifiedName,
    TsTypeQuery,
    TsImportType,
    TsTypeParameter,
    TsTypeParameterDeclaration,
    TsTypeParameterInstantiation,
    TsLiteralType,
    TsTemplateLiteralType,
    TsArrayType,
    TsIndexedAccessType,
    TsTupleType,
    TsNamedTupleMember,
    TsOptionalType,
    TsRestType,
    TsJsDocNullableType,
    TsJsDocNonNullableType,
    TsJsDocUnknownType,
    TsUnionType,
    TsIntersectionType,
    TsConditionalType,
    TsInferType,
    TsTypeOperator,
    TsParenthesizedType,
    TsFunctionType,
    TsConstructorType,
    TsTypePredicate,
    TsTypeLiteral,
    TsMappedType,
    TsPropertySignature,
    TsMethodSignature,
    TsCallSignatureDeclaration,
    TsConstructSignatureDeclaration,
    TsIndexSignature,
    TsTypeAliasDeclaration,
    TsInterfaceDeclaration,
    TsInterfaceBody,
    TsInterfaceHeritage,
    TsClassImplements,
    TsEnumDeclaration,
    TsEnumBody,
    TsEnumMember,
    TsModuleDeclaration,
    TsModuleBlock,
    TsGlobalDeclaration,
    TsParameterProperty,
    TsThisParameter,
    TsAsExpression,
    TsSatisfiesExpression,
    TsTypeAssertion,
    TsNonNullExpression,
    TsInstantiationExpression,
    TsExportAssignment,
    TsNamespaceExportDeclaration,
    TsImportEqualsDeclaration,
    TsExternalModuleReference,
    JsxElement,
    JsxOpeningElement,
    JsxClosingElement,
    JsxFragment,
    JsxOpeningFragment,
    JsxClosingFragment,
    JsxIdentifier,
    JsxNamespacedName,
    JsxMemberExpression,
    JsxAttribute,
    JsxSpreadAttribute,
    JsxExpressionContainer,
    JsxEmptyExpression,
    JsxText,
    JsxSpreadChild,
}
impl NodeKind {
    pub const COUNT: u8 = 174u8;
    #[inline]
    pub const fn is_stmt(self) -> bool {
        matches!(
            self,
            Self::BlockStatement
                | Self::BreakStatement
                | Self::ContinueStatement
                | Self::DebuggerStatement
                | Self::DoWhileStatement
                | Self::EmptyStatement
                | Self::ExpressionStatement
                | Self::ForInStatement
                | Self::ForOfStatement
                | Self::ForStatement
                | Self::IfStatement
                | Self::LabeledStatement
                | Self::ReturnStatement
                | Self::SwitchStatement
                | Self::ThrowStatement
                | Self::TryStatement
                | Self::WhileStatement
                | Self::WithStatement
                | Self::VariableDeclaration
                | Self::Function
                | Self::Class
                | Self::TsFunction
                | Self::TsTypeAliasDeclaration
                | Self::TsInterfaceDeclaration
                | Self::TsEnumDeclaration
                | Self::TsModuleDeclaration
                | Self::TsGlobalDeclaration
                | Self::TsImportEqualsDeclaration
                | Self::ImportDeclaration
                | Self::ExportNamedDeclaration
                | Self::ExportDefaultDeclaration
                | Self::ExportAllDeclaration
                | Self::TsExportAssignment
                | Self::TsNamespaceExportDeclaration
        )
    }
    #[inline]
    pub const fn is_decl(self) -> bool {
        matches!(
            self,
            Self::VariableDeclaration
                | Self::Function
                | Self::Class
                | Self::TsFunction
                | Self::TsTypeAliasDeclaration
                | Self::TsInterfaceDeclaration
                | Self::TsEnumDeclaration
                | Self::TsModuleDeclaration
                | Self::TsGlobalDeclaration
                | Self::TsImportEqualsDeclaration
        )
    }
    #[inline]
    pub const fn is_expr(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
                | Self::TemplateLiteral
                | Self::BinaryExpression
                | Self::LogicalExpression
                | Self::UnaryExpression
                | Self::UpdateExpression
                | Self::AssignmentExpression
                | Self::ConditionalExpression
                | Self::SequenceExpression
                | Self::MemberExpression
                | Self::CallExpression
                | Self::ChainExpression
                | Self::NewExpression
                | Self::TaggedTemplateExpression
                | Self::ArrowFunctionExpression
                | Self::ArrayExpression
                | Self::ObjectExpression
                | Self::ParenthesizedExpression
                | Self::Function
                | Self::Class
                | Self::ImportExpression
                | Self::AwaitExpression
                | Self::YieldExpression
                | Self::PrivateIdentifier
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::TsInstantiationExpression
                | Self::JsxElement
                | Self::JsxFragment
        )
    }
    #[inline]
    pub const fn is_binding_pattern(self) -> bool {
        matches!(
            self,
            Self::BindingIdentifier
                | Self::IdentifierReference
                | Self::MemberExpression
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::AssignmentPattern
                | Self::BindingRestElement
                | Self::ArrayPattern
                | Self::ObjectPattern
        )
    }
    #[inline]
    pub const fn is_assignment_target(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::MemberExpression
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::ArrayPattern
                | Self::ObjectPattern
        )
    }
    #[inline]
    pub const fn is_simple_assignment_target(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::MemberExpression
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
        )
    }
    #[inline]
    pub const fn is_property_key(self) -> bool {
        matches!(
            self,
            Self::IdentifierName
                | Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
                | Self::TemplateLiteral
                | Self::BinaryExpression
                | Self::LogicalExpression
                | Self::UnaryExpression
                | Self::UpdateExpression
                | Self::AssignmentExpression
                | Self::ConditionalExpression
                | Self::SequenceExpression
                | Self::MemberExpression
                | Self::CallExpression
                | Self::ChainExpression
                | Self::NewExpression
                | Self::TaggedTemplateExpression
                | Self::ArrowFunctionExpression
                | Self::ArrayExpression
                | Self::ObjectExpression
                | Self::ParenthesizedExpression
                | Self::Function
                | Self::Class
                | Self::ImportExpression
                | Self::AwaitExpression
                | Self::YieldExpression
                | Self::PrivateIdentifier
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::TsInstantiationExpression
                | Self::JsxElement
                | Self::JsxFragment
        )
    }
    #[inline]
    pub const fn is_module_export_name(self) -> bool {
        matches!(self, Self::IdentifierName | Self::StringLiteral)
    }
    #[inline]
    pub const fn is_argument(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
                | Self::TemplateLiteral
                | Self::BinaryExpression
                | Self::LogicalExpression
                | Self::UnaryExpression
                | Self::UpdateExpression
                | Self::AssignmentExpression
                | Self::ConditionalExpression
                | Self::SequenceExpression
                | Self::MemberExpression
                | Self::CallExpression
                | Self::ChainExpression
                | Self::NewExpression
                | Self::TaggedTemplateExpression
                | Self::ArrowFunctionExpression
                | Self::ArrayExpression
                | Self::ObjectExpression
                | Self::ParenthesizedExpression
                | Self::Function
                | Self::Class
                | Self::ImportExpression
                | Self::AwaitExpression
                | Self::YieldExpression
                | Self::PrivateIdentifier
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::TsInstantiationExpression
                | Self::JsxElement
                | Self::JsxFragment
                | Self::SpreadElement
        )
    }
    #[inline]
    pub const fn is_object_property_kind(self) -> bool {
        matches!(self, Self::ObjectProperty | Self::SpreadElement)
    }
    #[inline]
    pub const fn is_for_statement_init(self) -> bool {
        matches!(
            self,
            Self::VariableDeclaration
                | Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
                | Self::TemplateLiteral
                | Self::BinaryExpression
                | Self::LogicalExpression
                | Self::UnaryExpression
                | Self::UpdateExpression
                | Self::AssignmentExpression
                | Self::ConditionalExpression
                | Self::SequenceExpression
                | Self::MemberExpression
                | Self::CallExpression
                | Self::ChainExpression
                | Self::NewExpression
                | Self::TaggedTemplateExpression
                | Self::ArrowFunctionExpression
                | Self::ArrayExpression
                | Self::ObjectExpression
                | Self::ParenthesizedExpression
                | Self::Function
                | Self::Class
                | Self::ImportExpression
                | Self::AwaitExpression
                | Self::YieldExpression
                | Self::PrivateIdentifier
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::TsInstantiationExpression
                | Self::JsxElement
                | Self::JsxFragment
        )
    }
    #[inline]
    pub const fn is_for_statement_left(self) -> bool {
        matches!(
            self,
            Self::VariableDeclaration
                | Self::IdentifierReference
                | Self::MemberExpression
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::ArrayPattern
                | Self::ObjectPattern
        )
    }
    #[inline]
    pub const fn is_arrow_function_body(self) -> bool {
        matches!(
            self,
            Self::FunctionBody
                | Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
                | Self::TemplateLiteral
                | Self::BinaryExpression
                | Self::LogicalExpression
                | Self::UnaryExpression
                | Self::UpdateExpression
                | Self::AssignmentExpression
                | Self::ConditionalExpression
                | Self::SequenceExpression
                | Self::MemberExpression
                | Self::CallExpression
                | Self::ChainExpression
                | Self::NewExpression
                | Self::TaggedTemplateExpression
                | Self::ArrowFunctionExpression
                | Self::ArrayExpression
                | Self::ObjectExpression
                | Self::ParenthesizedExpression
                | Self::Function
                | Self::Class
                | Self::ImportExpression
                | Self::AwaitExpression
                | Self::YieldExpression
                | Self::PrivateIdentifier
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::TsInstantiationExpression
                | Self::JsxElement
                | Self::JsxFragment
        )
    }
    #[inline]
    pub const fn is_formal_parameter_item(self) -> bool {
        matches!(self, Self::FormalParameter | Self::TsParameterProperty)
    }
    #[inline]
    pub const fn is_formal_parameter_pattern(self) -> bool {
        matches!(
            self,
            Self::BindingIdentifier
                | Self::IdentifierReference
                | Self::MemberExpression
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::AssignmentPattern
                | Self::BindingRestElement
                | Self::ArrayPattern
                | Self::ObjectPattern
                | Self::TsThisParameter
        )
    }
    #[inline]
    pub const fn is_class_element(self) -> bool {
        matches!(
            self,
            Self::MethodDefinition
                | Self::PropertyDefinition
                | Self::StaticBlock
                | Self::TsMethodDefinition
                | Self::TsIndexSignature
        )
    }
    #[inline]
    pub const fn is_import_declaration_specifier(self) -> bool {
        matches!(
            self,
            Self::ImportSpecifier | Self::ImportDefaultSpecifier | Self::ImportNamespaceSpecifier
        )
    }
    #[inline]
    pub const fn is_export_default_declaration_kind(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
                | Self::TemplateLiteral
                | Self::BinaryExpression
                | Self::LogicalExpression
                | Self::UnaryExpression
                | Self::UpdateExpression
                | Self::AssignmentExpression
                | Self::ConditionalExpression
                | Self::SequenceExpression
                | Self::MemberExpression
                | Self::CallExpression
                | Self::ChainExpression
                | Self::NewExpression
                | Self::TaggedTemplateExpression
                | Self::ArrowFunctionExpression
                | Self::ArrayExpression
                | Self::ObjectExpression
                | Self::ParenthesizedExpression
                | Self::Function
                | Self::Class
                | Self::ImportExpression
                | Self::AwaitExpression
                | Self::YieldExpression
                | Self::PrivateIdentifier
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::TsInstantiationExpression
                | Self::JsxElement
                | Self::JsxFragment
                | Self::TsFunction
                | Self::TsInterfaceDeclaration
        )
    }
    #[inline]
    pub const fn is_ts_type(self) -> bool {
        matches!(
            self,
            Self::TsAnyKeyword
                | Self::TsUnknownKeyword
                | Self::TsNeverKeyword
                | Self::TsVoidKeyword
                | Self::TsNullKeyword
                | Self::TsUndefinedKeyword
                | Self::TsStringKeyword
                | Self::TsNumberKeyword
                | Self::TsBigIntKeyword
                | Self::TsBooleanKeyword
                | Self::TsSymbolKeyword
                | Self::TsObjectKeyword
                | Self::TsIntrinsicKeyword
                | Self::TsThisType
                | Self::TsTypeReference
                | Self::TsTypeQuery
                | Self::TsImportType
                | Self::TsLiteralType
                | Self::TsTemplateLiteralType
                | Self::TsArrayType
                | Self::TsIndexedAccessType
                | Self::TsTupleType
                | Self::TsOptionalType
                | Self::TsRestType
                | Self::TsJsDocNullableType
                | Self::TsJsDocNonNullableType
                | Self::TsJsDocUnknownType
                | Self::TsUnionType
                | Self::TsIntersectionType
                | Self::TsConditionalType
                | Self::TsInferType
                | Self::TsTypeOperator
                | Self::TsParenthesizedType
                | Self::TsFunctionType
                | Self::TsConstructorType
                | Self::TsTypePredicate
                | Self::TsTypeLiteral
                | Self::TsMappedType
        )
    }
    #[inline]
    pub const fn is_ts_type_name(self) -> bool {
        matches!(self, Self::IdentifierReference | Self::TsQualifiedName)
    }
    #[inline]
    pub const fn is_ts_qualified_name_left(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::IdentifierName
                | Self::BindingIdentifier
                | Self::ThisExpression
                | Self::TsQualifiedName
        )
    }
    #[inline]
    pub const fn is_ts_import_type_qualifier(self) -> bool {
        matches!(self, Self::IdentifierName | Self::TsQualifiedName)
    }
    #[inline]
    pub const fn is_ts_type_query_expr_name(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::TsQualifiedName
                | Self::ThisExpression
                | Self::TsImportType
        )
    }
    #[inline]
    pub const fn is_ts_literal(self) -> bool {
        matches!(
            self,
            Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::TemplateLiteral
                | Self::UnaryExpression
        )
    }
    #[inline]
    pub const fn is_ts_tuple_element(self) -> bool {
        matches!(
            self,
            Self::TsAnyKeyword
                | Self::TsUnknownKeyword
                | Self::TsNeverKeyword
                | Self::TsVoidKeyword
                | Self::TsNullKeyword
                | Self::TsUndefinedKeyword
                | Self::TsStringKeyword
                | Self::TsNumberKeyword
                | Self::TsBigIntKeyword
                | Self::TsBooleanKeyword
                | Self::TsSymbolKeyword
                | Self::TsObjectKeyword
                | Self::TsIntrinsicKeyword
                | Self::TsThisType
                | Self::TsTypeReference
                | Self::TsTypeQuery
                | Self::TsImportType
                | Self::TsLiteralType
                | Self::TsTemplateLiteralType
                | Self::TsArrayType
                | Self::TsIndexedAccessType
                | Self::TsTupleType
                | Self::TsOptionalType
                | Self::TsRestType
                | Self::TsJsDocNullableType
                | Self::TsJsDocNonNullableType
                | Self::TsJsDocUnknownType
                | Self::TsUnionType
                | Self::TsIntersectionType
                | Self::TsConditionalType
                | Self::TsInferType
                | Self::TsTypeOperator
                | Self::TsParenthesizedType
                | Self::TsFunctionType
                | Self::TsConstructorType
                | Self::TsTypePredicate
                | Self::TsTypeLiteral
                | Self::TsMappedType
                | Self::TsNamedTupleMember
        )
    }
    #[inline]
    pub const fn is_ts_type_element(self) -> bool {
        matches!(
            self,
            Self::TsPropertySignature
                | Self::TsMethodSignature
                | Self::TsCallSignatureDeclaration
                | Self::TsConstructSignatureDeclaration
                | Self::TsIndexSignature
        )
    }
    #[inline]
    pub const fn is_ts_type_predicate_name(self) -> bool {
        matches!(self, Self::IdentifierName | Self::TsThisType)
    }
    #[inline]
    pub const fn is_ts_enum_member_name(self) -> bool {
        matches!(
            self,
            Self::IdentifierName | Self::StringLiteral | Self::TemplateLiteral
        )
    }
    #[inline]
    pub const fn is_ts_module_name(self) -> bool {
        matches!(
            self,
            Self::BindingIdentifier | Self::StringLiteral | Self::TsQualifiedName
        )
    }
    #[inline]
    pub const fn is_ts_module_body(self) -> bool {
        matches!(self, Self::TsModuleBlock | Self::TsModuleDeclaration)
    }
    #[inline]
    pub const fn is_ts_module_reference(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference | Self::TsQualifiedName | Self::TsExternalModuleReference
        )
    }
    #[inline]
    pub const fn is_jsx_element_name(self) -> bool {
        matches!(
            self,
            Self::JsxIdentifier | Self::JsxNamespacedName | Self::JsxMemberExpression
        )
    }
    #[inline]
    pub const fn is_jsx_member_expression_object(self) -> bool {
        matches!(self, Self::JsxIdentifier | Self::JsxMemberExpression)
    }
    #[inline]
    pub const fn is_jsx_attribute_name(self) -> bool {
        matches!(self, Self::JsxIdentifier | Self::JsxNamespacedName)
    }
    #[inline]
    pub const fn is_jsx_attribute_item(self) -> bool {
        matches!(self, Self::JsxAttribute | Self::JsxSpreadAttribute)
    }
    #[inline]
    pub const fn is_jsx_attribute_value(self) -> bool {
        matches!(
            self,
            Self::StringLiteral
                | Self::JsxExpressionContainer
                | Self::JsxElement
                | Self::JsxFragment
        )
    }
    #[inline]
    pub const fn is_jsx_expression(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
                | Self::TemplateLiteral
                | Self::BinaryExpression
                | Self::LogicalExpression
                | Self::UnaryExpression
                | Self::UpdateExpression
                | Self::AssignmentExpression
                | Self::ConditionalExpression
                | Self::SequenceExpression
                | Self::MemberExpression
                | Self::CallExpression
                | Self::ChainExpression
                | Self::NewExpression
                | Self::TaggedTemplateExpression
                | Self::ArrowFunctionExpression
                | Self::ArrayExpression
                | Self::ObjectExpression
                | Self::ParenthesizedExpression
                | Self::Function
                | Self::Class
                | Self::ImportExpression
                | Self::AwaitExpression
                | Self::YieldExpression
                | Self::PrivateIdentifier
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::TsInstantiationExpression
                | Self::JsxElement
                | Self::JsxFragment
                | Self::JsxEmptyExpression
        )
    }
    #[inline]
    pub const fn is_jsx_child(self) -> bool {
        matches!(
            self,
            Self::JsxText
                | Self::JsxExpressionContainer
                | Self::JsxSpreadChild
                | Self::JsxElement
                | Self::JsxFragment
        )
    }
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StmtData {
    BlockStatement(BlockStatement) = 1u32,
    BreakStatement(BreakStatement) = 2u32,
    ContinueStatement(ContinueStatement) = 3u32,
    DebuggerStatement(DebuggerStatement) = 4u32,
    DoWhileStatement(DoWhileStatement) = 5u32,
    EmptyStatement(EmptyStatement) = 6u32,
    ExpressionStatement(ExpressionStatement) = 7u32,
    ForInStatement(ForInStatement) = 8u32,
    ForOfStatement(ForOfStatement) = 9u32,
    ForStatement(ForStatement) = 10u32,
    IfStatement(IfStatement) = 11u32,
    LabeledStatement(LabeledStatement) = 12u32,
    ReturnStatement(ReturnStatement) = 13u32,
    SwitchStatement(SwitchStatement) = 14u32,
    ThrowStatement(ThrowStatement) = 15u32,
    TryStatement(TryStatement) = 16u32,
    WhileStatement(WhileStatement) = 17u32,
    WithStatement(WithStatement) = 18u32,
    Declaration(Decl) = 19u32,
    ImportDeclaration(ImportDeclaration) = 20u32,
    ExportNamedDeclaration(ExportNamedDeclaration) = 21u32,
    ExportDefaultDeclaration(ExportDefaultDeclaration) = 22u32,
    ExportAllDeclaration(ExportAllDeclaration) = 23u32,
    TsExportAssignment(TsExportAssignment) = 24u32,
    TsNamespaceExportDeclaration(TsNamespaceExportDeclaration) = 25u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DeclData {
    VariableDeclaration(VariableDeclaration) = NodeKind::VariableDeclaration as u8,
    Function(Function) = NodeKind::Function as u8,
    Class(Class) = NodeKind::Class as u8,
    TsFunction(TsFunction) = NodeKind::TsFunction as u8,
    TsTypeAliasDeclaration(TsTypeAliasDeclaration) = NodeKind::TsTypeAliasDeclaration as u8,
    TsInterfaceDeclaration(TsInterfaceDeclaration) = NodeKind::TsInterfaceDeclaration as u8,
    TsEnumDeclaration(TsEnumDeclaration) = NodeKind::TsEnumDeclaration as u8,
    TsModuleDeclaration(TsModuleDeclaration) = NodeKind::TsModuleDeclaration as u8,
    TsGlobalDeclaration(TsGlobalDeclaration) = NodeKind::TsGlobalDeclaration as u8,
    TsImportEqualsDeclaration(TsImportEqualsDeclaration) =
        NodeKind::TsImportEqualsDeclaration as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExprData {
    IdentifierReference(IdentifierReference) = NodeKind::IdentifierReference as u8,
    ThisExpression(ThisExpression) = NodeKind::ThisExpression as u8,
    Super(Super) = NodeKind::Super as u8,
    MetaProperty(MetaProperty) = NodeKind::MetaProperty as u8,
    StringLiteral(StringLiteral) = NodeKind::StringLiteral as u8,
    NumericLiteral(NumericLiteral) = NodeKind::NumericLiteral as u8,
    BigIntLiteral(BigIntLiteral) = NodeKind::BigIntLiteral as u8,
    BooleanLiteral(BooleanLiteral) = NodeKind::BooleanLiteral as u8,
    NullLiteral(NullLiteral) = NodeKind::NullLiteral as u8,
    RegExpLiteral(RegExpLiteral) = NodeKind::RegExpLiteral as u8,
    TemplateLiteral(TemplateLiteral) = NodeKind::TemplateLiteral as u8,
    BinaryExpression(BinaryExpression) = NodeKind::BinaryExpression as u8,
    LogicalExpression(LogicalExpression) = NodeKind::LogicalExpression as u8,
    UnaryExpression(UnaryExpression) = NodeKind::UnaryExpression as u8,
    UpdateExpression(UpdateExpression) = NodeKind::UpdateExpression as u8,
    AssignmentExpression(AssignmentExpression) = NodeKind::AssignmentExpression as u8,
    ConditionalExpression(ConditionalExpression) = NodeKind::ConditionalExpression as u8,
    SequenceExpression(SequenceExpression) = NodeKind::SequenceExpression as u8,
    MemberExpression(MemberExpression) = NodeKind::MemberExpression as u8,
    CallExpression(CallExpression) = NodeKind::CallExpression as u8,
    ChainExpression(ChainExpression) = NodeKind::ChainExpression as u8,
    NewExpression(NewExpression) = NodeKind::NewExpression as u8,
    TaggedTemplateExpression(TaggedTemplateExpression) = NodeKind::TaggedTemplateExpression as u8,
    ArrowFunctionExpression(ArrowFunctionExpression) = NodeKind::ArrowFunctionExpression as u8,
    ArrayExpression(ArrayExpression) = NodeKind::ArrayExpression as u8,
    ObjectExpression(ObjectExpression) = NodeKind::ObjectExpression as u8,
    ParenthesizedExpression(ParenthesizedExpression) = NodeKind::ParenthesizedExpression as u8,
    Function(Function) = NodeKind::Function as u8,
    Class(Class) = NodeKind::Class as u8,
    ImportExpression(ImportExpression) = NodeKind::ImportExpression as u8,
    AwaitExpression(AwaitExpression) = NodeKind::AwaitExpression as u8,
    YieldExpression(YieldExpression) = NodeKind::YieldExpression as u8,
    PrivateIdentifier(PrivateIdentifier) = NodeKind::PrivateIdentifier as u8,
    TsAsExpression(TsAsExpression) = NodeKind::TsAsExpression as u8,
    TsSatisfiesExpression(TsSatisfiesExpression) = NodeKind::TsSatisfiesExpression as u8,
    TsTypeAssertion(TsTypeAssertion) = NodeKind::TsTypeAssertion as u8,
    TsNonNullExpression(TsNonNullExpression) = NodeKind::TsNonNullExpression as u8,
    TsInstantiationExpression(TsInstantiationExpression) =
        NodeKind::TsInstantiationExpression as u8,
    JsxElement(JsxElement) = NodeKind::JsxElement as u8,
    JsxFragment(JsxFragment) = NodeKind::JsxFragment as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BindingPatternData {
    BindingIdentifier(BindingIdentifier) = 1u32,
    SimpleAssignmentTarget(SimpleAssignmentTarget) = 2u32,
    AssignmentPattern(AssignmentPattern) = 3u32,
    BindingRestElement(BindingRestElement) = 4u32,
    ArrayPattern(ArrayPattern) = 5u32,
    ObjectPattern(ObjectPattern) = 6u32,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AssignmentTargetData {
    SimpleAssignmentTarget(SimpleAssignmentTarget) = 1u32,
    ArrayAssignmentTarget(ArrayPattern) = 2u32,
    ObjectAssignmentTarget(ObjectPattern) = 3u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SimpleAssignmentTargetData {
    IdentifierReference(IdentifierReference) = NodeKind::IdentifierReference as u8,
    MemberExpression(MemberExpression) = NodeKind::MemberExpression as u8,
    TsAsExpression(TsAsExpression) = NodeKind::TsAsExpression as u8,
    TsSatisfiesExpression(TsSatisfiesExpression) = NodeKind::TsSatisfiesExpression as u8,
    TsTypeAssertion(TsTypeAssertion) = NodeKind::TsTypeAssertion as u8,
    TsNonNullExpression(TsNonNullExpression) = NodeKind::TsNonNullExpression as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PropertyKeyData {
    IdentifierName(IdentifierName) = 1u32,
    Expr(Expr) = 2u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ModuleExportNameData {
    IdentifierName(IdentifierName) = NodeKind::IdentifierName as u8,
    StringLiteral(StringLiteral) = NodeKind::StringLiteral as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ArgumentData {
    Expr(Expr) = 1u32,
    SpreadElement(SpreadElement) = 2u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ObjectPropertyKindData {
    ObjectProperty(ObjectProperty) = NodeKind::ObjectProperty as u8,
    SpreadElement(SpreadElement) = NodeKind::SpreadElement as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ForStatementInitData {
    VariableDeclaration(VariableDeclaration) = 1u32,
    Expr(Expr) = 2u32,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ForStatementLeftData {
    VariableDeclaration(VariableDeclaration) = 1u32,
    AssignmentTarget(AssignmentTarget) = 2u32,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ArrowFunctionBodyData {
    FunctionBody(FunctionBody) = 1u32,
    Expr(Expr) = 2u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FormalParameterItemData {
    FormalParameter(FormalParameter) = NodeKind::FormalParameter as u8,
    TsParameterProperty(TsParameterProperty) = NodeKind::TsParameterProperty as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FormalParameterPatternData {
    BindingPattern(BindingPattern) = 1u32,
    TsThisParameter(TsThisParameter) = 2u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ClassElementData {
    MethodDefinition(MethodDefinition) = NodeKind::MethodDefinition as u8,
    PropertyDefinition(PropertyDefinition) = NodeKind::PropertyDefinition as u8,
    StaticBlock(StaticBlock) = NodeKind::StaticBlock as u8,
    TsMethodDefinition(TsMethodDefinition) = NodeKind::TsMethodDefinition as u8,
    TsIndexSignature(TsIndexSignature) = NodeKind::TsIndexSignature as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ImportDeclarationSpecifierData {
    ImportSpecifier(ImportSpecifier) = NodeKind::ImportSpecifier as u8,
    ImportDefaultSpecifier(ImportDefaultSpecifier) = NodeKind::ImportDefaultSpecifier as u8,
    ImportNamespaceSpecifier(ImportNamespaceSpecifier) = NodeKind::ImportNamespaceSpecifier as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExportDefaultDeclarationKindData {
    Expr(Expr) = 1u32,
    Function(Function) = 2u32,
    Class(Class) = 3u32,
    TsFunction(TsFunction) = 4u32,
    TsInterfaceDeclaration(TsInterfaceDeclaration) = 5u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsTypeData {
    TsAnyKeyword(TsAnyKeyword) = NodeKind::TsAnyKeyword as u8,
    TsUnknownKeyword(TsUnknownKeyword) = NodeKind::TsUnknownKeyword as u8,
    TsNeverKeyword(TsNeverKeyword) = NodeKind::TsNeverKeyword as u8,
    TsVoidKeyword(TsVoidKeyword) = NodeKind::TsVoidKeyword as u8,
    TsNullKeyword(TsNullKeyword) = NodeKind::TsNullKeyword as u8,
    TsUndefinedKeyword(TsUndefinedKeyword) = NodeKind::TsUndefinedKeyword as u8,
    TsStringKeyword(TsStringKeyword) = NodeKind::TsStringKeyword as u8,
    TsNumberKeyword(TsNumberKeyword) = NodeKind::TsNumberKeyword as u8,
    TsBigIntKeyword(TsBigIntKeyword) = NodeKind::TsBigIntKeyword as u8,
    TsBooleanKeyword(TsBooleanKeyword) = NodeKind::TsBooleanKeyword as u8,
    TsSymbolKeyword(TsSymbolKeyword) = NodeKind::TsSymbolKeyword as u8,
    TsObjectKeyword(TsObjectKeyword) = NodeKind::TsObjectKeyword as u8,
    TsIntrinsicKeyword(TsIntrinsicKeyword) = NodeKind::TsIntrinsicKeyword as u8,
    TsThisType(TsThisType) = NodeKind::TsThisType as u8,
    TsTypeReference(TsTypeReference) = NodeKind::TsTypeReference as u8,
    TsTypeQuery(TsTypeQuery) = NodeKind::TsTypeQuery as u8,
    TsImportType(TsImportType) = NodeKind::TsImportType as u8,
    TsLiteralType(TsLiteralType) = NodeKind::TsLiteralType as u8,
    TsTemplateLiteralType(TsTemplateLiteralType) = NodeKind::TsTemplateLiteralType as u8,
    TsArrayType(TsArrayType) = NodeKind::TsArrayType as u8,
    TsIndexedAccessType(TsIndexedAccessType) = NodeKind::TsIndexedAccessType as u8,
    TsTupleType(TsTupleType) = NodeKind::TsTupleType as u8,
    TsOptionalType(TsOptionalType) = NodeKind::TsOptionalType as u8,
    TsRestType(TsRestType) = NodeKind::TsRestType as u8,
    TsJsDocNullableType(TsJsDocNullableType) = NodeKind::TsJsDocNullableType as u8,
    TsJsDocNonNullableType(TsJsDocNonNullableType) = NodeKind::TsJsDocNonNullableType as u8,
    TsJsDocUnknownType(TsJsDocUnknownType) = NodeKind::TsJsDocUnknownType as u8,
    TsUnionType(TsUnionType) = NodeKind::TsUnionType as u8,
    TsIntersectionType(TsIntersectionType) = NodeKind::TsIntersectionType as u8,
    TsConditionalType(TsConditionalType) = NodeKind::TsConditionalType as u8,
    TsInferType(TsInferType) = NodeKind::TsInferType as u8,
    TsTypeOperator(TsTypeOperator) = NodeKind::TsTypeOperator as u8,
    TsParenthesizedType(TsParenthesizedType) = NodeKind::TsParenthesizedType as u8,
    TsFunctionType(TsFunctionType) = NodeKind::TsFunctionType as u8,
    TsConstructorType(TsConstructorType) = NodeKind::TsConstructorType as u8,
    TsTypePredicate(TsTypePredicate) = NodeKind::TsTypePredicate as u8,
    TsTypeLiteral(TsTypeLiteral) = NodeKind::TsTypeLiteral as u8,
    TsMappedType(TsMappedType) = NodeKind::TsMappedType as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsTypeNameData {
    IdentifierReference(IdentifierReference) = NodeKind::IdentifierReference as u8,
    TsQualifiedName(TsQualifiedName) = NodeKind::TsQualifiedName as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsQualifiedNameLeftData {
    IdentifierReference(IdentifierReference) = NodeKind::IdentifierReference as u8,
    IdentifierName(IdentifierName) = NodeKind::IdentifierName as u8,
    BindingIdentifier(BindingIdentifier) = NodeKind::BindingIdentifier as u8,
    ThisExpression(ThisExpression) = NodeKind::ThisExpression as u8,
    TsQualifiedName(TsQualifiedName) = NodeKind::TsQualifiedName as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsImportTypeQualifierData {
    IdentifierName(IdentifierName) = NodeKind::IdentifierName as u8,
    TsQualifiedName(TsQualifiedName) = NodeKind::TsQualifiedName as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsTypeQueryExprNameData {
    TsTypeName(TsTypeName) = 1u32,
    ThisExpression(ThisExpression) = 2u32,
    TsImportType(TsImportType) = 3u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsLiteralData {
    StringLiteral(StringLiteral) = NodeKind::StringLiteral as u8,
    NumericLiteral(NumericLiteral) = NodeKind::NumericLiteral as u8,
    BigIntLiteral(BigIntLiteral) = NodeKind::BigIntLiteral as u8,
    BooleanLiteral(BooleanLiteral) = NodeKind::BooleanLiteral as u8,
    TemplateLiteral(TemplateLiteral) = NodeKind::TemplateLiteral as u8,
    UnaryExpression(UnaryExpression) = NodeKind::UnaryExpression as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsTupleElementData {
    TsType(TsType) = 1u32,
    TsNamedTupleMember(TsNamedTupleMember) = 2u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsTypeElementData {
    TsPropertySignature(TsPropertySignature) = NodeKind::TsPropertySignature as u8,
    TsMethodSignature(TsMethodSignature) = NodeKind::TsMethodSignature as u8,
    TsCallSignatureDeclaration(TsCallSignatureDeclaration) =
        NodeKind::TsCallSignatureDeclaration as u8,
    TsConstructSignatureDeclaration(TsConstructSignatureDeclaration) =
        NodeKind::TsConstructSignatureDeclaration as u8,
    TsIndexSignature(TsIndexSignature) = NodeKind::TsIndexSignature as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsTypePredicateNameData {
    IdentifierName(IdentifierName) = NodeKind::IdentifierName as u8,
    TsThisType(TsThisType) = NodeKind::TsThisType as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsEnumMemberNameData {
    IdentifierName(IdentifierName) = NodeKind::IdentifierName as u8,
    StringLiteral(StringLiteral) = NodeKind::StringLiteral as u8,
    TemplateLiteral(TemplateLiteral) = NodeKind::TemplateLiteral as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsModuleNameData {
    BindingIdentifier(BindingIdentifier) = NodeKind::BindingIdentifier as u8,
    StringLiteral(StringLiteral) = NodeKind::StringLiteral as u8,
    TsQualifiedName(TsQualifiedName) = NodeKind::TsQualifiedName as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsModuleBodyData {
    TsModuleBlock(TsModuleBlock) = NodeKind::TsModuleBlock as u8,
    TsModuleDeclaration(TsModuleDeclaration) = NodeKind::TsModuleDeclaration as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsModuleReferenceData {
    TsTypeName(TsTypeName) = 1u32,
    TsExternalModuleReference(TsExternalModuleReference) = 2u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsxElementNameData {
    JsxIdentifier(JsxIdentifier) = NodeKind::JsxIdentifier as u8,
    JsxNamespacedName(JsxNamespacedName) = NodeKind::JsxNamespacedName as u8,
    JsxMemberExpression(JsxMemberExpression) = NodeKind::JsxMemberExpression as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsxMemberExpressionObjectData {
    JsxIdentifier(JsxIdentifier) = NodeKind::JsxIdentifier as u8,
    JsxMemberExpression(JsxMemberExpression) = NodeKind::JsxMemberExpression as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsxAttributeNameData {
    JsxIdentifier(JsxIdentifier) = NodeKind::JsxIdentifier as u8,
    JsxNamespacedName(JsxNamespacedName) = NodeKind::JsxNamespacedName as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsxAttributeItemData {
    JsxAttribute(JsxAttribute) = NodeKind::JsxAttribute as u8,
    JsxSpreadAttribute(JsxSpreadAttribute) = NodeKind::JsxSpreadAttribute as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsxAttributeValueData {
    StringLiteral(StringLiteral) = NodeKind::StringLiteral as u8,
    JsxExpressionContainer(JsxExpressionContainer) = NodeKind::JsxExpressionContainer as u8,
    JsxElement(JsxElement) = NodeKind::JsxElement as u8,
    JsxFragment(JsxFragment) = NodeKind::JsxFragment as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsxExpressionData {
    Expr(Expr) = 1u32,
    JsxEmptyExpression(JsxEmptyExpression) = 2u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsxChildData {
    JsxText(JsxText) = NodeKind::JsxText as u8,
    JsxExpressionContainer(JsxExpressionContainer) = NodeKind::JsxExpressionContainer as u8,
    JsxSpreadChild(JsxSpreadChild) = NodeKind::JsxSpreadChild as u8,
    JsxElement(JsxElement) = NodeKind::JsxElement as u8,
    JsxFragment(JsxFragment) = NodeKind::JsxFragment as u8,
}
#[repr(C)]
struct AstEnumDataRepr {
    kind: NodeKind,
    node_id: NodeId,
}
/// The typed wrapper for the concrete node stored at a node ID.
#[repr(C, u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKindData {
    Program(Program) = NodeKind::Program as u8,
    Super(Super) = NodeKind::Super as u8,
    ThisExpression(ThisExpression) = NodeKind::ThisExpression as u8,
    StringLiteral(StringLiteral) = NodeKind::StringLiteral as u8,
    NumericLiteral(NumericLiteral) = NodeKind::NumericLiteral as u8,
    BigIntLiteral(BigIntLiteral) = NodeKind::BigIntLiteral as u8,
    BooleanLiteral(BooleanLiteral) = NodeKind::BooleanLiteral as u8,
    NullLiteral(NullLiteral) = NodeKind::NullLiteral as u8,
    RegExpLiteral(RegExpLiteral) = NodeKind::RegExpLiteral as u8,
    TemplateLiteral(TemplateLiteral) = NodeKind::TemplateLiteral as u8,
    TemplateElement(TemplateElement) = NodeKind::TemplateElement as u8,
    IdentifierReference(IdentifierReference) = NodeKind::IdentifierReference as u8,
    PrivateIdentifier(PrivateIdentifier) = NodeKind::PrivateIdentifier as u8,
    BindingIdentifier(BindingIdentifier) = NodeKind::BindingIdentifier as u8,
    IdentifierName(IdentifierName) = NodeKind::IdentifierName as u8,
    LabelIdentifier(LabelIdentifier) = NodeKind::LabelIdentifier as u8,
    AssignmentPattern(AssignmentPattern) = NodeKind::AssignmentPattern as u8,
    BindingRestElement(BindingRestElement) = NodeKind::BindingRestElement as u8,
    ArrayPattern(ArrayPattern) = NodeKind::ArrayPattern as u8,
    ObjectPattern(ObjectPattern) = NodeKind::ObjectPattern as u8,
    BindingProperty(BindingProperty) = NodeKind::BindingProperty as u8,
    BinaryExpression(BinaryExpression) = NodeKind::BinaryExpression as u8,
    LogicalExpression(LogicalExpression) = NodeKind::LogicalExpression as u8,
    ConditionalExpression(ConditionalExpression) = NodeKind::ConditionalExpression as u8,
    UnaryExpression(UnaryExpression) = NodeKind::UnaryExpression as u8,
    UpdateExpression(UpdateExpression) = NodeKind::UpdateExpression as u8,
    AssignmentExpression(AssignmentExpression) = NodeKind::AssignmentExpression as u8,
    VariableDeclaration(VariableDeclaration) = NodeKind::VariableDeclaration as u8,
    VariableDeclarator(VariableDeclarator) = NodeKind::VariableDeclarator as u8,
    IfStatement(IfStatement) = NodeKind::IfStatement as u8,
    SwitchStatement(SwitchStatement) = NodeKind::SwitchStatement as u8,
    SwitchCase(SwitchCase) = NodeKind::SwitchCase as u8,
    ForStatement(ForStatement) = NodeKind::ForStatement as u8,
    ForInStatement(ForInStatement) = NodeKind::ForInStatement as u8,
    ForOfStatement(ForOfStatement) = NodeKind::ForOfStatement as u8,
    TryStatement(TryStatement) = NodeKind::TryStatement as u8,
    CatchClause(CatchClause) = NodeKind::CatchClause as u8,
    BreakStatement(BreakStatement) = NodeKind::BreakStatement as u8,
    ContinueStatement(ContinueStatement) = NodeKind::ContinueStatement as u8,
    LabeledStatement(LabeledStatement) = NodeKind::LabeledStatement as u8,
    ExpressionStatement(ExpressionStatement) = NodeKind::ExpressionStatement as u8,
    ReturnStatement(ReturnStatement) = NodeKind::ReturnStatement as u8,
    ThrowStatement(ThrowStatement) = NodeKind::ThrowStatement as u8,
    DebuggerStatement(DebuggerStatement) = NodeKind::DebuggerStatement as u8,
    EmptyStatement(EmptyStatement) = NodeKind::EmptyStatement as u8,
    Directive(Directive) = NodeKind::Directive as u8,
    Decorator(Decorator) = NodeKind::Decorator as u8,
    Class(Class) = NodeKind::Class as u8,
    ClassBody(ClassBody) = NodeKind::ClassBody as u8,
    MethodDefinition(MethodDefinition) = NodeKind::MethodDefinition as u8,
    PropertyDefinition(PropertyDefinition) = NodeKind::PropertyDefinition as u8,
    StaticBlock(StaticBlock) = NodeKind::StaticBlock as u8,
    Function(Function) = NodeKind::Function as u8,
    FunctionBody(FunctionBody) = NodeKind::FunctionBody as u8,
    BlockStatement(BlockStatement) = NodeKind::BlockStatement as u8,
    WhileStatement(WhileStatement) = NodeKind::WhileStatement as u8,
    DoWhileStatement(DoWhileStatement) = NodeKind::DoWhileStatement as u8,
    WithStatement(WithStatement) = NodeKind::WithStatement as u8,
    FormalParameters(FormalParameters) = NodeKind::FormalParameters as u8,
    FormalParameter(FormalParameter) = NodeKind::FormalParameter as u8,
    ArrayExpression(ArrayExpression) = NodeKind::ArrayExpression as u8,
    ObjectExpression(ObjectExpression) = NodeKind::ObjectExpression as u8,
    SpreadElement(SpreadElement) = NodeKind::SpreadElement as u8,
    ObjectProperty(ObjectProperty) = NodeKind::ObjectProperty as u8,
    ParenthesizedExpression(ParenthesizedExpression) = NodeKind::ParenthesizedExpression as u8,
    ArrowFunctionExpression(ArrowFunctionExpression) = NodeKind::ArrowFunctionExpression as u8,
    SequenceExpression(SequenceExpression) = NodeKind::SequenceExpression as u8,
    MemberExpression(MemberExpression) = NodeKind::MemberExpression as u8,
    CallExpression(CallExpression) = NodeKind::CallExpression as u8,
    ChainExpression(ChainExpression) = NodeKind::ChainExpression as u8,
    TaggedTemplateExpression(TaggedTemplateExpression) = NodeKind::TaggedTemplateExpression as u8,
    NewExpression(NewExpression) = NodeKind::NewExpression as u8,
    AwaitExpression(AwaitExpression) = NodeKind::AwaitExpression as u8,
    YieldExpression(YieldExpression) = NodeKind::YieldExpression as u8,
    MetaProperty(MetaProperty) = NodeKind::MetaProperty as u8,
    ImportExpression(ImportExpression) = NodeKind::ImportExpression as u8,
    ImportDeclaration(ImportDeclaration) = NodeKind::ImportDeclaration as u8,
    ImportSpecifier(ImportSpecifier) = NodeKind::ImportSpecifier as u8,
    ImportDefaultSpecifier(ImportDefaultSpecifier) = NodeKind::ImportDefaultSpecifier as u8,
    ImportNamespaceSpecifier(ImportNamespaceSpecifier) = NodeKind::ImportNamespaceSpecifier as u8,
    ImportAttribute(ImportAttribute) = NodeKind::ImportAttribute as u8,
    ExportNamedDeclaration(ExportNamedDeclaration) = NodeKind::ExportNamedDeclaration as u8,
    ExportDefaultDeclaration(ExportDefaultDeclaration) = NodeKind::ExportDefaultDeclaration as u8,
    ExportAllDeclaration(ExportAllDeclaration) = NodeKind::ExportAllDeclaration as u8,
    ExportSpecifier(ExportSpecifier) = NodeKind::ExportSpecifier as u8,
    Hashbang(Hashbang) = NodeKind::Hashbang as u8,
    TsFunction(TsFunction) = NodeKind::TsFunction as u8,
    TsMethodDefinition(TsMethodDefinition) = NodeKind::TsMethodDefinition as u8,
    TsTypeAnnotation(TsTypeAnnotation) = NodeKind::TsTypeAnnotation as u8,
    TsAnyKeyword(TsAnyKeyword) = NodeKind::TsAnyKeyword as u8,
    TsUnknownKeyword(TsUnknownKeyword) = NodeKind::TsUnknownKeyword as u8,
    TsNeverKeyword(TsNeverKeyword) = NodeKind::TsNeverKeyword as u8,
    TsVoidKeyword(TsVoidKeyword) = NodeKind::TsVoidKeyword as u8,
    TsNullKeyword(TsNullKeyword) = NodeKind::TsNullKeyword as u8,
    TsUndefinedKeyword(TsUndefinedKeyword) = NodeKind::TsUndefinedKeyword as u8,
    TsStringKeyword(TsStringKeyword) = NodeKind::TsStringKeyword as u8,
    TsNumberKeyword(TsNumberKeyword) = NodeKind::TsNumberKeyword as u8,
    TsBigIntKeyword(TsBigIntKeyword) = NodeKind::TsBigIntKeyword as u8,
    TsBooleanKeyword(TsBooleanKeyword) = NodeKind::TsBooleanKeyword as u8,
    TsSymbolKeyword(TsSymbolKeyword) = NodeKind::TsSymbolKeyword as u8,
    TsObjectKeyword(TsObjectKeyword) = NodeKind::TsObjectKeyword as u8,
    TsIntrinsicKeyword(TsIntrinsicKeyword) = NodeKind::TsIntrinsicKeyword as u8,
    TsThisType(TsThisType) = NodeKind::TsThisType as u8,
    TsTypeReference(TsTypeReference) = NodeKind::TsTypeReference as u8,
    TsQualifiedName(TsQualifiedName) = NodeKind::TsQualifiedName as u8,
    TsTypeQuery(TsTypeQuery) = NodeKind::TsTypeQuery as u8,
    TsImportType(TsImportType) = NodeKind::TsImportType as u8,
    TsTypeParameter(TsTypeParameter) = NodeKind::TsTypeParameter as u8,
    TsTypeParameterDeclaration(TsTypeParameterDeclaration) =
        NodeKind::TsTypeParameterDeclaration as u8,
    TsTypeParameterInstantiation(TsTypeParameterInstantiation) =
        NodeKind::TsTypeParameterInstantiation as u8,
    TsLiteralType(TsLiteralType) = NodeKind::TsLiteralType as u8,
    TsTemplateLiteralType(TsTemplateLiteralType) = NodeKind::TsTemplateLiteralType as u8,
    TsArrayType(TsArrayType) = NodeKind::TsArrayType as u8,
    TsIndexedAccessType(TsIndexedAccessType) = NodeKind::TsIndexedAccessType as u8,
    TsTupleType(TsTupleType) = NodeKind::TsTupleType as u8,
    TsNamedTupleMember(TsNamedTupleMember) = NodeKind::TsNamedTupleMember as u8,
    TsOptionalType(TsOptionalType) = NodeKind::TsOptionalType as u8,
    TsRestType(TsRestType) = NodeKind::TsRestType as u8,
    TsJsDocNullableType(TsJsDocNullableType) = NodeKind::TsJsDocNullableType as u8,
    TsJsDocNonNullableType(TsJsDocNonNullableType) = NodeKind::TsJsDocNonNullableType as u8,
    TsJsDocUnknownType(TsJsDocUnknownType) = NodeKind::TsJsDocUnknownType as u8,
    TsUnionType(TsUnionType) = NodeKind::TsUnionType as u8,
    TsIntersectionType(TsIntersectionType) = NodeKind::TsIntersectionType as u8,
    TsConditionalType(TsConditionalType) = NodeKind::TsConditionalType as u8,
    TsInferType(TsInferType) = NodeKind::TsInferType as u8,
    TsTypeOperator(TsTypeOperator) = NodeKind::TsTypeOperator as u8,
    TsParenthesizedType(TsParenthesizedType) = NodeKind::TsParenthesizedType as u8,
    TsFunctionType(TsFunctionType) = NodeKind::TsFunctionType as u8,
    TsConstructorType(TsConstructorType) = NodeKind::TsConstructorType as u8,
    TsTypePredicate(TsTypePredicate) = NodeKind::TsTypePredicate as u8,
    TsTypeLiteral(TsTypeLiteral) = NodeKind::TsTypeLiteral as u8,
    TsMappedType(TsMappedType) = NodeKind::TsMappedType as u8,
    TsPropertySignature(TsPropertySignature) = NodeKind::TsPropertySignature as u8,
    TsMethodSignature(TsMethodSignature) = NodeKind::TsMethodSignature as u8,
    TsCallSignatureDeclaration(TsCallSignatureDeclaration) =
        NodeKind::TsCallSignatureDeclaration as u8,
    TsConstructSignatureDeclaration(TsConstructSignatureDeclaration) =
        NodeKind::TsConstructSignatureDeclaration as u8,
    TsIndexSignature(TsIndexSignature) = NodeKind::TsIndexSignature as u8,
    TsTypeAliasDeclaration(TsTypeAliasDeclaration) = NodeKind::TsTypeAliasDeclaration as u8,
    TsInterfaceDeclaration(TsInterfaceDeclaration) = NodeKind::TsInterfaceDeclaration as u8,
    TsInterfaceBody(TsInterfaceBody) = NodeKind::TsInterfaceBody as u8,
    TsInterfaceHeritage(TsInterfaceHeritage) = NodeKind::TsInterfaceHeritage as u8,
    TsClassImplements(TsClassImplements) = NodeKind::TsClassImplements as u8,
    TsEnumDeclaration(TsEnumDeclaration) = NodeKind::TsEnumDeclaration as u8,
    TsEnumBody(TsEnumBody) = NodeKind::TsEnumBody as u8,
    TsEnumMember(TsEnumMember) = NodeKind::TsEnumMember as u8,
    TsModuleDeclaration(TsModuleDeclaration) = NodeKind::TsModuleDeclaration as u8,
    TsModuleBlock(TsModuleBlock) = NodeKind::TsModuleBlock as u8,
    TsGlobalDeclaration(TsGlobalDeclaration) = NodeKind::TsGlobalDeclaration as u8,
    TsParameterProperty(TsParameterProperty) = NodeKind::TsParameterProperty as u8,
    TsThisParameter(TsThisParameter) = NodeKind::TsThisParameter as u8,
    TsAsExpression(TsAsExpression) = NodeKind::TsAsExpression as u8,
    TsSatisfiesExpression(TsSatisfiesExpression) = NodeKind::TsSatisfiesExpression as u8,
    TsTypeAssertion(TsTypeAssertion) = NodeKind::TsTypeAssertion as u8,
    TsNonNullExpression(TsNonNullExpression) = NodeKind::TsNonNullExpression as u8,
    TsInstantiationExpression(TsInstantiationExpression) =
        NodeKind::TsInstantiationExpression as u8,
    TsExportAssignment(TsExportAssignment) = NodeKind::TsExportAssignment as u8,
    TsNamespaceExportDeclaration(TsNamespaceExportDeclaration) =
        NodeKind::TsNamespaceExportDeclaration as u8,
    TsImportEqualsDeclaration(TsImportEqualsDeclaration) =
        NodeKind::TsImportEqualsDeclaration as u8,
    TsExternalModuleReference(TsExternalModuleReference) =
        NodeKind::TsExternalModuleReference as u8,
    JsxElement(JsxElement) = NodeKind::JsxElement as u8,
    JsxOpeningElement(JsxOpeningElement) = NodeKind::JsxOpeningElement as u8,
    JsxClosingElement(JsxClosingElement) = NodeKind::JsxClosingElement as u8,
    JsxFragment(JsxFragment) = NodeKind::JsxFragment as u8,
    JsxOpeningFragment(JsxOpeningFragment) = NodeKind::JsxOpeningFragment as u8,
    JsxClosingFragment(JsxClosingFragment) = NodeKind::JsxClosingFragment as u8,
    JsxIdentifier(JsxIdentifier) = NodeKind::JsxIdentifier as u8,
    JsxNamespacedName(JsxNamespacedName) = NodeKind::JsxNamespacedName as u8,
    JsxMemberExpression(JsxMemberExpression) = NodeKind::JsxMemberExpression as u8,
    JsxAttribute(JsxAttribute) = NodeKind::JsxAttribute as u8,
    JsxSpreadAttribute(JsxSpreadAttribute) = NodeKind::JsxSpreadAttribute as u8,
    JsxExpressionContainer(JsxExpressionContainer) = NodeKind::JsxExpressionContainer as u8,
    JsxEmptyExpression(JsxEmptyExpression) = NodeKind::JsxEmptyExpression as u8,
    JsxText(JsxText) = NodeKind::JsxText as u8,
    JsxSpreadChild(JsxSpreadChild) = NodeKind::JsxSpreadChild as u8,
}
impl NodeKindData {
    #[inline]
    pub fn node_id(self) -> NodeId {
        unsafe { std::mem::transmute::<Self, AstEnumDataRepr>(self).node_id }
    }
}
impl Program {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for Program {}
impl AstNode for Program {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Program {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Program {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Stmt {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for Stmt {}
impl AstNode for Stmt {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Stmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Stmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Stmt {
    #[allow(non_snake_case)]
    #[inline]
    pub fn BlockStatement(node: BlockStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BreakStatement(node: BreakStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ContinueStatement(node: ContinueStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn DebuggerStatement(node: DebuggerStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn DoWhileStatement(node: DoWhileStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn EmptyStatement(node: EmptyStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ExpressionStatement(node: ExpressionStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ForInStatement(node: ForInStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ForOfStatement(node: ForOfStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ForStatement(node: ForStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn IfStatement(node: IfStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn LabeledStatement(node: LabeledStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ReturnStatement(node: ReturnStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn SwitchStatement(node: SwitchStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ThrowStatement(node: ThrowStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TryStatement(node: TryStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn WhileStatement(node: WhileStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn WithStatement(node: WithStatement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Declaration(node: Decl) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ImportDeclaration(node: ImportDeclaration) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ExportNamedDeclaration(node: ExportNamedDeclaration) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ExportDefaultDeclaration(node: ExportDefaultDeclaration) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ExportAllDeclaration(node: ExportAllDeclaration) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsExportAssignment(node: TsExportAssignment) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsNamespaceExportDeclaration(node: TsNamespaceExportDeclaration) -> Self {
        Self(node.node_id())
    }
}
impl Decl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for Decl {}
impl AstNode for Decl {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Decl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Decl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Decl {
    #[allow(non_snake_case)]
    #[inline]
    pub fn VariableDeclaration(node: VariableDeclaration) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Function(node: Function) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Class(node: Class) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsFunction(node: TsFunction) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeAliasDeclaration(node: TsTypeAliasDeclaration) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsInterfaceDeclaration(node: TsInterfaceDeclaration) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsEnumDeclaration(node: TsEnumDeclaration) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsModuleDeclaration(node: TsModuleDeclaration) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsGlobalDeclaration(node: TsGlobalDeclaration) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsImportEqualsDeclaration(node: TsImportEqualsDeclaration) -> Self {
        Self(node.node_id())
    }
}
impl Expr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for Expr {}
impl AstNode for Expr {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Expr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Expr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Expr {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierReference(node: IdentifierReference) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ThisExpression(node: ThisExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Super(node: Super) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn MetaProperty(node: MetaProperty) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn StringLiteral(node: StringLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn NumericLiteral(node: NumericLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BigIntLiteral(node: BigIntLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BooleanLiteral(node: BooleanLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn NullLiteral(node: NullLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn RegExpLiteral(node: RegExpLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TemplateLiteral(node: TemplateLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BinaryExpression(node: BinaryExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn LogicalExpression(node: LogicalExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn UnaryExpression(node: UnaryExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn UpdateExpression(node: UpdateExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn AssignmentExpression(node: AssignmentExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ConditionalExpression(node: ConditionalExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn SequenceExpression(node: SequenceExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn MemberExpression(node: MemberExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn CallExpression(node: CallExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ChainExpression(node: ChainExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn NewExpression(node: NewExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TaggedTemplateExpression(node: TaggedTemplateExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ArrowFunctionExpression(node: ArrowFunctionExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ArrayExpression(node: ArrayExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ObjectExpression(node: ObjectExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ParenthesizedExpression(node: ParenthesizedExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Function(node: Function) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Class(node: Class) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ImportExpression(node: ImportExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn AwaitExpression(node: AwaitExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn YieldExpression(node: YieldExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn PrivateIdentifier(node: PrivateIdentifier) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsAsExpression(node: TsAsExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsSatisfiesExpression(node: TsSatisfiesExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeAssertion(node: TsTypeAssertion) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsNonNullExpression(node: TsNonNullExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsInstantiationExpression(node: TsInstantiationExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxElement(node: JsxElement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxFragment(node: JsxFragment) -> Self {
        Self(node.node_id())
    }
}
impl BindingPattern {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for BindingPattern {}
impl AstNode for BindingPattern {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BindingPattern {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BindingPattern {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BindingPattern {
    #[allow(non_snake_case)]
    #[inline]
    pub fn BindingIdentifier(node: BindingIdentifier) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn SimpleAssignmentTarget(node: SimpleAssignmentTarget) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn AssignmentPattern(node: AssignmentPattern) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BindingRestElement(node: BindingRestElement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ArrayPattern(node: ArrayPattern) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ObjectPattern(node: ObjectPattern) -> Self {
        Self(node.node_id())
    }
}
impl AssignmentTarget {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for AssignmentTarget {}
impl AstNode for AssignmentTarget {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for AssignmentTarget {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AssignmentTarget {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl AssignmentTarget {
    #[allow(non_snake_case)]
    #[inline]
    pub fn SimpleAssignmentTarget(node: SimpleAssignmentTarget) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ArrayAssignmentTarget(node: ArrayPattern) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ObjectAssignmentTarget(node: ObjectPattern) -> Self {
        Self(node.node_id())
    }
}
impl SimpleAssignmentTarget {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for SimpleAssignmentTarget {}
impl AstNode for SimpleAssignmentTarget {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for SimpleAssignmentTarget {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SimpleAssignmentTarget {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl SimpleAssignmentTarget {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierReference(node: IdentifierReference) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn MemberExpression(node: MemberExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsAsExpression(node: TsAsExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsSatisfiesExpression(node: TsSatisfiesExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeAssertion(node: TsTypeAssertion) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsNonNullExpression(node: TsNonNullExpression) -> Self {
        Self(node.node_id())
    }
}
impl PropertyKey {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for PropertyKey {}
impl AstNode for PropertyKey {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for PropertyKey {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for PropertyKey {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl PropertyKey {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierName(node: IdentifierName) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Expr(node: Expr) -> Self {
        Self(node.node_id())
    }
}
impl ModuleExportName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ModuleExportName {}
impl AstNode for ModuleExportName {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ModuleExportName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ModuleExportName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ModuleExportName {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierName(node: IdentifierName) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn StringLiteral(node: StringLiteral) -> Self {
        Self(node.node_id())
    }
}
impl Argument {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for Argument {}
impl AstNode for Argument {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Argument {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Argument {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Argument {
    #[allow(non_snake_case)]
    #[inline]
    pub fn Expr(node: Expr) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn SpreadElement(node: SpreadElement) -> Self {
        Self(node.node_id())
    }
}
impl ObjectPropertyKind {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ObjectPropertyKind {}
impl AstNode for ObjectPropertyKind {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ObjectPropertyKind {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ObjectPropertyKind {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ObjectPropertyKind {
    #[allow(non_snake_case)]
    #[inline]
    pub fn ObjectProperty(node: ObjectProperty) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn SpreadElement(node: SpreadElement) -> Self {
        Self(node.node_id())
    }
}
impl ForStatementInit {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ForStatementInit {}
impl AstNode for ForStatementInit {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ForStatementInit {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ForStatementInit {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ForStatementInit {
    #[allow(non_snake_case)]
    #[inline]
    pub fn VariableDeclaration(node: VariableDeclaration) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Expr(node: Expr) -> Self {
        Self(node.node_id())
    }
}
impl ForStatementLeft {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ForStatementLeft {}
impl AstNode for ForStatementLeft {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ForStatementLeft {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ForStatementLeft {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ForStatementLeft {
    #[allow(non_snake_case)]
    #[inline]
    pub fn VariableDeclaration(node: VariableDeclaration) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn AssignmentTarget(node: AssignmentTarget) -> Self {
        Self(node.node_id())
    }
}
impl ArrowFunctionBody {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ArrowFunctionBody {}
impl AstNode for ArrowFunctionBody {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ArrowFunctionBody {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ArrowFunctionBody {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ArrowFunctionBody {
    #[allow(non_snake_case)]
    #[inline]
    pub fn FunctionBody(node: FunctionBody) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Expr(node: Expr) -> Self {
        Self(node.node_id())
    }
}
impl FormalParameterItem {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for FormalParameterItem {}
impl AstNode for FormalParameterItem {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for FormalParameterItem {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for FormalParameterItem {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl FormalParameterItem {
    #[allow(non_snake_case)]
    #[inline]
    pub fn FormalParameter(node: FormalParameter) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsParameterProperty(node: TsParameterProperty) -> Self {
        Self(node.node_id())
    }
}
impl FormalParameterPattern {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for FormalParameterPattern {}
impl AstNode for FormalParameterPattern {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for FormalParameterPattern {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for FormalParameterPattern {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl FormalParameterPattern {
    #[allow(non_snake_case)]
    #[inline]
    pub fn BindingPattern(node: BindingPattern) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsThisParameter(node: TsThisParameter) -> Self {
        Self(node.node_id())
    }
}
impl ClassElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ClassElement {}
impl AstNode for ClassElement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ClassElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ClassElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ClassElement {
    #[allow(non_snake_case)]
    #[inline]
    pub fn MethodDefinition(node: MethodDefinition) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn PropertyDefinition(node: PropertyDefinition) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn StaticBlock(node: StaticBlock) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsMethodDefinition(node: TsMethodDefinition) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsIndexSignature(node: TsIndexSignature) -> Self {
        Self(node.node_id())
    }
}
impl ImportDeclarationSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ImportDeclarationSpecifier {}
impl AstNode for ImportDeclarationSpecifier {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ImportDeclarationSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportDeclarationSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ImportDeclarationSpecifier {
    #[allow(non_snake_case)]
    #[inline]
    pub fn ImportSpecifier(node: ImportSpecifier) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ImportDefaultSpecifier(node: ImportDefaultSpecifier) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ImportNamespaceSpecifier(node: ImportNamespaceSpecifier) -> Self {
        Self(node.node_id())
    }
}
impl ExportDefaultDeclarationKind {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ExportDefaultDeclarationKind {}
impl AstNode for ExportDefaultDeclarationKind {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExportDefaultDeclarationKind {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportDefaultDeclarationKind {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExportDefaultDeclarationKind {
    #[allow(non_snake_case)]
    #[inline]
    pub fn Expr(node: Expr) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Function(node: Function) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Class(node: Class) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsFunction(node: TsFunction) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsInterfaceDeclaration(node: TsInterfaceDeclaration) -> Self {
        Self(node.node_id())
    }
}
impl Super {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for Super {}
impl AstNode for Super {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Super {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Super {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ThisExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ThisExpression {}
impl AstNode for ThisExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ThisExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ThisExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl StringLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for StringLiteral {}
impl AstNode for StringLiteral {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for StringLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for StringLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl NumericLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for NumericLiteral {}
impl AstNode for NumericLiteral {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for NumericLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for NumericLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BigIntLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for BigIntLiteral {}
impl AstNode for BigIntLiteral {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BigIntLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BigIntLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BooleanLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for BooleanLiteral {}
impl AstNode for BooleanLiteral {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BooleanLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BooleanLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl NullLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for NullLiteral {}
impl AstNode for NullLiteral {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for NullLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for NullLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl RegExpLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for RegExpLiteral {}
impl AstNode for RegExpLiteral {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for RegExpLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for RegExpLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TemplateLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TemplateLiteral {}
impl AstNode for TemplateLiteral {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TemplateLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TemplateLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TemplateElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TemplateElement {}
impl AstNode for TemplateElement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TemplateElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TemplateElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl IdentifierReference {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for IdentifierReference {}
impl AstNode for IdentifierReference {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for IdentifierReference {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for IdentifierReference {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl PrivateIdentifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for PrivateIdentifier {}
impl AstNode for PrivateIdentifier {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for PrivateIdentifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for PrivateIdentifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BindingIdentifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for BindingIdentifier {}
impl AstNode for BindingIdentifier {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BindingIdentifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BindingIdentifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl IdentifierName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for IdentifierName {}
impl AstNode for IdentifierName {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for IdentifierName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for IdentifierName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl LabelIdentifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for LabelIdentifier {}
impl AstNode for LabelIdentifier {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for LabelIdentifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for LabelIdentifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl AssignmentPattern {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for AssignmentPattern {}
impl AstNode for AssignmentPattern {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for AssignmentPattern {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AssignmentPattern {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BindingRestElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for BindingRestElement {}
impl AstNode for BindingRestElement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BindingRestElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BindingRestElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ArrayPattern {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ArrayPattern {}
impl AstNode for ArrayPattern {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ArrayPattern {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ArrayPattern {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ObjectPattern {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ObjectPattern {}
impl AstNode for ObjectPattern {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ObjectPattern {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ObjectPattern {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BindingProperty {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for BindingProperty {}
impl AstNode for BindingProperty {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BindingProperty {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BindingProperty {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BinaryExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for BinaryExpression {}
impl AstNode for BinaryExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BinaryExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BinaryExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl LogicalExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for LogicalExpression {}
impl AstNode for LogicalExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for LogicalExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for LogicalExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ConditionalExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ConditionalExpression {}
impl AstNode for ConditionalExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ConditionalExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ConditionalExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl UnaryExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for UnaryExpression {}
impl AstNode for UnaryExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for UnaryExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for UnaryExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl UpdateExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for UpdateExpression {}
impl AstNode for UpdateExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for UpdateExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for UpdateExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl AssignmentExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for AssignmentExpression {}
impl AstNode for AssignmentExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for AssignmentExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AssignmentExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl VariableDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for VariableDeclaration {}
impl AstNode for VariableDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for VariableDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for VariableDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl VariableDeclarator {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for VariableDeclarator {}
impl AstNode for VariableDeclarator {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for VariableDeclarator {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for VariableDeclarator {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl IfStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for IfStatement {}
impl AstNode for IfStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for IfStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for IfStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl SwitchStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for SwitchStatement {}
impl AstNode for SwitchStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for SwitchStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SwitchStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl SwitchCase {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for SwitchCase {}
impl AstNode for SwitchCase {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for SwitchCase {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SwitchCase {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ForStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ForStatement {}
impl AstNode for ForStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ForStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ForStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ForInStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ForInStatement {}
impl AstNode for ForInStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ForInStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ForInStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ForOfStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ForOfStatement {}
impl AstNode for ForOfStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ForOfStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ForOfStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TryStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TryStatement {}
impl AstNode for TryStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TryStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TryStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl CatchClause {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for CatchClause {}
impl AstNode for CatchClause {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for CatchClause {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for CatchClause {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BreakStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for BreakStatement {}
impl AstNode for BreakStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BreakStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BreakStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ContinueStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ContinueStatement {}
impl AstNode for ContinueStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ContinueStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ContinueStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl LabeledStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for LabeledStatement {}
impl AstNode for LabeledStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for LabeledStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for LabeledStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExpressionStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ExpressionStatement {}
impl AstNode for ExpressionStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExpressionStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExpressionStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ReturnStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ReturnStatement {}
impl AstNode for ReturnStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ReturnStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ReturnStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ThrowStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ThrowStatement {}
impl AstNode for ThrowStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ThrowStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ThrowStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl DebuggerStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for DebuggerStatement {}
impl AstNode for DebuggerStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for DebuggerStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for DebuggerStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl EmptyStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for EmptyStatement {}
impl AstNode for EmptyStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for EmptyStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for EmptyStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Directive {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for Directive {}
impl AstNode for Directive {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Directive {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Directive {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Decorator {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for Decorator {}
impl AstNode for Decorator {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Decorator {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Decorator {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Class {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for Class {}
impl AstNode for Class {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Class {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Class {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ClassBody {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ClassBody {}
impl AstNode for ClassBody {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ClassBody {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ClassBody {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl MethodDefinition {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for MethodDefinition {}
impl AstNode for MethodDefinition {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for MethodDefinition {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for MethodDefinition {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl PropertyDefinition {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for PropertyDefinition {}
impl AstNode for PropertyDefinition {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for PropertyDefinition {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for PropertyDefinition {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl StaticBlock {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for StaticBlock {}
impl AstNode for StaticBlock {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for StaticBlock {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for StaticBlock {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Function {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for Function {}
impl AstNode for Function {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Function {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Function {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl FunctionBody {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for FunctionBody {}
impl AstNode for FunctionBody {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for FunctionBody {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for FunctionBody {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl BlockStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for BlockStatement {}
impl AstNode for BlockStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for BlockStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BlockStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl WhileStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for WhileStatement {}
impl AstNode for WhileStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for WhileStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for WhileStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl DoWhileStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for DoWhileStatement {}
impl AstNode for DoWhileStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for DoWhileStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for DoWhileStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl WithStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for WithStatement {}
impl AstNode for WithStatement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for WithStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for WithStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl FormalParameters {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for FormalParameters {}
impl AstNode for FormalParameters {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for FormalParameters {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for FormalParameters {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl FormalParameter {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for FormalParameter {}
impl AstNode for FormalParameter {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for FormalParameter {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for FormalParameter {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ArrayExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ArrayExpression {}
impl AstNode for ArrayExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ArrayExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ArrayExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ObjectExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ObjectExpression {}
impl AstNode for ObjectExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ObjectExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ObjectExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl SpreadElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for SpreadElement {}
impl AstNode for SpreadElement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for SpreadElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SpreadElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ObjectProperty {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ObjectProperty {}
impl AstNode for ObjectProperty {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ObjectProperty {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ObjectProperty {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ParenthesizedExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ParenthesizedExpression {}
impl AstNode for ParenthesizedExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ParenthesizedExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ParenthesizedExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ArrowFunctionExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ArrowFunctionExpression {}
impl AstNode for ArrowFunctionExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ArrowFunctionExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ArrowFunctionExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl SequenceExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for SequenceExpression {}
impl AstNode for SequenceExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for SequenceExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SequenceExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl MemberExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for MemberExpression {}
impl AstNode for MemberExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for MemberExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for MemberExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl CallExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for CallExpression {}
impl AstNode for CallExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for CallExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for CallExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ChainExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ChainExpression {}
impl AstNode for ChainExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ChainExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ChainExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TaggedTemplateExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TaggedTemplateExpression {}
impl AstNode for TaggedTemplateExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TaggedTemplateExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TaggedTemplateExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl NewExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for NewExpression {}
impl AstNode for NewExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for NewExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for NewExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl AwaitExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for AwaitExpression {}
impl AstNode for AwaitExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for AwaitExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AwaitExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl YieldExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for YieldExpression {}
impl AstNode for YieldExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for YieldExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for YieldExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl MetaProperty {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for MetaProperty {}
impl AstNode for MetaProperty {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for MetaProperty {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for MetaProperty {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ImportExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ImportExpression {}
impl AstNode for ImportExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ImportExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ImportDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ImportDeclaration {}
impl AstNode for ImportDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ImportDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ImportSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ImportSpecifier {}
impl AstNode for ImportSpecifier {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ImportSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ImportDefaultSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ImportDefaultSpecifier {}
impl AstNode for ImportDefaultSpecifier {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ImportDefaultSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportDefaultSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ImportNamespaceSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ImportNamespaceSpecifier {}
impl AstNode for ImportNamespaceSpecifier {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ImportNamespaceSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportNamespaceSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ImportAttribute {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ImportAttribute {}
impl AstNode for ImportAttribute {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ImportAttribute {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportAttribute {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExportNamedDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ExportNamedDeclaration {}
impl AstNode for ExportNamedDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExportNamedDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportNamedDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExportDefaultDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ExportDefaultDeclaration {}
impl AstNode for ExportDefaultDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExportDefaultDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportDefaultDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExportAllDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ExportAllDeclaration {}
impl AstNode for ExportAllDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExportAllDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportAllDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl ExportSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for ExportSpecifier {}
impl AstNode for ExportSpecifier {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for ExportSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl Hashbang {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for Hashbang {}
impl AstNode for Hashbang {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for Hashbang {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Hashbang {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsType {}
impl AstNode for TsType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsType {
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsAnyKeyword(node: TsAnyKeyword) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsUnknownKeyword(node: TsUnknownKeyword) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsNeverKeyword(node: TsNeverKeyword) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsVoidKeyword(node: TsVoidKeyword) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsNullKeyword(node: TsNullKeyword) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsUndefinedKeyword(node: TsUndefinedKeyword) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsStringKeyword(node: TsStringKeyword) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsNumberKeyword(node: TsNumberKeyword) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsBigIntKeyword(node: TsBigIntKeyword) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsBooleanKeyword(node: TsBooleanKeyword) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsSymbolKeyword(node: TsSymbolKeyword) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsObjectKeyword(node: TsObjectKeyword) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsIntrinsicKeyword(node: TsIntrinsicKeyword) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsThisType(node: TsThisType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeReference(node: TsTypeReference) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeQuery(node: TsTypeQuery) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsImportType(node: TsImportType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsLiteralType(node: TsLiteralType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTemplateLiteralType(node: TsTemplateLiteralType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsArrayType(node: TsArrayType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsIndexedAccessType(node: TsIndexedAccessType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTupleType(node: TsTupleType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsOptionalType(node: TsOptionalType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsRestType(node: TsRestType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsJsDocNullableType(node: TsJsDocNullableType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsJsDocNonNullableType(node: TsJsDocNonNullableType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsJsDocUnknownType(node: TsJsDocUnknownType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsUnionType(node: TsUnionType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsIntersectionType(node: TsIntersectionType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsConditionalType(node: TsConditionalType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsInferType(node: TsInferType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeOperator(node: TsTypeOperator) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsParenthesizedType(node: TsParenthesizedType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsFunctionType(node: TsFunctionType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsConstructorType(node: TsConstructorType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypePredicate(node: TsTypePredicate) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeLiteral(node: TsTypeLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsMappedType(node: TsMappedType) -> Self {
        Self(node.node_id())
    }
}
impl TsTypeName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypeName {}
impl AstNode for TsTypeName {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypeName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypeName {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierReference(node: IdentifierReference) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsQualifiedName(node: TsQualifiedName) -> Self {
        Self(node.node_id())
    }
}
impl TsQualifiedNameLeft {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsQualifiedNameLeft {}
impl AstNode for TsQualifiedNameLeft {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsQualifiedNameLeft {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsQualifiedNameLeft {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsQualifiedNameLeft {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierReference(node: IdentifierReference) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierName(node: IdentifierName) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BindingIdentifier(node: BindingIdentifier) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ThisExpression(node: ThisExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsQualifiedName(node: TsQualifiedName) -> Self {
        Self(node.node_id())
    }
}
impl TsImportTypeQualifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsImportTypeQualifier {}
impl AstNode for TsImportTypeQualifier {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsImportTypeQualifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsImportTypeQualifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsImportTypeQualifier {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierName(node: IdentifierName) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsQualifiedName(node: TsQualifiedName) -> Self {
        Self(node.node_id())
    }
}
impl TsTypeQueryExprName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypeQueryExprName {}
impl AstNode for TsTypeQueryExprName {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypeQueryExprName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeQueryExprName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypeQueryExprName {
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeName(node: TsTypeName) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ThisExpression(node: ThisExpression) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsImportType(node: TsImportType) -> Self {
        Self(node.node_id())
    }
}
impl TsLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsLiteral {}
impl AstNode for TsLiteral {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsLiteral {
    #[allow(non_snake_case)]
    #[inline]
    pub fn StringLiteral(node: StringLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn NumericLiteral(node: NumericLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BigIntLiteral(node: BigIntLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BooleanLiteral(node: BooleanLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TemplateLiteral(node: TemplateLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn UnaryExpression(node: UnaryExpression) -> Self {
        Self(node.node_id())
    }
}
impl TsTupleElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTupleElement {}
impl AstNode for TsTupleElement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTupleElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTupleElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTupleElement {
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsType(node: TsType) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsNamedTupleMember(node: TsNamedTupleMember) -> Self {
        Self(node.node_id())
    }
}
impl TsTypeElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypeElement {}
impl AstNode for TsTypeElement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypeElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypeElement {
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsPropertySignature(node: TsPropertySignature) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsMethodSignature(node: TsMethodSignature) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsCallSignatureDeclaration(node: TsCallSignatureDeclaration) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsConstructSignatureDeclaration(node: TsConstructSignatureDeclaration) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsIndexSignature(node: TsIndexSignature) -> Self {
        Self(node.node_id())
    }
}
impl TsTypePredicateName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypePredicateName {}
impl AstNode for TsTypePredicateName {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypePredicateName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypePredicateName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypePredicateName {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierName(node: IdentifierName) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsThisType(node: TsThisType) -> Self {
        Self(node.node_id())
    }
}
impl TsEnumMemberName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsEnumMemberName {}
impl AstNode for TsEnumMemberName {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsEnumMemberName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsEnumMemberName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsEnumMemberName {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierName(node: IdentifierName) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn StringLiteral(node: StringLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TemplateLiteral(node: TemplateLiteral) -> Self {
        Self(node.node_id())
    }
}
impl TsModuleName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsModuleName {}
impl AstNode for TsModuleName {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsModuleName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsModuleName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsModuleName {
    #[allow(non_snake_case)]
    #[inline]
    pub fn BindingIdentifier(node: BindingIdentifier) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn StringLiteral(node: StringLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsQualifiedName(node: TsQualifiedName) -> Self {
        Self(node.node_id())
    }
}
impl TsModuleBody {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsModuleBody {}
impl AstNode for TsModuleBody {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsModuleBody {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsModuleBody {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsModuleBody {
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsModuleBlock(node: TsModuleBlock) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsModuleDeclaration(node: TsModuleDeclaration) -> Self {
        Self(node.node_id())
    }
}
impl TsModuleReference {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsModuleReference {}
impl AstNode for TsModuleReference {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsModuleReference {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsModuleReference {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsModuleReference {
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeName(node: TsTypeName) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsExternalModuleReference(node: TsExternalModuleReference) -> Self {
        Self(node.node_id())
    }
}
impl TsFunction {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsFunction {}
impl AstNode for TsFunction {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsFunction {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsFunction {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsMethodDefinition {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsMethodDefinition {}
impl AstNode for TsMethodDefinition {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsMethodDefinition {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsMethodDefinition {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypeAnnotation {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypeAnnotation {}
impl AstNode for TsTypeAnnotation {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypeAnnotation {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeAnnotation {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsAnyKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsAnyKeyword {}
impl AstNode for TsAnyKeyword {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsAnyKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsAnyKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsUnknownKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsUnknownKeyword {}
impl AstNode for TsUnknownKeyword {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsUnknownKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsUnknownKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsNeverKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsNeverKeyword {}
impl AstNode for TsNeverKeyword {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsNeverKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsNeverKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsVoidKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsVoidKeyword {}
impl AstNode for TsVoidKeyword {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsVoidKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsVoidKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsNullKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsNullKeyword {}
impl AstNode for TsNullKeyword {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsNullKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsNullKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsUndefinedKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsUndefinedKeyword {}
impl AstNode for TsUndefinedKeyword {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsUndefinedKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsUndefinedKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsStringKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsStringKeyword {}
impl AstNode for TsStringKeyword {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsStringKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsStringKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsNumberKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsNumberKeyword {}
impl AstNode for TsNumberKeyword {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsNumberKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsNumberKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsBigIntKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsBigIntKeyword {}
impl AstNode for TsBigIntKeyword {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsBigIntKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsBigIntKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsBooleanKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsBooleanKeyword {}
impl AstNode for TsBooleanKeyword {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsBooleanKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsBooleanKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsSymbolKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsSymbolKeyword {}
impl AstNode for TsSymbolKeyword {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsSymbolKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsSymbolKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsObjectKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsObjectKeyword {}
impl AstNode for TsObjectKeyword {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsObjectKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsObjectKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsIntrinsicKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsIntrinsicKeyword {}
impl AstNode for TsIntrinsicKeyword {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsIntrinsicKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsIntrinsicKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsThisType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsThisType {}
impl AstNode for TsThisType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsThisType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsThisType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypeReference {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypeReference {}
impl AstNode for TsTypeReference {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypeReference {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeReference {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsQualifiedName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsQualifiedName {}
impl AstNode for TsQualifiedName {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsQualifiedName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsQualifiedName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypeQuery {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypeQuery {}
impl AstNode for TsTypeQuery {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypeQuery {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeQuery {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsImportType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsImportType {}
impl AstNode for TsImportType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsImportType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsImportType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypeParameter {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypeParameter {}
impl AstNode for TsTypeParameter {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypeParameter {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeParameter {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypeParameterDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypeParameterDeclaration {}
impl AstNode for TsTypeParameterDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypeParameterDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeParameterDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypeParameterInstantiation {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypeParameterInstantiation {}
impl AstNode for TsTypeParameterInstantiation {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypeParameterInstantiation {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeParameterInstantiation {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsLiteralType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsLiteralType {}
impl AstNode for TsLiteralType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsLiteralType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsLiteralType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTemplateLiteralType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTemplateLiteralType {}
impl AstNode for TsTemplateLiteralType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTemplateLiteralType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTemplateLiteralType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsArrayType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsArrayType {}
impl AstNode for TsArrayType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsArrayType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsArrayType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsIndexedAccessType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsIndexedAccessType {}
impl AstNode for TsIndexedAccessType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsIndexedAccessType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsIndexedAccessType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTupleType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTupleType {}
impl AstNode for TsTupleType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTupleType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTupleType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsNamedTupleMember {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsNamedTupleMember {}
impl AstNode for TsNamedTupleMember {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsNamedTupleMember {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsNamedTupleMember {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsOptionalType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsOptionalType {}
impl AstNode for TsOptionalType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsOptionalType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsOptionalType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsRestType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsRestType {}
impl AstNode for TsRestType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsRestType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsRestType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsJsDocNullableType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsJsDocNullableType {}
impl AstNode for TsJsDocNullableType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsJsDocNullableType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsJsDocNullableType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsJsDocNonNullableType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsJsDocNonNullableType {}
impl AstNode for TsJsDocNonNullableType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsJsDocNonNullableType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsJsDocNonNullableType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsJsDocUnknownType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsJsDocUnknownType {}
impl AstNode for TsJsDocUnknownType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsJsDocUnknownType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsJsDocUnknownType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsUnionType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsUnionType {}
impl AstNode for TsUnionType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsUnionType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsUnionType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsIntersectionType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsIntersectionType {}
impl AstNode for TsIntersectionType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsIntersectionType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsIntersectionType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsConditionalType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsConditionalType {}
impl AstNode for TsConditionalType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsConditionalType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsConditionalType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsInferType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsInferType {}
impl AstNode for TsInferType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsInferType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsInferType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypeOperator {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypeOperator {}
impl AstNode for TsTypeOperator {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypeOperator {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeOperator {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsParenthesizedType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsParenthesizedType {}
impl AstNode for TsParenthesizedType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsParenthesizedType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsParenthesizedType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsFunctionType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsFunctionType {}
impl AstNode for TsFunctionType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsFunctionType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsFunctionType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsConstructorType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsConstructorType {}
impl AstNode for TsConstructorType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsConstructorType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsConstructorType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypePredicate {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypePredicate {}
impl AstNode for TsTypePredicate {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypePredicate {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypePredicate {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypeLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypeLiteral {}
impl AstNode for TsTypeLiteral {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypeLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsMappedType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsMappedType {}
impl AstNode for TsMappedType {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsMappedType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsMappedType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsPropertySignature {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsPropertySignature {}
impl AstNode for TsPropertySignature {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsPropertySignature {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsPropertySignature {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsMethodSignature {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsMethodSignature {}
impl AstNode for TsMethodSignature {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsMethodSignature {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsMethodSignature {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsCallSignatureDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsCallSignatureDeclaration {}
impl AstNode for TsCallSignatureDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsCallSignatureDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsCallSignatureDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsConstructSignatureDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsConstructSignatureDeclaration {}
impl AstNode for TsConstructSignatureDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsConstructSignatureDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsConstructSignatureDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsIndexSignature {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsIndexSignature {}
impl AstNode for TsIndexSignature {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsIndexSignature {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsIndexSignature {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypeAliasDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypeAliasDeclaration {}
impl AstNode for TsTypeAliasDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypeAliasDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeAliasDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsInterfaceDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsInterfaceDeclaration {}
impl AstNode for TsInterfaceDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsInterfaceDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsInterfaceDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsInterfaceBody {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsInterfaceBody {}
impl AstNode for TsInterfaceBody {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsInterfaceBody {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsInterfaceBody {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsInterfaceHeritage {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsInterfaceHeritage {}
impl AstNode for TsInterfaceHeritage {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsInterfaceHeritage {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsInterfaceHeritage {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsClassImplements {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsClassImplements {}
impl AstNode for TsClassImplements {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsClassImplements {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsClassImplements {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsEnumDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsEnumDeclaration {}
impl AstNode for TsEnumDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsEnumDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsEnumDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsEnumBody {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsEnumBody {}
impl AstNode for TsEnumBody {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsEnumBody {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsEnumBody {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsEnumMember {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsEnumMember {}
impl AstNode for TsEnumMember {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsEnumMember {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsEnumMember {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsModuleDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsModuleDeclaration {}
impl AstNode for TsModuleDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsModuleDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsModuleDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsModuleBlock {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsModuleBlock {}
impl AstNode for TsModuleBlock {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsModuleBlock {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsModuleBlock {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsGlobalDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsGlobalDeclaration {}
impl AstNode for TsGlobalDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsGlobalDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsGlobalDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsParameterProperty {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsParameterProperty {}
impl AstNode for TsParameterProperty {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsParameterProperty {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsParameterProperty {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsThisParameter {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsThisParameter {}
impl AstNode for TsThisParameter {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsThisParameter {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsThisParameter {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsAsExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsAsExpression {}
impl AstNode for TsAsExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsAsExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsAsExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsSatisfiesExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsSatisfiesExpression {}
impl AstNode for TsSatisfiesExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsSatisfiesExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsSatisfiesExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsTypeAssertion {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsTypeAssertion {}
impl AstNode for TsTypeAssertion {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsTypeAssertion {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeAssertion {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsNonNullExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsNonNullExpression {}
impl AstNode for TsNonNullExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsNonNullExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsNonNullExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsInstantiationExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsInstantiationExpression {}
impl AstNode for TsInstantiationExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsInstantiationExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsInstantiationExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsExportAssignment {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsExportAssignment {}
impl AstNode for TsExportAssignment {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsExportAssignment {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsExportAssignment {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsNamespaceExportDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsNamespaceExportDeclaration {}
impl AstNode for TsNamespaceExportDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsNamespaceExportDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsNamespaceExportDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsImportEqualsDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsImportEqualsDeclaration {}
impl AstNode for TsImportEqualsDeclaration {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsImportEqualsDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsImportEqualsDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl TsExternalModuleReference {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for TsExternalModuleReference {}
impl AstNode for TsExternalModuleReference {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for TsExternalModuleReference {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsExternalModuleReference {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxElementName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxElementName {}
impl AstNode for JsxElementName {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxElementName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxElementName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxElementName {
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxIdentifier(node: JsxIdentifier) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxNamespacedName(node: JsxNamespacedName) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxMemberExpression(node: JsxMemberExpression) -> Self {
        Self(node.node_id())
    }
}
impl JsxMemberExpressionObject {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxMemberExpressionObject {}
impl AstNode for JsxMemberExpressionObject {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxMemberExpressionObject {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxMemberExpressionObject {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxMemberExpressionObject {
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxIdentifier(node: JsxIdentifier) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxMemberExpression(node: JsxMemberExpression) -> Self {
        Self(node.node_id())
    }
}
impl JsxAttributeName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxAttributeName {}
impl AstNode for JsxAttributeName {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxAttributeName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxAttributeName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxAttributeName {
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxIdentifier(node: JsxIdentifier) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxNamespacedName(node: JsxNamespacedName) -> Self {
        Self(node.node_id())
    }
}
impl JsxAttributeItem {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxAttributeItem {}
impl AstNode for JsxAttributeItem {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxAttributeItem {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxAttributeItem {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxAttributeItem {
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxAttribute(node: JsxAttribute) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxSpreadAttribute(node: JsxSpreadAttribute) -> Self {
        Self(node.node_id())
    }
}
impl JsxAttributeValue {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxAttributeValue {}
impl AstNode for JsxAttributeValue {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxAttributeValue {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxAttributeValue {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxAttributeValue {
    #[allow(non_snake_case)]
    #[inline]
    pub fn StringLiteral(node: StringLiteral) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxExpressionContainer(node: JsxExpressionContainer) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxElement(node: JsxElement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxFragment(node: JsxFragment) -> Self {
        Self(node.node_id())
    }
}
impl JsxExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxExpression {}
impl AstNode for JsxExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxExpression {
    #[allow(non_snake_case)]
    #[inline]
    pub fn Expr(node: Expr) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxEmptyExpression(node: JsxEmptyExpression) -> Self {
        Self(node.node_id())
    }
}
impl JsxChild {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxChild {}
impl AstNode for JsxChild {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxChild {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxChild {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxChild {
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxText(node: JsxText) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxExpressionContainer(node: JsxExpressionContainer) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxSpreadChild(node: JsxSpreadChild) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxElement(node: JsxElement) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxFragment(node: JsxFragment) -> Self {
        Self(node.node_id())
    }
}
impl JsxElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxElement {}
impl AstNode for JsxElement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxOpeningElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxOpeningElement {}
impl AstNode for JsxOpeningElement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxOpeningElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxOpeningElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxClosingElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxClosingElement {}
impl AstNode for JsxClosingElement {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxClosingElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxClosingElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxFragment {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxFragment {}
impl AstNode for JsxFragment {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxFragment {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxFragment {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxOpeningFragment {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxOpeningFragment {}
impl AstNode for JsxOpeningFragment {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxOpeningFragment {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxOpeningFragment {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxClosingFragment {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxClosingFragment {}
impl AstNode for JsxClosingFragment {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxClosingFragment {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxClosingFragment {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxIdentifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxIdentifier {}
impl AstNode for JsxIdentifier {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxIdentifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxIdentifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxNamespacedName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxNamespacedName {}
impl AstNode for JsxNamespacedName {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxNamespacedName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxNamespacedName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxMemberExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxMemberExpression {}
impl AstNode for JsxMemberExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxMemberExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxMemberExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxAttribute {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxAttribute {}
impl AstNode for JsxAttribute {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxAttribute {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxAttribute {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxSpreadAttribute {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxSpreadAttribute {}
impl AstNode for JsxSpreadAttribute {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxSpreadAttribute {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxSpreadAttribute {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxExpressionContainer {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxExpressionContainer {}
impl AstNode for JsxExpressionContainer {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxExpressionContainer {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxExpressionContainer {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxEmptyExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxEmptyExpression {}
impl AstNode for JsxEmptyExpression {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxEmptyExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxEmptyExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxText {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxText {}
impl AstNode for JsxText {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxText {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxText {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl JsxSpreadChild {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl SealedAstNode for JsxSpreadChild {}
impl AstNode for JsxSpreadChild {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId {
        self.0
    }
}
impl GetSpan for JsxSpreadChild {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxSpreadChild {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0) = span;
        }
    }
}
impl<'a> Ast<'a> {
    /// Returns the typed wrapper for the concrete node stored at `node_id`.
    #[inline]
    pub fn kind_data(&self, node_id: NodeId) -> NodeKindData {
        let kind = self.node_kind(node_id);
        unsafe { std::mem::transmute(AstEnumDataRepr { kind, node_id }) }
    }
    #[inline]
    pub fn stmt_data(&self, node: Stmt) -> StmtData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::BlockStatement => StmtData::BlockStatement(BlockStatement(node.0)),
            NodeKind::BreakStatement => StmtData::BreakStatement(BreakStatement(node.0)),
            NodeKind::ContinueStatement => StmtData::ContinueStatement(ContinueStatement(node.0)),
            NodeKind::DebuggerStatement => StmtData::DebuggerStatement(DebuggerStatement(node.0)),
            NodeKind::DoWhileStatement => StmtData::DoWhileStatement(DoWhileStatement(node.0)),
            NodeKind::EmptyStatement => StmtData::EmptyStatement(EmptyStatement(node.0)),
            NodeKind::ExpressionStatement => {
                StmtData::ExpressionStatement(ExpressionStatement(node.0))
            }
            NodeKind::ForInStatement => StmtData::ForInStatement(ForInStatement(node.0)),
            NodeKind::ForOfStatement => StmtData::ForOfStatement(ForOfStatement(node.0)),
            NodeKind::ForStatement => StmtData::ForStatement(ForStatement(node.0)),
            NodeKind::IfStatement => StmtData::IfStatement(IfStatement(node.0)),
            NodeKind::LabeledStatement => StmtData::LabeledStatement(LabeledStatement(node.0)),
            NodeKind::ReturnStatement => StmtData::ReturnStatement(ReturnStatement(node.0)),
            NodeKind::SwitchStatement => StmtData::SwitchStatement(SwitchStatement(node.0)),
            NodeKind::ThrowStatement => StmtData::ThrowStatement(ThrowStatement(node.0)),
            NodeKind::TryStatement => StmtData::TryStatement(TryStatement(node.0)),
            NodeKind::WhileStatement => StmtData::WhileStatement(WhileStatement(node.0)),
            NodeKind::WithStatement => StmtData::WithStatement(WithStatement(node.0)),
            NodeKind::VariableDeclaration => StmtData::Declaration(Decl(node.0)),
            NodeKind::Function => StmtData::Declaration(Decl(node.0)),
            NodeKind::Class => StmtData::Declaration(Decl(node.0)),
            NodeKind::TsFunction => StmtData::Declaration(Decl(node.0)),
            NodeKind::TsTypeAliasDeclaration => StmtData::Declaration(Decl(node.0)),
            NodeKind::TsInterfaceDeclaration => StmtData::Declaration(Decl(node.0)),
            NodeKind::TsEnumDeclaration => StmtData::Declaration(Decl(node.0)),
            NodeKind::TsModuleDeclaration => StmtData::Declaration(Decl(node.0)),
            NodeKind::TsGlobalDeclaration => StmtData::Declaration(Decl(node.0)),
            NodeKind::TsImportEqualsDeclaration => StmtData::Declaration(Decl(node.0)),
            NodeKind::ImportDeclaration => StmtData::ImportDeclaration(ImportDeclaration(node.0)),
            NodeKind::ExportNamedDeclaration => {
                StmtData::ExportNamedDeclaration(ExportNamedDeclaration(node.0))
            }
            NodeKind::ExportDefaultDeclaration => {
                StmtData::ExportDefaultDeclaration(ExportDefaultDeclaration(node.0))
            }
            NodeKind::ExportAllDeclaration => {
                StmtData::ExportAllDeclaration(ExportAllDeclaration(node.0))
            }
            NodeKind::TsExportAssignment => {
                StmtData::TsExportAssignment(TsExportAssignment(node.0))
            }
            NodeKind::TsNamespaceExportDeclaration => {
                StmtData::TsNamespaceExportDeclaration(TsNamespaceExportDeclaration(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn decl_data(&self, node: Decl) -> DeclData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_decl());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn expr_data(&self, node: Expr) -> ExprData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_expr());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn binding_pattern_data(&self, node: BindingPattern) -> BindingPatternData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::BindingIdentifier => {
                BindingPatternData::BindingIdentifier(BindingIdentifier(node.0))
            }
            NodeKind::IdentifierReference => {
                BindingPatternData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::MemberExpression => {
                BindingPatternData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsAsExpression => {
                BindingPatternData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsSatisfiesExpression => {
                BindingPatternData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsTypeAssertion => {
                BindingPatternData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsNonNullExpression => {
                BindingPatternData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::AssignmentPattern => {
                BindingPatternData::AssignmentPattern(AssignmentPattern(node.0))
            }
            NodeKind::BindingRestElement => {
                BindingPatternData::BindingRestElement(BindingRestElement(node.0))
            }
            NodeKind::ArrayPattern => BindingPatternData::ArrayPattern(ArrayPattern(node.0)),
            NodeKind::ObjectPattern => BindingPatternData::ObjectPattern(ObjectPattern(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn assignment_target_data(&self, node: AssignmentTarget) -> AssignmentTargetData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierReference => {
                AssignmentTargetData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::MemberExpression => {
                AssignmentTargetData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsAsExpression => {
                AssignmentTargetData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsSatisfiesExpression => {
                AssignmentTargetData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsTypeAssertion => {
                AssignmentTargetData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsNonNullExpression => {
                AssignmentTargetData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::ArrayPattern => {
                AssignmentTargetData::ArrayAssignmentTarget(ArrayPattern(node.0))
            }
            NodeKind::ObjectPattern => {
                AssignmentTargetData::ObjectAssignmentTarget(ObjectPattern(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn simple_assignment_target_data(
        &self,
        node: SimpleAssignmentTarget,
    ) -> SimpleAssignmentTargetData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_simple_assignment_target());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn property_key_data(&self, node: PropertyKey) -> PropertyKeyData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierName => PropertyKeyData::IdentifierName(IdentifierName(node.0)),
            NodeKind::IdentifierReference => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ThisExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::Super => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::MetaProperty => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::StringLiteral => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::NumericLiteral => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::BigIntLiteral => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::BooleanLiteral => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::NullLiteral => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::RegExpLiteral => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::TemplateLiteral => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::BinaryExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::LogicalExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::UnaryExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::UpdateExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::AssignmentExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ConditionalExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::SequenceExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::MemberExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::CallExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ChainExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::NewExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::TaggedTemplateExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ArrowFunctionExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ArrayExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ObjectExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ParenthesizedExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::Function => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::Class => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ImportExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::AwaitExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::YieldExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::PrivateIdentifier => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::TsAsExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::TsSatisfiesExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::TsTypeAssertion => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::TsNonNullExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::TsInstantiationExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::JsxElement => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::JsxFragment => PropertyKeyData::Expr(Expr(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn module_export_name_data(&self, node: ModuleExportName) -> ModuleExportNameData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_module_export_name());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn argument_data(&self, node: Argument) -> ArgumentData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierReference => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ThisExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::Super => ArgumentData::Expr(Expr(node.0)),
            NodeKind::MetaProperty => ArgumentData::Expr(Expr(node.0)),
            NodeKind::StringLiteral => ArgumentData::Expr(Expr(node.0)),
            NodeKind::NumericLiteral => ArgumentData::Expr(Expr(node.0)),
            NodeKind::BigIntLiteral => ArgumentData::Expr(Expr(node.0)),
            NodeKind::BooleanLiteral => ArgumentData::Expr(Expr(node.0)),
            NodeKind::NullLiteral => ArgumentData::Expr(Expr(node.0)),
            NodeKind::RegExpLiteral => ArgumentData::Expr(Expr(node.0)),
            NodeKind::TemplateLiteral => ArgumentData::Expr(Expr(node.0)),
            NodeKind::BinaryExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::LogicalExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::UnaryExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::UpdateExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::AssignmentExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ConditionalExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::SequenceExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::MemberExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::CallExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ChainExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::NewExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::TaggedTemplateExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ArrowFunctionExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ArrayExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ObjectExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ParenthesizedExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::Function => ArgumentData::Expr(Expr(node.0)),
            NodeKind::Class => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ImportExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::AwaitExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::YieldExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::PrivateIdentifier => ArgumentData::Expr(Expr(node.0)),
            NodeKind::TsAsExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::TsSatisfiesExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::TsTypeAssertion => ArgumentData::Expr(Expr(node.0)),
            NodeKind::TsNonNullExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::TsInstantiationExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::JsxElement => ArgumentData::Expr(Expr(node.0)),
            NodeKind::JsxFragment => ArgumentData::Expr(Expr(node.0)),
            NodeKind::SpreadElement => ArgumentData::SpreadElement(SpreadElement(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn object_property_kind_data(&self, node: ObjectPropertyKind) -> ObjectPropertyKindData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_object_property_kind());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn for_statement_init_data(&self, node: ForStatementInit) -> ForStatementInitData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::VariableDeclaration => {
                ForStatementInitData::VariableDeclaration(VariableDeclaration(node.0))
            }
            NodeKind::IdentifierReference => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ThisExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::Super => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::MetaProperty => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::StringLiteral => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::NumericLiteral => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::BigIntLiteral => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::BooleanLiteral => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::NullLiteral => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::RegExpLiteral => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::TemplateLiteral => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::BinaryExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::LogicalExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::UnaryExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::UpdateExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::AssignmentExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ConditionalExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::SequenceExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::MemberExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::CallExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ChainExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::NewExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::TaggedTemplateExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ArrowFunctionExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ArrayExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ObjectExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ParenthesizedExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::Function => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::Class => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ImportExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::AwaitExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::YieldExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::PrivateIdentifier => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::TsAsExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::TsSatisfiesExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::TsTypeAssertion => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::TsNonNullExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::TsInstantiationExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::JsxElement => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::JsxFragment => ForStatementInitData::Expr(Expr(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn for_statement_left_data(&self, node: ForStatementLeft) -> ForStatementLeftData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::VariableDeclaration => {
                ForStatementLeftData::VariableDeclaration(VariableDeclaration(node.0))
            }
            NodeKind::IdentifierReference => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            NodeKind::MemberExpression => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            NodeKind::TsAsExpression => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            NodeKind::TsSatisfiesExpression => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            NodeKind::TsTypeAssertion => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            NodeKind::TsNonNullExpression => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            NodeKind::ArrayPattern => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            NodeKind::ObjectPattern => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn arrow_function_body_data(&self, node: ArrowFunctionBody) -> ArrowFunctionBodyData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::FunctionBody => ArrowFunctionBodyData::FunctionBody(FunctionBody(node.0)),
            NodeKind::IdentifierReference => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ThisExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::Super => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::MetaProperty => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::StringLiteral => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::NumericLiteral => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::BigIntLiteral => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::BooleanLiteral => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::NullLiteral => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::RegExpLiteral => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::TemplateLiteral => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::BinaryExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::LogicalExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::UnaryExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::UpdateExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::AssignmentExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ConditionalExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::SequenceExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::MemberExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::CallExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ChainExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::NewExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::TaggedTemplateExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ArrowFunctionExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ArrayExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ObjectExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ParenthesizedExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::Function => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::Class => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ImportExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::AwaitExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::YieldExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::PrivateIdentifier => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::TsAsExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::TsSatisfiesExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::TsTypeAssertion => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::TsNonNullExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::TsInstantiationExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::JsxElement => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::JsxFragment => ArrowFunctionBodyData::Expr(Expr(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn formal_parameter_item_data(&self, node: FormalParameterItem) -> FormalParameterItemData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_formal_parameter_item());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn formal_parameter_pattern_data(
        &self,
        node: FormalParameterPattern,
    ) -> FormalParameterPatternData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::BindingIdentifier => {
                FormalParameterPatternData::BindingPattern(BindingPattern(node.0))
            }
            NodeKind::IdentifierReference => {
                FormalParameterPatternData::BindingPattern(BindingPattern(node.0))
            }
            NodeKind::MemberExpression => {
                FormalParameterPatternData::BindingPattern(BindingPattern(node.0))
            }
            NodeKind::TsAsExpression => {
                FormalParameterPatternData::BindingPattern(BindingPattern(node.0))
            }
            NodeKind::TsSatisfiesExpression => {
                FormalParameterPatternData::BindingPattern(BindingPattern(node.0))
            }
            NodeKind::TsTypeAssertion => {
                FormalParameterPatternData::BindingPattern(BindingPattern(node.0))
            }
            NodeKind::TsNonNullExpression => {
                FormalParameterPatternData::BindingPattern(BindingPattern(node.0))
            }
            NodeKind::AssignmentPattern => {
                FormalParameterPatternData::BindingPattern(BindingPattern(node.0))
            }
            NodeKind::BindingRestElement => {
                FormalParameterPatternData::BindingPattern(BindingPattern(node.0))
            }
            NodeKind::ArrayPattern => {
                FormalParameterPatternData::BindingPattern(BindingPattern(node.0))
            }
            NodeKind::ObjectPattern => {
                FormalParameterPatternData::BindingPattern(BindingPattern(node.0))
            }
            NodeKind::TsThisParameter => {
                FormalParameterPatternData::TsThisParameter(TsThisParameter(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn class_element_data(&self, node: ClassElement) -> ClassElementData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_class_element());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn import_declaration_specifier_data(
        &self,
        node: ImportDeclarationSpecifier,
    ) -> ImportDeclarationSpecifierData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_import_declaration_specifier());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn export_default_declaration_kind_data(
        &self,
        node: ExportDefaultDeclarationKind,
    ) -> ExportDefaultDeclarationKindData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierReference => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::ThisExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::Super => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::MetaProperty => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::StringLiteral => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::NumericLiteral => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::BigIntLiteral => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::BooleanLiteral => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::NullLiteral => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::RegExpLiteral => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TemplateLiteral => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::BinaryExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::LogicalExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::UnaryExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::UpdateExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::AssignmentExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::ConditionalExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::SequenceExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::MemberExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::CallExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::ChainExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::NewExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TaggedTemplateExpression => {
                ExportDefaultDeclarationKindData::Expr(Expr(node.0))
            }
            NodeKind::ArrowFunctionExpression => {
                ExportDefaultDeclarationKindData::Expr(Expr(node.0))
            }
            NodeKind::ArrayExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::ObjectExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::ParenthesizedExpression => {
                ExportDefaultDeclarationKindData::Expr(Expr(node.0))
            }
            NodeKind::Function => {
                let function = Function(node.0);
                if function.function_type(self) == FunctionType::FunctionDeclaration {
                    ExportDefaultDeclarationKindData::Function(function)
                } else {
                    ExportDefaultDeclarationKindData::Expr(Expr(node.0))
                }
            }
            NodeKind::Class => {
                let class = Class(node.0);
                if class.class_type(self) == ClassType::ClassDeclaration {
                    ExportDefaultDeclarationKindData::Class(class)
                } else {
                    ExportDefaultDeclarationKindData::Expr(Expr(node.0))
                }
            }
            NodeKind::ImportExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::AwaitExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::YieldExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::PrivateIdentifier => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TsAsExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TsSatisfiesExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TsTypeAssertion => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TsNonNullExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TsInstantiationExpression => {
                ExportDefaultDeclarationKindData::Expr(Expr(node.0))
            }
            NodeKind::JsxElement => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::JsxFragment => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TsFunction => {
                ExportDefaultDeclarationKindData::TsFunction(TsFunction(node.0))
            }
            NodeKind::TsInterfaceDeclaration => {
                ExportDefaultDeclarationKindData::TsInterfaceDeclaration(TsInterfaceDeclaration(
                    node.0,
                ))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_type_data(&self, node: TsType) -> TsTypeData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_ts_type());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_type_name_data(&self, node: TsTypeName) -> TsTypeNameData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_ts_type_name());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_qualified_name_left_data(
        &self,
        node: TsQualifiedNameLeft,
    ) -> TsQualifiedNameLeftData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_ts_qualified_name_left());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_import_type_qualifier_data(
        &self,
        node: TsImportTypeQualifier,
    ) -> TsImportTypeQualifierData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_ts_import_type_qualifier());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_type_query_expr_name_data(
        &self,
        node: TsTypeQueryExprName,
    ) -> TsTypeQueryExprNameData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierReference => {
                TsTypeQueryExprNameData::TsTypeName(TsTypeName(node.0))
            }
            NodeKind::TsQualifiedName => TsTypeQueryExprNameData::TsTypeName(TsTypeName(node.0)),
            NodeKind::ThisExpression => {
                TsTypeQueryExprNameData::ThisExpression(ThisExpression(node.0))
            }
            NodeKind::TsImportType => TsTypeQueryExprNameData::TsImportType(TsImportType(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_literal_data(&self, node: TsLiteral) -> TsLiteralData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_ts_literal());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_tuple_element_data(&self, node: TsTupleElement) -> TsTupleElementData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::TsAnyKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsUnknownKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsNeverKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsVoidKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsNullKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsUndefinedKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsStringKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsNumberKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsBigIntKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsBooleanKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsSymbolKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsObjectKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsIntrinsicKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsThisType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsTypeReference => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsTypeQuery => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsImportType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsLiteralType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsTemplateLiteralType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsArrayType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsIndexedAccessType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsTupleType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsOptionalType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsRestType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsJsDocNullableType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsJsDocNonNullableType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsJsDocUnknownType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsUnionType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsIntersectionType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsConditionalType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsInferType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsTypeOperator => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsParenthesizedType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsFunctionType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsConstructorType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsTypePredicate => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsTypeLiteral => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsMappedType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsNamedTupleMember => {
                TsTupleElementData::TsNamedTupleMember(TsNamedTupleMember(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_type_element_data(&self, node: TsTypeElement) -> TsTypeElementData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_ts_type_element());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_type_predicate_name_data(
        &self,
        node: TsTypePredicateName,
    ) -> TsTypePredicateNameData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_ts_type_predicate_name());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_enum_member_name_data(&self, node: TsEnumMemberName) -> TsEnumMemberNameData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_ts_enum_member_name());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_module_name_data(&self, node: TsModuleName) -> TsModuleNameData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_ts_module_name());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_module_body_data(&self, node: TsModuleBody) -> TsModuleBodyData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_ts_module_body());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_module_reference_data(&self, node: TsModuleReference) -> TsModuleReferenceData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierReference => TsModuleReferenceData::TsTypeName(TsTypeName(node.0)),
            NodeKind::TsQualifiedName => TsModuleReferenceData::TsTypeName(TsTypeName(node.0)),
            NodeKind::TsExternalModuleReference => {
                TsModuleReferenceData::TsExternalModuleReference(TsExternalModuleReference(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_element_name_data(&self, node: JsxElementName) -> JsxElementNameData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_jsx_element_name());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn jsx_member_expression_object_data(
        &self,
        node: JsxMemberExpressionObject,
    ) -> JsxMemberExpressionObjectData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_jsx_member_expression_object());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn jsx_attribute_name_data(&self, node: JsxAttributeName) -> JsxAttributeNameData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_jsx_attribute_name());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn jsx_attribute_item_data(&self, node: JsxAttributeItem) -> JsxAttributeItemData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_jsx_attribute_item());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn jsx_attribute_value_data(&self, node: JsxAttributeValue) -> JsxAttributeValueData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_jsx_attribute_value());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn jsx_expression_data(&self, node: JsxExpression) -> JsxExpressionData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierReference => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ThisExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::Super => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::MetaProperty => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::StringLiteral => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::NumericLiteral => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::BigIntLiteral => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::BooleanLiteral => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::NullLiteral => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::RegExpLiteral => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::TemplateLiteral => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::BinaryExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::LogicalExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::UnaryExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::UpdateExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::AssignmentExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ConditionalExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::SequenceExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::MemberExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::CallExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ChainExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::NewExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::TaggedTemplateExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ArrowFunctionExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ArrayExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ObjectExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ParenthesizedExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::Function => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::Class => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ImportExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::AwaitExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::YieldExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::PrivateIdentifier => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::TsAsExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::TsSatisfiesExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::TsTypeAssertion => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::TsNonNullExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::TsInstantiationExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::JsxElement => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::JsxFragment => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::JsxEmptyExpression => {
                JsxExpressionData::JsxEmptyExpression(JsxEmptyExpression(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_child_data(&self, node: JsxChild) -> JsxChildData {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0) };
        debug_assert!(kind.is_jsx_child());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
}
