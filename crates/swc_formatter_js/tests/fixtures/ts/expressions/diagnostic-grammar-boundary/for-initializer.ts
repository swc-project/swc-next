export function evaluate(first: string): void {
  for ((first in
    // @ts-expect-error
    missingRhs /* tail */); missingCondition; missingUpdate) {}
  const outsideSentinel: number = "outside";
}
