declare function dec<U>(): any;
class C<T> {
  m<T>(@dec<T>() value: T) {}
}
