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
import Alias = N.X;
const value: 1 = Alias.v;
const typed: Alias = { right: true };
const nested: Alias.T = { ok: true };
