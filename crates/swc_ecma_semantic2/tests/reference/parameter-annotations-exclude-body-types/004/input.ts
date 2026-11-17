enum T {
  A,
}
class C {
  m(...args: T[]) {
    enum T {
      B,
    }
  }
}
