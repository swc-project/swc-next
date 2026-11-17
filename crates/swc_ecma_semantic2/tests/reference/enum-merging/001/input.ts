enum Outer {
  A = (() => {
    enum Inner {
      X,
    }
    return 0;
  })(),
  B = 1,
  C = B,
}
