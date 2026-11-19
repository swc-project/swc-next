export {};
declare namespace N {
  export interface X {
    right: true;
  }
  const original: 1;
  export { original as X };
}
declare namespace N {
  namespace Inner {
    export { X as Y };
  }
}
import Alias = N.Inner.Y;
const value: 1 = Alias;
const typed: Alias = { right: true };
export const queried: typeof Alias = 1;
