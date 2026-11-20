export {};
namespace N {
  export namespace N1 {
    export declare namespace Inner { export { X as Y }; }
  }
  export declare namespace N1 { export { X }; }
}
declare namespace N {
  export interface X { right: true }
  const original: 1;
  export { original as X };
}
import Alias = N.N1.Inner.Y;
const value: 1 = Alias;
const typed: Alias = { right: true };
const queried: typeof Alias = 1;
