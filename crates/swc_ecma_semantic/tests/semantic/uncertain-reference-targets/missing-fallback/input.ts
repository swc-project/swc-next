const key = "x";
class C {
  static [key] = 1;
}
namespace C {
  export interface x {}
}
namespace Use {
  import A = C.x;
  const result = A;
}
