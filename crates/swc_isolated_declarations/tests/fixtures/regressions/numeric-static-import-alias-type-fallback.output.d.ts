type Alias = number;
declare class C {
  static 1e400: "value";
}
declare namespace C {
  interface Marker {  }
}
export declare namespace Use {
  import Alias = C.Infinity;
  export const value: typeof Alias;
  export const typed: Alias;
  export {};
}
export {};
