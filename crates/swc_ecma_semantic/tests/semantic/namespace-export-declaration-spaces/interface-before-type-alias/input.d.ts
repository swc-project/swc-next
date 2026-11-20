declare namespace A {
  interface T {}
}
declare namespace N {
  interface X {}
  export import X = A.T;
}
