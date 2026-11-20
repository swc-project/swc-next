const key = "x";
class C {
  static [key] = 1;
}
namespace C {
  export interface x {}
}
const A = 0;
namespace Use {
  import A = C.x;
  const result = A;
}
