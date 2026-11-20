declare function dec<U>(value?: unknown): any;
type T = string;
const C =
  @dec<T>()
  class Inner<T> {};
