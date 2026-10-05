type Ordinary = | /* inline */ { key: string };
type OwnLine = &
  /* head */
  "value";
type Mixed = /* previous */ &
  // line
  "value";
type LineAndBlock =
  // line
  & /* block */
  "value";
type LiteralBlock = | /* opening
   raw second line */ { value: string };
type RetainedTail = | (number /* tail */);
type PairHead = | (
  /* prettier-ignore */
  12.00
);
type Whole = /* prettier-ignore */ | 13.00;
type First = | /* prettier-ignore */ 14.00;
type Grammar = { item: (| number)[]; call: & (() => string) };
type Value = | number;
// @ts-expect-error -- incompatible value remains active
const invalid: Value = "wrong";
type LongAliasName = | (string | number | boolean);
