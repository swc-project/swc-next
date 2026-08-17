use std::mem;

use swc_next_ast_macros::ast;

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

/// Kind of a class method definition.
///
/// ## Example
/// ```js
/// class C {
///   constructor() {}   // constructor
///   method() {}        // method
///   get prop() {}      // get
///   set prop(v) {}     // set
/// }
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum MethodDefinitionKind {
    Constructor,
    Method,
    Get,
    Set,
}

/// Accessibility modifier on a TypeScript class member.
///
/// `None` means no modifier was written. This is distinct from `Public`,
/// which is written explicitly.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Accessibility {
    None,
    Public,
    Private,
    Protected,
}

/// Class form.
///
/// `ClassDeclaration` for `class Foo {}`. `ClassExpression` for
/// `const x = class {}`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ClassType {
    ClassDeclaration,
    ClassExpression,
}

/// Binary operators.
///
/// ## Example
/// ```js
/// a + b
/// a === b
/// a instanceof B
/// ```
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

/// Logical operators.
///
/// ## Example
/// ```js
/// a && b
/// a || b
/// a ?? b
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum LogicalOperator {
    And,
    Or,
    NullishCoalescing,
}

/// Unary operators.
///
/// ## Example
/// ```js
/// -x
/// !flag
/// typeof x
/// void 0
/// delete obj.prop
/// ```
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

/// Update operators.
///
/// ## Example
/// ```js
/// ++x
/// x--
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum UpdateOperator {
    Increment,
    Decrement,
}

/// Assignment operators.
///
/// ## Example
/// ```js
/// x = 1
/// x += 2
/// x &&= y
/// x ??= 0
/// ```
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

/// Variable declaration kind.
///
/// ## Example
/// ```js
/// var x;
/// let y;
/// const z = 0;
/// using r = resource();
/// await using ar = asyncResource();
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum VariableKind {
    Var,
    Let,
    Const,
    Using,
    AwaitUsing,
}

impl VariableKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Var => "var",
            Self::Let => "let",
            Self::Const => "const",
            Self::Using => "using",
            Self::AwaitUsing => "await using",
        }
    }
}

/// Form of a function node.
///
/// `FunctionDeclaration` for `function foo() {}`.
/// `FunctionExpression` for `const x = function () {}`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum FunctionType {
    FunctionDeclaration,
    FunctionExpression,
}

/// Which grammar production the parameter list came from.
///
/// Constrains which binding forms are legal and whether duplicates are
/// allowed in strict mode.
///
/// `FormalParameters` is a plain `function` parameter list.
/// `UniqueFormalParameters` covers parameters of a generator, async,
/// arrow, method, or setter. `ArrowFormalParameters` covers an arrow
/// function parameter list. `Signature` covers parameters of a TypeScript
/// type signature.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum FormalParameterKind {
    FormalParameters,
    UniqueFormalParameters,
    ArrowFormalParameters,
    Signature,
}

/// Object property kind.
///
/// ## Example
/// ```js
/// { a: 1,          // init
///   get b() {},    // get
///   set b(v) {} }  // set
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PropertyKind {
    Init,
    Get,
    Set,
}

/// `value` versus `type` on a TypeScript import or export specifier.
///
/// ## Example
/// ```ts
/// import type { T } from "m";
/// //     ^^^^ kind = type
/// import { type T, v } from "m";
/// //       ^^^^ specifier kind = type
/// //               ^ specifier kind = value
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ImportOrExportKind {
    Value,
    Type,
}

/// Stage 3 import phase modifier.
///
/// ## Example
/// ```js
/// import source x from "m";  // phase = Source
/// import defer * as x from "m";  // phase = Defer
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ImportPhase {
    Source,
    Defer,
}

impl_extra_data_compact!(
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

/// The top-level node of every parsed file.
///
/// See: <https://tc39.es/ecma262/#prod-Script>,
/// <https://tc39.es/ecma262/#prod-Module>
///
/// CommonJS sources use Script statement grammar with the top-level
/// allowances of an ordinary function body.
///
/// ## Example
/// ```js
/// #!/usr/bin/env node
/// "use strict";
/// import x from "y";
/// console.log(x);
/// ```
///
/// `hashbang` is the `#!` line if present. `directives` contains the directive
/// prologue (`"use strict";`), while `body` contains imports and statements.
#[ast]
pub struct Program {
    directives: Vec<Directive>,
    /// Import and export declarations are classified as statements.
    body: Vec<Stmt>,
    /// `null` when the file has no `#!` line.
    hashbang: Option<Hashbang>,
}

#[ast(repr, nested)]
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
}

