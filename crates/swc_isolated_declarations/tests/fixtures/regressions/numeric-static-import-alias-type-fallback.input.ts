type Alias = number;

declare class C {
  static 1e400: "value";
}
declare namespace C {
  export interface Marker {}
}

export namespace Use {
  import Alias = C.\u0049nfinity;
  export const value: typeof Alias = Alias;
  export const typed: Alias = 1;
}
