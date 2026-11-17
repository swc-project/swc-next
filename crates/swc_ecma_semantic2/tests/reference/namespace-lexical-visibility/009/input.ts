namespace A {
  export const x = 2;
}
namespace A.B {}
namespace A {
  const x = 1;
  namespace B {
    export const y = x;
  }
}
