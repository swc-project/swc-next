type Tail<T extends unknown[]> = T extends [
  unknown,
  ...
  // @ts-expect-error
  ( infer R )
] ? R : never;

type Tail<T extends unknown[]> = T extends [
  unknown,
  ...
  // @ts-ignore
  ( infer R )
] ? R : never;
