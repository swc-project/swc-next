export {};
declare namespace N {
  interface T {
    right: true;
  }
  export { T as X };
}
namespace N {
  export declare namespace Inner {
    export { X as Y };
  }
}
type Result = N.Inner.Y;
const result: Result = { right: true };
