declare function dec<U>(value?: unknown): any;
type T = string;
class C<T> {
  m<T>(@dec<T>() value: T) {}
}
