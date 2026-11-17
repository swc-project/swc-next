type U = number;
interface Box<T> {
  value: T;
  outer: U;
}
interface Box<T, U = string> {
  extra: T;
  inner: U;
}
