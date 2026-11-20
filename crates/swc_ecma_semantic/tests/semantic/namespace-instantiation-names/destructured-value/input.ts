export {};
const Inner = 1;
namespace Outer {
  const { T } = { T: 0 };
  interface T {}
  export declare namespace Inner {
    export { T };
  }
  export const selected = Inner;
}
