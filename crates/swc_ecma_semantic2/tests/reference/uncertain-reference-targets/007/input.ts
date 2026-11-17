const key = "other";
class C {
  static [key] = 1;
}
namespace C {
  export interface x {}
}
const A = 0;
namespace Use {
  import A = C.x;
  type Result = typeof A;
}
