declare namespace Outer {
  namespace A {
    const X: number;
    namespace B {
      export { Sibling };
    }
    const bad: typeof B;
  }
  namespace Sibling {
    export { X };
  }
  interface X {}
}
