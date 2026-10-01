interface Source {
  value: string;
}
export declare function mapped(value: {
  [K in keyof Source]: Source[K];
} | undefined, required: string): void;
export declare function keyOf(value: keyof Source | undefined, required: string): void;
export declare function readonlyTuple(value: readonly [string] | undefined, required: string): void;
export declare function intersection(value: {
  optional?: string;
} & number[] | undefined, required: string): void;
export declare function operatorIntersection(value: (keyof Source) & string | undefined, required: string): void;
export declare function unknownOperatorIntersection(value: unknown & keyof Source | undefined, required: string): void;
export declare function unresolvedIntersection<T>(value: T & string | undefined, required: string): void;
export declare function allUnknownIntersection(value: unknown & unknown | undefined, required: string): void;
export declare function unresolvedUndefinedUnionIntersection<T>(value: (T | undefined) & string | undefined, required: string): void;
export declare function unknownUndefinedUnionOperatorIntersection(value: (unknown | undefined) & keyof Source | undefined, required: string): void;
export declare function anyUndefinedUnionOperatorIntersection(value: (any | undefined) & keyof Source | undefined, required: string): void;
export declare function unresolvedNeverOperatorIntersection<T>(value: T & never & keyof Source | undefined, required: string): void;
export declare function voidUnionUndefinedIntersection(value: (void | string) & undefined, required: string): void;
export {};
