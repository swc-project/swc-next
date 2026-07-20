use std::mem;

use swc_experimental_ast_macros::ast;

use super::{jsx::*, ts::*};
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
pub enum SourceType {
    Script,
    Module,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum MethodDefinitionKind {
    Constructor,
    Method,
    Get,
    Set,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Accessibility {
    None,
    Public,
    Private,
    Protected,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ClassType {
    ClassDeclaration,
    ClassExpression,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BinaryOperator {
    Equal,
    NotEqual,
    StrictEqual,
    StrictNotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponent,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    LeftShift,
    RightShift,
    UnsignedRightShift,
    In,
    Instanceof,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum LogicalOperator {
    And,
    Or,
    NullishCoalescing,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum UnaryOperator {
    Negate,
    Positive,
    LogicalNot,
    BitwiseNot,
    Typeof,
    Void,
    Delete,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum UpdateOperator {
    Increment,
    Decrement,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,
    ExponentAssign,
    LeftShiftAssign,
    RightShiftAssign,
    UnsignedRightShiftAssign,
    BitwiseOrAssign,
    BitwiseXorAssign,
    BitwiseAndAssign,
    LogicalOrAssign,
    LogicalAndAssign,
    NullishAssign,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum VariableKind {
    Var,
    Let,
    Const,
    Using,
    AwaitUsing,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum FunctionType {
    FunctionDeclaration,
    FunctionExpression,
    TsDeclareFunction,
    TsEmptyBodyFunctionExpression,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum FormalParameterKind {
    FormalParameters,
    UniqueFormalParameters,
    ArrowFormalParameters,
    Signature,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PropertyKind {
    Init,
    Get,
    Set,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum NumericLiteralKind {
    Decimal,
    Hex,
    Octal,
    Binary,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ImportOrExportKind {
    Value,
    Type,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ImportPhase {
    Source,
    Defer,
}

impl_extra_data_compact!(
    SourceType,
    MethodDefinitionKind,
    Accessibility,
    ClassType,
    BinaryOperator,
    LogicalOperator,
    UnaryOperator,
    UpdateOperator,
    AssignmentOperator,
    VariableKind,
    FunctionType,
    FormalParameterKind,
    PropertyKind,
    NumericLiteralKind,
    ImportOrExportKind,
    ImportPhase,
);

impl<'a> ExtraDataCompact<'a> for Option<ImportPhase> {
    fn to_extra_data(self, _ast: &Ast<'a>) -> ExtraData {
        ExtraData {
            other: self.map_or(u8::MAX as u64, |phase| phase as u64),
        }
    }

    unsafe fn from_extra_data(data: ExtraData, _ast: &Ast<'a>) -> Self {
        let value = unsafe { data.other as u8 };
        (value != u8::MAX).then(|| unsafe { mem::transmute(value) })
    }
}

#[ast]
pub struct Program {
    source_type: SourceType,
    body: Vec<StatementListItem>,
    hashbang: Option<Hashbang>,
}

#[ast(repr)]
pub enum Stmt {
    BlockStatement(BlockStatement),
    BreakStatement(BreakStatement),
    ContinueStatement(ContinueStatement),
    DebuggerStatement(DebuggerStatement),
    DoWhileStatement(DoWhileStatement),
    EmptyStatement(EmptyStatement),
    ExpressionStatement(ExpressionStatement),
    ForInStatement(ForInStatement),
    ForOfStatement(ForOfStatement),
    ForStatement(ForStatement),
    IfStatement(IfStatement),
    LabeledStatement(LabeledStatement),
    ReturnStatement(ReturnStatement),
    SwitchStatement(SwitchStatement),
    ThrowStatement(ThrowStatement),
    TryStatement(TryStatement),
    WhileStatement(WhileStatement),
    WithStatement(WithStatement),
    Declaration(Decl),
    ImportDeclaration(ImportDeclaration),
    ExportNamedDeclaration(ExportNamedDeclaration),
    ExportDefaultDeclaration(ExportDefaultDeclaration),
    ExportAllDeclaration(ExportAllDeclaration),
    TsExportAssignment(TsExportAssignment),
    TsNamespaceExportDeclaration(TsNamespaceExportDeclaration),
    TsImportEqualsDeclaration(TsImportEqualsDeclaration),
}

#[ast(repr)]
pub enum StatementListItem {
    Stmt(Stmt),
    Directive(Directive),
}

#[ast(repr)]
pub enum Decl {
    VariableDeclaration(VariableDeclaration),
    Function(Function),
    Class(Class),
    TsTypeAliasDeclaration(TsTypeAliasDeclaration),
    TsInterfaceDeclaration(TsInterfaceDeclaration),
    TsEnumDeclaration(TsEnumDeclaration),
    TsModuleDeclaration(TsModuleDeclaration),
    TsGlobalDeclaration(TsGlobalDeclaration),
}

#[ast(repr)]
pub enum Expr {
    IdentifierReference(IdentifierReference),
    ThisExpression(ThisExpression),
    Super(Super),
    MetaProperty(MetaProperty),
    StringLiteral(StringLiteral),
    NumericLiteral(NumericLiteral),
    BigIntLiteral(BigIntLiteral),
    BooleanLiteral(BooleanLiteral),
    NullLiteral(NullLiteral),
    RegExpLiteral(RegExpLiteral),
    TemplateLiteral(TemplateLiteral),
    BinaryExpression(BinaryExpression),
    LogicalExpression(LogicalExpression),
    UnaryExpression(UnaryExpression),
    UpdateExpression(UpdateExpression),
    AssignmentExpression(AssignmentExpression),
    ConditionalExpression(ConditionalExpression),
    SequenceExpression(SequenceExpression),
    MemberExpression(MemberExpression),
    CallExpression(CallExpression),
    ChainExpression(ChainExpression),
    NewExpression(NewExpression),
    TaggedTemplateExpression(TaggedTemplateExpression),
    ArrowFunctionExpression(ArrowFunctionExpression),
    ArrayExpression(ArrayExpression),
    ObjectExpression(ObjectExpression),
    ParenthesizedExpression(ParenthesizedExpression),
    Function(Function),
    Class(Class),
    ImportExpression(ImportExpression),
    AwaitExpression(AwaitExpression),
    YieldExpression(YieldExpression),
    PrivateIdentifier(PrivateIdentifier),
    TsAsExpression(TsAsExpression),
    TsSatisfiesExpression(TsSatisfiesExpression),
    TsTypeAssertion(TsTypeAssertion),
    TsNonNullExpression(TsNonNullExpression),
    TsInstantiationExpression(TsInstantiationExpression),
    JsxElement(JsxElement),
    JsxFragment(JsxFragment),
}

#[ast(repr)]
pub enum BindingPattern {
    BindingIdentifier(BindingIdentifier),
    SimpleAssignmentTarget(SimpleAssignmentTarget),
    AssignmentPattern(AssignmentPattern),
    BindingRestElement(BindingRestElement),
    ArrayPattern(ArrayPattern),
    ObjectPattern(ObjectPattern),
}

#[ast(repr)]
pub enum AssignmentTarget {
    SimpleAssignmentTarget(SimpleAssignmentTarget),
    ArrayAssignmentTarget(ArrayPattern),
    ObjectAssignmentTarget(ObjectPattern),
}

#[ast(repr)]
pub enum SimpleAssignmentTarget {
    IdentifierReference(IdentifierReference),
    MemberExpression(MemberExpression),
    CallExpression(CallExpression),
    TsAsExpression(TsAsExpression),
    TsSatisfiesExpression(TsSatisfiesExpression),
    TsTypeAssertion(TsTypeAssertion),
    TsNonNullExpression(TsNonNullExpression),
}

#[ast(repr)]
pub enum PropertyKey {
    IdentifierName(IdentifierName),
    PrivateIdentifier(PrivateIdentifier),
    StringLiteral(StringLiteral),
    NumericLiteral(NumericLiteral),
    BigIntLiteral(BigIntLiteral),
    Expr(Expr),
}

#[ast(repr)]
pub enum ModuleExportName {
    IdentifierName(IdentifierName),
    StringLiteral(StringLiteral),
}

#[ast(repr)]
pub enum Argument {
    Expr(Expr),
    SpreadElement(SpreadElement),
}

#[ast(repr)]
pub enum ObjectPropertyKind {
    ObjectProperty(ObjectProperty),
    SpreadElement(SpreadElement),
}

#[ast(repr)]
pub enum ForStatementInit {
    VariableDeclaration(VariableDeclaration),
    Expr(Expr),
}

#[ast(repr)]
pub enum ForStatementLeft {
    VariableDeclaration(VariableDeclaration),
    AssignmentTarget(AssignmentTarget),
}

#[ast(repr)]
pub enum ArrowFunctionBody {
    FunctionBody(FunctionBody),
    Expr(Expr),
}

#[ast(repr)]
pub enum FormalParameterItem {
    FormalParameter(FormalParameter),
    TsParameterProperty(TsParameterProperty),
}

#[ast(repr)]
pub enum FormalParameterPattern {
    BindingPattern(BindingPattern),
    TsThisParameter(TsThisParameter),
}

#[ast(repr)]
pub enum ClassElement {
    MethodDefinition(MethodDefinition),
    PropertyDefinition(PropertyDefinition),
    StaticBlock(StaticBlock),
    TsIndexSignature(TsIndexSignature),
}

#[ast(repr)]
pub enum ImportDeclarationSpecifier {
    ImportSpecifier(ImportSpecifier),
    ImportDefaultSpecifier(ImportDefaultSpecifier),
    ImportNamespaceSpecifier(ImportNamespaceSpecifier),
}

#[ast(repr)]
pub enum ExportDefaultDeclarationKind {
    Expr(Expr),
    Function(Function),
    Class(Class),
    TsInterfaceDeclaration(TsInterfaceDeclaration),
}

#[ast]
pub struct Super {}

#[ast]
pub struct ThisExpression {}

#[ast]
pub struct StringLiteral {
    value: Wtf8Atom,
    raw: Atom,
}

#[ast]
pub struct NumericLiteral {
    kind: NumericLiteralKind,
    raw: Atom,
}

#[ast]
pub struct BigIntLiteral {
    raw: Atom,
}

#[ast]
pub struct BooleanLiteral {
    value: bool,
}

#[ast]
pub struct NullLiteral {}

#[ast]
pub struct RegExpLiteral {
    pattern: Atom,
    flags: Atom,
}

#[ast]
pub struct TemplateLiteral {
    quasis: Vec<TemplateElement>,
    expressions: Vec<Expr>,
}

#[ast]
pub struct TemplateElement {
    cooked: Atom,
    raw: Atom,
    tail: bool,
    is_cooked_undefined: bool,
}

#[ast]
pub struct IdentifierReference {
    name: Atom,
    symbol_id: Cell<Option<SymbolId>>,
}

#[ast]
pub struct PrivateIdentifier {
    name: Atom,
}

#[ast]
pub struct BindingIdentifier {
    name: Atom,
    decorators: Vec<Decorator>,
    type_annotation: Option<TsTypeAnnotation>,
    optional: bool,
    symbol_id: Cell<Option<SymbolId>>,
}

#[ast]
pub struct IdentifierName {
    name: Atom,
}

#[ast]
pub struct LabelIdentifier {
    name: Atom,
}

#[ast]
pub struct AssignmentPattern {
    left: BindingPattern,
    right: Expr,
    decorators: Vec<Decorator>,
    type_annotation: Option<TsTypeAnnotation>,
    optional: bool,
}

#[ast]
pub struct BindingRestElement {
    argument: BindingPattern,
    decorators: Vec<Decorator>,
    type_annotation: Option<TsTypeAnnotation>,
    optional: bool,
}

#[ast]
pub struct ArrayPattern {
    elements: Vec<Option<BindingPattern>>,
    rest: Option<BindingRestElement>,
    decorators: Vec<Decorator>,
    type_annotation: Option<TsTypeAnnotation>,
    optional: bool,
}

#[ast]
pub struct ObjectPattern {
    properties: Vec<BindingProperty>,
    rest: Option<BindingRestElement>,
    decorators: Vec<Decorator>,
    type_annotation: Option<TsTypeAnnotation>,
    optional: bool,
}

#[ast]
pub struct BindingProperty {
    key: PropertyKey,
    value: BindingPattern,
    shorthand: bool,
    computed: bool,
}

#[ast]
pub struct BinaryExpression {
    left: Expr,
    right: Expr,
    operator: BinaryOperator,
}

#[ast]
pub struct LogicalExpression {
    left: Expr,
    right: Expr,
    operator: LogicalOperator,
}

#[ast]
pub struct ConditionalExpression {
    test: Expr,
    consequent: Expr,
    alternate: Expr,
}

#[ast]
pub struct UnaryExpression {
    argument: Expr,
    operator: UnaryOperator,
}

#[ast]
pub struct UpdateExpression {
    argument: SimpleAssignmentTarget,
    operator: UpdateOperator,
    prefix: bool,
}

#[ast]
pub struct AssignmentExpression {
    left: AssignmentTarget,
    right: Expr,
    operator: AssignmentOperator,
}

#[ast]
pub struct VariableDeclaration {
    kind: VariableKind,
    declarators: Vec<VariableDeclarator>,
    declare: bool,
}

#[ast]
pub struct VariableDeclarator {
    id: BindingPattern,
    init: Option<Expr>,
    definite: bool,
}

#[ast]
pub struct IfStatement {
    test: Expr,
    consequent: Stmt,
    alternate: Option<Stmt>,
}

#[ast]
pub struct SwitchStatement {
    discriminant: Expr,
    cases: Vec<SwitchCase>,
}

#[ast]
pub struct SwitchCase {
    test: Option<Expr>,
    consequent: Vec<Stmt>,
}

#[ast]
pub struct ForStatement {
    init: Option<ForStatementInit>,
    test: Option<Expr>,
    update: Option<Expr>,
    body: Stmt,
}

#[ast]
pub struct ForInStatement {
    left: ForStatementLeft,
    right: Expr,
    body: Stmt,
}

#[ast]
pub struct ForOfStatement {
    left: ForStatementLeft,
    right: Expr,
    body: Stmt,
    r#await: bool,
}

#[ast]
pub struct TryStatement {
    block: BlockStatement,
    handler: Option<CatchClause>,
    finalizer: Option<BlockStatement>,
}

#[ast]
pub struct CatchClause {
    param: Option<BindingPattern>,
    body: BlockStatement,
}

#[ast]
pub struct BreakStatement {
    label: Option<LabelIdentifier>,
}

#[ast]
pub struct ContinueStatement {
    label: Option<LabelIdentifier>,
}

#[ast]
pub struct LabeledStatement {
    label: LabelIdentifier,
    body: Stmt,
}

#[ast]
pub struct ExpressionStatement {
    expression: Expr,
}

#[ast]
pub struct ReturnStatement {
    argument: Option<Expr>,
}

#[ast]
pub struct ThrowStatement {
    argument: Expr,
}

#[ast]
pub struct DebuggerStatement {}

#[ast]
pub struct EmptyStatement {}

#[ast]
pub struct Directive {
    expression: StringLiteral,
    value: Wtf8Atom,
}

#[ast]
pub struct Decorator {
    expression: Expr,
}

#[ast]
pub struct Class {
    class_type: ClassType,
    decorators: Vec<Decorator>,
    id: Option<BindingIdentifier>,
    super_class: Option<Expr>,
    body: ClassBody,
    type_parameters: Option<TsTypeParameterDeclaration>,
    super_type_arguments: Option<TsTypeParameterInstantiation>,
    implements: Vec<TsClassImplements>,
    r#abstract: bool,
    declare: bool,
}

#[ast]
pub struct ClassBody {
    body: Vec<ClassElement>,
}

#[ast]
pub struct MethodDefinition {
    decorators: Vec<Decorator>,
    key: PropertyKey,
    value: Function,
    kind: MethodDefinitionKind,
    computed: bool,
    r#static: bool,
    r#override: bool,
    optional: bool,
    r#abstract: bool,
    accessibility: Accessibility,
}

#[ast]
pub struct PropertyDefinition {
    decorators: Vec<Decorator>,
    key: PropertyKey,
    value: Option<Expr>,
    computed: bool,
    r#static: bool,
    accessor: bool,
    type_annotation: Option<TsTypeAnnotation>,
    declare: bool,
    r#override: bool,
    optional: bool,
    definite: bool,
    readonly: bool,
    r#abstract: bool,
    accessibility: Accessibility,
}

#[ast]
pub struct StaticBlock {
    body: Vec<Stmt>,
}

#[ast]
pub struct Function {
    function_type: FunctionType,
    id: Option<BindingIdentifier>,
    generator: bool,
    r#async: bool,
    declare: bool,
    params: FormalParameters,
    body: Option<FunctionBody>,
    type_parameters: Option<TsTypeParameterDeclaration>,
    return_type: Option<TsTypeAnnotation>,
}

#[ast]
pub struct FunctionBody {
    body: Vec<StatementListItem>,
}

#[ast]
pub struct BlockStatement {
    body: Vec<Stmt>,
}

#[ast]
pub struct WhileStatement {
    test: Expr,
    body: Stmt,
}

#[ast]
pub struct DoWhileStatement {
    body: Stmt,
    test: Expr,
}

#[ast]
pub struct WithStatement {
    object: Expr,
    body: Stmt,
}

#[ast]
pub struct FormalParameters {
    items: Vec<FormalParameterItem>,
    rest: Option<BindingRestElement>,
    kind: FormalParameterKind,
}

#[ast]
pub struct FormalParameter {
    pattern: FormalParameterPattern,
}

#[ast]
pub struct ArrayExpression {
    elements: Vec<Option<Argument>>,
}

#[ast]
pub struct ObjectExpression {
    properties: Vec<ObjectPropertyKind>,
}

#[ast]
pub struct SpreadElement {
    argument: Expr,
}

#[ast]
pub struct ObjectProperty {
    key: PropertyKey,
    value: Expr,
    kind: PropertyKind,
    method: bool,
    shorthand: bool,
    computed: bool,
}

#[ast]
pub struct ParenthesizedExpression {
    expression: Expr,
}

#[ast]
pub struct ArrowFunctionExpression {
    expression: bool,
    r#async: bool,
    params: FormalParameters,
    body: ArrowFunctionBody,
    type_parameters: Option<TsTypeParameterDeclaration>,
    return_type: Option<TsTypeAnnotation>,
}

#[ast]
pub struct SequenceExpression {
    expressions: Vec<Expr>,
}

#[ast]
pub struct MemberExpression {
    object: Expr,
    property: PropertyKey,
    computed: bool,
    optional: bool,
}

#[ast]
pub struct CallExpression {
    callee: Expr,
    arguments: Vec<Argument>,
    optional: bool,
    type_arguments: Option<TsTypeParameterInstantiation>,
}

#[ast]
pub struct ChainExpression {
    expression: Expr,
}

#[ast]
pub struct TaggedTemplateExpression {
    tag: Expr,
    quasi: TemplateLiteral,
    type_arguments: Option<TsTypeParameterInstantiation>,
}

#[ast]
pub struct NewExpression {
    callee: Expr,
    arguments: Vec<Argument>,
    type_arguments: Option<TsTypeParameterInstantiation>,
}

#[ast]
pub struct AwaitExpression {
    argument: Expr,
}

#[ast]
pub struct YieldExpression {
    argument: Option<Expr>,
    delegate: bool,
}

#[ast]
pub struct MetaProperty {
    meta: IdentifierName,
    property: IdentifierName,
}

#[ast]
pub struct ImportExpression {
    source: Expr,
    options: Option<Expr>,
    phase: Option<ImportPhase>,
}

#[ast]
pub struct ImportDeclaration {
    specifiers: Vec<ImportDeclarationSpecifier>,
    source: StringLiteral,
    attributes: Vec<ImportAttribute>,
    phase: Option<ImportPhase>,
    import_kind: ImportOrExportKind,
}

#[ast]
pub struct ImportSpecifier {
    imported: ModuleExportName,
    local: BindingIdentifier,
    import_kind: ImportOrExportKind,
}

#[ast]
pub struct ImportDefaultSpecifier {
    local: BindingIdentifier,
}

#[ast]
pub struct ImportNamespaceSpecifier {
    local: BindingIdentifier,
}

#[ast]
pub struct ImportAttribute {
    key: ModuleExportName,
    value: StringLiteral,
}

#[ast]
pub struct ExportNamedDeclaration {
    declaration: Option<Decl>,
    specifiers: Vec<ExportSpecifier>,
    source: Option<StringLiteral>,
    attributes: Vec<ImportAttribute>,
    export_kind: ImportOrExportKind,
}

#[ast]
pub struct ExportDefaultDeclaration {
    declaration: ExportDefaultDeclarationKind,
}

#[ast]
pub struct ExportAllDeclaration {
    exported: Option<ModuleExportName>,
    source: StringLiteral,
    attributes: Vec<ImportAttribute>,
    export_kind: ImportOrExportKind,
}

#[ast]
pub struct ExportSpecifier {
    local: ModuleExportName,
    exported: ModuleExportName,
    export_kind: ImportOrExportKind,
}

#[ast]
pub struct Hashbang {
    value: Atom,
}
