export {};
declare namespace N {
  export const X: 1;
  interface X {}
  export namespace Y {}
  export { X as Y };
  export namespace Inner {
    export interface Z {
      own: true;
    }
    export { Y as Z };
  }
}
import Alias = N.Inner.Z;
const value: 1 = Alias;
const typed: Alias = { own: true };
