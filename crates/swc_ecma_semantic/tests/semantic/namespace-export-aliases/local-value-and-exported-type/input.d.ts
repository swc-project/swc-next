declare namespace N {
  export interface X {}
  const X: number;
  export { X as Y };
}
import A = N.Y;
