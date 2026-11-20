namespace A {
  export interface T {}
}
namespace N {
  import X = A.T;
}
namespace N {
  export interface X {}
}
