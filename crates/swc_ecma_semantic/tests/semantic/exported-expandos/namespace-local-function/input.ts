namespace N {
  interface F {}
  export function F() {}
  export namespace F {
    export interface Marker {}
  }
  F.x = 1;
  type Alias = number;
  namespace Use {
    import Alias = F.x;
    const value = Alias;
    const typed: Alias = 0;
  }
}
