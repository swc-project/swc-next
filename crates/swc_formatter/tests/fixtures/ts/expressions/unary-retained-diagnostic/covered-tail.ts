function evaluate(first: boolean): boolean {
  return !(first &&
    // @ts-expect-error
    missingTarget /* tail */);
}
