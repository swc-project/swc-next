export function evaluate(first: boolean): boolean {
  const value = ((first ? first :
    // @ts-expect-error
    missingRhs /* tail */)) ? missingConsequent : missingAlternate;
  const outsideSentinel: number = "outside";
  return value;
}
