const key = "other";
class C {
  static [key] = 1;
}
namespace C {
  export interface x {}
}
namespace A {
  export interface Inner {}
}
namespace Use {
  import A = C.x;
  type Result = A.Inner;
}
