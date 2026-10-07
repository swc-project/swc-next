declare function f<F>(): void;
type C = typeof f<
  // @ts-expect-error
  <T>(v: Missing) => T
>;
