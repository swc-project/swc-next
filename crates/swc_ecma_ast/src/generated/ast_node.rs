#![allow(unused)]
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
                | Self::PrivateIdentifier
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
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
            Self::IdentifierName
                | Self::StringLiteral
                | Self::TemplateLiteral
                | Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
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
/// The typed wrapper for the concrete node stored at a node ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKindData {
    Program(Program),
    Super(Super),
    ThisExpression(ThisExpression),
    StringLiteral(StringLiteral),
    NumericLiteral(NumericLiteral),
    BigIntLiteral(BigIntLiteral),
    BooleanLiteral(BooleanLiteral),
    NullLiteral(NullLiteral),
    RegExpLiteral(RegExpLiteral),
    TemplateLiteral(TemplateLiteral),
    TemplateElement(TemplateElement),
    IdentifierReference(IdentifierReference),
    PrivateIdentifier(PrivateIdentifier),
    BindingIdentifier(BindingIdentifier),
    IdentifierName(IdentifierName),
    LabelIdentifier(LabelIdentifier),
    AssignmentPattern(AssignmentPattern),
    BindingRestElement(BindingRestElement),
    ArrayPattern(ArrayPattern),
    ObjectPattern(ObjectPattern),
    BindingProperty(BindingProperty),
    BinaryExpression(BinaryExpression),
    LogicalExpression(LogicalExpression),
    ConditionalExpression(ConditionalExpression),
    UnaryExpression(UnaryExpression),
    UpdateExpression(UpdateExpression),
    AssignmentExpression(AssignmentExpression),
    VariableDeclaration(VariableDeclaration),
    VariableDeclarator(VariableDeclarator),
    IfStatement(IfStatement),
    SwitchStatement(SwitchStatement),
    SwitchCase(SwitchCase),
    ForStatement(ForStatement),
    ForInStatement(ForInStatement),
    ForOfStatement(ForOfStatement),
    TryStatement(TryStatement),
    CatchClause(CatchClause),
    BreakStatement(BreakStatement),
    ContinueStatement(ContinueStatement),
    LabeledStatement(LabeledStatement),
    ExpressionStatement(ExpressionStatement),
    ReturnStatement(ReturnStatement),
    ThrowStatement(ThrowStatement),
    DebuggerStatement(DebuggerStatement),
    EmptyStatement(EmptyStatement),
    Directive(Directive),
    Decorator(Decorator),
    Class(Class),
    ClassBody(ClassBody),
    MethodDefinition(MethodDefinition),
    PropertyDefinition(PropertyDefinition),
    StaticBlock(StaticBlock),
    Function(Function),
    FunctionBody(FunctionBody),
    BlockStatement(BlockStatement),
    WhileStatement(WhileStatement),
    DoWhileStatement(DoWhileStatement),
    WithStatement(WithStatement),
    FormalParameters(FormalParameters),
    FormalParameter(FormalParameter),
    ArrayExpression(ArrayExpression),
    ObjectExpression(ObjectExpression),
    SpreadElement(SpreadElement),
    ObjectProperty(ObjectProperty),
    ParenthesizedExpression(ParenthesizedExpression),
    ArrowFunctionExpression(ArrowFunctionExpression),
    SequenceExpression(SequenceExpression),
    MemberExpression(MemberExpression),
    CallExpression(CallExpression),
    ChainExpression(ChainExpression),
    TaggedTemplateExpression(TaggedTemplateExpression),
    NewExpression(NewExpression),
    AwaitExpression(AwaitExpression),
    YieldExpression(YieldExpression),
    MetaProperty(MetaProperty),
    ImportExpression(ImportExpression),
    ImportDeclaration(ImportDeclaration),
    ImportSpecifier(ImportSpecifier),
    ImportDefaultSpecifier(ImportDefaultSpecifier),
    ImportNamespaceSpecifier(ImportNamespaceSpecifier),
    ImportAttribute(ImportAttribute),
    ExportNamedDeclaration(ExportNamedDeclaration),
    ExportDefaultDeclaration(ExportDefaultDeclaration),
    ExportAllDeclaration(ExportAllDeclaration),
    ExportSpecifier(ExportSpecifier),
    Hashbang(Hashbang),
    TsFunction(TsFunction),
    TsMethodDefinition(TsMethodDefinition),
    TsTypeAnnotation(TsTypeAnnotation),
    TsAnyKeyword(TsAnyKeyword),
    TsUnknownKeyword(TsUnknownKeyword),
    TsNeverKeyword(TsNeverKeyword),
    TsVoidKeyword(TsVoidKeyword),
    TsNullKeyword(TsNullKeyword),
    TsUndefinedKeyword(TsUndefinedKeyword),
    TsStringKeyword(TsStringKeyword),
    TsNumberKeyword(TsNumberKeyword),
    TsBigIntKeyword(TsBigIntKeyword),
    TsBooleanKeyword(TsBooleanKeyword),
    TsSymbolKeyword(TsSymbolKeyword),
    TsObjectKeyword(TsObjectKeyword),
    TsIntrinsicKeyword(TsIntrinsicKeyword),
    TsThisType(TsThisType),
    TsTypeReference(TsTypeReference),
    TsQualifiedName(TsQualifiedName),
    TsTypeQuery(TsTypeQuery),
    TsImportType(TsImportType),
    TsTypeParameter(TsTypeParameter),
    TsTypeParameterDeclaration(TsTypeParameterDeclaration),
    TsTypeParameterInstantiation(TsTypeParameterInstantiation),
    TsLiteralType(TsLiteralType),
    TsTemplateLiteralType(TsTemplateLiteralType),
    TsArrayType(TsArrayType),
    TsIndexedAccessType(TsIndexedAccessType),
    TsTupleType(TsTupleType),
    TsNamedTupleMember(TsNamedTupleMember),
    TsOptionalType(TsOptionalType),
    TsRestType(TsRestType),
    TsJsDocNullableType(TsJsDocNullableType),
    TsJsDocNonNullableType(TsJsDocNonNullableType),
    TsJsDocUnknownType(TsJsDocUnknownType),
    TsUnionType(TsUnionType),
    TsIntersectionType(TsIntersectionType),
    TsConditionalType(TsConditionalType),
    TsInferType(TsInferType),
    TsTypeOperator(TsTypeOperator),
    TsParenthesizedType(TsParenthesizedType),
    TsFunctionType(TsFunctionType),
    TsConstructorType(TsConstructorType),
    TsTypePredicate(TsTypePredicate),
    TsTypeLiteral(TsTypeLiteral),
    TsMappedType(TsMappedType),
    TsPropertySignature(TsPropertySignature),
    TsMethodSignature(TsMethodSignature),
    TsCallSignatureDeclaration(TsCallSignatureDeclaration),
    TsConstructSignatureDeclaration(TsConstructSignatureDeclaration),
    TsIndexSignature(TsIndexSignature),
    TsTypeAliasDeclaration(TsTypeAliasDeclaration),
    TsInterfaceDeclaration(TsInterfaceDeclaration),
    TsInterfaceBody(TsInterfaceBody),
    TsInterfaceHeritage(TsInterfaceHeritage),
    TsClassImplements(TsClassImplements),
    TsEnumDeclaration(TsEnumDeclaration),
    TsEnumBody(TsEnumBody),
    TsEnumMember(TsEnumMember),
    TsModuleDeclaration(TsModuleDeclaration),
    TsModuleBlock(TsModuleBlock),
    TsGlobalDeclaration(TsGlobalDeclaration),
    TsParameterProperty(TsParameterProperty),
    TsThisParameter(TsThisParameter),
    TsAsExpression(TsAsExpression),
    TsSatisfiesExpression(TsSatisfiesExpression),
    TsTypeAssertion(TsTypeAssertion),
    TsNonNullExpression(TsNonNullExpression),
    TsInstantiationExpression(TsInstantiationExpression),
    TsExportAssignment(TsExportAssignment),
    TsNamespaceExportDeclaration(TsNamespaceExportDeclaration),
    TsImportEqualsDeclaration(TsImportEqualsDeclaration),
    TsExternalModuleReference(TsExternalModuleReference),
    JsxElement(JsxElement),
    JsxOpeningElement(JsxOpeningElement),
    JsxClosingElement(JsxClosingElement),
    JsxFragment(JsxFragment),
    JsxOpeningFragment(JsxOpeningFragment),
    JsxClosingFragment(JsxClosingFragment),
    JsxIdentifier(JsxIdentifier),
    JsxNamespacedName(JsxNamespacedName),
    JsxMemberExpression(JsxMemberExpression),
    JsxAttribute(JsxAttribute),
    JsxSpreadAttribute(JsxSpreadAttribute),
    JsxExpressionContainer(JsxExpressionContainer),
    JsxEmptyExpression(JsxEmptyExpression),
    JsxText(JsxText),
    JsxSpreadChild(JsxSpreadChild),
}
impl Program {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Program {
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
impl AstNode for Stmt {
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
impl Decl {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Decl {
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
impl Expr {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Expr {
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
impl BindingPattern {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for BindingPattern {
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
impl AssignmentTarget {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for AssignmentTarget {
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
impl SimpleAssignmentTarget {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for SimpleAssignmentTarget {
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
impl PropertyKey {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for PropertyKey {
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
impl ModuleExportName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ModuleExportName {
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
impl Argument {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Argument {
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
impl ObjectPropertyKind {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ObjectPropertyKind {
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
impl ForStatementInit {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ForStatementInit {
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
impl ForStatementLeft {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ForStatementLeft {
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
impl ArrowFunctionBody {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ArrowFunctionBody {
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
impl FormalParameterItem {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for FormalParameterItem {
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
impl FormalParameterPattern {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for FormalParameterPattern {
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
impl ClassElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ClassElement {
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
impl ImportDeclarationSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ImportDeclarationSpecifier {
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
impl ExportDefaultDeclarationKind {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for ExportDefaultDeclarationKind {
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
impl Super {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for Super {
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
impl AstNode for ThisExpression {
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
impl AstNode for StringLiteral {
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
impl AstNode for NumericLiteral {
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
impl AstNode for BigIntLiteral {
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
impl AstNode for BooleanLiteral {
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
impl AstNode for NullLiteral {
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
impl AstNode for RegExpLiteral {
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
impl AstNode for TemplateLiteral {
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
impl AstNode for TemplateElement {
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
impl AstNode for IdentifierReference {
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
impl AstNode for PrivateIdentifier {
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
impl AstNode for BindingIdentifier {
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
impl AstNode for IdentifierName {
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
impl AstNode for LabelIdentifier {
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
impl AstNode for AssignmentPattern {
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
impl AstNode for BindingRestElement {
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
impl AstNode for ArrayPattern {
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
impl AstNode for ObjectPattern {
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
impl AstNode for BindingProperty {
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
impl AstNode for BinaryExpression {
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
impl AstNode for LogicalExpression {
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
impl AstNode for ConditionalExpression {
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
impl AstNode for UnaryExpression {
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
impl AstNode for UpdateExpression {
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
impl AstNode for AssignmentExpression {
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
impl AstNode for VariableDeclaration {
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
impl AstNode for VariableDeclarator {
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
impl AstNode for IfStatement {
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
impl AstNode for SwitchStatement {
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
impl AstNode for SwitchCase {
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
impl AstNode for ForStatement {
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
impl AstNode for ForInStatement {
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
impl AstNode for ForOfStatement {
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
impl AstNode for TryStatement {
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
impl AstNode for CatchClause {
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
impl AstNode for BreakStatement {
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
impl AstNode for ContinueStatement {
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
impl AstNode for LabeledStatement {
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
impl AstNode for ExpressionStatement {
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
impl AstNode for ReturnStatement {
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
impl AstNode for ThrowStatement {
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
impl AstNode for DebuggerStatement {
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
impl AstNode for EmptyStatement {
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
impl AstNode for Directive {
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
impl AstNode for Decorator {
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
impl AstNode for Class {
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
impl AstNode for ClassBody {
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
impl AstNode for MethodDefinition {
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
impl AstNode for PropertyDefinition {
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
impl AstNode for StaticBlock {
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
impl AstNode for Function {
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
impl AstNode for FunctionBody {
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
impl AstNode for BlockStatement {
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
impl AstNode for WhileStatement {
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
impl AstNode for DoWhileStatement {
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
impl AstNode for WithStatement {
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
impl AstNode for FormalParameters {
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
impl AstNode for FormalParameter {
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
impl AstNode for ArrayExpression {
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
impl AstNode for ObjectExpression {
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
impl AstNode for SpreadElement {
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
impl AstNode for ObjectProperty {
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
impl AstNode for ParenthesizedExpression {
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
impl AstNode for ArrowFunctionExpression {
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
impl AstNode for SequenceExpression {
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
impl AstNode for MemberExpression {
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
impl AstNode for CallExpression {
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
impl AstNode for ChainExpression {
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
impl AstNode for TaggedTemplateExpression {
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
impl AstNode for NewExpression {
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
impl AstNode for AwaitExpression {
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
impl AstNode for YieldExpression {
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
impl AstNode for MetaProperty {
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
impl AstNode for ImportExpression {
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
impl AstNode for ImportDeclaration {
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
impl AstNode for ImportSpecifier {
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
impl AstNode for ImportDefaultSpecifier {
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
impl AstNode for ImportNamespaceSpecifier {
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
impl AstNode for ImportAttribute {
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
impl AstNode for ExportNamedDeclaration {
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
impl AstNode for ExportDefaultDeclaration {
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
impl AstNode for ExportAllDeclaration {
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
impl AstNode for ExportSpecifier {
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
impl AstNode for Hashbang {
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
impl AstNode for TsType {
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
impl TsTypeName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TsTypeName {
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
impl TsQualifiedNameLeft {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TsQualifiedNameLeft {
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
impl TsImportTypeQualifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TsImportTypeQualifier {
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
impl TsTypeQueryExprName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TsTypeQueryExprName {
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
impl TsLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TsLiteral {
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
impl TsTupleElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TsTupleElement {
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
impl TsTypeElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TsTypeElement {
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
impl TsTypePredicateName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TsTypePredicateName {
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
impl TsEnumMemberName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TsEnumMemberName {
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
impl TsModuleName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TsModuleName {
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
impl TsModuleBody {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TsModuleBody {
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
impl TsModuleReference {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TsModuleReference {
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
impl TsFunction {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for TsFunction {
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
impl AstNode for TsMethodDefinition {
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
impl AstNode for TsTypeAnnotation {
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
impl AstNode for TsAnyKeyword {
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
impl AstNode for TsUnknownKeyword {
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
impl AstNode for TsNeverKeyword {
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
impl AstNode for TsVoidKeyword {
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
impl AstNode for TsNullKeyword {
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
impl AstNode for TsUndefinedKeyword {
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
impl AstNode for TsStringKeyword {
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
impl AstNode for TsNumberKeyword {
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
impl AstNode for TsBigIntKeyword {
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
impl AstNode for TsBooleanKeyword {
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
impl AstNode for TsSymbolKeyword {
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
impl AstNode for TsObjectKeyword {
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
impl AstNode for TsIntrinsicKeyword {
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
impl AstNode for TsThisType {
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
impl AstNode for TsTypeReference {
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
impl AstNode for TsQualifiedName {
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
impl AstNode for TsTypeQuery {
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
impl AstNode for TsImportType {
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
impl AstNode for TsTypeParameter {
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
impl AstNode for TsTypeParameterDeclaration {
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
impl AstNode for TsTypeParameterInstantiation {
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
impl AstNode for TsLiteralType {
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
impl AstNode for TsTemplateLiteralType {
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
impl AstNode for TsArrayType {
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
impl AstNode for TsIndexedAccessType {
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
impl AstNode for TsTupleType {
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
impl AstNode for TsNamedTupleMember {
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
impl AstNode for TsOptionalType {
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
impl AstNode for TsRestType {
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
impl AstNode for TsJsDocNullableType {
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
impl AstNode for TsJsDocNonNullableType {
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
impl AstNode for TsJsDocUnknownType {
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
impl AstNode for TsUnionType {
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
impl AstNode for TsIntersectionType {
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
impl AstNode for TsConditionalType {
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
impl AstNode for TsInferType {
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
impl AstNode for TsTypeOperator {
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
impl AstNode for TsParenthesizedType {
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
impl AstNode for TsFunctionType {
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
impl AstNode for TsConstructorType {
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
impl AstNode for TsTypePredicate {
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
impl AstNode for TsTypeLiteral {
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
impl AstNode for TsMappedType {
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
impl AstNode for TsPropertySignature {
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
impl AstNode for TsMethodSignature {
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
impl AstNode for TsCallSignatureDeclaration {
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
impl AstNode for TsConstructSignatureDeclaration {
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
impl AstNode for TsIndexSignature {
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
impl AstNode for TsTypeAliasDeclaration {
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
impl AstNode for TsInterfaceDeclaration {
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
impl AstNode for TsInterfaceBody {
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
impl AstNode for TsInterfaceHeritage {
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
impl AstNode for TsClassImplements {
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
impl AstNode for TsEnumDeclaration {
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
impl AstNode for TsEnumBody {
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
impl AstNode for TsEnumMember {
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
impl AstNode for TsModuleDeclaration {
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
impl AstNode for TsModuleBlock {
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
impl AstNode for TsGlobalDeclaration {
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
impl AstNode for TsParameterProperty {
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
impl AstNode for TsThisParameter {
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
impl AstNode for TsAsExpression {
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
impl AstNode for TsSatisfiesExpression {
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
impl AstNode for TsTypeAssertion {
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
impl AstNode for TsNonNullExpression {
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
impl AstNode for TsInstantiationExpression {
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
impl AstNode for TsExportAssignment {
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
impl AstNode for TsNamespaceExportDeclaration {
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
impl AstNode for TsImportEqualsDeclaration {
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
impl AstNode for TsExternalModuleReference {
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
impl AstNode for JsxElementName {
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
impl JsxMemberExpressionObject {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JsxMemberExpressionObject {
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
impl JsxAttributeName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JsxAttributeName {
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
impl JsxAttributeItem {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JsxAttributeItem {
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
impl JsxAttributeValue {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JsxAttributeValue {
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
impl JsxExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JsxExpression {
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
impl JsxChild {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JsxChild {
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
impl JsxElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
}
impl AstNode for JsxElement {
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
impl AstNode for JsxOpeningElement {
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
impl AstNode for JsxClosingElement {
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
impl AstNode for JsxFragment {
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
impl AstNode for JsxOpeningFragment {
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
impl AstNode for JsxClosingFragment {
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
impl AstNode for JsxIdentifier {
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
impl AstNode for JsxNamespacedName {
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
impl AstNode for JsxMemberExpression {
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
impl AstNode for JsxAttribute {
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
impl AstNode for JsxSpreadAttribute {
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
impl AstNode for JsxExpressionContainer {
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
impl AstNode for JsxEmptyExpression {
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
impl AstNode for JsxText {
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
impl AstNode for JsxSpreadChild {
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
        match self.node_kind(node_id) {
            NodeKind::Program => NodeKindData::Program(Program(node_id)),
            NodeKind::Super => NodeKindData::Super(Super(node_id)),
            NodeKind::ThisExpression => NodeKindData::ThisExpression(ThisExpression(node_id)),
            NodeKind::StringLiteral => NodeKindData::StringLiteral(StringLiteral(node_id)),
            NodeKind::NumericLiteral => NodeKindData::NumericLiteral(NumericLiteral(node_id)),
            NodeKind::BigIntLiteral => NodeKindData::BigIntLiteral(BigIntLiteral(node_id)),
            NodeKind::BooleanLiteral => NodeKindData::BooleanLiteral(BooleanLiteral(node_id)),
            NodeKind::NullLiteral => NodeKindData::NullLiteral(NullLiteral(node_id)),
            NodeKind::RegExpLiteral => NodeKindData::RegExpLiteral(RegExpLiteral(node_id)),
            NodeKind::TemplateLiteral => NodeKindData::TemplateLiteral(TemplateLiteral(node_id)),
            NodeKind::TemplateElement => NodeKindData::TemplateElement(TemplateElement(node_id)),
            NodeKind::IdentifierReference => {
                NodeKindData::IdentifierReference(IdentifierReference(node_id))
            }
            NodeKind::PrivateIdentifier => {
                NodeKindData::PrivateIdentifier(PrivateIdentifier(node_id))
            }
            NodeKind::BindingIdentifier => {
                NodeKindData::BindingIdentifier(BindingIdentifier(node_id))
            }
            NodeKind::IdentifierName => NodeKindData::IdentifierName(IdentifierName(node_id)),
            NodeKind::LabelIdentifier => NodeKindData::LabelIdentifier(LabelIdentifier(node_id)),
            NodeKind::AssignmentPattern => {
                NodeKindData::AssignmentPattern(AssignmentPattern(node_id))
            }
            NodeKind::BindingRestElement => {
                NodeKindData::BindingRestElement(BindingRestElement(node_id))
            }
            NodeKind::ArrayPattern => NodeKindData::ArrayPattern(ArrayPattern(node_id)),
            NodeKind::ObjectPattern => NodeKindData::ObjectPattern(ObjectPattern(node_id)),
            NodeKind::BindingProperty => NodeKindData::BindingProperty(BindingProperty(node_id)),
            NodeKind::BinaryExpression => NodeKindData::BinaryExpression(BinaryExpression(node_id)),
            NodeKind::LogicalExpression => {
                NodeKindData::LogicalExpression(LogicalExpression(node_id))
            }
            NodeKind::ConditionalExpression => {
                NodeKindData::ConditionalExpression(ConditionalExpression(node_id))
            }
            NodeKind::UnaryExpression => NodeKindData::UnaryExpression(UnaryExpression(node_id)),
            NodeKind::UpdateExpression => NodeKindData::UpdateExpression(UpdateExpression(node_id)),
            NodeKind::AssignmentExpression => {
                NodeKindData::AssignmentExpression(AssignmentExpression(node_id))
            }
            NodeKind::VariableDeclaration => {
                NodeKindData::VariableDeclaration(VariableDeclaration(node_id))
            }
            NodeKind::VariableDeclarator => {
                NodeKindData::VariableDeclarator(VariableDeclarator(node_id))
            }
            NodeKind::IfStatement => NodeKindData::IfStatement(IfStatement(node_id)),
            NodeKind::SwitchStatement => NodeKindData::SwitchStatement(SwitchStatement(node_id)),
            NodeKind::SwitchCase => NodeKindData::SwitchCase(SwitchCase(node_id)),
            NodeKind::ForStatement => NodeKindData::ForStatement(ForStatement(node_id)),
            NodeKind::ForInStatement => NodeKindData::ForInStatement(ForInStatement(node_id)),
            NodeKind::ForOfStatement => NodeKindData::ForOfStatement(ForOfStatement(node_id)),
            NodeKind::TryStatement => NodeKindData::TryStatement(TryStatement(node_id)),
            NodeKind::CatchClause => NodeKindData::CatchClause(CatchClause(node_id)),
            NodeKind::BreakStatement => NodeKindData::BreakStatement(BreakStatement(node_id)),
            NodeKind::ContinueStatement => {
                NodeKindData::ContinueStatement(ContinueStatement(node_id))
            }
            NodeKind::LabeledStatement => NodeKindData::LabeledStatement(LabeledStatement(node_id)),
            NodeKind::ExpressionStatement => {
                NodeKindData::ExpressionStatement(ExpressionStatement(node_id))
            }
            NodeKind::ReturnStatement => NodeKindData::ReturnStatement(ReturnStatement(node_id)),
            NodeKind::ThrowStatement => NodeKindData::ThrowStatement(ThrowStatement(node_id)),
            NodeKind::DebuggerStatement => {
                NodeKindData::DebuggerStatement(DebuggerStatement(node_id))
            }
            NodeKind::EmptyStatement => NodeKindData::EmptyStatement(EmptyStatement(node_id)),
            NodeKind::Directive => NodeKindData::Directive(Directive(node_id)),
            NodeKind::Decorator => NodeKindData::Decorator(Decorator(node_id)),
            NodeKind::Class => NodeKindData::Class(Class(node_id)),
            NodeKind::ClassBody => NodeKindData::ClassBody(ClassBody(node_id)),
            NodeKind::MethodDefinition => NodeKindData::MethodDefinition(MethodDefinition(node_id)),
            NodeKind::PropertyDefinition => {
                NodeKindData::PropertyDefinition(PropertyDefinition(node_id))
            }
            NodeKind::StaticBlock => NodeKindData::StaticBlock(StaticBlock(node_id)),
            NodeKind::Function => NodeKindData::Function(Function(node_id)),
            NodeKind::FunctionBody => NodeKindData::FunctionBody(FunctionBody(node_id)),
            NodeKind::BlockStatement => NodeKindData::BlockStatement(BlockStatement(node_id)),
            NodeKind::WhileStatement => NodeKindData::WhileStatement(WhileStatement(node_id)),
            NodeKind::DoWhileStatement => NodeKindData::DoWhileStatement(DoWhileStatement(node_id)),
            NodeKind::WithStatement => NodeKindData::WithStatement(WithStatement(node_id)),
            NodeKind::FormalParameters => NodeKindData::FormalParameters(FormalParameters(node_id)),
            NodeKind::FormalParameter => NodeKindData::FormalParameter(FormalParameter(node_id)),
            NodeKind::ArrayExpression => NodeKindData::ArrayExpression(ArrayExpression(node_id)),
            NodeKind::ObjectExpression => NodeKindData::ObjectExpression(ObjectExpression(node_id)),
            NodeKind::SpreadElement => NodeKindData::SpreadElement(SpreadElement(node_id)),
            NodeKind::ObjectProperty => NodeKindData::ObjectProperty(ObjectProperty(node_id)),
            NodeKind::ParenthesizedExpression => {
                NodeKindData::ParenthesizedExpression(ParenthesizedExpression(node_id))
            }
            NodeKind::ArrowFunctionExpression => {
                NodeKindData::ArrowFunctionExpression(ArrowFunctionExpression(node_id))
            }
            NodeKind::SequenceExpression => {
                NodeKindData::SequenceExpression(SequenceExpression(node_id))
            }
            NodeKind::MemberExpression => NodeKindData::MemberExpression(MemberExpression(node_id)),
            NodeKind::CallExpression => NodeKindData::CallExpression(CallExpression(node_id)),
            NodeKind::ChainExpression => NodeKindData::ChainExpression(ChainExpression(node_id)),
            NodeKind::TaggedTemplateExpression => {
                NodeKindData::TaggedTemplateExpression(TaggedTemplateExpression(node_id))
            }
            NodeKind::NewExpression => NodeKindData::NewExpression(NewExpression(node_id)),
            NodeKind::AwaitExpression => NodeKindData::AwaitExpression(AwaitExpression(node_id)),
            NodeKind::YieldExpression => NodeKindData::YieldExpression(YieldExpression(node_id)),
            NodeKind::MetaProperty => NodeKindData::MetaProperty(MetaProperty(node_id)),
            NodeKind::ImportExpression => NodeKindData::ImportExpression(ImportExpression(node_id)),
            NodeKind::ImportDeclaration => {
                NodeKindData::ImportDeclaration(ImportDeclaration(node_id))
            }
            NodeKind::ImportSpecifier => NodeKindData::ImportSpecifier(ImportSpecifier(node_id)),
            NodeKind::ImportDefaultSpecifier => {
                NodeKindData::ImportDefaultSpecifier(ImportDefaultSpecifier(node_id))
            }
            NodeKind::ImportNamespaceSpecifier => {
                NodeKindData::ImportNamespaceSpecifier(ImportNamespaceSpecifier(node_id))
            }
            NodeKind::ImportAttribute => NodeKindData::ImportAttribute(ImportAttribute(node_id)),
            NodeKind::ExportNamedDeclaration => {
                NodeKindData::ExportNamedDeclaration(ExportNamedDeclaration(node_id))
            }
            NodeKind::ExportDefaultDeclaration => {
                NodeKindData::ExportDefaultDeclaration(ExportDefaultDeclaration(node_id))
            }
            NodeKind::ExportAllDeclaration => {
                NodeKindData::ExportAllDeclaration(ExportAllDeclaration(node_id))
            }
            NodeKind::ExportSpecifier => NodeKindData::ExportSpecifier(ExportSpecifier(node_id)),
            NodeKind::Hashbang => NodeKindData::Hashbang(Hashbang(node_id)),
            NodeKind::TsFunction => NodeKindData::TsFunction(TsFunction(node_id)),
            NodeKind::TsMethodDefinition => {
                NodeKindData::TsMethodDefinition(TsMethodDefinition(node_id))
            }
            NodeKind::TsTypeAnnotation => NodeKindData::TsTypeAnnotation(TsTypeAnnotation(node_id)),
            NodeKind::TsAnyKeyword => NodeKindData::TsAnyKeyword(TsAnyKeyword(node_id)),
            NodeKind::TsUnknownKeyword => NodeKindData::TsUnknownKeyword(TsUnknownKeyword(node_id)),
            NodeKind::TsNeverKeyword => NodeKindData::TsNeverKeyword(TsNeverKeyword(node_id)),
            NodeKind::TsVoidKeyword => NodeKindData::TsVoidKeyword(TsVoidKeyword(node_id)),
            NodeKind::TsNullKeyword => NodeKindData::TsNullKeyword(TsNullKeyword(node_id)),
            NodeKind::TsUndefinedKeyword => {
                NodeKindData::TsUndefinedKeyword(TsUndefinedKeyword(node_id))
            }
            NodeKind::TsStringKeyword => NodeKindData::TsStringKeyword(TsStringKeyword(node_id)),
            NodeKind::TsNumberKeyword => NodeKindData::TsNumberKeyword(TsNumberKeyword(node_id)),
            NodeKind::TsBigIntKeyword => NodeKindData::TsBigIntKeyword(TsBigIntKeyword(node_id)),
            NodeKind::TsBooleanKeyword => NodeKindData::TsBooleanKeyword(TsBooleanKeyword(node_id)),
            NodeKind::TsSymbolKeyword => NodeKindData::TsSymbolKeyword(TsSymbolKeyword(node_id)),
            NodeKind::TsObjectKeyword => NodeKindData::TsObjectKeyword(TsObjectKeyword(node_id)),
            NodeKind::TsIntrinsicKeyword => {
                NodeKindData::TsIntrinsicKeyword(TsIntrinsicKeyword(node_id))
            }
            NodeKind::TsThisType => NodeKindData::TsThisType(TsThisType(node_id)),
            NodeKind::TsTypeReference => NodeKindData::TsTypeReference(TsTypeReference(node_id)),
            NodeKind::TsQualifiedName => NodeKindData::TsQualifiedName(TsQualifiedName(node_id)),
            NodeKind::TsTypeQuery => NodeKindData::TsTypeQuery(TsTypeQuery(node_id)),
            NodeKind::TsImportType => NodeKindData::TsImportType(TsImportType(node_id)),
            NodeKind::TsTypeParameter => NodeKindData::TsTypeParameter(TsTypeParameter(node_id)),
            NodeKind::TsTypeParameterDeclaration => {
                NodeKindData::TsTypeParameterDeclaration(TsTypeParameterDeclaration(node_id))
            }
            NodeKind::TsTypeParameterInstantiation => {
                NodeKindData::TsTypeParameterInstantiation(TsTypeParameterInstantiation(node_id))
            }
            NodeKind::TsLiteralType => NodeKindData::TsLiteralType(TsLiteralType(node_id)),
            NodeKind::TsTemplateLiteralType => {
                NodeKindData::TsTemplateLiteralType(TsTemplateLiteralType(node_id))
            }
            NodeKind::TsArrayType => NodeKindData::TsArrayType(TsArrayType(node_id)),
            NodeKind::TsIndexedAccessType => {
                NodeKindData::TsIndexedAccessType(TsIndexedAccessType(node_id))
            }
            NodeKind::TsTupleType => NodeKindData::TsTupleType(TsTupleType(node_id)),
            NodeKind::TsNamedTupleMember => {
                NodeKindData::TsNamedTupleMember(TsNamedTupleMember(node_id))
            }
            NodeKind::TsOptionalType => NodeKindData::TsOptionalType(TsOptionalType(node_id)),
            NodeKind::TsRestType => NodeKindData::TsRestType(TsRestType(node_id)),
            NodeKind::TsJsDocNullableType => {
                NodeKindData::TsJsDocNullableType(TsJsDocNullableType(node_id))
            }
            NodeKind::TsJsDocNonNullableType => {
                NodeKindData::TsJsDocNonNullableType(TsJsDocNonNullableType(node_id))
            }
            NodeKind::TsJsDocUnknownType => {
                NodeKindData::TsJsDocUnknownType(TsJsDocUnknownType(node_id))
            }
            NodeKind::TsUnionType => NodeKindData::TsUnionType(TsUnionType(node_id)),
            NodeKind::TsIntersectionType => {
                NodeKindData::TsIntersectionType(TsIntersectionType(node_id))
            }
            NodeKind::TsConditionalType => {
                NodeKindData::TsConditionalType(TsConditionalType(node_id))
            }
            NodeKind::TsInferType => NodeKindData::TsInferType(TsInferType(node_id)),
            NodeKind::TsTypeOperator => NodeKindData::TsTypeOperator(TsTypeOperator(node_id)),
            NodeKind::TsParenthesizedType => {
                NodeKindData::TsParenthesizedType(TsParenthesizedType(node_id))
            }
            NodeKind::TsFunctionType => NodeKindData::TsFunctionType(TsFunctionType(node_id)),
            NodeKind::TsConstructorType => {
                NodeKindData::TsConstructorType(TsConstructorType(node_id))
            }
            NodeKind::TsTypePredicate => NodeKindData::TsTypePredicate(TsTypePredicate(node_id)),
            NodeKind::TsTypeLiteral => NodeKindData::TsTypeLiteral(TsTypeLiteral(node_id)),
            NodeKind::TsMappedType => NodeKindData::TsMappedType(TsMappedType(node_id)),
            NodeKind::TsPropertySignature => {
                NodeKindData::TsPropertySignature(TsPropertySignature(node_id))
            }
            NodeKind::TsMethodSignature => {
                NodeKindData::TsMethodSignature(TsMethodSignature(node_id))
            }
            NodeKind::TsCallSignatureDeclaration => {
                NodeKindData::TsCallSignatureDeclaration(TsCallSignatureDeclaration(node_id))
            }
            NodeKind::TsConstructSignatureDeclaration => {
                NodeKindData::TsConstructSignatureDeclaration(TsConstructSignatureDeclaration(
                    node_id,
                ))
            }
            NodeKind::TsIndexSignature => NodeKindData::TsIndexSignature(TsIndexSignature(node_id)),
            NodeKind::TsTypeAliasDeclaration => {
                NodeKindData::TsTypeAliasDeclaration(TsTypeAliasDeclaration(node_id))
            }
            NodeKind::TsInterfaceDeclaration => {
                NodeKindData::TsInterfaceDeclaration(TsInterfaceDeclaration(node_id))
            }
            NodeKind::TsInterfaceBody => NodeKindData::TsInterfaceBody(TsInterfaceBody(node_id)),
            NodeKind::TsInterfaceHeritage => {
                NodeKindData::TsInterfaceHeritage(TsInterfaceHeritage(node_id))
            }
            NodeKind::TsClassImplements => {
                NodeKindData::TsClassImplements(TsClassImplements(node_id))
            }
            NodeKind::TsEnumDeclaration => {
                NodeKindData::TsEnumDeclaration(TsEnumDeclaration(node_id))
            }
            NodeKind::TsEnumBody => NodeKindData::TsEnumBody(TsEnumBody(node_id)),
            NodeKind::TsEnumMember => NodeKindData::TsEnumMember(TsEnumMember(node_id)),
            NodeKind::TsModuleDeclaration => {
                NodeKindData::TsModuleDeclaration(TsModuleDeclaration(node_id))
            }
            NodeKind::TsModuleBlock => NodeKindData::TsModuleBlock(TsModuleBlock(node_id)),
            NodeKind::TsGlobalDeclaration => {
                NodeKindData::TsGlobalDeclaration(TsGlobalDeclaration(node_id))
            }
            NodeKind::TsParameterProperty => {
                NodeKindData::TsParameterProperty(TsParameterProperty(node_id))
            }
            NodeKind::TsThisParameter => NodeKindData::TsThisParameter(TsThisParameter(node_id)),
            NodeKind::TsAsExpression => NodeKindData::TsAsExpression(TsAsExpression(node_id)),
            NodeKind::TsSatisfiesExpression => {
                NodeKindData::TsSatisfiesExpression(TsSatisfiesExpression(node_id))
            }
            NodeKind::TsTypeAssertion => NodeKindData::TsTypeAssertion(TsTypeAssertion(node_id)),
            NodeKind::TsNonNullExpression => {
                NodeKindData::TsNonNullExpression(TsNonNullExpression(node_id))
            }
            NodeKind::TsInstantiationExpression => {
                NodeKindData::TsInstantiationExpression(TsInstantiationExpression(node_id))
            }
            NodeKind::TsExportAssignment => {
                NodeKindData::TsExportAssignment(TsExportAssignment(node_id))
            }
            NodeKind::TsNamespaceExportDeclaration => {
                NodeKindData::TsNamespaceExportDeclaration(TsNamespaceExportDeclaration(node_id))
            }
            NodeKind::TsImportEqualsDeclaration => {
                NodeKindData::TsImportEqualsDeclaration(TsImportEqualsDeclaration(node_id))
            }
            NodeKind::TsExternalModuleReference => {
                NodeKindData::TsExternalModuleReference(TsExternalModuleReference(node_id))
            }
            NodeKind::JsxElement => NodeKindData::JsxElement(JsxElement(node_id)),
            NodeKind::JsxOpeningElement => {
                NodeKindData::JsxOpeningElement(JsxOpeningElement(node_id))
            }
            NodeKind::JsxClosingElement => {
                NodeKindData::JsxClosingElement(JsxClosingElement(node_id))
            }
            NodeKind::JsxFragment => NodeKindData::JsxFragment(JsxFragment(node_id)),
            NodeKind::JsxOpeningFragment => {
                NodeKindData::JsxOpeningFragment(JsxOpeningFragment(node_id))
            }
            NodeKind::JsxClosingFragment => {
                NodeKindData::JsxClosingFragment(JsxClosingFragment(node_id))
            }
            NodeKind::JsxIdentifier => NodeKindData::JsxIdentifier(JsxIdentifier(node_id)),
            NodeKind::JsxNamespacedName => {
                NodeKindData::JsxNamespacedName(JsxNamespacedName(node_id))
            }
            NodeKind::JsxMemberExpression => {
                NodeKindData::JsxMemberExpression(JsxMemberExpression(node_id))
            }
            NodeKind::JsxAttribute => NodeKindData::JsxAttribute(JsxAttribute(node_id)),
            NodeKind::JsxSpreadAttribute => {
                NodeKindData::JsxSpreadAttribute(JsxSpreadAttribute(node_id))
            }
            NodeKind::JsxExpressionContainer => {
                NodeKindData::JsxExpressionContainer(JsxExpressionContainer(node_id))
            }
            NodeKind::JsxEmptyExpression => {
                NodeKindData::JsxEmptyExpression(JsxEmptyExpression(node_id))
            }
            NodeKind::JsxText => NodeKindData::JsxText(JsxText(node_id)),
            NodeKind::JsxSpreadChild => NodeKindData::JsxSpreadChild(JsxSpreadChild(node_id)),
        }
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
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::VariableDeclaration => {
                DeclData::VariableDeclaration(VariableDeclaration(node.0))
            }
            NodeKind::Function => DeclData::Function(Function(node.0)),
            NodeKind::Class => DeclData::Class(Class(node.0)),
            NodeKind::TsFunction => DeclData::TsFunction(TsFunction(node.0)),
            NodeKind::TsTypeAliasDeclaration => {
                DeclData::TsTypeAliasDeclaration(TsTypeAliasDeclaration(node.0))
            }
            NodeKind::TsInterfaceDeclaration => {
                DeclData::TsInterfaceDeclaration(TsInterfaceDeclaration(node.0))
            }
            NodeKind::TsEnumDeclaration => DeclData::TsEnumDeclaration(TsEnumDeclaration(node.0)),
            NodeKind::TsModuleDeclaration => {
                DeclData::TsModuleDeclaration(TsModuleDeclaration(node.0))
            }
            NodeKind::TsGlobalDeclaration => {
                DeclData::TsGlobalDeclaration(TsGlobalDeclaration(node.0))
            }
            NodeKind::TsImportEqualsDeclaration => {
                DeclData::TsImportEqualsDeclaration(TsImportEqualsDeclaration(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn expr_data(&self, node: Expr) -> ExprData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierReference => {
                ExprData::IdentifierReference(IdentifierReference(node.0))
            }
            NodeKind::ThisExpression => ExprData::ThisExpression(ThisExpression(node.0)),
            NodeKind::Super => ExprData::Super(Super(node.0)),
            NodeKind::MetaProperty => ExprData::MetaProperty(MetaProperty(node.0)),
            NodeKind::StringLiteral => ExprData::StringLiteral(StringLiteral(node.0)),
            NodeKind::NumericLiteral => ExprData::NumericLiteral(NumericLiteral(node.0)),
            NodeKind::BigIntLiteral => ExprData::BigIntLiteral(BigIntLiteral(node.0)),
            NodeKind::BooleanLiteral => ExprData::BooleanLiteral(BooleanLiteral(node.0)),
            NodeKind::NullLiteral => ExprData::NullLiteral(NullLiteral(node.0)),
            NodeKind::RegExpLiteral => ExprData::RegExpLiteral(RegExpLiteral(node.0)),
            NodeKind::TemplateLiteral => ExprData::TemplateLiteral(TemplateLiteral(node.0)),
            NodeKind::BinaryExpression => ExprData::BinaryExpression(BinaryExpression(node.0)),
            NodeKind::LogicalExpression => ExprData::LogicalExpression(LogicalExpression(node.0)),
            NodeKind::UnaryExpression => ExprData::UnaryExpression(UnaryExpression(node.0)),
            NodeKind::UpdateExpression => ExprData::UpdateExpression(UpdateExpression(node.0)),
            NodeKind::AssignmentExpression => {
                ExprData::AssignmentExpression(AssignmentExpression(node.0))
            }
            NodeKind::ConditionalExpression => {
                ExprData::ConditionalExpression(ConditionalExpression(node.0))
            }
            NodeKind::SequenceExpression => {
                ExprData::SequenceExpression(SequenceExpression(node.0))
            }
            NodeKind::MemberExpression => ExprData::MemberExpression(MemberExpression(node.0)),
            NodeKind::CallExpression => ExprData::CallExpression(CallExpression(node.0)),
            NodeKind::ChainExpression => ExprData::ChainExpression(ChainExpression(node.0)),
            NodeKind::NewExpression => ExprData::NewExpression(NewExpression(node.0)),
            NodeKind::TaggedTemplateExpression => {
                ExprData::TaggedTemplateExpression(TaggedTemplateExpression(node.0))
            }
            NodeKind::ArrowFunctionExpression => {
                ExprData::ArrowFunctionExpression(ArrowFunctionExpression(node.0))
            }
            NodeKind::ArrayExpression => ExprData::ArrayExpression(ArrayExpression(node.0)),
            NodeKind::ObjectExpression => ExprData::ObjectExpression(ObjectExpression(node.0)),
            NodeKind::ParenthesizedExpression => {
                ExprData::ParenthesizedExpression(ParenthesizedExpression(node.0))
            }
            NodeKind::Function => ExprData::Function(Function(node.0)),
            NodeKind::Class => ExprData::Class(Class(node.0)),
            NodeKind::ImportExpression => ExprData::ImportExpression(ImportExpression(node.0)),
            NodeKind::AwaitExpression => ExprData::AwaitExpression(AwaitExpression(node.0)),
            NodeKind::YieldExpression => ExprData::YieldExpression(YieldExpression(node.0)),
            NodeKind::PrivateIdentifier => ExprData::PrivateIdentifier(PrivateIdentifier(node.0)),
            NodeKind::TsAsExpression => ExprData::TsAsExpression(TsAsExpression(node.0)),
            NodeKind::TsSatisfiesExpression => {
                ExprData::TsSatisfiesExpression(TsSatisfiesExpression(node.0))
            }
            NodeKind::TsTypeAssertion => ExprData::TsTypeAssertion(TsTypeAssertion(node.0)),
            NodeKind::TsNonNullExpression => {
                ExprData::TsNonNullExpression(TsNonNullExpression(node.0))
            }
            NodeKind::TsInstantiationExpression => {
                ExprData::TsInstantiationExpression(TsInstantiationExpression(node.0))
            }
            NodeKind::JsxElement => ExprData::JsxElement(JsxElement(node.0)),
            NodeKind::JsxFragment => ExprData::JsxFragment(JsxFragment(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
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
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierReference => {
                SimpleAssignmentTargetData::IdentifierReference(IdentifierReference(node.0))
            }
            NodeKind::MemberExpression => {
                SimpleAssignmentTargetData::MemberExpression(MemberExpression(node.0))
            }
            NodeKind::TsAsExpression => {
                SimpleAssignmentTargetData::TsAsExpression(TsAsExpression(node.0))
            }
            NodeKind::TsSatisfiesExpression => {
                SimpleAssignmentTargetData::TsSatisfiesExpression(TsSatisfiesExpression(node.0))
            }
            NodeKind::TsTypeAssertion => {
                SimpleAssignmentTargetData::TsTypeAssertion(TsTypeAssertion(node.0))
            }
            NodeKind::TsNonNullExpression => {
                SimpleAssignmentTargetData::TsNonNullExpression(TsNonNullExpression(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn property_key_data(&self, node: PropertyKey) -> PropertyKeyData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierName => PropertyKeyData::IdentifierName(IdentifierName(node.0)),
            NodeKind::PrivateIdentifier => {
                PropertyKeyData::PrivateIdentifier(PrivateIdentifier(node.0))
            }
            NodeKind::StringLiteral => PropertyKeyData::StringLiteral(StringLiteral(node.0)),
            NodeKind::NumericLiteral => PropertyKeyData::NumericLiteral(NumericLiteral(node.0)),
            NodeKind::BigIntLiteral => PropertyKeyData::BigIntLiteral(BigIntLiteral(node.0)),
            NodeKind::IdentifierReference => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ThisExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::Super => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::MetaProperty => PropertyKeyData::Expr(Expr(node.0)),
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
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierName => {
                ModuleExportNameData::IdentifierName(IdentifierName(node.0))
            }
            NodeKind::StringLiteral => ModuleExportNameData::StringLiteral(StringLiteral(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
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
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::ObjectProperty => {
                ObjectPropertyKindData::ObjectProperty(ObjectProperty(node.0))
            }
            NodeKind::SpreadElement => ObjectPropertyKindData::SpreadElement(SpreadElement(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
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
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::FormalParameter => {
                FormalParameterItemData::FormalParameter(FormalParameter(node.0))
            }
            NodeKind::TsParameterProperty => {
                FormalParameterItemData::TsParameterProperty(TsParameterProperty(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
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
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::MethodDefinition => {
                ClassElementData::MethodDefinition(MethodDefinition(node.0))
            }
            NodeKind::PropertyDefinition => {
                ClassElementData::PropertyDefinition(PropertyDefinition(node.0))
            }
            NodeKind::StaticBlock => ClassElementData::StaticBlock(StaticBlock(node.0)),
            NodeKind::TsMethodDefinition => {
                ClassElementData::TsMethodDefinition(TsMethodDefinition(node.0))
            }
            NodeKind::TsIndexSignature => {
                ClassElementData::TsIndexSignature(TsIndexSignature(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn import_declaration_specifier_data(
        &self,
        node: ImportDeclarationSpecifier,
    ) -> ImportDeclarationSpecifierData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::ImportSpecifier => {
                ImportDeclarationSpecifierData::ImportSpecifier(ImportSpecifier(node.0))
            }
            NodeKind::ImportDefaultSpecifier => {
                ImportDeclarationSpecifierData::ImportDefaultSpecifier(ImportDefaultSpecifier(
                    node.0,
                ))
            }
            NodeKind::ImportNamespaceSpecifier => {
                ImportDeclarationSpecifierData::ImportNamespaceSpecifier(ImportNamespaceSpecifier(
                    node.0,
                ))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
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
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::TsAnyKeyword => TsTypeData::TsAnyKeyword(TsAnyKeyword(node.0)),
            NodeKind::TsUnknownKeyword => TsTypeData::TsUnknownKeyword(TsUnknownKeyword(node.0)),
            NodeKind::TsNeverKeyword => TsTypeData::TsNeverKeyword(TsNeverKeyword(node.0)),
            NodeKind::TsVoidKeyword => TsTypeData::TsVoidKeyword(TsVoidKeyword(node.0)),
            NodeKind::TsNullKeyword => TsTypeData::TsNullKeyword(TsNullKeyword(node.0)),
            NodeKind::TsUndefinedKeyword => {
                TsTypeData::TsUndefinedKeyword(TsUndefinedKeyword(node.0))
            }
            NodeKind::TsStringKeyword => TsTypeData::TsStringKeyword(TsStringKeyword(node.0)),
            NodeKind::TsNumberKeyword => TsTypeData::TsNumberKeyword(TsNumberKeyword(node.0)),
            NodeKind::TsBigIntKeyword => TsTypeData::TsBigIntKeyword(TsBigIntKeyword(node.0)),
            NodeKind::TsBooleanKeyword => TsTypeData::TsBooleanKeyword(TsBooleanKeyword(node.0)),
            NodeKind::TsSymbolKeyword => TsTypeData::TsSymbolKeyword(TsSymbolKeyword(node.0)),
            NodeKind::TsObjectKeyword => TsTypeData::TsObjectKeyword(TsObjectKeyword(node.0)),
            NodeKind::TsIntrinsicKeyword => {
                TsTypeData::TsIntrinsicKeyword(TsIntrinsicKeyword(node.0))
            }
            NodeKind::TsThisType => TsTypeData::TsThisType(TsThisType(node.0)),
            NodeKind::TsTypeReference => TsTypeData::TsTypeReference(TsTypeReference(node.0)),
            NodeKind::TsTypeQuery => TsTypeData::TsTypeQuery(TsTypeQuery(node.0)),
            NodeKind::TsImportType => TsTypeData::TsImportType(TsImportType(node.0)),
            NodeKind::TsLiteralType => TsTypeData::TsLiteralType(TsLiteralType(node.0)),
            NodeKind::TsTemplateLiteralType => {
                TsTypeData::TsTemplateLiteralType(TsTemplateLiteralType(node.0))
            }
            NodeKind::TsArrayType => TsTypeData::TsArrayType(TsArrayType(node.0)),
            NodeKind::TsIndexedAccessType => {
                TsTypeData::TsIndexedAccessType(TsIndexedAccessType(node.0))
            }
            NodeKind::TsTupleType => TsTypeData::TsTupleType(TsTupleType(node.0)),
            NodeKind::TsOptionalType => TsTypeData::TsOptionalType(TsOptionalType(node.0)),
            NodeKind::TsRestType => TsTypeData::TsRestType(TsRestType(node.0)),
            NodeKind::TsJsDocNullableType => {
                TsTypeData::TsJsDocNullableType(TsJsDocNullableType(node.0))
            }
            NodeKind::TsJsDocNonNullableType => {
                TsTypeData::TsJsDocNonNullableType(TsJsDocNonNullableType(node.0))
            }
            NodeKind::TsJsDocUnknownType => {
                TsTypeData::TsJsDocUnknownType(TsJsDocUnknownType(node.0))
            }
            NodeKind::TsUnionType => TsTypeData::TsUnionType(TsUnionType(node.0)),
            NodeKind::TsIntersectionType => {
                TsTypeData::TsIntersectionType(TsIntersectionType(node.0))
            }
            NodeKind::TsConditionalType => TsTypeData::TsConditionalType(TsConditionalType(node.0)),
            NodeKind::TsInferType => TsTypeData::TsInferType(TsInferType(node.0)),
            NodeKind::TsTypeOperator => TsTypeData::TsTypeOperator(TsTypeOperator(node.0)),
            NodeKind::TsParenthesizedType => {
                TsTypeData::TsParenthesizedType(TsParenthesizedType(node.0))
            }
            NodeKind::TsFunctionType => TsTypeData::TsFunctionType(TsFunctionType(node.0)),
            NodeKind::TsConstructorType => TsTypeData::TsConstructorType(TsConstructorType(node.0)),
            NodeKind::TsTypePredicate => TsTypeData::TsTypePredicate(TsTypePredicate(node.0)),
            NodeKind::TsTypeLiteral => TsTypeData::TsTypeLiteral(TsTypeLiteral(node.0)),
            NodeKind::TsMappedType => TsTypeData::TsMappedType(TsMappedType(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_type_name_data(&self, node: TsTypeName) -> TsTypeNameData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierReference => {
                TsTypeNameData::IdentifierReference(IdentifierReference(node.0))
            }
            NodeKind::TsQualifiedName => TsTypeNameData::TsQualifiedName(TsQualifiedName(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_qualified_name_left_data(
        &self,
        node: TsQualifiedNameLeft,
    ) -> TsQualifiedNameLeftData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierReference => {
                TsQualifiedNameLeftData::IdentifierReference(IdentifierReference(node.0))
            }
            NodeKind::IdentifierName => {
                TsQualifiedNameLeftData::IdentifierName(IdentifierName(node.0))
            }
            NodeKind::BindingIdentifier => {
                TsQualifiedNameLeftData::BindingIdentifier(BindingIdentifier(node.0))
            }
            NodeKind::ThisExpression => {
                TsQualifiedNameLeftData::ThisExpression(ThisExpression(node.0))
            }
            NodeKind::TsQualifiedName => {
                TsQualifiedNameLeftData::TsQualifiedName(TsQualifiedName(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_import_type_qualifier_data(
        &self,
        node: TsImportTypeQualifier,
    ) -> TsImportTypeQualifierData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierName => {
                TsImportTypeQualifierData::IdentifierName(IdentifierName(node.0))
            }
            NodeKind::TsQualifiedName => {
                TsImportTypeQualifierData::TsQualifiedName(TsQualifiedName(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
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
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::StringLiteral => TsLiteralData::StringLiteral(StringLiteral(node.0)),
            NodeKind::NumericLiteral => TsLiteralData::NumericLiteral(NumericLiteral(node.0)),
            NodeKind::BigIntLiteral => TsLiteralData::BigIntLiteral(BigIntLiteral(node.0)),
            NodeKind::BooleanLiteral => TsLiteralData::BooleanLiteral(BooleanLiteral(node.0)),
            NodeKind::TemplateLiteral => TsLiteralData::TemplateLiteral(TemplateLiteral(node.0)),
            NodeKind::UnaryExpression => TsLiteralData::UnaryExpression(UnaryExpression(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
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
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::TsPropertySignature => {
                TsTypeElementData::TsPropertySignature(TsPropertySignature(node.0))
            }
            NodeKind::TsMethodSignature => {
                TsTypeElementData::TsMethodSignature(TsMethodSignature(node.0))
            }
            NodeKind::TsCallSignatureDeclaration => {
                TsTypeElementData::TsCallSignatureDeclaration(TsCallSignatureDeclaration(node.0))
            }
            NodeKind::TsConstructSignatureDeclaration => {
                TsTypeElementData::TsConstructSignatureDeclaration(TsConstructSignatureDeclaration(
                    node.0,
                ))
            }
            NodeKind::TsIndexSignature => {
                TsTypeElementData::TsIndexSignature(TsIndexSignature(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_type_predicate_name_data(
        &self,
        node: TsTypePredicateName,
    ) -> TsTypePredicateNameData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierName => {
                TsTypePredicateNameData::IdentifierName(IdentifierName(node.0))
            }
            NodeKind::TsThisType => TsTypePredicateNameData::TsThisType(TsThisType(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_enum_member_name_data(&self, node: TsEnumMemberName) -> TsEnumMemberNameData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::IdentifierName => {
                TsEnumMemberNameData::IdentifierName(IdentifierName(node.0))
            }
            NodeKind::StringLiteral => TsEnumMemberNameData::StringLiteral(StringLiteral(node.0)),
            NodeKind::TemplateLiteral => {
                TsEnumMemberNameData::TemplateLiteral(TemplateLiteral(node.0))
            }
            NodeKind::IdentifierReference => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::ThisExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::Super => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::MetaProperty => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::NumericLiteral => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::BigIntLiteral => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::BooleanLiteral => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::NullLiteral => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::RegExpLiteral => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::BinaryExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::LogicalExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::UnaryExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::UpdateExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::AssignmentExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::ConditionalExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::SequenceExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::MemberExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::CallExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::ChainExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::NewExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::TaggedTemplateExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::ArrowFunctionExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::ArrayExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::ObjectExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::ParenthesizedExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::Function => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::Class => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::ImportExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::AwaitExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::YieldExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::PrivateIdentifier => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::TsAsExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::TsSatisfiesExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::TsTypeAssertion => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::TsNonNullExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::TsInstantiationExpression => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::JsxElement => TsEnumMemberNameData::Expr(Expr(node.0)),
            NodeKind::JsxFragment => TsEnumMemberNameData::Expr(Expr(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_module_name_data(&self, node: TsModuleName) -> TsModuleNameData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::BindingIdentifier => {
                TsModuleNameData::BindingIdentifier(BindingIdentifier(node.0))
            }
            NodeKind::StringLiteral => TsModuleNameData::StringLiteral(StringLiteral(node.0)),
            NodeKind::TsQualifiedName => TsModuleNameData::TsQualifiedName(TsQualifiedName(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_module_body_data(&self, node: TsModuleBody) -> TsModuleBodyData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::TsModuleBlock => TsModuleBodyData::TsModuleBlock(TsModuleBlock(node.0)),
            NodeKind::TsModuleDeclaration => {
                TsModuleBodyData::TsModuleDeclaration(TsModuleDeclaration(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
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
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::JsxIdentifier => JsxElementNameData::JsxIdentifier(JsxIdentifier(node.0)),
            NodeKind::JsxNamespacedName => {
                JsxElementNameData::JsxNamespacedName(JsxNamespacedName(node.0))
            }
            NodeKind::JsxMemberExpression => {
                JsxElementNameData::JsxMemberExpression(JsxMemberExpression(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_member_expression_object_data(
        &self,
        node: JsxMemberExpressionObject,
    ) -> JsxMemberExpressionObjectData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::JsxIdentifier => {
                JsxMemberExpressionObjectData::JsxIdentifier(JsxIdentifier(node.0))
            }
            NodeKind::JsxMemberExpression => {
                JsxMemberExpressionObjectData::JsxMemberExpression(JsxMemberExpression(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_attribute_name_data(&self, node: JsxAttributeName) -> JsxAttributeNameData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::JsxIdentifier => JsxAttributeNameData::JsxIdentifier(JsxIdentifier(node.0)),
            NodeKind::JsxNamespacedName => {
                JsxAttributeNameData::JsxNamespacedName(JsxNamespacedName(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_attribute_item_data(&self, node: JsxAttributeItem) -> JsxAttributeItemData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::JsxAttribute => JsxAttributeItemData::JsxAttribute(JsxAttribute(node.0)),
            NodeKind::JsxSpreadAttribute => {
                JsxAttributeItemData::JsxSpreadAttribute(JsxSpreadAttribute(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_attribute_value_data(&self, node: JsxAttributeValue) -> JsxAttributeValueData {
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::StringLiteral => JsxAttributeValueData::StringLiteral(StringLiteral(node.0)),
            NodeKind::JsxExpressionContainer => {
                JsxAttributeValueData::JsxExpressionContainer(JsxExpressionContainer(node.0))
            }
            NodeKind::JsxElement => JsxAttributeValueData::JsxElement(JsxElement(node.0)),
            NodeKind::JsxFragment => JsxAttributeValueData::JsxFragment(JsxFragment(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
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
        match unsafe { *self.nodes.kind_unchecked(node.0) } {
            NodeKind::JsxText => JsxChildData::JsxText(JsxText(node.0)),
            NodeKind::JsxExpressionContainer => {
                JsxChildData::JsxExpressionContainer(JsxExpressionContainer(node.0))
            }
            NodeKind::JsxSpreadChild => JsxChildData::JsxSpreadChild(JsxSpreadChild(node.0)),
            NodeKind::JsxElement => JsxChildData::JsxElement(JsxElement(node.0)),
            NodeKind::JsxFragment => JsxChildData::JsxFragment(JsxFragment(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
