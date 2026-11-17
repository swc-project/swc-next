namespace A {
  export const x = 1;
}
namespace A.B {
  export const y = x;
}
namespace A.B.C {
  export const z = x + y;
}