#[ast(repr)]
pub enum Decl {
    VariableDeclaration(VariableDeclaration),
    Function(Function),
    Class(Class),
    TsFunction(TsFunction),
    TsTypeAliasDeclaration(TsTypeAliasDeclaration),
    TsInterfaceDeclaration(TsInterfaceDeclaration),
    TsEnumDeclaration(TsEnumDeclaration),
    TsModuleDeclaration(TsModuleDeclaration),
    TsGlobalDeclaration(TsGlobalDeclaration),
    TsImportEqualsDeclaration(TsImportEqualsDeclaration),
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

#[ast(repr, nested)]
pub enum BindingPattern {
    BindingIdentifier(BindingIdentifier),
    SimpleAssignmentTarget(SimpleAssignmentTarget),
    AssignmentPattern(AssignmentPattern),
    BindingRestElement(BindingRestElement),
    ArrayPattern(ArrayPattern),
    ObjectPattern(ObjectPattern),
}

#[ast(repr, nested)]
pub enum AssignmentTarget {
    SimpleAssignmentTarget(SimpleAssignmentTarget),
    ArrayAssignmentTarget(ArrayPattern),
    ObjectAssignmentTarget(ObjectPattern),
}

#[ast(repr)]
pub enum SimpleAssignmentTarget {
    IdentifierReference(IdentifierReference),
    MemberExpression(MemberExpression),
    TsAsExpression(TsAsExpression),
    TsSatisfiesExpression(TsSatisfiesExpression),
    TsTypeAssertion(TsTypeAssertion),
    TsNonNullExpression(TsNonNullExpression),
}

#[ast(repr, nested)]
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

#[ast(repr, nested)]
pub enum Argument {
    Expr(Expr),
    SpreadElement(SpreadElement),
}

#[ast(repr)]
pub enum ObjectPropertyKind {
    ObjectProperty(ObjectProperty),
    SpreadElement(SpreadElement),
}

#[ast(repr, nested)]
pub enum ForStatementInit {
    VariableDeclaration(VariableDeclaration),
    Expr(Expr),
}

#[ast(repr, nested)]
pub enum ForStatementLeft {
    VariableDeclaration(VariableDeclaration),
    AssignmentTarget(AssignmentTarget),
}

#[ast(repr, nested)]
pub enum ArrowFunctionBody {
    FunctionBody(FunctionBody),
    Expr(Expr),
}

#[ast(repr)]
pub enum FormalParameterItem {
    FormalParameter(FormalParameter),
    TsParameterProperty(TsParameterProperty),
}

#[ast(repr, nested)]
pub enum FormalParameterPattern {
    BindingPattern(BindingPattern),
    TsThisParameter(TsThisParameter),
}

#[ast(repr)]
pub enum ClassElement {
    MethodDefinition(MethodDefinition),
    PropertyDefinition(PropertyDefinition),
    StaticBlock(StaticBlock),
    TsMethodDefinition(TsMethodDefinition),
    TsIndexSignature(TsIndexSignature),
}

#[ast(repr)]
pub enum ImportDeclarationSpecifier {
    ImportSpecifier(ImportSpecifier),
    ImportDefaultSpecifier(ImportDefaultSpecifier),
    ImportNamespaceSpecifier(ImportNamespaceSpecifier),
}

#[ast(repr, nested)]
pub enum ExportDefaultDeclarationKind {
    Expr(Expr),
    Function(Function),
    Class(Class),
    TsFunction(TsFunction),
    TsInterfaceDeclaration(TsInterfaceDeclaration),
}

/// The `super` keyword used as an expression head.
///
/// ## Example
/// ```js
/// foo(super);
/// //  ^^^^^ Super
/// ```
#[ast]
pub struct Super {}

/// The `this` keyword used as an expression.
///
/// ## Example
/// ```js
/// const x = this;
/// //        ^^^^ ThisExpression
/// ```
#[ast]
pub struct ThisExpression {}

/// A string literal.
///
/// ## Example
/// ```js
/// "hello\n"
/// //    ^^ escape sequences are decoded into `value`
/// ```
#[ast]
pub struct StringLiteral {
    /// decoded content with escape sequences resolved and quotes stripped
    value: AstWtf8,
    /// raw source lexeme including the surrounding quotes, exactly as written.
    raw: AstUtf8,
}

/// A numeric literal in one of four bases.
///
/// ## Example
/// ```js
/// 42          // decimal
/// 0xFF        // hex
/// 0o17        // octal
/// 0b1010      // binary
/// 1_000_000   // separators are stripped when computing `value`
/// ```
#[ast]
pub struct NumericLiteral {
    /// The authoritative ECMAScript Number value.
    #[ast(invalidates(raw))]
    value: f64,
    /// The source lexeme, including prefix (`0x`, `0b`, `0o`) and separators.
    ///
    /// `None` when no source spelling should be preserved.
    raw: Option<AstUtf8>,
}

/// A BigInt literal (a numeric literal with a trailing `n`).
///
/// See: MDN BigInt
/// developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt
///
/// ## Example
/// ```js
/// 42n
/// // raw = "42"
/// 0xFFn
/// // raw = "0xff"
/// ```
#[ast]
pub struct BigIntLiteral {
    /// digits without the trailing `n`
    raw: AstUtf8,
}

/// A `true` or `false` literal.
#[ast]
pub struct BooleanLiteral {
    value: bool,
}

/// The `null` literal.
///
/// ## Example
/// ```js
/// const x = null;
/// //        ^^^^ NullLiteral
/// ```
#[ast]
pub struct NullLiteral {}

/// A regular expression literal.
///
/// See: <https://tc39.es/ecma262/#prod-RegularExpressionLiteral>
///
/// ## Example
/// ```js
/// /foo/gi
/// // ^^^ pattern
/// //     ^^ flags
/// ```
#[ast]
pub struct RegExpLiteral {
    pattern: AstUtf8,
    flags: AstUtf8,
}

/// A template literal with zero or more interpolations.
///
/// See: <https://tc39.es/ecma262/#prod-TemplateLiteral>
///
/// ## Example
/// ```js
/// `hello ${name}!`
/// ```
///
/// `quasis` are the static text spans (`"hello "` and `"!"`), always
/// `expressions.len() + 1` elements. `expressions` are the interpolations
/// (here, `name`).
#[ast]
pub struct TemplateLiteral {
    /// `template_element[]`. Always `expressions.len() + 1` elements.
    quasis: Vec<TemplateElement>,
    /// any expression
    expressions: Vec<Expr>,
}

/// A single quasi span inside a template literal.
///
/// ## Example
/// ```js
/// `a ${x} b`
/// ```
///
/// The two quasi elements are `"a "` (`tail = false`) and `" b"`
/// (`tail = true`).
#[ast]
pub struct TemplateElement {
    /// escape-decoded content, empty when `is_cooked_undefined`
    cooked: AstWtf8,
    /// verbatim source text, line endings normalized to `\n`. empty for
    /// synthetic nodes
    raw: AstUtf8,
    /// true for the final element (after the last interpolation)
    tail: bool,
    /// true when the cooked value is undefined per ECMAScript TV semantics
    /// (an invalid escape inside a tagged template)
    is_cooked_undefined: bool,
}

/// An identifier used as an expression.
///
/// See: [ECMAScript Identifiers](https://tc39.es/ecma262/#sec-identifiers)
///
/// ## Example
/// ```js
/// console.log(x);
/// ```
///
/// `console` and `x` are both `identifier_reference`s. `log` is an
/// `identifier_name` because it appears in property-access position, not
/// as a value reference.
#[ast]
pub struct IdentifierReference {
    name: AstUtf8,
}

/// A `#privateName` identifier used inside a class.
///
/// The stored `name` does not include the leading `#`.
///
/// See: MDN Private class fields
/// developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Private_properties
///
/// ## Example
/// ```js
/// class C { #secret = 1; }
/// //         ^^^^^^ name = "secret"
/// ```
#[ast]
pub struct PrivateIdentifier {
    name: AstUtf8,
}

/// An identifier that introduces a new binding.
///
/// ## Example
/// ```ts
/// function foo(x?: number) {}
/// //       ^^^ id
/// //           ^ optional, with type_annotation
/// ```
#[ast]
pub struct BindingIdentifier {
    name: AstUtf8,
    /// `decorator[]`. Decorators on a parameter binding (legacy
    /// `experimentalDecorators`). Empty for non-parameter bindings.
    decorators: Vec<Decorator>,
    /// `ts_type_annotation`. `None` when absent.
    type_annotation: Option<TsTypeAnnotation>,
    /// true when marked optional (`x?: T`).
    optional: bool,
}

/// An identifier used as a property key or meta property name.
///
/// ## Example
/// ```js
/// obj.foo
/// //  ^^^ IdentifierName
/// { foo: 1 }
/// //^^^ IdentifierName
/// ```
#[ast]
pub struct IdentifierName {
    name: AstUtf8,
}

/// An identifier used as a statement label or as the target of `break` or
/// `continue`.
///
/// ## Example
/// ```js
/// outer: for (;;) break outer;
/// // ^^^ label
/// //                    ^^^^^ break target
/// ```
#[ast]
pub struct LabelIdentifier {
    name: AstUtf8,
}

/// A binding pattern with a default value.
///
/// ## Example
/// ```js
/// function f(x = 0) {}
/// //         ^^^^^ AssignmentPattern
/// //         ^ left
/// //             ^ right
/// ```
#[ast]
pub struct AssignmentPattern {
    /// `decorator[]`. Decorators on a parameter binding (legacy
    /// `experimentalDecorators`).
    decorators: Vec<Decorator>,
    /// any binding pattern
    left: BindingPattern,
    /// `ts_type_annotation`. `None` when absent.
    type_annotation: Option<TsTypeAnnotation>,
    /// any expression
    right: Expr,
    /// true when marked optional in a parameter position.
    optional: bool,
}

/// A `...rest` element in a binding pattern or function parameter list.
///
/// ## Example
/// ```js
/// function f(...args) {}
/// //         ^^^^^^^ BindingRestElement
/// //            ^^^^ argument
/// ```
#[ast]
pub struct BindingRestElement {
    /// `decorator[]`. Decorators on a parameter rest element (legacy
    /// `experimentalDecorators`).
    decorators: Vec<Decorator>,
    /// any binding pattern
    argument: BindingPattern,
    /// `ts_type_annotation`. `None` when absent.
    type_annotation: Option<TsTypeAnnotation>,
    /// true when marked optional in a parameter position.
    optional: bool,
}

/// An array destructuring pattern.
///
/// ## Example
/// ```js
/// const [a, , b, ...rest] = arr;
/// //     ^ elements[0]
/// //          ^ elements[2]
/// //             ^^^^^^^ rest
/// ```
///
/// `elements[1]` is `None` (the hole between the two commas).
#[ast]
pub struct ArrayPattern {
    /// `decorator[]`. Decorators on a parameter binding (legacy
    /// `experimentalDecorators`).
    decorators: Vec<Decorator>,
    /// any binding pattern. Entries are `None` for holes.
    elements: Vec<Option<BindingPattern>>,
    /// `binding_rest_element`. `None` when no rest element is present.
    rest: Option<BindingRestElement>,
    /// `ts_type_annotation`. `None` when absent.
    type_annotation: Option<TsTypeAnnotation>,
    /// true when marked optional in a parameter position.
    optional: bool,
}

/// An object destructuring pattern.
///
/// ## Example
/// ```js
/// const { a, b: c, ...rest } = obj;
/// //      ^ properties[0]
/// //         ^^^^ properties[1]
/// //               ^^^^^^^ rest
/// ```
#[ast]
pub struct ObjectPattern {
    /// `decorator[]`. Decorators on a parameter binding (legacy
    /// `experimentalDecorators`).
    decorators: Vec<Decorator>,
    /// `binding_property[]`
    properties: Vec<BindingProperty>,
    /// `binding_rest_element`. `None` when no rest element is present.
    rest: Option<BindingRestElement>,
    /// `ts_type_annotation`. `None` when absent.
    type_annotation: Option<TsTypeAnnotation>,
    /// true when marked optional in a parameter position.
    optional: bool,
}

/// A single property inside an `object_pattern`.
///
/// ## Example
/// ```js
/// const { a, b: c, [k]: d } = obj;
/// //      ^ shorthand
/// //         ^^^^ key colon value
/// //               ^^^^^^ computed key
/// ```
#[ast]
pub struct BindingProperty {
    /// `identifier_name`, `string_literal`, `numeric_literal`,
    /// `private_identifier` (class members only), or any expression when `computed = true`
    key: PropertyKey,
    /// any binding pattern
    value: BindingPattern,
    /// true when written as the shorthand `{ a }` form.
    shorthand: bool,
    /// true when the key is written inside `[...]`.
    computed: bool,
}

/// Binary expression with a non-logical operator.
///
/// ## Example
/// ```js
/// a + b
/// // ^ left
/// //   ^ operator
/// //     ^ right
/// ```
#[ast]
pub struct BinaryExpression {
    /// any expression
    left: Expr,
    /// any expression
    right: Expr,
    operator: BinaryOperator,
}

/// Short-circuiting logical expression.
///
/// ## Example
/// ```js
/// a && b
/// // ^ left
/// //   ^^ operator
/// //      ^ right
/// ```
#[ast]
pub struct LogicalExpression {
    /// any expression
    left: Expr,
    /// any expression
    right: Expr,
    operator: LogicalOperator,
}

/// Ternary expression.
///
/// ## Example
/// ```js
/// a ? b : c
/// // ^ test
/// //   ^ consequent
/// //       ^ alternate
/// ```
#[ast]
pub struct ConditionalExpression {
    test: Expr,
    /// any expression
    consequent: Expr,
    /// any expression
    alternate: Expr,
}

/// Unary prefix expression.
///
/// ## Example
/// ```js
/// typeof foo
/// // ^^^^^^ operator
/// //     ^^^ argument
/// ```
#[ast]
pub struct UnaryExpression {
    /// any expression
    argument: Expr,
    operator: UnaryOperator,
}

/// `++` or `--` applied to an assignable target.
///
/// ## Example
/// ```js
/// ++x
/// x--
/// ```
///
/// `prefix` is true for `++x` and false for `x++`.
#[ast]
pub struct UpdateExpression {
    /// any assignment target. In practice an `identifier_reference` or
    /// `member_expression`.
    argument: SimpleAssignmentTarget,
    operator: UpdateOperator,
    prefix: bool,
}

/// Assignment expression.
///
/// ## Example
/// ```js
/// x += 1
/// // ^ left
/// //  ^^ operator
/// //     ^ right
/// ```
#[ast]
pub struct AssignmentExpression {
    /// any assignment target
    left: AssignmentTarget,
    /// any expression
    right: Expr,
    operator: AssignmentOperator,
}

/// A `var`, `let`, `const`, `using`, or `await using` declaration.
///
/// See: <https://tc39.es/ecma262/#prod-VariableStatement>,
/// [Explicit Resource Management](https://github.com/tc39/proposal-explicit-resource-management)
///
/// ## Example
/// ```ts
/// const x = 1, y = 2;
/// //    ^^^^^^^^^^^^ declarators
/// ```
#[ast]
pub struct VariableDeclaration {
    kind: VariableKind,
    /// `variable_declarator[]`
    declarators: Vec<VariableDeclarator>,
    /// true for `declare var x: T`.
    declare: bool,
}

/// A single binding in a variable declaration.
///
/// ## Example
/// ```ts
/// let x!: number = 1;
/// // ^ id
/// //  ^ definite
/// //   ^^^^^^^^ type_annotation lives on `id`
/// //              ^ init
/// ```
#[ast]
pub struct VariableDeclarator {
    /// any binding pattern
    id: BindingPattern,
    /// any expression. `None` when there is no initializer.
    init: Option<Expr>,
    /// true for a definite assignment assertion (`let x!: T`).
    definite: bool,
}

/// An `if` / `else` statement.
///
/// ## Example
/// ```js
/// if (cond) thenStmt; else elseStmt;
/// //  ^^^^ test
/// //        ^^^^^^^^ consequent
/// //                       ^^^^^^^^ alternate
/// ```
#[ast]
pub struct IfStatement {
    test: Expr,
    /// any statement
    consequent: Stmt,
    /// any statement. `None` when there is no `else` clause.
    alternate: Option<Stmt>,
}

/// A `switch` statement.
///
/// ## Example
/// ```js
/// switch (x) { case 1: stmt; }
/// //      ^ discriminant
/// //           ^^^^^^^^^^^^^ cases[0]
/// ```
#[ast]
pub struct SwitchStatement {
    /// any expression
    discriminant: Expr,
    /// `switch_case[]`
    cases: Vec<SwitchCase>,
}

/// A single `case` or `default` clause inside a `switch`.
///
/// ## Example
/// ```js
/// case 1: doIt(); break;
/// //   ^ test
/// //      ^^^^^^^^^^^^^^ consequent
/// ```
#[ast]
pub struct SwitchCase {
    test: Option<Expr>,
    /// any statement
    consequent: Vec<Stmt>,
}

/// A `for (init; test; update)` loop.
///
/// ## Example
/// ```js
/// for (let i = 0; i < n; i++) body;
/// //   ^^^^^^^^^ init
/// //              ^^^^^ test
/// //                     ^^^ update
/// //                          ^^^^^ body
/// ```
#[ast]
pub struct ForStatement {
    /// `variable_declaration` or any expression. `None` when omitted.
    init: Option<ForStatementInit>,
    test: Option<Expr>,
    /// any expression. `None` when omitted.
    update: Option<Expr>,
    /// any statement
    body: Stmt,
}

/// A `for (... in ...)` loop.
///
/// ## Example
/// ```js
/// for (const k in obj) body;
/// //   ^^^^^^^ left
/// //              ^^^ right
/// //                   ^^^^^ body
/// ```
#[ast]
pub struct ForInStatement {
    /// `variable_declaration` or any assignment target
    left: ForStatementLeft,
    /// any expression
    right: Expr,
    /// any statement
    body: Stmt,
}

/// A `for (... of ...)` or `for await (... of ...)` loop.
///
/// ## Example
/// ```js
/// for await (const x of iter) body;
/// //  ^^^^^ await
/// //         ^^^^^^^ left
/// //                    ^^^^ right
/// //                          ^^^^^ body
/// ```
#[ast]
pub struct ForOfStatement {
    /// `variable_declaration` or any assignment target
    left: ForStatementLeft,
    /// any expression
    right: Expr,
    /// any statement
    body: Stmt,
    r#await: bool,
}

/// A `try` / `catch` / `finally` statement.
///
/// ## Example
/// ```js
/// try { a } catch (e) { b } finally { c }
/// //  ^^^^^ block
/// //        ^^^^^^^^^^^^^^^ handler
/// //                                ^^^^^ finalizer
/// ```
#[ast]
pub struct TryStatement {
    /// `block_statement`
    block: BlockStatement,
    /// `catch_clause`. `None` when no `catch` clause is present.
    handler: Option<CatchClause>,
    /// `block_statement`. `None` when no `finally` clause is present.
    finalizer: Option<BlockStatement>,
}

/// The `catch` clause of a `try` statement.
///
/// ## Example
/// ```js
/// try {} catch (e) { body }
/// //            ^ param
/// //               ^^^^^^^^ body
/// ```
#[ast]
pub struct CatchClause {
    /// any binding pattern. `None` for `catch { }` with no binding.
    param: Option<BindingPattern>,
    /// `block_statement`
    body: BlockStatement,
}

/// A `break` statement, optionally targeting a label.
///
/// ## Example
/// ```js
/// break;
/// break outer;
/// //    ^^^^^ label
/// ```
#[ast]
pub struct BreakStatement {
    /// `label_identifier`. `None` for bare `break`.
    label: Option<LabelIdentifier>,
}

/// A `continue` statement, optionally targeting a label.
///
/// ## Example
/// ```js
/// continue;
/// continue outer;
/// //       ^^^^^ label
/// ```
#[ast]
pub struct ContinueStatement {
    /// `label_identifier`. `None` for bare `continue`.
    label: Option<LabelIdentifier>,
}

/// A labeled statement.
///
/// ## Example
/// ```js
/// outer: for (;;) break outer;
/// // ^^^ label
/// //     ^^^^^^^^^^^^^^^^^^^^^ body
/// ```
#[ast]
pub struct LabeledStatement {
    /// `label_identifier`
    label: LabelIdentifier,
    /// any statement
    body: Stmt,
}

/// A statement consisting of a single expression.
///
/// ## Example
/// ```js
/// foo();
/// // ^^^^ expression
/// ```
#[ast]
pub struct ExpressionStatement {
    /// any expression
    expression: Expr,
}

/// A `return` statement.
///
/// ## Example
/// ```js
/// return;
/// return x + 1;
/// //     ^^^^^ argument
/// ```
#[ast]
pub struct ReturnStatement {
    /// any expression. `None` for bare `return`.
    argument: Option<Expr>,
}

/// A `throw` statement.
///
/// ## Example
/// ```js
/// throw new Error("boom");
/// //    ^^^^^^^^^^^^^^^^^ argument
/// ```
#[ast]
pub struct ThrowStatement {
    /// any expression
    argument: Expr,
}

/// The `debugger;` statement. Suspends execution when a debugger is attached.
///
/// ## Example
/// ```js
/// debugger;
/// ```
#[ast]
pub struct DebuggerStatement {}

/// A standalone `;` used as a statement.
///
/// ## Example
/// ```js
/// ;
/// ```
#[ast]
pub struct EmptyStatement {}

/// A directive prologue such as `"use strict";`.
///
/// ## Example
/// ```js
/// "use strict";
/// ```
///
/// `expression` is the underlying `string_literal` (`"use strict"`).
/// `value` is the text between the quotes (`use strict`).
#[ast]
pub struct Directive {
    /// `string_literal`
    expression: StringLiteral,
    /// the directive text without surrounding quotes
    value: AstUtf8,
}

/// A decorator applied to a class or class member.
///
/// See: [TC39 Decorators Proposal](https://github.com/tc39/proposal-decorators)
///
/// ## Example
/// ```js
/// class C { @foo bar() {} }
/// //         ^^^ expression
/// ```
#[ast]
pub struct Decorator {
    /// any expression
    expression: Expr,
}

/// A class declaration or expression.
///
/// See: <https://tc39.es/ecma262/#prod-ClassDeclaration>
///
/// ## Example
/// ```ts
/// class Foo<T> extends Base<T> implements I {}
/// //    ^^^ id
/// //       ^^^ type_parameters
/// //                   ^^^^ super_class
/// //                       ^^^ super_type_arguments
/// //                                      ^ implements[0]
/// //                                        ^^ body
/// ```
///
/// `@dec class` populates `decorators`. `abstract class` sets `abstract`.
/// `declare class` sets `declare`.
#[ast]
pub struct Class {
    class_type: ClassType,
    /// `decorator[]`
    decorators: Vec<Decorator>,
    /// `binding_identifier`. `None` for anonymous class expressions.
    id: Option<BindingIdentifier>,
    /// `ts_type_parameter_declaration`. `None` when the class has no `<T, U>`
    /// parameters.
    type_parameters: Option<TsTypeParameterDeclaration>,
    /// any expression. `None` when the class has no `extends` clause.
    super_class: Option<Expr>,
    /// `ts_type_parameter_instantiation`. `None` when `extends` has no `<T>`
    /// arguments.
    super_type_arguments: Option<TsTypeParameterInstantiation>,
    /// `ts_class_implements[]`. Empty when the class has no `implements`
    /// clause.
    implements: Vec<TsClassImplements>,
    /// `class_body`
    body: ClassBody,
    r#abstract: bool,
    /// true for `declare class`.
    declare: bool,
}

/// The `{ ... }` body of a class, holding its members.
///
/// ## Example
/// ```js
/// class C {
///   a;
///   b() {}
///   static {}
/// }
/// ```
#[ast]
pub struct ClassBody {
    /// `method_definition`, `property_definition`, `static_block`, or `ts_index_signature`
    body: Vec<ClassElement>,
}

/// A method, getter, setter, or constructor in a class body.
///
/// ## Example
/// ```ts
/// class C { public override foo?(): void {} }
/// //        ^^^^^^ accessibility
/// //               ^^^^^^^^ override
/// //                        ^^^ key
/// //                           ^ optional
/// //                            ^^^^^^^^^^^ value
/// ```
#[ast]
pub struct MethodDefinition {
    /// `decorator[]`
    decorators: Vec<Decorator>,
    /// `identifier_name`, `string_literal`, `numeric_literal`,
    /// `private_identifier` (class members only), or any expression when `computed = true`
    key: PropertyKey,
    /// `function`
    value: Function,
    kind: MethodDefinitionKind,
    /// true when the key is written inside `[...]`.
    computed: bool,
    r#static: bool,
    r#override: bool,
    /// true for an optional method (`foo?()`).
    optional: bool,
    r#abstract: bool,
    /// `None` when no accessibility modifier was written.
    accessibility: Accessibility,
}

/// A class field or auto-accessor declaration.
///
/// ## Example
/// ```ts
/// class C { public readonly foo!: number = 0 }
/// //        ^^^^^^ accessibility
/// //               ^^^^^^^^ readonly
/// //                        ^^^ key
/// //                           ^ definite
/// //                            ^^^^^^^^ type_annotation
/// //                                       ^ value
/// ```
#[ast]
pub struct PropertyDefinition {
    /// `decorator[]`
    decorators: Vec<Decorator>,
    /// `identifier_name`, `string_literal`, `numeric_literal`,
    /// `private_identifier` (class members only), or any expression when `computed = true`
    key: PropertyKey,
    /// `ts_type_annotation`. `None` when the field has no annotation.
    type_annotation: Option<TsTypeAnnotation>,
    /// any expression. `None` when the field has no initializer.
    value: Option<Expr>,
    /// true when the key is written inside `[...]`.
    computed: bool,
    r#static: bool,
    /// true for `accessor x;` auto-accessor fields.
    accessor: bool,
    /// true for the `declare` modifier.
    declare: bool,
    r#override: bool,
    /// true for an optional property (`foo?: T`).
    optional: bool,
    /// true for a definite assignment assertion (`foo!: T`).
    definite: bool,
    /// true for the `readonly` modifier.
    readonly: bool,
    r#abstract: bool,
    /// `None` when no accessibility modifier was written.
    accessibility: Accessibility,
}

/// A `static { ... }` block inside a class body.
///
/// See: MDN Static initialization blocks
/// developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Static_initialization_blocks
///
/// ## Example
/// ```js
/// class C { static { init(); } }
/// //                 ^^^^^^^ body
/// ```
#[ast]
pub struct StaticBlock {
    /// any statement
    body: Vec<Stmt>,
}

/// A function declaration or expression.
///
/// See: <https://tc39.es/ecma262/#prod-FunctionDeclaration>
///
/// ## Example
/// ```ts
/// async function* foo<T>(x: T): T { yield x; }
/// //            ^ generator
/// //              ^^^ id
/// //                 ^^^ type_parameters
/// //                    ^^^^^^ params
/// //                          ^^^ return_type
/// //                              ^^^^^^^^^^^^ body
/// ```
#[ast]
pub struct Function {
    function_type: FunctionType,
    /// `binding_identifier`. `None` for anonymous functions.
    id: Option<BindingIdentifier>,
    /// `ts_type_parameter_declaration`. `None` when absent.
    type_parameters: Option<TsTypeParameterDeclaration>,
    /// `formal_parameters`
    params: FormalParameters,
    /// `ts_type_annotation`. `None` when absent.
    return_type: Option<TsTypeAnnotation>,
    /// `function_body`
    body: FunctionBody,
    generator: bool,
    r#async: bool,
}

/// The body of a function.
///
/// See: <https://tc39.es/ecma262/#prod-FunctionBody>
#[ast]
pub struct FunctionBody {
    directives: Vec<Directive>,
    body: Vec<Stmt>,
}

/// A braced block statement.
///
/// ## Example
/// ```js
/// { stmt1; stmt2; }
/// //^^^^^^^^^^^^^ body
/// ```
#[ast]
pub struct BlockStatement {
    /// any statement
    body: Vec<Stmt>,
}

/// A `while` statement.
///
/// ## Example
/// ```js
/// while (cond) body;
/// //     ^^^^ test
/// //           ^^^^^ body
/// ```
#[ast]
pub struct WhileStatement {
    test: Expr,
    /// any statement
    body: Stmt,
}

/// A `do { ... } while (...)` statement.
///
/// ## Example
/// ```js
/// do body; while (cond);
/// // ^^^^^ body
/// //              ^^^^ test
/// ```
#[ast]
pub struct DoWhileStatement {
    /// any statement
    body: Stmt,
    test: Expr,
}

/// A `with` statement. Forbidden in strict mode.
///
/// ## Example
/// ```js
/// with (obj) body;
/// //    ^^^ object
/// //         ^^^^^ body
/// ```
#[ast]
pub struct WithStatement {
    /// any expression
    object: Expr,
    /// any statement
    body: Stmt,
}

/// The parameter list of a function.
///
/// ## Example
/// ```js
/// function f(a, b = 1, ...rest) {}
/// //         ^^^^^^^^ items
/// //                   ^^^^^^^ rest
/// ```
#[ast]
pub struct FormalParameters {
    /// `formal_parameter[]`. May also include `ts_parameter_property` entries
    /// (constructors only) and a leading `ts_this_parameter` (TypeScript).
    items: Vec<FormalParameterItem>,
    /// `binding_rest_element`. `None` when no rest element is present.
    rest: Option<BindingRestElement>,
    kind: FormalParameterKind,
}

/// A thin wrapper marking a binding pattern as a function parameter.
///
/// TypeScript metadata (decorators, type annotation, optional) lives on
/// the inner pattern.
#[ast]
pub struct FormalParameter {
    /// any binding pattern or `ts_this_parameter`.
    pattern: FormalParameterPattern,
}

/// An array literal expression.
///
/// ## Example
/// ```js
/// [a, , b, ...c]
/// // ^ elements[0]
/// //      ^ elements[2]
/// //         ^^^^ elements[3] (SpreadElement)
/// ```
///
/// `elements[1]` is `None` (the hole between the two commas).
#[ast]
pub struct ArrayExpression {
    /// any expression or `spread_element`. Entries are `None` for holes.
    elements: Vec<Option<Argument>>,
}

/// An object literal expression.
///
/// ## Example
/// ```js
/// { a: 1, b, ...c }
/// //^^^^ properties[0]
/// //      ^ properties[1] (shorthand)
/// //         ^^^^ properties[2] (SpreadElement)
/// ```
#[ast]
pub struct ObjectExpression {
    /// `object_property` or `spread_element`.
    properties: Vec<ObjectPropertyKind>,
}

/// A `...argument` element inside an array or object literal, or in a call
/// argument list.
///
/// ## Example
/// ```js
/// foo(...args)
/// //  ^^^^^^^ SpreadElement
/// //     ^^^^ argument
/// ```
#[ast]
pub struct SpreadElement {
    /// any expression
    argument: Expr,
}

/// A property in an object literal.
///
/// ## Example
/// ```js
/// ({
///   a: 1,        // kind = init
///   b() {},      // method = true
///   c,           // shorthand = true
///   [k]: v,      // computed = true
///   get d() {},  // kind = get
/// })
/// ```
#[ast]
pub struct ObjectProperty {
    /// `identifier_name`, `string_literal`, `numeric_literal`,
    /// `private_identifier` (class members only), or any expression when `computed = true`
    key: PropertyKey,
    /// any expression for `init` properties. `function` for methods, getters,
    /// and setters.
    value: Expr,
    kind: PropertyKind,
    /// true when written as the shorthand method form `b() {}`.
    method: bool,
    /// true when written as the shorthand `{ a }` form.
    shorthand: bool,
    /// true when the key is written inside `[...]`.
    computed: bool,
}

/// An expression wrapped in parentheses.
///
/// See: <https://tc39.es/ecma262/#prod-ParenthesizedExpression>
///
/// ## Example
/// ```js
/// x + (a + b)
/// //   ^^^^^ expression
/// ```
#[ast]
pub struct ParenthesizedExpression {
    /// any expression
    expression: Expr,
}

/// An arrow function expression.
///
/// See: <https://tc39.es/ecma262/#prod-ArrowFunction>
///
/// ## Example
/// ```ts
/// async <T>(x: T): T => x
/// //    ^^^ type_parameters
/// //       ^^^^^^ params
/// //             ^^^ return_type
/// //                    ^ body (expression = true)
/// ```
#[ast]
pub struct ArrowFunctionExpression {
    /// true for a concise body `() => expr`. false for a block body
    /// `() => { ... }`.
    expression: bool,
    r#async: bool,
    /// `ts_type_parameter_declaration`. `None` when absent.
    type_parameters: Option<TsTypeParameterDeclaration>,
    /// `formal_parameters`
    params: FormalParameters,
    /// `ts_type_annotation`. `None` when absent.
    return_type: Option<TsTypeAnnotation>,
    /// `function_body` when `expression` is false. any expression otherwise.
    body: ArrowFunctionBody,
}

/// A comma-separated sequence of expressions.
///
/// ## Example
/// ```js
/// (a, b, c)
/// // ^^^^^^^ expressions
/// ```
#[ast]
pub struct SequenceExpression {
    /// any expression
    expressions: Vec<Expr>,
}

/// Property access, in static, computed, or optional form.
///
/// See: <https://tc39.es/ecma262/#prod-MemberExpression>
///
/// ## Example
/// ```js
/// obj.foo          // computed = false, optional = false
/// obj[expr]        // computed = true
/// obj?.foo         // optional = true
/// obj.#priv        // property is a PrivateIdentifier
/// ```
#[ast]
pub struct MemberExpression {
    /// any expression
    object: Expr,
    /// any expression when `computed`. `identifier_name` or `private_identifier`
    /// otherwise.
    property: PropertyKey,
    computed: bool,
    /// true for `obj?.foo`.
    optional: bool,
}

/// A function call.
///
/// See: <https://tc39.es/ecma262/#prod-CallExpression>
///
/// ## Example
/// ```ts
/// foo<T>(a, ...b)
/// // ^^^ type_arguments
/// //     ^^^^^^^ arguments
/// ```
///
/// `optional` is true for `foo?.()`.
#[ast]
pub struct CallExpression {
    /// any expression
    callee: Expr,
    /// `ts_type_parameter_instantiation`. `None` when absent.
    type_arguments: Option<TsTypeParameterInstantiation>,
    /// any expression or `spread_element`.
    arguments: Vec<Argument>,
    /// true for `foo?.()`.
    optional: bool,
}

/// Wraps an optional chain so that short-circuiting applies to the whole
/// chain.
///
/// See: MDN Optional chaining
/// developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Optional_chaining
///
/// ## Example
/// ```js
/// foo?.bar.baz
/// // ^^^^^^^^^^ expression
/// ```
#[ast]
pub struct ChainExpression {
    /// `call_expression` or `member_expression` containing an optional link
    /// somewhere in the chain.
    expression: Expr,
}

/// A tagged template expression.
///
/// ## Example
/// ```ts
/// obj.tag<T>`hello`
/// //  ^^^ tag
/// //     ^^^ type_arguments
/// //        ^^^^^^^ quasi
/// ```
#[ast]
pub struct TaggedTemplateExpression {
    /// any expression
    tag: Expr,
    /// `ts_type_parameter_instantiation`. `None` when absent.
    type_arguments: Option<TsTypeParameterInstantiation>,
    /// `template_literal`
    quasi: TemplateLiteral,
}

/// A `new` expression.
///
/// See: <https://tc39.es/ecma262/#prod-NewExpression>
///
/// ## Example
/// ```ts
/// new Foo<T>(a, b)
/// //  ^^^ callee
/// //     ^^^ type_arguments
/// //         ^^^^ arguments
/// ```
#[ast]
pub struct NewExpression {
    /// any expression
    callee: Expr,
    /// `ts_type_parameter_instantiation`. `None` when absent.
    type_arguments: Option<TsTypeParameterInstantiation>,
    /// any expression or `spread_element`.
    arguments: Vec<Argument>,
}

/// An `await` expression.
///
/// See: <https://tc39.es/ecma262/#prod-AwaitExpression>
///
/// ## Example
/// ```js
/// await promise
/// //    ^^^^^^^ argument
/// ```
#[ast]
pub struct AwaitExpression {
    /// any expression
    argument: Expr,
}

/// A `yield` or `yield*` expression.
///
/// See: <https://tc39.es/ecma262/#prod-YieldExpression>
///
/// ## Example
/// ```js
/// yield x      // delegate = false
/// yield* iter  // delegate = true
/// yield        // argument = None
/// ```
#[ast]
pub struct YieldExpression {
    /// any expression. `None` for bare `yield`.
    argument: Option<Expr>,
    /// true for `yield*`.
    delegate: bool,
}

/// A meta property.
///
/// See: <https://tc39.es/ecma262/#prod-MetaProperty>
///
/// ## Example
/// ```js
/// import.meta
/// //     ^^^^ property
/// new.target
/// //  ^^^^^^ property
/// ```
///
/// `meta` is the head keyword (`import` or `new`).
#[ast]
pub struct MetaProperty {
    /// `identifier_name`
    meta: IdentifierName,
    /// `identifier_name`
    property: IdentifierName,
}

/// A dynamic `import()` call or phased import.
///
/// See: [Source phase imports](https://github.com/tc39/proposal-source-phase-imports),
/// [Deferred import evaluation](https://github.com/tc39/proposal-defer-import-eval)
///
/// ## Example
/// ```js
/// import("m")                 // phase = null
/// import("m", { with: {} })   // options set
/// import.source("m")          // phase = Source
/// import.defer("m")           // phase = Defer
/// import.sync("m")            // phase = null, sync = true
/// ```
#[ast]
pub struct ImportExpression {
    /// any expression. The module specifier expression.
    source: Expr,
    /// any expression. `None` when no options argument was passed.
    options: Option<Expr>,
    /// `null` for a plain `import(...)`.
    phase: Option<ImportPhase>,
    /// Import Sync
    /// See: <https://github.com/tc39/proposal-import-sync>
    #[estree(skip = "https://github.com/estree/estree/pull/335 get merged but not yet released")]
    sync: bool,
}

/// A static `import` declaration.
///
/// See: <https://tc39.es/ecma262/#prod-ImportDeclaration>
///
/// ## Example
/// ```ts
/// import { foo as bar } from "m";
/// //       ^^^^^^^^^^ specifiers[0]
/// //                         ^^^ source
/// ```
///
/// `import_kind = Type` for `import type { ... }`. `phase` is set for the
/// `import source` and `import defer` forms. `attributes` holds a trailing
/// `with { ... }` clause.
#[ast]
pub struct ImportDeclaration {
    /// `import_specifier`, `import_default_specifier`, or
    /// `import_namespace_specifier`. Empty for side-effect-only imports.
    specifiers: Vec<ImportDeclarationSpecifier>,
    /// `string_literal`
    source: StringLiteral,
    /// `import_attribute[]`
    attributes: Vec<ImportAttribute>,
    /// `null` for a regular import.
    phase: Option<ImportPhase>,
    /// `Type` for `import type { ... }`.
    import_kind: ImportOrExportKind,
}

/// A single `{ imported as local }` specifier in an import declaration.
///
/// ## Example
/// ```ts
/// import { foo as bar } from "m";
/// //       ^^^ imported
/// //              ^^^ local
///
/// import { type T } from "m";
/// //       ^^^^ import_kind = type
/// ```
#[ast]
pub struct ImportSpecifier {
    /// `identifier_name` or `string_literal`.
    imported: ModuleExportName,
    /// `binding_identifier`
    local: BindingIdentifier,
    /// `Type` for `import { type X }`.
    import_kind: ImportOrExportKind,
}

/// The default-binding specifier in an import declaration.
///
/// ## Example
/// ```js
/// import local from "m";
/// //     ^^^^^ local
/// ```
#[ast]
pub struct ImportDefaultSpecifier {
    /// `binding_identifier`
    local: BindingIdentifier,
}

/// A `* as local` namespace import specifier.
///
/// ## Example
/// ```js
/// import * as ns from "m";
/// //          ^^ local
/// ```
#[ast]
pub struct ImportNamespaceSpecifier {
    /// `binding_identifier`
    local: BindingIdentifier,
}

/// A single `key: value` attribute in a `with { ... }` clause.
///
/// ## Example
/// ```js
/// import x from "m" with { type: "json" };
/// //                       ^^^^ key
/// //                             ^^^^^^ value
/// ```
#[ast]
pub struct ImportAttribute {
    /// `identifier_name` or `string_literal`.
    key: ModuleExportName,
    /// `string_literal`
    value: StringLiteral,
}

/// An `export { ... }` or `export <decl>` declaration.
///
/// ## Example
/// ```ts
/// export type { foo, bar } from "m";
/// //     ^^^^ export_kind = type
/// //            ^^^^^^^^ specifiers
/// //                            ^^^ source
/// ```
///
/// For `export const x = 1`, `declaration` is the inner
/// `variable_declaration` and `specifiers` is empty.
#[ast]
pub struct ExportNamedDeclaration {
    /// `variable_declaration`, `function`, `class`, `ts_type_alias_declaration`,
    /// `ts_interface_declaration`, `ts_enum_declaration`, or
    /// `ts_module_declaration`. `None` for the `export { ... }` form.
    declaration: Option<Decl>,
    /// `export_specifier[]`
    specifiers: Vec<ExportSpecifier>,
    /// `string_literal`. `None` when there is no `from` clause.
    source: Option<StringLiteral>,
    /// `import_attribute[]`
    attributes: Vec<ImportAttribute>,
    /// `Type` for `export type { ... }`.
    export_kind: ImportOrExportKind,
}

/// An `export default ...` declaration.
///
/// ## Example
/// ```js
/// export default foo;
/// //             ^^^ declaration
/// export default function () {}
/// //             ^^^^^^^^^^^^^^ declaration
/// ```
#[ast]
pub struct ExportDefaultDeclaration {
    /// any expression, `function`, or `class`.
    declaration: ExportDefaultDeclarationKind,
}

/// An `export * from "m"` or `export * as ns from "m"` declaration.
///
/// ## Example
/// ```ts
/// export * as ns from "m";
/// //          ^^ exported
/// //                  ^^^ source
/// ```
///
/// `export_kind = Type` for `export type * from "..."`.
#[ast]
pub struct ExportAllDeclaration {
    /// `identifier_name` or `string_literal`. `None` for `export *` without
    /// `as`.
    exported: Option<ModuleExportName>,
    /// `string_literal`
    source: StringLiteral,
    /// `import_attribute[]`
    attributes: Vec<ImportAttribute>,
    /// `Type` for `export type * from "..."`.
    export_kind: ImportOrExportKind,
}

/// A single `{ local as exported }` specifier in an export declaration.
///
/// ## Example
/// ```ts
/// export { foo as bar };
/// //       ^^^ local
/// //              ^^^ exported
///
/// export { type X };
/// //       ^^^^ export_kind = type
/// ```
#[ast]
pub struct ExportSpecifier {
    /// `identifier_reference`. `identifier_name` or `string_literal` when
    /// re-exporting from a module.
    local: ModuleExportName,
    /// `identifier_name` or `string_literal`.
    exported: ModuleExportName,
    /// `Type` for `export { type X }`.
    export_kind: ImportOrExportKind,
}

/// A hashbang comment at the top of a source file.
///
/// ## Example
/// ```js
/// #!/usr/bin/env node
/// ```
///
/// `value` is everything after `#!`, so `/usr/bin/env node` here.
#[ast]
pub struct Hashbang {
    value: AstUtf8,
}
