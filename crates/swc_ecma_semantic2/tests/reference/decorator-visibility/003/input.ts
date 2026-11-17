declare function dec<U>(value?: unknown): any;
type T = string;
class C<T> {
  @dec<T>() m<T>() {}
}
