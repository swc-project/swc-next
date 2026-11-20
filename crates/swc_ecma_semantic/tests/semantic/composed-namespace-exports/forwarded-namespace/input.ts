export {};
declare namespace N {
  export interface X {
    right: true;
  }
  namespace Original {
    const v: 1;
    interface T {
      ok: true;
    }
  }
  export { Original as X };
}
namespace N {
  export declare namespace Inner {
    export { X as Y };
  }
}
import Alias = N.Inner.Y;
const value: 1 = Alias.v;
const typed: Alias = { right: true };
const nested: Alias.T = { ok: true };
