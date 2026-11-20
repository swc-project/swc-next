declare namespace A {
  interface T {}
}
declare namespace N {
  export import X = A.T;
  interface X {}
}
