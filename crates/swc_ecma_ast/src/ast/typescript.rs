use std::mem;

use swc_experimental_ast_macros::ast;

use crate::{
    ArrayPat, AssignPat, Ast, BigInt, BindingIdent, Bool, ExtraData, ExtraDataCompact, Ident,
    Number, ObjectPat, RestPat, Str,
};

#[ast]
pub struct TsTypeAnn {
    type_ann: TsType,
}

#[ast]
pub struct TsTypeParamDecl {
    params: Vec<TsTypeParam>,
}

#[ast]
pub struct TsTypeParam {
    name: Ident,
    is_in: bool,
    is_out: bool,
    is_const: bool,
    constraint: Option<TsType>,
    default: Option<TsType>,
}

#[ast]
pub struct TsTypeParamInstantiation {
    params: Vec<TsType>,
}

#[ast]
pub struct TsParamProp {
    decorators: Vec<Decorator>,
    accessibility: Option<Accessibility>,
    is_override: bool,
    readonly: bool,
    param: TsParamPropParam,
}

#[ast]
pub enum TsParamPropParam {
    Ident(BindingIdent),
    Assign(AssignPat),
}

#[ast]
pub struct TsQualifiedName {
    left: TsEntityName,
    right: IdentName,
}

#[ast]
pub enum TsEntityName {
    TsQualifiedName(TsQualifiedName),
    Ident(Ident),
}

// ================
// TypeScript type members (for type literal / interface / class)
// ================

#[ast]
pub enum TsTypeElement {
    TsCallSignatureDecl(TsCallSignatureDecl),
    TsConstructSignatureDecl(TsConstructSignatureDecl),
    TsPropertySignature(TsPropertySignature),
    TsGetterSignature(TsGetterSignature),
    TsSetterSignature(TsSetterSignature),
    TsMethodSignature(TsMethodSignature),
    TsIndexSignature(TsIndexSignature),
}

#[ast]
pub struct TsCallSignatureDecl {
    params: Vec<TsFnParam>,
    type_ann: Option<TsTypeAnn>,
    type_params: Option<TsTypeParamDecl>,
}

#[ast]
pub struct TsConstructSignatureDecl {
    params: Vec<TsFnParam>,
    type_ann: Option<TsTypeAnn>,
    type_params: Option<TsTypeParamDecl>,
}

#[ast]
pub struct TsPropertySignature {
    readonly: bool,
    key: Expr,
    computed: bool,
    optional: bool,
    type_ann: Option<TsTypeAnn>,
}

#[ast]
pub struct TsGetterSignature {
    key: Expr,
    computed: bool,
    type_ann: Option<TsTypeAnn>,
}

#[ast]
pub struct TsSetterSignature {
    key: Expr,
    computed: bool,
    param: TsFnParam,
}

#[ast]
pub struct TsMethodSignature {
    key: Expr,
    computed: bool,
    optional: bool,
    params: Vec<TsFnParam>,
    type_ann: Option<TsTypeAnn>,
    type_params: Option<TsTypeParamDecl>,
}

#[ast]
pub struct TsIndexSignature {
    params: Vec<TsFnParam>,
    type_ann: Option<TsTypeAnn>,
    readonly: bool,
    is_static: bool,
}

// ================
// TypeScript types
// ================

#[ast]
pub enum TsType {
    TsKeywordType(TsKeywordType),
    TsThisType(TsThisType),
    TsFnOrConstructorType(TsFnOrConstructorType),
    TsTypeRef(TsTypeRef),
    TsTypeQuery(TsTypeQuery),
    TsTypeLit(TsTypeLit),
    TsArrayType(TsArrayType),
    TsTupleType(TsTupleType),
    TsOptionalType(TsOptionalType),
    TsRestType(TsRestType),
    TsUnionOrIntersectionType(TsUnionOrIntersectionType),
    TsConditionalType(TsConditionalType),
    TsInferType(TsInferType),
    TsParenthesizedType(TsParenthesizedType),
    TsTypeOperator(TsTypeOperator),
    TsIndexedAccessType(TsIndexedAccessType),
    TsMappedType(TsMappedType),
    TsLitType(TsLitType),
    TsTypePredicate(TsTypePredicate),
    TsImportType(TsImportType),
}

#[ast]
pub enum TsFnOrConstructorType {
    TsFnType(TsFnType),
    TsConstructorType(TsConstructorType),
}

