declare namespace A {
  namespace T {
    interface U {}
  }
  export type { T as X };
}
declare namespace B {
  export import Y = A.X.U;
}
