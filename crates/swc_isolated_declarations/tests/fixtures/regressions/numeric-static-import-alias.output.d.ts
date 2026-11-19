declare class C {
  static 1e400: number;
}
declare namespace C {
  interface Infinity {
    own: true;
  }
}
import Alias = C.Infinity;
export declare const queried: typeof Alias;
export {};
