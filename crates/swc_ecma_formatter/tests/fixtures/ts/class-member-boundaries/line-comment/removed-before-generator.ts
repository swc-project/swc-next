class C {
  a = 1
  // @ts-expect-error
  ;*gen() {}
}
