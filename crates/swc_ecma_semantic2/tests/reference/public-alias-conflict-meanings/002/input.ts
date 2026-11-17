export {};
declare namespace N {
  export const X: 1;
  interface X {}
  export namespace Y {}
  export { X as Y };
}
import Alias = N.Y;
interface Alias {
  own: true;
}
