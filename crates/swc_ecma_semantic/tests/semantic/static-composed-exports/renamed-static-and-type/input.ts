export {};
class C {
  static x = 1;
}
declare namespace C {
  export namespace Y {
    interface T {
      t: true;
    }
  }
  interface Original {
    own: true;
  }
  export { Original as Y };
  export { Y as x };
}
import Alias = C.x;
const value: number = Alias;
const typed: Alias = { own: true };
const nested: Alias.T = { t: true };
