class Box<T> {
  constructor(public T: T) {
    const value = T;
  }
}
interface Box<T> {
  extra: T;
}
