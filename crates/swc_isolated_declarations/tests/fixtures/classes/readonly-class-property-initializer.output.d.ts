export declare class ReadonlyLiteralInitializers {
  readonly directTrue = true;
  readonly parenTrue = true;
  readonly constAssertTrue: true;
  readonly nestedConstAssert: true;
  readonly directString = "x";
  readonly parenString = "x";
  readonly templateNoExpr = "x";
  readonly constAssertString: "x";
  readonly unaryNumber = -1;
  readonly unaryParenNumber = -1;
  readonly constAssertNumber: 1;
  readonly directObject: {
    a: number;
  };
  readonly parenObject: {
    a: number;
  };
  readonly constAssertObject: {
    readonly a: 1;
  };
  readonly nestedConstAssertObject: {
    readonly a: 1;
  };
  readonly directNestedObject: {
    nested: {
      a: number;
    };
  };
  readonly constAssertNestedObject: {
    readonly nested: {
      readonly a: 1;
    };
  };
  writableObject: {
    a: number;
  };
  writableTrue: boolean;
}
export declare class ReadonlyArrayInitializers {
  readonly emptyArray: unknown;
  readonly arrayAsConst: readonly [];
  readonly arrayWithTypeAnnotation: number[];
  readonly nonEmptyArray: unknown;
  readonly arrayWithSpread: unknown;
  readonly arrayIdentifierReference: unknown;
  readonly arrayIdentifierReferenceAndAnnotation: [readonly [1, 2]];
}
