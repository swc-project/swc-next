declare function f<F>(): void;
type C = typeof f<(
  // prettier-ignore
  <T>( v : T ) => T
)>;
