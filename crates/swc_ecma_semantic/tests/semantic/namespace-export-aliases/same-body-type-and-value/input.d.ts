declare namespace N {
  export const X: number;
  interface X {}
  export { X as Y };
}
import A = N.Y;
