type Tail<T extends unknown[]> = T extends [
  unknown,
  ...
  // prettier-ignore
  ( infer    R )
] ? R : never;

type Tail<T extends unknown[]> = T extends [
  unknown,
  // prettier-ignore
  ...  ( infer    R )
] ? R : never;

type Tail<T extends unknown[]> = T extends [
  unknown,
  ...
  // prettier-ignore inactive
  ( infer    R )
] ? R : never;
