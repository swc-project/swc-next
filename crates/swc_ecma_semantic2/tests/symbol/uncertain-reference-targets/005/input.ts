const key = "other";
class C {
  static [key] = 1;
}
namespace C {
  export interface x {}
}
const A = 0;
namespace Outer {
  const A = 1;
  namespace Inner {
    import A = C.x;
    const result = A;
  }
}
