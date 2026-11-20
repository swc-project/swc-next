declare function dec<U>(value?: unknown): any;
type T = string;
class C<T> {
  constructor(@dec<T>() value: T) {}
}
