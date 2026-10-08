export function evaluate(first: boolean, consume: (value: boolean, extra: unknown) => boolean): boolean {
  const value = !(first &&
    // @ts-expect-error
    missingRhs /* tail */) ? missingConsequent : missingAlternate;
  const outsideSentinel: number = "outside";
  return value;
}
