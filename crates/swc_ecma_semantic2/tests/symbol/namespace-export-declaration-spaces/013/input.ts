declare namespace N {
  export const X: number;
  interface X {}
  export { X as Y };
}
import Alias = N.Y;
interface Alias {}
