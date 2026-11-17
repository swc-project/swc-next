const key = "other";
class C {
  static [(key)]() {}
}
namespace C {
  export interface x {}
}
const A = 0;
namespace Use {
  import A = C.x;
  const result = A;
}
