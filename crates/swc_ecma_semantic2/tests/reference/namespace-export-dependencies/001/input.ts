export {};
namespace N {
  export declare namespace Inner {
    export { X as Y };
  }
}
declare namespace N {
  interface T {
    right: true;
  }
  export { T as X };
}
type Result = N.Inner.Y;
const result: Result = { right: true };
