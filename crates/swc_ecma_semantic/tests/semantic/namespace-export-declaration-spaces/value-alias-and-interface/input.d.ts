declare namespace A {
  const T: number;
}
declare namespace N {
  export import X = A.T;
  interface X {}
}
