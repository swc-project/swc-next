export {};
namespace Item {
  export interface T {
    outer: true;
  }
}
declare namespace N {
  namespace Local {
    export interface T {
      inner: true;
    }
  }
  export { Local as Item };
  export interface Item {
    direct: true;
  }
}
declare namespace N {
  export import A = Item;
  const value: A.T;
  const direct: A;
}
