declare namespace N {
  export const X: number;
}
declare namespace N {
  interface X {}
  export { X as Y };
}
import A = N.Y;
