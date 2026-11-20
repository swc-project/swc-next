export {};
declare namespace N {
  export interface X {
    right: true;
  }
  const original: 1;
  export type { original as X };
}
namespace N {
  export declare namespace Inner {
    export { X as Y };
  }
}
import Alias = N.Inner.Y;
const typed: Alias = { right: true };
const queried: typeof Alias = 1;
