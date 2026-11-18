class C {}
namespace C {
  export namespace N {
    export interface T {}
  }
}
const B = 0;
namespace Use {
  import A = C.N;
  import B = A.T;
  type Typed = B;
  const value = B;
}
