const F = {
  string: `string`,
  templateLiteral: `templateLiteral`,
  number: 1.23,
  bigint: -1_2_3n,
  boolean: true,
  null: null,
  undefined: undefined,
  function(a: string): void {},
  arrow: (a: string): void => {},
  object: {
    a: `a`,
    b: `b`
  },
  array: [`a`, , { b: `\n` }],
} as const

const positive = +1;
const signedObject = { positive: +1, negative: -1 } as const;
const signedTuple = [+1, -1] as const;
class SignedFields {
  readonly positive = +1;
  readonly negative = -1;
}
