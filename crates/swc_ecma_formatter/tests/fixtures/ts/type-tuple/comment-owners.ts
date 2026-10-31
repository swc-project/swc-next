type Empty=[];
type Single=[/* ordinary block */];
type Packet=[
  /* first block */

  /* second block */

  // first line

  // second line
];
type Body=[/* keep

literal body
*/];
type Ignored=[/* prettier-ignore */ /* first pinned */

  /* second pinned */];
type Protected=[
  // @ts-expect-error

  // directive covers the closing line
] & MissingEmptyTupleType;
type InlineFirst=[/* prettier-ignore */12.00|13.00,number];
type InlineLater=[number,/* prettier-ignore */12.00|13.00,string];
type Adjacent = [
  /**
   * first
   *//**
   * second
   */
];
