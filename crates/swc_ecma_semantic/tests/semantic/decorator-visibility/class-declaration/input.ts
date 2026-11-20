declare function dec<U>(value?: unknown): any;
type T = string;
@dec<T>()
class C<T> {}
