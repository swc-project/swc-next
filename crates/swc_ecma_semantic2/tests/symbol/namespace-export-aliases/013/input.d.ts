declare namespace N {
  export const X: number;
  interface X {}
  export namespace Inner {
    export { X as Y };
  }
}
import A = N.Inner.Y;
