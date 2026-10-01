// Correct
export declare function fnDeclGood(p?: T, rParam?: string): void;
export declare function fnDeclGood2(p?: T, rParam?: number): void;
export declare function fooGood([a, b]?: any[]): number;
export declare const fooGood2: ({ a, b }?: object) => number;
export declare function fooGood3({ a, b: [{ c }] }: object): void;
// Incorrect
export declare function fnDeclBad<T>(p: T | undefined, rParam: T | undefined, r2: T): void;
export declare function fnDeclBad2<T>(p: T | undefined, r2: T): void;
export declare function fnDeclBad3<T>(p: T | undefined, r2: T, rParam?: T): void;
export declare function fooBad(): number;
export declare const fooBad2: () => number;
export declare function withAny(a: any | undefined, b: string): void;
export declare function withAnyUnion(a: (any | string) | undefined, b: string): void;
export declare function withAnyArrayUnion(a: (any | number[]) | undefined, b: string): void;
export declare function withAnyUndefinedUnion(a: any | undefined, b: string): void;
type Foo = {};
export declare function withAnyTypeReferenceUnion(a: (any | Foo) | undefined, b: string): void;
export declare function withAnyTypeReferenceUndefinedUnion(a: any | Foo | undefined, b: string): void;
export declare function withAnyTypeOperatorUnion(a: (any | keyof {}) | undefined, b: string): void;
export declare function withAnyIntersectionTypeReferenceUnion(a: (any | (Foo & {})) | undefined, b: string): void;
export declare function withUnknown(a: unknown | undefined, b: string): void;
export declare function withUnknownUnion(a: (unknown | string) | undefined, b: string): void;
export declare function withTypeAssertion(a?: any): void;
export declare function restWithDefault(...[value, options]: [string, object]): void;
export declare function restWithNestedDefault(...[a, { b, c }]: [number, {
  b: number;
  c: number;
}]): void;
export {};
