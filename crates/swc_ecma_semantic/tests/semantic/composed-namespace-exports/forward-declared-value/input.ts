export {};
namespace N {
  export declare namespace Inner {
    export { X as Y };
  }
}
declare namespace N {
  export interface X {
    right: true;
  }
  const original: 1;
  export { original as X };
}
import Alias = N.Inner.Y;
const value: 1 = Alias;
const typed: Alias = { right: true };
const queried: typeof Alias = 1;
