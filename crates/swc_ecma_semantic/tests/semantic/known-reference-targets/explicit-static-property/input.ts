const key = "other";
class C {
  static x = 1;
  static [key] = 2;
}
namespace C {
  export interface x {}
}
const A = 0;
namespace Use {
  import A = C.x;
  const result = A;
}
