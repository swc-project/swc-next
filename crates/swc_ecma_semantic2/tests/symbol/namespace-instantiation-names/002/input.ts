export {};
const Inner = 1;
namespace Outer {
  const { fn }: { fn: <T>(x: T) => T } = { fn: (x) => x };
  interface T {}
  export declare namespace Inner {
    export { T };
  }
  export const selected = Inner;
}
