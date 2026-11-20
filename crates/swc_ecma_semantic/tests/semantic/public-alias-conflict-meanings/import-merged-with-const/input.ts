export {};
declare namespace N {
  export const X: 1;
  interface X {}
  export { X as Y };
}
import Alias = N.Y;
declare const Alias: string;
