use std::mem;

use swc_next_ast_macros::ast;

use crate::{
    ExtraData,
    node_id::{ExtraDataValue, sealed::ExtraDataValue as SealedExtraDataValue},
};

macro_rules! impl_extra_data_value {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl SealedExtraDataValue for $ty {}
            impl ExtraDataValue for $ty {
                #[inline]
                fn into_extra_data(self) -> ExtraData {
                    ExtraData::from_raw(self as u64)
                }

                #[inline]
                unsafe fn from_extra_data(data: &ExtraData) -> Self {
                    unsafe { mem::transmute(data.raw() as u8) }
                }
            }
        )+
    };
}

/// The prefix operator used by `ts_type_operator`.
///
/// `keyof` produces the union of property keys of the operand. `unique`
/// marks a unique symbol type (only meaningful before `symbol`). `readonly`
/// applies to tuple and array types to make their elements immutable.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TsTypeOperatorKind {
    Keyof,
    Unique,
    Readonly,
}

/// The kind of a method signature. Accessors use `get` or `set`. All
/// other method signatures use `method`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TsMethodSignatureKind {
    Method,
    Get,
    Set,
}

/// The keyword used to introduce a `ts_module_declaration`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TsModuleDeclarationKind {
    Namespace,
    Module,
}

/// The tri-state `+` or `-` modifier that may decorate the `?` or
/// `readonly` slots of a `ts_mapped_type`.
///
/// `None` means the modifier keyword is absent. `True` means it is present
/// without a sign. `Plus` and `Minus` explicitly add (`+?`, `+readonly`)
/// or remove (`-?`, `-readonly`) the modifier from the mapped property.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TsMappedTypeModifier {
    None,
    True,
    Plus,
    Minus,
}

