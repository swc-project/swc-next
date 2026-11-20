export {};
declare namespace N {
  export const X: 1;
  interface X {}
  export namespace Y {}
  export { X as Y };
  export namespace Inner {
    export interface Z {}
    export { Y as Z };
  }
}
