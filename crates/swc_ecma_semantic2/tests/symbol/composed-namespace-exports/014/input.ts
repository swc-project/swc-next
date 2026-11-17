export {};
declare namespace Values {
  namespace Original {
    const v: 1;
    interface T {
      ok: true;
    }
  }
}
declare namespace N {
  export interface X {
    right: true;
  }
  import original = Values.Original;
  export { original as X };
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