impl_extra_data_value!(
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

#[ast(repr, nested)]
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

#[ast(repr, nested)]
pub enum TsTupleElement {
    TsType(TsType),
    TsNamedTupleMember(TsNamedTupleMember),
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

#[ast(repr, nested)]
pub enum TsModuleReference {
    TsTypeName(TsTypeName),
    TsExternalModuleReference(TsExternalModuleReference),
}

/// A body-less TypeScript function declaration or overload signature.
///
/// ## Example
/// ```ts
/// declare function foo<T>(x: T): T;
/// //^^^^^^^ declare
/// //               ^^^ id
/// //                  ^^^ type_parameters
/// //                     ^^^^^^ params
/// //                           ^^^ return_type
/// ```
#[ast]
pub struct TsFunction {
    /// `binding_identifier`. `None` for anonymous default exports and method
    /// definitions.
    id: Option<BindingIdentifier>,
    /// `ts_type_parameter_declaration`. `None` when absent.
    type_parameters: Option<TsTypeParameterDeclaration>,
    /// The explicit receiver type; does not consume a runtime argument.
    this_param: Option<TsThisParameter>,
    /// `formal_parameters`, excluding the explicit receiver.
    params: FormalParameters,
    /// `ts_type_annotation`. `None` when absent.
    return_type: Option<TsTypeAnnotation>,
    generator: bool,
    r#async: bool,
    /// true when preceded by the `declare` modifier. Distinguishes a real
    /// ambient declaration from a plain overload signature.
    declare: bool,
}

/// A body-less TypeScript method, accessor, or constructor signature in a
/// class body.
///
/// ## Example
/// ```ts
/// abstract class C { public abstract foo?<T>(x: T): void; }
/// //                 ^^^^^^ accessibility
/// //                        ^^^^^^^^ abstract
/// //                                 ^^^ key
/// //                                    ^ optional
/// //                                     ^^^ type_parameters
/// //                                        ^^^^^^ params
/// //                                              ^^^^^ return_type
/// ```
#[ast]
pub struct TsMethodDefinition {
    /// `decorator[]`
    decorators: SubRange<Decorator>,
    /// `identifier_name`, `string_literal`, `numeric_literal`,
    /// `private_identifier` (class members only), or any expression when `computed = true`
    key: PropertyKey,
    /// `id` is always `None`; `declare` is always `false`.
    value: TsFunction,
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

/// A `: Type` annotation wrapper.
///
/// The span starts at the `:` token and covers the inner type.
///
/// ## Example
/// ```ts
/// let x: number = 0;
/// //   ^^^^^^^^ TSTypeAnnotation
/// //     ^^^^^^ type_annotation
/// ```
#[ast]
pub struct TsTypeAnnotation {
    /// any ts type
    type_annotation: TsType,
}

/// The `any` primitive type. Disables all type checking for the annotated
/// value.
///
/// See: TypeScript Handbook any
/// www.typescriptlang.org/docs/handbook/2/everyday-types.html#any
///
/// ## Example
/// ```ts
/// let x: any;
/// //     ^^^ TSAnyKeyword
/// ```
#[ast]
pub struct TsAnyKeyword {}

/// The `unknown` primitive type. The type-safe counterpart of `any`.
///
/// See: TypeScript 3.0 Release Notes unknown
/// www.typescriptlang.org/docs/handbook/release-notes/typescript-3-0.html#new-unknown-top-type
///
/// ## Example
/// ```ts
/// let x: unknown;
/// //     ^^^^^^^ TSUnknownKeyword
/// ```
#[ast]
pub struct TsUnknownKeyword {}

/// The `never` primitive type. Represents values that never occur, for
/// example the return type of a function that always throws.
///
/// See: TypeScript Handbook never
/// www.typescriptlang.org/docs/handbook/2/functions.html#never
///
/// ## Example
/// ```ts
/// function fail(): never { throw new Error(); }
/// //               ^^^^^ TSNeverKeyword
/// ```
#[ast]
pub struct TsNeverKeyword {}

/// The `void` keyword used in type position. Typically the return type of a
/// function that returns no meaningful value.
///
/// ## Example
/// ```ts
/// function log(): void {}
/// //              ^^^^ TSVoidKeyword
/// ```
#[ast]
pub struct TsVoidKeyword {}

/// The `null` keyword used in type position.
///
/// ## Example
/// ```ts
/// let x: string | null;
/// //              ^^^^ TSNullKeyword
/// ```
#[ast]
pub struct TsNullKeyword {}

/// The `undefined` keyword used in type position.
///
/// ## Example
/// ```ts
/// let x: string | undefined;
/// //              ^^^^^^^^^ TSUndefinedKeyword
/// ```
#[ast]
pub struct TsUndefinedKeyword {}

/// The `string` primitive type.
///
/// ## Example
/// ```ts
/// let x: string;
/// //     ^^^^^^ TSStringKeyword
/// ```
#[ast]
pub struct TsStringKeyword {}

/// The `number` primitive type.
///
/// ## Example
/// ```ts
/// let x: number;
/// //     ^^^^^^ TSNumberKeyword
/// ```
#[ast]
pub struct TsNumberKeyword {}

/// The `bigint` primitive type. Integer values of arbitrary precision.
///
/// ## Example
/// ```ts
/// let x: bigint;
/// //     ^^^^^^ TSBigIntKeyword
/// ```
#[ast]
pub struct TsBigIntKeyword {}

/// The `boolean` primitive type.
///
/// ## Example
/// ```ts
/// let x: boolean;
/// //     ^^^^^^^ TSBooleanKeyword
/// ```
#[ast]
pub struct TsBooleanKeyword {}

/// The `symbol` primitive type.
///
/// ## Example
/// ```ts
/// let x: symbol;
/// //     ^^^^^^ TSSymbolKeyword
/// ```
#[ast]
pub struct TsSymbolKeyword {}

/// The `object` primitive type. Any non-primitive value.
///
/// ## Example
/// ```ts
/// let x: object;
/// //     ^^^^^^ TSObjectKeyword
/// ```
#[ast]
pub struct TsObjectKeyword {}

/// The `intrinsic` keyword. Marks a type as built into the TypeScript
/// compiler, for example `Uppercase<T>` and other string-manipulation
/// utilities.
///
/// ## Example
/// ```ts
/// type Uppercase<S extends string> = intrinsic;
/// //                                 ^^^^^^^^^ TSIntrinsicKeyword
/// ```
#[ast]
pub struct TsIntrinsicKeyword {}

/// The polymorphic `this` type. Refers to the type of the enclosing class
/// or interface at the usage site.
///
/// ## Example
/// ```ts
/// class C { self(): this { return this; } }
/// //                ^^^^ TSThisType
/// ```
#[ast]
pub struct TsThisType {}

/// A reference to a named type, optionally applied to type arguments.
///
/// ## Example
/// ```ts
/// let x: Foo;
/// //     ^^^ TSTypeReference (type_arguments = None)
/// let y: Promise<number>;
/// //     ^^^^^^^^^^^^^^^ TSTypeReference
/// //     ^^^^^^^ type_name
/// //            ^^^^^^^^ type_arguments
/// let z: Tools.Pos;
/// //     ^^^^^^^^^ TSTypeReference (type_name is a TSQualifiedName)
/// ```
#[ast]
pub struct TsTypeReference {
    /// `identifier_reference`, `ts_qualified_name`, or `this_expression`
    type_name: TsTypeName,
    /// `ts_type_parameter_instantiation`. `None` when absent.
    type_arguments: Option<TsTypeParameterInstantiation>,
}

/// A dotted type name like `A.B.C`. Left associative, so `A.B.C` parses as
/// `(A.B).C` with the outer `ts_qualified_name` holding the inner one as
/// `left`.
///
/// ## Example
/// ```ts
/// let x: Tools.Pos;
/// //     ^^^^^^^^^ TSQualifiedName
/// //     ^^^^^ left
/// //           ^^^ right
/// ```
#[ast]
pub struct TsQualifiedName {
    /// `identifier_reference`, `identifier_name`, `ts_qualified_name`, or
    /// `this_expression`.
    left: TsQualifiedNameLeft,
    /// `identifier_name`
    right: IdentifierName,
}

/// The `typeof` type operator applied to a value reference. Extracts the
/// type of an existing binding or a dotted member path at the usage site.
///
/// ## Example
/// ```ts
/// let x: typeof console;
/// //     ^^^^^^^^^^^^^^ TSTypeQuery
/// let y: typeof console.log;
/// //     ^^^^^^^^^^^^^^^^^^ TSTypeQuery (expr_name is a TSQualifiedName)
/// let z: typeof Err<number>;
/// //     ^^^^^^^^^^^^^^^^^^ TSTypeQuery (with type_arguments)
/// let w: typeof import("foo").Bar;
/// //     ^^^^^^^^^^^^^^^^^^^^^^^^ TSTypeQuery (expr_name is a TSImportType)
/// ```
#[ast]
pub struct TsTypeQuery {
    /// `identifier_reference`, `ts_qualified_name`, or `ts_import_type`.
    expr_name: TsTypeQueryExprName,
    /// `ts_type_parameter_instantiation`. `None` when absent.
    type_arguments: Option<TsTypeParameterInstantiation>,
}

/// A reference to a named type imported from a module path, written
/// `import("module").Foo<T>` in type position. The `import("...")` head
/// names the module. An optional dotted `qualifier` selects a specific type
/// from the module namespace, and an optional `<T, U>` instantiates a
/// generic type.
///
/// ## Example
/// ```ts
/// type A = import("./mod");
/// //       ^^^^^^^^^^^^^^^ TSImportType
/// type B = import("./mod").Foo;
/// //       ^^^^^^^^^^^^^^^^^^^ TSImportType (qualifier is an IdentifierName)
/// type C = import("./mod").Foo.Bar;
/// //       ^^^^^^^^^^^^^^^^^^^^^^^ TSImportType (qualifier is a TSQualifiedName)
/// type D = import("./mod").Foo<number>;
/// //       ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TSImportType (with type_arguments)
/// type E = import("./mod", { with: { type: "json" } });
/// //                        ^^^^^^^^^^^^^^^^^^^^^^^^ options
/// ```
#[ast]
pub struct TsImportType {
    /// `string_literal` naming the imported module.
    source: StringLiteral,
    /// `object_expression` carrying the import attributes object. `None`
    /// when absent.
    options: Option<Expr>,
    /// `identifier_name` for a single segment, or a left-associative
    /// `ts_qualified_name` chain whose leaves are all `identifier_name`s.
    /// `None` when there is no qualifier.
    qualifier: Option<TsImportTypeQualifier>,
    /// `ts_type_parameter_instantiation`. `None` when absent.
    type_arguments: Option<TsTypeParameterInstantiation>,
}

/// A single `<T>` type parameter introduced by a generic declaration.
/// Carries the parameter name along with the optional `extends` constraint,
/// optional default type, and the three variance and invariance modifier
/// keywords.
///
/// The span starts at the first modifier keyword when present, otherwise at
/// the name, and ends at the default if present, the constraint if
/// present, or the name.
///
/// ## Example
/// ```ts
/// type Foo<T extends Bar = Baz> = T;
/// //       ^^^^^^^^^^^^^^^^^^^ TSTypeParameter
/// //       ^                   name
/// //                 ^^^       constraint
/// //                       ^^^ default
/// type Rec<in out K, const T> = ...;
/// //       ^^^^^^^^^          in = true, out = true
/// //                 ^^^^^^^  const = true
/// ```
#[ast]
pub struct TsTypeParameter {
    /// `binding_identifier`
    name: BindingIdentifier,
    /// any ts type. `None` when there is no `extends` constraint.
    constraint: Option<TsType>,
    /// any ts type. `None` when there is no default.
    default: Option<TsType>,
    r#in: bool,
    /// true when the `out` variance modifier was written.
    out: bool,
    r#const: bool,
}

/// The `<T, U>` type parameter list introduced by a generic declaration
/// (type alias, interface, class, function, method, constructor, mapped
/// type, and so on). Holds the parameters in source order.
///
/// ## Example
/// ```ts
/// type Pair<A, B extends A> = [A, B];
/// //       ^^^^^^^^^^^^^^^^ TSTypeParameterDeclaration
/// //        ^  ^^^^^^^^^^^  params
/// ```
#[ast]
pub struct TsTypeParameterDeclaration {
    /// `ts_type_parameter[]`
    params: SubRange<TsTypeParameter>,
}

/// The `<T, U>` type argument list applied to a type reference, call site,
/// `new` expression, tagged template, JSX opening element, or instantiation
/// expression. Holds the arguments in source order.
///
/// ## Example
/// ```ts
/// let x: Promise<number, string>;
/// //            ^^^^^^^^^^^^^^^^ TSTypeParameterInstantiation
/// //             ^^^^^^  ^^^^^^ params
/// ```
#[ast]
pub struct TsTypeParameterInstantiation {
    /// any ts type
    params: SubRange<TsType>,
}

/// A literal value used in type position. Wraps a string, numeric, bigint,
/// boolean, or no-substitution template literal directly, or a
/// `unary_expression` when the literal is preceded by a `-` or `+` sign (for
/// example `-1`).
///
/// Template literals that contain interpolations use
/// `ts_template_literal_type` instead. The `null` keyword uses `ts_null_keyword`.
///
/// ## Example
/// ```ts
/// type A = "hello";
/// //       ^^^^^^^ TSLiteralType
/// type B = 42;
/// //       ^^ TSLiteralType
/// type C = true;
/// //       ^^^^ TSLiteralType
/// type D = -1;
/// //       ^^ TSLiteralType (literal is a UnaryExpression)
/// ```
#[ast]
pub struct TsLiteralType {
    /// `string_literal`, `numeric_literal`, `bigint_literal`, `boolean_literal`,
    /// `template_literal`, or `unary_expression` wrapping one of the numeric
    /// kinds.
    literal: TsLiteral,
}

/// A template literal used in type position with one or more
/// interpolations.
///
/// Parallels the expression-level `template_literal` but with types filling
/// the interpolation slots instead of expressions. `quasis` holds the
/// static text spans as `template_element` nodes and always has exactly
/// `types.len() + 1` elements. The final quasi is marked `tail = true`.
///
/// A template literal with no interpolations is still parsed as a
/// `ts_literal_type` wrapping a `template_literal`, not as a
/// `ts_template_literal_type`.
///
/// The span covers the opening and closing backticks.
///
/// ## Example
/// ```ts
/// type Greeting<N extends string> = `Hello, ${N}!`;
/// //                                 ^^^^^^^^^^^^^ TSTemplateLiteralType
/// //                                 ^^^^^^^^      quasis[0] ("Hello, ")
/// //                                         ^     types[0]  (N)
/// //                                          ^^   quasis[1] ("!", tail)
/// type Dot<T extends string, U extends string> = `${T}.${U}`;
/// //                                              ^^^^^^^^^ TSTemplateLiteralType
/// //                                              quasis: ["", ".", ""]
/// //                                              types:  [T, U]
/// ```
#[ast]
pub struct TsTemplateLiteralType {
    /// `template_element[]`. Always `types.len() + 1` elements.
    quasis: SubRange<TemplateElement>,
    /// any ts type. One entry per `${...}` slot.
    types: SubRange<TsType>,
}

/// An array type. Applies the postfix `[]` suffix to an element type.
///
/// Stacks naturally. `T[][]` is a `ts_array_type` whose `element_type` is
/// another `ts_array_type`.
///
/// ## Example
/// ```ts
/// let xs: number[];
/// //      ^^^^^^^^ TSArrayType
/// //      ^^^^^^ element_type
/// let ys: string[][];
/// //      ^^^^^^^^^^ TSArrayType (element_type is another TSArrayType)
/// ```
#[ast]
pub struct TsArrayType {
    /// any ts type
    element_type: TsType,
}

/// An indexed access type. Looks up the type of the property named by
/// `index_type` on `object_type`, mirroring expression-level member access
/// but in type position.
///
/// Stacks naturally. `T[K][L]` is a `ts_indexed_access_type` whose
/// `object_type` is another `ts_indexed_access_type`.
///
/// ## Example
/// ```ts
/// type A = Person["age"];
/// //       ^^^^^^^^^^^^^ TSIndexedAccessType
/// //       ^^^^^^ object_type
/// //              ^^^^^ index_type
/// type B = T[K];
/// //       ^^^^ TSIndexedAccessType
/// ```
#[ast]
pub struct TsIndexedAccessType {
    /// any ts type
    object_type: TsType,
    /// any ts type
    index_type: TsType,
}

/// A tuple type. A fixed length sequence of positional or named elements
/// whose types can differ from slot to slot. Elements may be marked
/// optional with `?` and the trailing element may be a rest element with
/// `...`.
///
/// Each entry in `element_types` is one of:
/// - a plain any ts type for a positional element
/// - a `ts_optional_type` for `Type?` in positional form
/// - a `ts_rest_type` for `...Type` in positional form, optionally wrapping a
///   `ts_named_tuple_member`
/// - a `ts_named_tuple_member` for `label: Type` or `label?: Type`
///
/// The span covers the opening `[` through the closing `]`.
///
/// ## Example
/// ```ts
/// type T = [string, number?, ...boolean[]];
/// //       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TSTupleType
/// type P = [first: string, second: number];
/// //       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TSTupleType with TSNamedTupleMember entries
/// ```
#[ast]
pub struct TsTupleType {
    /// any ts type, `ts_optional_type`, `ts_rest_type`, or `ts_named_tuple_member`.
    element_types: SubRange<TsTupleElement>,
}

/// A labeled element inside a tuple type. Carries a name that appears only
/// as documentation in TypeScript (it has no runtime effect) alongside the
/// element type. The whole member may be marked optional with a trailing
/// `?` on the label. The `?` modifies the tuple slot itself rather than
/// wrapping the inner type in a `ts_optional_type`.
///
/// When preceded by `...` the member is wrapped in a `ts_rest_type` whose
/// inner is this `ts_named_tuple_member`, matching TypeScript's spec shape for
/// named rest elements like `[...selectors: S]`.
///
/// ## Example
/// ```ts
/// type Pair = [first: string, second?: number];
/// //           ^^^^^^^^^^^^^^ TSNamedTupleMember (optional = false)
/// //                          ^^^^^^^^^^^^^^^^ TSNamedTupleMember (optional = true)
/// type R = [...rest: number[]];
/// //        ^^^^^^^^^^^^^^^^^ TSRestType wrapping TSNamedTupleMember
/// ```
#[ast]
pub struct TsNamedTupleMember {
    /// `identifier_name`
    label: IdentifierName,
    /// any ts type
    element_type: TsType,
    /// true when the label is followed by `?`.
    optional: bool,
}

/// An optional element inside a tuple type. Parsed as a `Type?` suffix on
/// an unnamed tuple element. The `?` marks the slot as optional without
/// changing the element's underlying type.
///
/// Only valid as a direct tuple element. An optional marker on a named
/// tuple element (`label?: Type`) is captured on
/// `TSNamedTupleMember.optional` rather than producing a `ts_optional_type`.
///
/// ## Example
/// ```ts
/// type T = [number, string?];
/// //                ^^^^^^^ TSOptionalType
/// //                ^^^^^^ type_annotation
/// ```
#[ast]
pub struct TsOptionalType {
    /// any ts type
    type_annotation: TsType,
}

/// A rest element inside a tuple type. Marks the trailing slot as
/// consuming zero or more elements whose type is the inner annotation.
///
/// The inner may be any any ts type (typically an array or tuple type) or a
/// `ts_named_tuple_member` for named rest elements like `[...rest: T[]]`.
///
/// ## Example
/// ```ts
/// type T = [number, ...string[]];
/// //                ^^^^^^^^^^^ TSRestType
/// //                   ^^^^^^^^ type_annotation
/// type U = [...rest: number[]];
/// //        ^^^^^^^^^^^^^^^^^ TSRestType wrapping TSNamedTupleMember
/// ```
#[ast]
pub struct TsRestType {
    /// any ts type or `ts_named_tuple_member`.
    type_annotation: TsType,
}

/// A JSDoc-style nullable type marker, written as a prefix `?T` or a
/// postfix `T?`.
///
/// ## Example
/// ```ts
/// let a: ?string;
/// //     ^^^^^^^ TSJSDocNullableType (postfix = false)
/// let b: number?;
/// //     ^^^^^^^ TSJSDocNullableType (postfix = true)
/// ```
#[ast]
pub struct TsJsDocNullableType {
    /// any ts type
    type_annotation: TsType,
    /// true when the `?` follows the type, false when it precedes it.
    postfix: bool,
}

/// A JSDoc-style non-nullable type marker. Written as a prefix `!T` or a
/// postfix `T!`.
///
/// ## Example
/// ```ts
/// let a: !string;
/// //     ^^^^^^^ TSJSDocNonNullableType (postfix = false)
/// let b: number!;
/// //     ^^^^^^^ TSJSDocNonNullableType (postfix = true)
/// ```
#[ast]
pub struct TsJsDocNonNullableType {
    /// any ts type
    type_annotation: TsType,
    /// true when the `!` follows the type, false when it precedes it.
    postfix: bool,
}

/// A JSDoc-style unknown type written as a bare `?`, valid only in a type
/// argument slot (`Foo<?>`, `Foo<?, T>`).
///
/// ## Example
/// ```ts
/// const x = foo<?>;
/// //            ^ TSJSDocUnknownType
/// ```
#[ast]
pub struct TsJsDocUnknownType {}

/// A union type. Combines two or more types with `|`, representing a value
/// that is any one of the constituents.
///
/// Binds looser than `ts_intersection_type`, so `A & B | C` parses as
/// `(A & B) | C`. A leading `|` before the first operand is allowed and is
/// preserved in the span. `type A = | string` yields a single-operand
/// `ts_union_type` whose span starts at the leading `|` rather than at the
/// first operand.
///
/// ## Example
/// ```ts
/// type A = string | number | boolean;
/// //       ^^^^^^^^^^^^^^^^^^^^^^^^^ TSUnionType
/// type B =
///     | { kind: "ok" }
///     | { kind: "err"; message: string };
/// ```
#[ast]
pub struct TsUnionType {
    /// any ts type
    types: SubRange<TsType>,
}

/// An intersection type. Combines two or more types with `&`, representing
/// a value that satisfies every constituent simultaneously.
///
/// ## Example
/// ```ts
/// type A = Named & Aged;
/// //       ^^^^^^^^^^^^ TSIntersectionType
/// type B = { name: string } & { age: number } & Serializable;
/// ```
#[ast]
pub struct TsIntersectionType {
    /// any ts type
    types: SubRange<TsType>,
}

/// A conditional type. Selects between two branches based on whether
/// `check_type` is assignable to `extends_type`.
///
/// See: TypeScript Handbook Conditional Types
/// www.typescriptlang.org/docs/handbook/2/conditional-types.html
///
/// ## Example
/// ```ts
/// type IsString<T> = T extends string ? "yes" : "no";
/// //                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TSConditionalType
/// //                 ^ check_type
/// //                           ^^^^^^ extends_type
/// //                                    ^^^^^ true_type
/// //                                            ^^^^ false_type
/// ```
#[ast]
pub struct TsConditionalType {
    /// any ts type
    check_type: TsType,
    /// any ts type
    extends_type: TsType,
    /// any ts type
    true_type: TsType,
    /// any ts type
    false_type: TsType,
}

/// An `infer` type placeholder. Introduces a new type variable that
/// captures a position inside the extends branch of a conditional type,
/// made available in the true branch of that conditional.
///
/// The span starts at the `infer` keyword and ends at the name or the
/// constraint when present.
///
/// ## Example
/// ```ts
/// type ReturnTypeOf<V> = V extends (...args: any[]) => infer R ? R : never;
/// //                                                   ^^^^^^^ TSInferType
/// //                                                         ^ type_parameter (name only)
/// type Head<T> = T extends [infer H extends string, ...any[]] ? H : never;
/// //                        ^^^^^^^^^^^^^^^^^^^^^^ TSInferType
/// //                              ^^^^^^^^^^^^^^^^ type_parameter (name + constraint)
/// ```
#[ast]
pub struct TsInferType {
    /// `ts_type_parameter`
    type_parameter: TsTypeParameter,
}

/// A prefix type operator. Wraps an inner type with `keyof`, `unique`, or
/// `readonly`.
///
/// The span starts at the operator keyword and extends to the end of the
/// inner type.
///
/// ## Example
/// ```ts
/// type Keys = keyof Person;
/// //          ^^^^^^^^^^^^ TSTypeOperator (operator = keyof)
/// let id: unique symbol;
/// //      ^^^^^^^^^^^^^ TSTypeOperator (operator = unique)
/// type R = readonly number[];
/// //       ^^^^^^^^^^^^^^^^^ TSTypeOperator (operator = readonly)
/// ```
#[ast]
pub struct TsTypeOperator {
    operator: TsTypeOperatorKind,
    /// any ts type
    type_annotation: TsType,
}

/// A parenthesized type. Wraps any type in parentheses for grouping and
/// precedence control, letting the inner type bind looser than any
/// surrounding postfix operators.
///
/// The span includes the opening and closing parentheses.
///
/// ## Example
/// ```ts
/// type A = (string | number)[];
/// //       ^^^^^^^^^^^^^^^^^ TSParenthesizedType
/// type B = (() => void) | null;
/// //       ^^^^^^^^^^^^ TSParenthesizedType
/// ```
#[ast]
pub struct TsParenthesizedType {
    /// any ts type
    type_annotation: TsType,
}

/// A TypeScript function type. Describes a callable signature in type
/// position, written with `=>` between the parameter list and the return
/// type. Function types may carry generic parameters: `<T>(x: T) => T`.
///
/// ## Example
/// ```ts
/// type F = (x: number) => string;
/// //       ^^^^^^^^^^^^^^^^^^^^^ TSFunctionType
/// type G = <T>(x: T) => T;
/// //       ^^^^^^^^^^^^^^ TSFunctionType with type_parameters
/// ```
#[ast]
pub struct TsFunctionType {
    /// `ts_type_parameter_declaration`. `None` when absent.
    type_parameters: Option<TsTypeParameterDeclaration>,
    /// The explicit receiver type; does not consume a runtime argument.
    this_param: Option<TsThisParameter>,
    /// `formal_parameters`, excluding the explicit receiver.
    params: FormalParameters,
    /// `ts_type_annotation` wrapping the return type. The wrapper's span
    /// starts at the `=>` token.
    return_type: TsTypeAnnotation,
}

/// A TypeScript constructor type. Describes a signature that is invoked
/// with `new`, optionally marked `abstract` to forbid direct instantiation
/// of the referenced class.
///
/// ## Example
/// ```ts
/// type C = new (x: number) => Foo;
/// //       ^^^^^^^^^^^^^^^^^^^^^^^ TSConstructorType
/// type A = abstract new <T>(x: T) => T;
/// //       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TSConstructorType (abstract = true)
/// ```
#[ast]
pub struct TsConstructorType {
    /// `ts_type_parameter_declaration`. `None` when absent.
    type_parameters: Option<TsTypeParameterDeclaration>,
    /// `formal_parameters`
    params: FormalParameters,
    /// `ts_type_annotation` wrapping the return type. The wrapper's span
    /// starts at the `=>` token.
    return_type: TsTypeAnnotation,
    r#abstract: bool,
}

/// A TypeScript type predicate. Appears only in a function's return-type
/// position and narrows the type of a parameter (or `this`) in the call
/// site's control-flow analysis.
///
/// See: TypeScript Handbook Type Predicates
/// www.typescriptlang.org/docs/handbook/2/narrowing.html#using-type-predicates
///
/// ## Example
/// ```ts
/// function isString(x: unknown): x is string { ... }
/// //                             ^^^^^^^^^^^ TSTypePredicate
/// function assertNumber(x: unknown): asserts x is number { ... }
/// //                                 ^^^^^^^^^^^^^^^^^^^ TSTypePredicate (asserts = true)
/// function assert(c: boolean): asserts c { ... }
/// //                           ^^^^^^^^^ TSTypePredicate (asserts = true, type_annotation = None)
/// ```
#[ast]
pub struct TsTypePredicate {
    /// `identifier_name` for a named parameter, or `ts_this_type` for the
    /// implicit `this` parameter.
    parameter_name: TsTypePredicateName,
    /// `ts_type_annotation` wrapping the narrowed type. The wrapper's span
    /// equals the inner type's span. `None` for a bare `asserts x`
    /// predicate that carries no `is Type` clause.
    type_annotation: Option<TsTypeAnnotation>,
    /// true when the predicate is introduced by the `asserts` keyword.
    asserts: bool,
}

/// An anonymous object type. Holds a list of signatures (properties,
/// methods, index signatures, call signatures, construct signatures) in
/// source order.
///
/// ## Example
/// ```ts
/// type Point = { x: number; y: number };
/// //           ^^^^^^^^^^^^^^^^^^^^^^^^ TSTypeLiteral
/// type Callable = { (x: number): string; length: number };
/// //              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TSTypeLiteral
/// ```
#[ast]
pub struct TsTypeLiteral {
    /// `ts_property_signature`, `ts_method_signature`,
    /// `ts_call_signature_declaration`, `ts_construct_signature_declaration`,
    /// or `ts_index_signature`
    members: SubRange<TsTypeElement>,
}

/// A TypeScript mapped type. Projects every key in a union to a new
/// property type: `{ [K in Keys]: Value }`. Supports the `as` remapping
/// clause, the `?` / `+?` / `-?` optionality modifiers, and the
/// `readonly` / `+readonly` / `-readonly` mutability modifiers.
///
/// See: TypeScript Handbook Mapped Types
/// www.typescriptlang.org/docs/handbook/2/mapped-types.html
///
/// ## Example
/// ```ts
/// type Readonly<T> = { readonly [K in keyof T]: T[K] };
/// //                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TSMappedType
/// //                            ^ key
/// //                                  ^^^^^^^ constraint
/// //                                              ^^^^ type_annotation
/// type Mutable<T> = { -readonly [K in keyof T]: T[K] };
/// //                  ^^^^^^^^^ readonly = minus
/// type Partial<T> = { [K in keyof T]?: T[K] };
/// //                                ^ optional = true
/// type Remap<T> = { [K in keyof T as `get${Capitalize<string & K>}`]: () => T[K] };
/// //                                ^^ name_type = template literal type
/// ```
#[ast]
pub struct TsMappedType {
    /// `binding_identifier` naming the binding introduced by the `in` clause
    /// (`K` in `[K in T]`). Visible inside `name_type` and
    /// `type_annotation`.
    key: BindingIdentifier,
    /// any ts type. The constraint the key iterates over, to the right of
    /// `in`.
    constraint: TsType,
    /// any ts type. The optional remapping type supplied after `as`. `None`
    /// when no `as` clause is present.
    name_type: Option<TsType>,
    /// any ts type. `None` when the annotation is omitted (`{ [K in T] }`
    /// is accepted by the grammar).
    type_annotation: Option<TsType>,
    /// the `?` modifier
    optional: TsMappedTypeModifier,
    /// the `readonly` modifier
    readonly: TsMappedTypeModifier,
}

/// A property declaration inside a type literal or interface body. Written
/// as `key: Type`, with optional `readonly` and `?` modifiers. A missing
/// type annotation is allowed.
///
/// ## Example
/// ```ts
/// type T = { readonly x?: number };
/// //         ^^^^^^^^^^^^^^^^^^^^ TSPropertySignature (readonly, optional)
/// //                    ^          key
/// //                      ^^^^^^^^ type_annotation
/// ```
#[ast]
pub struct TsPropertySignature {
    /// `identifier_name`, `string_literal`, `numeric_literal`,
    /// `private_identifier` (class members only), or any expression when `computed = true`
    key: PropertyKey,
    /// `ts_type_annotation`. `None` when absent.
    type_annotation: Option<TsTypeAnnotation>,
    /// true when the key is written inside `[...]`.
    computed: bool,
    /// true when a `?` follows the key.
    optional: bool,
    /// true when preceded by the `readonly` modifier.
    readonly: bool,
}

/// A method, getter, or setter declaration inside a type literal or
/// interface body. Carries the key, an optional generic parameter list,
/// the parameter list, and an optional return type.
///
/// Getters accept no parameters and may declare a return type. Setters
/// accept exactly one parameter and carry no return type
/// (`return_type = None`).
///
/// ## Example
/// ```ts
/// type T = {
///   add(a: number, b: number): number;
/// //^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TSMethodSignature (kind = method)
///   get size(): number;
/// //^^^^^^^^^^^^^^^^^^ TSMethodSignature (kind = get)
///   set name(v: string);
/// //^^^^^^^^^^^^^^^^^^^ TSMethodSignature (kind = set)
///   optional?<T>(x: T): T;
/// //^^^^^^^^^^^^^^^^^^^^^ TSMethodSignature (optional = true, type_parameters set)
/// };
/// ```
#[ast]
pub struct TsMethodSignature {
    /// `identifier_name`, `string_literal`, `numeric_literal`,
    /// `private_identifier` (class members only), or any expression when `computed = true`
    key: PropertyKey,
    /// `ts_type_parameter_declaration`. `None` when absent.
    type_parameters: Option<TsTypeParameterDeclaration>,
    /// The explicit receiver type; does not consume a runtime argument.
    this_param: Option<TsThisParameter>,
    /// `formal_parameters`, excluding the explicit receiver.
    params: FormalParameters,
    /// `ts_type_annotation`. `None` when absent. Setters always have
    /// `None`.
    return_type: Option<TsTypeAnnotation>,
    kind: TsMethodSignatureKind,
    /// true when the key is written inside `[...]`.
    computed: bool,
    /// true when a `?` follows the key.
    optional: bool,
}

/// A bare call signature inside a type literal or interface body. Written
/// as `(params): ReturnType`. Distinct from `ts_function_type`, which is
/// used as a standalone type. Call signatures are members of an enclosing
/// object type.
///
/// ## Example
/// ```ts
/// type Callable = { <T>(x: T): T };
/// //                ^^^^^^^^^^^^^ TSCallSignatureDeclaration
/// ```
#[ast]
pub struct TsCallSignatureDeclaration {
    /// `ts_type_parameter_declaration`. `None` when absent.
    type_parameters: Option<TsTypeParameterDeclaration>,
    /// The explicit receiver type; does not consume a runtime argument.
    this_param: Option<TsThisParameter>,
    /// `formal_parameters`, excluding the explicit receiver.
    params: FormalParameters,
    /// `ts_type_annotation`. `None` when absent.
    return_type: Option<TsTypeAnnotation>,
}

/// A bare construct signature inside a type literal or interface body.
/// Written as `new (params): ReturnType`. Distinct from `ts_constructor_type`,
/// which is used as a standalone type.
///
/// ## Example
/// ```ts
/// type Ctor = { new <T>(x: T): T };
/// //            ^^^^^^^^^^^^^^^^^ TSConstructSignatureDeclaration
/// ```
#[ast]
pub struct TsConstructSignatureDeclaration {
    /// `ts_type_parameter_declaration`. `None` when absent.
    type_parameters: Option<TsTypeParameterDeclaration>,
    /// `formal_parameters`
    params: FormalParameters,
    /// `ts_type_annotation`. `None` when absent.
    return_type: Option<TsTypeAnnotation>,
}

/// An index signature inside a type literal, interface body, or class
/// body. Written as `[name: KeyType]: ValueType`, optionally preceded by
/// `readonly`. The parameter list is a small array of identifier-like
/// bindings carrying a type annotation. Almost always one entry.
///
/// ## Example
/// ```ts
/// type Dict = { [k: string]: number };
/// //            ^^^^^^^^^^^^^^^^^^^^ TSIndexSignature
/// //             ^^^^^^^^^ parameters
/// //                         ^^^^^^^ type_annotation
/// type ReadOnly = { readonly [i: number]: string };
/// //                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TSIndexSignature (readonly)
/// ```
#[ast]
pub struct TsIndexSignature {
    parameters: SubRange<TsIndexSignatureParameter>,
    /// `ts_type_annotation`
    type_annotation: TsTypeAnnotation,
    /// true when preceded by the `readonly` modifier.
    readonly: bool,
    r#static: bool,
}

/// A TypeScript `type` alias declaration. Binds an identifier to a type,
/// optionally parameterized by one or more type parameters, and optionally
/// prefixed by the `declare` modifier for ambient contexts.
///
/// See: TypeScript Handbook Type Aliases
/// www.typescriptlang.org/docs/handbook/2/everyday-types.html#type-aliases
///
/// ## Example
/// ```ts
/// type Maybe<T> = T | null | undefined;
/// //   ^^^^^ id
/// //        ^^^ type_parameters
/// //              ^^^^^^^^^^^^^^^^^^^^ type_annotation
/// declare type Id = number;
/// // ^^^^^^^ declare = true
/// ```
#[ast]
pub struct TsTypeAliasDeclaration {
    /// `binding_identifier`
    id: BindingIdentifier,
    /// `ts_type_parameter_declaration`. `None` when the alias has no `<T>`
    /// list.
    type_parameters: Option<TsTypeParameterDeclaration>,
    /// any ts type. Stored as a bare type without a wrapping
    /// `ts_type_annotation` node.
    type_annotation: TsType,
    /// true when preceded by the `declare` modifier.
    declare: bool,
}

/// A TypeScript `interface` declaration. Introduces a named structural
/// type with an optional list of parent interfaces and a body of
/// signatures. May be prefixed by the `declare` modifier for ambient
/// contexts.
///
/// See: TypeScript Handbook Interfaces
/// www.typescriptlang.org/docs/handbook/2/objects.html#interfaces
///
/// ## Example
/// ```ts
/// interface Foo<T> extends Bar, Base.Thing<T> { a: T; b(): void }
/// //        ^^^ id
/// //           ^^^ type_parameters
/// //                       ^^^^^^^^^^^^^^^^^^ extends
/// //                                         ^^^^^^^^^^^^^^^^^^^^^^ body
/// declare interface Id { x: number }
/// // ^^^^^^^ declare = true
/// ```
#[ast]
pub struct TsInterfaceDeclaration {
    /// `binding_identifier`
    id: BindingIdentifier,
    /// `ts_type_parameter_declaration`. `None` when the interface has no
    /// `<T>` list.
    type_parameters: Option<TsTypeParameterDeclaration>,
    /// `ts_interface_heritage[]`. Empty when the interface has no `extends`
    /// clause.
    extends: SubRange<TsInterfaceHeritage>,
    /// `ts_interface_body`
    body: TsInterfaceBody,
    /// true when preceded by the `declare` modifier.
    declare: bool,
}

/// The body of an interface. Holds the signatures (property, method, call,
/// construct, index) in source order. Shares the same signature family as
/// `ts_type_literal`.
///
/// ## Example
/// ```ts
/// interface Foo { a: T; b(): void; [k: string]: U }
/// //            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TSInterfaceBody
/// ```
#[ast]
pub struct TsInterfaceBody {
    /// `ts_property_signature`, `ts_method_signature`,
    /// `ts_call_signature_declaration`, `ts_construct_signature_declaration`,
    /// or `ts_index_signature`
    body: SubRange<TsTypeElement>,
}

/// One entry in an interface's `extends` clause. The parent interface is
/// identified by a runtime expression (an identifier path) with an
/// optional `<T, U>` type argument list.
///
/// ## Example
/// ```ts
/// interface Foo extends Bar, Base.Thing<T> {}
/// //                    ^^^ TSInterfaceHeritage (expression = IdentifierReference "Bar")
/// //                         ^^^^^^^^^^^^^^^ TSInterfaceHeritage
/// //                         (expression = MemberExpression, type_arguments = <T>)
/// ```
#[ast]
pub struct TsInterfaceHeritage {
    /// `identifier_reference` or a left-associative `member_expression`
    /// chain of identifier names. Calls, computed access, and optional
    /// chaining are rejected by the grammar.
    expression: Expr,
    /// `ts_type_parameter_instantiation`. `None` when the heritage has no
    /// `<T>` arguments.
    type_arguments: Option<TsTypeParameterInstantiation>,
}

/// One entry in a class's `implements` clause. The implemented interface
/// is identified by a runtime expression (an identifier path) with an
/// optional `<T, U>` type argument list.
///
/// ## Example
/// ```ts
/// class Foo implements Bar, Base.Thing<T> {}
/// //                   ^^^ TSClassImplements (expression = IdentifierReference "Bar")
/// //                        ^^^^^^^^^^^^^^^ TSClassImplements
/// //                        (expression = MemberExpression, type_arguments = <T>)
/// ```
#[ast]
pub struct TsClassImplements {
    /// `identifier_reference` or a left-associative `member_expression`
    /// chain of identifier names. Calls, computed access, and optional
    /// chaining are rejected by the grammar.
    expression: Expr,
    /// `ts_type_parameter_instantiation`. `None` when the entry has no
    /// `<T>` arguments.
    type_arguments: Option<TsTypeParameterInstantiation>,
}

/// A TypeScript `enum` declaration. Creates a named runtime binding that
/// also acts as a type. May be prefixed by `const` (an enum whose members
/// are inlined at use sites) or `declare` (ambient declaration, no emitted
/// runtime), or both.
///
/// See: [TypeScript Handbook Enums](https://www.typescriptlang.org/docs/handbook/enums.html)
///
/// ## Example
/// ```ts
/// enum Color { Red, Green = 2, Blue }
/// //   ^^^^^ id
/// //         ^^^^^^^^^^^^^^^^^^^^^^^^ body
/// const enum Flags { A = 1, B = 2 }
/// // ^^^^^ is_const = true
/// declare enum Ambient { X, Y }
/// // ^^^^^^^ declare = true
/// ```
#[ast]
pub struct TsEnumDeclaration {
    /// `binding_identifier`
    id: BindingIdentifier,
    /// `ts_enum_body`
    body: TsEnumBody,
    /// true when preceded by the `const` modifier.
    is_const: bool,
    /// true when preceded by the `declare` modifier.
    declare: bool,
}

/// The body of an enum declaration. Holds the members in source order,
/// delimited by commas (a trailing comma is permitted).
///
/// ## Example
/// ```ts
/// enum Foo { A, B = 1, C }
/// //       ^^^^^^^^^^^^^^ TSEnumBody
/// ```
#[ast]
pub struct TsEnumBody {
    /// `ts_enum_member[]`
    members: SubRange<TsEnumMember>,
}

/// A single member in an enum body. The name is an identifier, a string
/// literal, or a no-substitution template literal. Literal names may have
/// a computed `[...]` wrapper. The value is an optional initializer expression.
///
/// ## Example
/// ```ts
/// enum E {
///     A,        // id = IdentifierName "A",  initializer = None
///     B = 1,    // id = IdentifierName "B",  initializer = NumericLiteral 1
///     "s" = 2,  // id = StringLiteral "s",   initializer = NumericLiteral 2
/// }
/// ```
#[ast]
pub struct TsEnumMember {
    /// `identifier_name`, `string_literal`, or `template_literal`. When the
    /// source used the computed `[...]` form, `computed` is true.
    id: TsEnumMemberName,
    /// any expression. `None` when there is no `=` initializer.
    initializer: Option<Expr>,
    /// true when the source wrote the name as a computed key `[...]`.
    computed: bool,
}

/// A TypeScript `namespace` or `module` declaration.
///
/// See: TypeScript Handbook Namespaces
/// www.typescriptlang.org/docs/handbook/namespaces.html
///
/// ## Example
/// ```ts
/// namespace Foo { ... }
/// //        ^^^ id, kind = namespace
/// namespace A.B.C { ... }
/// //        ^^^^^ id (TSQualifiedName, left-associative)
/// declare module "./mod" { ... }
/// //             ^^^^^^^ id (StringLiteral), kind = module, declare = true
/// ```
#[ast]
pub struct TsModuleDeclaration {
    /// `binding_identifier`, `string_literal`, or `ts_qualified_name`.
    id: TsModuleName,
    /// `ts_module_block`. `None` for body-less forward declarations like
    /// `declare module "foo";`.
    body: Option<TsModuleBody>,
    kind: TsModuleDeclarationKind,
    /// true when preceded by the `declare` modifier.
    declare: bool,
}

/// The body of a `namespace`, `module`, or `declare global` declaration.
/// Holds the inner statements and declarations in source order, delimited
/// by the enclosing `{` and `}`.
///
/// ## Example
/// ```ts
/// namespace Foo { var x = 1; class C {} }
/// //            ^^^^^^^^^^^^^^^^^^^^^^^^^^ TSModuleBlock
/// ```
#[ast]
pub struct TsModuleBlock {
    directives: SubRange<Directive>,
    body: SubRange<Stmt>,
}

/// A TypeScript `declare global { ... }` augmentation block.
///
/// ## Example
/// ```ts
/// declare global { interface Window { x: number } }
/// //      ^^^^^^ id
/// //             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ body
/// ```
#[ast]
pub struct TsGlobalDeclaration {
    /// `identifier_name` capturing the span of the `global` keyword.
    id: IdentifierName,
    /// `ts_module_block`
    body: TsModuleBlock,
    /// true when preceded by the `declare` modifier. In valid TypeScript
    /// `declare global` always has `declare = true`. A bare `global { ... }`
    /// nested inside an ambient `declare namespace` body has
    /// `declare = false`.
    declare: bool,
}

/// A TypeScript parameter property. A constructor parameter that carries
/// an accessibility, `readonly`, or `override` modifier and therefore
/// implicitly declares a class field of the same name and initial value.
///
/// See: TypeScript Handbook Parameter Properties
/// www.typescriptlang.org/docs/handbook/2/classes.html#parameter-properties
///
/// ## Example
/// ```ts
/// class C {
///   constructor(
///     public x: number,        // accessibility = public
///     readonly y: string,      // readonly
///     protected override z: T, // accessibility = protected, override
///   ) {}
/// }
/// ```
#[ast]
pub struct TsParameterProperty {
    /// `decorator[]`. Decorators preceding the modifier, in source order.
    decorators: SubRange<Decorator>,
    /// The parameter declaration wrapped by these property modifiers.
    parameter: FormalParameter,
    r#override: bool,
    /// true for the `readonly` modifier.
    readonly: bool,
    /// `None` when no accessibility modifier was written.
    accessibility: Accessibility,
}

/// A TypeScript explicit `this` parameter. Declares the type of the
/// `this` binding inside the function body. Not a real parameter. It
/// contributes nothing to the call's argument positions and is erased at
/// emit time. TypeScript requires `this` to be the first parameter when
/// present.
///
/// See: TypeScript Handbook this parameters
/// www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function
///
/// ## Example
/// ```ts
/// function f(this: void, x: number) {}
/// //         ^^^^^^^^^^ TSThisParameter
/// type Handler = (this: Element, e: Event) => void;
/// //              ^^^^^^^^^^^^^ TSThisParameter
/// interface I { m(this: this): void }
/// //              ^^^^^^^^^^ TSThisParameter (type_annotation = TSThisType)
/// ```
#[ast]
pub struct TsThisParameter {
    /// `ts_type_annotation` wrapping the declared type of `this`. `None`
    /// when the parameter is written as bare `this` with no annotation.
    type_annotation: Option<TsTypeAnnotation>,
}

/// TypeScript `expr as Type` postfix assertion.
///
/// See: TypeScript Handbook Type Assertions
/// www.typescriptlang.org/docs/handbook/2/everyday-types.html#type-assertions
///
/// ## Example
/// ```ts
/// const n = value as number;
/// //        ^^^^^ expression
/// //                 ^^^^^^ type_annotation
/// ```
#[ast]
pub struct TsAsExpression {
    /// any expression
    expression: Expr,
    /// any ts type
    type_annotation: TsType,
}

/// TypeScript `expr satisfies Type` postfix check.
///
/// ## Example
/// ```ts
/// const config = { port: 3000 } satisfies Config;
/// //             ^^^^^^^^^^^^^^ expression
/// //                                      ^^^^^^ type_annotation
/// ```
#[ast]
pub struct TsSatisfiesExpression {
    /// any expression
    expression: Expr,
    /// any ts type
    type_annotation: TsType,
}

/// TypeScript `<Type>expr` prefix type assertion.
///
/// ## Example
/// ```ts
/// const n = <number>value;
/// //         ^^^^^^ type_annotation
/// //                ^^^^^ expression
/// ```
#[ast]
pub struct TsTypeAssertion {
    /// any ts type
    type_annotation: TsType,
    /// any expression
    expression: Expr,
}

/// TypeScript `expr!` postfix non-null assertion.
///
/// ## Example
/// ```ts
/// const n = value!;
/// //        ^^^^^ expression
/// ```
#[ast]
pub struct TsNonNullExpression {
    /// any expression
    expression: Expr,
}

/// TypeScript `expr<T>` instantiation expression without call parens.
///
/// ## Example
/// ```ts
/// const f = makeBox<number>;
/// //        ^^^^^^^ expression
/// //               ^^^^^^^^ type_arguments
/// ```
#[ast]
pub struct TsInstantiationExpression {
    /// any expression. Any left-hand-side expression.
    expression: Expr,
    /// `ts_type_parameter_instantiation`
    type_arguments: TsTypeParameterInstantiation,
}

/// TypeScript `export = expr` (CommonJS-style ambient export).
///
/// See: TypeScript Handbook export =
/// www.typescriptlang.org/docs/handbook/modules.html#export--and-import--require
///
/// ## Example
/// ```ts
/// export = MyNamespace;
/// //       ^^^^^^^^^^^ expression
/// ```
#[ast]
pub struct TsExportAssignment {
    /// any expression
    expression: Expr,
}

/// TypeScript `export as namespace Name` (UMD ambient namespace export).
///
/// ## Example
/// ```ts
/// export as namespace MyLib;
/// //                  ^^^^^ id
/// ```
#[ast]
pub struct TsNamespaceExportDeclaration {
    /// `binding_identifier`
    id: IdentifierName,
}

/// TypeScript `import x = <module reference>` declaration. Binds the local
/// name `id` to either an external module (`require("m")`) or an entity
/// name path (`Foo.Bar`).
///
/// See: TypeScript Handbook import = and export =
/// www.typescriptlang.org/docs/handbook/modules.html#export--and-import--require
///
/// ## Example
/// ```ts
/// import fs = require("fs");
/// //     ^^ id
/// //          ^^^^^^^^^^^^ module_reference (TSExternalModuleReference)
/// import alias = Foo.Bar;
/// //     ^^^^^ id
/// //             ^^^^^^^ module_reference (TSQualifiedName)
/// import alias = Foo;
/// //     ^^^^^ id
/// //             ^^^ module_reference (IdentifierReference)
/// ```
#[ast]
pub struct TsImportEqualsDeclaration {
    /// `binding_identifier`
    id: BindingIdentifier,
    /// `ts_external_module_reference`, `identifier_reference`, or
    /// `ts_qualified_name`.
    module_reference: TsModuleReference,
    /// `Type` for `import type x = ...`. `Value` otherwise.
    import_kind: ImportOrExportKind,
}

/// TypeScript `require("module")` on the right hand side of an
/// `import x = require("m")` declaration.
///
/// ## Example
/// ```ts
/// import x = require("./m");
/// //         ^^^^^^^^^^^^^^^ TSExternalModuleReference
/// //                 ^^^^^^ expression
/// ```
#[ast]
pub struct TsExternalModuleReference {
    /// `string_literal` naming the required module.
    expression: StringLiteral,
}

/// The named, typed binding of an index signature.
#[ast]
pub struct TsIndexSignatureParameter {
    name: BindingIdentifier,
    type_annotation: TsTypeAnnotation,
}