#[ast]
pub struct TsKeywordType {
    kind: TsKeywordTypeKind,
}

#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum TsKeywordTypeKind {
    TsAnyKeyword,
    TsUnknownKeyword,
    TsNumberKeyword,
    TsObjectKeyword,
    TsBooleanKeyword,
    TsBigIntKeyword,
    TsStringKeyword,
    TsSymbolKeyword,
    TsVoidKeyword,
    TsUndefinedKeyword,
    TsNullKeyword,
    TsNeverKeyword,
    TsIntrinsicKeyword,
}

impl ExtraDataCompact for TsKeywordTypeKind {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { other: self as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { mem::transmute(data.other as u8) }
    }
}

#[ast]
pub struct TsThisType {}

#[ast]
pub enum TsFnParam {
    Ident(BindingIdent),
    Array(ArrayPat),
    Rest(RestPat),
    Object(ObjectPat),
}

#[ast]
pub struct TsFnType {
    params: Vec<TsFnParam>,
    type_params: Option<TsTypeParamDecl>,
    type_ann: TsTypeAnn,
}

#[ast]
pub struct TsConstructorType {
    params: Vec<TsFnParam>,
    type_params: Option<TsTypeParamDecl>,
    type_ann: TsTypeAnn,
    is_abstract: bool,
}

#[ast]
pub struct TsTypeRef {
    type_name: TsEntityName,
    type_params: Option<TsTypeParamInstantiation>,
}

#[ast]
pub struct TsTypePredicate {
    asserts: bool,
    param_name: TsThisTypeOrIdent,
    type_ann: Option<TsTypeAnn>,
}

#[ast]
pub enum TsThisTypeOrIdent {
    TsThisType(TsThisType),
    Ident(Ident),
}

/// `typeof` operator
#[ast]
pub struct TsTypeQuery {
    expr_name: TsTypeQueryExpr,
    type_args: Option<TsTypeParamInstantiation>,
}

#[ast]
pub enum TsTypeQueryExpr {
    TsEntityName(TsEntityName),
    Import(TsImportType),
}

#[ast]
pub struct TsImportCallOptions {
    with: ObjectLit,
}

#[ast]
pub struct TsImportType {
    arg: Str,
    qualifier: Option<TsEntityName>,
    type_args: Option<TsTypeParamInstantiation>,
    attributes: Option<TsImportCallOptions>,
}

#[ast]
pub struct TsTypeLit {
    members: Vec<TsTypeElement>,
}

#[ast]
pub struct TsArrayType {
    elem_type: TsType,
}

#[ast]
pub struct TsTupleType {
    elem_types: Vec<TsTupleElement>,
}

#[ast]
pub struct TsTupleElement {
    /// `Ident` or `RestPat { arg: Ident }`
    label: Option<Pat>,
    ty: TsType,
}

#[ast]
pub struct TsOptionalType {
    type_ann: TsType,
}

#[ast]
pub struct TsRestType {
    type_ann: TsType,
}

#[ast]
pub enum TsUnionOrIntersectionType {
    TsUnionType(TsUnionType),
    TsIntersectionType(TsIntersectionType),
}

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
    type_param: TsTypeParam,
}

#[ast]
pub struct TsParenthesizedType {
    type_ann: TsType,
}

#[ast]
pub struct TsTypeOperator {
    op: TsTypeOperatorOp,
    type_ann: TsType,
}

#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum TsTypeOperatorOp {
    KeyOf,
    Unique,
    ReadOnly,
}

impl ExtraDataCompact for TsTypeOperatorOp {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { other: self as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { mem::transmute(data.other as u8) }
    }
}

#[ast]
pub struct TsIndexedAccessType {
    readonly: bool,
    obj_type: TsType,
    index_type: TsType,
}

#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum TruePlusMinus {
    True,
    Plus,
    Minus,
}

