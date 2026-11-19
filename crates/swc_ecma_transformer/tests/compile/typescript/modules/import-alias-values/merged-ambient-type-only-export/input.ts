export {};
declare namespace N {
  export namespace M {
    const x: number;
  }
  type T = string;
  export type { T as M };
  export { M as Y };
}
import A = N.Y.x;
const value: number = A;
