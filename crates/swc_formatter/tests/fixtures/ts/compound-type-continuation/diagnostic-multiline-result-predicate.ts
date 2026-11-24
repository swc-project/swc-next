class C {
  f(x: unknown):
  // @ts-expect-error
  x is A
  & (B
  // tail
  ) { return true; }
}
