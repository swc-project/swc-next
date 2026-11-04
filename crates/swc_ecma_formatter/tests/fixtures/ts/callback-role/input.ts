export const observed: unknown[] = [];

function consume(callback: (first: { amount: number }, second: number) => unknown): void {
  observed.push(callback({ amount: 3 }, 5));
}

function labeled(label: string, callback: (first: { amount: number }, second: number) => unknown): void {
  observed.push(label);
  consume(callback);
}

function first(callback: (first: { amount: number }, second: number) => unknown, enabled: boolean): void {
  if (enabled) consume(callback);
}

consume(function defaults(firstValue = { amount: 1 }, secondValue = 2, thirdValue = 3) { return firstValue.amount + secondValue + thirdValue; });

consume(function destructured({ amount }, secondValue = 2) { return amount + secondValue; });

consume(function typed(firstValue: { amount: number }, secondValue: number) { return firstValue.amount + secondValue; });

consume(function identifiers(firstLongIdentifier, secondLongIdentifier) { return firstLongIdentifier.amount + secondLongIdentifier; });

labeled("label", function final(firstValue = { amount: 1 }, secondValue = 2) { return firstValue.amount + secondValue; });

first(function initial(firstValue = { amount: 1 }, secondValue = 2) { return firstValue.amount + secondValue; }, true);

consume(
  // prettier-ignore
  function ignored(firstValue = { amount : 1 }, secondValue=2) { return firstValue.amount+secondValue; }
);

consume(function parameterOwner(
  // prettier-ignore
  firstValue: { amount : number },
  secondValue: number
) { return firstValue.amount + secondValue; });

consume(function active(firstValue: { amount: number }, secondValue: number) {
  // @ts-expect-error
  const rejected: string = secondValue;
  return [firstValue.amount, rejected];
});
