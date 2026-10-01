export declare class Foo {
  private constructor();
}
export declare class Bar {
  constructor(a?: number);
}
export declare class Zoo {
  foo<F>(f: F): F;
}
export declare abstract class Qux {
  abstract foo(): void;
  protected foo2?(): void;
  bar(): void;
  baz(): void;
}
export declare class Baz {
  readonly prop1 = /**
   * Just a comment
   */
"some string";
  prop2: string;
  private prop3;
  private prop4;
  private static prop5;
  private static prop6;
  /**
   * Just a comment
   */
  /**
   * Just a comment
   */
  /**
   * Just a comment
   */
  /**
   * Just a comment
   */
  /**
   * Just a comment
   */
}
export declare class Boo {
  readonly prop: number;
  private readonly prop2;
  readonly prop3: number;
  constructor(prop?: number, prop2?: number, prop3?: number);
}
export declare class Bux {
  readonly prop: number;
  private readonly prop2;
  readonly prop3: number;
  private constructor();
}
export declare class PrivateFieldsWithConstructorAssignments {
  first: number;
  private second;
  constructor(first: number);
}
export declare class DefaultedAnyParameterProperty {
  value: any;
  constructor(value: any | undefined, required: string);
}
export declare class DefaultedArrayParameterProperty {
  values: unknown;
  constructor(values: unknown | undefined, required: string);
}
export declare class PrivateMethodClass {
  private good;
  private get goodGetter();
}
export declare class PublicMethodClass {
  bad(a): void;
  get badGetter(): {
    x: number;
  };
}
export declare class ConstructorOverloadsClass {
  readonly a: any;
  constructor(a: number);
  constructor(a: string);
}
export declare class ClassParameterProperties {
  readonly b: string;
  constructor(a: number, b: string);
}
export declare class PrivateConstructorWithParameterProperties {
  readonly bar: string;
  private constructor();
}
export declare class ProtectedConstructorWithParameterProperties {
  foo: string;
  private readonly bar;
  protected baz: boolean;
  readonly qux: string;
  protected constructor(foo: string, bar: number, baz: boolean, qux: string);
}
export declare class PrivateConstructorMixedParameterProperties {
  publicProp: string;
  private privateProp;
  protected protectedProp: boolean;
  readonly readonlyProp: string;
  readonly publicReadonlyProp: number;
  private readonly privateReadonlyProp;
  private constructor();
}
export declare class PrivateConstructorWithOverloads {
  private constructor();
  private constructor();
  private constructor();
}
export declare class PrivateConstructorWithOptionalParameters {
  publicOptional?: boolean | undefined;
  private privateOptional?;
  readonly readonlyOptional?: number | undefined;
  private constructor();
}
export declare class PrivateConstructorWithRestParameters {
  private constructor();
}
export declare class PrivateConstructorWithDefaultParameters {
  prop1: string;
  private prop2;
  readonly prop3: boolean;
  private constructor();
}
// Regression coverage for parameter properties.
// A typed private setter should provide the return type for its paired
// public/protected getter when the getter has no explicit return type.
export declare class AccessorPairWithPrivateSetter {
  get isRunning(): boolean;
  private set isRunning(value);
}
export declare class AccessorPairWithProtectedPrivateSetter {
  protected get value(): number;
  private set value(value);
}
// Private parameter properties on a private constructor need no explicit type:
// the constructor signature is collapsed and the class member is type-erased.
export declare class PrivateConstructorWithPrivateParamPropertyInferredDefault {
  private readonly a;
  private b;
  private constructor();
}
// Non-private parameter properties on a private constructor still need a type,
// because the class member is still emitted.
export declare class PrivateConstructorWithPublicParamPropertyInferredDefault {
  readonly a;
  private constructor();
}
// The suppression must be limited to `private` constructors — a `protected`
// constructor keeps its signature, so a private parameter property with an
// uninferrable default still requires an explicit type.
export declare class ProtectedConstructorWithPrivateParamPropertyInferredDefault {
  private readonly a;
  protected constructor(a?);
}
export declare class NullBase extends null {
}