impl ExtraDataCompact for Option<TruePlusMinus> {
    fn to_extra_data(self) -> ExtraData {
        let data: u8 = unsafe { mem::transmute(self) };
        ExtraData { other: data as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { mem::transmute(data.other as u8) }
    }
}

#[ast]
pub struct TsMappedType {
    readonly: Option<TruePlusMinus>,
    type_param: TsTypeParam,
    name_type: Option<TsType>,
    optional: Option<TruePlusMinus>,
    type_ann: Option<TsType>,
}

#[ast]
pub struct TsLitType {
    lit: TsLit,
}

#[ast]
pub enum TsLit {
    Number(Number),
    Str(Str),
    Bool(Bool),
    BigInt(BigInt),
    Tpl(TsTplLitType),
}

#[ast]
pub struct TsTplLitType {
    types: Vec<TsType>,
    quasis: Vec<TplElement>,
}

// // ================
// // TypeScript declarations
// // ================

#[ast]
pub struct TsInterfaceDecl {
    id: Ident,
    declare: bool,
    type_params: Option<TsTypeParamDecl>,
    extends: Vec<TsExprWithTypeArgs>,
    body: TsInterfaceBody,
}

#[ast]
pub struct TsInterfaceBody {
    body: Vec<TsTypeElement>,
}

#[ast]
pub struct TsExprWithTypeArgs {
    expr: Expr,
    type_args: Option<TsTypeParamInstantiation>,
}

#[ast]
pub struct TsTypeAliasDecl {
    declare: bool,
    id: Ident,
    type_params: Option<TsTypeParamDecl>,
    pub type_ann: TsType,
}

#[ast]
pub struct TsEnumDecl {
    declare: bool,
    is_const: bool,
    id: Ident,
    members: Vec<TsEnumMember>,
}

#[ast]
pub struct TsEnumMember {
    id: TsEnumMemberId,
    init: Option<Expr>,
}

///
/// - Invalid: [Ident] with empty symbol.
#[ast]
pub enum TsEnumMemberId {
    Ident(Ident),
    Str(Str),
}

#[ast]
pub struct TsModuleDecl {
    declare: bool,
    /// In TypeScript, this is only available through`node.flags`.
    global: bool,
    namespace: bool,
    id: TsModuleName,
    body: Option<TsNamespaceBody>,
}

/// `namespace A.B { }` is a namespace named `A` with another TsNamespaceDecl as
/// its body.
#[ast]
pub enum TsNamespaceBody {
    TsModuleBlock(TsModuleBlock),
    TsNamespaceDecl(TsNamespaceDecl),
}

#[ast]
pub struct TsModuleBlock {
    body: Vec<ModuleItem>,
}

#[ast]
pub struct TsNamespaceDecl {
    declare: bool,
    /// In TypeScript, this is only available through`node.flags`.
    global: bool,
    id: Ident,
    body: TsNamespaceBody,
}

#[ast]
pub enum TsModuleName {
    Ident(Ident),
    Str(Str),
}

#[ast]
pub struct TsImportEqualsDecl {
    is_export: bool,
    is_type_only: bool,
    id: Ident,
    module_ref: TsModuleRef,
}

#[ast]
pub enum TsModuleRef {
    TsEntityName(TsEntityName),
    TsExternalModuleRef(TsExternalModuleRef),
}

#[ast]
pub struct TsExternalModuleRef {
    expr: Str,
}

/// TypeScript's own parser uses ExportAssignment for both `export default` and
/// `export =`. But for @babel/parser, `export default` is an ExportDefaultDecl,
/// so a TsExportAssignment is always `export =`.
#[ast]
pub struct TsExportAssignment {
    expr: Expr,
}

#[ast]
pub struct TsNamespaceExportDecl {
    id: Ident,
}

// // ================
// // TypeScript exprs
// // ================

#[ast]
pub struct TsAsExpr {
    expr: Expr,
    type_ann: TsType,
}

#[ast]
pub struct TsTypeAssertion {
    expr: Expr,
    type_ann: TsType,
}

#[ast]
pub struct TsNonNullExpr {
    expr: Expr,
}

#[ast]
pub struct TsSatisfiesExpr {
    expr: Expr,
    type_ann: TsType,
}

#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum Accessibility {
    Public,
    Protected,
    Private,
}

impl ExtraDataCompact for Accessibility {
    fn to_extra_data(self) -> ExtraData {
        ExtraData { other: self as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { mem::transmute(data.other as u8) }
    }
}

impl ExtraDataCompact for Option<Accessibility> {
    fn to_extra_data(self) -> ExtraData {
        let data: u8 = unsafe { mem::transmute(self) };
        ExtraData { other: data as u64 }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast) -> Self {
        unsafe { mem::transmute(data.other as u8) }
    }
}

#[ast]
pub struct TsConstAssertion {
    expr: Expr,
}

#[ast]
pub struct TsInstantiation {
    expr: Expr,
    type_args: TsTypeParamInstantiation,
}
