namespace Outer {
  export class C {}
  namespace C {
    export interface I {}
  }
  type A = string;
  const A = 1;
  namespace Test {
    import A = C;
    export let text: A;
    export const value = A;
  }
}
