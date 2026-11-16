// @ts-expect-error
const fn = (foo: boolean) => foo ? missingA : missingB;
const outsideSentinel: number = "x";
