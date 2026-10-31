type Tail<T extends unknown[]> = T extends [unknown, .../* rest */ (/* infer */ infer R /* result */)] ? R : never;

type ArrayTail<T extends unknown[]> = T extends [...(infer R /* element */)[]] ? R : never;

type Tail<T extends unknown[]> = T extends [
  unknown,
  ... // rest
  infer R
] ? R : never;
