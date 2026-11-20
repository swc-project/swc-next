export {};
const Inner = 1;
namespace Outer {
  const { fn }: { fn: <U>(x: U) => U } = { fn: (x) => x };
  interface T {}
  export declare namespace Inner {
    export { T };
  }
  export const selected = Inner;
}
