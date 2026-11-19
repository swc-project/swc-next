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
export declare const queried: typeof Alias;
