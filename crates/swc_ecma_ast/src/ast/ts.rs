use std::mem;

use swc_experimental_ast_macros::ast;

use super::js::*;
use crate::{Ast, ExtraData, node_id::ExtraDataCompact};

macro_rules! impl_extra_data_compact {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl<'a> ExtraDataCompact<'a> for $ty {
                fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
                    ExtraData { other: self as u64 }
                }

                unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
                    unsafe { mem::transmute(data.other as u8) }
                }
            }
        )+
    };
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TsTypeOperatorKind {
    Keyof,
    Unique,
    Readonly,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TsMethodSignatureKind {
    Method,
    Get,
    Set,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TsModuleDeclarationKind {
    Namespace,
    Module,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TsMappedTypeModifier {
    None,
    True,
    Plus,
    Minus,
}

impl_extra_data_compact!(
    TsTypeOperatorKind,
    TsMethodSignatureKind,
    TsModuleDeclarationKind,
    TsMappedTypeModifier,
);

#[ast(repr)]
pub enum TsType {
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
    TsTypeQuery(TsTypeQuery),
    TsImportType(TsImportType),
    TsLiteralType(TsLiteralType),
    TsTemplateLiteralType(TsTemplateLiteralType),
    TsArrayType(TsArrayType),
    TsIndexedAccessType(TsIndexedAccessType),
    TsTupleType(TsTupleType),
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
}

#[ast(repr)]
pub enum TsTypeName {
    IdentifierReference(IdentifierReference),
    TsQualifiedName(TsQualifiedName),
}

#[ast(repr)]
pub enum TsQualifiedNameLeft {
    IdentifierReference(IdentifierReference),
    IdentifierName(IdentifierName),
    BindingIdentifier(BindingIdentifier),
    ThisExpression(ThisExpression),
    TsQualifiedName(TsQualifiedName),
}

#[ast(repr)]
pub enum TsImportTypeQualifier {
    IdentifierName(IdentifierName),
    TsQualifiedName(TsQualifiedName),
}

#[ast(repr)]
pub enum TsTypeQueryExprName {
    TsTypeName(TsTypeName),
    ThisExpression(ThisExpression),
    TsImportType(TsImportType),
}

#[ast(repr)]
pub enum TsLiteral {
    StringLiteral(StringLiteral),
    NumericLiteral(NumericLiteral),
    BigIntLiteral(BigIntLiteral),
    BooleanLiteral(BooleanLiteral),
    TemplateLiteral(TemplateLiteral),
    UnaryExpression(UnaryExpression),
}

#[ast(repr)]
pub enum TsTupleElement {
    TsType(TsType),
    TsNamedTupleMember(TsNamedTupleMember),
    TsOptionalType(TsOptionalType),
    TsRestType(TsRestType),
}

#[ast(repr)]
pub enum TsTypeElement {
    TsPropertySignature(TsPropertySignature),
    TsMethodSignature(TsMethodSignature),
    TsCallSignatureDeclaration(TsCallSignatureDeclaration),
    TsConstructSignatureDeclaration(TsConstructSignatureDeclaration),
    TsIndexSignature(TsIndexSignature),
}

#[ast(repr)]
pub enum TsTypePredicateName {
    IdentifierName(IdentifierName),
    TsThisType(TsThisType),
}

#[ast(repr)]
pub enum TsEnumMemberName {
    IdentifierName(IdentifierName),
    StringLiteral(StringLiteral),
    TemplateLiteral(TemplateLiteral),
    Expr(Expr),
}

#[ast(repr)]
pub enum TsModuleName {
    BindingIdentifier(BindingIdentifier),
    StringLiteral(StringLiteral),
    TsQualifiedName(TsQualifiedName),
}

#[ast(repr)]
pub enum TsModuleBody {
    TsModuleBlock(TsModuleBlock),
    TsModuleDeclaration(TsModuleDeclaration),
}

#[ast(repr)]
pub enum TsModuleReference {
    TsTypeName(TsTypeName),
    TsExternalModuleReference(TsExternalModuleReference),
}

#[ast]
pub struct TsTypeAnnotation {
    type_annotation: TsType,
}

#[ast]
pub struct TsAnyKeyword {}

#[ast]
pub struct TsUnknownKeyword {}

#[ast]
pub struct TsNeverKeyword {}

#[ast]
pub struct TsVoidKeyword {}

#[ast]
pub struct TsNullKeyword {}

#[ast]
pub struct TsUndefinedKeyword {}

#[ast]
pub struct TsStringKeyword {}

#[ast]
pub struct TsNumberKeyword {}

#[ast]
pub struct TsBigIntKeyword {}

#[ast]
pub struct TsBooleanKeyword {}

#[ast]
pub struct TsSymbolKeyword {}

#[ast]
pub struct TsObjectKeyword {}

#[ast]
pub struct TsIntrinsicKeyword {}

#[ast]
pub struct TsThisType {}

#[ast]
pub struct TsTypeReference {
    type_name: TsTypeName,
    type_arguments: Option<TsTypeParameterInstantiation>,
}

#[ast]
pub struct TsQualifiedName {
    left: TsQualifiedNameLeft,
    right: IdentifierName,
}

#[ast]
pub struct TsTypeQuery {
    expr_name: TsTypeQueryExprName,
    type_arguments: Option<TsTypeParameterInstantiation>,
}

#[ast]
pub struct TsImportType {
    source: StringLiteral,
    options: Option<Expr>,
    qualifier: Option<TsImportTypeQualifier>,
    type_arguments: Option<TsTypeParameterInstantiation>,
}

#[ast]
pub struct TsTypeParameter {
    name: BindingIdentifier,
    constraint: Option<TsType>,
    default: Option<TsType>,
    r#in: bool,
    out: bool,
    r#const: bool,
}

#[ast]
pub struct TsTypeParameterDeclaration {
    params: Vec<TsTypeParameter>,
}

#[ast]
pub struct TsTypeParameterInstantiation {
    params: Vec<TsType>,
}

#[ast]
pub struct TsLiteralType {
    literal: TsLiteral,
}

#[ast]
pub struct TsTemplateLiteralType {
    quasis: Vec<TemplateElement>,
    types: Vec<TsType>,
}

#[ast]
pub struct TsArrayType {
    element_type: TsType,
}

#[ast]
pub struct TsIndexedAccessType {
    object_type: TsType,
    index_type: TsType,
}

#[ast]
pub struct TsTupleType {
    element_types: Vec<TsTupleElement>,
}

#[ast]
pub struct TsNamedTupleMember {
    label: IdentifierName,
    element_type: TsType,
    optional: bool,
}

#[ast]
pub struct TsOptionalType {
    type_annotation: TsType,
}

#[ast]
pub struct TsRestType {
    type_annotation: TsType,
}

#[ast]
pub struct TsJsDocNullableType {
    type_annotation: TsType,
    postfix: bool,
}

#[ast]
pub struct TsJsDocNonNullableType {
    type_annotation: TsType,
    postfix: bool,
}

#[ast]
pub struct TsJsDocUnknownType {}

#[ast]
pub struct TsUnionType {
    types: Vec<TsType>,
}

#[ast]
pub struct TsIntersectionType {
    types: Vec<TsType>,
}

#[ast]
pub struct TsConditionalType {
    check_type: TsType,
    extends_type: TsType,
    true_type: TsType,
    false_type: TsType,
}

#[ast]
pub struct TsInferType {
    type_parameter: TsTypeParameter,
}

#[ast]
pub struct TsTypeOperator {
    operator: TsTypeOperatorKind,
    type_annotation: TsType,
}

#[ast]
pub struct TsParenthesizedType {
    type_annotation: TsType,
}

#[ast]
pub struct TsFunctionType {
    type_parameters: Option<TsTypeParameterDeclaration>,
    params: FormalParameters,
    return_type: TsTypeAnnotation,
}

#[ast]
pub struct TsConstructorType {
    type_parameters: Option<TsTypeParameterDeclaration>,
    params: FormalParameters,
    return_type: TsTypeAnnotation,
    r#abstract: bool,
}

#[ast]
pub struct TsTypePredicate {
    parameter_name: TsTypePredicateName,
    type_annotation: Option<TsTypeAnnotation>,
    asserts: bool,
}

#[ast]
pub struct TsTypeLiteral {
    members: Vec<TsTypeElement>,
}

#[ast]
pub struct TsMappedType {
    key: BindingIdentifier,
    constraint: TsType,
    name_type: Option<TsType>,
    type_annotation: Option<TsType>,
    optional: TsMappedTypeModifier,
    readonly: TsMappedTypeModifier,
}

#[ast]
pub struct TsPropertySignature {
    key: PropertyKey,
    type_annotation: Option<TsTypeAnnotation>,
    computed: bool,
    optional: bool,
    readonly: bool,
}

#[ast]
pub struct TsMethodSignature {
    key: PropertyKey,
    type_parameters: Option<TsTypeParameterDeclaration>,
    params: FormalParameters,
    return_type: Option<TsTypeAnnotation>,
    kind: TsMethodSignatureKind,
    computed: bool,
    optional: bool,
}

#[ast]
pub struct TsCallSignatureDeclaration {
    type_parameters: Option<TsTypeParameterDeclaration>,
    params: FormalParameters,
    return_type: Option<TsTypeAnnotation>,
}

#[ast]
pub struct TsConstructSignatureDeclaration {
    type_parameters: Option<TsTypeParameterDeclaration>,
    params: FormalParameters,
    return_type: Option<TsTypeAnnotation>,
}

#[ast]
pub struct TsIndexSignature {
    parameters: Vec<BindingIdentifier>,
    type_annotation: TsTypeAnnotation,
    readonly: bool,
    r#static: bool,
}

#[ast]
pub struct TsTypeAliasDeclaration {
    id: BindingIdentifier,
    type_parameters: Option<TsTypeParameterDeclaration>,
    type_annotation: TsType,
    declare: bool,
}

#[ast]
pub struct TsInterfaceDeclaration {
    id: BindingIdentifier,
    type_parameters: Option<TsTypeParameterDeclaration>,
    extends: Vec<TsInterfaceHeritage>,
    body: TsInterfaceBody,
    declare: bool,
}

#[ast]
pub struct TsInterfaceBody {
    body: Vec<TsTypeElement>,
}

#[ast]
pub struct TsInterfaceHeritage {
    expression: Expr,
    type_arguments: Option<TsTypeParameterInstantiation>,
}

#[ast]
pub struct TsClassImplements {
    expression: Expr,
    type_arguments: Option<TsTypeParameterInstantiation>,
}

#[ast]
pub struct TsEnumDeclaration {
    id: BindingIdentifier,
    body: TsEnumBody,
    is_const: bool,
    declare: bool,
}

#[ast]
pub struct TsEnumBody {
    members: Vec<TsEnumMember>,
}

#[ast]
pub struct TsEnumMember {
    id: TsEnumMemberName,
    initializer: Option<Expr>,
    computed: bool,
}

#[ast]
pub struct TsModuleDeclaration {
    id: TsModuleName,
    body: Option<TsModuleBody>,
    kind: TsModuleDeclarationKind,
    declare: bool,
}

#[ast]
pub struct TsModuleBlock {
    body: Vec<StatementListItem>,
}

#[ast]
pub struct TsGlobalDeclaration {
    id: IdentifierName,
    body: TsModuleBlock,
    declare: bool,
}

#[ast]
pub struct TsParameterProperty {
    decorators: Vec<Decorator>,
    parameter: BindingPattern,
    r#override: bool,
    readonly: bool,
    accessibility: Accessibility,
}

#[ast]
pub struct TsThisParameter {
    type_annotation: Option<TsTypeAnnotation>,
}

#[ast]
pub struct TsAsExpression {
    expression: Expr,
    type_annotation: TsType,
}

#[ast]
pub struct TsSatisfiesExpression {
    expression: Expr,
    type_annotation: TsType,
}

#[ast]
pub struct TsTypeAssertion {
    type_annotation: TsType,
    expression: Expr,
}

#[ast]
pub struct TsNonNullExpression {
    expression: Expr,
}

#[ast]
pub struct TsInstantiationExpression {
    expression: Expr,
    type_arguments: TsTypeParameterInstantiation,
}

#[ast]
pub struct TsExportAssignment {
    expression: Expr,
}

#[ast]
pub struct TsNamespaceExportDeclaration {
    id: IdentifierName,
}

#[ast]
pub struct TsImportEqualsDeclaration {
    id: BindingIdentifier,
    module_reference: TsModuleReference,
    import_kind: ImportOrExportKind,
}

#[ast]
pub struct TsExternalModuleReference {
    expression: StringLiteral,
}
