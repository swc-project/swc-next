export {};
declare namespace N {
  export const X: 1;
  interface Original {
    right: true;
  }
  export { Original as X };
}
namespace N {
  export declare namespace Inner {
    export { X as Y };
  }
}
import Alias = N.Inner.Y;
const value: 1 = Alias;
const typed: Alias = { right: true };
const queried: typeof Alias = 1;
