class C {
  static [(("x"))]() {}
}
namespace C {
  export interface x {}
}
const A = 0;
namespace Use {
  import A = C.x;
  const result = A;
}
