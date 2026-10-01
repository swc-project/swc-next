declare const F: {
  readonly string: "string";
  readonly templateLiteral: "templateLiteral";
  readonly number: 1.23;
  readonly bigint: -123n;
  readonly boolean: true;
  readonly null: null;
  readonly undefined: undefined;
  readonly function: (a: string) => void;
  readonly arrow: (a: string) => void;
  readonly object: {
    readonly a: "a";
    readonly b: "b";
  };
  readonly array: readonly ["a", undefined, {
    readonly b: "\n";
  }];
};
declare const positive = 1;
declare const signedObject: {
  readonly positive: 1;
  readonly negative: -1;
};
declare const signedTuple: readonly [1, -1];
declare class SignedFields {
  readonly positive = 1;
  readonly negative = -1;
}
